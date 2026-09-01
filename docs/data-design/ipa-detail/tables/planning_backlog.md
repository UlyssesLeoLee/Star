# planning.backlog — テーブル詳細設計書

> **テーブル ID**: T20
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.7.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T20 |
| **物理名** | `planning.backlog` |
| **論理名** | バックログ（排序池） |
| **スキーマ** | `planning` |
| **Module** | `domain-planning` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Project 単位の Backlog 排序池。`work_item_order UUID[]` で WorkItem 順序を保持。Project 1 : Backlog 1。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ（1:1） |
| 4 | `work_item_order` | WorkItem 並び順 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | WorkItem ID 配列、配列順で表示順制御 |
| 5 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 6 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 7 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 8 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `backlog_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_backlog_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_backlog_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | 親 PJ 削除時 Backlog 削除 |
| `uq_backlog_per_project` | UNIQUE | `(project_id, deleted_at)` | − | − | PJ 1 : Backlog 1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `backlog_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_backlog_per_project` | btree (UK/PT) | `(project_id, deleted_at)` | − | PJ 1:1 |
| `idx_backlog_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |
| `idx_backlog_order_gin` | GIN | `work_item_order` | − | 配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_backlog_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 500 B + 配列分 (avg 50 WI × 16B = 800B) | 100,000 | 約 130 MB |

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
ALTER TABLE planning.backlog ENABLE ROW LEVEL SECURITY;
ALTER TABLE planning.backlog FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_backlog_tenant_isolation ON planning.backlog
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
