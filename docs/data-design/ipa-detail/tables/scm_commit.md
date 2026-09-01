# scm.commit — テーブル詳細設計書

> **テーブル ID**: T57
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T57 |
| **物理名** | `scm.commit` |
| **論理名** | コミット |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Weak Entity（`repository_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Git Commit 镜像。`sha` 40-64 文字 hex 制約。`parent_shas` 配列で merge commit 表現。`linked_work_item_id` で WorkItem 紐付け。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `sha` | SHA | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_commit_repo_sha` | Git SHA-1/SHA-256 hex |
| 5 | `author_name` | 作者名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 6 | `author_email` | 作者メール | VARCHAR | 320 | NO | − | − | − | − | − | 連絡先 |
| 7 | `committer_name` | コミッタ名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 8 | `committer_email` | コミッタメール | VARCHAR | 320 | NO | − | − | − | − | − | 連絡先 |
| 9 | `message` | コミットメッセージ | TEXT | − | NO | − | − | − | − | − | Git コミットメッセージ |
| 10 | `parent_shas` | 親 SHA 配列 | VARCHAR(64)[] | − | NO | `'{}'::varchar[]` | − | − | − | GIN | Merge commit 表現 |
| 11 | `tree_sha` | Tree SHA | VARCHAR | 64 | YES | `NULL` | − | − | − | − | Git tree 参照 |
| 12 | `linked_work_item_id` | 関連 WorkItem ID | UUID | YES | `NULL` | − | − | `work_item.work_item(id)` (App) | − | idx | WorkItem 紐付け |
| 13 | `committed_at` | コミット日時 | TIMESTAMPTZ | 8 | NO | − | − | − | − | idx | 業務日時 |
| 14 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 15 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 16 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 17 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `commit_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_commit_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_commit_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` ON DELETE CASCADE | 親 Repo 削除時 Commit 削除 |
| `uq_commit_repo_sha` | UNIQUE | `(repository_id, sha, deleted_at)` | − | Repo 内 SHA 一意 |
| `ck_commit_sha_format` | CHECK | `sha` | `sha ~ '^[a-f0-9]{40,64}$'` | hex 40-64 文字 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `commit_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_commit_repo_sha` | btree (UK/PT) | `(repository_id, sha, deleted_at)` | − | Repo 内 SHA 一意 |
| `idx_commit_tenant_repo_committed` | btree (PT) | `(tenant_id, repository_id, committed_at DESC)` | `deleted_at IS NULL` | Repo + コミット順 |
| `idx_commit_tenant_workitem` | btree (PT) | `(tenant_id, linked_work_item_id)` | `linked_work_item_id IS NOT NULL` | WorkItem 別 |
| `idx_commit_parent_shas_gin` | GIN | `parent_shas` | − | 親 SHA 配列検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_commit_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 1.5 KB | 500,000,000 | 約 750 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |
| `work_item.work_item` (App 検証) | `linked_work_item_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `scm.branch` | `head_commit_id` / `base_commit_id` |
| `scm.pull_request` | `head_commit_id` (App) |
| `scm.pipeline` | `commit_sha_id` (App) |
| `development.change_set` | `commit_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE scm.commit ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.commit FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_commit_tenant_isolation ON scm.commit
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
