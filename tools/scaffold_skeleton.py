#!/usr/bin/env python3
"""
Scaffold the 25 domain-* + 3 supporting crates for the Star platform.

Idempotent: re-running overwrites existing files.
Run from anywhere; the path is hardcoded to D:/Star-worktrees/dev-skeleton.
"""
from __future__ import annotations

import os
from pathlib import Path
from textwrap import dedent

ROOT = Path("D:/Star-worktrees/dev-skeleton")
CRATES = ROOT / "crates"

# ----------------------------------------------------------------------
# Crate specifications.
#
# Each entry captures everything we need to render Cargo.toml + lib.rs:
#   - chinese_name: 中文名(用于 doc comment)
#   - spec_section: spec doc 章节号(用于 doc comment)
#   - basic_design: basic-design 章节号
#   - data_design:  data-design 章节号
#   - api_design:   api-design 章节号
#   - entities:     主要聚合根(多个)
#   - cmd_port:     CommandPort 名
#   - cmd_methods:  CommandPort 方法签名(占位)
#   - query_port:   QueryPort 名
#   - query_methods:QueryPort 方法签名(占位)
#   - events:       关键 Domain Event subject
#   - deps:         上游 domain-* 依赖(仅 doc 引用,骨架阶段不实际 Cargo 依赖)
#   - key_invariants: 关键不变量(2-4 条)
# ----------------------------------------------------------------------

