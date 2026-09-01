# development.symbol_index — テーブル詳細設計書

> **テーブル ID**: T69
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.7

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T69 |
| **物理名** | `development.symbol_index` |
| **論理名** | シンボルインデックス（Projection） |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | **Projection（P）**（派生、非 SoR） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Symbol インデックス Projection。`signature tsvector GIN` 全文検索。Worker `repository-analysis` 异步 rebuild。`snapshot_ref` で > 10MB 時 Object Storage 移動。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `file_path` | ファイルパス | VARCHAR | 2048 | NO | − | − | − | − | idx | Symbol 所在ファイル |
| 5 | `symbol_ref` | シンボル参照 | VARCHAR | 512 | NO | − | − | − | − | idx | qualified name |
| 6 | `symbol_kind` | シンボル種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'function'` / `'class'` 等 |
| 7 | `signature` | シグネチャ | TEXT | − | YES | `NULL` | − | − | − | GIN | 全文検索対象 |
| 8 | `line_start` | 開始行 | INT | 4 | NO | − | − | − | − | − | − |
| 9 | `line_end` | 終了行 | INT | 4 | NO | − | − | − | − | − | − |
| 10 | `snapshot_at` | 快照日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 索引時刻 |
| 11 | `snapshot_ref` | 快照参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key (>10MB 時) |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `symbol_index_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_symbol_index_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_symbol_index_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` | CASCADE | 親 Repo 削除時 索引削除 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `symbol_index_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_symbol_index_tenant_repo_file` | btree | `(tenant_id, repository_id, file_path)` | − | Repo + ファイル |
| `idx_symbol_index_ref` | btree | `(tenant_id, symbol_ref)` | − | qualified name 検索 |
| `idx_symbol_index_signature_gin` | GIN | `to_tsvector('simple', coalesce(signature, ''))` | − | シグネチャ全文検索 |

---

## 5. トリガー / Worker

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| Worker | `repository-analysis` | `symbol-change` 増分 → `symbol_index` upsert |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000,000 |
| 1 年後 | 100,000,000 |
| 3 年後 | 1,000,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB | 1,000,000,000 | 約 1 TB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |

### 8.2 被参照元

なし（Read-Only Projection）

---

## 9. RLS Policy

```sql
ALTER TABLE development.symbol_index ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.symbol_index FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_symbol_index_tenant_isolation ON development.symbol_index
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（Worker INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
