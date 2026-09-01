# relation.relation — テーブル詳細設計書

> **テーブル ID**: T23
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.8.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T23 |
| **物理名** | `relation.relation` |
| **論理名** | 関連（WorkItem 間） |
| **スキーマ** | `relation` |
| **Module** | `domain-relation` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WorkItem 間の関連。4 種別（`blocks` / `blocked_by` / `relates_to` / `duplicates`）。`source <> target` CHECK 強制。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `source_work_item_id` | 関連元 WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 関連元 |
| 5 | `target_work_item_id` | 関連先 WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 関連先 |
| 6 | `relation_type` | 関連種別 | VARCHAR | 16 | NO | − | − | − | ✓ | `uq_relation` | 4 種別 |
| 7 | `created_by_user_id` | 作成者 ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | − | 関連作成者 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `relation_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_relation_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_relation_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `fk_relation_source` | FOREIGN KEY | `source_work_item_id` | `work_item.work_item(id)` | CASCADE | − |
| `fk_relation_target` | FOREIGN KEY | `target_work_item_id` | `work_item.work_item(id)` | CASCADE | − |
| `fk_relation_creator` | FOREIGN KEY | `created_by_user_id` | `identity.user(id)` | RESTRICT | − |
| `uq_relation` | UNIQUE | `(source_work_item_id, target_work_item_id, relation_type, deleted_at)` | − | − | 関連重複禁止 |
| `ck_relation_type` | CHECK | `relation_type` | `IN ('blocks','blocked_by','relates_to','duplicates')` | − | 4 種別 |
| `ck_relation_no_self` | CHECK | `source_work_item_id`/`target_work_item_id` | `source_work_item_id <> target_work_item_id` | − | 自己関連禁止 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `relation_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_relation` | btree (UK/PT) | `(source_work_item_id, target_work_item_id, relation_type, deleted_at)` | − | 関連重複禁止 |
| `idx_relation_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |
| `idx_relation_source` | btree | `source_work_item_id` | − | 関連元検索 |
| `idx_relation_target` | btree | `target_work_item_id` | − | 関連先検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_relation_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 500 B | 10,000,000 | 約 5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `work_item.work_item` | `source_work_item_id` / `target_work_item_id` |
| `identity.user` | `created_by_user_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE relation.relation ENABLE ROW LEVEL SECURITY;
ALTER TABLE relation.relation FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_relation_tenant_isolation ON relation.relation
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
