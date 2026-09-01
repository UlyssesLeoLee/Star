# tenant.tenant_policy — テーブル詳細設計書

> **テーブル ID**: T02
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.1.2
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T02 | per `00-INVENTORY.md` |
| **物理名** | `tenant.tenant_policy` | − |
| **論理名** | テナントポリシー | AI データ境界 6 維 Policy |
| **スキーマ** | `tenant` | − |
| **Module** | `domain-tenant` | − |
| **種別** | Entity | E |
| **主キー** | `id UUID` | − |
| **R/W 識別** | R/W（SoR） | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **パーティション** | None | − |
| **soft delete** | Yes | `deleted_at` |
| **概要** | テナント級 AI データ境界 Policy。6 維互斥 / 組合せで AI Provider への送信可否を制御。`is_default = TRUE` の Policy は各テナントに 1 件のみ。 | §4.10.5, §R-SEC-002, §R-92 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 | §3.1.2 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | テナント分離キー、RLS 必須 | §7 |
| 3 | `cloud_ai_allowed` | クラウド AI 許可 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | クラウド AI 利用可否 | §4.1.2 / `ck_policy_xor` 連動 |
| 4 | `cloud_ai_restricted` | クラウド AI 制限 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 特定 Provider のみ許可（許可リスト利用） | §4.1.2 / `ck_policy_specific` 連動 |
| 5 | `local_ai_only` | ローカル AI のみ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | ローカル AI のみ（既定、保守） | §4.1.2 / `ck_policy_xor` 連動 |
| 6 | `specific_provider_allowed` | 特定 Provider 許可 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 許可 Provider 配列、例: `["openai","anthropic"]` | §4.1.2 |
| 7 | `no_code_upload` | コードアップロード禁止 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | コードの AI 送信を禁止 | §4.10.5 |
| 8 | `metadata_only` | メタデータのみ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | メタデータのみ送信、本文送信禁止 | §4.10.5 |
| 9 | `effective_from` | 有効開始日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | Policy 有効開始日時（時点管理） | §4.1.2 |
| 10 | `effective_to` | 有効終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | Policy 有効終了日時、NULL = 永久 | §4.1.2 |
| 11 | `is_default` | 既定 Policy | BOOLEAN | 1 | NO | `FALSE` | − | − | ✓ | `uq_tenant_policy_default` | 既定 Policy フラグ、テナント内 1 件のみ | §4.1.2 |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード作成日時 | §3.5 |
| 13 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード更新日時 | §3.5 |
| 14 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除日時 | §3.1.5 |
| 15 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック | §3.1.2 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `tenant_policy_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_tenant_policy_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | テナント削除時ポリシー削除 |
| `uq_tenant_policy_default` | UNIQUE | `tenant_id` (partial) | `WHERE is_default = TRUE AND deleted_at IS NULL` | − | 既定 Policy 1 件保証 |
| `ck_policy_xor` | CHECK | `cloud_ai_allowed`/`local_ai_only` | `(cloud_ai_allowed = FALSE AND local_ai_only = TRUE) OR (cloud_ai_allowed = TRUE AND local_ai_only = FALSE)` | − | クラウド AI とローカル AI 排他 |
| `ck_policy_specific` | CHECK | `specific_provider_allowed`/`cloud_ai_restricted` | `(specific_provider_allowed <> '[]'::jsonb) OR (cloud_ai_restricted = FALSE)` | − | 特定 Provider 許可整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 (`WHERE`) | 説明 |
|---|---|---|---|---|---|
| `tenant_policy_pkey` | btree (PK) | `id` | − | − | 主キー |
| `uq_tenant_policy_default` | btree (UK/PT) | `tenant_id` | − | `is_default = TRUE AND deleted_at IS NULL` | 既定 Policy 1 件保証 |
| `idx_tenant_policy_tenant_effective` | btree (PT) | `(tenant_id, effective_from DESC)` | − | `deleted_at IS NULL` | テナント + 有効期間降順 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_tenant_policy_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at = NOW()` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP | 100 | 100 テナント × 1 既定 Policy + 数件履歴 = 200-500 件 |
| 1 年後 | 1,000 | 同上 |
| 3 年後 | 10,000 | 同上 |

---

## 7. 想定容量

| 1 行バイト（推定） | 想定件数 | 想定容量 | 備考 |
|---|---|---|---|
| 約 600 B | 10,000 | 約 6 MB | UUID(16) + bool×5 + JSONB(変動) + TIMESTAMPTZ(8×3) + INT(4) + 28B overhead |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `tenant.tenant` | `tenant_id` | N:1 | N ポリシー : 1 テナント |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `tenant.provider_data_boundary` | `tenant_policy_id` | 1:N (SET NULL) | 1 ポリシー : N プロバイダ境界 |

### 8.3 兄弟・関連

- `tables/tenant_tenant.md` — 親テーブル
- `tables/tenant_provider_data_boundary.md` — `tenant_policy_id` 経由

---

## 9. RLS Policy

```sql
ALTER TABLE tenant.tenant_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant.tenant_policy FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_tenant_policy_tenant_isolation ON tenant.tenant_policy
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);

CREATE POLICY policy_tenant_policy_insert ON tenant.tenant_policy
  FOR INSERT
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);

CREATE POLICY policy_tenant_policy_update ON tenant.tenant_policy
  FOR UPDATE
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);

CREATE POLICY policy_tenant_policy_delete ON tenant.tenant_policy
  FOR DELETE
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：IPA 標準章立てに整流（15 列 / 5 制約 / 3 INDEX） | per 2026-09-01 15:30 JST Ulysses 拍板 |
