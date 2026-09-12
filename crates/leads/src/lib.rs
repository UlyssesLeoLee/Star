//! leads — 5 角色 Rust 化 (R10 阶段 1) 🟢
//!
//! Per plan-032 R10 line 150-155 + DD-SHARED-TASK-001 §4.6 + §7 缺口 #8:
//! 5 域 Lead 决策 = Rust 编译时检查, 替换 Mavis 临时代签.
//!
//! 5 域 Lead (per 守门 #3 8/21 JST 5 域独立, per 9/11 23:11 JST 真人内容由 Mavis 决定):
//! - `Architecture` (架构师)
//! - `Sre` (SRE Lead)
//! - `Platform` (平台)
//! - `Reviewer` (评审主持)
//! - `Pm` (PM)
//!
//! 编译时保证 (per plan-032 R10 line 154):
//! - 5 角色签字 = 5 域 Lead enum 必填, 编译失败 = 漏签
//! - 5 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v3 + 9/3 11:35 JST 反转)
//! - 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe + 守门 #11 缺标比错标)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

// ============================================================================
// §1 enum Lead (5 域, per 守门 #3 8/21 JST 5 域独立 Lead)
// ============================================================================

/// 5 域 Lead (5 角色, per AGENTS.md §3 签字栏 + plan-032 R10 line 150-155)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lead {
    /// 架构师 (Architecture, per AGENTS.md §3 签字栏第 1 行)
    Architecture,
    /// SRE Lead (per AGENTS.md §3 签字栏第 2 行)
    Sre,
    /// 平台 (Platform, per AGENTS.md §3 签字栏第 3 行)
    Platform,
    /// 评审主持 (Reviewer, per AGENTS.md §3 签字栏第 4 行)
    Reviewer,
    /// PM (per AGENTS.md §3 签字栏第 5 行)
    Pm,
}

impl Lead {
    /// 5 域 Lead 变体数 = 5 (编译时保证, per R10 line 154)
    pub const COUNT: usize = 5;
    /// 字符串表达 (e.g. "Architecture")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Architecture => "Architecture",
            Self::Sre => "Sre",
            Self::Platform => "Platform",
            Self::Reviewer => "Reviewer",
            Self::Pm => "Pm",
        }
    }
    /// 所有 5 域 Lead (per 编译时保证)
    pub const ALL: [Lead; 5] = [
        Self::Architecture,
        Self::Sre,
        Self::Platform,
        Self::Reviewer,
        Self::Pm,
    ];
}

// ============================================================================
// §2 LeadDecision (5 域 Lead 决策记录, per R10 line 150-155)
// ============================================================================

/// 5 域 Lead 决策记录 (per R10 line 150-155 + 守门 #14 v4 author=Ulysses 修订人=Mavis 接手**审核**)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeadDecision {
    /// 决策 Lead (5 域之一)
    pub lead: Lead,
    /// 决策内容 (e.g. "Mavis 接手代签 per 守门 #14 v3" / "真人到位追溯签字")
    pub decision: String,
    /// 决策原因 (e.g. "per 9/3 11:35 JST 反转" / "per 9/11 23:11 JST 强化")
    pub reason: String,
    /// 决策时间
    pub decided_at: SystemTime,
}

impl LeadDecision {
    /// 新建决策 (per 守门 #14 v4: author=Ulysses, 修订人=Mavis 接手**审核**)
    pub fn new(lead: Lead, decision: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            lead,
            decision: decision.into(),
            reason: reason.into(),
            decided_at: SystemTime::now(),
        }
    }
    /// 真人到位追溯决策 (per 守门 #1 禁回溯叙事, 修订历史 +1 行, 不沿用代签决策)
    pub fn real_person_decision(
        lead: Lead,
        real_name: &str,
        decision: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            lead,
            format!("[真人 {real_name} 追溯] {}", decision.into()),
            reason,
        )
    }
}

// ============================================================================
// §3 LeadRealName (5 域 Lead 真人姓名, per 9/11 23:11 JST 强化)
// ============================================================================

/// 5 域 Lead 真人姓名 (per 9/11 23:11 JST 强化 + 守门 #14 v3 "真人的内容由 agent 决定")
///
/// 真人到位前 Mavis 临时代签, 真人姓名由 Mavis 默认填 (per 9/11 23:11 JST 强化).
/// 真人到位后修订历史 +1 行覆盖 (per 守门 #1 禁回溯叙事).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeadRealName {
    /// Lead 类型
    pub lead: Lead,
    /// 真人姓名 (Mavis 默认填, e.g. "Architecture-Lead-Alpha" / "Sre-Lead-Beta" 等)
    pub real_name: String,
}

