# work_item.requirement — テーブル詳細設計書

> **テーブル ID**: T09
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.4.2
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T09 |
| **物理名** | `work_item.requirement` |
| **論理名** | 業務 Requirement |
| **スキーマ** | `work_item` |
| **Module** | `domain-work-item` |
| **種別** | Entity（業務 Requirement 独立管理） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 業務 Requirement。複数の WorkItem に関連付け可能（`linked_work_item_ids` 配列）。`business_goal_id` で業務目標と紐付け。§4.9 / §R-39 Traceability。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `business_goal_id` | 業務目標 ID | UUID | − | YES | `NULL` | − | (cross-schema App 検証) | − | idx | 業務目標紐付け |
| 4 | `statement` | 要求文 | TEXT | − | NO | − | − | − | ✓ | `uq_requirement_tenant_statement` | 业务要求文（論理削除考慮 UK） |
| 5 | `rationale` | 根拠 | TEXT | − | YES | `NULL` | − | − | − | − | 业务根拠 |
| 6 | `linked_work_item_ids` | 関連 WorkItem ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 複数 WorkItem 紐付け |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `requirement_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_requirement_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `uq_requirement_tenant_statement` | UNIQUE | `(tenant_id, statement, deleted_at)` | − | − | 要求文一意（論理削除考慮） |

> **business_goal_id** は cross-schema FK のため DB 制約外、App 層検証（per §4.1.3 派生制約）

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `requirement_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_requirement_tenant_statement` | btree (UK/PT) | `(tenant_id, statement, deleted_at)` | − | 要求文一意 |
| `idx_requirement_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_requirement_business_goal` | btree (PT) | `business_goal_id` | `business_goal_id IS NOT NULL` | 業務目標別 |
| `idx_requirement_work_items_gin` | GIN | `linked_work_item_ids` | − | 配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_requirement_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000 |
| 1 年後 | 100,000 |
| 3 年後 | 1,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (text×2 + uuid array) | 1,000,000 | 約 1.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `work_item.business_goal` | `business_goal_id` (App 検証) |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `work_item.acceptance_criterion` | `requirement_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE work_item.requirement ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_item.requirement FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_requirement_tenant_isolation ON work_item.requirement
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
