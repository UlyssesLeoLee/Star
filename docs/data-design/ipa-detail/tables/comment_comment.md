# comment.comment — テーブル詳細設計書

> **テーブル ID**: T25
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.9.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T25 |
| **物理名** | `comment.comment` |
| **論理名** | コメント |
| **スキーマ** | `comment` |
| **Module** | `domain-comment` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WorkItem / PullRequest / Decision 親リソースへのコメント。3 可視性（PUBLIC / INTERNAL / PRIVATE）。Markdown 本文 + リアクション。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `parent_type` | 親リソース種別 | VARCHAR | 32 | NO | − | − | − | − | idx | `'work_item'` / `'pull_request'` / `'decision'` |
| 5 | `parent_id` | 親リソース ID | UUID | − | NO | − | − | (App 検証) | − | idx | 親リソース ID |
| 6 | `author_user_id` | 著者ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | idx | コメント投稿ユーザ |
| 7 | `body` | 本文 | TEXT | − | NO | − | − | − | − | − | Markdown 形式 |
| 8 | `visibility` | 可視性 | VARCHAR | 16 | NO | `'PUBLIC'` | − | − | − | − | 3 段階 |
| 9 | `reactions` | リアクション | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | `{ '👍': 3, '👎': 1 }` 形式 |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `comment_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_comment_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_comment_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `fk_comment_author` | FOREIGN KEY | `author_user_id` | `identity.user(id)` | RESTRICT | − |
| `ck_comment_visibility` | CHECK | `visibility` | `IN ('PUBLIC','INTERNAL','PRIVATE')` | − | 3 段階 |
| `ck_comment_parent_type` | CHECK | `parent_type` | `IN ('work_item','pull_request','decision')` | − | 3 親タイプ |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `comment_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_comment_tenant_parent` | btree (PT) | `(tenant_id, parent_type, parent_id)` | `deleted_at IS NULL` | 親リソース別 |
| `idx_comment_tenant_project_created` | btree (PT) | `(tenant_id, project_id, created_at DESC)` | `deleted_at IS NULL` | PJ + 作成順 |
| `idx_comment_author` | btree | `author_user_id` | − | 著者別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_comment_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 2 KB (TEXT body) | 100,000,000 | 約 200 GB |

> §1.5 派生: body > 1MB は Object Storage へ（将来拡張）

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `identity.user` | `author_user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `comment.mention` | `comment_id` |
| `comment.attachment` | (parent_type=comment 想定, App 検証) |

---

## 9. RLS Policy

```sql
ALTER TABLE comment.comment ENABLE ROW LEVEL SECURITY;
ALTER TABLE comment.comment FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_comment_tenant_isolation ON comment.comment
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
