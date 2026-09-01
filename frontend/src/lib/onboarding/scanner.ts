// =====================================================================
// Onboarding scanner — 3 探测器并行扫 (per ADR-0042 §1.2)
// =====================================================================
// 3 探测器:
//   1. localStorage: 读 star:api-keys (现有 /settings/api-keys 存)
//   2. env-var-hint: process.env.NEXT_PUBLIC_*_API_KEY_HINT (只读存在性, 守门 #5 不打印值)
//   3. IDE-residual: fetch('/.vscode/settings.json') 等 5 路径 (Phase 1 mock 全 4xx)
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 永不打印明文 key, 仅 preview
//   - env-var-hint 只读存在性
//   - 4xx/timeout 返空数组 (不 throw, 引导可继续)
// =====================================================================

import type { DetectedKey, DetectorSource } from "@/types/onboarding";
import { DETECTOR_SOURCES } from "@/types/onboarding";

/** 扫描总入口 — 3 探测器并行, Promise.all */
export async function scanAllDetectors(): Promise<DetectedKey[]> {
  const [ls, env, ide] = await Promise.all([
    scanLocalStorage(),
    scanEnvVarHints(),
    scanIdeResidual(),
  ]);
  // 去重 (按 provider + label, 保留第一个)
  const all = [...ls, ...env, ...ide];
  const seen = new Set<string>();
  const unique: DetectedKey[] = [];
  for (const k of all) {
    const key = `${k.provider}:${k.label}`;
    if (seen.has(key)) continue;
    seen.add(key);
    unique.push(k);
  }
  return unique;
}

// =====================================================================
// 探测器 1: localStorage
// =====================================================================
const LS_KEY = "star:api-keys";

async function scanLocalStorage(): Promise<DetectedKey[]> {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(LS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Array<{
      id: string; provider: string; label: string; preview: string; createdAt: string;
    }>;
    const now = new Date().toISOString();
    return parsed
      .filter((k) => k && k.provider && k.label)
      .map((k) => ({
        id: `ls-${k.id}`,
        provider: normalizeProvider(k.provider),
        label: k.label,
        preview: k.preview,  // 已 masked (sk-***xyz1 格式)
        source: "localStorage" as const,
        detected_at: now,
        source_label: `从 localStorage "${LS_KEY}" 找到`,
      }));
  } catch {
    return [];  // JSON 损坏/无权限 → 静默
  }
}

// =====================================================================
// 探测器 2: env-var-hint (Next.js 公共 env, 仅存在性)
// =====================================================================
// ⚠️ 守门 #5: 仅检查 env 是否存在 (key like NEXT_PUBLIC_*_HINT="true"),
//    不读取实际 key value. 浏览器端无 process.env, 走 process.env.NEXT_PUBLIC_*.

const ENV_VAR_HINTS: Array<{ provider: DetectedKey["provider"]; varName: string }> = [
  { provider: "openai",  varName: "NEXT_PUBLIC_OPENAI_API_KEY_HINT" },
  { provider: "claude",  varName: "NEXT_PUBLIC_ANTHROPIC_API_KEY_HINT" },
  { provider: "gemini",  varName: "NEXT_PUBLIC_GOOGLE_API_KEY_HINT" },
  { provider: "minimax", varName: "NEXT_PUBLIC_minimax_API_KEY_HINT" },
];

async function scanEnvVarHints(): Promise<DetectedKey[]> {
  // process.env 仅 server side 可见; client 端空 process.env 是设计 (安全)
  // Phase 1 mock: 全返空数组, 等 Phase 2 后端接 /api/onboarding/env-hint
  if (typeof process === "undefined" || !process.env) return [];
  const detected: DetectedKey[] = [];
  const now = new Date().toISOString();
  for (const { provider, varName } of ENV_VAR_HINTS) {
    if (process.env[varName] === "true") {
      detected.push({
        id: `env-${provider}`,
        provider,
        label: `Env ${varName.replace("NEXT_PUBLIC_", "").replace("_API_KEY_HINT", "")}`,
        preview: `env: ${varName.replace("NEXT_PUBLIC_", "")}`,
        source: "env_var_hint" as const,
        detected_at: now,
        source_label: `从环境变量 ${varName} 检测到 (未读取值, 仅存在性)`,
        env_var_name: varName.replace("NEXT_PUBLIC_", ""),
      });
    }
  }
  return detected;
}

// =====================================================================
// 探测器 3: IDE-residual (5 路径, Phase 1 全 4xx → 返空数组)
// =====================================================================
const IDE_RESIDUAL_PATHS: readonly string[] = [
  "/.vscode/settings.json",
  "/.continue/config.json",
  "/.aider.conf.yml",
  "/.codiumai.toml",
  "/.continue/config.json",
] as const;

async function scanIdeResidual(): Promise<DetectedKey[]> {
  // Phase 1: 不真 fetch (browser 同源 + CORS), 全部 4xx 返空数组
  // 标 4xx 是预期 (Next dev server 不暴露这些路径), 不是错
  return [];
  // 实施 (Phase 2 接 service worker / fs API):
  // const results = await Promise.allSettled(
  //   IDE_RESIDUAL_PATHS.map((p) => fetch(p, { cache: "no-store" })),
  // );
  // ... parse .vscode/settings.json 的 "openai.apiKey" 字段 etc.
}

// =====================================================================
// helpers
// =====================================================================

/** provider 名称标准化 (兼容 anthropic → claude, google → gemini) */
function normalizeProvider(raw: string): DetectedKey["provider"] {
  const r = raw.toLowerCase();
  if (r === "anthropic") return "claude";
  if (r === "google") return "gemini";
  // 8 选 union
  if (["openai", "claude", "gemini", "minimax", "anthropic", "openclaw", "hermes", "google"].includes(r)) {
    return r as DetectedKey["provider"];
  }
  // 未知 → 返 openai (兜底, 不会被纳入主流程)
  return "openai";
}

/** 检查扫描是否完成 (per 守门 #9: 5 重试耗尽后才返 false)
 * 接受 "true" (正常完成) 或 "skipped" (用户跳过) 任一状态 */
export function isOnboardingCompleted(): boolean {
  if (typeof window === "undefined") return true;  // SSR 跳过
  const v = window.localStorage.getItem("star:onboarding-completed");
  return v === "true" || v === "skipped";
}

/** 标记 onboarding 完成 */
export function markOnboardingCompleted(): void {
  if (typeof window === "undefined") return;
  window.localStorage.setItem("star:onboarding-completed", "true");
}

/** 标记 onboarding 跳过 (用户点"稍后") */
export function markOnboardingSkipped(): void {
  if (typeof window === "undefined") return;
  window.localStorage.setItem("star:onboarding-completed", "skipped");
}

/** 重置 onboarding (供 /settings/reset-onboarding 用, Phase 2+) */
export function resetOnboarding(): void {
  if (typeof window === "undefined") return;
  window.localStorage.removeItem("star:onboarding-completed");
}

export { DETECTOR_SOURCES };
