# identity.device — テーブル詳細設計書

> **テーブル ID**: T41
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.14.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T41 |
| **物理名** | `identity.device` |
| **論理名** | デバイス |
| **スキーマ** | `identity` |
| **Module** | `domain-identity` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | デバイス。Local Runtime mTLS Cert 紐付け。`device_kind` 5 値（web / cli / ide_plugin / local_daemon / mobile）。`device_fingerprint` でデバイス識別。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE CASCADE | − | idx | 所有者 |
| 4 | `device_name` | デバイス名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 5 | `device_kind` | デバイス種別 | VARCHAR | 32 | NO | − | − | − | − | − | 5 値 |
| 6 | `device_fingerprint` | デバイス指紋 | VARCHAR | 2048 | NO | − | − | − | ✓ | `uq_device_tenant_fingerprint` | デバイス識別子 |
| 7 | `public_key` | 公開鍵 | TEXT | − | YES | `NULL` | − | − | − | − | mTLS PEM |
| 8 | `cert_serial` | Cert シリアル | VARCHAR | 128 | YES | `NULL` | − | − | − | − | mTLS Cert |
| 9 | `cert_expires_at` | Cert 有効期限 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx | Cert 期限 |
| 10 | `status` | 状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | idx | 3 状態 |
| 11 | `is_revoked` | 失効フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 即時失効 |
| 12 | `revoked_at` | 失効日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 13 | `revoked_reason` | 失効理由 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 14 | `last_seen_at` | 最終確認日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx | 接続追跡 |
| 15 | `ip_addresses` | IP アドレス配列 | INET[] | − | YES | `NULL` | − | − | − | − | 過去 IP 履歴 |
| 16 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 17 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 18 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 19 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `device_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_device_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_device_user` | FOREIGN KEY | `user_id` | `identity.user(id)` | CASCADE | ユーザ削除時 デバイス削除 |
| `uq_device_tenant_fingerprint` | UNIQUE | `(tenant_id, device_fingerprint)` | − | − | デバイス一意 |
| `ck_device_kind` | CHECK | `device_kind` | `IN ('web','cli','ide_plugin','local_daemon','mobile')` | − | 5 値 |
| `ck_device_status` | CHECK | `status` | `IN ('ACTIVE','REVOKED','EXPIRED')` | − | 3 状態 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `device_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_device_tenant_fingerprint` | btree (UK) | `(tenant_id, device_fingerprint)` | − | デバイス一意 |
| `idx_device_tenant_user` | btree (PT) | `(tenant_id, user_id)` | `deleted_at IS NULL` | テナント + ユーザ |
| `idx_device_tenant_status` | btree (PT) | `(tenant_id, status)` | `deleted_at IS NULL` | ステータス別 |
| `idx_device_cert_expires` | btree (PT) | `(cert_expires_at)` | `cert_expires_at IS NOT NULL AND status = 'ACTIVE'` | Cert 期限監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_device_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 20,000 |
| 1 年後 | 200,000 |
| 3 年後 | 2,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB (公開鍵 + IP配列) | 2,000,000 | 約 4 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` | `user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `identity.device_binding` | `device_id` |
| `local_runtime.runtime` | `device_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE identity.device ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.device FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_device_tenant_isolation ON identity.device
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
