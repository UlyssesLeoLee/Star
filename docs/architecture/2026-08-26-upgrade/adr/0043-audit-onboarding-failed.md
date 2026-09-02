# ADR-0043: audit-onboarding-failed — 既有 audit_audit_event への onboarding.failed イベント追加

> **ステータス**: Draft v0.1
> **日付**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 自审
> **触发**: per 2026-09-02 08:39 JST Ulysses 4 拍板 (仅 audit / .env 凭证 / 单 session / 增量 commit)
> **依据**: [ADR-0042-onboarding-first-run v0.1 §1.3 + §1.4](../architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md) + [commit `a54c79d` OnboardingGuard 实现](../.git) + [audit_audit_event.md v0.1 既有 schema](../../../data-design/ipa-detail/tables/audit_audit_event.md)

> **dual-use 提醒 (per AGENTS.md §5)**: 本 ADR 涉及 既存 audit_audit_event (T 類 append-only) への 1 event_type 追加. RGS 5 域 / 25 domain マッピング非該当 (audit は cross-cutting).

---

## §0 目的

Onboarding 5 回 retry 後の audit log 書き込みを Phase 1 の localStorage mock から Phase 2 の 既存 `audit_audit_event` テーブル (Transaction T 類, WORM) へ遷移する. 重要: **新表は作らない** (audit_audit_event は既に存在, 16 字段, RLS 13 類, WORM, 月次 BRIN partition, 7 年保持).

---

## §1 既存 audit_audit_event 状況 (per 9/2 08:40 JST 仓内実証)

- **物理名**: `audit.audit_event` (T30, per data-design.md §4.11.1)
- **種別**: Append-only (A) — REVOKE UPDATE/DELETE FROM PUBLIC + star_app_role
- **RLS**: Yes, 13 類 (per `policy_audit_event_tenant_isolation`)
- **Partition**: RANGE (occurred_at) 月次 BRIN
- **主キー**: `id UUID`
- **关键字段** (16 列, 1.5KB/行):
  - `actor_type` VARCHAR(16): `'user' | 'agent' | 'system'` (per `ck_audit_actor_type` CHECK)
  - `actor_id` UUID: NULL 可能
  - `action` VARCHAR(64): 自由文字列 (例 `'work_item:create'`)
  - `resource_type` VARCHAR(64): 自由文字列 (例 `'work_item'`)
  - `resource_id` UUID: NOT NULL
  - `before_state` JSONB, `after_state` JSONB
  - `context_refs` JSONB: Provenance 連動, GIN 索引
  - `tenant_id` UUID: NOT NULL, FK なし (Tenant 削除後記録保持)

---

## §2 設計 (4 段: 既存活用 / 1 event / 1 endpoint / 1 frontend bridge)

### 2.1 既存活用 (新表 0)

| Phase 1 (commit `a54c79d`) | Phase 2 (本 ADR) |
|---|---|
| localStorage `star:onboarding-audit` JSON 配列 | `audit.audit_event` INSERT (WORM) |
| `{ id, action, detected_key_id, provider, label, attempts, status_code, error_message, timestamp }` 9 字段 | `{ id, tenant_id, actor_type='system', action='onboarding.test_key.failed', resource_type='api_key', resource_id=detected_key_id, after_state={provider, label, attempts, status_code, error_message, detected_key_id}, occurred_at }` 既存 16 字段内で表現 |

### 2.2 1 event_type 追加

`audit_event.action` の値域 (自由文字列, CHECK 制約なし) に以下 1 値を追加:

| action | resource_type | 触发 | actor_type |
|---|---|---|---|
| `onboarding.test_key.failed` | `api_key` | 5 回 retry 後, retry.ts 内で trigger | `system` (onboarding は user ではなく system 行為) |

### 2.3 1 endpoint 追加

**`POST /api/audit/onboarding-failed`** (per spec/agent-api/onboarding.md §6.3):

```typescript
// Request
interface AuditOnboardingFailedRequest {
  detected_key_id: string;
  provider: string;
  label: string;
  attempts: number;          // 固定 5
  status_code: number;        // 0 = network error
  error_message: string;
  tenant_id: Uuid;            // 13 類必帯, per REQ-SEC-001
  client_ip?: string;         // 任意, audit_audit_event.client_ip に転記
  request_id?: Uuid;          // 任意, X-Request-Id
}

// Response 201
interface AuditOnboardingFailedResponse {
  audit_event_id: Uuid;
}
```

### 2.4 1 frontend bridge (retry.ts writeAuditLog)

Phase 1 の localStorage 書き込みを `fetch POST /api/audit/onboarding-failed` に置換, **fallback で localStorage 保持** (backend 障害時もデータ失わない):