CRATE_SPECS: list[dict] = [
    # ============== 核心域 Core Domain ==============
    {
        "name": "domain-work-item",
        "chinese_name": "WorkItem 领域",
        "spec_section": "§4.9 / §7.2 (WorkItem 默认三态)",
        "basic_design": "§2.1 / §4.9 / §7.2",
        "data_design": "§4.4 (`work_item` schema)",
        "api_design": "§3.5 (CRUD + 状态机 + AC)",
        "entities": ["WorkItem", "Requirement", "AcceptanceCriterion", "BusinessGoal"],
        "cmd_port": "WorkItemCommandPort",
        "cmd_methods": [
            ("create_work_item", "CreateWorkItemCommand", "WorkItemId"),
            ("update_work_item", "UpdateWorkItemCommand", "WorkItem"),
            ("delete_work_item", "WorkItemId", "()"),
            ("transition_status", "TransitionStatusCommand", "WorkItem"),
            ("bulk_update", "WorkItemBulkUpdate", "BulkResult"),
            ("link_repository", "LinkRepositoryCommand", "WorkItem"),
            ("create_requirement", "CreateRequirementCommand", "RequirementId"),
            ("create_acceptance_criterion", "CreateAcceptanceCriterionCommand", "AcceptanceCriterionId"),
        ],
        "query_port": "WorkItemQueryPort",
        "query_methods": [
            ("list_by_project", "ListWorkItemQuery", "Vec<WorkItem>"),
            ("get_by_id", "WorkItemId", "WorkItem"),
            ("list_transitions", "WorkItemId", "Vec<Transition>"),
            ("list_requirements", "WorkItemId", "Vec<Requirement>"),
            ("list_acceptance_criteria", "WorkItemId", "Vec<AcceptanceCriterion>"),
            ("list_business_goals", "ListBusinessGoalQuery", "Vec<BusinessGoal>"),
        ],
        "events": [
            "star.events.work_item.work_item.created.v1",
            "star.events.work_item.work_item.status_changed.v1",
            "star.events.work_item.work_item.worktree_linked.v1",
            "star.events.work_item.work_item.deleted.v1",
            "star.events.work_item.acceptance_criterion.created.v1",
            "star.events.work_item.acceptance_criterion.covered.v1",
        ],
        "deps": ["domain-workflow", "domain-project", "domain-permission"],
        "key_invariants": [
            "WorkItem 默认三态 TODO → IN_PROGRESS → DONE(REQ-WF-001,§7.2)",
            "WorkItem ≠ Git Branch(§44.3);1 WorkItem → 0/1/N Repository",
            "1 WorkItem → 0/1/N Worktree;Worktree Status 独立于 WorkItem Status(§22.2)",
            "任何 WorkItem INSERT/UPDATE 必须带 tenant_id(§6.1,REQ-SEC-001)",
        ],
        "extra_refs": "WorkItem 3 态 = TODO / IN_PROGRESS / DONE(§4.9.3 修复后)",
    },
    {
        "name": "domain-worktree",
        "chinese_name": "Worktree 领域",
        "spec_section": "§22 Worktree 生命周期",
        "basic_design": "§2.1 / §4.1 / §7 状态机",
        "data_design": "§4.2 (`worktree` schema)",
        "api_design": "§3.6 (Worktree CRUD + 状态机)",
        "entities": ["Worktree", "ConflictState", "HealthState"],
        "cmd_port": "WorktreeCommandPort",
        "cmd_methods": [
            ("create_worktree", "CreateWorktreeCommand", "WorktreeId"),
            ("update_worktree", "UpdateWorktreeCommand", "Worktree"),
            ("delete_worktree", "WorktreeId", "()"),
            ("transition_status", "TransitionWorktreeStatusCommand", "Worktree"),
            ("attach_runtime", "AttachRuntimeCommand", "Worktree"),
            ("register_observation", "RegisterObservationCommand", "Worktree"),
        ],
        "query_port": "WorktreeQueryPort",
        "query_methods": [
            ("get_by_id", "WorktreeId", "WorkItem"),
            ("list_by_work_item", "WorkItemId", "Vec<WorkItem>"),
            ("list_by_runtime", "RuntimeId", "Vec<WorkItem>"),
            ("list_observations", "WorktreeId", "Vec<RuntimeObservation>"),
        ],
        "events": [
            "star.events.worktree.worktree.created.v1",
            "star.events.worktree.worktree.status_changed.v1",
            "star.events.worktree.worktree.observation_registered.v1",
        ],
        "deps": ["domain-work-item", "domain-scm", "domain-development"],
        "key_invariants": [
            "Worktree Status 独立于 WorkItem Status(§22.2,REQ-WF-002)",
            "Worktree 17 状态机:Created/Initializing/Ready/Dirty/Cleaning/Conflict/.../Archived(§7)",
            "1 Worktree 绑定 1 Runtime(Local Daemon);1 Runtime 可承载 N Worktree(§23)",
        ],
        "extra_refs": "Worktree 17 状态机(§7.6):详见 lib.rs 注释",
    },
    {
        "name": "domain-agent",
        "chinese_name": "Agent 领域",
        "spec_section": "§4.2 / §24 Agent 策略",
        "basic_design": "§2.1 / §4.2 / §7 状态机",
        "data_design": "§4.3 (`agent` schema)",
        "api_design": "§3.7 (Agent CRUD + AgentSession 状态机)",
        "entities": ["Agent", "AgentSession", "AgentPolicy"],
        "cmd_port": "AgentCommandPort",
        "cmd_methods": [
            ("register_agent", "RegisterAgentCommand", "AgentId"),
            ("update_agent_policy", "UpdateAgentPolicyCommand", "Agent"),
            ("start_session", "StartAgentSessionCommand", "AgentSessionId"),
            ("submit_tool_result", "SubmitToolResultCommand", "AgentSession"),
            ("submit_feedback", "SubmitFeedbackToSessionCommand", "AgentSession"),
            ("abort_session", "AbortSessionCommand", "AgentSession"),
            ("complete_session", "CompleteSessionCommand", "AgentSession"),
        ],
        "query_port": "AgentQueryPort",
        "query_methods": [
            ("get_agent", "AgentId", "Agent"),
            ("list_sessions", "ListAgentSessionQuery", "Vec<AgentSession>"),
            ("get_session", "AgentSessionId", "AgentSession"),
            ("list_agent_policy", "ProjectId", "Vec<AgentPolicy>"),
        ],
        "events": [
            "star.events.agent.agent_session.started.v1",
            "star.events.agent.agent_session.state_changed.v1",
            "star.events.agent.agent_session.completed.v1",
        ],
        "deps": ["domain-worktree", "domain-feedback", "domain-validation"],
        "key_invariants": [
            "1 AgentSession → 1 Active Worktree(§21,REQ-DEV-003)",
            "AgentSession 14 状态机(§7.4 修复后 F-08):Created/Starting/Running/.../Timeout",
            "Agent 操作必须 Application/Authorization 强制(§11,REQ-PERM-002)",
        ],
        "extra_refs": "AgentSession 14 状态(§7.4,已修复 F-08);1 Session → 1 Active Worktree",
    },
    {
        "name": "domain-feedback",
        "chinese_name": "Feedback 领域",
        "spec_section": "§25 Feedback Model",
        "basic_design": "§2.1 / §4.3",
        "data_design": "§4.5 (`feedback` schema)",
        "api_design": "§3.8 (Feedback CRUD + Resolution)",
        "entities": ["Feedback", "FeedbackResolution"],
        "cmd_port": "FeedbackCommandPort",
        "cmd_methods": [
            ("submit_feedback", "SubmitFeedbackCommand", "FeedbackId"),
            ("resolve_feedback", "ResolveFeedbackCommand", "FeedbackResolution"),
            ("link_to_validation", "LinkFeedbackValidationCommand", "Feedback"),
            ("bulk_resolve", "BulkResolveFeedbackCommand", "BulkResult"),
        ],
        "query_port": "FeedbackQueryPort",
        "query_methods": [
            ("get_by_id", "FeedbackId", "Feedback"),
            ("list_by_target", "ListFeedbackByTargetQuery", "Vec<Feedback>"),
            ("list_unresolved", "ProjectId", "Vec<Feedback>"),
        ],
        "events": [
            "star.events.feedback.feedback.submitted.v1",
            "star.events.feedback.feedback.resolved.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree", "domain-agent"],
        "key_invariants": [
            "Feedback Target 覆盖 WorkItem→Diff Hunk 全粒度(§25.1)",
            "Feedback 6 状态(§7.5):Pending/InReview/Accepted/Rejected/Superseded/Withdrawn",
            "Feedback 是 Context 的输入源之一,不被 Context 反向依赖(§2.3 禁线)",
        ],
        "extra_refs": "Feedback 6 状态(§7.5);覆盖 WorkItem/Diff Hunk 全粒度(§25.1)",
    },
    {
        "name": "domain-context",
        "chinese_name": "Context Compiler 领域",
        "spec_section": "§26 Context Model + Token Budget P0-P4",
        "basic_design": "§2.1 / §4.4 / §4.4.4(P0-P4)",
        "data_design": "§4.6 (`context_packet` schema)",
        "api_design": "§3.9 (Context Packet 编译 + Decision Memory)",
        "entities": ["ContextPacket", "Decision", "ContextBudget"],
        "cmd_port": "ContextCommandPort",
        "cmd_methods": [
            ("compile_packet", "CompileContextPacketCommand", "ContextPacketId"),
            ("record_decision", "RecordDecisionCommand", "DecisionId"),
            ("supersede_decision", "SupersedeDecisionCommand", "Decision"),
        ],
        "query_port": "ContextQueryPort",
        "query_methods": [
            ("get_packet", "ContextPacketId", "ContextPacket"),
            ("list_decisions", "ListDecisionQuery", "Vec<Decision>"),
            ("estimate_budget", "EstimateBudgetQuery", "ContextBudget"),
        ],
        "events": [
            "star.events.context.packet.compiled.v1",
            "star.events.context.decision.recorded.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree", "domain-feedback", "domain-validation"],
        "key_invariants": [
            "Context Provenance 强制可追溯(§26.3)",
            "Token Budget P0-P4 五层结构(§4.4.4,F-02 修复后)",
            "Untrusted Repo Content(P5)单独隔离,绝不入 P0-P4(§26.1)",
        ],
        "extra_refs": "P0-P4 五层(§4.4.4);P5 单独(§26.1);Decision 3 态(§7.5)",
    },
    {
        "name": "domain-validation",
        "chinese_name": "Validation 领域",
        "spec_section": "§27 Validation Model",
        "basic_design": "§2.1 / §4.5 / §4.10.7",
        "data_design": "§4.7 (`validation` schema)",
        "api_design": "§3.10 (Validation Evidence + Coverage)",
        "entities": ["ValidationResult", "AcceptanceCoverage"],
        "cmd_port": "ValidationCommandPort",
        "cmd_methods": [
            ("submit_evidence", "SubmitValidationEvidenceCommand", "ValidationResultId"),
            ("mark_passed", "MarkValidationPassedCommand", "ValidationResult"),
            ("mark_failed", "MarkValidationFailedCommand", "ValidationResult"),
            ("record_coverage", "RecordCoverageCommand", "AcceptanceCoverage"),
        ],
        "query_port": "ValidationQueryPort",
        "query_methods": [
            ("get_by_id", "ValidationResultId", "ValidationResult"),
            ("list_by_work_item", "WorkItemId", "Vec<ValidationResult>"),
            ("compute_coverage", "WorkItemId", "AcceptanceCoverage"),
        ],
        "events": [
            "star.events.validation.result.submitted.v1",
            "star.events.validation.result.passed.v1",
            "star.events.validation.result.failed.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree", "domain-agent"],
        "key_invariants": [
            "AI 自我报告不构成完成(§27.3,VAL-001)",
            "Acceptance Coverage 必须由 Validation Evidence 驱动(REQ-VAL-002)",
            "ValidationFailed 触发 Outbox 通知(§2.4)",
        ],
        "extra_refs": "AI 自我报告不构成完成(VAL-001,F-06 修复后)",
    },
    # ============== 支撑域 Supporting Domain ==============
    {
        "name": "domain-scm",
        "chinese_name": "SCM Adapter 领域",
        "spec_section": "§18-19 SCM Integration",
        "basic_design": "§2.1 / §4.7 / §4.8",
        "data_design": "§4.8 (`scm` schema)",
        "api_design": "§3.11 (Repository / Branch / Commit / PR)",
        "entities": ["Repository", "Branch", "Commit", "PullRequest", "Review", "Pipeline"],
        "cmd_port": "ScmCommandPort",
        "cmd_methods": [
            ("link_repository", "LinkRepositoryCommand", "RepositoryId"),
            ("sync_repository", "SyncRepositoryCommand", "Repository"),
            ("upsert_branch", "UpsertBranchCommand", "Branch"),
            ("upsert_pull_request", "UpsertPullRequestCommand", "PullRequest"),
            ("record_pipeline_run", "RecordPipelineRunCommand", "Pipeline"),
        ],
        "query_port": "ScmQueryPort",
        "query_methods": [
            ("get_repository", "RepositoryId", "Repository"),
            ("list_branches", "RepositoryId", "Vec<Branch>"),
            ("list_pull_requests", "ListPullRequestQuery", "Vec<PullRequest>"),
        ],
        "events": [
            "star.events.scm.repository.linked.v1",
            "star.events.scm.repository.synced.v1",
            "star.events.scm.pull_request.opened.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree"],
        "key_invariants": [
            "Domain 层无厂商对象(§19.1,REQ-SCM-002)",
            "External SCM 是事实源,非镜像(§19.2,§30.6)",
            "PR/Review/Pipeline 与 Provider 弱耦合(§18.2)",
        ],
        "extra_refs": "Domain 层无厂商对象(§19.1,REQ-SCM-002)",
    },
    {
        "name": "domain-development",
        "chinese_name": "Development Execution + Development Context 领域",
        "spec_section": "§20-21 Development Domain",
        "basic_design": "§2.1 / §4.9.5 / §4.10.4",
        "data_design": "§4.9 (`development` schema)",
        "api_design": "§3.12 (ChangeSet / Link / SymbolIndex)",
        "entities": ["DevelopmentExecution", "ChangeSet", "Link", "SymbolIndex", "RepositoryContext", "DevelopmentContext"],
        "cmd_port": "DevelopmentCommandPort",
        "cmd_methods": [
            ("create_execution", "CreateExecutionCommand", "ExecutionId"),
            ("record_change_set", "RecordChangeSetCommand", "ChangeSet"),
            ("create_link", "CreateLinkCommand", "LinkId"),
            ("register_symbol_index", "RegisterSymbolIndexCommand", "SymbolIndex"),
        ],
        "query_port": "DevelopmentQueryPort",
        "query_methods": [
            ("get_execution", "ExecutionId", "DevelopmentExecution"),
            ("list_change_sets", "ListChangeSetQuery", "Vec<ChangeSet>"),
            ("get_symbol_index", "RepositoryId", "SymbolIndex"),
        ],
        "events": [
            "star.events.development.execution.created.v1",
            "star.events.development.change_set.recorded.v1",
            "star.events.development.link.created.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree", "domain-agent", "domain-scm"],
        "key_invariants": [
            "ChangeSet ≠ Git Diff(§21.1)",
            "Symbol-aware Context 逐步演进(§21.2)",
            "Development Context(§20)合并入本 crate(SymbolIndex/RepositoryContext/DevelopmentContext,F-02 修复后)",
        ],
        "extra_refs": "Development Context(§20)合并入本 crate(F-02 修复)",
    },
    {
        "name": "domain-workflow",
        "chinese_name": "Workflow 领域",
        "spec_section": "§9 Workflow 状态机",
        "basic_design": "§2.1 / §4.9.3 / §7.6",
        "data_design": "§4.10 (`workflow` schema)",
        "api_design": "§3.13 (Workflow Definition + Transition)",
        "entities": ["WorkflowDefinition", "State", "Transition"],
        "cmd_port": "WorkflowCommandPort",
        "cmd_methods": [
            ("create_workflow", "CreateWorkflowCommand", "WorkflowDefinitionId"),
            ("update_workflow", "UpdateWorkflowCommand", "WorkflowDefinition"),
            ("add_state", "AddStateCommand", "State"),
            ("add_transition", "AddTransitionCommand", "Transition"),
        ],
        "query_port": "WorkflowQueryPort",
        "query_methods": [
            ("get_workflow", "WorkflowDefinitionId", "WorkflowDefinition"),
            ("list_transitions", "WorkflowDefinitionId", "Vec<Transition>"),
            ("list_states", "WorkflowDefinitionId", "Vec<State>"),
        ],
        "events": [
            "star.events.workflow.workflow.created.v1",
            "star.events.workflow.workflow.updated.v1",
        ],
        "deps": ["domain-work-item"],
        "key_invariants": [
            "Worktree Status 与 WorkItem Status 独立(REQ-WF-002)",
            "WorkflowDefinition 决定 WorkItem 合法迁移(§4.9.3,INV-WI-09)",
            "扩展状态由 ProjectPolicy 自定义,默认三态不可变(§4.9.3 修复后)",
        ],
        "extra_refs": "WorkItem 默认三态由本 crate WorkflowDefinition 判定(§4.9.3,INV-WI-09)",
    },
    {
        "name": "domain-board",
        "chinese_name": "Board 视图领域",
        "spec_section": "§9 / §10 Board 视图",
        "basic_design": "§2.1 / §4.10.1",
        "data_design": "§4.11 (`board` schema)",
        "api_design": "§3.14 (Board / Column / Swimlane)",
        "entities": ["Board", "Column", "Swimlane"],
        "cmd_port": "BoardCommandPort",
        "cmd_methods": [
            ("create_board", "CreateBoardCommand", "BoardId"),
            ("add_column", "AddColumnCommand", "Column"),
            ("add_swimlane", "AddSwimlaneCommand", "Swimlane"),
            ("reorder_columns", "ReorderColumnsCommand", "Board"),
        ],
        "query_port": "BoardQueryPort",
        "query_methods": [
            ("get_board", "BoardId", "Board"),
            ("list_by_project", "ProjectId", "Vec<Board>"),
        ],
        "events": [
            "star.events.board.board.created.v1",
            "star.events.board.column.moved.v1",
        ],
        "deps": ["domain-work-item", "domain-planning"],
        "key_invariants": [
            "Board 与 Sprint/Gantt 共享数据模型(§9,REQ-PLAN-003)",
            "Column 由 WorkflowDefinition 派生,不允许独立定义状态(§4.10.1)",
        ],
        "extra_refs": "Board/Sprint/Gantt 共享 WorkItem 投影(§9)",
    },
    {
        "name": "domain-planning",
        "chinese_name": "Sprint / Backlog / Roadmap 规划领域",
        "spec_section": "§9 / §10 / §11 Planning",
        "basic_design": "§2.1 / §4.10.1",
        "data_design": "§4.12 (`planning` schema)",
        "api_design": "§3.15 (Sprint / Backlog / Roadmap)",
        "entities": ["Sprint", "Backlog", "Roadmap", "Burndown"],
        "cmd_port": "PlanningCommandPort",
        "cmd_methods": [
            ("create_sprint", "CreateSprintCommand", "SprintId"),
            ("add_to_sprint", "AddToSprintCommand", "Sprint"),
            ("create_roadmap", "CreateRoadmapCommand", "RoadmapId"),
            ("record_burndown_snapshot", "RecordBurndownCommand", "Burndown"),
        ],
        "query_port": "PlanningQueryPort",
        "query_methods": [
            ("get_sprint", "SprintId", "Sprint"),
            ("list_active_sprints", "ProjectId", "Vec<Sprint>"),
            ("compute_burndown", "SprintId", "Burndown"),
        ],
        "events": [
            "star.events.planning.sprint.created.v1",
            "star.events.planning.sprint.started.v1",
        ],
        "deps": ["domain-work-item", "domain-board"],
        "key_invariants": [
            "Sprint 期间 WorkItem 状态由 Workflow 决定,本 crate 仅编排(§4.10.1)",
            "Burndown 最小必需,Velocity/CFD 控制图 V1(§9)",
        ],
        "extra_refs": "Sprint/Backlog/Roadmap;Burndown MVP 必需(§9)",
    },
    {
        "name": "domain-relation",
        "chinese_name": "WorkItem 关系领域",
        "spec_section": "§12 Relation",
        "basic_design": "§2.1 / §4.10.2",
        "data_design": "§4.13 (`relation` schema)",
        "api_design": "§3.16 (Relation / Dependency)",
        "entities": ["Relation", "Dependency"],
        "cmd_port": "RelationCommandPort",
        "cmd_methods": [
            ("create_relation", "CreateRelationCommand", "RelationId"),
            ("delete_relation", "RelationId", "()"),
            ("detect_cycle", "WorkItemId", "CycleReport"),
        ],
        "query_port": "RelationQueryPort",
        "query_methods": [
            ("list_relations", "WorkItemId", "Vec<Relation>"),
            ("list_dependencies", "WorkItemId", "Vec<Dependency>"),
        ],
        "events": [
            "star.events.relation.relation.created.v1",
            "star.events.relation.cycle.detected.v1",
        ],
        "deps": ["domain-work-item"],
        "key_invariants": [
            "关系图谱无环(§4.10.2);Dependency 关系不允许循环",
            "Gantt 依赖与冲突分析基础(REQ-COLLAB-002)",
        ],
        "extra_refs": "Relation 仅聚合 WorkItem 关系,Worktree/Agent 关系不属本 crate",
    },
    {
        "name": "domain-comment",
        "chinese_name": "Comment 领域",
        "spec_section": "§13 Comment / @ Mention / Attachment",
        "basic_design": "§2.1 / §4.10.3",
        "data_design": "§4.14 (`comment` schema)",
        "api_design": "§3.17 (Comment / Mention / Attachment)",
        "entities": ["Comment", "Mention", "Attachment"],
        "cmd_port": "CommentCommandPort",
        "cmd_methods": [
            ("create_comment", "CreateCommentCommand", "CommentId"),
            ("update_comment", "UpdateCommentCommand", "Comment"),
            ("delete_comment", "CommentId", "()"),
            ("attach_file", "AttachFileCommand", "AttachmentId"),
        ],
        "query_port": "CommentQueryPort",
        "query_methods": [
            ("list_by_work_item", "WorkItemId", "Vec<Comment>"),
            ("list_mentions", "UserId", "Vec<Mention>"),
        ],
        "events": [
            "star.events.comment.comment.created.v1",
            "star.events.comment.mention.created.v1",
        ],
        "deps": ["domain-work-item"],
        "key_invariants": [
            "Comment 不替代 Feedback(§25.1):Comment 是协作层,Feedback 是结构化结果",
            "@ Mention 触发 notification(§2.3)",
        ],
        "extra_refs": "Comment ≠ Feedback(§25.1)",
    },
    {
        "name": "domain-search",
        "chinese_name": "Search Projection 领域",
        "spec_section": "§14 Search Projection",
        "basic_design": "§2.1 / §3 ACL",
        "data_design": "§4.15 (`search_index` schema)",
        "api_design": "§3.18 (Search Query)",
        "entities": ["SearchIndex", "SearchQuery"],
        "cmd_port": "SearchCommandPort",
        "cmd_methods": [
            ("upsert_document", "UpsertSearchDocumentCommand", "SearchIndexId"),
            ("delete_document", "SearchIndexId", "()"),
            ("bulk_reindex", "BulkReindexCommand", "BulkResult"),
        ],
        "query_port": "SearchQueryPort",
        "query_methods": [
            ("search", "SearchQuery", "SearchResultPage"),
            ("suggest", "SuggestQuery", "Vec<Suggestion>"),
        ],
        "events": [
            "star.events.search.index.updated.v1",
        ],
        "deps": [],
        "key_invariants": [
            "Search 不得成为业务事实源(§12,REQ-SEARCH-001)",
            "读侧仅 API 可见,写侧由各 domain 投影(§3.1)",
        ],
        "extra_refs": "Search 仅 Projection,不得为 SoR(§12,REQ-SEARCH-001)",
    },
    {
        "name": "domain-audit",
        "chinese_name": "Audit 领域",
        "spec_section": "§17 Audit Log / AI Audit Metadata",
        "basic_design": "§2.1 / §9 Traceability / §28.2",
        "data_design": "§4.16 (`audit` schema)",
        "api_design": "§3.19 (Audit Query / 不可删)",
        "entities": ["AuditEvent", "AIAuditMetadata"],
        "cmd_port": "AuditCommandPort",
        "cmd_methods": [
            ("record_event", "RecordAuditEventCommand", "AuditEventId"),
            ("record_ai_metadata", "RecordAIAuditMetadataCommand", "AIAuditMetadataId"),
        ],
        "query_port": "AuditQueryPort",
        "query_methods": [
            ("list_by_actor", "ListAuditQuery", "Vec<AuditEvent>"),
            ("get_ai_metadata", "AIAuditMetadataId", "AIAuditMetadata"),
        ],
        "events": [
            "star.events.audit.event.recorded.v1",
        ],
        "deps": [],
        "key_invariants": [
            "Audit 仅 Append,不可读其它 domain,不可删(§2.3 禁线,§3 ACL)",
            "敏感 Prompt/Code 不默认进入普通日志(§17,§28.2)",
        ],
        "extra_refs": "Audit Append-only,不可读其它 domain(§2.3 禁线)",
    },
    {
        "name": "domain-notification",
        "chinese_name": "Notification 领域",
        "spec_section": "§15 Notification",
        "basic_design": "§2.1 / §4.10.5",
        "data_design": "§4.17 (`notification` schema)",
        "api_design": "§3.20 (Notification Channel / Template)",
        "entities": ["NotificationChannel", "NotificationTemplate", "NotificationDelivery"],
        "cmd_port": "NotificationCommandPort",
        "cmd_methods": [
            ("register_channel", "RegisterChannelCommand", "NotificationChannelId"),
            ("upsert_template", "UpsertTemplateCommand", "NotificationTemplateId"),
            ("send_notification", "SendNotificationCommand", "NotificationDeliveryId"),
        ],
        "query_port": "NotificationQueryPort",
        "query_methods": [
            ("list_channels", "TenantId", "Vec<NotificationChannel>"),
            ("list_deliveries", "UserId", "Vec<NotificationDelivery>"),
        ],
        "events": [
            "star.events.notification.delivery.sent.v1",
        ],
        "deps": [],
        "key_invariants": [
            "MVP 邮件 + 站内(REQ-NOTIF-001)",
            "Notification 与 Audit 互不依赖(Separate Ways,§3.1)",
        ],
        "extra_refs": "Notification 与 Audit Separate Ways(§3.1)",
    },
    {
        "name": "domain-integration",
        "chinese_name": "第三方平台集成领域",
        "spec_section": "§18 Integration / 双向同步",
        "basic_design": "§2.1 / §3.1 接触点表",
        "data_design": "§4.18 (`integration` schema)",
        "api_design": "§3.21 (Integration / SyncState)",
        "entities": ["Integration", "SyncState"],
        "cmd_port": "IntegrationCommandPort",
        "cmd_methods": [
            ("create_integration", "CreateIntegrationCommand", "IntegrationId"),
            ("trigger_sync", "TriggerSyncCommand", "SyncState"),
            ("pause_integration", "PauseIntegrationCommand", "Integration"),
        ],
        "query_port": "IntegrationQueryPort",
        "query_methods": [
            ("get_integration", "IntegrationId", "Integration"),
            ("list_sync_state", "IntegrationId", "Vec<SyncState>"),
        ],
        "events": [
            "star.events.integration.sync.started.v1",
            "star.events.integration.sync.completed.v1",
        ],
        "deps": ["domain-scm", "domain-work-item"],
        "key_invariants": [
            "区分 Link / Mirror / Bidirectional / Platform-owned(§18.1)",
            "Integration 弱耦合于具体 provider,通过 ACL 转译(§3)",
        ],
        "extra_refs": "Link/Mirror/Bidirectional/Platform-owned 4 模式(§18.1)",
    },
    {
        "name": "domain-automation",
        "chinese_name": "Automation 规则领域",
        "spec_section": "§11 Automation",
        "basic_design": "§2.1 / §4.10.5",
        "data_design": "§4.19 (`automation` schema)",
        "api_design": "§3.22 (Rule / Trigger / Action)",
        "entities": ["Rule", "Trigger", "Action"],
        "cmd_port": "AutomationCommandPort",
        "cmd_methods": [
            ("create_rule", "CreateRuleCommand", "RuleId"),
            ("update_rule", "UpdateRuleCommand", "Rule"),
            ("enable_rule", "RuleId", "Rule"),
            ("disable_rule", "RuleId", "Rule"),
        ],
        "query_port": "AutomationQueryPort",
        "query_methods": [
            ("list_rules", "ProjectId", "Vec<Rule>"),
            ("get_rule", "RuleId", "Rule"),
        ],
        "events": [
            "star.events.automation.rule.fired.v1",
        ],
        "deps": ["domain-work-item", "domain-notification"],
        "key_invariants": [
            "MVP 不强制可视化配置器(§11,REQ-AUTO-001)",
            "Rule 触发由 Worker `--role automation` 异步执行(§13.4)",
        ],
        "extra_refs": "Rule 由 Worker role=automation 异步执行(§13.4)",
    },
    # ============== 通用域 Generic Domain ==============
    {
        "name": "domain-tenant",
        "chinese_name": "Tenant 最高安全边界",
        "spec_section": "§4.10.2 / §6.1 (13 类对象 1-2)",
        "basic_design": "§2.1(表 18) / §4.10.2 / §5.7",
        "data_design": "§4.1 (`tenant` schema) / §7 (RLS)",
        "api_design": "§3.2 (domain-tenant 端点) / §5.5 / §8.3.7",
        "entities": ["Tenant", "TenantPolicy", "SecurityPolicy", "ProviderDataBoundary"],
        "cmd_port": "TenantCommandPort",
        "cmd_methods": [
            ("create_tenant", "CreateTenantCommand", "TenantId"),
            ("update_tenant", "UpdateTenantCommand", "Tenant"),
            ("replace_security_policy", "ReplaceSecurityPolicyCommand", "SecurityPolicy"),
            ("upsert_provider_boundary", "UpsertProviderBoundaryCommand", "ProviderDataBoundary"),
            ("transition_status", "TransitionTenantStatusCommand", "Tenant"),
        ],
        "query_port": "TenantQueryPort",
        "query_methods": [
            ("get_current", "()", "Tenant"),
            ("get_by_id", "TenantId", "Tenant"),
            ("get_security_policy", "TenantId", "SecurityPolicy"),
            ("list_provider_boundaries", "TenantId", "Vec<ProviderDataBoundary>"),
            ("get_usage_report", "TenantId", "TenantUsageReport"),
        ],
        "events": [
            "star.events.tenant.tenant.created.v1",
            "star.events.tenant.tenant.security_policy_replaced.v1",
            "star.events.tenant.tenant.provider_boundary_upserted.v1",
            "star.events.tenant.tenant.status_changed.v1",
        ],
        "deps": [],
        "key_invariants": [
            "任何聚合根 INSERT/UPDATE 必须携带 tenant_id(§6.1,REQ-SEC-001)",
            "tenant_id 由本 crate 颁发(UUIDv7),不可调用方传入(§5.7,security-design §4.1)",
            "跨 tenant 访问返回 403 SEC-007 + Audit(security-design §3.5.4)",
            "ProviderDataBoundary.credential_ref 永不明文化(security-design §5.4)",
        ],
        "extra_refs": "13 类 tenant_id 对象(§6.1,§4.10.4,已修复 F-06):本 crate 颁发 tenant_id",
    },
    {
        "name": "domain-workspace",
        "chinese_name": "Workspace 协作单位",
        "spec_section": "§7 Workspace 协作单位",
        "basic_design": "§2.1(表 19)",
        "data_design": "§4.20 (`workspace` schema)",
        "api_design": "§3.3 (Workspace CRUD)",
        "entities": ["Workspace", "WorkspaceMember"],
        "cmd_port": "WorkspaceCommandPort",
        "cmd_methods": [
            ("create_workspace", "CreateWorkspaceCommand", "WorkspaceId"),
            ("add_member", "AddWorkspaceMemberCommand", "WorkspaceMember"),
            ("remove_member", "RemoveWorkspaceMemberCommand", "()"),
        ],
        "query_port": "WorkspaceQueryPort",
        "query_methods": [
            ("get_workspace", "WorkspaceId", "Workspace"),
            ("list_members", "WorkspaceId", "Vec<WorkspaceMember>"),
        ],
        "events": [
            "star.events.workspace.workspace.created.v1",
            "star.events.workspace.member.added.v1",
        ],
        "deps": ["domain-tenant"],
        "key_invariants": [
            "Workspace → 多个 Project(§7)",
            "Workspace 必须归属唯一 Tenant(§6.1)",
        ],
        "extra_refs": "1 Workspace → N Project(§7)",
    },
    {
        "name": "domain-project",
        "chinese_name": "Project 模板与配置",
        "spec_section": "§8 Project 模板",
        "basic_design": "§2.1(表 20)",
        "data_design": "§4.21 (`project` schema)",
        "api_design": "§3.4 (Project CRUD + Policy)",
        "entities": ["Project", "ProjectTemplate", "ProjectPolicy"],
        "cmd_port": "ProjectCommandPort",
        "cmd_methods": [
            ("create_project", "CreateProjectCommand", "ProjectId"),
            ("update_project_policy", "UpdateProjectPolicyCommand", "Project"),
            ("apply_template", "ApplyProjectTemplateCommand", "Project"),
        ],
        "query_port": "ProjectQueryPort",
        "query_methods": [
            ("get_project", "ProjectId", "Project"),
            ("list_by_workspace", "WorkspaceId", "Vec<Project>"),
            ("get_template", "ProjectTemplateId", "ProjectTemplate"),
        ],
        "events": [
            "star.events.project.project.created.v1",
            "star.events.project.policy.updated.v1",
        ],
        "deps": ["domain-tenant", "domain-workspace"],
        "key_invariants": [
            "可独立配置 Workflow/Permission/Notification/Agent Policy(REQ-TWP-003)",
            "ProjectPolicy 是 WorkflowDefinition 的运行时注入点(§4.9.3)",
        ],
        "extra_refs": "ProjectPolicy 注入 Workflow/Agent/Permission 配置(REQ-TWP-003)",
    },
    {
        "name": "domain-permission",
        "chinese_name": "Permission Scheme / RBAC",
        "spec_section": "§11 Permission",
        "basic_design": "§2.1(表 21)",
        "data_design": "§4.22 (`permission` schema)",
        "api_design": "§3.5 (Permission CRUD + Role)",
        "entities": ["Role", "Permission", "PermissionScheme"],
        "cmd_port": "PermissionCommandPort",
        "cmd_methods": [
            ("create_role", "CreateRoleCommand", "RoleId"),
            ("assign_permission", "AssignPermissionCommand", "Role"),
            ("create_scheme", "CreatePermissionSchemeCommand", "PermissionSchemeId"),
        ],
        "query_port": "PermissionQueryPort",
        "query_methods": [
            ("check", "AuthorizationCheckQuery", "AuthorizationDecision"),
            ("list_roles", "TenantId", "Vec<Role>"),
        ],
        "events": [
            "star.events.permission.role.created.v1",
        ],
        "deps": ["domain-tenant"],
        "key_invariants": [
            "Agent 操作必须 Application/Authorization 强制(§11,REQ-PERM-002)",
            "PermissionScheme 按 Project 注入(§3 ACL)",
        ],
        "extra_refs": "PermissionScheme = RBAC + Project Policy(§11)",
    },
    {
        "name": "domain-identity",
        "chinese_name": "用户 / 设备身份",
        "spec_section": "§23.2 Local Runtime 三重绑定",
        "basic_design": "§2.1(表 22) / §4.6.3 / §23.2",
        "data_design": "§4.23 (`identity` schema)",
        "api_design": "§3.6 (User / Device / Credential)",
        "entities": ["User", "Device", "Credential", "DeviceBinding"],
        "cmd_port": "IdentityCommandPort",
        "cmd_methods": [
            ("create_user", "CreateUserCommand", "UserId"),
            ("bind_device", "BindDeviceCommand", "DeviceBindingId"),
            ("revoke_device", "DeviceId", "()"),
            ("rotate_credential", "RotateCredentialCommand", "CredentialId"),
        ],
        "query_port": "IdentityQueryPort",
        "query_methods": [
            ("get_user", "UserId", "User"),
            ("list_devices", "UserId", "Vec<Device>"),
            ("verify_credential", "VerifyCredentialQuery", "CredentialVerification"),
        ],
        "events": [
            "star.events.identity.user.created.v1",
            "star.events.identity.device.bound.v1",
        ],
        "deps": ["domain-tenant"],
        "key_invariants": [
            "Device 需 Tenant+User+Project 三重绑定(§23.2)",
            "Credential 永不明文化,仅 CredentialRef(security-design §5.4)",
        ],
        "extra_refs": "Device 三重绑定 Tenant+User+Project(§23.2)",
    },
    {
        "name": "domain-collaboration",
        "chinese_name": "协作(Realtime Presence)",
        "spec_section": "§15 Realtime / Presence",
        "basic_design": "§2.1(表 24) / §1.1 部署图",
        "data_design": "§4.24 (`collaboration` schema)",
        "api_design": "§3.23 (Presence / RealtimeSubscription)",
        "entities": ["Presence", "RealtimeSubscription"],
        "cmd_port": "CollaborationCommandPort",
        "cmd_methods": [
            ("update_presence", "UpdatePresenceCommand", "Presence"),
            ("subscribe_realtime", "SubscribeRealtimeCommand", "RealtimeSubscriptionId"),
        ],
        "query_port": "CollaborationQueryPort",
        "query_methods": [
            ("list_presence", "ProjectId", "Vec<Presence>"),
            ("list_subscriptions", "UserId", "Vec<RealtimeSubscription>"),
        ],
        "events": [
            "star.events.collaboration.presence.updated.v1",
        ],
        "deps": ["domain-work-item", "domain-worktree"],
        "key_invariants": [
            "高频 Token Stream 可不入 SaaS(§15,REQ-RT-003)",
            "第一阶段不部署 realtime role(§13.1,§15)",
        ],
        "extra_refs": "Realtime 第一阶段不部署(§13.1);高频 Token Stream 可旁路(§15)",
    },
    {
        "name": "domain-local-runtime",
        "chinese_name": "集群外 Runtime 服务器侧 Registry / Port",
        "spec_section": "§23 Local Runtime",
        "basic_design": "§4.6 / §23 / §1.1 LocalRuntime 子图",
        "data_design": "§4.25 (`local_runtime` schema)",
        "api_design": "§3.24 (Runtime Registry / Command / Observation)",
        "entities": ["Runtime", "RuntimeCommand", "RuntimeObservation"],
        "cmd_port": "LocalRuntimeCommandPort",
        "cmd_methods": [
            ("register_runtime", "RegisterRuntimeCommand", "RuntimeId"),
            ("send_command", "SendRuntimeCommand", "RuntimeCommandId"),
            ("revoke_runtime", "RuntimeId", "()"),
        ],
        "query_port": "LocalRuntimeQueryPort",
        "query_methods": [
            ("get_runtime", "RuntimeId", "Runtime"),
            ("list_observations", "RuntimeId", "Vec<RuntimeObservation>"),
            ("list_pending_commands", "RuntimeId", "Vec<RuntimeCommand>"),
        ],
        "events": [
            "star.events.local_runtime.runtime.registered.v1",
            "star.events.local_runtime.command.sent.v1",
            "star.events.local_runtime.observation.received.v1",
        ],
        "deps": ["domain-worktree", "domain-identity"],
        "key_invariants": [
            "Local Daemon 二进制不属本 crate(§4.6.1,F-07 修复后);本 crate 仅服务器侧 Registry/Port",
            "Local Runtime 不计入 K8s Workload(§23.1,§8.5)",
            "9 种白名单 RuntimeCommand(§4.6.2):PullRepo/CreateWorktree/RunAgent/ReportObservation 等",
        ],
        "extra_refs": "Local Daemon 是集群外独立进程,本 crate 仅服务器侧 Port(§4.6.1,F-07);9 种白名单命令(§4.6.2)",
    },
    # ============== 3 supporting crates ==============
    {
        "name": "application",
        "chinese_name": "Application Services 编排层",
        "spec_section": "§2.4 / §14.1 跨域事务",
        "basic_design": "§1.2 / §2.4",
        "data_design": "—",
        "api_design": "—",
        "entities": [],
        "cmd_port": "ApplicationService",
        "cmd_methods": [
            ("create_work_item_full", "CreateWorkItemFullCommand", "WorkItem"),
            ("register_worktree_full", "RegisterWorktreeFullCommand", "Worktree"),
            ("start_agent_session_full", "StartAgentSessionFullCommand", "AgentSession"),
            ("submit_feedback_full", "SubmitFeedbackFullCommand", "Feedback"),
            ("register_runtime_full", "RegisterRuntimeFullCommand", "Runtime"),
        ],
        "query_port": "ApplicationQueryService",
        "query_methods": [
            ("get_work_item_view", "WorkItemId", "WorkItemView"),
        ],
        "events": [],
        "deps": ["domain-work-item", "domain-worktree", "domain-agent", "domain-feedback", "domain-tenant", "domain-audit", "domain-permission", "domain-scm", "domain-development", "domain-validation", "domain-local-runtime", "domain-identity"],
        "key_invariants": [
            "跨域事务由本 crate 编排,单 PG 事务(§2.4,§14.1)",
            "Outbox 触发事件(非事务组成,异步):AgentSessionCreated / WorktreeStatusObserved / ValidationFailed(§2.4)",
            "本 crate 不持有 Entity,只编排 Domain Port 调用(§2.4)",
        ],
        "extra_refs": "Application Service 跨域事务编排(§2.4);不通过 Event Chain 拆分(§14.1,§58)",
    },
    {
        "name": "infrastructure",
        "chinese_name": "Adapter 实现层 (PostgreSQL / NATS / ObjectStorage / SCM / Agent)",
        "spec_section": "§3 ACL / §13.1 数据面",
        "basic_design": "§1.1 / §3 ACL",
        "data_design": "§6 / §7 (RLS) / §8 (索引)",
        "api_design": "—",
        "entities": [],
        "cmd_port": "AdapterRegistry",
        "cmd_methods": [
            ("register_postgres_adapter", "()", "()"),
            ("register_nats_adapter", "()", "()"),
            ("register_object_storage_adapter", "()", "()"),
            ("register_scm_adapter", "()", "()"),
            ("register_agent_adapter", "()", "()"),
        ],
        "query_port": "AdapterQuery",
        "query_methods": [
            ("list_registered_adapters", "()", "Vec<AdapterDescriptor>"),
        ],
        "events": [],
        "deps": [],
        "key_invariants": [
            "本 crate 不允许反向依赖 `domain`,只实现 Domain 定义的 Port(§3 ACL)",
            "PostgreSQL = 默认 SoR(§13.1,§30.6)",
            "Database 保持单一 PostgreSQL(非 Database per Domain,§13.5)",
        ],
        "extra_refs": "Adapter 仅实现 Domain Port(§3 ACL);非 Database per Domain(§13.5,§30.6)",
    },
    {
        "name": "api",
        "chinese_name": "API Gateway 入口 (REST / WS)",
        "spec_section": "§3 API 端点 / §13.1 gateway role",
        "basic_design": "§1.1 / §13.1",
        "data_design": "—",
        "api_design": "§3 全部 / §5 Event Subject / §8 错误码",
        "entities": [],
        "cmd_port": "ApiGateway",
        "cmd_methods": [
            ("register_route", "()", "()"),
            ("register_ws_handler", "()", "()"),
            ("register_middleware", "()", "()"),
        ],
        "query_port": "ApiQuery",
        "query_methods": [
            ("list_routes", "()", "Vec<RouteDescriptor>"),
        ],
        "events": [],
        "deps": [],
        "key_invariants": [
            "Gateway 角色与 work-core / identity / worker 同级最小闭环(§13.1)",
            "Realtime 仅在 Long Connection Scaling Boundary 出现后拆出(§13.1,§15)",
        ],
        "extra_refs": "Gateway 属最小闭环 4 角色之一(§13.1);Realtime 暂不部署(§15)",
    },
]


