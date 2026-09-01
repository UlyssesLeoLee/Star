# scm.branch — テーブル詳細設計書

> **テーブル ID**: T56
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T56 |
| **物理名** | `scm.branch` |
| **論理名** | ブランチ |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Weak Entity（`repository_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Git ブランチ镜像。`head_commit_id` / `base_commit_id` 経由で `scm.commit` 参照。`is_protected` / `is_default` フラグ。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `name` | ブランチ名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_branch_repo_name` | 業務表示名 |
| 5 | `head_commit_id` | HEAD コミット ID | UUID | YES | `NULL` | − | − | `scm.commit(id)` (App) | − | idx | 最新コミット |
| 6 | `base_commit_id` | ベースコミット ID | UUID | YES | `NULL` | − | − | `scm.commit(id)` (App) | − | − | 派生元 |
| 7 | `is_protected` | 保護 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 保護ブランチ |
| 8 | `is_default` | 既定ブランチ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | リポジトリ既定 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `branch_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_branch_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_branch_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` | CASCADE | 親 Repo 削除時 Branch 削除 |
| `uq_branch_repo_name` | UNIQUE | `(repository_id, name, deleted_at)` | − | − | Repo 内ブランチ名一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `branch_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_branch_repo_name` | btree (UK/PT) | `(repository_id, name, deleted_at)` | − | Repo 内一意 |
| `idx_branch_tenant_repo` | btree (PT) | `(tenant_id, repository_id)` | `deleted_at IS NULL` | テナント + Repo |
| `idx_branch_head` | btree (PT) | `head_commit_id` | `head_commit_id IS NOT NULL` | HEAD コミット別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_branch_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |（Repo 平均 10 ブランチ） |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 5,000,000 | 約 2.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |
| `scm.commit` (App 検証) | `head_commit_id` / `base_commit_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `scm.pull_request` | `source_branch_id` / `target_branch_id` (App) |
| `worktree.worktree` | `base_branch` (文字列) / `base_branch_id` (App) |

---

## 9. RLS Policy

```sql
ALTER TABLE scm.branch ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.branch FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_branch_tenant_isolation ON scm.branch
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
