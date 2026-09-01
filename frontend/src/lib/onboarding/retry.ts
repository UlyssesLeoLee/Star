// =====================================================================
// Onboarding retry — 5 重试 + 3-6-12-24-48s 指数 backoff (per ADR-0042 §1.3)
// =====================================================================
// 5 次重试, 第 6 次仍失败 → 写入 audit log + 返 ProviderErrorCode 让 UI 弹错误卡片
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖 (无 retry lib, 手写 Promise + setTimeout)
//   - 4xx/5xx 分类到 ProviderErrorCode (per types/onboarding.ts)
//   - 测试请求 timeout 10s (per fetch AbortController)
//   - 不打印明文 key (per 守门 #5)
// =====================================================================

import type {
  DetectedKey,
  TestResult,
  ErrorResolution,
  ProviderErrorCode,
} from "@/types/onboarding";
import {
  RETRY_BACKOFF_MS,
  MAX_RETRY_ATTEMPTS,
  PROVIDER_TEST_ENDPOINTS,
} from "@/types/onboarding";

// ---- 错误码 → 解决步骤 (per 拍板 retryreport_opt3) ----
export const ERROR_RESOLUTIONS: Record<ProviderErrorCode, ErrorResolution> = {
  unauthorized: {
    code: "unauthorized",
    steps: [
      "检查 key 是否有效, 或重新生成 (provider console → API keys)",
      "确认 key 没有多余空格 / 换行 (粘贴后常常出现)",
    ],
    doc_url: "https://platform.openai.com/account/api-keys",
    curl_test: 'curl -H "Authorization: Bearer $KEY" https://api.openai.com/v1/models',
  },
  forbidden: {
    code: "forbidden",
    steps: [
      "可能需要开通模型访问权限 (provider console → Models)",
      "检查账户余额 / 计费方式",
    ],
    doc_url: "https://console.anthropic.com/settings/billing",
    curl_test: 'curl -H "x-api-key: $KEY" https://api.anthropic.com/v1/messages',
  },
  rate_limited: {
    code: "rate_limited",
    steps: [
      "等待 1 分钟后重试, 或换备用 key",
      "考虑升级 provider 套餐 (rate limit tier)",
    ],
    doc_url: "https://platform.openai.com/docs/guides/rate-limits",
    curl_test: "(无需 curl, 等待 1 分钟即可)",
  },
  network_timeout: {
    code: "network_timeout",
    steps: [
      "检查网络 / VPN / 防火墙",
      "如果是 corporate network, 联系 IT 部门放行 provider API 域名",
    ],
    doc_url: "(无 — 内部网络问题)",
    curl_test: "curl -v https://api.openai.com/v1/models  # 测试 TLS handshake",
  },
  model_unavailable: {
    code: "model_unavailable",
    steps: [
      "检查 provider 状态页 (status.openai.com / status.anthropic.com)",
      "切换到其它模型 (e.g. gpt-4o → gpt-3.5-turbo)",
    ],
    doc_url: "https://status.openai.com",
    curl_test: "curl -s https://status.openai.com/api/v2/status.json | jq .status.description",
  },
  unknown: {
    code: "unknown",
    steps: [
      "重试一次 (偶发网络抖动)",
      "如果持续失败, 提交 issue 附 status_code",
    ],
    doc_url: "(无 — 未知错误)",
    curl_test: "(见 provider console debug 页)",
  },
};

// ---- 单次测试 attempt ----
async function testKeyOnce(
  key: DetectedKey,
  signal: AbortSignal,
): Promise<{ ok: boolean; status: number; errorCode: ProviderErrorCode; errorMessage: string }> {
  // 找 provider 端点配置
  const ep = PROVIDER_TEST_ENDPOINTS.find((e) => e.provider === key.provider);
  if (!ep) {
    return { ok: false, status: 0, errorCode: "unknown", errorMessage: `unknown provider: ${key.provider}` };
  }
  // preview 是 masked (sk-***xyz1), 仅知道存在不能复用 — Phase 1 mock 走随机
  // 守门 #5: 不从 preview 还原明文
  if (key.preview.startsWith("env:")) {
    // env 模式: 由后端 process.env 拿, 浏览器端无 key 直接返 success (跳过)
    return { ok: true, status: 200, errorCode: "unknown", errorMessage: "env mode: skip client test" };
  }
  // Phase 1 mock: 随机 90% 返 success, 10% 返 401 (测试重试 + 上报流程)
  // Phase 2 真接: 走 fetch + ep.build_headers
  await new Promise((r) => setTimeout(r, 50 + Math.random() * 200));  // 模拟 latency
  if (Math.random() < 0.1) {
    return { ok: false, status: 401, errorCode: "unauthorized", errorMessage: "401 Unauthorized (mock): Invalid API key" };
  }
  return { ok: true, status: 200, errorCode: "unknown", errorMessage: "" };
  // 实施 (Phase 2 真接):
  // const actualKey = ep.extract_key(key);
  // if (!actualKey) return { ok: false, status: 0, errorCode: "unknown", errorMessage: "extract failed" };
  // try {
  //   const res = await fetch(ep.test_url, { method: "GET", headers: ep.build_headers(actualKey), signal });
  //   if (res.ok) return { ok: true, status: res.status, errorCode: "unknown", errorMessage: "" };
  //   return { ok: false, status: res.status, errorCode: classifyStatus(res.status), errorMessage: res.statusText };
  // } catch (e) {
  //   if ((e as Error).name === "AbortError") {
  //     return { ok: false, status: 0, errorCode: "network_timeout", errorMessage: "timeout 10s" };
  //   }
  //   return { ok: false, status: 0, errorCode: "network_timeout", errorMessage: String(e) };
  // }
}

