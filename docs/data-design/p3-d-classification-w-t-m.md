# P3-D 段階 W/T/M 三類横展開 分類報告

> **基準**: ユーザー指定 DB 三類横展開原則（2026-09-01 18:30 JST）
> **適用範囲**: Star 仓 P3-D 段階 (22 domain-* crate) 全 entity
> **一次出典**: `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 + `00-CLASSIFICATION-RULES.md` v0.1
> **派生守門**: CW-01~CW-10 (per 守门 #DB-13 W/T/M)
> **生成**: `scripts/automation/wtm_classifier.py` v0.1 (per 守门 #19 [M] 拍板, 2026-09-04 拍板)

---

## §1 全局統計 (P3-D 段階 22 domain-* crate 全体)

| 業務分類 | 件数 | 比率 |
|---|---|---|
| **Master (M)** | 119 | 12.6% |
| **Transaction (T)** | 818 | 86.7% |
| **Work (W)** | 6 | 0.6% |
| **Skip** (Type alias, ID 等) | 0 | 0.0% |
| **合計** | 943 | 100.0% |

## §2 domain-* crate 別 分類件数 (P3-D 段階 22 crate)

| domain-* crate | M | T | W | Skip | 合計 | 評価 |
|---|---|---|---|---|---|---|
| `domain-agent` | 4 | 18 | 0 | 0 | 22 | 🟡 CW-02 違反 |
| `domain-agent-windows` | 3 | 16 | 0 | 0 | 19 | 🟡 CW-02 違反 |
| `domain-ai` | 3 | 6 | 0 | 0 | 9 | 🟡 CW-02 違反 |
| `domain-audit` | 2 | 17 | 0 | 0 | 19 | 🟡 CW-02 違反 |
| `domain-automation` | 2 | 28 | 1 | 0 | 31 | ✅ |
| `domain-batch` | 2 | 41 | 0 | 0 | 43 | 🟡 CW-02 違反 |
| `domain-board` | 2 | 26 | 0 | 0 | 28 | 🟡 CW-02 違反 |
| `domain-cli` | 7 | 47 | 0 | 0 | 54 | 🟡 CW-02 違反 |
| `domain-collaboration` | 0 | 21 | 2 | 0 | 23 | 🟡 CW-02 違反 |
| `domain-comment` | 0 | 37 | 0 | 0 | 37 | 🟡 CW-02 違反 |
| `domain-context` | 0 | 21 | 0 | 0 | 21 | 🟡 CW-02 違反 |
| `domain-dashboard` | 0 | 8 | 0 | 0 | 8 | 🟡 CW-02 違反 |
| `domain-development` | 0 | 26 | 0 | 0 | 26 | 🟡 CW-02 違反 |
| `domain-feedback` | 0 | 30 | 0 | 0 | 30 | 🟡 CW-02 違反 |
| `domain-form` | 0 | 14 | 0 | 0 | 14 | 🟡 CW-02 違反 |
| `domain-identity` | 15 | 28 | 0 | 0 | 43 | 🟡 CW-02 違反 |
| `domain-integration` | 2 | 39 | 0 | 0 | 41 | 🟡 CW-02 違反 |
| `domain-kms` | 0 | 6 | 0 | 0 | 6 | 🟡 CW-02 違反 |
| `domain-local-runtime` | 5 | 41 | 2 | 0 | 48 | ✅ |
| `domain-notification` | 3 | 12 | 0 | 0 | 15 | 🟡 CW-02 違反 |
| `domain-permission` | 10 | 8 | 0 | 0 | 18 | 🟡 CW-02 違反 |
| `domain-planning` | 0 | 24 | 0 | 0 | 24 | 🟡 CW-02 違反 |
| `domain-project` | 7 | 22 | 0 | 0 | 29 | 🟡 CW-02 違反 |
| `domain-relation` | 3 | 14 | 0 | 0 | 17 | 🟡 CW-02 違反 |
| `domain-report` | 5 | 65 | 1 | 0 | 71 | ✅ |
| `domain-scm` | 0 | 28 | 0 | 0 | 28 | 🟡 CW-02 違反 |
| `domain-search` | 0 | 33 | 0 | 0 | 33 | 🟡 CW-02 違反 |
| `domain-tenant` | 30 | 5 | 0 | 0 | 35 | 🟡 CW-02 違反 |
| `domain-theme` | 8 | 4 | 0 | 0 | 12 | 🟡 CW-02 違反 |
| `domain-validation` | 2 | 31 | 0 | 0 | 33 | 🟡 CW-02 違反 |
| `domain-work-item` | 0 | 55 | 0 | 0 | 55 | 🟡 CW-02 違反 |
| `domain-workflow` | 3 | 16 | 0 | 0 | 19 | 🟡 CW-02 違反 |
| `domain-workspace` | 1 | 14 | 0 | 0 | 15 | 🟡 CW-02 違反 |
| `domain-worktree` | 0 | 17 | 0 | 0 | 17 | 🟡 CW-02 違反 |

## §3 派生守門 10 条 チェック (CW-01 ~ CW-10)

| # | 派生守門 | crate | message |
|---|---|---|---|
| CW-02 | CW-02 | `domain-agent` | 三類分門別類漏れ: M=4, T=18, W=0 |
| CW-03 | CW-03 | `domain-agent` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-agent-windows` | 三類分門別類漏れ: M=3, T=16, W=0 |
| CW-03 | CW-03 | `domain-agent-windows` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-ai` | 三類分門別類漏れ: M=3, T=6, W=0 |
| CW-03 | CW-03 | `domain-ai` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-audit` | 三類分門別類漏れ: M=2, T=17, W=0 |
| CW-03 | CW-03 | `domain-audit` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-batch` | 三類分門別類漏れ: M=2, T=41, W=0 |
| CW-03 | CW-03 | `domain-batch` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-board` | 三類分門別類漏れ: M=2, T=26, W=0 |
| CW-03 | CW-03 | `domain-board` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-cli` | 三類分門別類漏れ: M=7, T=47, W=0 |
| CW-03 | CW-03 | `domain-cli` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-collaboration` | 三類分門別類漏れ: M=0, T=21, W=2 |
| CW-02 | CW-02 | `domain-comment` | 三類分門別類漏れ: M=0, T=37, W=0 |
| CW-03 | CW-03 | `domain-comment` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-context` | 三類分門別類漏れ: M=0, T=21, W=0 |
| CW-03 | CW-03 | `domain-context` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-dashboard` | 三類分門別類漏れ: M=0, T=8, W=0 |
| CW-03 | CW-03 | `domain-dashboard` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-development` | 三類分門別類漏れ: M=0, T=26, W=0 |
| CW-03 | CW-03 | `domain-development` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-feedback` | 三類分門別類漏れ: M=0, T=30, W=0 |
| CW-03 | CW-03 | `domain-feedback` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-form` | 三類分門別類漏れ: M=0, T=14, W=0 |
| CW-03 | CW-03 | `domain-form` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-identity` | 三類分門別類漏れ: M=15, T=28, W=0 |
| CW-03 | CW-03 | `domain-identity` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-integration` | 三類分門別類漏れ: M=2, T=39, W=0 |
| CW-03 | CW-03 | `domain-integration` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-kms` | 三類分門別類漏れ: M=0, T=6, W=0 |
| CW-03 | CW-03 | `domain-kms` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-notification` | 三類分門別類漏れ: M=3, T=12, W=0 |
| CW-03 | CW-03 | `domain-notification` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-permission` | 三類分門別類漏れ: M=10, T=8, W=0 |
| CW-03 | CW-03 | `domain-permission` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-planning` | 三類分門別類漏れ: M=0, T=24, W=0 |
| CW-03 | CW-03 | `domain-planning` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-project` | 三類分門別類漏れ: M=7, T=22, W=0 |
| CW-03 | CW-03 | `domain-project` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-relation` | 三類分門別類漏れ: M=3, T=14, W=0 |
| CW-03 | CW-03 | `domain-relation` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-scm` | 三類分門別類漏れ: M=0, T=28, W=0 |
| CW-03 | CW-03 | `domain-scm` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-search` | 三類分門別類漏れ: M=0, T=33, W=0 |
| CW-03 | CW-03 | `domain-search` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-tenant` | 三類分門別類漏れ: M=30, T=5, W=0 |
| CW-03 | CW-03 | `domain-tenant` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-theme` | 三類分門別類漏れ: M=8, T=4, W=0 |
| CW-03 | CW-03 | `domain-theme` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-validation` | 三類分門別類漏れ: M=2, T=31, W=0 |
| CW-03 | CW-03 | `domain-validation` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-work-item` | 三類分門別類漏れ: M=0, T=55, W=0 |
| CW-03 | CW-03 | `domain-work-item` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-workflow` | 三類分門別類漏れ: M=3, T=16, W=0 |
| CW-03 | CW-03 | `domain-workflow` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-workspace` | 三類分門別類漏れ: M=1, T=14, W=0 |
| CW-03 | CW-03 | `domain-workspace` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |
| CW-02 | CW-02 | `domain-worktree` | 三類分門別類漏れ: M=0, T=17, W=0 |
| CW-03 | CW-03 | `domain-worktree` | W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要) |

## §4 業務分類 詳細 (per crate, 全 entity)

### domain-agent

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `AgentPolicy` | `src\lib.rs` |
| **M** | `AgentPolicyTemplate` | `src\lib.rs` |
| **M** | `CreatePolicyTemplateCommand` | `src\lib.rs` |
| **M** | `PolicyCheck` | `src\lib.rs` |
| **T** | `AbortSessionCommand` | `src\lib.rs` |
| **T** | `Agent` | `src\lib.rs` |
| **T** | `AgentError` | `src\lib.rs` |
| **T** | `AgentSession` | `src\lib.rs` |
| **T** | `AgentSessionStatus` | `src\lib.rs` |
| **T** | `AgentSessionSummary` | `src\lib.rs` |
| **T** | `AgentType` | `src\lib.rs` |
| **T** | `GetSessionQuery` | `src\lib.rs` |
| **T** | `InMemoryAgentRepository` | `src\lib.rs` |
| **T** | `InMemoryAgentService` | `src\lib.rs` |
| **T** | `ListByWorktreeQuery` | `src\lib.rs` |
| **T** | `NetworkAccess` | `src\lib.rs` |
| **T** | `RecordToolActivityCommand` | `src\lib.rs` |
| **T** | `RegisterAgentCommand` | `src\lib.rs` |
| **T** | `SecretAccess` | `src\lib.rs` |
| **T** | `StartSessionCommand` | `src\lib.rs` |
| **T** | `SubmitFeedbackCommand` | `src\lib.rs` |
| **T** | `TransitionStatusCommand` | `src\lib.rs` |

### domain-agent-windows

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CommitTemplate` | `src\commit_template.rs` |
| **M** | `CommitTemplateBuilder` | `src\commit_template.rs` |
| **M** | `UploadConfig` | `src\upload_executor.rs` |
| **T** | `CommitScope` | `src\commit_template.rs` |
| **T** | `CommitType` | `src\commit_template.rs` |
| **T** | `RunResult` | `src\lib.rs` |
| **T** | `TabState` | `src\lib.rs` |
| **T** | `TaskTab` | `src\lib.rs` |
| **T** | `TaskWindow` | `src\lib.rs` |
| **T** | `TriggerMode` | `src\lib.rs` |
| **T** | `UploadError` | `src\upload_executor.rs` |
| **T** | `UploadExecutor` | `src\upload_executor.rs` |
| **T** | `UploadResult` | `src\upload_executor.rs` |
| **T** | `UploadStatus` | `src\lib.rs` |
| **T** | `UploadTask` | `src\lib.rs` |
| **T** | `WindowError` | `src\lib.rs` |
| **T** | `WindowService` | `src\lib.rs` |
| **T** | `WorktreeError` | `src\commit_template.rs` |
| **T** | `WorktreeStatus` | `src\commit_template.rs` |

