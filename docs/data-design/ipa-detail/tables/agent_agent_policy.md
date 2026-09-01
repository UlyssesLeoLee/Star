# agent.agent_policy — テーブル詳細設計書

> **テーブル ID**: T80
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.21.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T80 |
| **物理名** | `agent.agent_policy` |
| **論理名** | エージェントポリシー |
| **スキーマ** | `agent` |
| **Module** | `domain-agent` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Agent Policy テンプレート。**12 強制点**（§4.2.5 / §R-PERM-002）。`network_access` 3 値 + `secret_access` 3 値 + リソース制限 4 種 + Gate 3 種。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `name` | ポリシー名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_policy_tenant_name` | 業務表示名 |
| 4 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 5 | `allowed_repositories` | 許可リポジトリ配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | Repo 限定 |
| 6 | `allowed_worktrees` | 許可ワークツリー配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | Worktree 限定 |
| 7 | `allowed_paths` | 許可パス配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | − | − | パス限定 |
| 8 | `forbidden_paths` | 禁止パス配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | − | − | 禁止パス |
| 9 | `allowed_tools` | 許可ツール配列 | VARCHAR(64)[] | − | NO | `'{}'` | − | − | − | GIN | ツール限定 |
| 10 | `allowed_command_categories` | 許可コマンドカテゴリ配列 | VARCHAR(64)[] | − | NO | `'{}'` | − | − | − | − | コマンドカテゴリ |
| 11 | `network_access` | ネットワークアクセス | VARCHAR | 16 | NO | `'Deny'` | − | − | − | − | 3 値 |
| 12 | `secret_access` | シークレットアクセス | VARCHAR | 16 | NO | `'None'` | − | − | − | − | 3 値 |
| 13 | `max_runtime_seconds` | 最大ランタイム（秒） | INT | 4 | NO | `3600` | − | − | − | − | 1h 既定、≤ 24h |
| 14 | `max_context_tokens` | 最大コンテキストトークン | INT | 4 | NO | `128000` | − | − | − | − | 128K Standard |
| 15 | `max_change_files` | 最大変更ファイル数 | INT | 4 | NO | `50` | − | − | − | − | リソース制限 |
| 16 | `max_change_lines` | 最大変更行数 | INT | 4 | NO | `2000` | − | − | − | − | リソース制限 |
| 17 | `require_review` | レビュー必須 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | Gate |
| 18 | `require_test` | テスト必須 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | Gate |
| 19 | `require_approval` | 承認必須 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | Gate |
| 20 | `is_builtin` | Built-in フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | システム定義 |
| 21 | `is_enabled` | 有効フラグ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | PT | 有効 / 無効 |
| 22 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 23 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 24 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 25 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_policy_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_agent_policy_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `uq_policy_tenant_name` | UNIQUE | `(tenant_id, name, deleted_at)` | − | 業務名一意 |
| `ck_policy_network_access` | CHECK | `network_access` | `IN ('Allow','Deny','Scoped')` | 3 値 |
| `ck_policy_secret_access` | CHECK | `secret_access` | `IN ('BrokerOnly','Scoped','None')` | 3 値 |
| `ck_policy_max_runtime` | CHECK | `max_runtime_seconds` | `max_runtime_seconds > 0 AND max_runtime_seconds <= 86400` | 0..24h |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_policy_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_policy_tenant_name` | btree (UK/PT) | `(tenant_id, name, deleted_at)` | − | 業務名一意 |
| `idx_agent_policy_tenant_enabled` | btree (PT) | `(tenant_id, is_enabled)` | `deleted_at IS NULL` | テナント + 有効 |
| `idx_agent_policy_allowed_repos_gin` | GIN | `allowed_repositories` | `deleted_at IS NULL` | Repo 配列検索 |
| `idx_agent_policy_allowed_tools_gin` | GIN | `allowed_tools` | `deleted_at IS NULL` | ツール配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_agent_policy_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500 |（100 テナント × 5 Policy） |
| 1 年後 | 5,000 |
| 3 年後 | 50,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (配列 ×7) | 50,000 | 約 75 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `agent.agent.policy_template_id` (App) | − |

---

## 9. RLS Policy

```sql
ALTER TABLE agent.agent_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.agent_policy FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_agent_policy_tenant_isolation ON agent.agent_policy
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
