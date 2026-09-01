# local_runtime.runtime_command — テーブル詳細設計書

> **テーブル ID**: T97
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.25.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T97 |
| **物理名** | `local_runtime.runtime_command` |
| **論理名** | ランタイムコマンド（白名单） |
| **スキーマ** | `local_runtime` |
| **Module** | `domain-local-runtime` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | サーバー側 → Daemon コマンド下発。**8 種 白名单**（D-03 修正後、ReportObservation 除外）：GitStatus / CreateWorktree / ReadDiff / RunApprovedTest / QueryAgentStatus / SubmitFeedback / StartAuthorizedAgentSession / StopAgentSession。`command_token_hash` で 5min TTL 一次性消費。SEC-008 が ExecuteArbitraryShell 等を拦截（§4.6.3 / §API-7.2.1）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | idx | RLS 必須 |
| 3 | `runtime_id` | ランタイム ID | UUID | − | NO | − | − | `local_runtime.runtime(id)` ON DELETE CASCADE | idx | 親 Runtime |
| 4 | `command_type` | コマンド種別 | VARCHAR | 32 | NO | − | − | − | − | 8 種白名单 |
| 5 | `command_args` | コマンド引数 | JSONB | − | NO | − | − | − | − | 必帯 worktree_id / agent_session_id / repository_id |
| 6 | `command_token_hash` | コマンドトークンハッシュ | VARCHAR | 255 | NO | − | − | − | idx | bcrypt hash、一次性消費 |
| 7 | `expires_at` | 有効期限 | TIMESTAMPTZ | 8 | NO | − | − | − | idx (PT) | 5min TTL |
| 8 | `issued_by_user_id` | 発行者 ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | 発行ユーザ |
| 9 | `status` | 状態 | VARCHAR | 16 | NO | `'PENDING'` | − | − | idx (PT) | 6 値 |
| 10 | `result_payload` | 結果ペイロード | JSONB | − | YES | `NULL` | − | − | − | 実行結果 |
| 11 | `executed_at` | 実行日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | 実行開始 |
| 12 | `completed_at` | 完了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | 完了 |
| 13 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `runtime_command_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_runtime_command_*` | FOREIGN KEY (2) | `tenant_id` / `runtime_id` | 各親テーブル | CASCADE | − |
| `ck_runtime_command_type` | CHECK | `command_type` | `IN (8 値, D-03 修正後)` | 8 値白名单 |
| `ck_runtime_command_status` | CHECK | `status` | `IN ('PENDING','EXECUTING','COMPLETED','FAILED','EXPIRED','CANCELLED')` | 6 値 |
| `ck_runtime_command_expiry` | CHECK | `expires_at` | `expires_at > created_at` | TTL 整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `runtime_command_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_runtime_command_tenant_runtime` | btree | `(tenant_id, runtime_id, created_at DESC)` | − | テナント + Runtime + 順 |
| `idx_runtime_command_pending` | btree (PT) | `(runtime_id, created_at)` | `status = 'PENDING'` | 未実行キュー |
| `idx_runtime_command_token_hash` | btree | `command_token_hash` | − | トークン検索 |
| `idx_runtime_command_expires` | btree (PT) | `(expires_at)` | `status IN ('PENDING','EXECUTING')` | 期限監視 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB | 10,000,000 | 約 12 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `local_runtime.runtime` | `runtime_id` |
| `identity.user` (App) | `issued_by_user_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE local_runtime.runtime_command ENABLE ROW LEVEL SECURITY;
ALTER TABLE local_runtime.runtime_command FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_runtime_command_tenant_isolation ON local_runtime.runtime_command
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