### domain-ai

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `AgentRole` | `src\lib.rs` |
| **M** | `ModelConfig` | `src\lib.rs` |
| **M** | `PromptTemplate` | `src\lib.rs` |
| **T** | `AiError` | `src\lib.rs` |
| **T** | `AiRequest` | `src\lib.rs` |
| **T** | `AiResponse` | `src\lib.rs` |
| **T** | `AiService` | `src\lib.rs` |
| **T** | `LlmProvider` | `src\lib.rs` |
| **T** | `MockLlmProvider` | `src\lib.rs` |

### domain-audit

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CrossTenantAttempted` | `src\lib.rs` |
| **M** | `RecordCrossTenantAttemptCommand` | `src\lib.rs` |
| **T** | `AIAuditMetadata` | `src\lib.rs` |
| **T** | `AIAuditMetadataInput` | `src\lib.rs` |
| **T** | `Actor` | `src\lib.rs` |
| **T** | `AuditAction` | `src\lib.rs` |
| **T** | `AuditError` | `src\lib.rs` |
| **T** | `AuditEvent` | `src\lib.rs` |
| **T** | `AuditEventKind` | `src\lib.rs` |
| **T** | `AuditExportJob` | `src\lib.rs` |
| **T** | `AuditListQuery` | `src\lib.rs` |
| **T** | `AuditRecorded` | `src\lib.rs` |
| **T** | `EventMeta` | `src\lib.rs` |
| **T** | `ExportAuditCommand` | `src\lib.rs` |
| **T** | `ExportFormat` | `src\lib.rs` |
| **T** | `ExportStatus` | `src\lib.rs` |
| **T** | `InMemoryAuditService` | `src\lib.rs` |
| **T** | `RecordAIAuditCommand` | `src\lib.rs` |
| **T** | `RecordAuditCommand` | `src\lib.rs` |

### domain-automation

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `RbacConfig` | `src\governance.rs` |
| **M** | `ThrottleConfig` | `src\governance.rs` |
| **T** | `ActionType` | `src\lib.rs` |
| **T** | `ApprovalFlow` | `src\governance.rs` |
| **T** | `AuditEntry` | `src\governance.rs` |
| **T** | `AuditEvent` | `src\governance.rs` |
| **T** | `AutomationAction` | `src\lib.rs` |
| **T** | `AutomationCondition` | `src\lib.rs` |
| **T** | `AutomationError` | `src\lib.rs` |
| **T** | `AutomationEvent` | `src\lib.rs` |
| **T** | `AutomationExecution` | `src\lib.rs` |
| **T** | `AutomationRule` | `src\lib.rs` |
| **T** | `AutomationTrigger` | `src\lib.rs` |
| **T** | `ConditionOperator` | `src\lib.rs` |
| **T** | `CreateRuleCommand` | `src\lib.rs` |
| **T** | `DeadLetterEntry` | `src\governance.rs` |
| **T** | `DlqStatus` | `src\governance.rs` |
| **T** | `EventMeta` | `src\lib.rs` |
| **T** | `ExecutionResult` | `src\lib.rs` |
| **T** | `GovernanceError` | `src\governance.rs` |
| **T** | `GovernanceService` | `src\governance.rs` |
| **T** | `InMemoryAutomationService` | `src\lib.rs` |
| **T** | `ListRulesQuery` | `src\lib.rs` |
| **T** | `MaintenanceWindow` | `src\governance.rs` |
| **T** | `PauseState` | `src\governance.rs` |
| **T** | `RuleExecuted` | `src\lib.rs` |
| **T** | `TestRuleCommand` | `src\lib.rs` |
| **T** | `ThrottleCounter` | `src\governance.rs` |
| **T** | `TriggerType` | `src\lib.rs` |
| **T** | `UpdateRuleCommand` | `src\lib.rs` |
| **W** | `BlockedActions` | `src\governance.rs` |

### domain-batch

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CatchupPolicy` | `src\domain.rs` |
| **M** | `RetryPolicy` | `src\domain.rs` |
| **T** | `AlertChannel` | `src\domain.rs` |
| **T** | `AlertRule` | `src\domain.rs` |
| **T** | `AlertRuleKind` | `src\domain.rs` |
| **T** | `BatchDomain` | `src\domain.rs` |
| **T** | `BatchError` | `src\error.rs` |
| **T** | `BatchErrorCode` | `src\error.rs` |
| **T** | `BatchEvent` | `src\event.rs` |
| **T** | `BatchEventKind` | `src\event.rs` |
| **T** | `CreateTaskCommand` | `src\port.rs` |
| **T** | `Dag` | `src\domain.rs` |
| **T** | `DagNode` | `src\domain.rs` |
| **T** | `Event` | `src\domain.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `ListEventQuery` | `src\port.rs` |
| **T** | `ListNodeTypeQuery` | `src\port.rs` |
| **T** | `ListRunQuery` | `src\port.rs` |
| **T** | `ListTaskQuery` | `src\port.rs` |
| **T** | `Log` | `src\domain.rs` |
| **T** | `LogChunk` | `src\domain.rs` |
| **T** | `LogOffset` | `src\domain.rs` |
| **T** | `LogStream` | `src\domain.rs` |
| **T** | `Node` | `src\domain.rs` |
| **T** | `NodeExecutionResult` | `src\domain.rs` |
| **T** | `NodeStatus` | `src\domain.rs` |
| **T** | `NodeType` | `src\domain.rs` |
| **T** | `NoopBatchService` | `src\port.rs` |
| **T** | `NotifyOn` | `src\domain.rs` |
| **T** | `RegisterNodeTypeCommand` | `src\domain.rs` |
| **T** | `RetryStrategy` | `src\domain.rs` |
| **T** | `Run` | `src\domain.rs` |
| **T** | `RunStatus` | `src\domain.rs` |
| **T** | `RuntimeKind` | `src\domain.rs` |
| **T** | `Sla` | `src\domain.rs` |
| **T** | `SlaAction` | `src\domain.rs` |
| **T** | `Task` | `src\domain.rs` |
| **T** | `TaskStatus` | `src\domain.rs` |
| **T** | `TriggerTaskCommand` | `src\port.rs` |
| **T** | `TriggerType` | `src\domain.rs` |
| **T** | `UpdateTaskCommand` | `src\port.rs` |
| **T** | `UpsertAlertRuleCommand` | `src\port.rs` |
| **T** | `UpsertSlaCommand` | `src\port.rs` |

### domain-board

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `SwimlaneGroupBy` | `src\lib.rs` |
| **M** | `SwimlaneGroupBy` | `src\wip_swimlane.rs` |
| **T** | `AddColumnCommand` | `src\lib.rs` |
| **T** | `AddSwimlaneCommand` | `src\lib.rs` |
| **T** | `Board` | `src\lib.rs` |
| **T** | `BoardCard` | `src\lib.rs` |
| **T** | `BoardColumn` | `src\lib.rs` |
| **T** | `BoardError` | `src\lib.rs` |
| **T** | `BoardKind` | `src\lib.rs` |
| **T** | `BoardService` | `src\wip_swimlane.rs` |
| **T** | `BoardView` | `src\lib.rs` |
| **T** | `CreateBoardCommand` | `src\lib.rs` |
| **T** | `GetViewQuery` | `src\lib.rs` |
| **T** | `InMemoryBoardRepository` | `src\lib.rs` |
| **T** | `InMemoryBoardService` | `src\lib.rs` |
| **T** | `ListByProjectQuery` | `src\lib.rs` |
| **T** | `MoveCardCommand` | `src\lib.rs` |
| **T** | `NewColumnSpec` | `src\lib.rs` |
| **T** | `SavedView` | `src\wip_swimlane.rs` |
| **T** | `SetWipLimitCommand` | `src\lib.rs` |
| **T** | `Swimlane` | `src\lib.rs` |
| **T** | `Swimlane` | `src\wip_swimlane.rs` |
| **T** | `ViewDensity` | `src\wip_swimlane.rs` |
| **T** | `ViewFilters` | `src\wip_swimlane.rs` |
| **T** | `ViewLayout` | `src\wip_swimlane.rs` |
| **T** | `WipAction` | `src\wip_swimlane.rs` |
| **T** | `WipGuard` | `src\wip_swimlane.rs` |
| **T** | `WipLimit` | `src\wip_swimlane.rs` |

### domain-cli

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `BackoffConfig` | `src\quota.rs` |
| **M** | `FallbackPolicy` | `src\fallback.rs` |
| **M** | `HermesConfig` | `src\hermes_client.rs` |
| **M** | `HermesConfig` | `src\hermes\value_object.rs` |
| **M** | `OpenClawConfig` | `src\openclaw_client.rs` |
| **M** | `QuotaGuard` | `src\quota.rs` |
| **M** | `RetryPolicy` | `src\hermes\value_object.rs` |
| **T** | `ApiCallEvent` | `src\api_monitor.rs` |
| **T** | `ApiCallStatus` | `src\api_monitor.rs` |
| **T** | `ApiError` | `src\quota.rs` |
| **T** | `ApiKey` | `src\lib.rs` |
| **T** | `ApiKeyMode` | `src\lib.rs` |
| **T** | `ApiKeySummary` | `src\lib.rs` |
| **T** | `ApiMonitor` | `src\api_monitor.rs` |
| **T** | `AuthToken` | `src\hermes\entity.rs` |
| **T** | `CancelResponse` | `src\hermes\entity.rs` |
| **T** | `ChatMessage` | `src\hermes_client.rs` |
| **T** | `ChatMessage` | `src\openclaw_client.rs` |
| **T** | `Choice` | `src\hermes_client.rs` |
| **T** | `Choice` | `src\openclaw_client.rs` |
| **T** | `CliError` | `src\lib.rs` |
| **T** | `CliKind` | `src\lib.rs` |
| **T** | `CliProfile` | `src\lib.rs` |
| **T** | `CliService` | `src\lib.rs` |
| **T** | `FallbackChain` | `src\fallback.rs` |
| **T** | `FallbackDecision` | `src\fallback.rs` |
| **T** | `FallbackReason` | `src\fallback.rs` |
| **T** | `FallbackResult` | `src\fallback.rs` |
| **T** | `GenerateRequest` | `src\hermes_client.rs` |
| **T** | `GenerateRequest` | `src\openclaw_client.rs` |
| **T** | `GenerateResponse` | `src\hermes_client.rs` |
| **T** | `GenerateResponse` | `src\openclaw_client.rs` |
| **T** | `HermesClient` | `src\hermes_client.rs` |
| **T** | `HermesClient` | `src\hermes\service.rs` |
| **T** | `HermesClientBuilder` | `src\hermes\service.rs` |
| **T** | `HermesError` | `src\hermes_client.rs` |
| **T** | `HermesError` | `src\hermes\error.rs` |
| **T** | `HermesMode` | `src\hermes\value_object.rs` |
| **T** | `InMemorySink` | `src\api_monitor.rs` |
| **T** | `InvocationResult` | `src\lib.rs` |
| **T** | `OpenClawClient` | `src\openclaw_client.rs` |
| **T** | `OpenClawError` | `src\openclaw_client.rs` |
| **T** | `ProviderStats` | `src\api_monitor.rs` |
| **T** | `QueryRequest` | `src\hermes\entity.rs` |
| **T** | `RateLimiter` | `src\quota.rs` |
| **T** | `SubmitRequest` | `src\hermes\service.rs` |
| **T** | `TabState` | `src\lib.rs` |
| **T** | `Task` | `src\hermes\entity.rs` |
| **T** | `TaskStatus` | `src\hermes\entity.rs` |
| **T** | `TaskTab` | `src\lib.rs` |
| **T** | `TaskWindow` | `src\lib.rs` |
| **T** | `Usage` | `src\hermes_client.rs` |
| **T** | `Usage` | `src\openclaw_client.rs` |
| **T** | `WorktreeBinding` | `src\lib.rs` |

### domain-collaboration

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AddShapeCommand` | `src\lib.rs` |
| **T** | `CollabError` | `src\lib.rs` |
| **T** | `CollabParentType` | `src\lib.rs` |
| **T** | `CollabSessionStatus` | `src\lib.rs` |
| **T** | `CollaborationSession` | `src\lib.rs` |
| **T** | `Cursor` | `src\lib.rs` |
| **T** | `CursorPosition` | `src\lib.rs` |
| **T** | `DeleteShapeCommand` | `src\lib.rs` |
| **T** | `EndSessionCommand` | `src\lib.rs` |
| **T** | `GetSessionQuery` | `src\lib.rs` |
| **T** | `GetWhiteboardQuery` | `src\lib.rs` |
| **T** | `InMemoryCollabRepository` | `src\lib.rs` |
| **T** | `InMemoryCollabService` | `src\lib.rs` |
| **T** | `SelectionRange` | `src\lib.rs` |
| **T** | `ShapeKind` | `src\lib.rs` |
| **T** | `StartSessionCommand` | `src\lib.rs` |
| **T** | `UpdateCursorCommand` | `src\lib.rs` |
| **T** | `UpdatePresenceCommand` | `src\lib.rs` |
| **T** | `UpdateShapeCommand` | `src\lib.rs` |
| **T** | `Whiteboard` | `src\lib.rs` |
| **T** | `WhiteboardShape` | `src\lib.rs` |
| **W** | `ListActivePresencesQuery` | `src\lib.rs` |
| **W** | `Presence` | `src\lib.rs` |

