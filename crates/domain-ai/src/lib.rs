//! Star AI Engine (精简实装 v0.1)
//!
//! 3 大 Rovo-like Agent:
//! 1. Workflow Builder (自然语言 → Workflow JSON)
//! 2. Work Readiness Checker (工作项完整性检查)
//! 3. Report Insight (报告 → 自然语言洞察)
//!
//! + JQL AI (自然语言 → JQL)
//! + Rovo-like Chat (跨域问答 + SSE 流式)
//! + LLM Provider 抽象 (mock + openai/anthropic 接口)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object
// =====================================================================

/// LLM Provider 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Mock,    // 内置 mock (无外部 API)
    OpenAI,  // 接口预留
    Anthropic,
}

impl LlmProvider {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mock => "Mock",
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
        }
    }
}

/// 模型配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: LlmProvider,
    pub model_name: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Mock,
            model_name: "mock-1".into(),
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

/// Agent 角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    WorkflowBuilder,
    WorkReadinessChecker,
    ReportInsight,
    JqlGenerator,
    RovoChat,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            Self::WorkflowBuilder => "Workflow Builder",
            Self::WorkReadinessChecker => "Work Readiness Checker",
            Self::ReportInsight => "Report Insight",
            Self::JqlGenerator => "JQL Generator",
            Self::RovoChat => "Rovo Chat",
        }
    }
}

// =====================================================================
// 2. entity
// =====================================================================

/// AI 请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiRequest {
    pub id: Uuid,
    pub role: AgentRole,
    pub prompt: String,
    pub context: serde_json::Value,
    pub model_config: ModelConfig,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// AI 响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiResponse {
    pub request_id: Uuid,
    pub role: AgentRole,
    pub content: String,
    pub structured: Option<serde_json::Value>, // 结构化输出 (e.g. Workflow JSON)
    pub tokens_used: u32,
    pub latency_ms: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Prompt 模板
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: Uuid,
    pub role: AgentRole,
    pub name: String,
    pub template: String, // 含 {{var}} 占位
    pub version: u32,
}

impl PromptTemplate {
    pub fn render(&self, vars: &std::collections::HashMap<String, String>) -> String {
        let mut out = self.template.clone();
        for (k, v) in vars {
            out = out.replace(&format!("{{{{{}}}}}", k), v);
        }
        out
    }
}

// =====================================================================
// 3. port
// =====================================================================

#[async_trait]
pub trait LlmProviderPort: Send + Sync {
    async fn complete(&self, request: &AiRequest) -> Result<AiResponse, AiError>;
    async fn stream(&self, request: &AiRequest) -> Result<tokio::sync::mpsc::Receiver<String>, AiError>;
}

// =====================================================================
// 4. service — 3 Agent + JQL AI + Rovo Chat
// =====================================================================

pub struct AiService {
    provider: Box<dyn LlmProviderPort>,
}

impl AiService {
    pub fn new(provider: Box<dyn LlmProviderPort>) -> Self {
        Self { provider }
    }

