# 00-CONSTRAINTS.md — Star プラットフォーム 全制約 インベントリ

> **基準**: IPA データモデル詳細設計書 — 制約一覧
> **作成日**: 2026-09-01
> **一次出典**: `D:\Star\docs\data-design.md` v0.2 §4 全 `CONSTRAINT` 句
> **本ファイル役割**: 25 Schema × 100 テーブルの CHECK / UK / FK 制約俯瞰

---

## 0. 凡例

| 種別 | 説明 | 命名規約 |
|---|---|---|
| **PK** | 主キー (自動) | `pk_{table}` (PG 自動) |
| **FK** | 外部キー | `fk_{table}_{ref_table}` |
| **UK** | UNIQUE 制約 | `uq_{table}_{col}` または複合 UK |
| **CK** | CHECK 制約 | `ck_{table}_{col}` |

---

## 1. CHECK 制約一覧

> 値域・列挙・論理制約のすべて。Lookup Table との二重管理は §3.3.1 参照。

| 制約名 | テーブル | 列 | 条件式 | 説明 |
|---|---|---|---|---|
| `ck_tenant_status` | `tenant.tenant` | `status` | `IN ('ACTIVE','SUSPENDED','ARCHIVED')` | 3 状態 |
| `ck_tenant_plan` | `tenant.tenant` | `plan` | `IN ('free','pro','enterprise','trial')` | 4 プラン |
| `ck_policy_xor` | `tenant.tenant_policy` | `cloud_ai_allowed`/`local_ai_only` | `(cloud_ai_allowed = FALSE AND local_ai_only = TRUE) OR (cloud_ai_allowed = TRUE AND local_ai_only = FALSE)` | 排他 |
| `ck_policy_specific` | `tenant.tenant_policy` | `specific_provider_allowed`/`cloud_ai_restricted` | `(specific_provider_allowed <> '[]') OR (cloud_ai_restricted = FALSE)` | 整合性 |
| `ck_provider_data_boundary_retention` | `tenant.provider_data_boundary` | `retention_policy`/`retention_days` | `retention_policy <> 'N_DAYS' OR retention_days IS NOT NULL` | N_DAYS モード整合 |
| `ck_workspace_type` | `workspace.workspace` | `type` | `IN ('TEAM','PERSONAL','ENTERPRISE')` | 3 種別 |
| `ck_project_status` | `project.project` | `status` | `IN ('ACTIVE','ARCHIVED','SUSPENDED')` | 3 状態 |
| `ck_project_key_format` | `project.project` | `project_key` | `~ '^[A-Z][A-Z0-9]{1,9}$'` | 大文字英数 2-10 |
| `ck_work_item_status` | `work_item.work_item` | `status` | `IN ('TODO','IN_PROGRESS','DONE','BLOCKED','CANCELLED')` | 5 状態（§3.3.2） |
| `ck_work_item_priority` | `work_item.work_item` | `priority` | `IN ('P0','P1','P2','P3','P4')` | 5 段階 |
| `ck_acceptance_criterion_check` | `work_item.acceptance_criterion` | `is_checked` | `(is_checked = TRUE) = (checked_at IS NOT NULL)` | 整合性 |
| `ck_workflow_state_terminal` | `workflow.workflow_state` | `is_initial`/`is_terminal` | `NOT (is_initial = TRUE AND is_terminal = TRUE)` | 排他 |
| `ck_workflow_transition_valid` | `workflow.workflow_transition` | `from_state_id`/`to_state_id` | `from_state_id <> to_state_id` | 始点≠終点 |
| `ck_board_column_wip_limit` | `board.board_column` | `wip_limit` | `wip_limit IS NULL OR wip_limit > 0` | 0 以下禁止 |
| `ck_sprint_state` | `planning.sprint` | `state` | `IN ('PLANNING','ACTIVE','CLOSED')` | 3 状態 |
| `ck_sprint_dates` | `planning.sprint` | `start_date`/`end_date` | `end_date > start_date` | 期間整合 |
| `ck_relation_type` | `relation.relation` | `relation_type` | `IN ('BLOCKS','DUPLICATES','RELATES_TO','PARENT_OF','REFERENCES')` | 5 種別 |
| `ck_relation_no_self` | `relation.relation` | `source_id`/`target_id` | `source_id <> target_id` | 自己参照禁止 |
| `ck_comment_visibility` | `comment.comment` | `visibility` | `IN ('PUBLIC','INTERNAL','PRIVATE')` | 3 段階 |
| `ck_attachment_size` | `comment.attachment` | `size_bytes` | `size_bytes >= 0 AND size_bytes <= 104857600` | 0..100MB |
| `ck_search_index_resource_type` | `search.search_index` | `resource_type` | `IN ('WorkItem','Comment','Project','Repository')` | 4 種別 |
| `ck_audit_event_action` | `audit.audit_event` | `action` | `length(action) >= 1 AND length(action) <= 64` | 長さ制約 |
| `ck_outbox_published` | `audit.audit_event_outbox` | `published_at`/`retry_count` | `(published_at IS NULL AND retry_count >= 0) OR (published_at IS NOT NULL AND published_at >= created_at)` | 状態整合 |
| `ck_integration_status` | `integration.integration` | `status` | `IN ('ACTIVE','PAUSED','ERROR','DISABLED')` | 4 状態 |
| `ck_integration_provider` | `integration.integration` | `provider_id` | `IN ('github','gitlab','forgejo','jira','linear')` | 5 プロバイダ（V2 で forgejo 拡張） |
| `ck_automation_rule_status` | `automation.automation_rule` | `status` | `IN ('ENABLED','DISABLED')` | 2 状態 |
| `ck_user_email_format` | `identity.user` | `email` | `email ~* '^[^@\s]+@[^@\s]+\.[^@\s]+$'` | メール形式 |
| `ck_user_status` | `identity.user` | `status` | `IN ('ACTIVE','INVITED','SUSPENDED','DELETED')` | 4 状態 |
| `ck_device_platform` | `identity.device` | `platform` | `IN ('macos','windows','linux','ios','android','web')` | 6 プラットフォーム |
| `ck_credential_provider` | `identity.credential` | `provider_id` | `IN ('github','gitlab','openai','anthropic','google','azure','custom')` | 7 プロバイダ |
| `ck_user_session_expires` | `identity.user_session` | `expires_at`/`created_at` | `expires_at > created_at` | TTL 整合 |
| `ck_notification_status` | `notification.notification` | `status` | `IN ('PENDING','SENT','FAILED')` | 3 状態 |
| `ck_notification_audience_scope` | `notification.notification` | `audience_scope` | `IN ('human','agent','system')` | 3 種別（REQ-NOTIF-002） |
| `ck_role_scope` | `permission.role` | `scope` | `IN ('TENANT','PROJECT','WORKSPACE')` | 3 スコープ |
| `ck_security_policy_enforce_mfa` | `permission.security_policy` | `enforce_mfa` | `enforce_mfa IN (TRUE, FALSE)` | 真偽値 |
| `ck_presence_status` | `collaboration.presence` | `status` | `IN ('ONLINE','AWAY','OFFLINE')` | 3 状態 |
| `ck_realtime_subscription_expires` | `collaboration.realtime_subscription` | `expires_at` | `expires_at > NOW()` | 有効期間（CHECK は CHECK のみ、PT でフィルタ） |
| `ck_repository_provider` | `scm.repository` | `provider_id` | `IN ('github','gitlab','forgejo')` | 3 プロバイダ（V2 で forgejo） |
| `ck_pull_request_status` | `scm.pull_request` | `status` | `IN ('OPEN','MERGED','CLOSED','DRAFT','READY_FOR_REVIEW','APPROVED','CHANGES_REQUESTED')` | 7 状態 |
| `ck_review_state` | `scm.review` | `state` | `IN ('APPROVED','CHANGES_REQUESTED','COMMENTED','DISMISSED','PENDING')` | 5 状態 |
| `ck_pipeline_status` | `scm.pipeline` | `status` | `IN ('QUEUED','RUNNING','SUCCESS','FAILURE','CANCELLED','SKIPPED')` | 6 状態 |
| `ck_webhook_event_delivery` | `scm.webhook_event` | `delivery_status` | `IN ('RECEIVED','PROCESSED','FAILED','RETRYING')` | 4 状態 |
| `ck_development_execution_status` | `development.development_execution` | `status` | `IN ('PLANNED','IN_PROGRESS','PAUSED','COMPLETED','FAILED','CANCELLED')` | 6 状態 |
| `ck_change_set_status` | `development.change_set` | `status` | `IN ('DRAFT','STAGED','COMMITTED','PUSHED','PR_OPEN','MERGED','ABANDONED')` | 7 状態 |
| `ck_file_change_op` | `development.file_change` | `operation` | `IN ('ADD','MODIFY','DELETE','RENAME','COPY')` | 5 オペレーション |
| `ck_risk_signal_severity` | `development.risk_signal` | `severity` | `IN ('INFO','LOW','MEDIUM','HIGH','CRITICAL')` | 5 段階 |
| `ck_change_set_link_type` | `development.change_set_link` | `link_type` | `IN ('DEPENDS_ON','SUPERSEDES','CONFLICTS_WITH','RELATED_TO')` | 4 種別 |
| `ck_worktree_status` | `worktree.worktree` | `status` | `IN ('CREATED','READY','ASSIGNED','AGENT_RUNNING','WAITING_FEEDBACK','FEEDBACK_RECEIVED','VALIDATING','BLOCKED','CONFLICTED','READY_FOR_REVIEW','REVIEWING','READY_FOR_COMMIT','COMMITTED','PR_OPEN','MERGED','ABANDONED','ARCHIVED')` | **17 状態**（§3.3.1） |
| `ck_worktree_status_observed_validity` | `worktree.worktree_status_observed` | `last_observed_at` | `last_observed_at <= NOW() + INTERVAL '5 minutes'` | 未来時刻禁止（バグ防止） |
| `ck_worktree_conflict_severity` | `worktree.worktree_conflict` | `severity` | `IN ('INFO','WARNING','ERROR','CRITICAL')` | 4 段階 |
| `ck_worktree_conflict_resolution` | `worktree.worktree_conflict` | `resolved_at` | `(resolved_at IS NULL) = (resolution_note IS NULL)` | 整合性 |
| `ck_agent_type` | `agent.agent` | `type` | `IN ('AI_CODING','AI_REVIEW','AI_DOC','HUMAN_AI_PAIR','CUSTOM')` | 5 種別 |
| `ck_agent_session_status` | `agent.agent_session` | `status` | `IN ('CREATED','STARTING','RUNNING','WAITING_INPUT','PAUSED','COMPLETED','FAILED','CANCELLED','TIMEOUT','STALLED','RESUMED','OFFLINE','ERROR','ARCHIVED')` | **14 状態**（§3.3.2） |
| `ck_agent_session_lease_expires` | `agent.agent_session` | `lease_expires_at`/`heartbeat_at` | `(lease_expires_at IS NULL) OR (lease_expires_at > heartbeat_at)` | リース整合 |
| `ck_feedback_status` | `feedback.feedback` | `status` | `IN ('OPEN','ACKED','IN_PROGRESS','RESOLVED','REJECTED','EXPIRED')` | **6 状態**（§3.3.2） |
| `ck_feedback_priority` | `feedback.feedback` | `priority` | `IN ('P0','P1','P2','P3')` | 4 段階 |
| `ck_feedback_resolution` | `feedback.feedback` | `resolved_at` | `(resolved_at IS NULL) = (status NOT IN ('RESOLVED','REJECTED'))` | 整合性 |
| `ck_context_packet_compression` | `context.context_packet` | `compression_type` | `IN ('NONE','GZIP','ZSTD')` | 3 種別 |
| `ck_provenance_source_type` | `context.provenance_entry` | `source_type` | `IN ('Human','AI','System','Imported','Skill','Playbook','Squad')` | 7 種別（V2 で拡張） |
| `ck_decision_status` | `context.decision` | `status` | `IN ('PROPOSED','APPROVED','REJECTED')` | 3 状態（§3.3.2） |
| `ck_validation_status` | `validation.validation_result` | `status` | `IN ('PENDING','RUNNING','PASSED','FAILED','ERROR','SKIPPED')` | 6 状態 |
| `ck_validation_severity` | `validation.validation_result` | `severity` | `IN ('INFO','LOW','MEDIUM','HIGH','CRITICAL')` | 5 段階 |
| `ck_validation_evidence_type` | `validation.validation_evidence` | `evidence_type` | `IN ('TEST_OUTPUT','SCREENSHOT','LOG','METRIC','TRACE','MANUAL')` | 6 種別 |
| `ck_acceptance_coverage_status` | `validation.acceptance_coverage` | `status` | `IN ('UNCOVERED','PARTIAL','COVERED','VERIFIED')` | 4 段階 |
| `ck_runtime_status` | `local_runtime.runtime` | `status` | `IN ('ONLINE','OFFLINE','STALE')` | 3 状態 |
| `ck_runtime_command_status` | `local_runtime.runtime_command` | `status` | `IN ('QUEUED','DISPATCHED','RUNNING','SUCCEEDED','FAILED','REJECTED','EXPIRED')` | 7 状態 |
| `ck_runtime_observation_type` | `local_runtime.runtime_observation` | `observation_type` | `IN ('HEARTBEAT','LOG','METRIC','ERROR','STATE_CHANGE')` | 5 種別 |
| `ck_reconciliation_status` | `local_runtime.reconciliation_report` | `status` | `IN ('IN_SYNC','DIVERGED','RECONCILED','FAILED')` | 4 状態 |