### domain-comment

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AddReactionCommand` | `src\lib.rs` |
| **T** | `AddReactionCommand` | `src\port.rs` |
| **T** | `Attachment` | `src\entity.rs` |
| **T** | `Attachment` | `src\lib.rs` |
| **T** | `AttachmentDownloadURL` | `src\entity.rs` |
| **T** | `AttachmentUploaded` | `src\event.rs` |
| **T** | `Comment` | `src\entity.rs` |
| **T** | `Comment` | `src\lib.rs` |
| **T** | `CommentCreated` | `src\event.rs` |
| **T** | `CommentDeleted` | `src\event.rs` |
| **T** | `CommentError` | `src\error.rs` |
| **T** | `CommentError` | `src\lib.rs` |
| **T** | `CommentEvent` | `src\event.rs` |
| **T** | `CommentStatus` | `src\lib.rs` |
| **T** | `CommentStatus` | `src\value_object.rs` |
| **T** | `CommentUpdated` | `src\event.rs` |
| **T** | `CreateCommentCommand` | `src\lib.rs` |
| **T** | `CreateCommentCommand` | `src\port.rs` |
| **T** | `DeleteCommentCommand` | `src\lib.rs` |
| **T** | `EditCommentCommand` | `src\lib.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `GetCommentQuery` | `src\lib.rs` |
| **T** | `InMemoryCommentRepository` | `src\lib.rs` |
| **T** | `InMemoryCommentService` | `src\lib.rs` |
| **T** | `InMemoryCommentService` | `src\service.rs` |
| **T** | `ListByParentQuery` | `src\lib.rs` |
| **T** | `ListCommentQuery` | `src\port.rs` |
| **T** | `Mention` | `src\entity.rs` |
| **T** | `Mention` | `src\lib.rs` |
| **T** | `MentionNotified` | `src\event.rs` |
| **T** | `ParentType` | `src\lib.rs` |
| **T** | `ParentType` | `src\value_object.rs` |
| **T** | `Reaction` | `src\entity.rs` |
| **T** | `Reaction` | `src\lib.rs` |
| **T** | `RegisterAttachmentCommand` | `src\lib.rs` |
| **T** | `UpdateCommentCommand` | `src\port.rs` |
| **T** | `UploadAttachmentCommand` | `src\port.rs` |

