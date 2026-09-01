# permission.permission_scheme — テーブル詳細設計書

> **テーブル ID**: T51
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.16.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T51 |
| **物理名** | `permission.permission_scheme` |
| **論理名** | パーミッションスキーム |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Permission Scheme。`role_assignments JSONB` で ユーザ / グループ / デバイス → ロール。`agent_role_assignments` で Agent Role 強制割当（§R-PERM-002）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | スキーム名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_scheme_project_name` | 業務表示名 |
| 5 | `role_assignments` | ロール割当 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | GIN | ユーザ / グループ → ロール |
| 6 | `agent_role_assignments` | Agent ロール割当 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | Agent → ロール（§R-PERM-002 強制） |
| 7 | `is_default` | 既定スキーム | BOOLEAN | 1 | NO | `FALSE` | − | − | − | idx (PT) | PJ 既定 1 件 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `permission_scheme_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_permission_scheme_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_permission_scheme_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `uq_scheme_project_name` | UNIQUE | `(project_id, name, deleted_at)` | − | − | スキーム名一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `permission_scheme_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_scheme_project_name` | btree (UK/PT) | `(project_id, name, deleted_at)` | − | スキーム名一意 |
| `idx_permission_scheme_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |
| `idx_permission_scheme_role_assignments_gin` | GIN | `role_assignments` | − | ロール割当検索 |
| `idx_permission_scheme_default` | btree (PT) | `project_id` | `is_default = TRUE AND deleted_at IS NULL` | PJ 既定 1 件 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_permission_scheme_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB (JSONB ×2) | 100,000 | 約 100 MB |

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
ALTER TABLE permission.permission_scheme ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.permission_scheme FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_permission_scheme_tenant_isolation ON permission.permission_scheme
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
