# validation.validation_result — テーブル詳細設計書

> **テーブル ID**: T90
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.24.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T90 |
| **物理名** | `validation.validation_result` |
| **論理名** | 検証結果（核心） |
| **スキーマ** | `validation` |
| **Module** | `domain-validation` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ValidationResult 核心。**10 種 kind**（Build / UnitTest / IntegrationTest / Lint / Format / StaticAnalysis / SecurityCheck / AcceptanceCheck / Review / CustomValidation）+ **6 状態**（§A.5）。**`is_ai_complete_claim` 重要フラグ**：true 時は VAL-001 四重门（ValidationPassed && AcceptanceCoverage==100 && FeedbackResolved && GateApproved）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `work_item_id` | WorkItem ID | UUID | YES | `NULL` | − | − | `work_item.work_item(id)` ON DELETE SET NULL | − | idx | 親 WorkItem |
| 5 | `worktree_id` | ワークツリー ID | UUID | YES | `NULL` | − | − | `worktree.worktree(id)` ON DELETE SET NULL | − | idx (PT) | 親 Worktree |
| 6 | `agent_session_id` | エージェントセッション ID | UUID | YES | `NULL` | − | − | `agent.agent_session(id)` (App) | − | idx | 実行 Session |
| 7 | `change_set_id` | 変更セット ID | UUID | YES | `NULL` | − | − | `development.change_set(id)` (App) | − | idx | 検証対象 ChangeSet |
| 8 | `commit_id` | コミット ID | UUID | YES | `NULL` | − | − | `scm.commit(id)` (App) | − | − | 検証対象 Commit |
| 9 | `triggered_by` | トリガ元 | VARCHAR | 16 | NO | − | − | − | − | − | 4 値 |
| 10 | `triggered_by_id` | トリガ元 ID | UUID | YES | `NULL` | − | − | (App) | − | − | トリガ元 ID |
| 11 | `kind` | 種別 | VARCHAR | 32 | NO | − | − | − | − | idx (PT) | 10 値 |
| 12 | `status` | 状態 | VARCHAR | 16 | NO | `'PENDING'` | − | − | − | idx (PT) | 6 値 |
| 13 | `started_at` | 開始日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx | 開始 |
| 14 | `completed_at` | 完了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 完了 |
| 15 | `failure_summary` | 失敗サマリ | TEXT | − | YES | `NULL` | − | − | − | − | 失敗時概要 |
| 16 | `log_excerpt_ref` | ログ抜粋参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key |
| 17 | `policy_required` | ポリシー必須 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | Policy 必須フラグ |
| 18 | `is_ai_complete_claim` | AI 自報フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | VAL-001 四重门トリガ |
| 19 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 20 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 21 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 22 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `validation_result_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_validation_*` | FOREIGN KEY (3) | `tenant_id` / `project_id` / `work_item_id` / `worktree_id` | 各親テーブル | CASCADE / SET NULL | − |
| `ck_validation_kind` | CHECK | `kind` | `IN (10 値, §4.24.1)` | 10 値 |
| `ck_validation_status` | CHECK | `status` | `IN ('PENDING','RUNNING','PASSED','FAILED','ERRORED','SKIPPED')` | 6 値 |
| `ck_validation_triggered_by` | CHECK | `triggered_by` | `IN ('user','agent','webhook','schedule')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `validation_result_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_validation_tenant_worktree_kind_status` | btree (PT) | `(tenant_id, worktree_id, kind, status)` | `deleted_at IS NULL` | Worktree + Kind + 状態 |
| `idx_validation_tenant_workitem_started` | btree (PT) | `(tenant_id, work_item_id, started_at DESC)` | `deleted_at IS NULL` | WorkItem + 開始順 |
| `idx_validation_tenant_agent_session` | btree (PT) | `(tenant_id, agent_session_id)` | `agent_session_id IS NOT NULL` | Agent Session 別 |
| `idx_validation_tenant_change_set` | btree (PT) | `(tenant_id, change_set_id)` | `change_set_id IS NOT NULL` | ChangeSet 別 |
| `idx_validation_active` | btree (PT) | `(tenant_id, started_at)` | `status IN ('PENDING','RUNNING')` | Active 監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_validation_result_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 800 B | 100,000,000 | 約 80 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `work_item.work_item` | `work_item_id` |
| `worktree.worktree` | `worktree_id` |
| `agent.agent_session` (App) | `agent_session_id` |
| `development.change_set` (App) | `change_set_id` |
| `scm.commit` (App) | `commit_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `validation.validation_evidence` | `validation_result_id` |
| `validation.acceptance_coverage` | `validation_result_ids` 配列 |
| `audit.ai_audit_metadata` | `validation_result_ids` 配列 |
| `agent.agent_session.validation_result_ids` 配列 | 文字列 ID 参照 |

---

## 9. RLS Policy

```sql
ALTER TABLE validation.validation_result ENABLE ROW LEVEL SECURITY;
ALTER TABLE validation.validation_result FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_validation_result_tenant_isolation ON validation.validation_result
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