### domain-context

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `ContextError` | `src\lib.rs` |
| **T** | `ContextPacket` | `src\lib.rs` |
| **T** | `ContextPacketCreator` | `src\lib.rs` |
| **T** | `CreateContextPacketCommand` | `src\lib.rs` |
| **T** | `CreateDecisionCommand` | `src\lib.rs` |
| **T** | `Decision` | `src\lib.rs` |
| **T** | `DecisionScope` | `src\lib.rs` |
| **T** | `DecisionSource` | `src\lib.rs` |
| **T** | `DecisionStatus` | `src\lib.rs` |
| **T** | `GetContextPacketQuery` | `src\lib.rs` |
| **T** | `InMemoryContextRepository` | `src\lib.rs` |
| **T** | `InMemoryContextService` | `src\lib.rs` |
| **T** | `InvalidateDecisionCommand` | `src\lib.rs` |
| **T** | `ListDecisionsQuery` | `src\lib.rs` |
| **T** | `Priority` | `src\lib.rs` |
| **T** | `ProvenanceEntry` | `src\lib.rs` |
| **T** | `ProvenanceItem` | `src\lib.rs` |
| **T** | `ProvenanceSourceType` | `src\lib.rs` |
| **T** | `RelevantBucket` | `src\lib.rs` |
| **T** | `Scope` | `src\lib.rs` |
| **T** | `SupersedeDecisionCommand` | `src\lib.rs` |

### domain-dashboard

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `Dashboard` | `src\lib.rs` |
| **T** | `DashboardError` | `src\lib.rs` |
| **T** | `DashboardScope` | `src\lib.rs` |
| **T** | `DashboardService` | `src\lib.rs` |
| **T** | `Gadget` | `src\lib.rs` |
| **T** | `GadgetPosition` | `src\lib.rs` |
| **T** | `GadgetSize` | `src\lib.rs` |
| **T** | `GadgetType` | `src\lib.rs` |

### domain-development

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AddFileChangeCommand` | `src\lib.rs` |
| **T** | `ApproveChangeSetCommand` | `src\lib.rs` |
| **T** | `ChangeSet` | `src\lib.rs` |
| **T** | `ChangeSetStatus` | `src\lib.rs` |
| **T** | `ChangeStats` | `src\lib.rs` |
| **T** | `CreateChangeSetCommand` | `src\lib.rs` |
| **T** | `DevelopmentError` | `src\lib.rs` |
| **T** | `DevelopmentExecution` | `src\lib.rs` |
| **T** | `ExecutionActor` | `src\lib.rs` |
| **T** | `ExecutionResult` | `src\lib.rs` |
| **T** | `FileChange` | `src\lib.rs` |
| **T** | `FileChangeType` | `src\lib.rs` |
| **T** | `GetChangeSetQuery` | `src\lib.rs` |
| **T** | `GetSymbolQuery` | `src\lib.rs` |
| **T** | `InMemoryDevelopmentRepository` | `src\lib.rs` |
| **T** | `InMemoryDevelopmentService` | `src\lib.rs` |
| **T** | `ListByStatusQuery` | `src\lib.rs` |
| **T** | `ListByWorktreeQuery` | `src\lib.rs` |
| **T** | `MergeChangeSetCommand` | `src\lib.rs` |
| **T** | `RecordExecutionCommand` | `src\lib.rs` |
| **T** | `RejectChangeSetCommand` | `src\lib.rs` |
| **T** | `RequestChangesCommand` | `src\lib.rs` |
| **T** | `SubmitChangeSetCommand` | `src\lib.rs` |
| **T** | `SymbolIndex` | `src\lib.rs` |
| **T** | `SymbolKind` | `src\lib.rs` |
| **T** | `UpsertSymbolCommand` | `src\lib.rs` |

### domain-feedback

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `ActorContext` | `src\context.rs` |
| **T** | `ConsumedByKind` | `src\entity.rs` |
| **T** | `CreateFeedbackCommand` | `src\port.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `EvidenceKind` | `src\entity.rs` |
| **T** | `Feedback` | `src\entity.rs` |
| **T** | `FeedbackAcknowledged` | `src\event.rs` |
| **T** | `FeedbackApplied` | `src\event.rs` |
| **T** | `FeedbackConsumedEvent` | `src\entity.rs` |
| **T** | `FeedbackCreated` | `src\event.rs` |
| **T** | `FeedbackError` | `src\error.rs` |
| **T** | `FeedbackEvent` | `src\event.rs` |
| **T** | `FeedbackInboxItem` | `src\entity.rs` |
| **T** | `FeedbackInboxQuery` | `src\port.rs` |
| **T** | `FeedbackRejected` | `src\event.rs` |
| **T** | `FeedbackResolution` | `src\entity.rs` |
| **T** | `FeedbackStatus` | `src\value_object.rs` |
| **T** | `FeedbackSuperseded` | `src\event.rs` |
| **T** | `FeedbackTarget` | `src\value_object.rs` |
| **T** | `FeedbackType` | `src\value_object.rs` |
| **T** | `FeedbackVerified` | `src\event.rs` |
| **T** | `InMemoryFeedbackService` | `src\service.rs` |
| **T** | `LineRange` | `src\value_object.rs` |
| **T** | `ListFeedbackQuery` | `src\port.rs` |
| **T** | `ResolutionEvidence` | `src\entity.rs` |
| **T** | `ResolutionEvidenceRef` | `src\entity.rs` |
| **T** | `Severity` | `src\value_object.rs` |
| **T** | `SubmitResolutionCommand` | `src\port.rs` |
| **T** | `TransitionFeedbackStatusCommand` | `src\port.rs` |
| **T** | `UpdateFeedbackCommand` | `src\port.rs` |

### domain-form

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AccessControl` | `src\lib.rs` |
| **T** | `CondOperator` | `src\lib.rs` |
| **T** | `ConditionalAction` | `src\lib.rs` |
| **T** | `ConditionalRule` | `src\lib.rs` |
| **T** | `FieldOption` | `src\lib.rs` |
| **T** | `FieldType` | `src\lib.rs` |
| **T** | `FieldValidation` | `src\lib.rs` |
| **T** | `Form` | `src\lib.rs` |
| **T** | `FormError` | `src\lib.rs` |
| **T** | `FormField` | `src\lib.rs` |
| **T** | `FormService` | `src\lib.rs` |
| **T** | `FormSubmission` | `src\lib.rs` |
| **T** | `SubmitAction` | `src\lib.rs` |
| **T** | `SubmitActionType` | `src\lib.rs` |

### domain-identity

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CreateRoleCommand` | `src\port.rs` |
| **M** | `CreateUserCommand` | `src\lib.rs` |
| **M** | `CreateUserCommand` | `src\port.rs` |
| **M** | `GetUserQuery` | `src\lib.rs` |
| **M** | `ListDevicesByUserQuery` | `src\lib.rs` |
| **M** | `ListUserQuery` | `src\port.rs` |
| **M** | `Role` | `src\entity.rs` |
| **M** | `TenantRole` | `src\lib.rs` |
| **M** | `UpdateUserCommand` | `src\port.rs` |
| **M** | `User` | `src\entity.rs` |
| **M** | `User` | `src\lib.rs` |
| **M** | `UserCreated` | `src\event.rs` |
| **M** | `UserLoggedIn` | `src\event.rs` |
| **M** | `UserStatus` | `src\lib.rs` |
| **M** | `UserStatus` | `src\value_object.rs` |
| **T** | `ActorContext` | `src\context.rs` |
| **T** | `BindDeviceCommand` | `src\lib.rs` |
| **T** | `BindDeviceCommand` | `src\port.rs` |
| **T** | `BindingKind` | `src\lib.rs` |
| **T** | `Credential` | `src\entity.rs` |
| **T** | `Credential` | `src\lib.rs` |
| **T** | `CredentialKind` | `src\lib.rs` |
| **T** | `CredentialSpec` | `src\port.rs` |
| **T** | `CredentialType` | `src\value_object.rs` |
| **T** | `Device` | `src\entity.rs` |
| **T** | `Device` | `src\lib.rs` |
| **T** | `DeviceBinding` | `src\entity.rs` |
| **T** | `DeviceBinding` | `src\lib.rs` |
| **T** | `DeviceBound` | `src\event.rs` |
| **T** | `DeviceKind` | `src\lib.rs` |
| **T** | `DeviceStatus` | `src\lib.rs` |
| **T** | `DeviceType` | `src\value_object.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `IdentityError` | `src\error.rs` |
| **T** | `IdentityError` | `src\lib.rs` |
| **T** | `IdentityEvent` | `src\event.rs` |
| **T** | `InMemoryIdentityRepository` | `src\lib.rs` |
| **T** | `InMemoryIdentityService` | `src\lib.rs` |
| **T** | `InMemoryIdentityService` | `src\service.rs` |
| **T** | `RecordLoginCommand` | `src\lib.rs` |
| **T** | `RecordLoginCommand` | `src\port.rs` |
| **T** | `RegisterDeviceCommand` | `src\lib.rs` |
| **T** | `RevokeDeviceCommand` | `src\lib.rs` |

### domain-integration

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `ConfigureIntegrationCommand` | `src\port.rs` |
| **M** | `MappingConfig` | `src\entity.rs` |
| **T** | `ActorContext` | `src\context.rs` |
| **T** | `AdapterCapability` | `src\adapter.rs` |
| **T** | `AdapterError` | `src\adapter.rs` |
| **T** | `AuthToken` | `src\adapter.rs` |
| **T** | `ConflictStrategy` | `src\value_object.rs` |
| **T** | `ConfluenceAdapter` | `src\confluence.rs` |
| **T** | `ConfluenceLink` | `src\confluence.rs` |
| **T** | `ConfluencePage` | `src\confluence.rs` |
| **T** | `ConfluenceSpace` | `src\confluence.rs` |
| **T** | `CreateIntegrationCommand` | `src\port.rs` |
| **T** | `CredentialRefId` | `src\adapter.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `ExternalEntityId` | `src\value_object.rs` |
| **T** | `ExternalSystemName` | `src\value_object.rs` |
| **T** | `GetHistoryQuery` | `src\port.rs` |
| **T** | `HandleWebhookCommand` | `src\port.rs` |
| **T** | `InMemoryIntegrationService` | `src\service.rs` |
| **T** | `Integration` | `src\entity.rs` |
| **T** | `IntegrationCreated` | `src\event.rs` |
| **T** | `IntegrationError` | `src\error.rs` |
| **T** | `IntegrationEvent` | `src\event.rs` |
| **T** | `IntegrationRelationType` | `src\value_object.rs` |
| **T** | `IntegrationSource` | `src\value_object.rs` |
| **T** | `IntegrationState` | `src\value_object.rs` |
| **T** | `IntegrationStateChanged` | `src\event.rs` |
| **T** | `ListByProjectQuery` | `src\port.rs` |
| **T** | `OAuth2AuthRequest` | `src\adapter.rs` |
| **T** | `OAuth2Callback` | `src\adapter.rs` |
| **T** | `PauseIntegrationCommand` | `src\port.rs` |
| **T** | `ResumeIntegrationCommand` | `src\port.rs` |
| **T** | `StarWorkItemMacro` | `src\confluence.rs` |
| **T** | `SyncCompleted` | `src\event.rs` |
| **T** | `SyncConflictDetected` | `src\event.rs` |
| **T** | `SyncDirection` | `src\entity.rs` |
| **T** | `SyncOutcome` | `src\value_object.rs` |
| **T** | `SyncState` | `src\entity.rs` |
| **T** | `SyncTriggered` | `src\event.rs` |
| **T** | `TriggerSyncCommand` | `src\port.rs` |
| **T** | `UpdateIntegrationCommand` | `src\port.rs` |

