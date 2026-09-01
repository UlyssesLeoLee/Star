# 00-INDEXES.md — Star プラットフォーム 全インデックス インベントリ

> **基準**: IPA データモデル詳細設計書 — インデックス一覧
> **作成日**: 2026-09-01
> **一次出典**: `D:\Star\docs\data-design.md` v0.2 §4 全 `CREATE INDEX` ブロック
> **本ファイル役割**: 25 Schema × 100 テーブルのインデックスを俯瞰

---

## 0. 凡例

| 種別 | 説明 | 例 |
|---|---|---|
| **PK** | 主キー (btree, 自動) | `{table}_pkey` |
| **FK** | 外部キー補助インデックス | `idx_{table}_{fk_col}` |
| **UK** | UNIQUE 制約兼インデックス | `uq_{table}_{col}` |
| **BT** | 業務 btree インデックス | `idx_{table}_{col1}_{col2}` |
| **GIN** | JSONB / tsvector / 配列 | `idx_{table}_{col}_gin` |
| **GiST** | ltree / 幾何 / 全文曖昧 | `idx_{table}_{col}_gist` |
| **BRIN** | 大規模時系列 | `idx_{table}_{col}_brin` |
| **PT** | 部分インデックス (`WHERE deleted_at IS NULL` 等) | `idx_{table}_{col}_active` |

---

## 1. インデックス全体マップ

> 個別詳細・列構成は `tables/{schema}_{table}.md` §4 を参照。

### 1.1 tenant schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `tenant_pkey` | tenant | PK | `id` | − |
| `uq_tenant_slug` | tenant | UK/PT | `slug` | `WHERE deleted_at IS NULL` |
| `idx_tenant_status` | tenant | BT/PT | `status` | `WHERE deleted_at IS NULL` |
| `tenant_policy_pkey` | tenant_policy | PK | `id` | − |
| `uq_tenant_policy_default` | tenant_policy | UK/PT | `tenant_id` | `WHERE is_default = TRUE AND deleted_at IS NULL` |
| `idx_tenant_policy_tenant_effective` | tenant_policy | BT/PT | `(tenant_id, effective_from DESC)` | `WHERE deleted_at IS NULL` |
| `provider_data_boundary_pkey` | provider_data_boundary | PK | `id` | − |
| `idx_provider_data_boundary_tenant` | provider_data_boundary | BT | `(tenant_id, provider_id, model_id)` | − |
| `idx_provider_data_boundary_active` | provider_data_boundary | PT | `(tenant_id)` | `WHERE is_active = TRUE` |

### 1.2 workspace schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `workspace_pkey` | workspace | PK | `id` | − |
| `uq_workspace_tenant_slug` | workspace | UK/PT | `(tenant_id, slug)` | `WHERE deleted_at IS NULL` |
| `idx_workspace_tenant_type` | workspace | BT/PT | `(tenant_id, type)` | `WHERE deleted_at IS NULL` |
| `idx_workspace_tenant_owner` | workspace | BT/PT | `(tenant_id, owner_id)` | `WHERE deleted_at IS NULL` |

### 1.3 project schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `project_pkey` | project | PK | `id` | − |
| `uq_project_tenant_key` | project | UK/PT | `(tenant_id, project_key)` | `WHERE deleted_at IS NULL` |
| `idx_project_tenant_workspace` | project | BT/PT | `(tenant_id, workspace_id)` | `WHERE deleted_at IS NULL` |
| `idx_project_tenant_status` | project | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `project_policy_pkey` | project_policy | PK | `id` | − |
| `uq_project_policy_project` | project_policy | UK/PT | `project_id` | `WHERE deleted_at IS NULL` |
| `project_template_pkey` | project_template | PK | `id` | − |
| `idx_project_template_tenant_visibility` | project_template | BT/PT | `(tenant_id, visibility)` | `WHERE deleted_at IS NULL` |