```ts
async function writeAuditLog(key, result): Promise<string> {
  const payload = {
    detected_key_id: key.id,
    provider: key.provider,
    label: key.label,
    attempts: result.attempt + 1,
    status_code: result.status_code ?? 0,
    error_message: result.error_message ?? "",
    tenant_id: getTenantId(),  // 13 類必帯
  };
  try {
    const res = await fetch("/api/audit/onboarding-failed", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (res.ok) {
      const body = await res.json();
      return body.audit_event_id;
    }
  } catch (e) {
    // fall through to localStorage
  }
  // fallback: localStorage (Phase 1 mock, 既存挙動)
  return writeLocalStorageAuditLog(payload);
}
```

---

## §3 .env 設定 (per 9/2 08:39 JST phase2block 拍板)

`.env` (Mavis 接手は内容を読まない, 守門 #5). **变量名のみ契约** (`.env.example` に記述):

```bash
# Audit Backend (per ADR-0043 §2.3)
# Mavis 接手: 実値 記入しない. 变量名 + 形态のみ契约.
AUDIT_BACKEND_URL=http://localhost:3000     # audit_audit_event INSERT を受信する backend
AUDIT_TENANT_ID=tenant-physis-corp         # 13 類, per REQ-SEC-001
AUDIT_REQUEST_ID_HEADER=X-Request-Id       # 任意, audit_audit_event.request_id に転記
```

---

## §4 6 增量 commit 計画 (per 9/2 08:39 JST phase2bridge 拍板)

| # | commit | 内容 | token 估 |
|---|---|---|---|
| 1 | `docs(adr)` | 本 ADR-0043 v0.1 | 0.05M |
| 2 | `docs(audit)` | audit_audit_event.md v0.2 (action enum + onboarding event 追記) | 0.10M |
| 3 | `feat(audit-stub)` | domain-audit AuditRecorderPort stub (fn record_onboarding_failed) + cargo test 11 項 | 0.20M |
| 4 | `chore(env)` | .env.example 增量 (3 变量) + .env ガイド追記 | 0.05M |
| 5 | `feat(frontend-bridge)` | retry.ts writeAuditLog → fetch backend + localStorage fallback + MSW mock handler | 0.10M |
| 6 | `docs(spec)` | spec/agent-api/onboarding §6.3 后端契约 + ARCH-AGENT-GRAPH-001 报告 7 段更新 | 0.10M |
| **計** | | | **0.60M** |

---

## §5 守门对齐 (per AGENTS.md §4)

- **#1 禁回溯叙事**: 全て 9/2 08:39 JST 4 拍板 + commit a54c79d 实证, 既存 audit_audit_event を「なかったもの」にしない
- **#5 環境変数安全**: Mavis 接手は .env 内容を読まない, 变量名 + 形态のみ契约
- **#9 子代理実証**: Phase 2 全 6 commit 0 子代理调用, root 直实装
- **#10 代签规则**: author = Ulysses, Mavis 接手代签 (per 19:39/20:56/21:59 JST)
- **#11 缺标比错标**: 既存表活用, 新表 0, fallback 机制 (localStorage) 显式列
- **#12 文档治理**: ADR + audit_audit_event.md v0.2 + spec 6.3 跨引用闭环
- **#13 DB 三類横展開**: audit_audit_event は T 類 (既存), onboarding event は T 類内 1 action 追加, W 類 / M 類無关

---

## §6 既知の缺口 (per 缺标比错标, 守門 #11)

| # | 缺口 | Phase 計画 |
|---|---|---|
| 1 | backend 真接 (api crate は skeleton, infrastructure / application 同样是 skeleton, per AGENTS.md §7 P3-B 拍板待ち) | Phase 3+ |
| 2 | audit_audit_event INSERT の SQLx Adapter (infrastructure 層) | Phase 3+ |
| 3 | KMS 統合 (per domain-kms crate) | Phase 3+ |
| 4 | 真 fetch テスト (openai/claude/gemini/minimax 4 必备 provider) | Phase 3+ |
| 5 | IDE-residual 探测 (service worker) | Phase 3+ |
| 6 | Phase 1 localStorage → Phase 2 backend マイグレーション script | Phase 3+ |

---

## §7 签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | Mavis 接手代签 (per 19:39/20:56/21:59 JST) |
| SRE Lead | ⏳ 待签 | - | DDD Review 阶段补 |
| 平台 | ⏳ 待签 | - | 同上 |
| 评审主持 | ⏳ 待签 | - | 同上 |
| PM | ⏳ 待签 | - | 同上 |

---

## §8 修订历史

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 既存 audit_audit_event 活用 (新表 0) + 1 event 追加 + 1 endpoint + 1 frontend bridge + 6 增量 commit 計画 | 2026-09-02 08:39 JST Ulysses 4 拍板 (仅 audit / .env 凭证 / 单 session / 增量 commit) |

---

*本 ADR は Phase 2 限定 (audit のみ). 真 backend / KMS / 真 fetch は Phase 3+ (per 6 项 缺口).*
