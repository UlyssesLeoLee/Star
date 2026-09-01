# agent.agent_session — テーブル詳細設計書

> **テーブル ID**: T78
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.21.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T78 |
| **物理名** | `agent.agent_session` |
| **論理名** | エージェントセッション（核心） |
| **スキーマ** | `agent` |
| **Module** | `domain-agent` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | AgentSession 核心。14 状態（§A.4 F-08 修正）。`token_usage` + `cost_summary` JSONB（S4 落点、V1 候補）。`transcript_ref` で Object Storage 90 天保留。`change_set_ids` / `validation_result_ids` / `feedback_consumed_ids` / `decisions` 4 配列。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `agent_id` | エージェント ID | UUID | − | NO | − | − | `agent.agent(id)` ON DELETE RESTRICT | − | idx | 親 Agent |
| 5 | `agent_type` | 種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'Codex'` 等 |
| 6 | `agent_provider` | Provider | VARCHAR | 64 | NO | − | − | − | − | − | `'openai'` 等 |
| 7 | `agent_version` | バージョン | VARCHAR | 32 | NO | − | − | − | − | − | Agent バージョン |
| 8 | `worktree_id` | ワークツリー ID | UUID | − | NO | − | − | `worktree.worktree(id)` ON DELETE RESTRICT | − | idx | 親 Worktree |
| 9 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE RESTRICT | − | idx | 親 WorkItem |
| 10 | `context_packet_id` | コンテキストパケット ID | UUID | YES | `NULL` | − | − | `context.context_packet(id)` (App) | − | − | 入力 context |
| 11 | `status` | 状態 | VARCHAR | 32 | NO | `'CREATED'` | − | − | − | idx (PT) | 14 値 |
| 12 | `intent` | 意図 | TEXT | − | YES | `NULL` | − | − | − | − | 業務意図 |
| 13 | `plan` | 計画 | JSONB | − | YES | `NULL` | − | − | − | − | 実行計画 |
| 14 | `decisions` | 決定 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | Decision 連動 |
| 15 | `tool_activity_summary` | ツール活動サマリ | JSONB | − | YES | `NULL` | − | − | − | − | 全文は Object Storage |
| 16 | `change_set_ids` | 変更セット ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | ChangeSet 連動 |
| 17 | `validation_result_ids` | 検証結果 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | Validation 連動 |
| 18 | `feedback_consumed_ids` | FB 消費 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | FB 消費 |
| 19 | `result_summary` | 結果サマリ | TEXT | − | YES | `NULL` | − | − | − | − | 結果概要 |
| 20 | `token_usage` | トークン使用 | JSONB | − | YES | `NULL` | − | − | − | − | `{input, output, cached, total}` S4 落点 |
| 21 | `cost_summary` | コストサマリ | JSONB | − | YES | `NULL` | − | − | − | − | `{input_cost, output_cost, total, currency, computed_at}` S4 落点 |
| 22 | `trace_reference` | トレース参照 | VARCHAR | 64 | YES | `NULL` | − | − | − | − | OpenTelemetry TraceId |
| 23 | `transcript_ref` | Transcript 参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key |
| 24 | `started_at` | 開始日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx | 実行開始 |
| 25 | `ended_at` | 終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 実行終了 |
| 26 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 27 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 28 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 29 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `agent_session_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_agent_session_*` | FOREIGN KEY (4) | `tenant_id` / `project_id` / `agent_id` / `worktree_id` / `work_item_id` | 各親テーブル | CASCADE / RESTRICT | − |
| `ck_agent_session_status` | CHECK | `status` | `IN (14 値, §4.21.2)` | − | 14 値 |
| `ck_agent_session_time` | CHECK | `started_at`/`ended_at` | `(ended_at IS NULL) OR (started_at IS NULL) OR (ended_at >= started_at)` | − | 時間整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_session_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_agent_session_tenant_worktree` | btree (PT) | `(tenant_id, worktree_id, status)` | `deleted_at IS NULL` | Worktree + 状態 |
| `idx_agent_session_tenant_workitem` | btree (PT) | `(tenant_id, work_item_id)` | `deleted_at IS NULL` | WorkItem 別 |
| `idx_agent_session_tenant_started` | btree (PT) | `(tenant_id, started_at DESC)` | `deleted_at IS NULL` | 開始順 |
| `idx_agent_session_change_set_ids_gin` | GIN | `change_set_ids` | − | ChangeSet 配列 |
| `idx_agent_session_validation_ids_gin` | GIN | `validation_result_ids` | − | Validation 配列 |
| `idx_agent_session_feedback_ids_gin` | GIN | `feedback_consumed_ids` | − | FB 配列 |
| `idx_agent_session_decisions_gin` | GIN | `decisions` | − | Decision 配列 |
| `idx_agent_session_active` | btree (PT) | `(tenant_id, worktree_id)` | `status NOT IN ('COMPLETED','FAILED','ABORTED','CRASHED','TIMEOUT')` | アクティブ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_agent_session_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 3 KB (配列 ×4 + JSONB ×3 + TEXT) | 10,000,000 | 約 30 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `agent.agent` | `agent_id` |
| `worktree.worktree` | `worktree_id` |
| `work_item.work_item` | `work_item_id` |
| `context.context_packet` (App) | `context_packet_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `audit.ai_audit_metadata` | `agent_session_id` (App) |
| `feedback.feedback_consumed_event` | `consumed_by_session_id` (App) |
| `worktree.worktree` (App) | `current_agent_session_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE agent.agent_session ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.agent_session FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_agent_session_tenant_isolation ON agent.agent_session
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
