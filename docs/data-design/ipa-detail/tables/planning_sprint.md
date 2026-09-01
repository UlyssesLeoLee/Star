# planning.sprint — テーブル詳細設計書

> **テーブル ID**: T19
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.7.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T19 |
| **物理名** | `planning.sprint` |
| **論理名** | スプリント |
| **スキーマ** | `planning` |
| **Module** | `domain-planning` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Scrum スプリント。3 状態（PLANNING / ACTIVE / CLOSED）。`start_at` < `end_at` 強制。`state = 'ACTIVE'` 部分インデックスで現在の Active Sprint を高速検索。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | スプリント名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 5 | `goal` | ゴール | TEXT | − | YES | `NULL` | − | − | − | − | スプリント目標 |
| 6 | `start_at` | 開始日時 | TIMESTAMPTZ | 8 | NO | − | − | − | − | idx | 開始 UTC |
| 7 | `end_at` | 終了日時 | TIMESTAMPTZ | 8 | NO | − | − | − | − | idx | 終了 UTC（`end_at > start_at` CHECK） |
| 8 | `state` | 状態 | VARCHAR | 16 | NO | `'PLANNING'` | − | − | − | idx | `'PLANNING'` / `'ACTIVE'` / `'CLOSED'` |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `sprint_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_sprint_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_sprint_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `ck_sprint_state` | CHECK | `state` | `IN ('PLANNING','ACTIVE','CLOSED')` | − | 3 状態 |
| `ck_sprint_dates` | CHECK | `start_at`/`end_at` | `end_at > start_at` | − | 期間整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `sprint_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_sprint_tenant_project_state` | btree (PT) | `(tenant_id, project_id, state)` | `deleted_at IS NULL` | PJ + 状態 |
| `idx_sprint_tenant_active` | btree (PT) | `(tenant_id, start_at, end_at)` | `state = 'ACTIVE' AND deleted_at IS NULL` | Active Sprint 高速検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_sprint_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 2,000 |
| 1 年後 | 20,000 |
| 3 年後 | 200,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 600 B | 200,000 | 約 120 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `work_item.work_item` | `sprint_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE planning.sprint ENABLE ROW LEVEL SECURITY;
ALTER TABLE planning.sprint FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_sprint_tenant_isolation ON planning.sprint
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
