# worktree.worktree — テーブル詳細設計書

> **テーブル ID**: T72
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.20.1 + ON-001 修正

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T72 |
| **物理名** | `worktree.worktree` |
| **論理名** | ワークツリー（核心） |
| **スキーマ** | `worktree` |
| **Module** | `domain-worktree` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Worktree 核心。17 状態（§3.3.1）+ 4 health + 5 build_state + 4 context_state。30+ 列（`test_state` / `feedback_state` / `synchronization_state` JSONB 灵活）。`changed_files` / `changed_symbols` 配列で GIN 検索。 |

---

## 2. カラム一覧（主要）

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `workspace_id` | ワークスペース ID | UUID | − | NO | − | − | `workspace.workspace(id)` ON DELETE RESTRICT | − | idx | 所属 WS |
| 4 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE RESTRICT | − | idx | 所属 PJ |
| 5 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE RESTRICT | − | idx | 親 WorkItem |
| 6 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE RESTRICT | − | idx | 対象 Repo |
| 7 | `branch` | ブランチ | VARCHAR | 200 | NO | − | − | − | − | − | Git ブランチ名 |
| 8 | `base_branch` | ベースブランチ | VARCHAR | 200 | YES | `NULL` | − | − | − | − | 派生元 |
| 9 | `runtime_id` | ランタイム ID | UUID | YES | `NULL` | − | − | `local_runtime.runtime(id)` (App) | − | idx | 紐付 Runtime |
| 10 | `local_path_reference` | ローカルパス参照 | TEXT | − | YES | `NULL` | − | − | − | − | プラットフォーム不可信（Local Runtime 解释） |
| 11 | `owner_user_id` | オーナ ID | UUID | − | NO | − | − | `identity.user(id)` (App) | − | idx | オーナー |
| 12 | `assigned_agent_id` | 割当エージェント ID | UUID | YES | `NULL` | − | − | `agent.agent(id)` (App) | − | − | 担当 Agent |
| 13 | `current_agent_session_id` | 現セッション ID | UUID | YES | `NULL` | − | − | `agent.agent_session(id)` (App) | − | − | 現実行中 Session |
| 14 | `status` | 状態 | VARCHAR | 32 | NO | `'CREATED'` | − | − | − | idx (PT) | 17 値 |
| 15 | `health` | ヘルス | VARCHAR | 16 | NO | `'Unknown'` | − | − | − | − | 4 値 |
| 16 | `dirty_state` | Dirty 状態 | VARCHAR | 16 | NO | `'CLEAN'` | − | − | − | − | `'CLEAN'` / `'DIRTY'` |
| 17 | `conflict_state` | 衝突状態 | VARCHAR | 16 | NO | `'NONE'` | − | − | − | − | 3 値 |
| 18 | `ahead` | ahead カウント | INT | 4 | NO | `0` | − | − | − | − | ahead commits |
| 19 | `behind` | behind カウント | INT | 4 | NO | `0` | − | − | − | − | behind commits |
| 20 | `changed_files` | 変更ファイル配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | − | GIN | Heatmap / Conflict 用 |
| 21 | `changed_symbols` | 変更シンボル配列 | VARCHAR(512)[] | − | NO | `'{}'` | − | − | − | GIN | Symbol レベル |
| 22 | `test_state` | テスト状態 | JSONB | − | YES | `NULL` | − | − | − | − | `{total, passed, failed, skipped}` |
| 23 | `build_state` | ビルド状態 | VARCHAR | 16 | NO | `'UNKNOWN'` | − | − | − | − | 6 値 |
| 24 | `context_state` | コンテキスト状態 | VARCHAR | 16 | NO | `'NOT_BUILT'` | − | − | − | − | 4 値 |
| 25 | `feedback_state` | フィードバック状態 | JSONB | − | YES | `NULL` | − | − | − | − | `{open_count, critical_count}` |
| 26 | `synchronization_state` | 同期状態 | VARCHAR | 16 | NO | `'UNKNOWN'` | − | − | − | − | 5 値 |
| 27 | `last_activity_at` | 最終活動日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | ハートビート |
| 28 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | − |
| 29 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | − |
| 30 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 31 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `worktree_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_worktree_*` | FOREIGN KEY (5) | `tenant_id` / `workspace_id` / `project_id` / `work_item_id` / `repository_id` | 各親テーブル参照 | ON DELETE CASCADE / RESTRICT |
| `ck_worktree_status` | CHECK | `status` | `IN (17 値, §3.3.1 参照、ON-001 修正済)` | 17 値 |
| `ck_worktree_health` | CHECK | `health` | `IN ('Healthy','Degraded','Unhealthy','Unknown')` | 4 値 |
| `ck_worktree_build_state` | CHECK | `build_state` | `IN ('UNKNOWN','PENDING','RUNNING','PASSED','FAILED','ERRORED')` | 6 値 |
| `ck_worktree_context_state` | CHECK | `context_state` | `IN ('NOT_BUILT','BUILDING','BUILT','STALE')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `worktree_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_worktree_tenant_workitem` | btree (PT) | `(tenant_id, work_item_id)` | `deleted_at IS NULL` | テナント + WorkItem |
| `idx_worktree_tenant_runtime_status` | btree (PT) | `(tenant_id, runtime_id, status)` | `deleted_at IS NULL` | テナント + Runtime + 状態 |
| `idx_worktree_tenant_status_updated` | btree (PT) | `(tenant_id, status, updated_at DESC)` | `deleted_at IS NULL` | ステータス別 |
| `idx_worktree_tenant_owner` | btree (PT) | `(tenant_id, owner_user_id)` | `deleted_at IS NULL` | オーナー別 |
| `idx_worktree_changed_files_gin` | GIN | `changed_files` | `deleted_at IS NULL` | ファイル配列検索 |
| `idx_worktree_changed_symbols_gin` | GIN | `changed_symbols` | `deleted_at IS NULL` | Symbol 配列検索 |
| `idx_worktree_active` | btree (PT) | `(tenant_id, status)` | `deleted_at IS NULL AND status NOT IN ('ABANDONED','ARCHIVED','MERGED')` | アクティブ Worktree |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_worktree_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 3 KB (配列 + JSONB ×3) | 5,000,000 | 約 15 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `workspace.workspace` | `workspace_id` |
| `project.project` | `project_id` |
| `work_item.work_item` | `work_item_id` |
| `scm.repository` | `repository_id` |
| `local_runtime.runtime` (App) | `runtime_id` |
| `identity.user` (App) | `owner_user_id` |
| `agent.agent` (App) | `assigned_agent_id` |
| `agent.agent_session` (App) | `current_agent_session_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `worktree.worktree_status_observed` | `worktree_id` |
| `worktree.worktree_conflict` | `worktree_id` / `other_worktree_id` |
| `worktree.worktree_heatmap` (MV) | `worktree_id` 経由 |
| `development.development_execution` | `worktree_ids` 配列 |
| `development.change_set` | `worktree_id` (App) |

---

## 9. RLS Policy

```sql
ALTER TABLE worktree.worktree ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree.worktree FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_worktree_tenant_isolation ON worktree.worktree
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