### 1.4 work_item schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `work_item_pkey` | work_item | PK | `id` | − |
| `uq_work_item_tenant_key` | work_item | UK/PT | `(tenant_id, project_id, work_item_key)` | `WHERE deleted_at IS NULL` |
| `idx_work_item_tenant_project_status` | work_item | BT/PT | `(tenant_id, project_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_work_item_tenant_assignee` | work_item | BT/PT | `(tenant_id, assignee_id, status)` | `WHERE deleted_at IS NULL AND assignee_id IS NOT NULL` |
| `idx_work_item_tenant_priority` | work_item | BT/PT | `(tenant_id, priority, due_date)` | `WHERE deleted_at IS NULL` |
| `idx_work_item_search_gin` | work_item | GIN | `(search_vector)` | `WHERE deleted_at IS NULL` |
| `requirement_pkey` | requirement | PK | `id` | − |
| `uq_requirement_work_item_key` | requirement | UK/PT | `(work_item_id, requirement_key)` | `WHERE deleted_at IS NULL` |
| `acceptance_criterion_pkey` | acceptance_criterion | PK | `id` | − |
| `idx_acceptance_criterion_work_item` | acceptance_criterion | BT/PT | `(work_item_id, order_index)` | `WHERE deleted_at IS NULL` |
| `business_goal_pkey` | business_goal | PK | `id` | − |
| `idx_business_goal_tenant_project` | business_goal | BT/PT | `(tenant_id, project_id)` | `WHERE deleted_at IS NULL` |
| `work_item_status_pkey` | work_item_status | PK | `status_code` | − |

### 1.5 workflow schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `workflow_definition_pkey` | workflow_definition | PK | `id` | − |
| `uq_workflow_definition_tenant_key` | workflow_definition | UK/PT | `(tenant_id, project_id, workflow_key)` | `WHERE deleted_at IS NULL` |
| `idx_workflow_definition_tenant_type` | workflow_definition | BT/PT | `(tenant_id, type)` | `WHERE deleted_at IS NULL` |
| `workflow_state_pkey` | workflow_state | PK | `id` | − |
| `uq_workflow_state_workflow_key` | workflow_state | UK/PT | `(workflow_id, state_key)` | `WHERE deleted_at IS NULL` |
| `idx_workflow_state_workflow_order` | workflow_state | BT | `(workflow_id, order_index)` | − |
| `workflow_transition_pkey` | workflow_transition | PK | `id` | − |
| `idx_workflow_transition_workflow_from` | workflow_transition | BT | `(workflow_id, from_state_id)` | − |

### 1.6 board schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `board_pkey` | board | PK | `id` | − |
| `uq_board_tenant_project_key` | board | UK/PT | `(tenant_id, project_id, board_key)` | `WHERE deleted_at IS NULL` |
| `board_column_pkey` | board_column | PK | `id` | − |
| `uq_board_column_board_key` | board_column | UK/PT | `(board_id, column_key)` | `WHERE deleted_at IS NULL` |
| `idx_board_column_board_order` | board_column | BT | `(board_id, order_index)` | − |
| `board_swimlane_pkey` | board_swimlane | PK | `id` | − |
| `idx_board_swimlane_board_order` | board_swimlane | BT | `(board_id, order_index)` | − |

### 1.7 planning schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `sprint_pkey` | sprint | PK | `id` | − |
| `uq_sprint_tenant_project_key` | sprint | UK/PT | `(tenant_id, project_id, sprint_key)` | `WHERE deleted_at IS NULL` |
| `idx_sprint_tenant_state` | sprint | BT/PT | `(tenant_id, state)` | `WHERE deleted_at IS NULL` |
| `idx_sprint_tenant_dates` | sprint | BT/PT | `(tenant_id, start_date, end_date)` | `WHERE deleted_at IS NULL` |
| `backlog_pkey` | backlog | PK | `id` | − |
| `uq_backlog_tenant_project_key` | backlog | UK/PT | `(tenant_id, project_id, backlog_key)` | `WHERE deleted_at IS NULL` |
| `roadmap_pkey` | roadmap | PK | `id` | − |
| `idx_roadmap_tenant_quarters` | roadmap | BT/PT | `(tenant_id, fiscal_year, fiscal_quarter)` | `WHERE deleted_at IS NULL` |
| `sprint_state_pkey` | sprint_state | PK | `state_code` | − |

