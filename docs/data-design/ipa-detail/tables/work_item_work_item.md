# work_item.work_item — テーブル詳細設計書

> **テーブル ID**: T08
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.4.1
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T08 | per `00-INVENTORY.md` |
| **物理名** | `work_item.work_item` | − |
| **論理名** | ワークアイテム（核心） | 6 種別 × 10 状態 |
| **スキーマ** | `work_item` | − |
| **Module** | `domain-work-item` | − |
| **種別** | Entity（核心聚合根） | E |
| **主キー** | `id UUID` | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **概要** | WorkItem 核心。6 種別（Epic/Story/Task/Bug/Subtask/AITask）× 10 状態。`key` は `STAR-100` 形式でプロジェクト業務キー。`parent_work_item_id` で再帰階層（Epic → Story → Subtask）。 | §4.4.1 / §4.9, §R-8, §R-41.2 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `workspace_id` | ワークスペース ID | UUID | − | NO | − | − | `workspace.workspace(id)` ON DELETE RESTRICT | − | idx | 所属 WS |
| 4 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE RESTRICT | − | idx | 所属 PJ |
| 5 | `type` | 種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'Epic'` / `'Story'` / `'Task'` / `'Bug'` / `'Subtask'` / `'AITask'` |
| 6 | `status` | 状態 | VARCHAR | 32 | NO | `'TODO'` | − | − | − | idx | 10 状態（既定 3 + 拡張 7） |
| 7 | `key` | 業務キー | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_work_item_tenant_key` | `STAR-100` 形式（Project Key + 連番） |
| 8 | `title` | タイトル | VARCHAR | 500 | NO | − | − | − | − | − | 業務表示名 |
| 9 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 業務説明（リッチテキスト） |
| 10 | `assignee_user_id` | 担当ユーザ ID | UUID | − | YES | `NULL` | − | `identity.user(id)` ON DELETE SET NULL | − | idx | 人間担当 |
| 11 | `assignee_agent_id` | 担当エージェント ID | UUID | − | YES | `NULL` | − | `agent.agent(id)` ON DELETE SET NULL | − | − | AI 担当 |
| 12 | `reporter_user_id` | 起票者 ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | − | 報告者 |
| 13 | `priority` | 優先度 | VARCHAR | 8 | NO | `'P3'` | − | − | − | − | `'P0'` / `'P1'` / `'P2'` / `'P3'` |
| 14 | `severity` | 重大度 | VARCHAR | 8 | YES | `NULL` | − | − | − | − | Bug 専用、5 段階 |
| 15 | `story_points` | ストーリーポイント | INT | 4 | YES | `NULL` | − | − | − | − | Scrum 想定工数 |
| 16 | `parent_work_item_id` | 親 WorkItem ID | UUID | − | YES | `NULL` | − | `work_item.work_item(id)` ON DELETE SET NULL | − | idx | 再帰階層（Subtask → Story → Epic） |
| 17 | `sprint_id` | スプリント ID | UUID | − | YES | `NULL` | − | `planning.sprint(id)` ON DELETE SET NULL | − | idx | 所属スプリント |
| 18 | `repository_ids` | リポジトリ ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 関連リポジトリ 0..N |
| 19 | `worktree_ids` | ワークツリー ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 関連ワークツリー 0..N（冗長、主源は `worktree.worktree.work_item_id`） |
| 20 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 21 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 22 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | − |
| 23 | `due_date` | 期限 | DATE | 4 | YES | `NULL` | − | − | − | − | 業務期限 |
| 24 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `work_item_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_work_item_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_work_item_workspace` | FOREIGN KEY | `workspace_id` | `workspace.workspace(id)` | RESTRICT | − |
| `fk_work_item_project` | FOREIGN KEY | `project_id` | `project.project(id)` | RESTRICT | − |
| `fk_work_item_assignee_user` | FOREIGN KEY | `assignee_user_id` | `identity.user(id)` | SET NULL | − |
| `fk_work_item_assignee_agent` | FOREIGN KEY | `assignee_agent_id` | `agent.agent(id)` | SET NULL | − |
| `fk_work_item_reporter` | FOREIGN KEY | `reporter_user_id` | `identity.user(id)` | RESTRICT | − |
| `fk_work_item_parent` | FOREIGN KEY | `parent_work_item_id` | `work_item.work_item(id)` | SET NULL | 自己参照 |
| `fk_work_item_sprint` | FOREIGN KEY | `sprint_id` | `planning.sprint(id)` | SET NULL | − |
| `uq_work_item_tenant_key` | UNIQUE | `(tenant_id, project_id, key, deleted_at)` | − | − | 業務キー一意（論理削除考慮） |
| `ck_work_item_type` | CHECK | `type` | `IN ('Epic','Story','Task','Bug','Subtask','AITask')` | − | 6 種別 |
| `ck_work_item_status` | CHECK | `status` | `IN ('TODO','IN_PROGRESS','DONE','IN_REVIEW','BLOCKED','CANCELLED','IN_TESTING','READY_FOR_DEPLOY','NEEDS_INFO')` | − | 10 状態（既定 3 + 拡張 7） |
| `ck_work_item_priority` | CHECK | `priority` | `IN ('P0','P1','P2','P3')` | − | 4 段階 |
| `ck_work_item_severity` | CHECK | `severity` | `severity IS NULL OR severity IN ('P0','P1','P2','P3')` | − | Bug 専用 NULL 許容 |
| `ck_work_item_subtask_parent` | CHECK | `type`/`parent_work_item_id` | `(type = 'Subtask' AND parent_work_item_id IS NOT NULL) OR (type != 'Subtask')` | − | Subtask 必須親参照 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `work_item_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_work_item_tenant_key` | btree (UK/PT) | `(tenant_id, project_id, key, deleted_at)` | − | 業務キー一意 |
| `idx_work_item_tenant_project_status` | btree (PT) | `(tenant_id, project_id, status)` | `deleted_at IS NULL` | 高頻検索 |
| `idx_work_item_tenant_assignee_status` | btree (PT) | `(tenant_id, assignee_user_id, status)` | `deleted_at IS NULL AND assignee_user_id IS NOT NULL` | 担当別 |
| `idx_work_item_tenant_updated` | btree (PT) | `(tenant_id, updated_at DESC)` | `deleted_at IS NULL` | 最近の更新 |
| `idx_work_item_parent` | btree (PT) | `parent_work_item_id` | `parent_work_item_id IS NOT NULL` | 子階層の検索 |
| `idx_work_item_sprint` | btree (PT) | `sprint_id` | `sprint_id IS NOT NULL` | スプリント別 |
| `idx_work_item_repository_ids_gin` | GIN | `repository_ids` | `deleted_at IS NULL` | リポジトリ配列検索 |
| `idx_work_item_worktree_ids_gin` | GIN | `worktree_ids` | `deleted_at IS NULL` | ワークツリー配列検索 |
| `idx_work_item_active` | btree (PT) | `id` | `deleted_at IS NULL` | 存活レコードフィルタ |

