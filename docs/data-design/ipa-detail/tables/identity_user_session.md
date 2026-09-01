# identity.user_session — テーブル詳細設計設計書

> **テーブル ID**: T44
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.14.5

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T44 |
| **物理名** | `identity.user_session` |
| **論理名** | ユーザセッション（短 TTL） |
| **スキーマ** | `identity` |
| **Module** | `domain-identity` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | User Session。Refresh Token bcrypt hash。Valkey キャッシュ（§API-3.15）。`is_active` 部分インデックスで アクティブセッション管理。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE CASCADE | − | idx | 親ユーザ |
| 4 | `refresh_token_hash` | Refresh Token ハッシュ | VARCHAR | 255 | NO | − | − | − | ✓ | `uq_user_session_token` | bcrypt hash |
| 5 | `scopes` | スコープ | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | OAuth scope 配列 |
| 6 | `device_id` | デバイス ID | UUID | − | YES | `NULL` | − | `identity.device(id)` (App) | − | − | 紐付デバイス |
| 7 | `is_active` | アクティブ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx | − |
| 8 | `expires_at` | 有効期限 | TIMESTAMPTZ | 8 | NO | − | − | − | − | idx | Refresh 期限 |
| 9 | `client_ip` | クライアント IP | INET | − | YES | `NULL` | − | − | − | − | − |
| 10 | `user_agent` | User-Agent | TEXT | − | YES | `NULL` | − | − | − | − | HTTP UA |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `last_used_at` | 最終使用 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `user_session_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_user_session_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_user_session_user` | FOREIGN KEY | `user_id` | `identity.user(id)` | CASCADE | ユーザ削除時 セッション削除 |
| `uq_user_session_token` | UNIQUE | `refresh_token_hash` | − | − | トークン一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `user_session_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_user_session_token` | btree (UK) | `refresh_token_hash` | − | トークン検索 |
| `idx_user_session_tenant_user` | btree (PT) | `(tenant_id, user_id)` | `is_active = TRUE` | テナント + ユーザ |
| `idx_user_session_expires` | btree (PT) | `(expires_at)` | `is_active = TRUE` | 期限監視 |
| `idx_user_session_token_hash` | btree | `refresh_token_hash` | − | トークン検索（UK と同じ） |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 800 B | 10,000,000 | 約 8 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` | `user_id` |

### 8.2 被参照元

なし（末端、App 層で Valkey キャッシュ）

---

## 9. RLS Policy

```sql
ALTER TABLE identity.user_session ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.user_session FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_session_tenant_isolation ON identity.user_session
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
