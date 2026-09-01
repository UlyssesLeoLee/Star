# work_item.business_goal — テーブル詳細設計書

> **テーブル ID**: T11
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.4.4
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T11 |
| **物理名** | `work_item.business_goal` |
| **論理名** | 業務目標 |
| **スキーマ** | `work_item` |
| **Module** | `domain-work-item` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 業務目標。Requirement → BusinessGoal の階層で業務→要件→実装の Traceability を確立。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `statement` | 目標文 | TEXT | − | NO | − | − | − | − | − | 業務目標文 |
| 4 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 詳細説明 |
| 5 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 6 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 7 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 8 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `business_goal_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_business_goal_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `business_goal_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_business_goal_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_business_goal_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 500 B | 100,000 | 約 50 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `work_item.requirement` | `business_goal_id` (App 検証) |
| `planning.roadmap` | `business_goal_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE work_item.business_goal ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_item.business_goal FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_business_goal_tenant_isolation ON work_item.business_goal
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
