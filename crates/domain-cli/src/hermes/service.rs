//! Hermes 服务层 (B.2 4 层精简: service 层)
//!
//! 含:
//!   1. **HermesClient**: HTTP 客户端 (5 endpoint: auth / query / submit / status / cancel)
//!   2. **HermesClientBuilder**: 流式 builder (for tests / dynamic config)
//!
//! 5 endpoint (per Hermes task queue spec):
//!   - **auth**    POST   /v1/auth/token       → AuthToken
//!   - **query**   GET    /v1/tasks?status=... → Vec<Task>
//!   - **submit**  POST   /v1/tasks            → Task (status=Pending)
//!   - **status**  GET    /v1/tasks/{id}       → Task (updated status)
//!   - **cancel**  DELETE /v1/tasks/{id}       → CancelResponse
//!
//! mock 模式 (per 29692a7 mock 备选):
//!   - 5 endpoint 全部走 mock_response_*, 不发 HTTP
//!   - mock 响应跟真实 spec 一致 (per contract test 验证)
//!
//! 真实模式:
//!   - 发 HTTP reqwest POST / GET / DELETE
//!   - Bearer auth (api_key)
//!   - 错误映射: reqwest::Error → HermesError::Http, non-2xx → classify_status
//!
//! 重试策略 (B.2 RetryPolicy 简化版):
//!   - phase 2: 套 retry_with_backoff (B.7) wrapper, 当前仅 next_retry_delay helper
//!   - 当前实装: 单次 attempt, 不实际重试 (per §3 #3 已知缺口)

use super::entity::{AuthToken, CancelResponse, QueryRequest, Task};
use super::error::{classify_status, HermesError};
use super::value_object::{HermesConfig, HermesMode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// POST /v1/tasks 请求体 (submit endpoint, 跟 B.1 OpenClaw GenerateRequest 不同)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitRequest {
    /// 任务名称
    pub name: String,
    /// 任务优先级
    pub priority: u8,
    /// 任务载荷(序列化后的字符串)
    pub payload: String,
}

impl SubmitRequest {
    /// 构造提交请求
    pub fn new(name: impl Into<String>, priority: u8, payload: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority,
            payload: payload.into(),
        }
    }
}

/// Hermes HTTP 客户端 (B.2 service 层)
pub struct HermesClient {
    config: HermesConfig,
    http: reqwest::Client,
}

