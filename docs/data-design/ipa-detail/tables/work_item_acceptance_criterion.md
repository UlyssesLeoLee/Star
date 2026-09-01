# work_item.acceptance_criterion — テーブル詳細設計書

> **テーブル ID**: T10
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.4.3
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T10 |
| **物理名** | `work_item.acceptance_criterion` |
| **論理名** | 受入基準（Acceptance Criterion、AC） |
| **スキーマ** | `work_item` |
| **Module** | `domain-work-item` |
| **種別** | Weak Entity（`work_item_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WorkItem の受入基準（Given/When/Then 形式）。`coverage_status` は `validation` モジュールが書き込み（COVERED / PARTIAL / UNCOVERED / DISPUTED）。`covered_by_validation_ids` で Validation Result 参照。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 親 WorkItem（弱実体） |
| 4 | `requirement_id` | Requirement ID | UUID | − | YES | `NULL` | − | `work_item.requirement(id)` ON DELETE SET NULL | − | idx | 元 Requirement |
| 5 | `statement` | AC 文 | TEXT | − | NO | − | − | − | − | − | Given/When/Then 形式 |
| 6 | `coverage_status` | カバー状況 | VARCHAR | 32 | NO | `'UNCOVERED'` | − | − | − | − | 4 段階 |
| 7 | `covered_by_validation_ids` | 検証 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | Validation 結果参照 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `acceptance_criterion_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_ac_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_ac_work_item` | FOREIGN KEY | `work_item_id` | `work_item.work_item(id)` | CASCADE | 親削除時 AC 削除 |
| `fk_ac_requirement` | FOREIGN KEY | `requirement_id` | `work_item.requirement(id)` | SET NULL | − |
| `ck_ac_coverage_status` | CHECK | `coverage_status` | `IN ('COVERED','PARTIAL','UNCOVERED','DISPUTED')` | − | 4 段階 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `acceptance_criterion_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_ac_tenant_workitem` | btree (PT) | `(tenant_id, work_item_id)` | `deleted_at IS NULL` | WorkItem 内 AC 検索 |
| `idx_ac_requirement` | btree (PT) | `requirement_id` | `requirement_id IS NOT NULL` | Requirement 別 |
| `idx_ac_validation_ids_gin` | GIN | `covered_by_validation_ids` | − | 配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_acceptance_criterion_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 300,000 |（1 WorkItem 平均 3 AC） |
| 1 年後 | 3,000,000 |
| 3 年後 | 30,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 800 B | 30,000,000 | 約 24 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `work_item.work_item` | `work_item_id` |
| `work_item.requirement` | `requirement_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `validation.acceptance_coverage` | `acceptance_criterion_id` |
| `validation.acceptance_coverage_report` (MV) | `acceptance_criterion_id` 経由 |

---

## 9. RLS Policy

```sql
ALTER TABLE work_item.acceptance_criterion ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_item.acceptance_criterion FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_acceptance_criterion_tenant_isolation ON work_item.acceptance_criterion
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