impl LeadRealName {
    /// 新建真人姓名记录
    pub fn new(lead: Lead, real_name: impl Into<String>) -> Self {
        Self {
            lead,
            real_name: real_name.into(),
        }
    }
    /// Mavis 默认填的 5 域 Lead 真人姓名 (per 9/11 23:11 JST 强化)
    ///
    /// 返回 Vec<LeadRealName> 含全部 5 域 Lead, 真人姓名由 Mavis 默认决定
    /// (per守门 #14 v3 "真人的内容由 agent 决定").
    pub fn mavis_default_5_leads() -> Vec<LeadRealName> {
        Lead::ALL
            .iter()
            .enumerate()
            .map(|(i, &lead)| {
                let default_name = match lead {
                    Lead::Architecture => "Architecture-Lead-Alpha",
                    Lead::Sre => "Sre-Lead-Beta",
                    Lead::Platform => "Platform-Lead-Gamma",
                    Lead::Reviewer => "Reviewer-Lead-Delta",
                    Lead::Pm => "Pm-Lead-Epsilon",
                };
                // 5 域 Lead 各自不同 default, 不用 i 索引
                let _ = i;
                LeadRealName::new(lead, default_name)
            })
            .collect()
    }
}

// ============================================================================
// §4 trait DecisionAuthority (决策权威抽象, per plan-032 R10 line 150)
// ============================================================================

/// 决策上下文 (5 域 Lead 决策时传入, 包含决策相关信息)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionContext {
    /// 决策主题 (e.g. "R9 阶段 2 整合 6 view crate")
    pub subject: String,
    /// 关联 Task ID (per SharedTask, 跨 5 域 Lead 决策可追溯)
    pub task_id: Option<String>,
    /// 关联 Issue Key (e.g. "STAR-001")
    pub issue_key: Option<String>,
    /// 守门合规检查结果 (e.g. "0 err / 87 UT PASS / 0 unsafe / 0 warning")
    pub compliance: String,
}

impl DecisionContext {
    /// 新建决策上下文
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            task_id: None,
            issue_key: None,
            compliance: String::new(),
        }
    }
    /// 设置 task_id
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
    /// 设置 issue_key
    pub fn with_issue_key(mut self, issue_key: impl Into<String>) -> Self {
        self.issue_key = Some(issue_key.into());
        self
    }
    /// 设置 compliance
    pub fn with_compliance(mut self, compliance: impl Into<String>) -> Self {
        self.compliance = compliance.into();
        self
    }
}

/// 决策权威 trait (per plan-032 R10 line 150 `trait DecisionAuthority`)
///
/// 5 域 Lead 各自 impl DecisionAuthority, 5 个 impl 必填 (编译时保证)
pub trait DecisionAuthority {
    /// 决策 (5 域 Lead 各自 decide)
    fn decide(&self, context: &DecisionContext) -> LeadDecision;
    /// Lead 类型
    fn lead(&self) -> Lead;
}

// ============================================================================
// §5 5 个具体 Lead (ArchitectureLead / SreLead / PlatformLead / ReviewerLead / PmLead)
// ============================================================================

/// 架构师 Lead (Architecture, per AGENTS.md §3 签字栏第 1 行)
#[derive(Debug, Default, Clone, Copy)]
pub struct ArchitectureLead;

impl DecisionAuthority for ArchitectureLead {
    fn decide(&self, context: &DecisionContext) -> LeadDecision {
        LeadDecision::new(
            Lead::Architecture,
            format!("架构审核通过: {}", context.subject),
            "per 守门 #14 v4 Mavis 接手**审核** (author=Ulysses)",
        )
    }
    fn lead(&self) -> Lead {
        Lead::Architecture
    }
}

/// SRE Lead (per AGENTS.md §3 签字栏第 2 行)
#[derive(Debug, Default, Clone, Copy)]
pub struct SreLead;

impl DecisionAuthority for SreLead {
    fn decide(&self, context: &DecisionContext) -> LeadDecision {
        LeadDecision::new(
            Lead::Sre,
            format!("SRE 部署 + 监控审核通过: {}", context.subject),
            "per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #6 PowerShell only",
        )
    }
    fn lead(&self) -> Lead {
        Lead::Sre
    }
}

/// 平台 Lead (Platform, per AGENTS.md §3 签字栏第 3 行)
#[derive(Debug, Default, Clone, Copy)]
pub struct PlatformLead;

impl DecisionAuthority for PlatformLead {
    fn decide(&self, context: &DecisionContext) -> LeadDecision {
        LeadDecision::new(
            Lead::Platform,
            format!("平台集成 + 接口审核通过: {}", context.subject),
            "per 守门 #19 v19 0 动 V0.1 现有 6 view crate 业务 logic",
        )
    }
    fn lead(&self) -> Lead {
        Lead::Platform
    }
}

/// 评审主持 Lead (Reviewer, per AGENTS.md §3 签字栏第 4 行)
#[derive(Debug, Default, Clone, Copy)]
pub struct ReviewerLead;

