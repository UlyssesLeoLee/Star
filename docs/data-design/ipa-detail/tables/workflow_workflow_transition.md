# workflow.workflow_transition — テーブル詳細設計書

> **テーブル ID**: T15
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.5.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T15 |
| **物理名** | `workflow.workflow_transition` |
| **論理名** | ワークフロー遷移 |
| **スキーマ** | `workflow` |
| **Module** | `domain-workflow` |
| **種別** | Weak Entity（`workflow_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Workflow の合法な状態遷移を定義。`from_state_id <> to_state_id` CHECK 強制。`required_permission` で RBAC 校验（例: `work_item:transition` / `work_item:approve`）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `workflow_id` | Workflow ID | UUID | − | NO | − | − | `workflow.workflow_definition(id)` ON DELETE CASCADE | − | idx | 親 Workflow |
| 4 | `from_state_id` | 遷移元 State ID | UUID | − | NO | − | − | `workflow.workflow_state(id)` ON DELETE CASCADE | − | idx | 遷移元 |
| 5 | `to_state_id` | 遷移先 State ID | UUID | − | NO | − | − | `workflow.workflow_state(id)` ON DELETE CASCADE | − | idx | 遷移先 |
| 6 | `required_permission` | 必要パーミッション | VARCHAR | 64 | NO | − | − | − | − | − | RBAC キー |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `workflow_transition_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_transition_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_transition_workflow` | FOREIGN KEY | `workflow_id` | `workflow.workflow_definition(id)` | CASCADE | − |
| `fk_transition_from` | FOREIGN KEY | `from_state_id` | `workflow.workflow_state(id)` | CASCADE | − |
| `fk_transition_to` | FOREIGN KEY | `to_state_id` | `workflow.workflow_state(id)` | CASCADE | − |
| `uq_transition_from_to` | UNIQUE | `(workflow_id, from_state_id, to_state_id, deleted_at)` | − | − | 遷移一意 |
| `ck_workflow_transition_valid` | CHECK | `from_state_id`/`to_state_id` | `from_state_id <> to_state_id` | − | 自己遷移禁止 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `workflow_transition_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_transition_from_to` | btree (UK/PT) | `(workflow_id, from_state_id, to_state_id, deleted_at)` | − | 遷移一意 |
| `idx_transition_tenant_workflow` | btree (PT) | `(tenant_id, workflow_id)` | `deleted_at IS NULL` | テナント + Workflow |
| `idx_transition_from` | btree | `from_state_id` | − | 遷移元別 |
| `idx_transition_to` | btree | `to_state_id` | − | 遷移先別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_workflow_transition_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 30,000 |（1 Workflow = 15 遷移平均） |
| 1 年後 | 300,000 |
| 3 年後 | 3,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 3,000,000 | 約 1.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `workflow.workflow_definition` | `workflow_id` |
| `workflow.workflow_state` | `from_state_id` / `to_state_id` |

### 8.2 被参照元

なし（末端）

---

## 9. RLS Policy

```sql
ALTER TABLE workflow.workflow_transition ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow.workflow_transition FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_workflow_transition_tenant_isolation ON workflow.workflow_transition
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