### 1.8 relation schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `relation_pkey` | relation | PK | `id` | − |
| `uq_relation_tenant_source_target_type` | relation | UK/PT | `(tenant_id, source_type, source_id, target_type, target_id, relation_type)` | `WHERE deleted_at IS NULL` |
| `idx_relation_tenant_target` | relation | BT/PT | `(tenant_id, target_type, target_id)` | `WHERE deleted_at IS NULL` |
| `idx_relation_tenant_type` | relation | BT/PT | `(tenant_id, relation_type)` | `WHERE deleted_at IS NULL` |
| `dependency_mv_idx` | dependency (MV) | BT | `(tenant_id, blocked_work_item_id)` | − |

### 1.9 comment schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `comment_pkey` | comment | PK | `id` | − |
| `idx_comment_tenant_target` | comment | BT/PT | `(tenant_id, target_type, target_id, created_at DESC)` | `WHERE deleted_at IS NULL` |
| `idx_comment_tenant_author` | comment | BT/PT | `(tenant_id, author_id, created_at DESC)` | `WHERE deleted_at IS NULL` |
| `mention_pkey` | mention | PK | `id` | − |
| `idx_mention_tenant_mentioned` | mention | BT/PT | `(tenant_id, mentioned_user_id, read_at)` | `WHERE deleted_at IS NULL` |
| `attachment_pkey` | attachment | PK | `id` | − |
| `idx_attachment_tenant_target` | attachment | BT/PT | `(tenant_id, target_type, target_id)` | `WHERE deleted_at IS NULL` |
| `comment_visibility_pkey` | comment_visibility | PK | `visibility_code` | − |

### 1.10 search schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `search_index_pkey` | search_index | PK | `id` | − |
| `idx_search_index_tenant_type` | search_index | BT/PT | `(tenant_id, resource_type, indexed_at DESC)` | `WHERE deleted_at IS NULL` |
| `idx_search_index_vector_gin` | search_index | GIN | `(search_vector)` | − |
| `uq_search_index_tenant_resource` | search_index | UK/PT | `(tenant_id, resource_type, resource_id)` | `WHERE deleted_at IS NULL` |

### 1.11 audit schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `audit_event_pkey` | audit_event | PK | `id` | − |
| `idx_audit_event_tenant_created_brin` | audit_event | BRIN | `(created_at)` | − |
| `idx_audit_event_tenant_actor` | audit_event | BT | `(tenant_id, actor_id, created_at DESC)` | − |
| `idx_audit_event_tenant_target` | audit_event | BT | `(tenant_id, target_type, target_id, created_at DESC)` | − |
| `idx_audit_event_payload_gin` | audit_event | GIN | `(payload_json)` | − |
| `ai_audit_metadata_pkey` | ai_audit_metadata | PK | `id` | − |
| `idx_ai_audit_metadata_tenant_session` | ai_audit_metadata | BT | `(tenant_id, agent_session_id)` | − |
| `idx_ai_audit_metadata_ai_questions_gin` | ai_audit_metadata | GIN | `(ai_questions_answers_json)` | − |
| `audit_event_outbox_pkey` | audit_event_outbox | PK | `outbox_id` | − |
| `idx_outbox_unpublished` | audit_event_outbox | PT | `(created_at)` | `WHERE published_at IS NULL` |
| `idx_outbox_retry` | audit_event_outbox | PT | `(retry_count, created_at)` | `WHERE published_at IS NULL` |

