# Frontend TypeScript Schema → Backend PostgreSQL テーブル マッピング

> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **目的**: `frontend/src/types/ids.ts` / `frontend/src/mocks/schemas/*.ts` / `frontend/src/lib/store.ts` の TypeScript 構造を `docs/data-design.md` §4 の PostgreSQL テーブルに 1:1 マッピングし、Backend-Frontend 同期の正本とする
> **本ファイル役割**: scope_everything（per 2026-09-01 15:30 JST 拍板）の一環

---

## 0. 概要

Star フロントエンドはバックエンドの正確なプロジェクションを MSW（Mock Service Worker）経由で取得する。`mocks/schemas/*.ts` の Zod schema は Backend テーブルと 1:1 対応する設計だが、現状は：

1. `mocks/schemas/` 11 ファイルが `agent` / `analytics` / `cli` / `design-artifact` / `five-domain` / `inbox` / `incident` / `validation` 等バラバラの名前空間で実装
2. `lib/store.ts`（23KB）の Zustand state shape は Backend テーブルと一部独自拡張
3. `types/ids.ts`（36KB）の ID 命名が `{Resource}Id` / `{resource}_id` 混在

IPA 化では **Backend テーブルが正本**とし、Frontend 構造を以下にマッピング：

| Frontend | Backend 1:1 対応 | 状態 |
|---|---|---|
| `mocks/schemas/agent.ts` | `agent.agent` / `agent.agent_session` | ✓ 1:1 |
| `mocks/schemas/analytics.ts` | (Projection、独自) | △ Frontend 独自 |
| `mocks/schemas/cli.ts` | (独自、CLI ツールの出力) | △ Frontend 独自 |
| `mocks/schemas/design-artifact.ts` | (独自、生成物) | △ Frontend 独自 |
| `mocks/schemas/five-domain.ts` | (5 域集約ビュー、**Backend に同名なし**、per AGENTS.md §5 disclaimer) | 🔴 命名衝突リスク |
| `mocks/schemas/inbox.ts` | `feedback.feedback_inbox_item` (MV) | ✓ 1:1 |
| `mocks/schemas/incident.ts` | `audit.audit_event` (部分) + `local_runtime.runtime_observation` | △ 部分 1:1 |
| `mocks/schemas/validation.ts` | `validation.validation_result` / `validation.validation_evidence` | ✓ 1:1 |
| `types/ids.ts` | 全 PG `UUID` 列 | ✓ 1:1（型エイリアス） |
| `lib/store.ts` (Board slice) | `board.board` / `board.board_column` | ✓ 1:1 |
| `lib/store.ts` (WorkItem slice) | `work_item.work_item` | ✓ 1:1 |
| `lib/store.ts` (Project slice) | `project.project` | ✓ 1:1 |
| `lib/store.ts` (Worktree slice) | `worktree.worktree` / `worktree.worktree_status_observed` | ✓ 1:1 |
| `lib/store.ts` (Agent slice) | `agent.agent` / `agent.agent_session` | ✓ 1:1 |
| `lib/store.ts` (UI slice) | (独自、UI 状態のみ) | △ Frontend 独自 |

---

## 1. ID Types（`types/ids.ts`）→ PG `UUID` マッピング