> **ON-201 推奨追加**: `(tenant_id, project_id, due_date)` 部分 — 期限順ボード表示用（次升版で追加）

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_work_item_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP | 100,000 | 1,000 プロジェクト × 100 WorkItem |
| 1 年後 | 1,000,000 | 10,000 × 100 |
| 3 年後 | 10,000,000 | 100,000 × 100 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB (title 500 + text + uuid×8 + ts×3 + 配列×2) | 10,000,000 | 約 20 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `workspace.workspace` | `workspace_id` |
| `project.project` | `project_id` |
| `identity.user` | `assignee_user_id` / `reporter_user_id` |
| `agent.agent` | `assignee_agent_id` |
| `work_item.work_item` (self) | `parent_work_item_id` |
| `planning.sprint` | `sprint_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `work_item.requirement` | (linked_work_item_ids 配列) |
| `work_item.acceptance_criterion` | `work_item_id` |
| `validation.acceptance_coverage` | `work_item_id` |
| `worktree.worktree` | `work_item_id` |
| `feedback.feedback` | `work_item_id` |
| `relation.relation` | source_id / target_id (動的) |

### 8.3 兄弟・関連

- `tables/work_item_requirement.md` — 弱実体（Requirement）
- `tables/work_item_acceptance_criterion.md` — 弱実体（AC）
- `tables/work_item_business_goal.md` — 業務目標
- `tables/work_item_work_item_status.md` — 状態 Lookup

---

## 9. RLS Policy

```sql
ALTER TABLE work_item.work_item ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_item.work_item FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_work_item_tenant_isolation ON work_item.work_item
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：24 列 / 15 制約 / 10 INDEX | per 2026-09-01 15:30 JST Ulysses 拍板 |