# ----------------------------------------------------------------------
# Render functions
# ----------------------------------------------------------------------

def render_cargo_toml(spec: dict) -> str:
    """Each crate's Cargo.toml.

    Phase 1 (skeleton) does NOT depend on other domain-* crates even if
    the dependency graph (basic-design §2.3) suggests it. The doc comments
    in lib.rs reference the upstream domains, but Cargo.toml is minimal.
    """
    name = spec["name"]
    is_supporting = name in {"application", "infrastructure", "api"}
    # supporting crates get tokio (for future async use), domain-* don't yet
    extra_deps = ""
    if name == "application":
        extra_deps = 'tokio = { workspace = true }\n'
    elif name == "infrastructure":
        extra_deps = (
            'tokio = { workspace = true }\n'
            'sqlx = { workspace = true }\n'
        )
    elif name == "api":
        extra_deps = (
            'tokio = { workspace = true }\n'
        )
    return dedent(f"""\
        [package]
        name = "{name}"
        version.workspace = true
        edition.workspace = true
        rust-version.workspace = true
        license.workspace = true
        authors.workspace = true
        repository.workspace = true

        [dependencies]
        # 基础依赖(全部 crate 共享)
        serde = {{ workspace = true }}
        async-trait = {{ workspace = true }}
        thiserror = {{ workspace = true }}
        uuid = {{ workspace = true }}
        chrono = {{ workspace = true }}
        {extra_deps}
        [lints]
        workspace = true
        """).rstrip() + "\n"


