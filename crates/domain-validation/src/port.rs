//! Validation 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:`docs/specs/domain-validation-spec.md` §4

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{
    AcceptanceCoverage, AcceptanceCoverageReport, EvidenceDownloadURL, ValidationEvidence,
    ValidationOverride, ValidationPolicy, ValidationResult,
};
use crate::error::ValidationError;
use crate::value_object::{
    EvidenceType, ProjectId, TenantId, UserId, ValidationEvidenceId, ValidationId,
    ValidationKind, ValidationPolicyId, WorkItemId, WorktreeId,
};

// =====================================================================
// 命令 DTO
// =====================================================================

/// **SubmitValidationResultCommand**(spec §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitValidationResultCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub work_item_id: Option<WorkItemId>,
    pub worktree_id: Option<WorktreeId>,
    pub kind: ValidationKind,
    /// 必带 evidence_ref(INV-VL-04)
    pub log_excerpt_ref: String,
    /// 可选关联 Evidence IDs
    pub evidence_ids: Vec<ValidationEvidenceId>,
    /// 触发方
    pub triggered_by_id: Option<uuid::Uuid>,
    pub policy_id: Option<ValidationPolicyId>,
    pub policy_required: bool,
    /// AI 自我声明完成(VAL-001 强约束:默认 false)
    pub is_ai_complete_claim: bool,
}

/// **OverrideValidationCommand**(spec §4,Protected 鉴权)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideValidationCommand {
    pub tenant_id: TenantId,
    pub validation_id: ValidationId,
    pub reason: String,
    pub approver_user_id: UserId,
}

/// **LinkEvidenceCommand**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEvidenceCommand {
    pub tenant_id: TenantId,
    pub validation_id: ValidationId,
    pub evidence_id: ValidationEvidenceId,
}

/// **LinkAcceptanceEvidenceCommand**(AC -> ValidationResult 映射)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkAcceptanceEvidenceCommand {
    pub tenant_id: TenantId,
    pub work_item_id: WorkItemId,
    pub acceptance_criterion_id: uuid::Uuid,
    pub validation_id: ValidationId,
}

/// **CreateValidationPolicyCommand**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateValidationPolicyCommand {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub required_kinds: Vec<ValidationKind>,
    pub optional_kinds: Vec<ValidationKind>,
    pub pass_thresholds: std::collections::HashMap<String, f64>,
    pub allow_ai_self_claim: bool,
    pub override_allow: bool,
}

/// **AddEvidenceCommand**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEvidenceCommand {
    pub tenant_id: TenantId,
    pub validation_id: ValidationId,
    pub evidence_type: EvidenceType,
    pub storage_ref: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
}

/// **MarkValidationStatusCommand**(内部使用,Service-Internal 触发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkValidationStatusCommand {
    pub tenant_id: TenantId,
    pub validation_id: ValidationId,
    pub new_status: crate::value_object::ValidationStatus,
    pub failure_summary: Option<String>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// **ListValidationQuery**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListValidationQuery {
    pub tenant_id: TenantId,
    pub work_item_id: Option<WorkItemId>,
    pub worktree_id: Option<WorktreeId>,
    pub kind: Option<ValidationKind>,
    pub status: Option<crate::value_object::ValidationStatus>,
    pub limit: u32,
    pub offset: u32,
}

// =====================================================================
// 端口:ValidationCommandPort
// =====================================================================

/// **Validation 命令端口**(spec §4)
#[async_trait]
pub trait ValidationCommandPort: Send + Sync {
    /// Service-Internal(CI / Local Runtime)提交 ValidationResult(INV-VL-04 必带 evidence)
    async fn submit_result(
        &self,
        cmd: SubmitValidationResultCommand,
        actor: ActorContext,
    ) -> Result<ValidationResult, ValidationError>;

    /// 状态变更(Service-Internal)
    async fn mark_status(
        &self,
        cmd: MarkValidationStatusCommand,
        actor: ActorContext,
    ) -> Result<ValidationResult, ValidationError>;

    /// 人类 Override(Protected,INV-VL-06)
    async fn override_result(
        &self,
        cmd: OverrideValidationCommand,
        actor: ActorContext,
    ) -> Result<ValidationOverride, ValidationError>;