### domain-kms

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `EncryptedBlob` | `src\lib.rs` |
| **T** | `KeyId` | `src\lib.rs` |
| **T** | `KmsError` | `src\lib.rs` |
| **T** | `KmsHealth` | `src\lib.rs` |
| **T** | `KmsMode` | `src\lib.rs` |
| **T** | `LocalMockKms` | `src\lib.rs` |

### domain-local-runtime

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CliSpawnConfig` | `src\cli_spawn.rs` |
| **M** | `HubAdapterConfig` | `src\spawn_upload_hub.rs` |
| **M** | `HubSpawnConfig` | `src\subscribe_integration.rs` |
| **M** | `IntegrationConfig` | `src\spawn_upload_integration.rs` |
| **M** | `ListByUserQuery` | `src\lib.rs` |
| **T** | `AgentExecutionContext` | `src\lib.rs` |
| **T** | `CliSpawnError` | `src\cli_spawn.rs` |
| **T** | `CreateExecContextCommand` | `src\lib.rs` |
| **T** | `DefaultLocalRuntime` | `src\process.rs` |
| **T** | `EchoCmd` | `src\e2e_integration.rs` |
| **T** | `GetRuntimeQuery` | `src\lib.rs` |
| **T** | `HeartbeatCommand` | `src\lib.rs` |
| **T** | `HttpClient` | `src\http_client.rs` |
| **T** | `HttpError` | `src\http_client.rs` |
| **T** | `HttpMethod` | `src\http_client.rs` |
| **T** | `HttpRequest` | `src\http_client.rs` |
| **T** | `HttpResponse` | `src\http_client.rs` |
| **T** | `HubAdapterError` | `src\spawn_upload_hub.rs` |
| **T** | `HubCliRuntime` | `src\subscribe_integration.rs` |
| **T** | `HubIntegrationError` | `src\subscribe_integration.rs` |
| **T** | `HubIntegratorAdapter` | `src\spawn_upload_hub.rs` |
| **T** | `InMemoryRuntimeRepository` | `src\lib.rs` |
| **T** | `InMemoryRuntimeService` | `src\lib.rs` |
| **T** | `IntegrationError` | `src\spawn_upload_integration.rs` |
| **T** | `IntegrationResult` | `src\spawn_upload_integration.rs` |
| **T** | `LocalRuntime` | `src\lib.rs` |
| **T** | `MountStatus` | `src\lib.rs` |
| **T** | `MountWorktreeCommand` | `src\lib.rs` |
| **T** | `OutputHub` | `src\subscribe_real.rs` |
| **T** | `OutputLine` | `src\process.rs` |
| **T** | `OutputStream` | `src\process.rs` |
| **T** | `ProcessHandle` | `src\process.rs` |
| **T** | `ProcessState` | `src\process.rs` |
| **T** | `RealCliRuntime` | `src\cli_spawn.rs` |
| **T** | `RealHttpRuntime` | `src\http_client.rs` |
| **T** | `RegisterRuntimeCommand` | `src\lib.rs` |
| **T** | `RuntimeError` | `src\lib.rs` |
| **T** | `RuntimeError` | `src\process.rs` |
| **T** | `RuntimeStatus` | `src\lib.rs` |
| **T** | `SpawnUploadIntegrator` | `src\spawn_upload_integration.rs` |
| **T** | `SseChunk` | `src\sse_parser.rs` |
| **T** | `SseParseError` | `src\sse_parser.rs` |
| **T** | `SseParser` | `src\sse_parser.rs` |
| **T** | `SubscribeError` | `src\subscribe_real.rs` |
| **T** | `UnmountWorktreeCommand` | `src\lib.rs` |
| **T** | `WorktreeMount` | `src\lib.rs` |
| **W** | `GetHeartbeatsQuery` | `src\lib.rs` |
| **W** | `RuntimeHeartbeat` | `src\lib.rs` |

### domain-notification

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `ListByUserQuery` | `src\lib.rs` |
| **M** | `NotificationTemplate` | `src\lib.rs` |
| **M** | `UpsertTemplateCommand` | `src\lib.rs` |
| **T** | `ChannelKind` | `src\lib.rs` |
| **T** | `DispatchNotificationCommand` | `src\lib.rs` |
| **T** | `GetNotificationQuery` | `src\lib.rs` |
| **T** | `InMemoryNotificationRepository` | `src\lib.rs` |
| **T** | `InMemoryNotificationService` | `src\lib.rs` |
| **T** | `MarkReadCommand` | `src\lib.rs` |
| **T** | `Notification` | `src\lib.rs` |
| **T** | `NotificationChannel` | `src\lib.rs` |
| **T** | `NotificationError` | `src\lib.rs` |
| **T** | `NotificationEventType` | `src\lib.rs` |
| **T** | `NotificationStatus` | `src\lib.rs` |
| **T** | `RegisterChannelCommand` | `src\lib.rs` |

### domain-permission

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `GrantRoleCommand` | `src\lib.rs` |
| **M** | `InMemoryPermissionRepository` | `src\lib.rs` |
| **M** | `InMemoryPermissionService` | `src\lib.rs` |
| **M** | `ListRolesQuery` | `src\lib.rs` |
| **M** | `PermissionError` | `src\lib.rs` |
| **M** | `PermissionRule` | `src\lib.rs` |
| **M** | `PermissionScheme` | `src\lib.rs` |
| **M** | `RevokeRoleCommand` | `src\lib.rs` |
| **M** | `Role` | `src\lib.rs` |
| **M** | `RoleBinding` | `src\lib.rs` |
| **T** | `Action` | `src\lib.rs` |
| **T** | `CheckQuery` | `src\lib.rs` |
| **T** | `CreateSchemeCommand` | `src\lib.rs` |
| **T** | `Effect` | `src\lib.rs` |
| **T** | `GetSchemeQuery` | `src\lib.rs` |
| **T** | `ResourceType` | `src\lib.rs` |
| **T** | `SubjectType` | `src\lib.rs` |
| **T** | `UpsertRuleCommand` | `src\lib.rs` |

### domain-planning

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AddToBacklogCommand` | `src\lib.rs` |
| **T** | `AppendBurndownPointCommand` | `src\lib.rs` |
| **T** | `Baseline` | `src\whatif.rs` |
| **T** | `BaselineChange` | `src\whatif.rs` |
| **T** | `BaselineDiff` | `src\whatif.rs` |
| **T** | `BurndownPoint` | `src\lib.rs` |
| **T** | `Capacity` | `src\lib.rs` |
| **T** | `Confidence` | `src\whatif.rs` |
| **T** | `CreateMilestoneCommand` | `src\lib.rs` |
| **T** | `CreateSprintCommand` | `src\lib.rs` |
| **T** | `GetBurndownQuery` | `src\lib.rs` |
| **T** | `GetSprintQuery` | `src\lib.rs` |
| **T** | `InMemoryPlanningRepository` | `src\lib.rs` |
| **T** | `InMemoryPlanningService` | `src\lib.rs` |
| **T** | `ListActiveSprintQuery` | `src\lib.rs` |
| **T** | `Milestone` | `src\lib.rs` |
| **T** | `MilestoneStatus` | `src\lib.rs` |
| **T** | `PlanningError` | `src\lib.rs` |
| **T** | `ScheduleAdjustment` | `src\whatif.rs` |
| **T** | `Sprint` | `src\lib.rs` |
| **T** | `SprintBacklogItem` | `src\lib.rs` |
| **T** | `SprintStatus` | `src\lib.rs` |
| **T** | `WhatIfScenario` | `src\whatif.rs` |
| **T** | `WhatIfService` | `src\whatif.rs` |

