# project.project_template — テーブル詳細設計書

> **テーブル ID**: T07
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.3.3
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T07 | per `00-INVENTORY.md` |
| **物理名** | `project.project_template` | − |
| **論理名** | プロジェクトテンプレート | プロジェクト雛形 |
| **スキーマ** | `project` | − |
| **Module** | `domain-project` | − |
| **種別** | Entity | E |
| **主キー** | `id UUID` | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **概要** | プロジェクト雛形。新規プロジェクト作成時にテンプレート選択で自動セットアップ。`visibility = 'PUBLIC'` は全テナント共有、`'PRIVATE'` はテナント内。 | §4.3.3 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 |
| 2 | `tenant_id` | テナント ID | UUID | − | YES | `NULL` | − | `tenant.tenant(id)` ON DELETE RESTRICT | − | idx | NULL = システム全体テンプレ（Platform Admin 作成） |
| 3 | `name` | テンプレート名 | VARCHAR | 200 | NO | − | − | − | − | − | − |
| 4 | `template_key` | 業務キー | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_project_template_tenant_key` | テナント内一意 |
| 5 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 6 | `visibility` | 可視性 | VARCHAR | 32 | NO | `'PRIVATE'` | − | − | − | idx | `'PUBLIC'` / `'PRIVATE'` / `'INTERNAL'` |
| 7 | `template_json` | テンプレート本体 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 雛形（Board / Workflow / Sprint 設定） |
| 8 | `category` | カテゴリ | VARCHAR | 64 | YES | `NULL` | − | − | − | − | 業務分類（`'web'` / `'mobile'` / `'backend'` 等） |
| 9 | `icon_url` | アイコン URL | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | − |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | − |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `project_template_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_project_template_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | RESTRICT | NULL = 全体テンプレ |
| `uq_project_template_tenant_key` | UNIQUE | `(tenant_id, template_key)` | `WHERE deleted_at IS NULL` | − | 業務キー一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `project_template_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_project_template_tenant_key` | btree (UK/PT) | `(tenant_id, template_key)` | `deleted_at IS NULL` | 業務キー一意 |
| `idx_project_template_tenant_visibility` | btree (PT) | `(tenant_id, visibility)` | `deleted_at IS NULL` | 可視性別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_project_template_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50 |
| 1 年後 | 500 |
| 3 年後 | 5,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB (template_json 含む) | 5,000 | 約 10 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` (NULL 許容) |

### 8.2 被参照元

なし（雛形、参照されない）

---

## 9. RLS Policy

```sql
ALTER TABLE project.project_template ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project_template FORCE ROW LEVEL SECURITY;

-- tenant_id NULL = システム全体テンプレ、全テナント可視
-- tenant_id NOT NULL = 自テナントのみ
CREATE POLICY policy_project_template_tenant_isolation ON project.project_template
  USING (
    tenant_id IS NULL
    OR tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid
  );
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