def render_lib_rs(spec: dict) -> str:
    """Each crate's lib.rs.

    Contains:
      - module docstring (reference to spec/basic/data/api design sections)
      - entity struct(s) — placeholder with key fields
      - CommandPort trait — method signatures only (no body)
      - QueryPort trait — method signatures only
      - Domain Event struct(s) — placeholder
      - Error enum — thiserror
      - ActorContext / shared types — placeholders
      - one stub unit test
    """
    name = spec["name"]
    chinese = spec["chinese_name"]
    is_supporting = name in {"application", "infrastructure", "api"}

    # PascalCase for trait/error/entity, snake_case for command
    module = snake_to_pascal(name.replace("domain-", "").replace("-", "_"))
    # domain-tenant -> module=DomainTenant; work-item -> module=WorkItem
    cmd_port = spec["cmd_port"]
    query_port = spec["query_port"]
    entities = spec["entities"]
    cmd_methods = spec["cmd_methods"]
    query_methods = spec["query_methods"]
    events = spec["events"]
    deps = spec["deps"]
    invariants = spec["key_invariants"]
    extra = spec.get("extra_refs", "")

    # Build the entity section
    entity_block = "\n".join(
        f"""
/// {ent} (聚合根 / 实体)
///
/// 来源: docs/data-design.md {spec["data_design"]}
///
/// **骨架阶段**: 仅占位字段,完整字段与不变量留待 Phase 2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {ent} {{
    /// 主键 UUID
    pub id: Uuid,
    /// 租户隔离(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}}"""
        for ent in entities
    ) or """\
// (本 crate 为 supporting 层,无业务实体 — 实体由 domain-* crate 拥有)"""

    # Build CommandPort
    cmd_signatures = "\n".join(
        f"    async fn {m}(\n"
        f"        &self,\n"
        f"        cmd: {arg},\n"
        f"        actor: ActorContext,\n"
        f"    ) -> Result<{ret}, {module}Error>;"
        for (m, arg, ret) in cmd_methods
    )
    cmd_port_block = f"""
/// **{cmd_port}**(命令端口)
///
/// 来源: docs/api-design.md {spec["api_design"]}
///
/// **骨架阶段**: 仅方法签名,无 body 实现。Phase 2 在
/// `crates/infrastructure/<adapter>.rs` 中提供 SQLx / NATS / SCM Adapter 实现。
#[async_trait]
pub trait {cmd_port}: Send + Sync {{
{cmd_signatures}
}}"""

    # Build QueryPort
    query_signatures = "\n".join(
        f"    async fn {m}(\n"
        f"        &self,\n"
        f"        {param_decl(arg)},\n"
        f"        viewer: ActorContext,\n"
        f"    ) -> Result<{ret}, {module}Error>;"
        for (m, arg, ret) in query_methods
    )
    query_port_block = f"""
/// **{query_port}**(查询端口)
///
/// 来源: docs/api-design.md {spec["api_design"]}
#[async_trait]
pub trait {query_port}: Send + Sync {{
{query_signatures}
}}"""

    # Build Domain Events
    if events:
        event_block = "\n".join(
            f"""
/// Domain Event: `{subj}`
///
/// 来源: docs/api-design.md §5 (CloudEvents 1.0)
///
/// **骨架阶段**: 仅占位字段,Phase 2 补充完整 Payload 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {module}{i:02d}Event {{
    /// 事件唯一 ID(UUIDv7)
    pub event_id: Uuid,
    /// 租户 ID(必带)
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}}"""
            for i, subj in enumerate(events, start=1)
        )
    else:
        event_block = """\
// (本 crate 不直接发布 Domain Event,事件由 domain-* crate 拥有)"""

    # Build Error enum (5 standard variants, can be extended in Phase 2)
    # Build type aliases and placeholder structs so the Port trait
    # signatures compile without cross-crate imports.
    type_aliases = collect_type_aliases(cmd_methods, query_methods, entities)
    placeholder_structs = collect_placeholder_structs(cmd_methods, query_methods, entities)
    types_block = render_types_block(type_aliases, placeholder_structs)

    error_block = f"""
/// **{module} 错误**
///
/// 来源: docs/api-design.md §8 (错误码)
/// 5 个标准变体;具体错误码在 Phase 2 由本 enum 派生 + 实现 `Into<ApiError>`。
#[derive(Debug, thiserror::Error)]
pub enum {module}Error {{
    #[error("not found: {{0}}")]
    NotFound(Uuid),
    #[error("invalid state: {{0}}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {{0}}")]
    Conflict(String),
    #[error("internal: {{0}}")]
    Internal(String),
}}"""

    # Build deps doc block
    if deps:
        deps_list = "\n".join(f"//!   - `{d}`" for d in deps)
        deps_block = f"""
//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 依赖以下 domain-*(骨架阶段不实际 import,Cargo.toml 仅声明本 crate 自身需要的外部依赖):
//!
{deps_list}
//!
//! **禁止反向依赖**(§2.3 禁线)。"""
    elif is_supporting:
        deps_block = """
//! ## 上游依赖
//!
//! 本 supporting crate 编排多个 domain-*(骨架阶段不实际 import,仅占位模块结构)。"""
    else:
        deps_block = """
//! ## 上游依赖
//!
//! 本 crate 为依赖图最底层(basic-design §2.3),无 domain-* 上游依赖。"""

    # Build invariants block
    inv_block = "\n".join(f"//! - {inv}" for inv in invariants)

    # Extra refs (cross-cutting concerns)
    extra_block = f"\n//! ## 关键引用\n//!\n//! {extra}\n" if extra else ""

    # Build the full lib.rs
    lib_rs = f"""//! {chinese}
//!
//! **crate**: `{name}`
//! **上游 spec**: docs/specs/{name.replace("domain-", "domain-")}-spec.md {spec["spec_section"]}
//! **基本设计**: docs/basic-design.md {spec["basic_design"]}
//! **数据设计**: docs/data-design.md {spec["data_design"]}
//! **API 设计**: docs/api-design.md {spec["api_design"]}
//!
//! ## 职责
//!
//! 详细职责边界见 spec 文档第 1 节。骨架阶段仅声明 Port trait + Entity + Error,
//! 具体实现由 `crates/infrastructure` 中的 Adapter 提供。
//!
//! ## 关键不变量
//!
//! {inv_block}
{deps_block}
{extra_block}
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

// =====================================================================
// 实体(Entity / Aggregate Root)
// =====================================================================
{entity_block}

// =====================================================================
// 端口(Port / 抽象)
// =====================================================================
{cmd_port_block}

{query_port_block}

// =====================================================================
// Domain Events(CloudEvents 1.0,见 api-design §5)
// =====================================================================
{event_block}

// =====================================================================
// 类型别名与命令/查询/返回类型占位
// =====================================================================
{types_block}

// =====================================================================
// Error
// =====================================================================
{error_block}

// =====================================================================
// 共享类型
// =====================================================================

/// **Actor 上下文**(来自 `domain-identity` / `domain-permission` 的 JWT claim)
///
/// **骨架阶段**: 字段占位;Phase 2 由 `domain-identity` 颁发的 ActorContext 取代
/// 本 crate 内的占位定义(避免循环依赖)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {{
    /// 当前用户 ID
    pub user_id: Uuid,
    /// 当前租户 ID(13 类对象必带,§6.1)
    pub tenant_id: Uuid,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<Uuid>,
    /// 当前 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<Uuid>,
    /// 当前用户角色(`tenant_admin` / `project_admin` / `developer` / `viewer`)
    pub roles: Vec<String>,
}}

// =====================================================================
// 单元测试占位
// =====================================================================

#[cfg(test)]
mod tests {{
    use super::*;

    /// **骨架阶段**: 最小冒烟测试,验证 crate 可编译、ActorContext 字段可达。
    /// Phase 2 由具体 spec 引入完整单元测试(状态机覆盖 / RLS 矩阵等)。
    #[test]
    fn actor_context_skeleton() {{
        let actor = ActorContext {{
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            device_id: None,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
        }};
        assert!(!actor.tenant_id.is_nil(), "tenant_id must be non-nil (§6.1,REQ-SEC-001)");
    }}
}}
"""
    return lib_rs