### domain-project

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `ProjectPolicy` | `src\entity.rs` |
| **M** | `ProjectPolicy` | `src\lib.rs` |
| **M** | `ProjectPolicyUpdated` | `src\event.rs` |
| **M** | `ProjectTemplate` | `src\entity.rs` |
| **M** | `ProjectTemplate` | `src\lib.rs` |
| **M** | `ProjectTemplateType` | `src\value_object.rs` |
| **M** | `TemplateCategory` | `src\lib.rs` |
| **T** | `ArchiveProjectCommand` | `src\lib.rs` |
| **T** | `ArchiveProjectCommand` | `src\port.rs` |
| **T** | `CreateProjectCommand` | `src\lib.rs` |
| **T** | `CreateProjectCommand` | `src\port.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `GetProjectQuery` | `src\lib.rs` |
| **T** | `InMemoryProjectRepository` | `src\lib.rs` |
| **T** | `InMemoryProjectService` | `src\lib.rs` |
| **T** | `InMemoryProjectService` | `src\service.rs` |
| **T** | `ListByWorkspaceQuery` | `src\lib.rs` |
| **T** | `ListProjectQuery` | `src\port.rs` |
| **T** | `Project` | `src\entity.rs` |
| **T** | `Project` | `src\lib.rs` |
| **T** | `ProjectCreated` | `src\event.rs` |
| **T** | `ProjectError` | `src\error.rs` |
| **T** | `ProjectError` | `src\lib.rs` |
| **T** | `ProjectEvent` | `src\event.rs` |
| **T** | `ProjectStatus` | `src\lib.rs` |
| **T** | `ProjectStatus` | `src\value_object.rs` |
| **T** | `ReplaceProjectPolicyCommand` | `src\lib.rs` |
| **T** | `UpdateProjectCommand` | `src\port.rs` |
| **T** | `UpdateProjectPolicyCommand` | `src\port.rs` |

### domain-relation

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `AddToGroupCommand` | `src\lib.rs` |
| **M** | `CreateRelationGroupCommand` | `src\lib.rs` |
| **M** | `RelationGroup` | `src\lib.rs` |
| **T** | `CreateRelationCommand` | `src\lib.rs` |
| **T** | `DeleteRelationCommand` | `src\lib.rs` |
| **T** | `GetGraphQuery` | `src\lib.rs` |
| **T** | `GraphEdge` | `src\lib.rs` |
| **T** | `GraphNode` | `src\lib.rs` |
| **T** | `InMemoryRelationRepository` | `src\lib.rs` |
| **T** | `InMemoryRelationService` | `src\lib.rs` |
| **T** | `ListByEndpointQuery` | `src\lib.rs` |
| **T** | `ListByTypeQuery` | `src\lib.rs` |
| **T** | `Relation` | `src\lib.rs` |
| **T** | `RelationError` | `src\lib.rs` |
| **T** | `RelationGraph` | `src\lib.rs` |
| **T** | `RelationType` | `src\lib.rs` |
| **T** | `ResourceType` | `src\lib.rs` |

### domain-report

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `GroupRow` | `src\domain\c11_resolution_time.rs` |
| **M** | `Groups` | `src\domain\c04_sprint_report.rs` |
| **M** | `InMemoryPermissionPort` | `src\infrastructure\port_stubs.rs` |
| **M** | `InMemoryUserPort` | `src\infrastructure\port_stubs.rs` |
| **M** | `UserInfo` | `src\application\ports.rs` |
| **T** | `Bucket` | `src\domain\c07_cycle_time.rs` |
| **T** | `BucketAvg` | `src\domain\c08_throughput.rs` |
| **T** | `BucketCount` | `src\domain\c08_throughput.rs` |
| **T** | `BurndownData` | `src\domain\c01_burndown.rs` |
| **T** | `BurndownSeries` | `src\domain\c01_burndown.rs` |
| **T** | `BurndownSummary` | `src\domain\c01_burndown.rs` |
| **T** | `BurnupData` | `src\domain\c02_burnup.rs` |
| **T** | `BurnupSeries` | `src\domain\c02_burnup.rs` |
| **T** | `BurnupSummary` | `src\domain\c02_burnup.rs` |
| **T** | `CfdData` | `src\domain\c05_cfd.rs` |
| **T** | `CompletedIssue` | `src\domain\c01_burndown.rs` |
| **T** | `ControlChartData` | `src\domain\c06_control_chart.rs` |
| **T** | `ControlPoint` | `src\domain\c06_control_chart.rs` |
| **T** | `ControlStats` | `src\domain\c06_control_chart.rs` |
| **T** | `CvrData` | `src\domain\c13_created_vs_resolved.rs` |
| **T** | `CvrSummary` | `src\domain\c13_created_vs_resolved.rs` |
| **T** | `CycleStats` | `src\domain\c07_cycle_time.rs` |
| **T** | `CycleTimeData` | `src\domain\c07_cycle_time.rs` |
| **T** | `DateRange` | `src\domain\c05_cfd.rs` |
| **T** | `DayCompliance` | `src\domain\c12_sla_compliance.rs` |
| **T** | `DayCount` | `src\domain\c05_cfd.rs` |
| **T** | `DayStat` | `src\domain\c13_created_vs_resolved.rs` |
| **T** | `ForecastData` | `src\domain\c09_forecast.rs` |
| **T** | `ForecastResult` | `src\domain\c09_forecast.rs` |
| **T** | `HistoricalData` | `src\domain\c09_forecast.rs` |
| **T** | `InMemorySprintPort` | `src\infrastructure\port_stubs.rs` |
| **T** | `InMemoryWorkItemPort` | `src\infrastructure\port_stubs.rs` |
| **T** | `IssueRow` | `src\domain\c04_sprint_report.rs` |
| **T** | `IssueTypeDistData` | `src\domain\c14_issue_type_dist.rs` |
| **T** | `Percentiles` | `src\domain\c07_cycle_time.rs` |
| **T** | `PriorityDistData` | `src\domain\c15_priority_dist.rs` |
| **T** | `PrioritySlice` | `src\domain\c15_priority_dist.rs` |
| **T** | `PriorityStat` | `src\domain\c12_sla_compliance.rs` |
| **T** | `RefLine` | `src\domain\c06_control_chart.rs` |
| **T** | `Report` | `src\lib.rs` |
| **T** | `ReportError` | `src\lib.rs` |
| **T** | `ReportFilter` | `src\lib.rs` |
| **T** | `ReportPoint` | `src\lib.rs` |
| **T** | `ReportResult` | `src\lib.rs` |
| **T** | `ReportService` | `src\lib.rs` |
| **T** | `ReportSummary` | `src\lib.rs` |
| **T** | `ReportType` | `src\lib.rs` |
| **T** | `ResolutionTimeData` | `src\domain\c11_resolution_time.rs` |
| **T** | `ScopeChange` | `src\domain\c01_burndown.rs` |
| **T** | `SlaData` | `src\domain\c12_sla_compliance.rs` |
| **T** | `SlaSummary` | `src\domain\c12_sla_compliance.rs` |
| **T** | `SprintInfo` | `src\domain\c04_sprint_report.rs` |
| **T** | `SprintMeta` | `src\domain\c01_burndown.rs` |
| **T** | `SprintReportData` | `src\domain\c04_sprint_report.rs` |
| **T** | `SprintReportSummary` | `src\domain\c04_sprint_report.rs` |
| **T** | `SprintStatus` | `src\domain\c03_velocity.rs` |
| **T** | `SprintVelocity` | `src\domain\c03_velocity.rs` |
| **T** | `SprintVelocity` | `src\domain\c09_forecast.rs` |
| **T** | `ThroughputData` | `src\domain\c08_throughput.rs` |
| **T** | `ThroughputStats` | `src\domain\c08_throughput.rs` |
| **T** | `TimeRange` | `src\lib.rs` |
| **T** | `TimeSeriesPoint` | `src\domain\c01_burndown.rs` |
| **T** | `TimeSeriesPoint` | `src\domain\c02_burnup.rs` |
| **T** | `TimeTrackingData` | `src\domain\c10_time_tracking.rs` |
| **T** | `TrackingRow` | `src\domain\c10_time_tracking.rs` |
| **T** | `TrackingSummary` | `src\domain\c10_time_tracking.rs` |
| **T** | `Trend` | `src\lib.rs` |
| **T** | `TypeSlice` | `src\domain\c14_issue_type_dist.rs` |
| **T** | `VelocityData` | `src\domain\c03_velocity.rs` |
| **T** | `VelocityTrend` | `src\domain\c03_velocity.rs` |
| **W** | `InMemoryCache` | `src\infrastructure\in_memory_cache.rs` |

### domain-scm

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `Branch` | `src\lib.rs` |
| **T** | `Commit` | `src\lib.rs` |
| **T** | `ConflictStrategy` | `src\lib.rs` |
| **T** | `EventMeta` | `src\lib.rs` |
| **T** | `ExternalRepositoryId` | `src\lib.rs` |
| **T** | `InMemoryScmService` | `src\lib.rs` |
| **T** | `Pipeline` | `src\lib.rs` |
| **T** | `PipelineStatus` | `src\lib.rs` |
| **T** | `PullRequest` | `src\lib.rs` |
| **T** | `PullRequestState` | `src\lib.rs` |
| **T** | `PullRequestStateChanged` | `src\lib.rs` |
| **T** | `RecordPullRequestTransitionCommand` | `src\lib.rs` |
| **T** | `RegisterRepositoryCommand` | `src\lib.rs` |
| **T** | `Repository` | `src\lib.rs` |
| **T** | `RepositoryOwnership` | `src\lib.rs` |
| **T** | `RepositoryRegistered` | `src\lib.rs` |
| **T** | `Review` | `src\lib.rs` |
| **T** | `ReviewState` | `src\lib.rs` |
| **T** | `ScmError` | `src\lib.rs` |
| **T** | `ScmEvent` | `src\lib.rs` |
| **T** | `ScmProvider` | `src\lib.rs` |
| **T** | `SyncState` | `src\lib.rs` |
| **T** | `SyncStatus` | `src\lib.rs` |
| **T** | `UpdateSyncStateCommand` | `src\lib.rs` |
| **T** | `WebhookEvent` | `src\lib.rs` |
| **T** | `WebhookEventInput` | `src\lib.rs` |
| **T** | `WebhookEventType` | `src\lib.rs` |
| **T** | `WebhookReceived` | `src\lib.rs` |

### domain-search

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `BulkReindexCommand` | `src\lib.rs` |
| **T** | `BulkReindexResult` | `src\lib.rs` |
| **T** | `CmpOp` | `src\jql.rs` |
| **T** | `Comparison` | `src\jql.rs` |
| **T** | `DeleteIndexCommand` | `src\lib.rs` |
| **T** | `DeleteSavedSearchCommand` | `src\lib.rs` |
| **T** | `Facet` | `src\lib.rs` |
| **T** | `FuncCall` | `src\jql.rs` |
| **T** | `InMemorySearchRepository` | `src\lib.rs` |
| **T** | `InMemorySearchService` | `src\lib.rs` |
| **T** | `JqlError` | `src\jql.rs` |
| **T** | `JqlExecutor` | `src\jql.rs` |
| **T** | `JqlExpr` | `src\jql.rs` |
| **T** | `JqlField` | `src\jql.rs` |
| **T** | `JqlParser` | `src\jql.rs` |
| **T** | `JqlValue` | `src\jql.rs` |
| **T** | `OrderByItem` | `src\jql.rs` |
| **T** | `ResourceType` | `src\lib.rs` |
| **T** | `SaveSearchCommand` | `src\lib.rs` |
| **T** | `SavedSearch` | `src\lib.rs` |
| **T** | `SearchError` | `src\lib.rs` |
| **T** | `SearchHit` | `src\lib.rs` |
| **T** | `SearchIndex` | `src\lib.rs` |
| **T** | `SearchQuery` | `src\lib.rs` |
| **T** | `SearchQueryDto` | `src\lib.rs` |
| **T** | `SearchResult` | `src\lib.rs` |
| **T** | `SortDir` | `src\jql.rs` |
| **T** | `SuggestQuery` | `src\lib.rs` |
| **T** | `SuggestQueryDto` | `src\lib.rs` |
| **T** | `Suggestion` | `src\lib.rs` |
| **T** | `SymbolMetadata` | `src\lib.rs` |
| **T** | `UpsertIndexCommand` | `src\lib.rs` |
| **T** | `WorkItemRow` | `src\jql.rs` |

### domain-tenant

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `ChangeTenantStatusCommand` | `src\port.rs` |
| **M** | `CreateTenantCommand` | `src\lib.rs` |
| **M** | `CreateTenantCommand` | `src\port.rs` |
| **M** | `GetTenantQuery` | `src\lib.rs` |
| **M** | `InMemoryTenantRepository` | `src\lib.rs` |
| **M** | `InMemoryTenantService` | `src\lib.rs` |
| **M** | `InMemoryTenantService` | `src\service.rs` |
| **M** | `ListTenantQuery` | `src\port.rs` |
| **M** | `RetentionPolicy` | `src\lib.rs` |
| **M** | `SecurityPolicy` | `src\lib.rs` |
| **M** | `SuspendTenantCommand` | `src\lib.rs` |
| **M** | `Tenant` | `src\entity.rs` |
| **M** | `Tenant` | `src\lib.rs` |
| **M** | `TenantCreated` | `src\event.rs` |
| **M** | `TenantError` | `src\error.rs` |
| **M** | `TenantError` | `src\lib.rs` |
| **M** | `TenantEvent` | `src\event.rs` |
| **M** | `TenantPolicy` | `src\entity.rs` |
| **M** | `TenantPolicy` | `src\lib.rs` |
| **M** | `TenantPolicySpec` | `src\port.rs` |
| **M** | `TenantPolicyUpdated` | `src\event.rs` |
| **M** | `TenantQuota` | `src\entity.rs` |
| **M** | `TenantStatus` | `src\lib.rs` |
| **M** | `TenantStatus` | `src\value_object.rs` |
| **M** | `TenantStatusChanged` | `src\event.rs` |
| **M** | `TenantTier` | `src\value_object.rs` |
| **M** | `UpdateSecurityPolicyCommand` | `src\lib.rs` |
| **M** | `UpdateTenantCommand` | `src\port.rs` |
| **M** | `UpdateTenantPolicyCommand` | `src\lib.rs` |
| **M** | `UpdateTenantPolicyCommand` | `src\port.rs` |
| **T** | `DataKind` | `src\lib.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `PlanTier` | `src\lib.rs` |
| **T** | `ProviderDataBoundary` | `src\lib.rs` |
| **T** | `RegisterProviderBoundaryCommand` | `src\lib.rs` |