---

## 2. UNIQUE 制約一覧

| 制約名 | テーブル | 列 | 説明 |
|---|---|---|---|
| `uq_tenant_slug` | `tenant.tenant` | `slug` | URL 友好短标识（per §3.1.1） |
| `uq_tenant_policy_default` | `tenant.tenant_policy` | `(tenant_id) WHERE is_default = TRUE` | 既定 Policy 1 件保証 |
| `uq_workspace_tenant_slug` | `workspace.workspace` | `(tenant_id, slug)` | テナント内重複禁止 |
| `uq_project_tenant_key` | `project.project` | `(tenant_id, project_key)` | 2-10 文字大文字英数 |
| `uq_project_policy_project` | `project.project_policy` | `(project_id)` | 1 プロジェクト 1 Policy |
| `uq_project_template_tenant_key` | `project.project_template` | `(tenant_id, template_key)` | テナント内一意 |
| `uq_work_item_tenant_key` | `work_item.work_item` | `(tenant_id, project_id, work_item_key)` | プロジェクト内業務キー |
| `uq_requirement_work_item_key` | `work_item.requirement` | `(work_item_id, requirement_key)` | WorkItem 内一意 |
| `uq_workflow_definition_tenant_key` | `workflow.workflow_definition` | `(tenant_id, project_id, workflow_key)` | プロジェクト内 |
| `uq_workflow_state_workflow_key` | `workflow.workflow_state` | `(workflow_id, state_key)` | Workflow 内 |
| `uq_board_tenant_project_key` | `board.board` | `(tenant_id, project_id, board_key)` | プロジェクト内 |
| `uq_board_column_board_key` | `board.board_column` | `(board_id, column_key)` | Board 内 |
| `uq_sprint_tenant_project_key` | `planning.sprint` | `(tenant_id, project_id, sprint_key)` | プロジェクト内 |
| `uq_backlog_tenant_project_key` | `planning.backlog` | `(tenant_id, project_id, backlog_key)` | プロジェクト内 |
| `uq_relation_tenant_source_target_type` | `relation.relation` | `(tenant_id, source_type, source_id, target_type, target_id, relation_type)` | 関連重複禁止 |
| `uq_search_index_tenant_resource` | `search.search_index` | `(tenant_id, resource_type, resource_id)` | 1 リソース 1 索引 |
| `uq_integration_tenant_provider` | `integration.integration` | `(tenant_id, provider_id, external_id)` | プロバイダ内一意 |
| `uq_integration_sync_state_resource` | `integration.integration_sync_state` | `(integration_id, resource_type, resource_id)` | 同期状態重複禁止 |
| `uq_automation_rule_tenant_key` | `automation.automation_rule` | `(tenant_id, project_id, rule_key)` | プロジェクト内 |
| `uq_user_tenant_email` | `identity.user` | `(tenant_id, email)` | citext、email 一意 |
| `uq_device_tenant_fingerprint` | `identity.device` | `(tenant_id, device_fingerprint)` | デバイス一意 |
| `uq_device_binding_triple` | `identity.device_binding` | `(user_id, device_id, credential_id)` | 三重 1:1:1（§R-23.2） |
| `uq_credential_tenant_name` | `identity.credential` | `(tenant_id, name)` | citext、資格情報名 |
| `uq_user_session_token` | `identity.user_session` | `(session_token_hash)` | トークン一意 |
| `uq_notification_channel_tenant_key` | `notification.notification_channel` | `(tenant_id, channel_key)` | テナント内 |
| `uq_notification_template_tenant_key` | `notification.notification_template` | `(tenant_id, template_key)` | テナント内 |
| `uq_role_tenant_key` | `permission.role` | `(tenant_id, role_key)` | テナント内 |
| `uq_permission_scheme_tenant_key` | `permission.permission_scheme` | `(tenant_id, scheme_key)` | テナント内 |
| `uq_security_policy_tenant_key` | `permission.security_policy` | `(tenant_id, policy_key)` | テナント内 |
| `uq_presence_tenant_user` | `collaboration.presence` | `(tenant_id, user_id)` | 1 ユーザ 1 在席 |
| `uq_repository_tenant_external` | `scm.repository` | `(tenant_id, provider_id, external_id)` | プロバイダ内外部 ID |
| `uq_branch_repo_name` | `scm.branch` | `(repository_id, name)` | Repo 内ブランチ一意 |
| `uq_commit_repo_sha` | `scm.commit` | `(repository_id, sha)` | Repo 内 SHA 一意 |
| `uq_pull_request_repo_external` | `scm.pull_request` | `(repository_id, external_id)` | Repo 内 PR 外部 ID |
| `uq_development_execution_tenant_key` | `development.development_execution` | `(tenant_id, execution_key)` | テナント内 |
| `uq_change_set_tenant_key` | `development.change_set` | `(tenant_id, change_set_key)` | テナント内 |
| `uq_change_set_link_pair` | `development.change_set_link` | `(source_change_set_id, target_change_set_id, link_type)` | リンク重複禁止 |
| `uq_worktree_tenant_key` | `worktree.worktree` | `(tenant_id, worktree_key)` | テナント内 |
| `uq_agent_tenant_key` | `agent.agent` | `(tenant_id, agent_key)` | テナント内 |
| `uq_agent_session_tenant_key` | `agent.agent_session` | `(tenant_id, session_key)` | テナント内 |
| `uq_agent_policy_tenant_key` | `agent.agent_policy` | `(tenant_id, policy_key)` | テナント内 |
| `uq_feedback_tenant_key` | `feedback.feedback` | `(tenant_id, feedback_key)` | テナント内 |
| `uq_context_packet_tenant_key` | `context.context_packet` | `(tenant_id, packet_key)` | テナント内 |
| `uq_validation_result_tenant_key` | `validation.validation_result` | `(tenant_id, result_key)` | テナント内 |
| `uq_acceptance_coverage_tenant` | `validation.acceptance_coverage` | `(tenant_id, work_item_id, acceptance_criterion_id)` | WorkItem + AC 1 対 1 |
| `uq_validation_policy_tenant_key` | `validation.validation_policy` | `(tenant_id, policy_key)` | テナント内 |
| `uq_runtime_tenant_device` | `local_runtime.runtime` | `(tenant_id, device_id)` | 1 デバイス 1 ランタイム |