### 1.12 integration schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `integration_pkey` | integration | PK | `id` | − |
| `uq_integration_tenant_provider` | integration | UK/PT | `(tenant_id, provider_id, external_id)` | `WHERE deleted_at IS NULL` |
| `idx_integration_tenant_status` | integration | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `integration_sync_state_pkey` | integration_sync_state | PK | `id` | − |
| `uq_integration_sync_state_resource` | integration_sync_state | UK/PT | `(integration_id, resource_type, resource_id)` | `WHERE deleted_at IS NULL` |
| `idx_integration_sync_state_pending` | integration_sync_state | PT | `(integration_id, next_sync_at)` | `WHERE status = 'PENDING' AND deleted_at IS NULL` |
| `integration_status_pkey` | integration_status | PK | `status_code` | − |

### 1.13 automation schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `automation_rule_pkey` | automation_rule | PK | `id` | − |
| `uq_automation_rule_tenant_key` | automation_rule | UK/PT | `(tenant_id, project_id, rule_key)` | `WHERE deleted_at IS NULL` |
| `idx_automation_rule_tenant_active` | automation_rule | PT | `(tenant_id, status)` | `WHERE status = 'ENABLED' AND deleted_at IS NULL` |
| `automation_trigger_pkey` | automation_trigger | PK | `id` | − |
| `idx_automation_trigger_rule_order` | automation_trigger | BT | `(rule_id, order_index)` | − |
| `automation_action_pkey` | automation_action | PK | `id` | − |
| `idx_automation_action_rule_order` | automation_action | BT | `(rule_id, order_index)` | − |
| `rule_status_pkey` | rule_status | PK | `status_code` | − |

### 1.14 identity schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `user_pkey` | user | PK | `id` | − |
| `uq_user_tenant_email` | user | UK | `(tenant_id, email)` | − (citext) |
| `idx_user_tenant_status` | user | BT | `(tenant_id, status)` | − |
| `idx_user_tenant_display_name` | user | BT | `(tenant_id, display_name)` | − |
| `device_pkey` | device | PK | `id` | − |
| `uq_device_tenant_fingerprint` | device | UK | `(tenant_id, device_fingerprint)` | − |
| `idx_device_tenant_user` | device | BT | `(tenant_id, user_id)` | − |
| `idx_device_tenant_last_seen` | device | BT | `(tenant_id, last_seen_at DESC)` | − |
| `device_binding_pkey` | device_binding | PK | `id` | − |
| `uq_device_binding_triple` | device_binding | UK | `(user_id, device_id, credential_id)` | − (三重 1:1:1) |
| `credential_pkey` | credential | PK | `id` | − |
| `uq_credential_tenant_name` | credential | UK | `(tenant_id, name)` | − (citext) |
| `idx_credential_tenant_provider` | credential | BT | `(tenant_id, provider_id)` | − |
| `user_session_pkey` | user_session | PK | `id` | − |
| `idx_user_session_tenant_user_expires` | user_session | BT | `(tenant_id, user_id, expires_at)` | − |
| `idx_user_session_token` | user_session | UK | `(session_token_hash)` | − |

### 1.15 notification schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `notification_channel_pkey` | notification_channel | PK | `id` | − |
| `uq_notification_channel_tenant_key` | notification_channel | UK/PT | `(tenant_id, channel_key)` | `WHERE deleted_at IS NULL` |
| `notification_template_pkey` | notification_template | PK | `id` | − |
| `uq_notification_template_tenant_key` | notification_template | UK/PT | `(tenant_id, template_key)` | `WHERE deleted_at IS NULL` |
| `notification_pkey` | notification | PK | `id` | − |
| `idx_notification_tenant_recipient_unread` | notification | BT/PT | `(tenant_id, recipient_id, created_at DESC)` | `WHERE status = 'PENDING' AND read_at IS NULL` |
| `idx_notification_tenant_audience_scope` | notification | BT/PT | `(tenant_id, audience_scope)` | `WHERE deleted_at IS NULL` |
| `notification_status_pkey` | notification_status | PK | `status_code` | − |

