# scm.pipeline — テーブル詳細設計書

> **テーブル ID**: T60
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.6

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T60 |
| **物理名** | `scm.pipeline` |
| **論理名** | パイプライン（CI/CD） |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Weak Entity（`repository_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | CI パイプライン镜像。3 種別（`ci` / `cd` / `test`）、5 状態（PENDING / RUNNING / SUCCESS / FAILED / CANCELED）。`pull_request_id` で PR 連動。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `pull_request_id` | プルリクエスト ID | UUID | YES | `NULL` | − | − | `scm.pull_request(id)` ON DELETE SET NULL | − | idx | 親 PR |
| 5 | `external_id` | 外部 ID | VARCHAR | 256 | NO | − | − | − | − | − | Provider 内 ID |
| 6 | `pipeline_type` | パイプライン種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'ci'` / `'cd'` / `'test'` |
| 7 | `status` | 状態 | VARCHAR | 32 | NO | − | − | − | − | PT | 5 値 |
| 8 | `started_at` | 開始日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | CI 開始 |
| 9 | `completed_at` | 完了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | CI 完了 |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `pipeline_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_pipeline_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_pipeline_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` | CASCADE | 親 Repo 削除時 Pipeline 削除 |
| `fk_pipeline_pull_request` | FOREIGN KEY | `pull_request_id` | `scm.pull_request(id)` | SET NULL | 親 PR 削除時 NULL 化 |
| `ck_pipeline_status` | CHECK | `status` | `IN ('PENDING','RUNNING','SUCCESS','FAILED','CANCELED')` | − | 5 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `pipeline_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_pipeline_tenant_pr_status` | btree (PT) | `(tenant_id, pull_request_id, status)` | `deleted_at IS NULL` | PR + 状態 |
| `idx_pipeline_tenant_repo` | btree (PT) | `(tenant_id, repository_id)` | `deleted_at IS NULL` | Repo 別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_pipeline_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 600 B | 100,000,000 | 約 60 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |
| `scm.pull_request` | `pull_request_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE scm.pipeline ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.pipeline FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_pipeline_tenant_isolation ON scm.pipeline
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