---

## 3. FOREIGN KEY 制約一覧

> 詳細は `00-FK-GRAPH.md` 参照。本節は一覧。

### 3.1 tenant schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_tenant_policy_tenant` | `tenant.tenant_policy` | `tenant_id` | `tenant.tenant` | `id` | CASCADE |
| `fk_provider_data_boundary_tenant` | `tenant.provider_data_boundary` | `tenant_id` | `tenant.tenant` | `id` | CASCADE |
| `fk_provider_data_boundary_policy` | `tenant.provider_data_boundary` | `tenant_policy_id` | `tenant.tenant_policy` | `id` | SET NULL |

### 3.2 workspace / project schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_workspace_tenant` | `workspace.workspace` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_workspace_owner` | `workspace.workspace` | `owner_id` | `identity.user` | `id` | RESTRICT |
| `fk_project_tenant` | `project.project` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_project_workspace` | `project.project` | `workspace_id` | `workspace.workspace` | `id` | RESTRICT |
| `fk_project_lead` | `project.project` | `lead_user_id` | `identity.user` | `id` | RESTRICT |
| `fk_project_policy_project` | `project.project_policy` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_project_template_tenant` | `project.project_template` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |

### 3.3 work_item / workflow / board schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_work_item_tenant` | `work_item.work_item` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_work_item_project` | `work_item.work_item` | `project_id` | `project.project` | `id` | RESTRICT |
| `fk_work_item_assignee` | `work_item.work_item` | `assignee_id` | `identity.user` | `id` | SET NULL |
| `fk_work_item_reporter` | `work_item.work_item` | `reporter_id` | `identity.user` | `id` | RESTRICT |
| `fk_work_item_workflow` | `work_item.work_item` | `workflow_id` | `workflow.workflow_definition` | `id` | SET NULL |
| `fk_work_item_state` | `work_item.work_item` | `current_state_id` | `workflow.workflow_state` | `id` | SET NULL |
| `fk_work_item_parent` | `work_item.work_item` | `parent_id` | `work_item.work_item` | `id` | CASCADE |
| `fk_requirement_work_item` | `work_item.requirement` | `work_item_id` | `work_item.work_item` | `id` | CASCADE |
| `fk_acceptance_criterion_work_item` | `work_item.acceptance_criterion` | `work_item_id` | `work_item.work_item` | `id` | CASCADE |
| `fk_business_goal_tenant` | `work_item.business_goal` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_business_goal_project` | `work_item.business_goal` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_workflow_definition_tenant` | `workflow.workflow_definition` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_workflow_definition_project` | `workflow.workflow_definition` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_workflow_state_workflow` | `workflow.workflow_state` | `workflow_id` | `workflow.workflow_definition` | `id` | CASCADE |
| `fk_workflow_transition_workflow` | `workflow.workflow_transition` | `workflow_id` | `workflow.workflow_definition` | `id` | CASCADE |
| `fk_workflow_transition_from` | `workflow.workflow_transition` | `from_state_id` | `workflow.workflow_state` | `id` | CASCADE |
| `fk_workflow_transition_to` | `workflow.workflow_transition` | `to_state_id` | `workflow.workflow_state` | `id` | CASCADE |
| `fk_board_tenant` | `board.board` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_board_project` | `board.board` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_board_column_board` | `board.board_column` | `board_id` | `board.board` | `id` | CASCADE |
| `fk_board_swimlane_board` | `board.board_swimlane` | `board_id` | `board.board` | `id` | CASCADE |