### 1.16 permission schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `role_pkey` | role | PK | `id` | − |
| `uq_role_tenant_key` | role | UK/PT | `(tenant_id, role_key)` | `WHERE deleted_at IS NULL` |
| `permission_pkey` | permission | PK | `code` | − (全局 enum) |
| `permission_scheme_pkey` | permission_scheme | PK | `id` | − |
| `uq_permission_scheme_tenant_key` | permission_scheme | UK/PT | `(tenant_id, scheme_key)` | `WHERE deleted_at IS NULL` |
| `idx_permission_scheme_tenant_default` | permission_scheme | PT | `(tenant_id)` | `WHERE is_default = TRUE AND deleted_at IS NULL` |
| `security_policy_pkey` | security_policy | PK | `id` | − |
| `uq_security_policy_tenant_key` | security_policy | UK/PT | `(tenant_id, policy_key)` | `WHERE deleted_at IS NULL` |

### 1.17 collaboration schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `presence_pkey` | presence | PK | `id` | − |
| `uq_presence_tenant_user` | presence | UK/PT | `(tenant_id, user_id)` | `WHERE deleted_at IS NULL` |
| `idx_presence_tenant_workspace` | presence | BT/PT | `(tenant_id, workspace_id, last_seen_at DESC)` | `WHERE deleted_at IS NULL` |
| `realtime_subscription_pkey` | realtime_subscription | PK | `id` | − |
| `idx_realtime_subscription_tenant_user` | realtime_subscription | BT/PT | `(tenant_id, user_id)` | `WHERE expires_at > NOW()` |

