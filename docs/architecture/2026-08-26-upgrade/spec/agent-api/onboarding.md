# arch-agent-onboarding 詳細設計 (Spec)

> **ドキュメントバージョン**: v0.1 (2026-09-02)
> **ステータス**: 🟢 Phase 1 完了 (frontend contract + 3 探测器 + 5 retry mock)
> **位置付け**: 詳細設計 (per 仓内惯例, agent-api/spec 配下)
> **一次出典**: [ADR-0042-onboarding-first-run v0.1](../../adr/0042-onboarding-first-run.md)
> **実装**: commit `a54c79d` (frontend 8 ファイル, 1553 行, tsc 0 + 337/337 vitest pass)
> **関連**: 需求 §49 (REQ-ONB-001~005) + 基本設計 §12

> **dual-use 提醒 (per AGENTS.md §5)**: 本 spec は UX + ブラウザ storage 范囲内の事, 25 domain / RGS 5 域マッピング非該当。

---

## §0 概要

### 0.1 目的

ユーザーが Star Platform を **初回起動した時**, システムが既存 LLM API key を **自動識別** (localStorage / env-var-hint / IDE-residual 3 探测器), ユーザーに **エージェント選択 + 関連付け** UI を提示, 関連付け **失敗時 5 回自動リトライ** (3-6-12-24-48s 指数 backoff), 最終的に失敗したら **解決ステップ + audit log** で対応。

### 0.2 範囲

| # | 含む | 含まない |
|---|---|---|
| 1 | 3 探测器並列スキャン (localStorage / env-var-hint / IDE-residual) | IDE 残留 Phase 1 (Phase 2 service worker) |
| 2 | ユーザー agent 選択 + 関連付け | backend 真接 Phase 1 (Phase 2 KMS) |
| 3 | 5 回リトライ + 3-6-12-24-48s backoff | 真 fetch テスト (Phase 1 mock) |
| 4 | 6 ProviderErrorCode 解決ステップ | audit テーブル Phase 1 (Phase 2 audit_audit_event) |
| 5 | audit log localStorage (Phase 1 mock) | per-user 集約 (Phase 2+) |
| 6 | encrypted_rust 存储 (既存沿用) | KMS 統合 (Phase 2) |

### 0.3 用語

| 用語 | 意味 |
|---|---|
| **3 探测器** | localStorage / env-var-hint / IDE-residual 並列スキャン |
| **DetectedKey** | 検出された 1 個の LLM API key, 8 フィールド (id/provider/label/preview/source/detected_at/source_label/env_var_name) |
| **ProviderErrorCode** | 6 種類 (unauthorized / forbidden / rate_limited / model_unavailable / network_timeout / unknown) |
| **3-6-12-24-48s backoff** | 5 回リトライ時の各 attempt 間待機時間 (per 拍板 retry_opt3) |
| **audit log** | 5 回失敗時記録, Phase 1 localStorage, Phase 2 audit_audit_event (Transaction T) |

---

## §1 アーキテクチャ概要

### 1.1 3 層 + UI 層構造

```
┌────────────────────────────────────────────────────────────┐
│ Mount 点: app/layout.tsx → <OnboardingGuard />             │
└────────────────────────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────────┐
│ UI 層: components/OnboardingGuard.tsx                       │
│   1. mount 时 scanAllDetectors()                            │
│   2. isOnboardingCompleted() → skip if true                │
│   3. runTests(detectedKeys) → 5 retry per key              │
│   4. 状态: idle/scanning/reviewing/associating/completed/error│
└────────────────────────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────────┐
│ Modal 層: lib/onboarding/Guide.tsx (4 阶段 UI)             │
│   - scanning: spinner + "3 探测并列扫"                      │
│   - reviewing: DetectedKey 列表 + per-key agent select     │
│   - associating: per-key progress (attempt/next_retry)     │
│   - completed | error: 成功/失敗カード + 解决步骤            │
└────────────────────────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────────┐
│ Scanner 層: lib/onboarding/scanner.ts                       │
│   - scanLocalStorage() → 既存 /settings/api-keys            │
│   - scanEnvVarHints() → process.env.NEXT_PUBLIC_*_HINT     │
│   - scanIdeResidual() → 5 路径 fetch (Phase 1 mock 返空)   │
│   - 去重: provider+label 一致 → 最初の 1 件                   │
└────────────────────────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────────┐
│ Retry 層: lib/onboarding/retry.ts                            │
│   - testKeyWithRetry(key, onProgress) → 5 attempts         │
│   - 单 attempt: 10s timeout, AbortController                │
│   - backoff: [3s, 6s, 12s, 24s, 48s]                       │
│   - 5 回失敗 → audit log + writeAuditLog()                  │
│   - ERROR_RESOLUTIONS: 6 ProviderErrorCode × 解决步骤       │
└────────────────────────────────────────────────────────────┘
```

