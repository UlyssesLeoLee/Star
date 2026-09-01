# scm.review — テーブル詳細設計書

> **テーブル ID**: T59
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.5

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T59 |
| **物理名** | `scm.review` |
| **論理名** | レビュー |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Weak Entity（`pull_request_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | PR レビュー。4 状態（APPROVED / CHANGES_REQUESTED / COMMENTED / DISMISSED）。`comments JSONB` でインラインコメント保持。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `pull_request_id` | プルリクエスト ID | UUID | − | NO | − | − | `scm.pull_request(id)` ON DELETE CASCADE | − | idx | 親 PR |
| 4 | `reviewer_user_id` | レビュア ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | idx | レビュアユーザ |
| 5 | `state` | 状態 | VARCHAR | 32 | NO | − | − | − | − | − | 4 値 |
| 6 | `comments` | コメント | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | インラインコメント配列 |
| 7 | `submitted_at` | 提出日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レビュー提出時刻 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 11 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `review_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_review_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_review_pull_request` | FOREIGN KEY | `pull_request_id` | `scm.pull_request(id)` | CASCADE | 親 PR 削除時 レビュー削除 |
| `fk_review_reviewer` | FOREIGN KEY | `reviewer_user_id` | `identity.user(id)` | RESTRICT | レビュア削除禁止 |
| `ck_review_state` | CHECK | `state` | `IN ('APPROVED','CHANGES_REQUESTED','COMMENTED','DISMISSED')` | − | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `review_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_review_tenant_pr` | btree (PT) | `(tenant_id, pull_request_id)` | `deleted_at IS NULL` | テナント + PR |
| `idx_review_reviewer` | btree | `reviewer_user_id` | − | レビュア別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_review_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500,000 |（PR 平均 5 レビュー） |
| 1 年後 | 5,000,000 |
| 3 年後 | 50,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (JSONB) | 50,000,000 | 約 75 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.pull_request` | `pull_request_id` |
| `identity.user` | `reviewer_user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `scm.pull_request.review_ids` (配列) | 文字列 ID 参照 |

---

## 9. RLS Policy

```sql
ALTER TABLE scm.review ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.review FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_review_tenant_isolation ON scm.review
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
