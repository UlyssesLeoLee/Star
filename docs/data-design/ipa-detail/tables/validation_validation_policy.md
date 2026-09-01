# validation.validation_policy — テーブル詳細設計書

> **テーブル ID**: T93
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.24.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T93 |
| **物理名** | `validation.validation_policy` |
| **論理名** | 検証ポリシー |
| **スキーマ** | `validation` |
| **Module** | `domain-validation` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Validation Policy テンプレート。`required_kinds JSONB`（既定: Build / UnitTest / Lint / Format / AcceptanceCheck）+ `optional_kinds JSONB`（5 種）+ `pass_thresholds JSONB`（カバレッジ閾値等）。**`allow_ai_self_claim` 既定 false（VAL-001 強约束）**。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | ポリシー名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_validation_policy_project_name` | 業務表示名 |
| 5 | `required_kinds` | 必須種別 | JSONB | − | NO | `'["Build","UnitTest","Lint","Format","AcceptanceCheck"]'::jsonb` | − | − | − | − | 5 種必須 |
| 6 | `optional_kinds` | 任意種別 | JSONB | − | NO | `'["IntegrationTest","StaticAnalysis","SecurityCheck","Review","CustomValidation"]'::jsonb` | − | − | − | − | 5 種任意 |
| 7 | `pass_thresholds` | 合格閾値 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | `{'unit_test_coverage': 0.80, 'lint_warnings': 0}` |
| 8 | `allow_ai_self_claim` | AI 自報許可 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | **VAL-001 強约束：既定 false** |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `validation_policy_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_validation_policy_*` | FOREIGN KEY (2) | `tenant_id` / `project_id` | 各親テーブル | CASCADE | − |
| `uq_validation_policy_project_name` | UNIQUE | `(project_id, name, deleted_at)` | − | − | PJ 内業務名一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `validation_policy_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_validation_policy_project_name` | btree (UK/PT) | `(project_id, name, deleted_at)` | − | PJ 内業務名一意 |
| `idx_validation_policy_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_validation_policy_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |（1 PJ 1 Policy） |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB (JSONB ×3) | 100,000 | 約 100 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE validation.validation_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE validation.validation_policy FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_validation_policy_tenant_isolation ON validation.validation_policy
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