| Frontend ID 型 | Backend 列 | Schema.Table | 備考 |
|---|---|---|---|
| `TenantId` | `tenant_id` | 全 13 類 RLS 表 | UUID v7 推奨 |
| `WorkspaceId` | `workspace_id` | `workspace.workspace` | − |
| `ProjectId` | `project_id` | `project.project` | − |
| `WorkItemId` | `work_item_id` | `work_item.work_item` | − |
| `WorktreeId` | `worktree_id` | `worktree.worktree` | − |
| `AgentSessionId` | `agent_session_id` | `agent.agent_session` | − |
| `FeedbackId` | `feedback_id` | `feedback.feedback` | − |
| `ValidationResultId` | `validation_id` | `validation.validation_result` | − |
| `BoardId` | `board_id` | `board.board` | − |
| `BoardColumnId` | `board_column_id` | `board.board_column` | − |
| `SprintId` | `sprint_id` | `planning.sprint` | − |
| `UserId` | `user_id` / `actor_id` / `assignee_id` 等 | `identity.user` | 用途で FK 列名異なる |
| `DeviceId` | `device_id` | `identity.device` | − |
| `RepositoryId` | `repository_id` | `scm.repository` | − |
| `BranchId` | `branch_id` | `scm.branch` | − |
| `CommitId` | `commit_id` | `scm.commit` | − |
| `PullRequestId` | `pull_request_id` | `scm.pull_request` | − |
| `ChangeSetId` | `change_set_id` | `development.change_set` | − |
| `RiskSignalId` | `risk_signal_id` | `development.risk_signal` | − |
| `ContextPacketId` | `context_packet_id` | `context.context_packet` | − |
| `DecisionId` | `decision_id` | `context.decision` | − |
| `CommentId` | `comment_id` | `comment.comment` | − |
| `MentionId` | `mention_id` | `comment.mention` | − |
| `AttachmentId` | `attachment_id` | `comment.attachment` | − |
| `NotificationId` | `notification_id` | `notification.notification` | − |
| `AuditEventId` | `audit_event_id` | `audit.audit_event` | − |
| `IntegrationId` | `integration_id` | `integration.integration` | − |
| `AutomationRuleId` | `rule_id` | `automation.automation_rule` | 命名注意（`rule_id` ≠ `automation_rule_id`） |
| `RuntimeId` | `runtime_id` | `local_runtime.runtime` | − |
| `RuntimeCommandId` | `command_id` | `local_runtime.runtime_command` | − |
| `RoleId` | `role_id` | `permission.role` | − |
| `PermissionSchemeId` | `scheme_id` | `permission.permission_scheme` | 命名注意（`scheme_id` ≠ `permission_scheme_id`） |

### 1.1 命名衝突 (ON-801 系)

| 問題 | 詳細 | 推奨 |
|---|---|---|
| **UserId 多目的化** | `user_id` / `actor_id` / `assignee_id` / `submitter_id` / `recipient_id` / `reviewer_user_id` 等で `identity.user(id)` 参照だが FK 列名が用途で異なる | コンテキスト依存を明示、`@context('actor')` 等 TypeScript branded type 化 |
| **BranchId vs BaseBranchId** | `scm.branch` 汎用 vs `worktree.worktree.base_branch_id` 役割用 | Branded Type `BranchId` / `BaseBranchId` 分離 |
| **CommitId vs HeadCommitId** | `scm.commit` 汎用 vs `scm.pull_request.head_commit_id` 役割用 | Branded Type 分離 |

### 1.2 UUID v7 適用 (ON-403 系)

`types/ids.ts` で UUID として宣言されているが、実装時の `crypto.randomUUID()` (v4) と Backend UUID v7 の整合性は **V1 候補** として残置。Frontend 側で UUID v7 生成する場合は `uuid` パッケージの `v7()` 関数（1.10+）を使用。

---

## 2. Mocks Schemas → Backend テーブル マッピング

### 2.1 `mocks/schemas/agent.ts` (1.18 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `AgentSchema` | `agent.agent` | ✓ | 5 種別 enum 整合 |
| `AgentSessionSchema` | `agent.agent_session` | ✓ | 14 状態 enum 整合 |
| `AgentSessionEventSchema` | `agent.agent_session_event` | ✓ | 状態遷移イベント |
| `AgentPolicySchema` | `agent.agent_policy` | ✓ | Policy 設定 |

> 参照: `tables/agent_agent.md` / `tables/agent_agent_session.md` / `tables/agent_agent_session_event.md` / `tables/agent_agent_policy.md`

### 2.2 `mocks/schemas/analytics.ts` (1.16 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `MetricPointSchema` | (Projection なし) | △ | Frontend 独自、Backend に永続化なし |
| `DashboardSchema` | (Projection なし) | △ | 同上 |

> **推奨**: Backend `analytics` schema 追加（V2 候補）または Frontend のみの Ephemeral state として明示

### 2.3 `mocks/schemas/cli.ts` (2.6 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `CliCommandSchema` | (独自) | △ | CLI ツールの出力、Backend 永続化なし |
| `CliResultSchema` | (独自) | △ | 同上 |

> **推奨**: CLI 実行履歴を `local_runtime.runtime_command` 経由で Backend 永続化可能、現状は Frontend のみ

