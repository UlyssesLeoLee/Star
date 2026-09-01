# board.board_column — テーブル詳細設計書

> **テーブル ID**: T17
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.6.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T17 |
| **物理名** | `board.board_column` |
| **論理名** | ボードカラム |
| **スキーマ** | `board` |
| **Module** | `domain-board` |
| **種別** | Weak Entity（`board_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Board の列。`order_index` で表示順制御。`state_id` で `workflow.workflow_state` 紐付け（cross-schema、App 層検証）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `board_id` | Board ID | UUID | − | NO | − | − | `board.board(id)` ON DELETE CASCADE | − | idx | 親 Board |
| 4 | `state_id` | State ID | UUID | − | NO | − | − | (cross-schema, App 検証) | − | idx | 紐付 Workflow State |
| 5 | `name` | カラム名 | VARCHAR | 64 | NO | − | − | − | − | − | 業務表示名 |
| 6 | `order_index` | 並び順 | INT | 4 | NO | − | − | − | ✓ | `uq_column_board_order` | 表示順制御 |
| 7 | `wip_limit` | WIP 上限 | INT | 4 | YES | `NULL` | − | − | − | − | Work In Progress 上限（ON-103 派生） |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `board_column_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_column_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_column_board` | FOREIGN KEY | `board_id` | `board.board(id)` | CASCADE | 親 Board 削除時 Column 削除 |
| `uq_column_board_order` | UNIQUE | `(board_id, order_index, deleted_at)` | − | − | Board 内並び順一意 |
| `ck_board_column_wip_limit` | CHECK | `wip_limit` | `wip_limit IS NULL OR wip_limit > 0` | − | WIP 0 以下禁止 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `board_column_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_column_board_order` | btree (UK/PT) | `(board_id, order_index, deleted_at)` | − | 並び順一意 |
| `idx_column_tenant_board` | btree (PT) | `(tenant_id, board_id)` | `deleted_at IS NULL` | テナント + Board |
| `idx_column_state` | btree | `state_id` | − | State 別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_board_column_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 |（1 Board = 5 Column 平均） |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 350 B | 500,000 | 約 175 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `board.board` | `board_id` |
| `workflow.workflow_state` | `state_id` (App 検証) |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE board.board_column ENABLE ROW LEVEL SECURITY;
ALTER TABLE board.board_column FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_board_column_tenant_isolation ON board.board_column
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
