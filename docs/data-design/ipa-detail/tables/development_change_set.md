# development.change_set — テーブル詳細設計書

> **テーブル ID**: T64
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T64 |
| **物理名** | `development.change_set` |
| **論理名** | 変更セット（核心） |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ChangeSet。5 種 集計列（files_added/modified/deleted/renamed/generated）、2 統計列（added/deleted_lines）、1 diff 全文 Object Storage 参照（§5.1 派生）。`worktree_id` 経由で Worktree 紐付け。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `worktree_id` | ワークツリー ID | UUID | − | NO | − | − | (App 検証) | − | idx | 親 Worktree |
| 5 | `agent_session_id` | エージェントセッション ID | UUID | YES | `NULL` | − | − | `agent.agent_session(id)` (App) | − | idx | 関連 Session |
| 6 | `commit_id` | コミット ID | UUID | YES | `NULL` | − | − | `scm.commit(id)` (App) | − | idx | 関連 Commit |
| 7 | `files_added` | 追加ファイル数 | INT | 4 | NO | `0` | − | − | − | − | 集計 |
| 8 | `files_modified` | 変更ファイル数 | INT | 4 | NO | `0` | − | − | − | − | 集計 |
| 9 | `files_deleted` | 削除ファイル数 | INT | 4 | NO | `0` | − | − | − | − | 集計 |
| 10 | `files_renamed` | リネームファイル数 | INT | 4 | NO | `0` | − | − | − | − | 集計 |
| 11 | `files_generated` | 生成ファイル数 | INT | 4 | NO | `0` | − | − | − | − | 集計 |
| 12 | `added_lines` | 追加行数 | INT | 4 | NO | `0` | − | − | − | − | 統計 |
| 13 | `deleted_lines` | 削除行数 | INT | 4 | NO | `0` | − | − | − | − | 統計 |
| 14 | `diff_reference` | Diff 参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key |
| 15 | `diff_size_bytes` | Diff サイズ | BIGINT | 8 | YES | `NULL` | − | − | − | − | バイトサイズ |
| 16 | `symbol_count` | シンボル数 | INT | 4 | NO | `0` | − | − | − | − | Symbol 統計 |
| 17 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | 集計基準時刻 |
| 18 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 19 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 20 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `change_set_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_change_set_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_change_set_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |

> **注**: `worktree_id` / `agent_session_id` / `commit_id` の FK は data-design §4.19.2 で未実装、§00-INVENTORY.md 派生

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `change_set_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_change_set_tenant_worktree` | btree (PT) | `(tenant_id, worktree_id)` | `deleted_at IS NULL` | テナント + Worktree |
| `idx_change_set_tenant_agent_session` | btree (PT) | `(tenant_id, agent_session_id)` | `agent_session_id IS NOT NULL` | テナント + Agent Session |
| `idx_change_set_tenant_commit` | btree (PT) | `(tenant_id, commit_id)` | `commit_id IS NOT NULL` | テナント + Commit |
| `idx_change_set_tenant_project_created` | btree (PT) | `(tenant_id, project_id, created_at DESC)` | `deleted_at IS NULL` | PJ + 作成順 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_change_set_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB | 100,000,000 | 約 120 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `worktree.worktree` (App 検証) | `worktree_id` |
| `agent.agent_session` (App 検証) | `agent_session_id` |
| `scm.commit` (App 検証) | `commit_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `development.file_change` | `change_set_id` |
| `development.symbol_change` | `change_set_id` |
| `development.risk_signal` | `change_set_id` |
| `development.change_set_link` | `source_change_set_id` / `target_change_set_id` |
| `development.development_context` (配列) | `change_set_id` 経由 |

---

## 9. RLS Policy

```sql
ALTER TABLE development.change_set ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.change_set FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_change_set_tenant_isolation ON development.change_set
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