impl HermesClient {
    /// 新建客户端 (per config)
    pub fn new(config: HermesConfig) -> Result<Self, HermesError> {
        // Real 模式: base_url + api_key 必填 (config 构造时已校验, 这里冗余防御)
        if matches!(config.mode, HermesMode::Real) {
            if config.base_url.is_empty() {
                return Err(HermesError::Auth("base_url is empty".into()));
            }
            if config.api_key.is_empty() {
                return Err(HermesError::Auth("api_key is empty".into()));
            }
        }
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { config, http })
    }

    /// 当前 config (ref)
    pub fn config(&self) -> &HermesConfig {
        &self.config
    }

    // =====================================================================
    // 5 endpoint 1: auth (POST /v1/auth/token)
    // =====================================================================

    /// POST /v1/auth/token
    ///
    /// mock 模式: 直接返回 mock AuthToken
    /// 真实模式: POST base_url + "/auth/token" + Bearer auth
    pub async fn auth(&self) -> Result<AuthToken, HermesError> {
        if matches!(self.config.mode, HermesMode::Mock) {
            return Ok(Self::mock_auth());
        }

        let url = format!("{}/auth/token", self.config.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_status(status.as_u16(), body));
        }

        let parsed: AuthToken = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    // =====================================================================
    // 5 endpoint 2: query (GET /v1/tasks?status=...&priority=...&limit=...)
    // =====================================================================

    /// GET /v1/tasks
    ///
    /// mock 模式: 返回 mock Vec<Task> (1 个 pending + 1 个 running)
    /// 真实模式: GET base_url + "/tasks" + query params
    pub async fn query(&self, req: &QueryRequest) -> Result<Vec<Task>, HermesError> {
        if matches!(self.config.mode, HermesMode::Mock) {
            return Ok(Self::mock_query(req));
        }

        let url = format!("{}/tasks", self.config.base_url);
        let mut request = self.http.get(&url).bearer_auth(&self.config.api_key);

        if let Some(status) = req.status {
            request = request.query(&[("status", status.as_str())]);
        }
        if let Some(priority) = req.priority {
            request = request.query(&[("priority", priority.to_string())]);
        }
        if let Some(limit) = req.limit {
            request = request.query(&[("limit", limit.to_string())]);
        }
        if let Some(created_after) = req.created_after {
            request = request.query(&[("created_after", created_after.to_rfc3339())]);
        }

        let resp = request.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_status(status.as_u16(), body));
        }

        let parsed: Vec<Task> = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    // =====================================================================
    // 5 endpoint 3: submit (POST /v1/tasks)
    // =====================================================================

    /// POST /v1/tasks
    ///
    /// mock 模式: 返回 mock Task (status=Pending)
    /// 真实模式: POST base_url + "/tasks" + JSON body
    pub async fn submit(&self, req: &SubmitRequest) -> Result<Task, HermesError> {
        if matches!(self.config.mode, HermesMode::Mock) {
            return Ok(Self::mock_submit(req));
        }

        let url = format!("{}/tasks", self.config.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_status(status.as_u16(), body));
        }

        let parsed: Task = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    // =====================================================================
    // 5 endpoint 4: status (GET /v1/tasks/{id})
    // =====================================================================

    /// GET /v1/tasks/{id}
    ///
    /// mock 模式: 返回 mock Task (status=Running)
    /// 真实模式: GET base_url + "/tasks/{id}"
    pub async fn status(&self, id: Uuid) -> Result<Task, HermesError> {
        if matches!(self.config.mode, HermesMode::Mock) {
            return Ok(Self::mock_status(id));
        }

        let url = format!("{}/tasks/{}", self.config.base_url, id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_status(status.as_u16(), body));
        }

        let parsed: Task = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    // =====================================================================
    // 5 endpoint 5: cancel (DELETE /v1/tasks/{id})
    // =====================================================================

    /// DELETE /v1/tasks/{id}
    ///
    /// mock 模式: 返回 mock CancelResponse (cancelled=true)
    /// 真实模式: DELETE base_url + "/tasks/{id}"
    pub async fn cancel(&self, id: Uuid) -> Result<CancelResponse, HermesError> {
        if matches!(self.config.mode, HermesMode::Mock) {
            return Ok(Self::mock_cancel(id));
        }

        let url = format!("{}/tasks/{}", self.config.base_url, id);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(classify_status(status.as_u16(), body));
        }

        let parsed: CancelResponse = resp
            .json()
            .await
            .map_err(|e| HermesError::Parse(e.to_string()))?;
        Ok(parsed)
    }

    // =====================================================================
    // Mock 响应 (per 29692a7 mock 备选, 不发 HTTP, 跟真实 spec 同 shape)
    // =====================================================================

    /// Mock auth response
    fn mock_auth() -> AuthToken {
        use chrono::Utc;
        AuthToken {
            access_token: format!("mock-token-{}", Uuid::new_v4()),
            token_type: "Bearer".into(),
            expires_at: Utc::now() + chrono::Duration::seconds(3600),
        }
    }

    /// Mock query response (返回 2 个 task: 1 pending + 1 running)
    fn mock_query(_req: &QueryRequest) -> Vec<Task> {
        use chrono::Utc;
        vec![
            Task {
                id: Uuid::new_v4(),
                name: "build-package".into(),
                status: super::entity::TaskStatus::Pending,
                priority: 3,
                payload: r#"{"input":"mock-payload-1"}"#.into(),
                created_at: Utc::now(),
                updated_at: None,
                result: None,
            },
            Task {
                id: Uuid::new_v4(),
                name: "run-tests".into(),
                status: super::entity::TaskStatus::Running,
                priority: 5,
                payload: r#"{"input":"mock-payload-2"}"#.into(),
                created_at: Utc::now(),
                updated_at: Some(Utc::now()),
                result: None,
            },
        ]
    }

    /// Mock submit response
    fn mock_submit(req: &SubmitRequest) -> Task {
        use chrono::Utc;
        Task {
            id: Uuid::new_v4(),
            name: req.name.clone(),
            status: super::entity::TaskStatus::Pending,
            priority: req.priority,
            payload: req.payload.clone(),
            created_at: Utc::now(),
            updated_at: None,
            result: None,
        }
    }

    /// Mock status response
    fn mock_status(id: Uuid) -> Task {
        use chrono::Utc;
        Task {
            id,
            name: "mock-task".into(),
            status: super::entity::TaskStatus::Running,
            priority: 3,
            payload: r#"{"input":"mock"}"#.into(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            result: None,
        }
    }

    /// Mock cancel response
    fn mock_cancel(id: Uuid) -> CancelResponse {
        use chrono::Utc;
        CancelResponse {
            cancelled: true,
            cancelled_at: Some(Utc::now()),
            current_status: super::entity::TaskStatus::Cancelled,
        }
    }
}

/// 流式 builder (for tests / dynamic config)
pub struct HermesClientBuilder {
    config: HermesConfig,
}

