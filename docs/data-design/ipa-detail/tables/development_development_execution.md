# development.development_execution — テーブル詳細設計書

> **テーブル ID**: T63
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T63 |
| **物理名** | `development.development_execution` |
| **論理名** | 開発実行（核心） |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WorkItem の実行環境における 1 回 / 複数回 実行 聚合。8 ID 配列（worktree / agent_session / change_set / validation / feedback / commit / pull_request）で cross-domain 連動。3 状態。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 親 WorkItem |
| 5 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE RESTRICT | − | idx | 対象 Repo |
| 6 | `worktree_ids` | ワークツリー ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 関連 Worktree |
| 7 | `agent_session_ids` | エージェントセッション ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 関連 Agent Session |
| 8 | `change_set_ids` | 変更セット ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 ChangeSet |
| 9 | `validation_result_ids` | 検証結果 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 Validation |
| 10 | `feedback_ids` | フィードバック ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 Feedback |
| 11 | `commit_ids` | コミット ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 Commit |
| 12 | `pull_request_ids` | PR ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 PR |
| 13 | `started_at` | 開始日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 実行開始 |
| 14 | `ended_at` | 終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 実行終了 |
| 15 | `execution_state` | 実行状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | − | 3 値 |
| 16 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 17 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 18 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 19 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `development_execution_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_dev_exec_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_dev_exec_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `fk_dev_exec_workitem` | FOREIGN KEY | `work_item_id` | `work_item.work_item(id)` | CASCADE | 親 WorkItem 削除時 実行削除 |
| `fk_dev_exec_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` | RESTRICT | Repo 削除禁止 |
| `ck_execution_state` | CHECK | `execution_state` | `IN ('ACTIVE','COMPLETED','ABANDONED')` | − | 3 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `development_execution_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_development_execution_tenant_workitem` | btree (PT) | `(tenant_id, work_item_id)` | `deleted_at IS NULL` | テナント + WorkItem |
| `idx_development_execution_tenant_repo` | btree (PT) | `(tenant_id, repository_id)` | `deleted_at IS NULL` | テナント + Repo |
| `idx_development_execution_worktree_ids_gin` | GIN | `worktree_ids` | − | Worktree 配列検索 |
| `idx_development_execution_agent_session_ids_gin` | GIN | `agent_session_ids` | − | Agent Session 配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_development_execution_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 1.5 KB (配列×8) | 10,000,000 | 約 15 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `work_item.work_item` | `work_item_id` |
| `scm.repository` | `repository_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `development.development_context` | `execution_id` |
| `development.change_set` (reverse 配列) | `change_set_ids` |

---

## 9. RLS Policy

```sql
ALTER TABLE development.development_execution ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.development_execution FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_development_execution_tenant_isolation ON development.development_execution
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