    /// Workflow Builder: 自然语言 → Workflow JSON
    pub async fn build_workflow(&self, description: &str) -> Result<AiResponse, AiError> {
        let req = AiRequest {
            id: Uuid::new_v4(),
            role: AgentRole::WorkflowBuilder,
            prompt: description.into(),
            context: serde_json::json!({}),
            model_config: ModelConfig::default(),
            user_id: None,
            tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        self.provider.complete(&req).await
    }

    /// Work Readiness Checker: 工作项 → 0-100 分 + 改进建议
    pub async fn check_readiness(
        &self,
        work_item: &serde_json::Value,
    ) -> Result<AiResponse, AiError> {
        let req = AiRequest {
            id: Uuid::new_v4(),
            role: AgentRole::WorkReadinessChecker,
            prompt: "检查工作项完整性".into(),
            context: work_item.clone(),
            model_config: ModelConfig::default(),
            user_id: None,
            tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        self.provider.complete(&req).await
    }

    /// Report Insight: 报告 → 自然语言洞察
    pub async fn report_insight(&self, report: &serde_json::Value) -> Result<AiResponse, AiError> {
        let req = AiRequest {
            id: Uuid::new_v4(),
            role: AgentRole::ReportInsight,
            prompt: "分析报告并给出风险点 + 改进建议".into(),
            context: report.clone(),
            model_config: ModelConfig::default(),
            user_id: None,
            tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        self.provider.complete(&req).await
    }

    /// JQL AI: 自然语言 → JQL
    pub async fn jql_from_natural(&self, natural: &str) -> Result<AiResponse, AiError> {
        let req = AiRequest {
            id: Uuid::new_v4(),
            role: AgentRole::JqlGenerator,
            prompt: format!("Convert to JQL: {}", natural),
            context: serde_json::json!({}),
            model_config: ModelConfig::default(),
            user_id: None,
            tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        self.provider.complete(&req).await
    }

    /// Rovo Chat: 跨域问答
    pub async fn rovo_chat(&self, question: &str, context: &serde_json::Value) -> Result<AiResponse, AiError> {
        let req = AiRequest {
            id: Uuid::new_v4(),
            role: AgentRole::RovoChat,
            prompt: question.into(),
            context: context.clone(),
            model_config: ModelConfig::default(),
            user_id: None,
            tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        self.provider.complete(&req).await
    }
}

// =====================================================================
// 5. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AiError {
    #[error("LLM provider error: {0}")]
    Provider(String),
    #[error("rate limited")]
    RateLimited,
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("data isolation violation: 客户数据不得参与训练")]
    DataIsolation,
}

// =====================================================================
// 6. Mock LLM Provider (默认, 无外部 API 依赖)
// =====================================================================

pub struct MockLlmProvider;

#[async_trait]
impl LlmProviderPort for MockLlmProvider {
    async fn complete(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        // 模拟响应 (per role 走不同 stub)
        let content = match request.role {
            AgentRole::WorkflowBuilder => mock_workflow(&request.prompt),
            AgentRole::WorkReadinessChecker => mock_readiness(&request.context),
            AgentRole::ReportInsight => mock_insight(&request.context),
            AgentRole::JqlGenerator => mock_jql(&request.prompt),
            AgentRole::RovoChat => mock_chat(&request.prompt),
        };
        Ok(AiResponse {
            request_id: request.id,
            role: request.role,
            content,
            structured: None,
            tokens_used: content.len() as u32 / 4,
            latency_ms: 50,
            created_at: chrono::Utc::now(),
        })
    }

    async fn stream(&self, request: &AiRequest) -> Result<tokio::sync::mpsc::Receiver<String>, AiError> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let content = match request.role {
            AgentRole::RovoChat => format!("[mock] 收到: {}", request.prompt),
            _ => format!("[mock] role={:?}", request.role),
        };
        tokio::spawn(async move {
            for chunk in content.split_whitespace() {
                let _ = tx.send(format!("{} ", chunk)).await;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        Ok(rx)
    }
}

fn mock_workflow(desc: &str) -> String {
    serde_json::json!({
        "name": "AI-generated workflow",
        "description": desc,
        "statuses": [
            {"id": "todo", "name": "To Do"},
            {"id": "doing", "name": "In Progress"},
            {"id": "done", "name": "Done"}
        ],
        "transitions": [
            {"from": "todo", "to": "doing"},
            {"from": "doing", "to": "done"}
        ]
    }).to_string()
}

fn mock_readiness(ctx: &serde_json::Value) -> String {
    let score = if ctx.get("description").is_some() { 75 } else { 45 };
    format!("工作项就绪度评分: {}/100. 缺验收标准 / 子任务 / 关联.", score)
}

fn mock_insight(_ctx: &serde_json::Value) -> String {
    "本 Sprint 速度下降 12%, 风险点: 1 个高优工作项已 5 天未更新. 建议立即 review 并拆分.".to_string()
}

fn mock_jql(prompt: &str) -> String {
    if prompt.contains("我") {
        "assignee = currentUser() AND status != Done".to_string()
    } else if prompt.contains("高") {
        "priority = High ORDER BY created DESC".to_string()
    } else {
        "status = Open".to_string()
    }
}

fn mock_chat(prompt: &str) -> String {
    format!("根据 Star 数据, 关于 '{}' 的回答: [mock] 涉及 3 个工作项, 1 个项目, 2 个评论.", prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_name() {
        assert_eq!(LlmProvider::Mock.name(), "Mock");
        assert_eq!(LlmProvider::OpenAI.name(), "OpenAI");
    }

    #[tokio::test]
    async fn test_agent_role_name() {
        assert_eq!(AgentRole::WorkflowBuilder.name(), "Workflow Builder");
        assert_eq!(AgentRole::RovoChat.name(), "Rovo Chat");
    }

    #[tokio::test]
    async fn test_workflow_builder() {
        let svc = AiService::new(Box::new(MockLlmProvider));
        let r = svc.build_workflow("订单状态: 待支付 → 支付中 → 已支付 → 已发货").await.unwrap();
        assert!(r.content.contains("statuses"));
        assert!(r.content.contains("transitions"));
    }

    #[tokio::test]
    async fn test_work_readiness_checker() {
        let svc = AiService::new(Box::new(MockLlmProvider));
        let item = serde_json::json!({"title": "X", "description": "Y"});
        let r = svc.check_readiness(&item).await.unwrap();
        assert!(r.content.contains("就绪度评分"));
    }

    #[tokio::test]
    async fn test_jql_generator() {
        let svc = AiService::new(Box::new(MockLlmProvider));
        let r = svc.jql_from_natural("我负责的未完成").await.unwrap();
        assert!(r.content.contains("currentUser"));
    }

    #[tokio::test]
    async fn test_rovo_chat() {
        let svc = AiService::new(Box::new(MockLlmProvider));
        let r = svc.rovo_chat("本周高优工作项", &serde_json::json!({})).await.unwrap();
        assert!(r.content.contains("mock"));
    }

    #[tokio::test]
    async fn test_stream() {
        let provider = MockLlmProvider;
        let req = AiRequest {
            id: Uuid::new_v4(), role: AgentRole::RovoChat, prompt: "test".into(),
            context: serde_json::json!({}), model_config: ModelConfig::default(),
            user_id: None, tenant_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };
        let mut rx = provider.stream(&req).await.unwrap();
        let mut got = String::new();
        while let Some(s) = rx.recv().await {
            got.push_str(&s);
        }
        assert!(got.contains("mock"));
    }

    #[test]
    fn test_prompt_template_render() {
        let t = PromptTemplate {
            id: Uuid::new_v4(),
            role: AgentRole::JqlGenerator,
            name: "test".into(),
            template: "Convert {{natural}} to JQL".into(),
            version: 1,
        };
        let mut vars = std::collections::HashMap::new();
        vars.insert("natural".to_string(), "my open issues".to_string());
        assert_eq!(t.render(&vars), "Convert my open issues to JQL");
    }
}