### 2.4 `mocks/schemas/design-artifact.ts` (3.24 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `DesignArtifactSchema` | (独自) | △ | 設計生成物、Backend に永続化層なし |
| `DesignDiagramSchema` | (独自) | △ | 同上 |

> **推奨**: V2 候補で `comment.attachment` の `attachment_type = 'design_artifact'` 拡張、または新規 schema `design_artifact` 追加

### 2.5 `mocks/schemas/five-domain.ts` (10.07 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `FiveDomainSummarySchema` | (Backend に同名なし) | 🔴 | **per AGENTS.md §5 disclaimer: 5 域独立 (player/economy/match/social/admin) は歴史治理命名、Star 仓 22 DDD bounded context と非対応** |
| `FiveDomainMetricSchema` | (同上) | 🔴 | 同上 |

> **重要**: `five-domain.ts` のスキーマは Backend の **どのテーブルにも直接対応しない**。5 域独立 Lead 確認の上、Backend Projection 化（V1/V2 候補）または Frontend のみの集約ビューとして明示分離。
> **per AGENTS.md §5 命名 disclaimer**: 「5 域」は RGS 仓の 5 位真人 Lead 構造、Star 仓の 22 DDD bounded context とは別分類。マッピング禁止。

### 2.6 `mocks/schemas/inbox.ts` (1.10 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `InboxItemSchema` | `feedback.feedback_inbox_item` (MV) | ✓ | 1:1 対応 |
| `InboxFilterSchema` | (App 層フィルタ) | △ | `audience_scope` / `requires_human_decision` は `notification.notification.audience_scope` 連動 |

> 参照: `tables/feedback_feedback_inbox_item.md`

### 2.7 `mocks/schemas/incident.ts` (3.60 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `IncidentSchema` | `audit.audit_event` (部分) | △ | `severity = 'CRITICAL'` 抽出 |
| `IncidentTimelineSchema` | `audit.audit_event` (時系列) | △ | 同上 |
| `RuntimeAnomalySchema` | `local_runtime.runtime_observation` | ✓ | 1:1 対応 |

> **推奨**: 新規 `audit.incident` スキーマ（V2 候補）または `audit.audit_event` の `incident_*` 列追加

### 2.8 `mocks/schemas/validation.ts` (4.56 KB)

| Zod Schema | Backend Table | 1:1 状態 | 備考 |
|---|---|---|---|
| `ValidationResultSchema` | `validation.validation_result` | ✓ | 6 状態 enum 整合 |
| `ValidationEvidenceSchema` | `validation.validation_evidence` | ✓ | 6 種別 enum 整合 |
| `AcceptanceCoverageSchema` | `validation.acceptance_coverage` | ✓ | 4 段階整合 |
| `ValidationPolicySchema` | `validation.validation_policy` | ✓ | 1:1 対応 |

> 参照: `tables/validation_validation_result.md` / `tables/validation_validation_evidence.md` / `tables/validation_acceptance_coverage.md` / `tables/validation_validation_policy.md`

### 2.9 集約サマリ

| 状態 | 件数 | Frontend Schema | 備考 |
|---|---|---|---|
| ✓ 1:1 対応 | 8 | agent(4) + inbox(1) + incident(1) + validation(4) | Backend テーブル完全同期 |
| △ 部分 1:1 | 6 | analytics(2) + cli(2) + incident(2) | Frontend 独自拡張あり |
| 🔴 Backend 同名なし | 2 | five-domain(2) | **per AGENTS.md §5 disclaimer** で DDD Review 5 域 Lead 拍板待ち |

---

## 3. Zustand Store Slices → Backend テーブル

> `lib/store.ts` (23.34 KB) のスライス構造と Backend テーブル対応。

