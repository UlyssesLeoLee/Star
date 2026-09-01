# identity.user — テーブル詳細設計書

> **テーブル ID**: T40
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.14.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T40 |
| **物理名** | `identity.user` |
| **論理名** | ユーザ |
| **スキーマ** | `identity` |
| **Module** | `domain-identity` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ユーザ。`email` は citext（大文字小区別無視）、OAuth 連動可、password_hash は bcrypt / argon2id。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `email` | メール | CITEXT | 320 | NO | − | − | − | ✓ | `uq_user_tenant_email` | citext、大文字小区別無視 |
| 4 | `display_name` | 表示名 | VARCHAR | 200 | NO | − | − | − | − | idx | 業務表示名 |
| 5 | `avatar_url` | アバター URL | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | アバター画像 |
| 6 | `status` | 状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | idx | 4 状態 |
| 7 | `password_hash` | パスワードハッシュ | VARCHAR | 255 | YES | `NULL` | − | − | − | − | bcrypt / argon2id |
| 8 | `mfa_secret` | MFA シークレット | VARCHAR | 255 | YES | `NULL` | − | − | − | − | TOTP secret |
| 9 | `mfa_enabled` | MFA 有効 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | MFA フラグ |
| 10 | `oauth_provider` | OAuth Provider | VARCHAR | 32 | YES | `NULL` | − | − | − | idx | `'github'` / `'gitlab'` / `'google'` |
| 11 | `oauth_subject` | OAuth Subject | VARCHAR | 255 | YES | `NULL` | − | − | − | idx | Provider 内ユーザ ID |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 14 | `last_login_at` | 最終ログイン | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | ログイン追跡 |
| 15 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 16 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `user_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_user_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `uq_user_tenant_email` | UNIQUE | `(tenant_id, email)` | `WHERE deleted_at IS NULL` | テナント内 email 一意 |
| `ck_user_status` | CHECK | `status` | `IN ('ACTIVE','SUSPENDED','INVITED','ARCHIVED')` | 4 状態 |
| `ck_user_oauth` | CHECK | `oauth_provider`/`oauth_subject` | `(oauth_provider IS NULL AND oauth_subject IS NULL) OR (oauth_provider IS NOT NULL AND oauth_subject IS NOT NULL)` | OAuth 整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `user_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_user_tenant_email` | btree (UK/PT) | `(tenant_id, email)` | `deleted_at IS NULL` | email 一意 |
| `idx_user_tenant_status` | btree (PT) | `(tenant_id, status)` | `deleted_at IS NULL` | ステータス別 |
| `idx_user_tenant_display_name` | btree | `(tenant_id, display_name)` | − | 表示名検索 |
| `idx_user_oauth_lookup` | btree (PT) | `(oauth_provider, oauth_subject)` | `oauth_provider IS NOT NULL` | OAuth login 検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_user_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 1.5 KB | 1,000,000 | 約 1.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `identity.device` | `user_id` |
| `identity.user_session` | `user_id` |
| `work_item.work_item` | `assignee_user_id` / `reporter_user_id` |
| `agent.agent_session` | `initiated_by_user_id` |
| ... 60+ テーブル | 多目的 |

---

## 9. RLS Policy

```sql
ALTER TABLE identity.user ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.user FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_tenant_isolation ON identity.user
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