### 3.4 planning / relation / comment / search schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_sprint_tenant` | `planning.sprint` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_sprint_project` | `planning.sprint` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_backlog_tenant` | `planning.backlog` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_backlog_project` | `planning.backlog` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_roadmap_tenant` | `planning.roadmap` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_roadmap_project` | `planning.roadmap` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_roadmap_business_goal` | `planning.roadmap` | `business_goal_id` | `work_item.business_goal` | `id` | SET NULL |
| `fk_relation_tenant` | `relation.relation` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_comment_tenant` | `comment.comment` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_comment_author` | `comment.comment` | `author_id` | `identity.user` | `id` | RESTRICT |
| `fk_mention_comment` | `comment.mention` | `comment_id` | `comment.comment` | `id` | CASCADE |
| `fk_mention_user` | `comment.mention` | `mentioned_user_id` | `identity.user` | `id` | CASCADE |
| `fk_attachment_tenant` | `comment.attachment` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_attachment_uploader` | `comment.attachment` | `uploader_id` | `identity.user` | `id` | RESTRICT |
| `fk_search_index_tenant` | `search.search_index` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |

### 3.5 audit / integration / automation schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_audit_event_tenant` | `audit.audit_event` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_audit_event_actor` | `audit.audit_event` | `actor_id` | `identity.user` | `id` | SET NULL |
| `fk_ai_audit_metadata_tenant` | `audit.ai_audit_metadata` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_ai_audit_metadata_event` | `audit.ai_audit_metadata` | `audit_event_id` | `audit.audit_event` | `id` | CASCADE |
| `fk_ai_audit_metadata_session` | `audit.ai_audit_metadata` | `agent_session_id` | `agent.agent_session` | `id` | SET NULL |
| `fk_audit_event_outbox_tenant` | `audit.audit_event_outbox` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_integration_tenant` | `integration.integration` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_integration_project` | `integration.integration` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_integration_sync_state_integration` | `integration.integration_sync_state` | `integration_id` | `integration.integration` | `id` | CASCADE |
| `fk_automation_rule_tenant` | `automation.automation_rule` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_automation_rule_project` | `automation.automation_rule` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_automation_trigger_rule` | `automation.automation_trigger` | `rule_id` | `automation.automation_rule` | `id` | CASCADE |
| `fk_automation_action_rule` | `automation.automation_action` | `rule_id` | `automation.automation_rule` | `id` | CASCADE |

### 3.6 identity / notification / permission schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_user_tenant` | `identity.user` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_device_tenant` | `identity.device` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_device_user` | `identity.device` | `user_id` | `identity.user` | `id` | CASCADE |
| `fk_device_binding_user` | `identity.device_binding` | `user_id` | `identity.user` | `id` | CASCADE |
| `fk_device_binding_device` | `identity.device_binding` | `device_id` | `identity.device` | `id` | CASCADE |
| `fk_device_binding_credential` | `identity.device_binding` | `credential_id` | `identity.credential` | `id` | CASCADE |
| `fk_credential_tenant` | `identity.credential` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_credential_creator` | `identity.credential` | `created_by_user_id` | `identity.user` | `id` | RESTRICT |
| `fk_user_session_tenant` | `identity.user_session` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_user_session_user` | `identity.user_session` | `user_id` | `identity.user` | `id` | CASCADE |
| `fk_notification_channel_tenant` | `notification.notification_channel` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_notification_template_tenant` | `notification.notification_template` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_notification_tenant` | `notification.notification` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_notification_recipient` | `notification.notification` | `recipient_id` | `identity.user` | `id` | CASCADE |
| `fk_notification_template` | `notification.notification` | `template_id` | `notification.notification_template` | `id` | RESTRICT |
| `fk_notification_channel` | `notification.notification` | `channel_id` | `notification.notification_channel` | `id` | RESTRICT |
| `fk_role_tenant` | `permission.role` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_permission_scheme_tenant` | `permission.permission_scheme` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_security_policy_tenant` | `permission.security_policy` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |

### 3.7 collaboration / scm schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_presence_tenant` | `collaboration.presence` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_presence_user` | `collaboration.presence` | `user_id` | `identity.user` | `id` | CASCADE |
| `fk_presence_workspace` | `collaboration.presence` | `workspace_id` | `workspace.workspace` | `id` | CASCADE |
| `fk_realtime_subscription_tenant` | `collaboration.realtime_subscription` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_realtime_subscription_user` | `collaboration.realtime_subscription` | `user_id` | `identity.user` | `id` | CASCADE |
| `fk_repository_tenant` | `scm.repository` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_repository_project` | `scm.repository` | `project_id` | `project.project` | `id` | CASCADE |
| `fk_branch_repository` | `scm.branch` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_commit_repository` | `scm.commit` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_commit_author` | `scm.commit` | `author_user_id` | `identity.user` | `id` | SET NULL |
| `fk_pull_request_repository` | `scm.pull_request` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_pull_request_source_branch` | `scm.pull_request` | `source_branch_id` | `scm.branch` | `id` | SET NULL |
| `fk_pull_request_target_branch` | `scm.pull_request` | `target_branch_id` | `scm.branch` | `id` | SET NULL |
| `fk_pull_request_author` | `scm.pull_request` | `author_user_id` | `identity.user` | `id` | SET NULL |
| `fk_pull_request_commit` | `scm.pull_request` | `head_commit_id` | `scm.commit` | `id` | SET NULL |
| `fk_review_pull_request` | `scm.review` | `pull_request_id` | `scm.pull_request` | `id` | CASCADE |
| `fk_review_reviewer` | `scm.review` | `reviewer_user_id` | `identity.user` | `id` | RESTRICT |
| `fk_pipeline_repository` | `scm.pipeline` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_pipeline_commit` | `scm.pipeline` | `commit_sha_id` | `scm.commit` | `id` | SET NULL |
| `fk_webhook_event_tenant` | `scm.webhook_event` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_webhook_event_integration` | `scm.webhook_event` | `integration_id` | `integration.integration` | `id` | SET NULL |