### 1.18 scm schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `repository_pkey` | repository | PK | `id` | − |
| `uq_repository_tenant_external` | repository | UK/PT | `(tenant_id, provider_id, external_id)` | `WHERE deleted_at IS NULL` |
| `idx_repository_tenant_project` | repository | BT/PT | `(tenant_id, project_id)` | `WHERE deleted_at IS NULL` |
| `branch_pkey` | branch | PK | `id` | − |
| `uq_branch_repo_name` | branch | UK/PT | `(repository_id, name)` | `WHERE deleted_at IS NULL` |
| `commit_pkey` | commit | PK | `id` | − |
| `uq_commit_repo_sha` | commit | UK/PT | `(repository_id, sha)` | `WHERE deleted_at IS NULL` |
| `idx_commit_repo_parent` | commit | BT | `(repository_id, parent_sha)` | − |
| `pull_request_pkey` | pull_request | PK | `id` | − |
| `uq_pull_request_repo_external` | pull_request | UK/PT | `(repository_id, external_id)` | `WHERE deleted_at IS NULL` |
| `idx_pull_request_tenant_status` | pull_request | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_pull_request_repo_source_target` | pull_request | BT | `(repository_id, source_branch, target_branch)` | − |
| `review_pkey` | review | PK | `id` | − |
| `idx_review_pr_created` | review | BT | `(pull_request_id, created_at DESC)` | − |
| `pipeline_pkey` | pipeline | PK | `id` | − |
| `idx_pipeline_repo_commit` | pipeline | BT | `(repository_id, commit_sha)` | − |
| `idx_pipeline_tenant_status` | pipeline | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `webhook_event_pkey` | webhook_event | PK | `id` | − |
| `idx_webhook_event_tenant_received` | webhook_event | BT | `(tenant_id, received_at)` | − |
| `pull_request_status_pkey` | pull_request_status | PK | `status_code` | − |

### 1.19 development schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `development_execution_pkey` | development_execution | PK | `id` | − |
| `uq_development_execution_tenant_key` | development_execution | UK/PT | `(tenant_id, execution_key)` | `WHERE deleted_at IS NULL` |
| `change_set_pkey` | change_set | PK | `id` | − |
| `uq_change_set_tenant_key` | change_set | UK/PT | `(tenant_id, change_set_key)` | `WHERE deleted_at IS NULL` |
| `idx_change_set_tenant_worktree` | change_set | BT/PT | `(tenant_id, worktree_id)` | `WHERE deleted_at IS NULL` |
| `idx_change_set_tenant_commit` | change_set | BT/PT | `(tenant_id, commit_id)` | `WHERE deleted_at IS NULL` |
| `file_change_pkey` | file_change | PK | `id` | − |
| `idx_file_change_change_set_path` | file_change | BT | `(change_set_id, file_path)` | − |
| `symbol_change_pkey` | symbol_change | PK | `id` | − |
| `idx_symbol_change_change_set` | symbol_change | BT | `(change_set_id, symbol_id)` | − |
| `idx_symbol_change_tenant_path` | symbol_change | BT/PT | `(tenant_id, file_path)` | `WHERE deleted_at IS NULL` |
| `risk_signal_pkey` | risk_signal | PK | `id` | − |
| `idx_risk_signal_tenant_change_set` | risk_signal | BT/PT | `(tenant_id, change_set_id, severity)` | `WHERE deleted_at IS NULL` |
| `change_set_link_pkey` | change_set_link | PK | `id` | − |
| `uq_change_set_link_pair` | change_set_link | UK | `(source_change_set_id, target_change_set_id, link_type)` | − |
| `symbol_index_pkey` | symbol_index | PK | `id` | − |
| `idx_symbol_index_tenant_path_gist` | symbol_index | GiST | `(file_path ltree)` | − |
| `idx_symbol_index_tenant_qualified_name` | symbol_index | BT/PT | `(tenant_id, qualified_name)` | `WHERE deleted_at IS NULL` |
| `repository_context_pkey` | repository_context | PK | `id` | − |
| `idx_repository_context_tenant_repo` | repository_context | BT/PT | `(tenant_id, repository_id)` | `WHERE deleted_at IS NULL` |
| `idx_repository_context_tree_gist` | repository_context | GiST | `(file_tree ltree)` | − |
| `development_context_pkey` | development_context | PK | `id` | − |
| `idx_development_context_tenant_change_set` | development_context | BT/PT | `(tenant_id, change_set_id)` | `WHERE deleted_at IS NULL` |

### 1.20 worktree schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `worktree_pkey` | worktree | PK | `id` | − |
| `uq_worktree_tenant_key` | worktree | UK/PT | `(tenant_id, worktree_key)` | `WHERE deleted_at IS NULL` |
| `idx_worktree_tenant_status` | worktree | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_worktree_tenant_assignee` | worktree | BT/PT | `(tenant_id, assignee_id)` | `WHERE deleted_at IS NULL` |
| `idx_worktree_tenant_work_item` | worktree | BT/PT | `(tenant_id, work_item_id)` | `WHERE deleted_at IS NULL` |
| `idx_worktree_path_gist` | worktree | GiST | `(local_path ltree)` | − |
| `worktree_status_observed_pkey` | worktree_status_observed | PK | `id` | − |
| `idx_worktree_status_observed_tenant_worktree_recent` | worktree_status_observed | BT/PT | `(tenant_id, worktree_id, last_observed_at DESC)` | `WHERE deleted_at IS NULL` |
| `worktree_conflict_pkey` | worktree_conflict | PK | `id` | − |
| `idx_worktree_conflict_tenant_worktree` | worktree_conflict | BT/PT | `(tenant_id, worktree_id, severity)` | `WHERE deleted_at IS NULL` |
| `worktree_heatmap_idx` | worktree_heatmap (MV) | BT | `(repository_id, file_path)` | − |
| `worktree_status_pkey` | worktree_status | PK | `status_code` | − |

### 1.21 agent schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `agent_pkey` | agent | PK | `id` | − |
| `uq_agent_tenant_key` | agent | UK/PT | `(tenant_id, agent_key)` | `WHERE deleted_at IS NULL` |
| `idx_agent_tenant_type` | agent | BT/PT | `(tenant_id, type)` | `WHERE deleted_at IS NULL` |
| `agent_session_pkey` | agent_session | PK | `id` | − |
| `uq_agent_session_tenant_key` | agent_session | UK/PT | `(tenant_id, session_key)` | `WHERE deleted_at IS NULL` |
| `idx_agent_session_tenant_worktree` | agent_session | BT/PT | `(tenant_id, worktree_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_agent_session_tenant_status` | agent_session | BT/PT | `(tenant_id, status, started_at DESC)` | `WHERE deleted_at IS NULL` |
| `idx_agent_session_token_usage_gin` | agent_session | GIN | `(token_usage)` | − |
| `agent_session_event_pkey` | agent_session_event | PK | `id` | − |
| `idx_agent_session_event_tenant_session_seq` | agent_session_event | BT | `(tenant_id, agent_session_id, sequence_number)` | − |
| `agent_policy_pkey` | agent_policy | PK | `id` | − |
| `uq_agent_policy_tenant_key` | agent_policy | UK/PT | `(tenant_id, policy_key)` | `WHERE deleted_at IS NULL` |
| `agent_session_status_pkey` | agent_session_status | PK | `status_code` | − |

