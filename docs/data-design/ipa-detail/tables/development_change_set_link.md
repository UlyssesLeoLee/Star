# development.change_set_link — テーブル詳細設計書

> **テーブル ID**: T68
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.6

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T68 |
| **物理名** | `development.change_set_link` |
| **論理名** | 変更セットリンク |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Entity（多対多） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WorkItem ↔ ChangeSet 関連リンク。1 link_type 値（`PRODUCED` 既定、`CONSUMED` / `RELATED` 拡張）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 親 WorkItem |
| 4 | `change_set_id` | 変更セット ID | UUID | − | NO | − | − | `development.change_set(id)` ON DELETE CASCADE | − | idx | 親 ChangeSet |
| 5 | `link_type` | リンク種別 | VARCHAR | 16 | NO | `'PRODUCED'` | − | − | ✓ | `uq_change_set_link_pair` | 4 値 |
| 6 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

> **注**: data-design §4.19.6 には `link_type` の CHECK 制約が未定義、UK 制約も未実装

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `change_set_link_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_change_set_link_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_change_set_link_workitem` | FOREIGN KEY | `work_item_id` | `work_item.work_item(id)` | CASCADE | 親 WorkItem 削除時 リンク削除 |
| `fk_change_set_link_change_set` | FOREIGN KEY | `change_set_id` | `development.change_set(id)` | CASCADE | 親 ChangeSet 削除時 リンク削除 |
| `uq_change_set_link_pair` | UNIQUE | `(work_item_id, change_set_id, link_type)` | − | − | 重複禁止 |

> **注**: 想定 CHECK: `link_type IN ('PRODUCED','CONSUMED','RELATED')`（V1 拡張）

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `change_set_link_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_change_set_link_pair` | btree (UK) | `(work_item_id, change_set_id, link_type)` | − | 重複禁止 |
| `idx_change_set_link_tenant_workitem` | btree | `(tenant_id, work_item_id)` | − | テナント + WorkItem |
| `idx_change_set_link_tenant_changeset` | btree | `(tenant_id, change_set_id)` | − | テナント + ChangeSet |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 200 B | 100,000,000 | 約 20 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `work_item.work_item` | `work_item_id` |
| `development.change_set` | `change_set_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE development.change_set_link ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.change_set_link FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_change_set_link_tenant_isolation ON development.change_set_link
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
