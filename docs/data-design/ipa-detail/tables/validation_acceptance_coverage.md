# validation.acceptance_coverage — テーブル詳細設計書

> **テーブル ID**: T92
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.24.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T92 |
| **物理名** | `validation.acceptance_coverage` |
| **論理名** | 受入カバレッジ |
| **スキーマ** | `validation` |
| **Module** | `domain-validation` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | AC → ValidationEvidence マッピング。**4 状態**（COVERED / PARTIAL / UNCOVERED / DISPUTED、§4.24.3）。`human_acknowledged_*` で人間確認追跡。AC 1:1 UK 制御。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `acceptance_criterion_id` | AC ID | UUID | − | NO | − | − | `work_item.acceptance_criterion(id)` ON DELETE CASCADE | ✓ | `uq_acceptance_coverage_per_ac` | 親 AC |
| 4 | `validation_result_ids` | 検証結果 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 検証連動 |
| 5 | `review_finding_ids` | レビュー所見 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | レビュー連動 |
| 6 | `human_acknowledged_by` | 人間確認者 ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | − | 人間確認者 |
| 7 | `human_acknowledged_at` | 人間確認日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 人間確認時刻 |
| 8 | `coverage_status` | カバー状況 | VARCHAR | 16 | NO | `'UNCOVERED'` | − | − | − | idx (PT) | 4 値 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `acceptance_coverage_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_acceptance_coverage_*` | FOREIGN KEY (2) | `tenant_id` / `acceptance_criterion_id` | 各親テーブル | CASCADE | − |
| `uq_acceptance_coverage_per_ac` | UNIQUE | `(acceptance_criterion_id, deleted_at)` | − | − | AC 1:1 |
| `ck_acceptance_coverage_status` | CHECK | `coverage_status` | `IN ('COVERED','PARTIAL','UNCOVERED','DISPUTED')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `acceptance_coverage_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_acceptance_coverage_per_ac` | btree (UK/PT) | `(acceptance_criterion_id, deleted_at)` | − | AC 1:1 |
| `idx_acceptance_coverage_tenant_status` | btree (PT) | `(tenant_id, coverage_status)` | `deleted_at IS NULL` | ステータス別 |
| `idx_acceptance_coverage_validation_ids_gin` | GIN | `validation_result_ids` | − | 検証配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_acceptance_coverage_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 300,000 |（AC と 1:1） |
| 1 年後 | 3,000,000 |
| 3 年後 | 30,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 30,000,000 | 約 15 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `work_item.acceptance_criterion` | `acceptance_criterion_id` |
| `identity.user` (App) | `human_acknowledged_by` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `validation.acceptance_coverage_report` (MV) | `acceptance_criterion_id` 経由 |

---

## 9. RLS Policy

```sql
ALTER TABLE validation.acceptance_coverage ENABLE ROW LEVEL SECURITY;
ALTER TABLE validation.acceptance_coverage FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_acceptance_coverage_tenant_isolation ON validation.acceptance_coverage
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
