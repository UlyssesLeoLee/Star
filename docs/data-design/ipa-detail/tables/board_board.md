# board.board — テーブル詳細設計書

> **テーブル ID**: T16
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.6.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T16 |
| **物理名** | `board.board` |
| **論理名** | ボード |
| **スキーマ** | `board` |
| **Module** | `domain-board` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Kanban / Scrum 板。`board_type` で 2 種類。WorkItem データモデルは共通（§R-PLAN-003）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | ボード名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 5 | `board_type` | 種別 | VARCHAR | 16 | NO | − | − | − | − | − | `'Kanban'` / `'Scrum'` |
| 6 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 7 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 9 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `board_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_board_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_board_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `ck_board_type` | CHECK | `board_type` | `IN ('Kanban','Scrum')` | − | 2 種別 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `board_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_board_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | PJ 別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_board_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B | 100,000 | 約 40 MB |

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
| `board.board_column` | `board_id` |
| `board.board_swimlane` | `board_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE board.board ENABLE ROW LEVEL SECURITY;
ALTER TABLE board.board FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_board_tenant_isolation ON board.board
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
