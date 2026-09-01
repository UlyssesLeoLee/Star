# permission.role — テーブル詳細設計書

> **テーブル ID**: T49
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.16.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T49 |
| **物理名** | `permission.role` |
| **論理名** | ロール |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ロール。4 種 Built-in（`tenant_admin` / `project_admin` / `developer` / `viewer`）。`permission_keys VARCHAR[]` でパーミッション一覧。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `name` | ロール名 | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_role_tenant_name` | 業務表示名 |
| 4 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 5 | `is_builtin` | Built-in フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | ✓ | `uq_role_builtin_key` | システム定義 |
| 6 | `builtin_key` | Built-in キー | VARCHAR | 32 | YES | `NULL` | − | − | ✓ | `uq_role_builtin_key` | 4 値 |
| 7 | `permission_keys` | パーミッションキー配列 | VARCHAR(128)[] | − | NO | `'{}'::varchar[]` | − | − | − | GIN | `work_item:read` 等 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `role_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_role_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `uq_role_tenant_name` | UNIQUE | `(tenant_id, name, deleted_at)` | − | ロール名一意 |
| `uq_role_builtin_key` | UNIQUE (PT) | `(tenant_id, builtin_key)` | `WHERE is_builtin = TRUE AND deleted_at IS NULL` | Built-in 1 件 |
| `ck_role_builtin_xor` | CHECK | `is_builtin`/`builtin_key` | `(is_builtin = FALSE) OR (is_builtin = TRUE AND builtin_key IS NOT NULL)` | Built-in 整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `role_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_role_tenant_name` | btree (UK/PT) | `(tenant_id, name, deleted_at)` | − | ロール名一意 |
| `uq_role_builtin_key` | btree (UK/PT) | `(tenant_id, builtin_key)` | `is_builtin = TRUE AND deleted_at IS NULL` | Built-in 一意 |
| `idx_role_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_role_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 400 |（100 テナント × 4 Built-in） |
| 1 年後 | 4,000 |
| 3 年後 | 40,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 40,000 | 約 20 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `permission.permission_scheme` | `role_assignments` (JSONB 経由) |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.role ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.role FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_role_tenant_isolation ON permission.role
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
