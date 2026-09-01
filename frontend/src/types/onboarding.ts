// =====================================================================
// Star Platform — Onboarding (First-Run) Type Definitions
// =====================================================================
// 来源: docs/architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md v0.1
//       + 2026-09-02 08:01 JST Ulysses 4 拍板 (trigger_opt1/scope_opt4/retry_opt3/storage_opt1)
//
// 用途: 首次启动 onboarding 引导 — 扫 localStorage / env-var-hint / IDE-residual 3 探测器
//       → 列出 DetectedKey → 用户选 agent 关联 → 5 重试测试 → 失败上报
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖, 复用 React Query + Lucide + clsx
//   - 字段对应后端契约, 必加 audit_log 字段 (per #9 审计必带)
//   - provider 4 必备 (openai/claude/gemini/minimax) + 兼容 4
// =====================================================================

import type { Uuid, Iso8601 } from "./ids";

// ---- 1. 探测器 (3 路径) ----
export type DetectorSource =
  | "localStorage"        // 现有 /settings/api-keys 存 (star:api-keys)
  | "env_var_hint"        // process.env.NEXT_PUBLIC_*_API_KEY_HINT (只读存在性)
  | "ide_residual";       // fetch('/.vscode/settings.json') 等 5 路径

export const DETECTOR_SOURCES: readonly DetectorSource[] = [
  "localStorage",
  "env_var_hint",
  "ide_residual",
] as const;

// ---- 2. 探测到的 key (per 探测器) ----
export interface DetectedKey {
  /** 唯一 ID (per 探测器, scan-time 临时) */
  id: string;
  /** 4 必备 (openai/claude/gemini/minimax) + 兼容 4 */
  provider: "openai" | "claude" | "gemini" | "minimax" | "anthropic" | "openclaw" | "hermes" | "google";
  /** 用户视角的 label (e.g. "Primary" / "Backup") */
  label: string;
  /** 探测到的值 (e.g. "sk-***xyz1") — **永远不存明文, 仅 preview** */
  preview: string;
  /** 来源 (3 探测器) */
  source: DetectorSource;
  /** 探测时间 (ISO 8601) */
  detected_at: Iso8601;
  /** 用于用户选择的 source context (e.g. "从 localStorage star:api-keys 找到") */
  source_label: string;
  /** 用户可填的 env var 名 (e.g. "OPENAI_API_KEY"), 供"仅提示"模式 */
  env_var_name?: string;
}

// ---- 3. 关联结果 (per DetectedKey + 选中的 agent) ----
export interface AssociationRequest {
  detected_key_id: string;
  /** 关联到哪个 agent (per CliTab.id) — 用户选 */
  agent_id: string;
  /** 关联到哪个 CLI profile (per profileName) — 可选 */
  cli_profile_id?: string;
  /** 关联到哪个 agent_kind — 可选 (从 profileName 推断) */
  agent_kind?: "claude-sonnet" | "gpt-4o" | "codex" | "internal-vibe-coder" | "gemini-2" | "minimax-v1";
  /** 存储模式 (per 2026-09-02 02:49 JST 拍板 storage_opt1: encrypted_rust) */
  mode: "encrypted_rust";
  /** tenant_id (13 類必带, per REQ-SEC-001) */
  tenant_id: Uuid;
}

// ---- 4. 测试结果 (per retry 5 次) ----
export interface TestResult {
  detected_key_id: string;
  status: "success" | "running" | "failed" | "skipped";
  /** 当前 attempt (0-5) */
  attempt: number;
  /** 最多 attempt 数 (5+ per retry_opt3) */
  max_attempts: number;
  /** 当前 attempt 失败的 status code (4xx/5xx), 0 表示网络错误 */
  status_code?: number;
  /** 错误信息 (e.g. "rate limited", "unauthorized") */
  error_message?: string;
  /** 下次 retry 等待时间 (ms) */
  next_retry_in_ms?: number;
  /** 5 retry 起始时间 (ISO 8601) */
  started_at?: Iso8601;
  /** audit log entry ID (per 守门 #9) */
  audit_event_id?: Uuid;
}

