# development.symbol_change — テーブル詳細設計書

> **テーブル ID**: T66
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T66 |
| **物理名** | `development.symbol_change` |
| **論理名** | シンボル変更 |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Weak Entity（`change_set_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Symbol 級変更。`symbol_ref VARCHAR(512)` で qualified name 保持（`module::class::method`）。`line_range INT4RANGE` で 行範囲保持。`old_signature` / `new_signature` で シグネチャ変化追跡。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `change_set_id` | 変更セット ID | UUID | − | NO | − | − | `development.change_set(id)` ON DELETE CASCADE | − | idx | 親 ChangeSet |
| 4 | `symbol_ref` | シンボル参照 | VARCHAR | 512 | NO | − | − | − | − | idx | qualified name |
| 5 | `symbol_kind` | シンボル種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'function'` / `'class'` 等 |
| 6 | `file_path` | ファイルパス | VARCHAR | 2048 | NO | − | − | − | − | idx | − |
| 7 | `line_range` | 行範囲 | INT4RANGE | − | YES | `NULL` | − | − | − | − | PostgreSQL 範囲型 |
| 8 | `status` | 状態 | VARCHAR | 16 | NO | − | − | − | − | − | 4 値 |
| 9 | `old_signature` | 旧シグネチャ | TEXT | − | YES | `NULL` | − | − | − | − | 変化前 |
| 10 | `new_signature` | 新シグネチャ | TEXT | − | YES | `NULL` | − | − | − | − | 変化後 |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `symbol_change_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_symbol_change_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_symbol_change_change_set` | FOREIGN KEY | `change_set_id` | `development.change_set(id)` ON DELETE CASCADE | 親 ChangeSet 削除時 Symbol 削除 |
| `ck_symbol_change_status` | CHECK | `status` | `IN ('ADDED','MODIFIED','DELETED','RENAMED')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `symbol_change_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_symbol_change_tenant_changeset` | btree | `(tenant_id, change_set_id)` | − | テナント + ChangeSet |
| `idx_symbol_change_ref` | btree | `symbol_ref` | − | qualified name 検索 |
| `idx_symbol_change_path` | btree | `file_path` | − | ファイルパス検索 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000,000 |
| 1 年後 | 50,000,000 |
| 3 年後 | 500,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB | 500,000,000 | 約 500 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `development.change_set` | `change_set_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE development.symbol_change ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.symbol_change FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_symbol_change_tenant_isolation ON development.symbol_change
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