### 3.8 development / worktree schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_development_execution_tenant` | `development.development_execution` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_development_execution_worktree` | `development.development_execution` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_development_execution_creator` | `development.development_execution` | `created_by_user_id` | `identity.user` | `id` | RESTRICT |
| `fk_change_set_tenant` | `development.change_set` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_change_set_worktree` | `development.change_set` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_change_set_commit` | `development.change_set` | `commit_id` | `scm.commit` | `id` | SET NULL |
| `fk_change_set_development_execution` | `development.change_set` | `development_execution_id` | `development.development_execution` | `id` | SET NULL |
| `fk_file_change_change_set` | `development.file_change` | `change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_symbol_change_change_set` | `development.symbol_change` | `change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_risk_signal_change_set` | `development.risk_signal` | `change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_risk_signal_tenant` | `development.risk_signal` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_change_set_link_source` | `development.change_set_link` | `source_change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_change_set_link_target` | `development.change_set_link` | `target_change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_symbol_index_tenant` | `development.symbol_index` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_repository_context_tenant` | `development.repository_context` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_repository_context_repository` | `development.repository_context` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_development_context_tenant` | `development.development_context` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_development_context_change_set` | `development.development_context` | `change_set_id` | `development.change_set` | `id` | CASCADE |
| `fk_worktree_tenant` | `worktree.worktree` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_worktree_work_item` | `worktree.worktree` | `work_item_id` | `work_item.work_item` | `id` | CASCADE |
| `fk_worktree_assignee` | `worktree.worktree` | `assignee_id` | `identity.user` | `id` | SET NULL |
| `fk_worktree_repository` | `worktree.worktree` | `repository_id` | `scm.repository` | `id` | CASCADE |
| `fk_worktree_base_branch` | `worktree.worktree` | `base_branch_id` | `scm.branch` | `id` | SET NULL |
| `fk_worktree_status_observed_tenant` | `worktree.worktree_status_observed` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_worktree_status_observed_worktree` | `worktree.worktree_status_observed` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_worktree_conflict_tenant` | `worktree.worktree_conflict` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_worktree_conflict_worktree` | `worktree.worktree_conflict` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_worktree_conflict_change_set` | `worktree.worktree_conflict` | `change_set_id` | `development.change_set` | `id` | SET NULL |

