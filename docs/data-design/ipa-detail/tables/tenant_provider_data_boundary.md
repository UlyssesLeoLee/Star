# tenant.provider_data_boundary — テーブル詳細設計書

> **テーブル ID**: T03
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.1.3
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T03 | per `00-INVENTORY.md` |
| **物理名** | `tenant.provider_data_boundary` | − |
| **論理名** | プロバイダデータ境界 | AI Provider 単位のデータ送信設定 |
| **スキーマ** | `tenant` | − |
| **Module** | `domain-tenant` | − |
| **種別** | Entity | E |
| **主キー** | `id UUID` | − |
| **R/W 識別** | R/W（SoR） | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **パーティション** | None | − |
| **soft delete** | Yes | `deleted_at` |
| **概要** | AI Provider 単位のデータ送信設定。Provider/Model/Region/送信データ種別 / Retention Policy / Credential 参照 を保持。テナント Policy と Project Policy を参照可能。 | §4.10.2, §R-SEC-003, §R-93 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 | §3.1.2 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | テナント分離キー、RLS 必須 | §7 |
| 3 | `provider_id` | Provider ID | VARCHAR | 64 | NO | − | − | − | − | idx | Provider 一意識別子（`'openai'` / `'anthropic'` / `'google'` / `'azure'` / `'custom'`） | §4.1.3 |
| 4 | `model_id` | Model ID | VARCHAR | 128 | NO | − | − | − | − | idx | Model 一意識別子（`'gpt-4'` / `'claude-opus-4'` 等） | §4.1.3 |
| 5 | `region` | リージョン | VARCHAR | 32 | NO | `'us-east-1'` | − | − | − | − | Provider リージョン | §4.1.3 |
| 6 | `data_sent` | 送信データ種別 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 送信データ種別配列（`["Prompt","Code","Diff","Symbol","Test","BuildLog"]`） | §4.10.2, §R-93 |
| 7 | `retention_policy` | Retention Policy | VARCHAR | 32 | NO | `'N_DAYS_90'` | − | − | − | − | Provider 側データ保持期間 | §4.1.3 / `ck_provider_data_boundary_retention` |
| 8 | `retention_days` | Retention 日数 | INT | 4 | YES | `NULL` | − | − | − | − | `N_DAYS` モード時の保持日数 | §4.1.3 |
| 9 | `credential_ref` | 資格情報参照 | VARCHAR | 255 | NO | − | − | − | − | − | Credential Broker への参照キー（明文保存禁止） | §4.10.8, §28.4 |
| 10 | `tenant_policy_id` | テナント Policy ID | UUID | − | YES | `NULL` | − | `tenant.tenant_policy(id)` ON DELETE SET NULL | − | − | 適用テナント Policy | §4.1.3 |
| 11 | `project_policy_id` | プロジェクト Policy ID | UUID | − | YES | `NULL` | − | (cross-schema, App 側検証) | − | − | 適用プロジェクト Policy（cross-schema FK、App 層検証） | §4.1.3 |
| 12 | `is_active` | 有効フラグ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | PT | Provider 境界の有効 / 無効 | §4.1.3 |
| 13 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード作成日時 | §3.5 |
| 14 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード更新日時 | §3.5 |
| 15 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除日時 | §3.1.5 |
| 16 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック | §3.1.2 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `provider_data_boundary_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_provider_data_boundary_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | テナント削除時 Provider 境界削除 |
| `fk_provider_data_boundary_policy` | FOREIGN KEY | `tenant_policy_id` | `tenant.tenant_policy(id)` | SET NULL | ポリシー削除時は NULL 化 |
| `ck_provider_data_boundary_retention` | CHECK | `retention_policy`/`retention_days` | `retention_policy <> 'N_DAYS' OR retention_days IS NOT NULL` | − | N_DAYS モード整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 (`WHERE`) | 説明 |
|---|---|---|---|---|---|
| `provider_data_boundary_pkey` | btree (PK) | `id` | − | − | 主キー |
| `idx_provider_data_boundary_tenant` | btree | `(tenant_id, provider_id, model_id)` | − | − | Provider + Model 検索 |
| `idx_provider_data_boundary_active` | btree (PT) | `(tenant_id)` | − | `is_active = TRUE` | 有効 Provider のみ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_provider_data_boundary_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at = NOW()` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP | 500 | 100 テナント × 平均 5 Provider |
| 1 年後 | 5,000 | 1,000 テナント × 平均 5 Provider |
| 3 年後 | 50,000 | 10,000 テナント × 平均 5 Provider |

---

## 7. 想定容量

| 1 行バイト（推定） | 想定件数 | 想定容量 | 備考 |
|---|---|---|---|
| 約 1 KB | 50,000 | 約 50 MB | JSONB(data_sent + credential_ref) が大きめ |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `tenant.tenant` | `tenant_id` | N:1 | − |
| `tenant.tenant_policy` | `tenant_policy_id` | N:1 (SET NULL) | − |
| `project.project_policy` | `project_policy_id` (App 検証) | N:1 (cross-schema) | − |

### 8.2 被参照元

なし（末端テーブル）

### 8.3 兄弟・関連

- `tables/tenant_tenant.md`
- `tables/tenant_tenant_policy.md`
- `tables/project_project_policy.md` (cross-schema)

---

## 9. RLS Policy

```sql
ALTER TABLE tenant.provider_data_boundary ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant.provider_data_boundary FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_provider_data_boundary_tenant_isolation ON tenant.provider_data_boundary
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：IPA 標準章立てに整流（16 列 / 4 制約 / 3 INDEX） | per 2026-09-01 15:30 JST Ulysses 拍板 |