| Slice | Backend Table 主 | Backend Table 副 | 1:1 状態 |
|---|---|---|---|
| `authSlice` | `identity.user_session` | `identity.user` | ✓ |
| `tenantSlice` | `tenant.tenant` | `tenant.tenant_policy` | ✓ |
| `workspaceSlice` | `workspace.workspace` | − | ✓ |
| `projectSlice` | `project.project` | `project.project_policy` | ✓ |
| `workItemSlice` | `work_item.work_item` | `work_item.requirement` / `acceptance_criterion` | ✓ |
| `boardSlice` | `board.board` / `board.board_column` | `board.board_swimlane` | ✓ |
| `worktreeSlice` | `worktree.worktree` | `worktree.worktree_status_observed` | ✓ |
| `agentSlice` | `agent.agent` / `agent.agent_session` | `agent.agent_session_event` | ✓ |
| `feedbackSlice` | `feedback.feedback` | `feedback.feedback_inbox_item` (MV) | ✓ |
| `validationSlice` | `validation.validation_result` | `validation.validation_evidence` | ✓ |
| `inboxSlice` | `feedback.feedback_inbox_item` (MV) | `notification.notification` | ✓ |
| `uiSlice` | (Frontend 独自) | − | △ UI 状態 |
| `themeSlice` | (Frontend 独自) | − | △ テーマ設定 |
| `selectionSlice` | (Frontend 独自) | − | △ 多選状態 |

### 3.1 computed / virtual field 補足

| Slice | Computed Field | Backend 対応 | 推奨 |
|---|---|---|---|
| `workItemSlice` | `assignee_avatar_url` | (Frontend 独自) | `identity.user.avatar_url` から計算 |
| `workItemSlice` | `is_overdue` | (Frontend 独自) | `due_date < NOW() AND status NOT IN ('DONE','CANCELLED')` で計算 |
| `worktreeSlice` | `is_observed_stale` | `worktree.worktree_status_observed.last_observed_at` から計算 | Backend 列参照可 |
| `agentSlice` | `is_alive` | `agent.agent_session.lease_expires_at > NOW()` | Backend 列参照可 |
| `feedbackSlice` | `requires_human_decision` | `notification.notification.audience_scope = 'human'` | Backend 列参照可 |

### 3.2 命名 / 構造の乖離 (ON-803 系)

| 問題 | 詳細 | 推奨 |
|---|---|---|
| Slice が 14 個で monorepo monorepo monorepo | `lib/store.ts` 23KB 単一ファイルに全 slice | Zustand slice 分割、1 slice 1 ファイル化（V1 候補） |
| `worktreeSlice` 内に `WorktreeStatusBadge` の color map 埋め込み | Backend `worktree.worktree_status` Lookup Table の `display_name` と同等 | Lookup Table に `color_hex` 列追加、Frontend は API 経由で取得 |
| `boardSlice.cards` 配列に `assignees: string[]` 埋め込み | Backend は `work_item.work_item.assignee_id` 単一参照 | 業務上 1 WorkItem = 1 assignee、Backend と整合 |

---

## 4. 同期ギャップ要約

| 区分 | 状態 | 件数 | 摘要 |
|---|---|---|---|
| **完全 1:1 同期** | ✓ | 18+ | ID Types / agent / inbox / validation / 主要 store slice |
| **部分 1:1 同期** | △ | 8+ | analytics / cli / incident / design-artifact |
| **Backend 同名なし** | 🔴 | 2+ | five-domain（per AGENTS.md §5 disclaimer） |
| **Frontend 独自** | △ | 3+ | uiSlice / themeSlice / selectionSlice |

---

## 5. IPA 化時の推奨アクション

### 5.1 即時 (P1)

1. `types/ids.ts` の Branded Type 統一（ON-801 系）
2. `mocks/schemas/five-domain.ts` の **per AGENTS.md §5 disclaimer** をコメント追加（誤解防止）
3. `lib/store.ts` の color map ハードコード → Backend `worktree_status.color_hex` 化（ON-203 連動）

### 5.2 V1 候補 (P2)

1. `analytics` schema 追加（V1 候補）
2. `cli` schema Backend 永続化（V1 候補）
3. `design_artifact` 新規 schema 追加（V2 候補）
4. `incident` を `audit.incident` に昇格（V2 候補）
5. Zustand slice 分割（V1 候補）

### 5.3 DDD Review 待ち (P3)

1. `five-domain.ts` の 5 域集約ビューを Backend Projection 化するかの判断（per AGENTS.md §5、5 域独立 Lead 拍板待ち）

---

## 6. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：ID Types 30+ / Schemas 11 / Store Slices 14 マッピング | per 2026-09-01 15:30 JST Ulysses 拍板（scope_everything） |