def snake_to_pascal(s: str) -> str:
    return "".join(part.capitalize() for part in s.split("_"))


# ----------------------------------------------------------------------
# Type-alias / placeholder-struct generation
# ----------------------------------------------------------------------

# Fixed list of return-type names that should be defined as placeholder
# structs (e.g. `Transition`, `CycleReport`, `BulkResult`).
PLACEHOLDER_RETURN_TYPES = {
    "Transition", "BulkResult", "CycleReport", "AcceptanceCoverage",
    "ValidationResultId", "DeviceBindingId", "CredentialId",
    "TenantUsageReport", "AuthorizationDecision", "CredentialVerification",
    "RuntimeObservation", "RuntimeCommand", "AdapterDescriptor",
    "RouteDescriptor", "WorkItemView", "SearchResultPage", "Suggestion",
    "Presence", "RealtimeSubscription", "NotificationDelivery",
    "ContextBudget", "SearchIndexId", "AuditEventId", "AIAuditMetadataId",
    "Sprint", "Backlog", "Roadmap", "Burndown",
    "Comment", "Mention", "Attachment",
    "Relation", "Dependency",
    "SearchIndex",
    "Role", "Permission", "PermissionScheme",
    "NotificationChannel", "NotificationTemplate", "NotificationChannelId",
    "NotificationTemplateId", "NotificationDeliveryId",
    "Workspace", "WorkspaceMember",
    "WorkspaceId",
    "Board", "Column", "Swimlane", "BoardId",
    "SprintId", "RoadmapId",
    "CommentId", "AttachmentId", "RelationId", "RuleId",
    "Integration", "SyncState", "IntegrationId",
    "Rule", "Trigger", "Action",
    "Repository", "Branch", "Commit", "PullRequest", "Review", "Pipeline",
    "RepositoryId",
    "DevelopmentExecution", "ChangeSet", "Link", "SymbolIndex",
    "RepositoryContext", "DevelopmentContext",
    "ExecutionId", "LinkId",
    "Agent", "AgentSession", "AgentPolicy",
    "AgentId", "AgentSessionId",
    "Feedback", "FeedbackResolution", "FeedbackId",
    "ContextPacket", "Decision", "ContextPacketId", "DecisionId",
    "ValidationResult",
    "ProviderDataBoundary", "TenantPolicy", "SecurityPolicy",
    "Runtime", "RuntimeId", "RuntimeCommandId", "RuntimeObservationId",
    "SymbolIndex",
    "User", "Device", "Credential", "DeviceBinding",
    "UserId", "DeviceId",
    "Project", "ProjectTemplate", "ProjectPolicy",
    "ProjectId", "ProjectTemplateId",
    "RoleId", "PermissionSchemeId",
    "ContextPacketId",
    "Feedback",
    "Integration",
    "ListWorkItemQuery",
    "ListBusinessGoalQuery",
    "ListChangeSetQuery",
    "ListFeedbackByTargetQuery",
    "ListAuditQuery",
    "ListAgentSessionQuery",
    "ListDecisionQuery",
    "ListPullRequestQuery",
    "EstimateBudgetQuery",
    "SearchQuery",
    "SuggestQuery",
    "AuthorizationCheckQuery",
    "VerifyCredentialQuery",
    "ListPresenceQuery",
}


