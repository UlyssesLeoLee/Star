# identity.credential — テーブル詳細設計書

> **テーブル ID**: T43
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.14.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T43 |
| **物理名** | `identity.credential` |
| **論理名** | 資格情報（Credential Broker 抽象） |
| **スキーマ** | `identity` |
| **Module** | `domain-identity` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Credential Broker 抽象。`encrypted_value BYTEA` (pgcrypto PGP 暗号化)。Owner 4 選 1 (user / device / integration / agent) XOR 制約。7 種別。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | YES | `NULL` | − | `identity.user(id)` (App) | − | idx | ユーザ所有 |
| 4 | `device_id` | デバイス ID | UUID | − | YES | `NULL` | − | `identity.device(id)` (App) | − | idx | デバイス所有 |
| 5 | `integration_id` | 統合 ID | UUID | − | YES | `NULL` | − | `integration.integration(id)` (App) | − | idx | 統合所有 |
| 6 | `agent_id` | エージェント ID | UUID | − | YES | `NULL` | − | `agent.agent(id)` (App) | − | idx | Agent 所有 |
| 7 | `credential_type` | 資格情報種別 | VARCHAR | 32 | NO | − | − | − | − | − | 7 種別 |
| 8 | `encrypted_value` | 暗号化値 | BYTEA | − | NO | − | − | − | − | − | PGP 暗号化 |
| 9 | `encryption_key_id` | 暗号化キー ID | VARCHAR | 64 | NO | − | − | − | − | − | KMS / Vault 参照 |
| 10 | `scope` | スコープ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | `{'repos':['STAR/*'], 'permissions':['read','write']}` |
| 11 | `is_active` | 有効フラグ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx | − |
| 12 | `expires_at` | 有効期限 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx | − |
| 13 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 14 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 15 | `last_used_at` | 最終使用 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 使用追跡 |
| 16 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 17 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `credential_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_credential_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `ck_credential_type` | CHECK | `credential_type` | `IN ('password','oauth_token','api_key','mTLS_cert','scm_pat','ai_provider_key','webhook_secret')` | 7 種別 |
| `ck_credential_owner_xor` | CHECK | `user_id`/`device_id`/`integration_id`/`agent_id` | `((user_id IS NOT NULL)::int + (device_id IS NOT NULL)::int + (integration_id IS NOT NULL)::int + (agent_id IS NOT NULL)::int) = 1` | Owner 4 選 1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `credential_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_credential_tenant_owner` | btree (PT) | `(tenant_id, user_id)` | `user_id IS NOT NULL AND deleted_at IS NULL` | ユーザ所有 |
| `idx_credential_tenant_device` | btree (PT) | `(tenant_id, device_id)` | `device_id IS NOT NULL AND deleted_at IS NULL` | デバイス所有 |
| `idx_credential_tenant_integration` | btree (PT) | `(tenant_id, integration_id)` | `integration_id IS NOT NULL AND deleted_at IS NULL` | 統合所有 |
| `idx_credential_tenant_agent` | btree (PT) | `(tenant_id, agent_id)` | `agent_id IS NOT NULL AND deleted_at IS NULL` | Agent 所有 |
| `idx_credential_expires` | btree (PT) | `(expires_at)` | `expires_at IS NOT NULL AND is_active = TRUE AND deleted_at IS NULL` | 期限監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_credential_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB | 5,000,000 | 約 5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 (App 検証) |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` | `user_id` |
| `identity.device` | `device_id` |
| `integration.integration` | `integration_id` |
| `agent.agent` | `agent_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `identity.device_binding` (Credential Broker 経由) | `credential_id` |
| `integration.integration.credential_ref` | − |
| `scm.repository.credential_id` | − |

---

## 9. RLS Policy

```sql
ALTER TABLE identity.credential ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.credential FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_credential_tenant_isolation ON identity.credential
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