### 1.2 4 阶段状態機 (sequence 図)

```mermaid
sequenceDiagram
    autonumber
    actor U as User (Ulysses)
    participant L as app/layout.tsx
    participant G as OnboardingGuard
    participant S as scanner.scanAllDetectors
    participant R as retry.testKeyWithRetry
    participant API as /api/api-keys
    participant LS as localStorage

    L->>G: mount
    G->>LS: isOnboardingCompleted()
    alt completed
        LS-->>G: true | skipped
        G-->>L: (modal 閉)
    else 初回
        LS-->>G: false
        G->>G: setStage("scanning")
        G->>S: scanAllDetectors()
        par parallel 3 探测
            S->>LS: localStorage star:api-keys
        and
            S->>S: process.env.NEXT_PUBLIC_*_HINT
        and
            S->>S: 5 路径 fetch (mock)
        end
        S-->>G: DetectedKey[]

        alt 空 (0 keys)
            G->>G: setStage("reviewing")
            U->>G: 点"稍后" → markOnboardingSkipped() → 閉
        else 有 keys
            G->>G: setStage("associating")
            loop per key (parallel OK)
                G->>R: testKeyWithRetry(key, onProgress)
                R->>R: attempt 0
                R->>API: (mock) GET provider endpoint
                alt success
                    R-->>G: TestResult{status: "success"}
                else fail
                    R->>R: wait 3s, attempt 1
                    R->>R: wait 6s, attempt 2
                    ...
                    R->>R: wait 48s, attempt 5 → fail
                    R->>LS: writeAuditLog()
                    R-->>G: TestResult{status: "failed", audit_event_id}
                end
            end
            G->>G: setStage("completed" | "error")
            U->>G: "完成" → markOnboardingCompleted() → 閉
        end
    end
```

---

## §2 3 探测器仕様

### 2.1 localStorage 探测器