def collect_type_aliases(cmd_methods, query_methods, entities):
    """Collect all `*Id` type aliases to generate.

    Sources:
      - each entity name => `<Entity>Id`
      - every type name appearing in any signature position
        (argument, return, Vec<...>) that ends with `Id`
    """
    aliases = set()
    # Entity ids (e.g. WorkItem -> WorkItemId, Agent -> AgentId, AgentSession -> AgentSessionId)
    for ent in entities:
        if not ent.endswith("Id"):
            aliases.add(f"{ent}Id")
    # All positions: arg, return
    for method in cmd_methods + query_methods:
        for pos in (method[1], method[2]):
            _collect_id_aliases(pos, aliases)
    return sorted(aliases)


def _collect_id_aliases(type_str: str, aliases: set) -> None:
    """Recursively extract `*Id` type names from a type string."""
    if type_str in ("()",):
        return
    if type_str.startswith("Vec<") and type_str.endswith(">"):
        inner = type_str[4:-1]
        _collect_id_aliases(inner, aliases)
        return
    if type_str.startswith("Option<") and type_str.endswith(">"):
        inner = type_str[7:-1]
        _collect_id_aliases(inner, aliases)
        return
    if type_str.endswith("Id"):
        aliases.add(type_str)


# Primitive/scalar types that should NOT be turned into placeholder structs.
_SCALAR_TYPES = {
    "Uuid", "String", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64",
    "f32", "f64", "bool", "usize", "isize", "()", "Vec<u8>",
    "chrono::DateTime<chrono::Utc>",
}


