# worktree.worktree_conflict — テーブル詳細設計書

> **テーブル ID**: T74
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.20.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T74 |
| **物理名** | `worktree.worktree_conflict` |
| **論理名** | ワークツリー衝突 |
| **スキーマ** | `worktree` |
| **Module** | `domain-worktree` |
| **種別** | Entity（Worktree 間 1:多） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Worktree 間 File/Symbol レベル衝突。`worktree_id <> other_worktree_id` CHECK 強制。4 段階 risk_level。2 detector（FileLevel / SymbolLevel）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `worktree_id` | ワークツリー ID | UUID | − | NO | − | − | `worktree.worktree(id)` ON DELETE CASCADE | − | idx | 衝突元 |
| 4 | `other_worktree_id` | 相手ワークツリー ID | UUID | − | NO | − | − | `worktree.worktree(id)` ON DELETE CASCADE | − | idx | 衝突先 |
| 5 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | − | 対象 Repo |
| 6 | `file_paths` | ファイルパス配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | − | GIN | 衝突ファイル |
| 7 | `risk_level` | リスクレベル | VARCHAR | 16 | NO | − | − | − | − | − | 4 値 |
| 8 | `detected_at` | 検出日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 検出時刻 |
| 9 | `detector` | 検出器 | VARCHAR | 64 | NO | − | − | − | − | − | 2 値（FileLevel / SymbolLevel） |
| 10 | `resolved_at` | 解決日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT (idx) | 解決時刻 |
| 11 | `resolved_by_user_id` | 解決者 ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | − | 解決者 |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 14 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 15 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `worktree_conflict_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_worktree_conflict_*` | FOREIGN KEY (4) | `tenant_id` / `worktree_id` / `other_worktree_id` / `repository_id` | 各親テーブル | ON DELETE CASCADE |
| `ck_conflict_no_self` | CHECK | `worktree_id`/`other_worktree_id` | `worktree_id <> other_worktree_id` | 自己衝突禁止 |
| `ck_conflict_risk_level` | CHECK | `risk_level` | `IN ('None','Low','Medium','High')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `worktree_conflict_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_worktree_conflict_tenant_worktree` | btree (PT) | `(tenant_id, worktree_id)` | `deleted_at IS NULL` | 衝突元別 |
| `idx_worktree_conflict_other` | btree (PT) | `(tenant_id, other_worktree_id)` | `deleted_at IS NULL` | 衝突先別 |
| `idx_worktree_conflict_file_paths_gin` | GIN | `file_paths` | − | ファイルパス配列検索 |
| `idx_worktree_conflict_unresolved` | btree (PT) | `worktree_id` | `resolved_at IS NULL` | 未解決監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_worktree_conflict_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB | 5,000,000 | 約 6 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `worktree.worktree` | `worktree_id` / `other_worktree_id` |
| `scm.repository` | `repository_id` |
| `identity.user` (App) | `resolved_by_user_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE worktree.worktree_conflict ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree.worktree_conflict FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_worktree_conflict_tenant_isolation ON worktree.worktree_conflict
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