    /// 关联 AC 与 ValidationResult
    async fn link_to_acceptance_criterion(
        &self,
        cmd: LinkAcceptanceEvidenceCommand,
        actor: ActorContext,
    ) -> Result<AcceptanceCoverage, ValidationError>;

    /// 追加 Evidence
    async fn link_evidence(
        &self,
        cmd: LinkEvidenceCommand,
        actor: ActorContext,
    ) -> Result<ValidationEvidence, ValidationError>;

    /// 添加 Evidence 子实体(创建 ValidationEvidence)
    async fn add_evidence(
        &self,
        cmd: AddEvidenceCommand,
        actor: ActorContext,
    ) -> Result<ValidationEvidence, ValidationError>;

    /// 创建 ValidationPolicy
    async fn create_policy(
        &self,
        cmd: CreateValidationPolicyCommand,
        actor: ActorContext,
    ) -> Result<ValidationPolicy, ValidationError>;
}

// =====================================================================
// 端口:ValidationQueryPort
// =====================================================================

/// **Validation 查询端口**(spec §4)
#[async_trait]
pub trait ValidationQueryPort: Send + Sync {
    async fn list_results(
        &self,
        q: ListValidationQuery,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationResult>, ValidationError>;
    async fn get_result(
        &self,
        id: ValidationId,
        viewer: ActorContext,
    ) -> Result<ValidationResult, ValidationError>;
    async fn get_evidence_url(
        &self,
        id: ValidationEvidenceId,
        viewer: ActorContext,
    ) -> Result<EvidenceDownloadURL, ValidationError>;
    async fn get_acceptance_coverage(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<AcceptanceCoverageReport, ValidationError>;
    async fn list_policies(
        &self,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationPolicy>, ValidationError>;
    async fn list_evidence(
        &self,
        validation_id: ValidationId,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationEvidence>, ValidationError>;
}

// =====================================================================
// 仓库端口
// =====================================================================

/// **Validation 仓库端口**
#[async_trait]
pub trait ValidationRepository: Send + Sync {
    async fn insert_result(&self, r: &ValidationResult) -> Result<(), ValidationError>;
    async fn save_result(&self, r: &ValidationResult) -> Result<(), ValidationError>;
    async fn find_result(
        &self,
        id: ValidationId,
    ) -> Result<Option<ValidationResult>, ValidationError>;
    async fn list_results_raw(
        &self,
        work_item_id: Option<WorkItemId>,
        worktree_id: Option<WorktreeId>,
        kind: Option<ValidationKind>,
        status: Option<crate::value_object::ValidationStatus>,
    ) -> Result<Vec<ValidationResult>, ValidationError>;

    async fn insert_evidence(&self, e: &ValidationEvidence) -> Result<(), ValidationError>;
    async fn find_evidence(
        &self,
        id: ValidationEvidenceId,
    ) -> Result<Option<ValidationEvidence>, ValidationError>;
    async fn list_evidence_by_validation(
        &self,
        validation_id: ValidationId,
    ) -> Result<Vec<ValidationEvidence>, ValidationError>;
    async fn save_evidence(&self, e: &ValidationEvidence) -> Result<(), ValidationError>;

    async fn insert_coverage(&self, c: &AcceptanceCoverage) -> Result<(), ValidationError>;
    async fn save_coverage(&self, c: &AcceptanceCoverage) -> Result<(), ValidationError>;
    async fn find_coverage_by_criterion(
        &self,
        ac_id: uuid::Uuid,
    ) -> Result<Option<AcceptanceCoverage>, ValidationError>;
    async fn list_coverage_by_work_item(
        &self,
        work_item_id: WorkItemId,
    ) -> Result<Vec<AcceptanceCoverage>, ValidationError>;

    async fn insert_policy(&self, p: &ValidationPolicy) -> Result<(), ValidationError>;
    async fn find_policy(
        &self,
        id: ValidationPolicyId,
    ) -> Result<Option<ValidationPolicy>, ValidationError>;
    async fn list_policies_raw(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ValidationPolicy>, ValidationError>;

    async fn insert_override(&self, o: &ValidationOverride) -> Result<(), ValidationError>;
}