def collect_placeholder_structs(cmd_methods, query_methods, entities):
    """Collect all type names that need a placeholder struct.

    Rules:
      - Skip scalar primitives.
      - Skip anything ending in `Id` (handled by type aliases).
      - Skip entity names (defined separately as full entities).
      - Everything else: emit a placeholder struct.
    """
    names = set()
    entity_names = set(entities)
    for method in cmd_methods + query_methods:
        for pos in (method[1], method[2]):
            _collect_placeholder_names(pos, names, entity_names)
    # Remove anything that's a known entity (avoid double-definition)
    return sorted(n for n in names if n not in entity_names)


def _collect_placeholder_names(type_str: str, names: set, entity_names: set) -> None:
    if type_str in _SCALAR_TYPES:
        return
    if type_str in ("()",):
        return
    if type_str.startswith("Vec<") and type_str.endswith(">"):
        _collect_placeholder_names(type_str[4:-1], names, entity_names)
        return
    if type_str.startswith("Option<") and type_str.endswith(">"):
        _collect_placeholder_names(type_str[7:-1], names, entity_names)
        return
    if type_str.endswith("Id"):
        return  # handled by type aliases
    if type_str in entity_names:
        return  # already defined
    names.add(type_str)


def render_types_block(type_aliases, placeholder_structs):
    """Render the `Types` section: aliases + placeholder structs."""
    lines = []
    # Type aliases
    if type_aliases:
        lines.append("/// **ID 类型别名**(Phase 1 骨架:均为 UUID 别名)")
        lines.append("///")
        lines.append("/// 真实使用应由 `domain-identity` 颁发强类型 ID(§23.2);")
        lines.append("/// 骨架阶段以 `Uuid` 替代以避免跨 crate 编译依赖。")
        lines.append("")
        for alias in type_aliases:
            lines.append(f"pub type {alias} = Uuid;")
        lines.append("")
    # Placeholder structs
    if placeholder_structs:
        lines.append("/// **命令 / 查询 / 跨 crate 类型占位结构**(Phase 1 骨架:最小字段集)")
        lines.append("")
        lines.append("/// Phase 2 由具体 spec 在 `domain-*` 内补全字段;`crates/application` 等")
        lines.append("/// supporting crate 的占位则在 Phase 2 删除,改为 `use domain_xxx::*;` 引用。")
        lines.append("")
        for name in sorted(placeholder_structs):
            lines.append("#[derive(Debug, Clone, Serialize, Deserialize)]")
            lines.append(f"pub struct {name} {{")
            lines.append("    /// 主键 UUID")
            lines.append("    pub id: Uuid,")
            lines.append("    /// 租户 ID(13 类对象必带,§6.1)")
            lines.append("    pub tenant_id: Uuid,")
            lines.append("    // 其它字段在 Phase 2 由具体 spec 补充")
            lines.append("}")
            lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def param_decl(arg: str) -> str:
    """Map a query method's argument placeholder to a (name, type) pair.

    Conventions used by the spec:
      - `WorkItemId`, `TenantId`, ... — single id, name `_id`
      - `ListFooQuery` — query struct, name `q`
      - `()` — no argument
      - `Vec<X>`  — already a return type, not a query arg
    """
    if arg == "()":
        return "_dummy: ()"
    if arg.endswith("Id"):
        name = "id"
        return f"{name}: {arg}"
    if arg.startswith("List") and arg.endswith("Query"):
        return f"q: {arg}"
    if arg.startswith("Verify") or arg.startswith("Estimate") or arg.startswith("Check"):
        return f"q: {arg}"
    if arg.startswith("List"):
        # already a query, no List<...> returns
        return f"q: {arg}"
    # default: assume it's a query struct
    return f"q: {arg}"