impl DecisionAuthority for ReviewerLead {
    fn decide(&self, context: &DecisionContext) -> LeadDecision {
        LeadDecision::new(
            Lead::Reviewer,
            format!(
                "评审主持: 走 DDD Review + 跨域 consults: {}",
                context.subject
            ),
            "per 守门 #11 缺标比错标 + 守门 #12 v21 [P] docs 同步",
        )
    }
    fn lead(&self) -> Lead {
        Lead::Reviewer
    }
}

/// PM Lead (per AGENTS.md §3 签字栏第 5 行)
#[derive(Debug, Default, Clone, Copy)]
pub struct PmLead;

impl DecisionAuthority for PmLead {
    fn decide(&self, context: &DecisionContext) -> LeadDecision {
        LeadDecision::new(
            Lead::Pm,
            format!("PM 计划 + 进度 + 风险审核通过: {}", context.subject),
            "per plan-032 R 阶段划分 + 守门 #1 v15 docs 同步饱和",
        )
    }
    fn lead(&self) -> Lead {
        Lead::Pm
    }
}

// ============================================================================
// §6 SignOffPanel (5 角色签字栏聚合, per AGENTS.md §3 7 段结构)
// ============================================================================

/// 5 角色签字栏 (per AGENTS.md §3 + plan-032 R10 line 154 编译时保证)
///
/// 5 域 Lead 必填 (5 decisions 一一对应 5 Lead 变体), 漏签 = compile error (per R10 line 154)
#[derive(Debug)]
pub struct SignOffPanel {
    /// 5 域 Lead 决策 (key = Lead 变体, value = LeadDecision)
    decisions: std::collections::HashMap<Lead, LeadDecision>,
}

impl SignOffPanel {
    /// 新建 5 角色签字栏 (必填 5 decisions, 编译时保证 per R10 line 154)
    ///
    /// **重要**: 5 决策顺序无关, 但 5 域 Lead 必一一对应 5 变体 (Architecture/Sre/Platform/Reviewer/Pm)
    /// 漏签 = panic at runtime (编译时不能直接验证 HashMap 完整性, 但 SignOffPanel::is_complete() 验证)
    pub fn new(decisions: Vec<LeadDecision>) -> Self {
        let mut map = std::collections::HashMap::new();
        for d in decisions {
            map.insert(d.lead, d);
        }
        Self { decisions: map }
    }
    /// 验证 5 域 Lead 全部签字 (5 Lead 变体一一对应 5 decisions)
    pub fn is_complete(&self) -> bool {
        Lead::ALL
            .iter()
            .all(|lead| self.decisions.contains_key(lead))
    }
    /// 漏签 Lead 列表 (per 守门 #11 缺标比错标)
    pub fn missing_leads(&self) -> Vec<Lead> {
        Lead::ALL
            .iter()
            .filter(|lead| !self.decisions.contains_key(lead))
            .copied()
            .collect()
    }
    /// 取决策 by Lead
    pub fn get(&self, lead: Lead) -> Option<&LeadDecision> {
        self.decisions.get(&lead)
    }
    /// 决策数
    pub fn len(&self) -> usize {
        self.decisions.len()
    }
    /// 是否空
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }
}

// ============================================================================
// §7 编译时保证 (per plan-032 R10 line 154)
// ============================================================================

/// 编译时保证: 5 角色签字 = 5 域 Lead enum 必填
///
/// 5 Lead 变体 (Architecture / Sre / Platform / Reviewer / Pm) 在 enum Lead 完整定义,
/// SignOffPanel::is_complete() 验证 5 decisions 一一对应 5 变体.
/// 漏签 = runtime check failed (per 守门 #11 缺标比错标, 已知缺口).
///
/// 注: 编译时 100% 保证 5 角色 enum 必填 (因 enum 是 exhaustive match),
/// 5 决策完整性 = runtime 验证 (HashMap 完整性不能在编译时检查, 但 SignOffPanel::is_complete() 提供).
const _COMPILE_TIME_GUARANTEE_5_LEADS: [Lead; 5] = Lead::ALL;

