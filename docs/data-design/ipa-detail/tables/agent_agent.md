# agent.agent — テーブル詳細設計書

> **テーブル ID**: T77
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.21.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T77 |
| **物理名** | `agent.agent` |
| **論理名** | エージェント（登録） |
| **スキーマ** | `agent` |
| **Module** | `domain-agent` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Agent 登録表。6 種別（Codex / ClaudeCode / GeminiCLI / OpenAICompatible / Local / Future）。`capabilities JSONB` で能力配列。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `agent_type` | 種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_agent_tenant_key` | 6 値 |
| 4 | `agent_provider` | Provider | VARCHAR | 64 | NO | − | − | − | − | − | 厂商識別子 |
| 5 | `agent_version` | バージョン | VARCHAR | 32 | NO | − | − | − | − | − | Agent バージョン |
| 6 | `display_name` | 表示名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 7 | `capabilities` | 能力 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | `['git','build','test',...]` |
| 8 | `policy_template_id` | ポリシーテンプレート ID | UUID | YES | `NULL` | − | − | `agent.agent_policy(id)` (App) | − | − | 紐付 Policy |
| 9 | `is_enabled` | 有効 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx (PT) | 登録有効 / 無効 |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_agent_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `uq_agent_tenant_key` | UNIQUE | `(tenant_id, agent_key)` | `WHERE deleted_at IS NULL` | 業務キー一意 |
| `ck_agent_type` | CHECK | `agent_type` | `IN ('Codex','ClaudeCode','GeminiCLI','OpenAICompatible','Local','Future')` | 6 値 |

> **注**: data-design §4.21.1 には `agent_key` 列が未実装、§00-INVENTORY.md 派生

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_agent_tenant_key` | btree (UK/PT) | `(tenant_id, agent_key)` | `deleted_at IS NULL` | 業務キー一意 |
| `idx_agent_tenant_type` | btree (PT) | `(tenant_id, agent_type)` | `deleted_at IS NULL` | 種別別 |
| `idx_agent_enabled` | btree (PT) | `(is_enabled)` | `is_enabled = TRUE` | 有効 Agent |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_agent_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500 |（100 テナント × 5 Agent） |
| 1 年後 | 5,000 |
| 3 年後 | 50,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 600 B | 50,000 | 約 30 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `agent.agent_policy` (App) | `policy_template_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `agent.agent_session` | `agent_id` |
| `worktree.worktree` (App) | `assigned_agent_id` |
| `feedback.feedback` | `author_agent_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE agent.agent ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.agent FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_agent_tenant_isolation ON agent.agent
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