/** 4xx/5xx → ProviderErrorCode
 * status = 0 表示网络错误 (timeout / DNS / TLS 等) → network_timeout
 */
export function classifyStatus(status: number): ProviderErrorCode {
  if (status === 0) return "network_timeout";
  if (status === 401) return "unauthorized";
  if (status === 403) return "forbidden";
  if (status === 429) return "rate_limited";
  if (status === 404 || status === 503) return "model_unavailable";
  return "unknown";
}

// =====================================================================
// 主入口: 测试一个 key, 5 重试 + backoff, 返最终 TestResult
// =====================================================================
// 返回的 TestResult 含:
//   - attempt = 最后一次 attempt 编号
//   - status = "success" | "failed"
//   - audit_event_id = 失败时 audit log ID (Phase 2 真接)
// =====================================================================
export async function testKeyWithRetry(
  detectedKey: DetectedKey,
  onProgress?: (result: TestResult) => void,
): Promise<TestResult> {
  const startedAt = new Date().toISOString();
  for (let attempt = 0; attempt < MAX_RETRY_ATTEMPTS; attempt++) {
    // 进度回调
    const partial: TestResult = {
      detected_key_id: detectedKey.id,
      status: "running",
      attempt,
      max_attempts: MAX_RETRY_ATTEMPTS,
      next_retry_in_ms: attempt < MAX_RETRY_ATTEMPTS - 1 ? RETRY_BACKOFF_MS[attempt] : undefined,
    };
    onProgress?.(partial);

    // 单次测试 (10s timeout, AbortController)
    const ac = new AbortController();
    const timer = setTimeout(() => ac.abort(), 10_000);
    const result = await testKeyOnce(detectedKey, ac.signal);
    clearTimeout(timer);

    if (result.ok) {
      return {
        detected_key_id: detectedKey.id,
        status: "success",
        attempt,
        max_attempts: MAX_RETRY_ATTEMPTS,
        started_at: startedAt,
      };
    }

    // 失败: 进度 + (最后一次) 立即返 failed
    const failed: TestResult = {
      detected_key_id: detectedKey.id,
      status: "failed",
      attempt,
      max_attempts: MAX_RETRY_ATTEMPTS,
      status_code: result.status,
      error_message: result.errorMessage,
      started_at: startedAt,
    };
    onProgress?.(failed);

    // 5 重试耗尽, 写 audit log (mock: console + localStorage)
    if (attempt === MAX_RETRY_ATTEMPTS - 1) {
      const auditEventId = writeAuditLog(detectedKey, failed);
      return { ...failed, audit_event_id: auditEventId };
    }

    // 等 backoff 再试下一次
    await new Promise((r) => setTimeout(r, RETRY_BACKOFF_MS[attempt]));
  }
  // 不可达
  throw new Error("retry loop unreachable");
}

// ---- audit log (Phase 1 mock, Phase 2 真接 audit_audit_event 表) ----
function writeAuditLog(key: DetectedKey, result: TestResult): string {
  if (typeof window === "undefined") return "audit-ssr";
  const id = `audit-${Date.now()}-${key.provider}`;
  const entry = {
    id,
    action: "onboarding.test_key.failed",
    detected_key_id: key.id,
    provider: key.provider,
    label: key.label,
    attempts: result.attempt + 1,
    status_code: result.status_code,
    error_message: result.error_message,
    timestamp: new Date().toISOString(),
  };
  try {
    const existing = JSON.parse(window.localStorage.getItem("star:onboarding-audit") || "[]");
    existing.push(entry);
    window.localStorage.setItem("star:onboarding-audit", JSON.stringify(existing));
  } catch {
    // 静默失败, 不影响主流程
  }
  return id;
}

/** 读 audit log 列表 (供 /settings/onboarding-audit 页用, Phase 2) */
export function readAuditLog(): Array<Record<string, unknown>> {
  if (typeof window === "undefined") return [];
  try {
    return JSON.parse(window.localStorage.getItem("star:onboarding-audit") || "[]");
  } catch {
    return [];
  }
}
