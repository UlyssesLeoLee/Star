# workflow.workflow_definition — テーブル詳細設計書

> **テーブル ID**: T13
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.5.1
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T13 |
| **物理名** | `workflow.workflow_definition` |
| **論理名** | ワークフロー定義 |
| **スキーマ** | `workflow` |
| **Module** | `domain-workflow` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Workflow 定義。Project 配下に 1 つ以上の Workflow を持ち、`is_default = TRUE` で 1 件既定。状態と遷移は `workflow_state` / `workflow_transition` に保持。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | ワークフロー名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_workflow_project_name` | 業務表示名 |
| 5 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 6 | `is_default` | 既定フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | ✓ | `uq_workflow_default_per_project` | Project 既定 1 件 |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `workflow_definition_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_workflow_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_workflow_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `uq_workflow_project_name` | UNIQUE | `(project_id, name, deleted_at)` | − | − | 業務名一意 |
| `uq_workflow_default_per_project` | UNIQUE (PT) | `project_id` | `WHERE is_default = TRUE AND deleted_at IS NULL` | − | 既定 1 件保証 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `workflow_definition_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_workflow_project_name` | btree (UK/PT) | `(project_id, name, deleted_at)` | − | 業務名一意 |
| `uq_workflow_default_per_project` | btree (UK/PT) | `project_id` | `is_default = TRUE AND deleted_at IS NULL` | 既定 1 件 |
| `idx_workflow_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_workflow_definition_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 2,000 |（1,000 プロジェクト × 2 Workflow） |
| 1 年後 | 20,000 |
| 3 年後 | 200,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 600 B | 200,000 | 約 120 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `workflow.workflow_state` | `workflow_id` |
| `workflow.workflow_transition` | `workflow_id` |
| `work_item.work_item` | `workflow_id` (V1 候補) |

---

## 9. RLS Policy

```sql
ALTER TABLE workflow.workflow_definition ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow.workflow_definition FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_workflow_definition_tenant_isolation ON workflow.workflow_definition
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