### 3.9 agent / feedback / context / validation / local_runtime schema FK

| 制約名 | ソース | ソース列 | ターゲット | ターゲット列 | ON DELETE |
|---|---|---|---|---|---|
| `fk_agent_tenant` | `agent.agent` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_agent_session_tenant` | `agent.agent_session` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_agent_session_agent` | `agent.agent_session` | `agent_id` | `agent.agent` | `id` | CASCADE |
| `fk_agent_session_worktree` | `agent.agent_session` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_agent_session_user` | `agent.agent_session` | `initiated_by_user_id` | `identity.user` | `id` | SET NULL |
| `fk_agent_session_event_tenant` | `agent.agent_session_event` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_agent_session_event_session` | `agent.agent_session_event` | `agent_session_id` | `agent.agent_session` | `id` | CASCADE |
| `fk_agent_policy_tenant` | `agent.agent_policy` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_feedback_tenant` | `feedback.feedback` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_feedback_worktree` | `feedback.feedback` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_feedback_submitter` | `feedback.feedback` | `submitter_id` | `identity.user` | `id` | RESTRICT |
| `fk_feedback_recipient` | `feedback.feedback` | `recipient_id` | `identity.user` | `id` | SET NULL |
| `fk_feedback_work_item` | `feedback.feedback` | `work_item_id` | `work_item.work_item` | `id` | SET NULL |
| `fk_feedback_consumed_event_tenant` | `feedback.feedback_consumed_event` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_feedback_consumed_event_feedback` | `feedback.feedback_consumed_event` | `feedback_id` | `feedback.feedback` | `id` | CASCADE |
| `fk_feedback_consumed_event_session` | `feedback.feedback_consumed_event` | `consumed_by_session_id` | `agent.agent_session` | `id` | SET NULL |
| `fk_context_packet_tenant` | `context.context_packet` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_context_packet_session` | `context.context_packet` | `agent_session_id` | `agent.agent_session` | `id` | CASCADE |
| `fk_provenance_entry_tenant` | `context.provenance_entry` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_decision_tenant` | `context.decision` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_decision_worktree` | `context.decision` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_decision_made_by` | `context.decision` | `made_by_user_id` | `identity.user` | `id` | SET NULL |
| `fk_validation_result_tenant` | `validation.validation_result` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_validation_result_worktree` | `validation.validation_result` | `worktree_id` | `worktree.worktree` | `id` | CASCADE |
| `fk_validation_evidence_validation` | `validation.validation_evidence` | `validation_id` | `validation.validation_result` | `id` | CASCADE |
| `fk_acceptance_coverage_tenant` | `validation.acceptance_coverage` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_acceptance_coverage_work_item` | `validation.acceptance_coverage` | `work_item_id` | `work_item.work_item` | `id` | CASCADE |
| `fk_acceptance_coverage_criterion` | `validation.acceptance_coverage` | `acceptance_criterion_id` | `work_item.acceptance_criterion` | `id` | CASCADE |
| `fk_validation_policy_tenant` | `validation.validation_policy` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_runtime_tenant` | `local_runtime.runtime` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_runtime_device` | `local_runtime.runtime` | `device_id` | `identity.device` | `id` | CASCADE |
| `fk_runtime_command_tenant` | `local_runtime.runtime_command` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_runtime_command_runtime` | `local_runtime.runtime_command` | `runtime_id` | `local_runtime.runtime` | `id` | CASCADE |
| `fk_runtime_command_requested_by` | `local_runtime.runtime_command` | `requested_by_user_id` | `identity.user` | `id` | SET NULL |
| `fk_runtime_observation_tenant` | `local_runtime.runtime_observation` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_runtime_observation_runtime` | `local_runtime.runtime_observation` | `runtime_id` | `local_runtime.runtime` | `id` | CASCADE |
| `fk_reconciliation_report_tenant` | `local_runtime.reconciliation_report` | `tenant_id` | `tenant.tenant` | `id` | RESTRICT |
| `fk_reconciliation_report_runtime` | `local_runtime.reconciliation_report` | `runtime_id` | `local_runtime.runtime` | `id` | CASCADE |

---

## 4. ON DELETE 戦略サマリ

| 戦略 | 件数 | 用途 |
|---|---|---|
| **RESTRICT** | 60+ | 親（Tenant / Project / Workspace / Repository / User）保護、誤削除連鎖防止 |
| **CASCADE** | 40+ | 弱実体（WorkItem → Requirement / AcceptanceCriterion 等）、所有関係 |
| **SET NULL** | 20+ | 任意参照（assignee / worktree / repository 削除時に null 化、業務継続） |

---

## 5. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：60+ CHECK + 47 UK + 120+ FK 制約列挙 | per 2026-09-01 15:30 JST Ulysses 拍板 |

---