| 項目 | 値 |
|---|---|
| キー | `star:api-keys` |
| 形式 | JSON 配列 |
| 1 要素 | `{ id, provider, label, preview, createdAt, ... }` |
| 失敗時 | 空配列 (JSON 损坏 → catch) |
| preview | masked (e.g. "sk-***xyz1", 守門 #5 永続化しない) |

**コード (per `lib/onboarding/scanner.ts`):**

```ts
const LS_KEY = "star:api-keys";
async function scanLocalStorage(): Promise<DetectedKey[]> {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(LS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as ApiKey[];
    return parsed.map((k) => ({
      id: `ls-${k.id}`,
      provider: normalizeProvider(k.provider),
      label: k.label,
      preview: k.preview,
      source: "localStorage" as const,
      detected_at: new Date().toISOString(),
      source_label: `从 localStorage "${LS_KEY}" 找到`,
    }));
  } catch { return []; }
}
```

### 2.2 env-var-hint 探测器

| 項目 | 値 |
|---|---|
| 変数 | 4 個 (`NEXT_PUBLIC_OPENAI_API_KEY_HINT` etc.) |
| 判定 | `=== "true"` で存在性のみ |
| preview | `env: OPENAI_API_KEY` (変数名のみ, 値なし, 守門 #5) |
| 失敗時 | 変数未設定 → skip |

### 2.3 IDE-residual 探测器

| 項目 | 値 |
|---|---|
| 5 路径 | `/.vscode/settings.json`, `/.continue/config.json`, `/.aider.conf.yml`, `/.codiumai.toml`, `/.continue/config.json` |
| Phase 1 | mock 返空 (CORS / browser 同源制約) |
| Phase 2 | service worker / fs API / Next.js API route (`/api/onboarding/ide-residual`) |

---

## §3 5 重试 + 3-6-12-24-48s backoff

### 3.1 流程

```
for attempt in 0..4:
  testKeyOnce(key, AbortController(10s))
  ├─ ok → return TestResult{status: "success", attempt}
  └─ fail → wait RETRY_BACKOFF_MS[attempt] → next attempt
            └─ attempt === 4 (last) → writeAuditLog() → return failed
```

### 3.2 RETRY_BACKOFF_MS 表

| Attempt | Backoff (ms) | Backoff (s) |
|---|---|---|
| 0 → 1 | 3,000 | 3s |
| 1 → 2 | 6,000 | 6s |
| 2 → 3 | 12,000 | 12s |
| 3 → 4 | 24,000 | 24s |
| 4 → 5 | 48,000 | 48s |

### 3.3 audit log 記録 (per 守門 #9)

```ts
function writeAuditLog(key, result): string {
  const id = `audit-${Date.now()}-${key.provider}`;
  const entry = {
    id, action: "onboarding.test_key.failed",
    detected_key_id: key.id, provider: key.provider, label: key.label,
    attempts: result.attempt + 1, status_code: result.status_code,
    error_message: result.error_message, timestamp: new Date().toISOString(),
  };
  localStorage.setItem("star:onboarding-audit", JSON.stringify([
    ...existing, entry,
  ]));
  return id;
}
```

### 3.4 6 ProviderErrorCode × 解决步骤

| Error Code | Status | 解决步骤 | doc URL | curl test |
|---|---|---|---|---|
| **unauthorized** | 401 | 检查 key, 重新生成, 確認无多余空格 | platform.openai.com/account/api-keys | `curl -H "Authorization: Bearer $KEY" .../v1/models` |
| **forbidden** | 403 | 开通模型访问权限, 检查账户余额 | console.anthropic.com/settings/billing | `curl -H "x-api-key: $KEY" .../v1/messages` |
| **rate_limited** | 429 | 等待 1 分钟, 换备用 key, 升级套餐 | platform.openai.com/docs/guides/rate-limits | (无需 curl) |
| **network_timeout** | 0 | 检查网络 VPN 防火墙, 联系 IT | (内部) | `curl -v https://api.openai.com/...` |
| **model_unavailable** | 404/503 | provider status 页, 切换其它模型 | status.openai.com | `curl -s .../api/v2/status.json` |
| **unknown** | 500/其它 | 重试, 提交 issue | (无) | (见 provider console debug) |

---

## §4 4 阶段 UI 仕様

### 4.1 scanning 阶段

- DOM: `<div data-testid="onboarding-stage-scanning">`
- 内容: `<Loader2 animate-spin />` + "正在扫描本地凭证..." + "3 探测器并行: localStorage + env-var-hint + IDE-residual"
- 退出: `runScan()` resolve → `setStage("reviewing")` | `setStage("associating")` (有 keys)

### 4.2 reviewing 阶段

- DOM: `<div data-testid="onboarding-stage-reviewing">`
- 内容:
  - `<ShieldCheck info />` + "检测到 N 个 API key"
  - per-key `<div data-testid="onboarding-key-{id}">`:
    - Row 1: `<Key />` + provider chip (4 必备 = primary + "必备" badge) + label + source badge + result icon
    - Row 2: `<font-mono preview>` (永不明文)
    - Row 3: "关联到:" + `<select data-testid="onboarding-agent-select-{id}">` (availableAgents 列表 + "暂不关联")
    - Row 4: `<font-mono source_label>`
- 退出: "确认关联 (N)" click → `onAssociate()` → `setStage("associating")`

### 4.3 associating 阶段

- DOM: `<div data-testid="onboarding-stage-associating">`
- 内容:
  - `<ShieldCheck info />` + "正在测试 N 个 key (5 重试 + 3-6-12-24-48s backoff)"
  - per-key `<div data-testid="onboarding-test-{id}">`:
    - provider + label + status icon (running spinner / success check / failed alert / 待开始)
    - running 时: `attempt X/5 · 下次重试 Xs 后`
    - failed 时: error message + status code
- 退出: 全 test resolve → `setStage("completed" | "error")`

### 4.4 completed 阶段

- DOM: `<div data-testid="onboarding-stage-completed">`
- 内容:
  - `<CheckCircle2 success size={32} />` + "关联完成"
  - "X 成功 / Y 失败 / Z 总计"
  - failed > 0 时: "⚠️ 失败 key 已写入 audit log. 你可以: agent tab 旁 ⚙️ 齿轮按钮重新关联 / /settings/api-keys"

### 4.5 error 阶段

- DOM: `<div data-testid="onboarding-stage-error">`
- 内容:
  - `<AlertTriangle danger size={28} />` + "N 个 key 5 次重试后仍失败"
  - per-failed-key `<div data-testid="onboarding-error-{id}">`:
    - provider + label + "重试" 按钮 (re-run test)
    - error_message + status_code + "5/5 失败"
    - `<ErrorResolutionCard>`:
      - 解决步骤 (1-3 steps)
      - doc URL (link, ExternalLink icon)
      - curl test command (details/summary, Code2 icon)

---

## §5 状態管理

### 5.1 OnboardingGuard 状態

```ts
const [open, setOpen] = useState(false);
const [stage, setStage] = useState<OnboardingStage>("idle");
const [detectedKeys, setDetectedKeys] = useState<DetectedKey[]>([]);
const [testResults, setTestResults] = useState<Map<string, TestResult>>(new Map());
const startedRef = useRef(false);  // 防止 mount 时 2 重実行
```

### 5.2 5 state 操作 (per localStorage)

| 関数 | 動作 |
|---|---|
| `isOnboardingCompleted()` | `star:onboarding-completed === "true" \| "skipped"` (両方 受入) |
| `markOnboardingCompleted()` | set "true" |
| `markOnboardingSkipped()` | set "skipped" |
| `resetOnboarding()` | remove (Phase 2 /settings/reset-onboarding 用) |

---

## §6 API 契約 (Phase 2 で追加)

### 6.1 `GET /api/onboarding/env-hint`

```typescript
// Response 200
interface EnvHintResponse {
  hints: Array<{ provider: "openai" | "claude" | "gemini" | "minimax"; varName: string }>;
  // Phase 1: 全空 (process.env 不可)
  // Phase 2: 后端 process.env 走
}
```

### 6.2 `POST /api/onboarding/test-key`

```typescript
// Request
interface TestKeyRequest {
  provider: "openai" | "claude" | "gemini" | "minimax" | ...;
  // 加密 key id, 后端 master key で復号
  api_key_id: string;
}

// Response 200
interface TestKeyResponse {
  status: "success" | "failed";
  status_code?: number;
  error_message?: string;
  // 1 attempt のみ, retry は client 側
}
```

### 6.3 `POST /api/audit/onboarding-failed` (per ADR-0043 v0.1, commit 62bc032 / fa05464 / f14ef0f 真接)

```typescript
// Request (per ADR-0043 §2.3, 10 字段)
interface AuditOnboardingFailedRequest {
  detected_key_id: string;        // DetectedKey.id (UUID)
  provider: string;                // "openai" | "claude" | "gemini" | "minimax" | 兼容 4
  label: string;                   // DetectedKey.label
  attempts: number;                // 固定 5
  status_code: number;             // 0 = network error / 4xx / 5xx
  error_message: string;
  tenant_id: Uuid;                  // 13 類必帯, per REQ-SEC-001
  client_ip?: string;               // 任意, audit_audit_event.client_ip に転記
  request_id?: Uuid;                // 任意, audit_audit_event.request_id に転記
}

// Response 201
interface AuditOnboardingFailedResponse {
  audit_event_id: string;            // Phase 2 実 backend → audit_audit_event.id (UUID)
  occurred_at: Iso8601;              // 返り値 追加 (per commit f14ef0f MSW mock)
}
```

**后端 INSERT 仕様** (per ADR-0043 §2.2 + commit fae5c66 v0.2):
- 既存 `audit_audit_event` テーブル (T30, T 類 append-only WORM, 16 字段) に直接 INSERT
- 新表 0
- `action` = `onboarding.test_key.failed` (per ck_audit_action_v0_2 DOMAIN 制約)
- `actor_type` = `system`
- `resource_type` = `api_key`
- `resource_id` = `detected_key_id`
- `after_state` (JSONB) = `{ provider, label, attempts, status_code, error_message, detected_key_id }`
- `tenant_id` = request.tenant_id (per 13 類 RLS 強制)

**frontend bridge** (per commit f14ef0f):
- `retry.ts` writeAuditLog async 化
- backend fetch 優先, 失敗時 localStorage fallback (Phase 1 挙動保持)
- 6 变量 (per .env.example, commit 62c18f5):
  - `AUDIT_BACKEND_URL` (default `http://localhost:3000`)
  - `AUDIT_TENANT_ID` (default `tenant-physis-corp`)
  - `AUDIT_REQUEST_ID_HEADER` (default `X-Request-Id`)
  - `AUDIT_BACKEND_TIMEOUT_MS` (default 10000)
  - `AUDIT_ONBOARDING_RETRY_ENABLED` (default `true`)
  - `AUDIT_FAILURE_MODE` (default `silent`)

---

## §7 セキュリティ (per AGENTS.md §4 #5, #9, #10)

| 観点 | 対策 |
|---|---|
| **テナント分離** | 13 類 tenant_id 必帯, OnboardingGuard prop, Phase 1 mock = `tenant-physis-corp` |
| **Secret 漏洩防止** | env-var-hint 存在性のみ, preview masked (`sk-***xyz`), audit log 内に status_code のみ, 値なし |
| **audit log 必帯** | 5 回失敗時 `star:onboarding-audit` localStorage (Phase 1), `audit_audit_event` テーブル (Phase 2) |
| **secret redaction** | error message 内に "Invalid API key (mock)" 形式, 値含まない |
| **PII 排除** | audit log 内に email / display_name なし, provider/label/status_code のみ |
| **timeout** | 单 attempt 10s AbortController, 5 回累计 93s 最大 (守門 #6 なし, UX 优先) |

---

## §8 テスト戦略

### 8.1 単体テスト (vitest 11/11 pass, per commit `a54c79d`)

| テスト | 検証 | 件数 |
|---|---|---|
| `scanAllDetectors` | localStorage 0/1/2 keys + 去重 | 3 |
| `testKeyWithRetry` | 5 attempts success / 5 fail + audit log | 2 |
| `ERROR_RESOLUTIONS` | 6 code 全 steps + curl_test | 1 |
| `classifyStatus` | 401/403/429/404/503/0/500 → 正确 code | 1 |
| 状態機 | completed / skipped / reset | 4 |

### 8.2 統合テスト (Phase 2)

- E2E: 「初回起動 → modal 弹起 → agent 选 → 5 retry → 成功/失败 → audit log」
- 6 ProviderErrorCode 解決ステップ E2E (per provider 別)
- vi.useFakeTimers で retry 5 回 高速化 (現状 45s → < 1s)

### 8.3 性能テスト (Phase 3)

- 3 探测並列完走 < 1s
- 5 retry 真等 < 93s (3+6+12+24+48 = 93s)
- localStorage audit log 容量 < 1MB (90 日後手動 cleanup)

---

## §9 既知の缺口 (per 缺标比错标, 守門 #11)

| # | 缺口 | Phase 計画 |
|---|---|---|
| 1 | IDE-residual 探测器 Phase 1 返空 | Phase 2+ 接 service worker |
| 2 | env-var-hint Phase 1 mock 返空 | Phase 2+ 接 /api/onboarding/env-hint |
| 3 | testKeyOnce Phase 1 mock ランダム | Phase 2 真接 fetch + ep.build_headers |
| 4 | audit log localStorage (Phase 1) | Phase 2 真接 audit_audit_event テーブル |
| 5 | retry 真等 3-6-12-24-48s (45s テスト) | Phase 1 OK, vi.useFakeTimers で最適化可能 |
| 6 | 関連付け backend 真接 | Phase 2 + KMS 統合 |
| 7 | per-user 集約 (analytics) | Phase 3+ |

---

## §10 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 10 段 (概要 / アーキテクチャ / 3 探测 / 5 retry / 4 stage UI / 状態管理 / API 契約 / セキュリティ / テスト / 缺口) + commit `a54c79d` 8 ファイル 1553 行 参照 | 2026-09-02 07:58 JST Ulysses "启动最新版 + 继续推进" + 08:01 JST 4 拍板 (trigger opt1 / scope opt4 / retry opt3 / storage opt1) |

---

*本 spec は Phase 2 実装 (audit_audit_event 真接 + 真 fetch + IDE-residual) の起点, 段階的に更新する。*