### domain-theme

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `Theme` | `src\entity.rs` |
| **M** | `ThemeContext` | `src\context.rs` |
| **M** | `ThemeDefinition` | `src\value_object.rs` |
| **M** | `ThemeError` | `src\error.rs` |
| **M** | `ThemeEvent` | `src\event.rs` |
| **M** | `ThemeId` | `src\value_object.rs` |
| **M** | `ThemeScope` | `src\value_object.rs` |
| **M** | `ThemeService` | `src\service.rs` |
| **T** | `ColorToken` | `src\value_object.rs` |
| **T** | `RadiusToken` | `src\value_object.rs` |
| **T** | `ScopeOwner` | `src\entity.rs` |
| **T** | `SpacingToken` | `src\value_object.rs` |

### domain-validation

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `CreateValidationPolicyCommand` | `src\port.rs` |
| **M** | `ValidationPolicy` | `src\entity.rs` |
| **T** | `AcceptanceCoverage` | `src\entity.rs` |
| **T** | `AcceptanceCoverageAchieved` | `src\event.rs` |
| **T** | `AcceptanceCoverageLinked` | `src\event.rs` |
| **T** | `AcceptanceCoverageReport` | `src\entity.rs` |
| **T** | `ActorContext` | `src\context.rs` |
| **T** | `AddEvidenceCommand` | `src\port.rs` |
| **T** | `CoverageStatus` | `src\value_object.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `EvidenceDownloadURL` | `src\entity.rs` |
| **T** | `EvidenceLinked` | `src\event.rs` |
| **T** | `EvidenceType` | `src\value_object.rs` |
| **T** | `FeedbackRequired` | `src\event.rs` |
| **T** | `InMemoryValidationService` | `src\service.rs` |
| **T** | `LinkAcceptanceEvidenceCommand` | `src\port.rs` |
| **T** | `LinkEvidenceCommand` | `src\port.rs` |
| **T** | `ListValidationQuery` | `src\port.rs` |
| **T** | `MarkValidationStatusCommand` | `src\port.rs` |
| **T** | `OverrideValidationCommand` | `src\port.rs` |
| **T** | `SubmitValidationResultCommand` | `src\port.rs` |
| **T** | `TriggeredBy` | `src\value_object.rs` |
| **T** | `ValidationError` | `src\error.rs` |
| **T** | `ValidationEvent` | `src\event.rs` |
| **T** | `ValidationEvidence` | `src\entity.rs` |
| **T** | `ValidationFailed` | `src\event.rs` |
| **T** | `ValidationKind` | `src\value_object.rs` |
| **T** | `ValidationOverridden` | `src\event.rs` |
| **T** | `ValidationOverride` | `src\entity.rs` |
| **T** | `ValidationPassed` | `src\event.rs` |
| **T** | `ValidationResult` | `src\entity.rs` |
| **T** | `ValidationResultSubmitted` | `src\event.rs` |
| **T** | `ValidationStatus` | `src\value_object.rs` |

### domain-work-item

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AcceptanceCriterion` | `src\entity.rs` |
| **T** | `AcceptanceCriterion` | `src\lib.rs` |
| **T** | `AcceptanceCriterionCovered` | `src\event.rs` |
| **T** | `ActorContext` | `src\context.rs` |
| **T** | `AiTaskData` | `src\lib.rs` |
| **T** | `AssignCommand` | `src\lib.rs` |
| **T** | `BulkFailure` | `src\port.rs` |
| **T** | `BulkResult` | `src\port.rs` |
| **T** | `BusinessGoal` | `src\entity.rs` |
| **T** | `BusinessGoal` | `src\lib.rs` |
| **T** | `CoverageStatus` | `src\entity.rs` |
| **T** | `CoverageStatus` | `src\lib.rs` |
| **T** | `CreateAcceptanceCriterionCommand` | `src\lib.rs` |
| **T** | `CreateAcceptanceCriterionCommand` | `src\port.rs` |
| **T** | `CreateRequirementCommand` | `src\lib.rs` |
| **T** | `CreateRequirementCommand` | `src\port.rs` |
| **T** | `CreateWorkItemCommand` | `src\lib.rs` |
| **T** | `CreateWorkItemCommand` | `src\port.rs` |
| **T** | `DeleteWorkItemCommand` | `src\port.rs` |
| **T** | `EventMeta` | `src\event.rs` |
| **T** | `GetWorkItemQuery` | `src\lib.rs` |
| **T** | `InMemoryWorkItemRepository` | `src\lib.rs` |
| **T** | `InMemoryWorkItemService` | `src\lib.rs` |
| **T** | `InMemoryWorkItemService` | `src\service.rs` |
| **T** | `LinkRepositoryCommand` | `src\port.rs` |
| **T** | `ListBusinessGoalQuery` | `src\port.rs` |
| **T** | `ListByProjectQuery` | `src\lib.rs` |
| **T** | `ListWorkItemQuery` | `src\port.rs` |
| **T** | `Priority` | `src\lib.rs` |
| **T** | `Priority` | `src\value_object.rs` |
| **T** | `RelationType` | `src\value_object.rs` |
| **T** | `Requirement` | `src\entity.rs` |
| **T** | `Requirement` | `src\lib.rs` |
| **T** | `Severity` | `src\lib.rs` |
| **T** | `Severity` | `src\value_object.rs` |
| **T** | `Transition` | `src\port.rs` |
| **T** | `TransitionStatusCommand` | `src\lib.rs` |
| **T** | `TransitionStatusCommand` | `src\port.rs` |
| **T** | `UpdateWorkItemCommand` | `src\port.rs` |
| **T** | `WorkItem` | `src\entity.rs` |
| **T** | `WorkItem` | `src\lib.rs` |
| **T** | `WorkItemBulkUpdate` | `src\port.rs` |
| **T** | `WorkItemCreated` | `src\event.rs` |
| **T** | `WorkItemDeleted` | `src\event.rs` |
| **T** | `WorkItemError` | `src\error.rs` |
| **T** | `WorkItemError` | `src\lib.rs` |
| **T** | `WorkItemEvent` | `src\event.rs` |
| **T** | `WorkItemRelation` | `src\entity.rs` |
| **T** | `WorkItemStatus` | `src\lib.rs` |
| **T** | `WorkItemStatus` | `src\value_object.rs` |
| **T** | `WorkItemStatusChanged` | `src\event.rs` |
| **T** | `WorkItemType` | `src\lib.rs` |
| **T** | `WorkItemType` | `src\value_object.rs` |
| **T** | `WorkItemUpdated` | `src\event.rs` |
| **T** | `WorkItemWorktreeLinked` | `src\event.rs` |

