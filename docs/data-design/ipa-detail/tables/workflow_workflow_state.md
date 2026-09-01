# workflow.workflow_state — テーブル詳細設計書

> **テーブル ID**: T14
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.5.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T14 |
| **物理名** | `workflow.workflow_state` |
| **論理名** | ワークフロー状態 |
| **スキーマ** | `workflow` |
| **Module** | `domain-workflow` |
| **種別** | Weak Entity（`workflow_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Workflow 内の State 集合。1 initial + 0..N terminal + N 通常。`category` は `work_item.work_item_status` の分類を継承。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `workflow_id` | Workflow ID | UUID | − | NO | − | − | `workflow.workflow_definition(id)` ON DELETE CASCADE | − | idx | 親 Workflow |
| 4 | `name` | 状態名 | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_state_workflow_name` | 業務表示名 |
| 5 | `is_initial` | 初期状態 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 入口状態 |
| 6 | `is_terminal` | 終端状態 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 出口状態 |
| 7 | `category` | カテゴリ | VARCHAR | 32 | NO | `'TODO'` | − | − | − | − | `work_item.work_item_status` 連動 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `workflow_state_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_state_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_state_workflow` | FOREIGN KEY | `workflow_id` | `workflow.workflow_definition(id)` | CASCADE | 親削除時 State 削除 |
| `uq_state_workflow_name` | UNIQUE | `(workflow_id, name, deleted_at)` | − | − | Workflow 内 State 名一意 |
| `ck_workflow_state_terminal` | CHECK | `is_initial`/`is_terminal` | `NOT (is_initial = TRUE AND is_terminal = TRUE)` | − | 排他制約 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `workflow_state_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_state_workflow_name` | btree (UK/PT) | `(workflow_id, name, deleted_at)` | − | 業務名一意 |
| `idx_state_tenant_workflow` | btree (PT) | `(tenant_id, workflow_id)` | `deleted_at IS NULL` | テナント + Workflow |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_workflow_state_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000 |（1 Workflow = 5 State 平均） |
| 1 年後 | 100,000 |
| 3 年後 | 1,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B | 1,000,000 | 約 400 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `workflow.workflow_definition` | `workflow_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `workflow.workflow_transition` | `from_state_id` / `to_state_id` |
| `board.board_column` | `state_id` (App 検証) |
| `work_item.work_item` | `current_state_id` (V1 候補) |

---

## 9. RLS Policy

```sql
ALTER TABLE workflow.workflow_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow.workflow_state FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_workflow_state_tenant_isolation ON workflow.workflow_state
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
