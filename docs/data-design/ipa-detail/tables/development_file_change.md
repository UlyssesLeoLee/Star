# development.file_change — テーブル詳細設計書

> **テーブル ID**: T65
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T65 |
| **物理名** | `development.file_change` |
| **論理名** | ファイル変更 |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Weak Entity（`change_set_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ChangeSet 子実体。5 状態（ADDED / MODIFIED / DELETED / RENAMED / GENERATED）。`old_path` 経由で rename 検出。`is_generated` で generated file 識別。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `change_set_id` | 変更セット ID | UUID | − | NO | − | − | `development.change_set(id)` ON DELETE CASCADE | − | idx | 親 ChangeSet |
| 4 | `path` | パス | VARCHAR | 2048 | NO | − | − | − | − | idx | ファイルパス |
| 5 | `old_path` | 旧パス | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | リネーム前 |
| 6 | `status` | 状態 | VARCHAR | 16 | NO | − | − | − | − | − | 5 値 |
| 7 | `lines_added` | 追加行数 | INT | 4 | NO | `0` | − | − | − | − | 統計 |
| 8 | `lines_deleted` | 削除行数 | INT | 4 | NO | `0` | − | − | − | − | 統計 |
| 9 | `language` | 言語 | VARCHAR | 32 | YES | `NULL` | − | − | − | − | `'rust'` / `'typescript'` 等 |
| 10 | `is_generated` | 生成ファイル | BOOLEAN | 1 | NO | `FALSE` | − | − | − | PT | generated code 識別 |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `file_change_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_file_change_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_file_change_change_set` | FOREIGN KEY | `change_set_id` | `development.change_set(id)` ON DELETE CASCADE | 親 ChangeSet 削除時 File 削除 |
| `ck_file_change_status` | CHECK | `status` | `IN ('ADDED','MODIFIED','DELETED','RENAMED','GENERATED')` | 5 値 |
| `ck_file_change_rename` | CHECK | `status`/`old_path` | `(status = 'RENAMED' AND old_path IS NOT NULL) OR (status <> 'RENAMED' AND old_path IS NULL)` | リネーム整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `file_change_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_file_change_tenant_changeset` | btree | `(tenant_id, change_set_id)` | − | テナント + ChangeSet |
| `idx_file_change_path` | btree | `path` | − | パス検索 |
| `idx_file_change_generated` | btree (PT) | `(change_set_id)` | `is_generated = TRUE` | 生成ファイル |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000,000 |（ChangeSet 平均 10 ファイル） |
| 1 年後 | 100,000,000 |
| 3 年後 | 1,000,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB | 1,000,000,000 | 約 1.5 TB |

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
ALTER TABLE development.file_change ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.file_change FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_file_change_tenant_isolation ON development.file_change
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
