# scm.repository — テーブル詳細設計書

> **テーブル ID**: T55
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T55 |
| **物理名** | `scm.repository` |
| **論理名** | リポジトリ |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | SCM リポジトリ。MVP `CONNECTED` モードのみ。5 プロバイダ（`github` / `gitlab` / `gitea` / `forgejo` / `bitbucket`）。4 所有権（CONNECTED / MIRRORED / MANAGED / LOCAL_ONLY）、5 同期状態。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `provider` | Provider | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_repository_tenant_provider_external` | 5 値 |
| 5 | `external_id` | 外部 ID | VARCHAR | 256 | NO | − | − | − | ✓ | `uq_repository_tenant_provider_external` | Provider 内 ID |
| 6 | `url` | URL | VARCHAR | 2048 | NO | − | − | − | − | − | リポジトリ URL |
| 7 | `default_branch` | デフォルトブランチ | VARCHAR | 200 | NO | `'main'` | − | − | − | − | Git デフォルト |
| 8 | `ownership` | 所有権 | VARCHAR | 32 | NO | `'CONNECTED'` | − | − | − | − | 4 値 |
| 9 | `sync_status` | 同期状態 | VARCHAR | 32 | NO | `'IN_SYNC'` | − | − | − | PT | 5 値 |
| 10 | `sync_token` | 同期トークン | VARCHAR | 1024 | YES | `NULL` | − | − | − | − | ETag / cursor |
| 11 | `last_synced_at` | 最終同期日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 同期時刻 |
| 12 | `credential_id` | 資格情報 ID | UUID | YES | `NULL` | − | − | `identity.credential(id)` (App) | − | − | Credential Broker 参照 |
| 13 | `is_archived` | アーカイブ済 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | アーカイブ |
| 14 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 15 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 16 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 17 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `repository_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_repository_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_repository_project` | FOREIGN KEY | `project_id` | `project.project(id)` ON DELETE CASCADE | − |
| `uq_repository_tenant_provider_external` | UNIQUE | `(tenant_id, provider, external_id, deleted_at)` | − | Provider 内一意 |
| `ck_repository_provider` | CHECK | `provider` | `IN ('github','gitlab','gitea','forgejo','bitbucket')` | 5 値 |
| `ck_repository_ownership` | CHECK | `ownership` | `IN ('CONNECTED','MIRRORED','MANAGED','LOCAL_ONLY')` | 4 値 |
| `ck_repository_sync_status` | CHECK | `sync_status` | `IN ('IN_SYNC','BEHIND','AHEAD','CONFLICT','DISABLED')` | 5 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `repository_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_repository_tenant_provider_external` | btree (UK/PT) | `(tenant_id, provider, external_id, deleted_at)` | − | Provider 内一意 |
| `idx_repository_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | PJ 別 |
| `idx_repository_sync_status` | btree (PT) | `(sync_status)` | `sync_status <> 'IN_SYNC' AND deleted_at IS NULL` | 異常同期 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_repository_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB | 500,000 | 約 1 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `identity.credential` (App 検証) | `credential_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `scm.branch` | `repository_id` |
| `scm.commit` | `repository_id` |
| `scm.pull_request` | `repository_id` |
| `scm.pipeline` | `repository_id` |
| `worktree.worktree` | `repository_id` |
| `development.development_execution` | `repository_id` |
| `development.symbol_index` | `repository_id` |
| `development.repository_context` | `repository_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE scm.repository ENABLE ROW LEVEL SECURITY;
ALTER TABLE scm.repository FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_repository_tenant_isolation ON scm.repository
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