### 1.22 feedback schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `feedback_pkey` | feedback | PK | `id` | − |
| `uq_feedback_tenant_key` | feedback | UK/PT | `(tenant_id, feedback_key)` | `WHERE deleted_at IS NULL` |
| `idx_feedback_tenant_worktree_status` | feedback | BT/PT | `(tenant_id, worktree_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_feedback_tenant_recipient_unread` | feedback | BT/PT | `(tenant_id, recipient_id, status)` | `WHERE status IN ('OPEN','ACKED') AND deleted_at IS NULL` |
| `idx_feedback_metadata_gin` | feedback | GIN | `(metadata_json)` | − |
| `feedback_consumed_event_pkey` | feedback_consumed_event | PK | `id` | − |
| `idx_feedback_consumed_event_tenant_feedback` | feedback_consumed_event | BT | `(tenant_id, feedback_id, consumed_at DESC)` | − |
| `feedback_inbox_item_idx` | feedback_inbox_item (MV) | BT | `(tenant_id, recipient_id, status, priority DESC)` | − |
| `feedback_status_pkey` | feedback_status | PK | `status_code` | − |

### 1.23 context schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `context_packet_pkey` | context_packet | PK | `id` | − |
| `uq_context_packet_tenant_key` | context_packet | UK/PT | `(tenant_id, packet_key)` | `WHERE deleted_at IS NULL` |
| `idx_context_packet_tenant_session` | context_packet | BT/PT | `(tenant_id, agent_session_id)` | `WHERE deleted_at IS NULL` |
| `provenance_entry_pkey` | provenance_entry | PK | `id` | − |
| `idx_provenance_entry_tenant_resource` | provenance_entry | BT/PT | `(tenant_id, resource_type, resource_id)` | `WHERE deleted_at IS NULL` |
| `idx_provenance_entry_tenant_source_gin` | provenance_entry | GIN | `(source_json)` | − |
| `decision_pkey` | decision | PK | `id` | − |
| `idx_decision_tenant_worktree` | decision | BT/PT | `(tenant_id, worktree_id, decided_at DESC)` | `WHERE deleted_at IS NULL` |
| `decision_status_pkey` | decision_status | PK | `status_code` | − |

### 1.24 validation schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `validation_result_pkey` | validation_result | PK | `id` | − |
| `uq_validation_result_tenant_key` | validation_result | UK/PT | `(tenant_id, result_key)` | `WHERE deleted_at IS NULL` |
| `idx_validation_result_tenant_worktree` | validation_result | BT/PT | `(tenant_id, worktree_id, status)` | `WHERE deleted_at IS NULL` |
| `idx_validation_result_tenant_status` | validation_result | BT/PT | `(tenant_id, status, started_at DESC)` | `WHERE deleted_at IS NULL` |
| `validation_evidence_pkey` | validation_evidence | PK | `id` | − |
| `idx_validation_evidence_validation_type` | validation_evidence | BT | `(validation_id, evidence_type)` | − |
| `acceptance_coverage_pkey` | acceptance_coverage | PK | `id` | − |
| `uq_acceptance_coverage_tenant` | acceptance_coverage | UK/PT | `(tenant_id, work_item_id, acceptance_criterion_id)` | `WHERE deleted_at IS NULL` |
| `validation_policy_pkey` | validation_policy | PK | `id` | − |
| `uq_validation_policy_tenant_key` | validation_policy | UK/PT | `(tenant_id, policy_key)` | `WHERE deleted_at IS NULL` |
| `acceptance_coverage_report_idx` | acceptance_coverage_report (MV) | BT | `(tenant_id, work_item_id)` | − |
| `validation_status_pkey` | validation_status | PK | `status_code` | − |