// ---- 5. Onboarding 状态机 (4 阶段) ----
export type OnboardingStage =
  | "idle"           // 初始, 未启动
  | "scanning"       // 3 探测器并行扫
  | "reviewing"      // 列出 DetectedKey, 等用户选
  | "associating"    // 提交关联 (per DetectedKey, 含 5 retry)
  | "completed"      // 全部完成或跳过
  | "error";         // 探测 + 关联都失败 (e.g. 5 retry 耗尽)

export interface OnboardingState {
  stage: OnboardingStage;
  detected_keys: DetectedKey[];
  selected_associations: Map<string, AssociationRequest>;  // detected_key_id → request
  test_results: Map<string, TestResult>;                   // detected_key_id → result
  started_at?: Iso8601;
  completed_at?: Iso8601;
  /** 跳过标记 (per 首次启动用户点"稍后") */
  skipped: boolean;
}

// ---- 6. Retry 策略 (per 拍板 retry_opt3) ----
export const RETRY_BACKOFF_MS: readonly number[] = [
  3_000,    // attempt 0 → 1: 3s
  6_000,    // attempt 1 → 2: 6s
  12_000,   // attempt 2 → 3: 12s
  24_000,   // attempt 3 → 4: 24s
  48_000,   // attempt 4 → 5: 48s
] as const;

export const MAX_RETRY_ATTEMPTS = RETRY_BACKOFF_MS.length;  // 5

// ---- 7. 错误码 (per 拍板 retryreport_opt3 解决步骤) ----
export type ProviderErrorCode =
  | "unauthorized"        // 401
  | "forbidden"           // 403
  | "rate_limited"        // 429
  | "network_timeout"     // 10s timeout
  | "model_unavailable"   // 404 / 503
  | "unknown";            // 其它

export interface ErrorResolution {
  code: ProviderErrorCode;
  /** 弹给用户的解决步骤 (e.g. "检查 key 是否有效, 重新生成") */
  steps: string[];
  /** 文档链接 (e.g. provider console API key page) */
  doc_url?: string;
  /** 调试命令 (e.g. curl test) */
  curl_test?: string;
}

// ---- 8. Provider 端点 (per provider 选 1 个测试 endpoint) ----
export interface ProviderTestEndpoint {
  provider: DetectedKey["provider"];
  /** 测试用 endpoint URL */
  test_url: string;
  /** 怎么从 DetectedKey 提取 key (e.g. "sk-...") */
  extract_key: (key: DetectedKey) => string | null;
  /** 怎么把 key 注入到 fetch headers */
  build_headers: (key: string) => Record<string, string>;
}

export const PROVIDER_TEST_ENDPOINTS: readonly ProviderTestEndpoint[] = [
  {
    provider: "openai",
    test_url: "https://api.openai.com/v1/models",
    extract_key: (k) => k.preview.startsWith("env:") ? null : k.preview.replace(/^.*-/, "sk-"),
    build_headers: (k) => ({ Authorization: `Bearer ${k}` }),
  },
  {
    provider: "claude",
    test_url: "https://api.anthropic.com/v1/messages",
    extract_key: (k) => k.preview.startsWith("env:") ? null : k.preview,
    build_headers: (k) => ({ "x-api-key": k, "anthropic-version": "2023-06-01" }),
  },
  {
    provider: "gemini",
    test_url: "https://generativelanguage.googleapis.com/v1beta/models",
    extract_key: (k) => k.preview.startsWith("env:") ? null : k.preview,
    build_headers: (k) => ({ "x-goog-api-key": k }),
  },
  {
    provider: "minimax",
    test_url: "https://api.minimax.chat/v1/models",
    extract_key: (k) => k.preview.startsWith("env:") ? null : k.preview,
    build_headers: (k) => ({ Authorization: `Bearer ${k}` }),
  },
] as const;
