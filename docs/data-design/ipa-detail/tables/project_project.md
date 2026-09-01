# project.project — テーブル詳細設計書

> **テーブル ID**: T05
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.3.1
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T05 | per `00-INVENTORY.md` |
| **物理名** | `project.project` | − |
| **論理名** | プロジェクト | Workspace 配下の業務単位 |
| **スキーマ** | `project` | − |
| **Module** | `domain-project` | − |
| **種別** | Entity | E |
| **主キー** | `id UUID` | − |
| **R/W 識別** | R/W（SoR） | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **パーティション** | None | − |
| **soft delete** | Yes | `deleted_at` |
| **概要** | ワークスペース配下の業務単位。WorkItem / Worktree / Board / Sprint 等の親。`project_key` は 2-10 文字大文字英数で業務横断識別子（Jira 風）。 | §4.3.1 / §1.3 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 | §3.1.2 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE RESTRICT | − | idx | RLS 必須 | §7 |
| 3 | `workspace_id` | ワークスペース ID | UUID | − | NO | − | − | `workspace.workspace(id)` ON DELETE RESTRICT | − | idx | 所属ワークスペース | §4.3.1 |
| 4 | `name` | プロジェクト名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 | §3.1.6 |
| 5 | `project_key` | 業務キー | VARCHAR | 10 | NO | − | − | − | ✓ | `uq_project_tenant_key` | 2-10 文字大文字英数、`ck_project_key_format` | §4.3.1 / Jira 風 |
| 6 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 業務説明 | §3.1.6 |
| 7 | `lead_user_id` | リードユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | − | プロジェクトリード | §4.3.1 |
| 8 | `status` | 状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | idx | ライフサイクル状態 | `ck_project_status` |
| 9 | `settings_json` | 設定 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | プロジェクト設定（通知 / 自動化 / 統合） | V1 候補 |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード作成日時 | §3.5 |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード更新日時 | §3.5 |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除日時 | §3.1.5 |
| 13 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック | §3.1.2 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `project_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_project_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | RESTRICT | − |
| `fk_project_workspace` | FOREIGN KEY | `workspace_id` | `workspace.workspace(id)` | RESTRICT | − |
| `fk_project_lead` | FOREIGN KEY | `lead_user_id` | `identity.user(id)` | RESTRICT | − |
| `uq_project_tenant_key` | UNIQUE | `(tenant_id, project_key)` | `WHERE deleted_at IS NULL` | − | 業務キー一意 |
| `ck_project_status` | CHECK | `status` | `IN ('ACTIVE','ARCHIVED','SUSPENDED')` | − | 3 状態 |
| `ck_project_key_format` | CHECK | `project_key` | `~ '^[A-Z][A-Z0-9]{1,9}$'` | − | 2-10 文字大文字英数 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 | 説明 |
|---|---|---|---|---|---|
| `project_pkey` | btree (PK) | `id` | − | − | 主キー |
| `uq_project_tenant_key` | btree (UK/PT) | `(tenant_id, project_key)` | − | `deleted_at IS NULL` | 業務キー一意 |
| `idx_project_tenant_workspace` | btree (PT) | `(tenant_id, workspace_id)` | − | `deleted_at IS NULL` | ワークスペース別 |
| `idx_project_tenant_status` | btree (PT) | `(tenant_id, status)` | − | `deleted_at IS NULL` | ステータス別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_project_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP | 1,000 | 100 テナント × 10 プロジェクト |
| 1 年後 | 10,000 | 1,000 × 10 |
| 3 年後 | 100,000 | 10,000 × 10 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 600 B | 100,000 | 約 60 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `workspace.workspace` | `workspace_id` |
| `identity.user` | `lead_user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `project.project_policy` | `project_id` |
| `work_item.work_item` | `project_id` |
| `workflow.workflow_definition` | `project_id` |
| `board.board` | `project_id` |
| `planning.sprint` | `project_id` |
| `planning.backlog` | `project_id` |
| `planning.roadmap` | `project_id` |
| `work_item.business_goal` | `project_id` |
| `integration.integration` | `project_id` |
| `automation.automation_rule` | `project_id` |
| `scm.repository` | `project_id` |
| `feedback.feedback` | `project_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE project.project ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_project_tenant_isolation ON project.project
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