### domain-workflow

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `ListByTenantQuery` | `src\lib.rs` |
| **M** | `NodeCategory` | `src\visualize.rs` |
| **M** | `StateCategory` | `src\lib.rs` |
| **T** | `CreateWorkflowCommand` | `src\lib.rs` |
| **T** | `Guard` | `src\lib.rs` |
| **T** | `InMemoryWorkflowRepository` | `src\lib.rs` |
| **T** | `InMemoryWorkflowService` | `src\lib.rs` |
| **T** | `StartInstanceCommand` | `src\lib.rs` |
| **T** | `StateChange` | `src\lib.rs` |
| **T** | `Transition` | `src\lib.rs` |
| **T** | `TransitionCommand` | `src\lib.rs` |
| **T** | `TransitionTrigger` | `src\lib.rs` |
| **T** | `VizEdge` | `src\visualize.rs` |
| **T** | `VizNode` | `src\visualize.rs` |
| **T** | `Workflow` | `src\lib.rs` |
| **T** | `WorkflowError` | `src\lib.rs` |
| **T** | `WorkflowInstance` | `src\lib.rs` |
| **T** | `WorkflowState` | `src\lib.rs` |
| **T** | `WorkflowViz` | `src\visualize.rs` |

### domain-workspace

| 業務分類 | entity 名 | file |
|---|---|---|
| **M** | `WorkspaceRole` | `src\lib.rs` |
| **T** | `AddMemberCommand` | `src\lib.rs` |
| **T** | `CreateWorkspaceCommand` | `src\lib.rs` |
| **T** | `EventMeta` | `src\lib.rs` |
| **T** | `InMemoryWorkspaceService` | `src\lib.rs` |
| **T** | `ListWorkspaceQuery` | `src\lib.rs` |
| **T** | `MemberAdded` | `src\lib.rs` |
| **T** | `MemberRemoved` | `src\lib.rs` |
| **T** | `RemoveMemberCommand` | `src\lib.rs` |
| **T** | `UpdateWorkspaceCommand` | `src\lib.rs` |
| **T** | `Workspace` | `src\lib.rs` |
| **T** | `WorkspaceCreated` | `src\lib.rs` |
| **T** | `WorkspaceError` | `src\lib.rs` |
| **T** | `WorkspaceEvent` | `src\lib.rs` |
| **T** | `WorkspaceMember` | `src\lib.rs` |

### domain-worktree

| 業務分類 | entity 名 | file |
|---|---|---|
| **T** | `AbandonCommand` | `src\lib.rs` |
| **T** | `AssignWorktreeCommand` | `src\lib.rs` |
| **T** | `ConflictState` | `src\lib.rs` |
| **T** | `CreateWorktreeCommand` | `src\lib.rs` |
| **T** | `HealthState` | `src\lib.rs` |
| **T** | `HeatmapData` | `src\lib.rs` |
| **T** | `InMemoryWorktreeRepository` | `src\lib.rs` |
| **T** | `InMemoryWorktreeService` | `src\lib.rs` |
| **T** | `ListByAgentQuery` | `src\lib.rs` |
| **T** | `ListByWorkItemQuery` | `src\lib.rs` |
| **T** | `RecordObservedStateCommand` | `src\lib.rs` |
| **T** | `Staleness` | `src\lib.rs` |
| **T** | `TransitionStatusCommand` | `src\lib.rs` |
| **T** | `Worktree` | `src\lib.rs` |
| **T** | `WorktreeError` | `src\lib.rs` |
| **T** | `WorktreeStatus` | `src\lib.rs` |
| **T** | `WorktreeSummary` | `src\lib.rs` |

## §5 既知の缺口 / 制約

1. **混合分類**: P3-D 段階 entity 中, 業務分類が 1 つの entity で複数軸に該当する場合あり (e.g. `agent_session` は T (業務事実) + W (active session 観測), 主分類で計上)
2. **V2 候補**: V2 化 (LangGraph 統合 / Agent Runtime 1M agents / Tree-sitter) で T → W 降格候補あり (per 00-CLASSIFICATION-RULES.md §7)
3. **Frontend 同期**: P3-D 22 domain-* crate Backend Rust のみ W/T/M 適用, Frontend Zustand store 状態分類は未同期 (per 同 §7)
4. **新規 crate**: 派生新規 crate (e.g. `star-dispatcher` v0.0.1) 28 entity 仍未在本報告 (P3-D 範囲外, P3-G 阶段扩展)
5. **DDD Review**: 5 域 Lead 真人到位后, 分類結果 Review + 修正 (per 守门 #14 5 域 Lead CONTENT 4 維)

## §6 関連文档

- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 (Star 100 テーブル W/T/M 三類索引実例)
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1 (跨項目ルール手册 + 4 段检查清单 + 派生守门 10 条)
- `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` (Agent Runtime SRS, P3-D 段階 1 entity 來源)
- `crates/star-saga/src/saga_5b_services.rs` (P3-E 阶段, 不在本 P3-D 報告範圍)
- `crates/star-dispatcher/src/lib.rs` (P3-G 阶段, 不在本 P3-D 報告範圍)
- `scripts/automation/wtm_classifier.py` v0.1 (本報告生成腳本, 守门 #19 [M] 拍板)
- `AGENTS.md` 守门 #DB-13 (DB 三類横展開強制分類)
- `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §F.4 (P3-D 跨項目 落地計画)
