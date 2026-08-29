//! Validation 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(ValidationId);
define_uuid_id!(ValidationEvidenceId);
define_uuid_id!(AcceptanceCoverageId);
define_uuid_id!(AcceptanceCriterionId);
define_uuid_id!(ValidationPolicyId);
define_uuid_id!(ValidationOverrideId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);
define_uuid_id!(WorkItemId);
define_uuid_id!(WorktreeId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(ChangeSetId);
define_uuid_id!(CommitId);

// =====================================================================
// 枚举:ValidationKind(7 类,SOW §实施范围 + spec §4.5.3)
// =====================================================================

/// **Validation 类型**(SOW 要求 7 类;data-design 列出 10 类,本 crate 实现 SOW 指定的 7 类 + 3 类附加)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationKind {
    /// CI / Local Runtime 编译
    Build,
    /// 单元测试
    UnitTest,
    /// 集成测试(CI)
    IntegrationTest,
    /// 静态检查(clippy / eslint)
    Lint,
    /// 格式检查(rustfmt / prettier)
    Format,
    /// 静态分析(SAST / type check)
    StaticAnalysis,
    /// 安全检查(dependency scan / secret scan)
    SecurityCheck,
    // 附加:与 data-design §4.24.1 ck_validation_kind 对齐
    /// 验收检查(AI / 人工)
    AcceptanceCheck,
    /// 代码评审
    Review,
    /// 用户自定义校验
    CustomValidation,
}

impl ValidationKind {
    /// SOW 指定的 7 类必交付类别
    pub const SOW_REQUIRED: &'static [ValidationKind] = &[
        ValidationKind::Build,
        ValidationKind::UnitTest,
        ValidationKind::IntegrationTest,
        ValidationKind::Lint,
        ValidationKind::Format,
        ValidationKind::StaticAnalysis,
        ValidationKind::SecurityCheck,
    ];

    /// 是否在 SOW 必交付 7 类中
    pub fn is_sow_required(&self) -> bool {
        Self::SOW_REQUIRED.contains(self)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Build => "BUILD",
            Self::UnitTest => "UNIT_TEST",
            Self::IntegrationTest => "INTEGRATION_TEST",
            Self::Lint => "LINT",
            Self::Format => "FORMAT",
            Self::StaticAnalysis => "STATIC_ANALYSIS",
            Self::SecurityCheck => "SECURITY_CHECK",
            Self::AcceptanceCheck => "ACCEPTANCE_CHECK",
            Self::Review => "REVIEW",
            Self::CustomValidation => "CUSTOM_VALIDATION",
        }
    }
}

impl std::fmt::Display for ValidationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:ValidationStatus(5 状态,SOW §实施范围;spec §A.5 列出 6 状态含 ERRORED)
// =====================================================================

/// **Validation 状态**(SOW 要求 5 状态:PENDING/RUNNING/PASSED/FAILED/SKIPPED)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationStatus {
    /// 已创建,等待执行
    Pending,
    /// 正在执行
    Running,
    /// 全部通过
    Passed,
    /// 至少一项断言失败
    Failed,
    /// 跳过(Policy 禁用 / 条件不满足)
    Skipped,
}

impl ValidationStatus {
    /// 是否终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Passed | Self::Failed | Self::Skipped)
    }
}

impl Default for ValidationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "PENDING",
            Self::Running => "RUNNING",
            Self::Passed => "PASSED",
            Self::Failed => "FAILED",
            Self::Skipped => "SKIPPED",
        };
        f.write_str(s)
    }
}

/// **状态机迁移合法性**(基础设计 §A.5)
pub fn is_valid_state_transition(from: ValidationStatus, to: ValidationStatus) -> bool {
    use ValidationStatus::*;
    // 同态禁止
    if from == to {
        return false;
    }
    // 终态不可迁出
    if from.is_terminal() {
        return false;
    }
    // Pending -> Running / Skipped
    // Running -> Passed / Failed / Skipped
    match (from, to) {
        (Pending, Running) => true,
        (Pending, Skipped) => true,
        (Pending, Passed) => true, // 立即通过(如 Cache Hit)
        (Pending, Failed) => true,
        (Running, Passed) => true,
        (Running, Failed) => true,
        (Running, Skipped) => true,
        _ => false,
    }
}

// =====================================================================
// 枚举:TriggeredBy
// =====================================================================

/// **触发方**(data-design §4.24.1)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriggeredBy {
    User,
    Agent,
    Webhook,
    Schedule,
}

impl TriggeredBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "USER",
            Self::Agent => "AGENT",
            Self::Webhook => "WEBHOOK",
            Self::Schedule => "SCHEDULE",
        }
    }
}

impl std::fmt::Display for TriggeredBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// 枚举:EvidenceType(data-design §4.24.2)
// =====================================================================

/// **ValidationEvidence 类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceType {
    TestReport,
    BuildLog,
    CoverageReport,
    StaticAnalysis,
    Screenshot,
    LogExcerpt,
}

impl EvidenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TestReport => "TEST_REPORT",
            Self::BuildLog => "BUILD_LOG",
            Self::CoverageReport => "COVERAGE_REPORT",
            Self::StaticAnalysis => "STATIC_ANALYSIS",
            Self::Screenshot => "SCREENSHOT",
            Self::LogExcerpt => "LOG_EXCERPT",
        }
    }
}

// =====================================================================
// 枚举:CoverageStatus(data-design §4.24.3)
// =====================================================================

/// **AcceptanceCoverage 状态**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoverageStatus {
    /// 完全覆盖
    Covered,
    /// 部分覆盖
    Partial,
    /// 未覆盖
    Uncovered,
    /// 存在争议
    Disputed,
}

impl Default for CoverageStatus {
    fn default() -> Self {
        Self::Uncovered
    }
}

// =====================================================================
// 角色
// =====================================================================

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
    /// CI / Local Runtime 服务身份(SOW §实施范围 submit_result)
    pub const SERVICE_INTERNAL: &str = "service_internal";
}