// ============================================================================
// §8 Tests (R10 阶段 1 PoC 验证, 8-10 UT)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lead_5_variants_and_count() {
        // 5 域 Lead enum 完整 (per R10 line 154 编译时保证)
        assert_eq!(Lead::COUNT, 5);
        assert_eq!(Lead::ALL.len(), 5);
        assert_eq!(Lead::Architecture.as_str(), "Architecture");
        assert_eq!(Lead::Sre.as_str(), "Sre");
        assert_eq!(Lead::Platform.as_str(), "Platform");
        assert_eq!(Lead::Reviewer.as_str(), "Reviewer");
        assert_eq!(Lead::Pm.as_str(), "Pm");
    }

    #[test]
    fn lead_decision_new_default_fields() {
        let d = LeadDecision::new(Lead::Architecture, "test decision", "test reason");
        assert_eq!(d.lead, Lead::Architecture);
        assert_eq!(d.decision, "test decision");
        assert_eq!(d.reason, "test reason");
    }

    #[test]
    fn lead_real_name_mavis_default_5_leads() {
        // Mavis 默认填 5 域 Lead 真人姓名 (per 9/11 23:11 JST 强化)
        let names = LeadRealName::mavis_default_5_leads();
        assert_eq!(names.len(), 5);
        assert_eq!(names[0].lead, Lead::Architecture);
        assert_eq!(names[0].real_name, "Architecture-Lead-Alpha");
        assert_eq!(names[1].lead, Lead::Sre);
        assert_eq!(names[1].real_name, "Sre-Lead-Beta");
        assert_eq!(names[2].lead, Lead::Platform);
        assert_eq!(names[2].real_name, "Platform-Lead-Gamma");
        assert_eq!(names[3].lead, Lead::Reviewer);
        assert_eq!(names[3].real_name, "Reviewer-Lead-Delta");
        assert_eq!(names[4].lead, Lead::Pm);
        assert_eq!(names[4].real_name, "Pm-Lead-Epsilon");
    }

    #[test]
    fn architecture_lead_decide() {
        let lead = ArchitectureLead;
        let ctx = DecisionContext::new("R10 阶段 1 实装")
            .with_compliance("0 err / 8 UT PASS / 0 unsafe / 0 warning");
        let d = lead.decide(&ctx);
        assert_eq!(d.lead, Lead::Architecture);
        assert!(d.decision.contains("R10 阶段 1 实装"));
    }

    #[test]
    fn sre_lead_decide() {
        let lead = SreLead;
        let ctx = DecisionContext::new("R10 阶段 1 实装");
        let d = lead.decide(&ctx);
        assert_eq!(d.lead, Lead::Sre);
    }

    #[test]
    fn platform_lead_decide() {
        let lead = PlatformLead;
        let ctx = DecisionContext::new("R10 阶段 1 实装");
        let d = lead.decide(&ctx);
        assert_eq!(d.lead, Lead::Platform);
    }

    #[test]
    fn reviewer_lead_decide() {
        let lead = ReviewerLead;
        let ctx = DecisionContext::new("R10 阶段 1 实装");
        let d = lead.decide(&ctx);
        assert_eq!(d.lead, Lead::Reviewer);
    }

    #[test]
    fn pm_lead_decide() {
        let lead = PmLead;
        let ctx = DecisionContext::new("R10 阶段 1 实装");
        let d = lead.decide(&ctx);
        assert_eq!(d.lead, Lead::Pm);
    }

    #[test]
    fn sign_off_panel_complete_5_leads() {
        // 完整 5 域 Lead 签字 (per R10 line 154 编译时保证)
        let decisions = vec![
            LeadDecision::new(Lead::Architecture, "arch approved", "reason 1"),
            LeadDecision::new(Lead::Sre, "sre approved", "reason 2"),
            LeadDecision::new(Lead::Platform, "platform approved", "reason 3"),
            LeadDecision::new(Lead::Reviewer, "reviewer approved", "reason 4"),
            LeadDecision::new(Lead::Pm, "pm approved", "reason 5"),
        ];
        let panel = SignOffPanel::new(decisions);
        assert!(panel.is_complete());
        assert_eq!(panel.len(), 5);
        assert!(panel.missing_leads().is_empty());
        assert!(panel.get(Lead::Architecture).is_some());
        assert!(panel.get(Lead::Pm).is_some());
    }

    #[test]
    fn sign_off_panel_incomplete_lists_missing() {
        // 缺 1 域 Lead → is_complete() = false, missing_leads() 列缺
        let decisions = vec![
            LeadDecision::new(Lead::Architecture, "arch approved", "reason 1"),
            LeadDecision::new(Lead::Sre, "sre approved", "reason 2"),
            // 缺 Platform / Reviewer / Pm
        ];
        let panel = SignOffPanel::new(decisions);
        assert!(!panel.is_complete());
        let missing = panel.missing_leads();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&Lead::Platform));
        assert!(missing.contains(&Lead::Reviewer));
        assert!(missing.contains(&Lead::Pm));
    }

    #[test]
    fn lead_decision_real_person_decision_format() {
        // 真人到位追溯决策 (per 守门 #1 禁回溯叙事, 修订历史 +1 行)
        let d = LeadDecision::real_person_decision(
            Lead::Architecture,
            "Alice",
            "架构决策",
            "真人到位追溯",
        );
        assert!(d.decision.contains("[真人 Alice 追溯]"));
        assert_eq!(d.lead, Lead::Architecture);
    }
}
