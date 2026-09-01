//! Star Hermes HTTP API 客户端 (B.2 实装, per 2026-09-01 22:30 JST wt-wbs-b2-hermes-mock)
//!
//! 职责:
//!   1. **HermesClient** (service 层): 5 endpoint HTTP 客户端 (auth / query / submit / status / cancel)
//!   2. **Task / AuthToken / CancelResponse** (entity 层): 请求/响应 schema
//!   3. **HermesConfig / RetryPolicy** (value_object 层): 客户端配置 + 重试策略
//!   4. **HermesError** (error 层): 4 变体错误模型 (Http / Auth / ServerError / Parse)
//!
//! **与 B.1 OpenClaw / B.6 Hermes 真实集成的差异** (per 10:58 JST 拍板):
//!   - **5 endpoint** (B.1 单 endpoint /chat/completions): auth / query / submit / status / cancel
//!   - **4 层精简** (B.1 单文件): mod / entity / value_object / error / service
//!   - **重试策略**: 3 变体 (NoRetry / FixedDelay / ExponentialBackoff)
//!   - **错误模型**: 4 变体 vs B.1 的 4 变体 (Http / Auth / ServerError / Parse, **新增** Auth/ServerError 区分 transient/permanent)
//!
//! **mock 备选** (per 29692a7 拍板):
//!   - `HermesConfig::new_mock()`: 默认 mock 模式, base_url = "http://localhost:8082/v1", 不发 HTTP
//!   - `HermesConfig::new_real()`: 真实模式, base_url + api_key 必填, 凭证到位后 1 commit 替换
//!   - contract test 走 `wiremock` mock server (per `docs/frontend/design/mock-msw-handlers.md` 既有模式)
//!
//! 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//!   1. 真实 Hermes endpoint 未接, 当前用 mock base_url (per 29692a7 mock 备选)
//!   2. 不接 KMS, API key 明文在 HermesConfig (per E.4 KMS 集成凭证到位后)
//!   3. 不接 B.7 retry_with_backoff, 当前 HermesConfig.retry_policy 简化版 (3 变体, 不带抖动)
//!   4. 不接 B.9 监控审计, 调用不进 ApiMonitor (per B.9 phase 续接)
//!   5. B.6 hermes_client.rs (chat completions) 与本 B.2 (task queue) 是不同 API, 5 endpoint 不重叠
//!
//! 不做 (per 守门):
//!   - 不动 network 层 (per Phase D)
//!   - 不写 UI (per B.9 API 监控审计)
//!   - 不动 B.1 OpenClaw / B.6 hermes_client.rs (独立子项)

pub mod entity;
pub mod error;
pub mod service;
pub mod value_object;

// Re-export public API surface (per 4 层精简 → 上层只看 mod.rs re-export)
pub use entity::{AuthToken, CancelResponse, QueryRequest, Task, TaskStatus};
pub use error::HermesError;
pub use service::{HermesClient, HermesClientBuilder, SubmitRequest};
pub use value_object::{HermesConfig, HermesMode, RetryPolicy};