### 1.25 local_runtime schema

| インデックス名 | テーブル | 種別 | キー列 | 包含 / 条件 |
|---|---|---|---|---|
| `runtime_pkey` | runtime | PK | `id` | − |
| `uq_runtime_tenant_device` | runtime | UK/PT | `(tenant_id, device_id)` | `WHERE deleted_at IS NULL` |
| `idx_runtime_tenant_status` | runtime | BT/PT | `(tenant_id, status)` | `WHERE deleted_at IS NULL` |
| `runtime_command_pkey` | runtime_command | PK | `id` | − |
| `idx_runtime_command_tenant_runtime_status` | runtime_command | BT/PT | `(tenant_id, runtime_id, status)` | `WHERE deleted_at IS NULL` |
| `runtime_observation_pkey` | runtime_observation | PK | `id` | − |
| `idx_runtime_observation_tenant_runtime_recent` | runtime_observation | BT/PT | `(tenant_id, runtime_id, observed_at DESC)` | `WHERE deleted_at IS NULL` |
| `reconciliation_report_pkey` | reconciliation_report | PK | `id` | − |
| `idx_reconciliation_report_tenant_runtime` | reconciliation_report | BT/PT | `(tenant_id, runtime_id, created_at DESC)` | `WHERE deleted_at IS NULL` |
| `runtime_status_pkey` | runtime_status | PK | `status_code` | − |

---

## 2. インデックス戦略まとめ

### 2.1 共通パターン

| パターン | 適用 | 効果 |
|---|---|---|
| **tenant_id 起首複合 BT** | 全 RLS 必須テーブル | 13 類 RLS フィルタ + 業務検索の同時最適化 |
| **`deleted_at IS NULL` 部分インデックス** | 全 SoR テーブル | 物理削除との互換（§3.1.5） |
| **業務一意制約 UK + 業務キー** | 全 Entity | API 重複作成防止 |
| **GIN (JSONB / tsvector)** | search / feedback / audit / agent 等 | メタデータ検索 |
| **GiST (ltree)** | worktree.path / development.tree | パス / ツリー検索 |
| **BRIN (時系列)** | audit_event | 大規模 Append-only テーブル |
| **部分 PT (status)** | outbox / pending sync | 状態別高速絞り込み |

### 2.2 特殊インデックス

| テーブル | インデックス | 種別 | 用途 |
|---|---|---|---|
| `worktree.worktree` | `idx_worktree_path_gist` | GiST | ltree パス検索（§R-23.4） |
| `development.symbol_index` | `idx_symbol_index_tenant_path_gist` | GiST | Symbol パス検索 |
| `development.repository_context` | `idx_repository_context_tree_gist` | GiST | リポジトリツリー |
| `audit.audit_event` | `idx_audit_event_tenant_created_brin` | BRIN | 30 日 WORM 大規模時系列 |
| `audit.audit_event_outbox` | `idx_outbox_unpublished` | PT (status) | NATS 未推送高速取得 |
| `search.search_index` | `idx_search_index_vector_gin` | GIN (tsvector) | 全文検索 |

### 2.3 推奨追加インデックス（OPTIMIZATION-NOTES §X.X 参照）

- `work_item.work_item` カラム `(tenant_id, project_id, due_date)` 部分 — 期限順ボード表示用
- `agent.agent_session` カラム `(tenant_id, agent_id, started_at DESC)` 部分 — Agent 別履歴
- `feedback.feedback` カラム `(tenant_id, project_id, status)` 部分 — プロジェクト横断 Feedback

---

## 3. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：25 Schema 全 200+ インデックス列挙 | per 2026-09-01 15:30 JST Ulysses 拍板 |

---
