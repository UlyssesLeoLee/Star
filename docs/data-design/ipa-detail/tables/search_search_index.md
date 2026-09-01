# search.search_index — テーブル詳細設計書

> **テーブル ID**: T29
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.10.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T29 |
| **物理名** | `search.search_index` |
| **論理名** | 検索インデックス（Projection） |
| **スキーマ** | `search` |
| **Module** | `domain-search` |
| **種別** | **Projection（P）**（派生、非 SoR） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 全文検索 Projection。`search_tsv tsvector` を GIN インデックス化、Worker 异步 rebuild（§12 / §R-SEARCH-001）。**软删除しない**（rebuild 戦略）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | − | − | idx | RLS 必須（FK なし、Worker 書き） |
| 3 | `resource_type` | リソース種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_search_resource` | `'work_item'` / `'comment'` / `'project'` / `'symbol'` |
| 4 | `resource_id` | リソース ID | UUID | − | NO | − | − | − | ✓ | `uq_search_resource` | 索引対象リソース |
| 5 | `title` | タイトル | TEXT | − | NO | − | − | − | − | − | 索引タイトル |
| 6 | `body` | 本文 | TEXT | − | YES | `NULL` | − | − | − | − | 索引本文 |
| 7 | `search_tsv` | tsvector | tsvector | − | YES | (trigger 自動) | − | − | − | GIN | 全文検索 vector |
| 8 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | フィルタ / ハイライト用 |
| 9 | `last_indexed_at` | 最終索引時刻 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 索引鮮度 |
| 10 | `indexed_version` | 索引バージョン | INT | 4 | NO | `1` | − | − | ✓ | `uq_search_resource` | INCR 重建用バージョン |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `search_index_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `uq_search_resource` | UNIQUE | `(resource_type, resource_id, indexed_version)` | − | リソース 1 : 索引 1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `search_index_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_search_resource` | btree (UK) | `(resource_type, resource_id, indexed_version)` | − | リソース 1:1 |
| `idx_search_tenant_tsv_gin` | GIN | `search_tsv` | − | tsvector 全文検索 |
| `idx_search_tenant_resource_type` | btree | `(tenant_id, resource_type)` | − | リソース種別別 |

> **注**: Search Index は `deleted_at` を持たない（Worker projection role 异步 rebuild 戦略）

---

## 5. トリガー / 関数

| 種別 | 名前 | 説明 |
|---|---|---|
| FUNCTION | `search.fn_update_search_tsv()` | `search_tsv = setweight(to_tsvector('simple', title), 'A') \|\| setweight(to_tsvector('simple', body), 'B')` |
| TRIGGER | `trg_search_update_tsv` BEFORE INSERT OR UPDATE OF title, body | `search_tsv` + `last_indexed_at` 自動更新 |

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
| 約 2 KB | 100,000,000 | 約 200 GB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（Projection、FK なし、Worker が書込）

### 8.2 被参照元

なし（Read-Only Projection）

---

## 9. RLS Policy

```sql
ALTER TABLE search.search_index ENABLE ROW LEVEL SECURITY;
ALTER TABLE search.search_index FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_search_index_tenant_isolation ON search.search_index
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
