# scm.pull_request — テーブル詳細設計書

> **テーブル ID**: T58
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T58 |
| **物理名** | `scm.pull_request` |
| **論理名** | プルリクエスト |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Weak Entity（`repository_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | PR/MR 镜像。8 状態（DRAFT / OPEN / REVIEWING / CHANGES_REQUESTED / APPROVED / MERGEABLE / MERGED / CLOSED）。`source_branch` / `target_branch` 文字列で参照（FK なし、App 検証）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `external_id` | 外部 ID | VARCHAR | 256 | NO | − | − | − | ✓ | `uq_pr_repo_external` | Provider 内 ID |
| 5 | `source_branch` | ソースブランチ | VARCHAR | 200 | NO | − | − | (App 検証) | − | − | PR 元ブランチ |
| 6 | `target_branch` | ターゲットブランチ | VARCHAR | 200 | NO | − | − | (App 検証) | − | − | PR 先ブランチ |
| 7 | `title` | タイトル | VARCHAR | 500 | NO | − | − | − | − | − | PR タイトル |
| 8 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | PR 説明 |
| 9 | `author_user_id` | 著者ユーザ ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | − | 著者 |
| 10 | `state` | 状態 | VARCHAR | 32 | NO | `'DRAFT'` | − | − | − | idx | 8 値 |
| 11 | `linked_work_item_id` | 関連 WorkItem ID | UUID | YES | `NULL` | − | − | `work_item.work_item(id)` (App) | − | idx | WorkItem 紐付け |
| 12 | `review_ids` | レビュー ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連レビュー |
| 13 | `pipeline_ids` | パイプライン ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連 Pipeline |
| 14 | `merged_at` | マージ日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | マージ成功 |
| 15 | `merged_by_user_id` | マージ実行者 ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | − | マージ実行ユーザ |
| 16 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 17 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 18 | `closed_at` | 終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | CLOSED 状態 |
| 19 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 20 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `pull_request_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_pr_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_pr_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` ON DELETE CASCADE | 親 Repo 削除時 PR 削除 |
| `uq_pr_repo_external` | UNIQUE | `(repository_id, external_id, deleted_at)` | − | Repo 内 PR 一意 |
| `ck_pr_state` | CHECK | `state` | `IN ('DRAFT','OPEN','REVIEWING','CHANGES_REQUESTED','APPROVED','MERGEABLE','MERGED','CLOSED')` | 8 状態 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `pull_request_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_pr_repo_external` | btree (UK/PT) | `(repository_id, external_id, deleted_at)` | − | Repo 内 PR 一意 |
| `idx_pr_tenant_repo_state` | btree (PT) | `(tenant_id, repository_id, state)` | `deleted_at IS NULL` | Repo + 状態 |
| `idx_pr_tenant_workitem` | btree (PT) | `(tenant_id, linked_work_item_id)` | `linked_work_item_id IS NOT NULL` | WorkItem 別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_pull_request_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 1.5 KB | 10,000,000 | 約 15 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |
| `identity.user` (App 検証) | `author_user_id` / `merged_by_user_id` |
| `work_item.work_item` (App 検証) | `linked_work_item_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `scm.review` | `pull_request_id` |
| `scm.pipeline` | `pull_request_id` |
| `comment.comment` | `parent_type='pull_request'` |

---

## 9. RLS Policy

```sql
ALTER TABLE scm.pull_request ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.pull_request FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_pull_request_tenant_isolation ON scm.pull_request
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