impl HermesClientBuilder {
    /// 新建 builder (从 HermesConfig 起)
    pub fn new(config: HermesConfig) -> Self {
        Self { config }
    }

    /// Mock 模式 builder (shorthand)
    pub fn mock() -> Self {
        Self::new(HermesConfig::new_mock())
    }

    /// 设置 base_url
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    /// 设置 api_key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = api_key.into();
        self
    }

    /// 设置 timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// 设置 mode
    pub fn mode(mut self, mode: HermesMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// build HermesClient
    pub fn build(self) -> Result<HermesClient, HermesError> {
        HermesClient::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hermes::entity::TaskStatus;

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Runtime::new().unwrap()
    }

    #[test]
    fn hermes_client_mock_mode_construction() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        assert_eq!(client.config().mode, HermesMode::Mock);
    }

    #[test]
    fn hermes_client_real_mode_construction() {
        let cfg = HermesConfig::new_real("https://api.hermes.dev/v1", "test-key").unwrap();
        let client = HermesClient::new(cfg).unwrap();
        assert_eq!(client.config().mode, HermesMode::Real);
    }

    #[test]
    fn hermes_client_real_mode_rejects_empty_url() {
        let cfg = HermesConfig {
            base_url: "".into(),
            api_key: "test-key".into(),
            timeout: Duration::from_secs(1),
            mode: HermesMode::Real,
            retry_policy: super::super::value_object::RetryPolicy::NoRetry,
        };
        let r = HermesClient::new(cfg);
        assert!(matches!(r, Err(HermesError::Auth(_))));
    }

    #[test]
    fn hermes_client_real_mode_rejects_empty_key() {
        let cfg = HermesConfig {
            base_url: "https://api.hermes.dev/v1".into(),
            api_key: "".into(),
            timeout: Duration::from_secs(1),
            mode: HermesMode::Real,
            retry_policy: super::super::value_object::RetryPolicy::NoRetry,
        };
        let r = HermesClient::new(cfg);
        assert!(matches!(r, Err(HermesError::Auth(_))));
    }

    #[test]
    fn mock_auth_returns_token() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let rt = runtime();
        let token = rt.block_on(client.auth()).unwrap();
        assert!(token.access_token.starts_with("mock-token-"));
        assert_eq!(token.token_type, "Bearer");
    }

    #[test]
    fn mock_query_returns_two_tasks() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let rt = runtime();
        let tasks = rt.block_on(client.query(&QueryRequest::default())).unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.status == TaskStatus::Pending));
        assert!(tasks.iter().any(|t| t.status == TaskStatus::Running));
    }

    #[test]
    fn mock_submit_returns_pending_task() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let rt = runtime();
        let req = SubmitRequest::new("build", 3, r#"{"input":"x"}"#);
        let task = rt.block_on(client.submit(&req)).unwrap();
        assert_eq!(task.name, "build");
        assert_eq!(task.priority, 3);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn mock_status_returns_running_task() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let rt = runtime();
        let id = Uuid::new_v4();
        let task = rt.block_on(client.status(id)).unwrap();
        assert_eq!(task.id, id);
        assert_eq!(task.status, TaskStatus::Running);
    }

    #[test]
    fn mock_cancel_returns_cancelled_response() {
        let cfg = HermesConfig::new_mock();
        let client = HermesClient::new(cfg).unwrap();
        let rt = runtime();
        let id = Uuid::new_v4();
        let resp = rt.block_on(client.cancel(id)).unwrap();
        assert!(resp.cancelled);
        assert_eq!(resp.current_status, TaskStatus::Cancelled);
        assert!(resp.cancelled_at.is_some());
    }

    #[test]
    fn hermes_client_builder_mock() {
        let client = HermesClientBuilder::mock()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        assert_eq!(client.config().mode, HermesMode::Mock);
        assert_eq!(client.config().timeout, Duration::from_secs(60));
    }

    #[test]
    fn hermes_client_builder_real() {
        let client = HermesClientBuilder::new(HermesConfig::new_mock())
            .mode(HermesMode::Real)
            .base_url("https://api.hermes.dev/v1")
            .api_key("test-key")
            .build()
            .unwrap();
        assert_eq!(client.config().mode, HermesMode::Real);
        assert_eq!(client.config().base_url, "https://api.hermes.dev/v1");
        assert_eq!(client.config().api_key, "test-key");
    }

    #[test]
    fn submit_request_serde_roundtrip() {
        let req = SubmitRequest::new("build", 3, "{}");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"name\":\"build\""));
        assert!(json.contains("\"priority\":3"));
        let back: SubmitRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.name, back.name);
        assert_eq!(req.priority, back.priority);
        assert_eq!(req.payload, back.payload);
    }
}