# ----------------------------------------------------------------------
# Workspace root
# ----------------------------------------------------------------------

def render_root_cargo_toml(crates: list[str]) -> str:
    # Stable member order (alphabetical) — paths must include `crates/` prefix.
    sorted_crates = sorted(f"crates/{c}" for c in crates)
    members = ",\n    ".join(f'"{c}"' for c in sorted_crates)
    # NOTE: do not use textwrap.dedent on this multi-line string — the first
    # `[workspace]` line has 4 spaces and the interpolated `members` are
    # double-indented, so dedent() produces garbled output. Hand-format
    # the file directly instead.
    return (
        "[workspace]\n"
        "resolver = \"2\"\n"
        "members = [\n"
        f"    {members},\n"
        "]\n"
        "exclude = [\"target\"]\n"
        "\n"
        "[workspace.lints.rust]\n"
        "missing_docs = \"warn\"\n"
        "rust_2018_idioms = \"warn\"\n"
        "unreachable_pub = \"warn\"\n"
        "\n"
        "[workspace.package]\n"
        "version = \"0.1.0\"\n"
        "edition = \"2021\"\n"
        "rust-version = \"1.75\"\n"
        "authors = [\"Star Team\"]\n"
        "license = \"Apache-2.0\"\n"
        "repository = \"https://github.com/UlyssesLeoLee/Star\"\n"
        "\n"
        "[workspace.dependencies]\n"
        "# 异步运行时与异步 trait\n"
        "tokio = { version = \"1.40\", features = [\"full\"] }\n"
        "async-trait = \"0.1\"\n"
        "\n"
        "# 序列化\n"
        "serde = { version = \"1\", features = [\"derive\"] }\n"
        "serde_json = \"1\"\n"
        "\n"
        "# 错误\n"
        "thiserror = \"1\"\n"
        "anyhow = \"1\"\n"
        "\n"
        "# ID / 时间\n"
        "uuid = { version = \"1\", features = [\"v4\", \"serde\"] }\n"
        "chrono = { version = \"0.4\", default-features = false, features = [\"serde\", \"clock\"] }\n"
        "\n"
        "# 数据库(Phase 2 使用,骨架阶段仅 workspace 声明,infrastructure crate 引用)\n"
        "sqlx = { version = \"0.8\", default-features = false, features = [\"runtime-tokio-rustls\", \"postgres\", \"uuid\", \"chrono\", \"json\", \"macros\"] }\n"
        "\n"
        "# Tracing\n"
        "tracing = \"0.1\"\n"
        "\n"
        "[profile.release]\n"
        "lto = \"thin\"\n"
        "codegen-units = 1\n"
        "\n"
        "[profile.dev]\n"
        "opt-level = 0\n"
        "debug = 1\n"
    )


# ----------------------------------------------------------------------
# Main
# ----------------------------------------------------------------------

def main() -> None:
    crate_names = [s["name"] for s in CRATE_SPECS]
    if len(crate_names) != 28:
        raise SystemExit(f"Expected 28 crates, got {len(crate_names)}: {crate_names}")

    # 1. Create directories
    for c in crate_names:
        (CRATES / c / "src").mkdir(parents=True, exist_ok=True)

    # 2. Write Cargo.toml + lib.rs for each crate
    for spec in CRATE_SPECS:
        cargo_path = CRATES / spec["name"] / "Cargo.toml"
        lib_path = CRATES / spec["name"] / "src" / "lib.rs"
        cargo_path.write_text(render_cargo_toml(spec), encoding="utf-8", newline="\n")
        lib_path.write_text(render_lib_rs(spec), encoding="utf-8", newline="\n")
        print(f"  wrote {cargo_path.relative_to(ROOT)} ({cargo_path.stat().st_size} bytes)")
        print(f"  wrote {lib_path.relative_to(ROOT)}  ({lib_path.stat().st_size} bytes)")

    # 3. Write workspace root Cargo.toml (members must be `crates/<name>`)
    root_cargo = ROOT / "Cargo.toml"
    root_cargo.write_text(render_root_cargo_toml(crate_names), encoding="utf-8", newline="\n")
    print(f"\n  wrote {root_cargo.relative_to(ROOT)} ({root_cargo.stat().st_size} bytes)")

    # 4. Sanity check: 25 domain-* + 3 supporting = 28
    domain_crates = [c for c in crate_names if c.startswith("domain-")]
    supporting = [c for c in crate_names if not c.startswith("domain-")]
    assert len(domain_crates) == 25, f"Expected 25 domain-* crates, got {len(domain_crates)}"
    assert len(supporting) == 3, f"Expected 3 supporting crates, got {len(supporting)}"
    assert sorted(domain_crates) == sorted([
        "domain-tenant", "domain-workspace", "domain-project", "domain-work-item",
        "domain-workflow", "domain-board", "domain-planning", "domain-permission",
        "domain-comment", "domain-relation", "domain-development", "domain-scm",
        "domain-worktree", "domain-agent", "domain-feedback", "domain-context",
        "domain-validation", "domain-audit", "domain-search", "domain-notification",
        "domain-integration", "domain-automation", "domain-identity",
        "domain-collaboration", "domain-local-runtime",
    ]), f"Domain crate list mismatch: {domain_crates}"
    print(f"\n  [OK] 25 domain-* crates + 3 supporting = 28 total")
    print(f"  [OK] Cargo workspace written to {root_cargo}")


if __name__ == "__main__":
    main()
