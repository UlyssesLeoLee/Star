//! 第三方平台 Adapter 抽象与具体实现
//!
//! **crate**: `domain-integration::adapter`
//!
//! **职责**: 在 `Integration` 实体基础上,提供各类第三方平台(Confluence / Slack / Teams
//! / Webhook)的具体适配实现。每个 adapter 暴露统一的 `Adapter` trait,内部封装:
//!
//! - OAuth2 / Bot Token 授权(走 domain-identity 的 credential broker)
//! - 拉取 / 推送 API 调用(此处用 trait mock 即可,真实 HTTP 在 infrastructure 层)
//! - 双向链接 / 嵌入 macro / 通知 / slash command 解析
//!
//! **Phase W15 范围**(per task brief):
//! - Confluence Adapter(拉 space / page + 双向链接 + 嵌入 macro)
//! - Slack Adapter(OAuth / Bot Token / 通知 / slash command / 线程双向)
//! - MS Teams Adapter(同 Slack,适配 Adaptive Card)
//!
//! **设计原则**(per 8/26 AI 协作文档治理):
//! - 不在 domain 层发真实 HTTP 调用(那是 infrastructure crate 的职责)
//! - 统一 trait,内部用 mock HTTP client 可注入测试
//! - 凭据只引 `CredentialRefId`,不存明文(沿用 INV-I-04)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// Adapter 错误
// =====================================================================

/// **Adapter 调用错误**
#[derive(Debug, Error)]
pub enum AdapterError {
    /// 未授权(缺 token / OAuth2 未完成)
    #[error("not authorized: {0}")]
    NotAuthorized(String),
    /// 凭据无效 / 过期
    #[error("invalid credential: {0}")]
    InvalidCredential(String),
    /// HTTP 错误(伪,Phase W15 mock)
    #[error("http error: {0}")]
    Http(String),
    /// 解析错误
    #[error("parse error: {0}")]
    Parse(String),
    /// 平台限流
    #[error("rate limited: retry_after_ms={0}")]
    RateLimited(u64),
    /// 资源未找到
    #[error("not found: {0}")]
    NotFound(String),
    /// 通用错误
    #[error("internal: {0}")]
    Internal(String),
}

/// **Adapter Result** 别名
pub type AdapterResult<T> = Result<T, AdapterError>;

// =====================================================================
// 共享值对象
// =====================================================================

/// **凭据引用**(指向 domain-identity 的 `CredentialRefId`,永远不存明文)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct CredentialRefId(pub Uuid);

impl CredentialRefId {
    /// 构造
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    /// 内部 uuid
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CredentialRefId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CredentialRefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// **OAuth2 / Bot Token**(Adapter 内部承载,仅在 adapter scope 内)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌(可选,Slack bot token 无需 refresh)
    pub refresh_token: Option<String>,
    /// 过期时间(UTC)
    pub expires_at: Option<DateTime<Utc>>,
    /// 授权范围(空格分隔)
    pub scope: String,
    /// 关联凭据引用
    pub credential_ref: CredentialRefId,
}

impl AuthToken {
    /// 是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            exp <= Utc::now()
        } else {
            false // 永不过期(如 Slack bot token)
        }
    }
}

/// **Adapter 能力位**(feature flag,标识 adapter 提供的能力)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdapterCapability {
    /// 拉取外部实体列表
    ListExternalEntities,
    /// 推送 Star 工作项到外部
    PushWorkItem,
    /// 接收外部事件(Webhook / Socket Mode)
    ReceiveExternalEvents,
    /// 双向链接
    BidirectionalLink,
    /// 嵌入式 macro / 卡片
    EmbedCard,
    /// Slash command 解析
    SlashCommand,
    /// 线程对话
    ThreadedComment,
    /// Adaptive Card(MS Teams)
    AdaptiveCard,
}

// =====================================================================
// HTTP 客户端抽象(可注入 mock)
// =====================================================================

/// **HTTP 客户端 trait**(供 adapter 发送 HTTP 请求,实现可以是 reqwest / mock)
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// GET 请求,返回状态码 + body
    async fn get(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> AdapterResult<(u16, String)>;
    /// POST 请求,JSON body
    async fn post(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: String,
    ) -> AdapterResult<(u16, String)>;
    /// PUT 请求
    async fn put(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: String,
    ) -> AdapterResult<(u16, String)>;
    /// DELETE 请求
    async fn delete(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> AdapterResult<(u16, String)>;
}

// =====================================================================
// 通用 Adapter 抽象
// =====================================================================

/// **统一 Adapter trait**(所有具体 adapter 必须实现)
#[async_trait]
pub trait Adapter: Send + Sync {
    /// adapter 类型
    fn kind(&self) -> &'static str;
    /// 能力位
    fn capabilities(&self) -> Vec<AdapterCapability>;
    /// 是否已授权
    fn is_authorized(&self) -> bool;
    /// 当前 token
    fn current_token(&self) -> Option<&AuthToken>;
    /// 注入 token(OAuth2 callback / Bot token 注入)
    fn set_token(&mut self, token: AuthToken);
    /// 清除 token(注销授权)
    fn clear_token(&mut self);
    /// 适配的 HTTP client(供测试 mock 注入)
    fn http(&self) -> &dyn HttpClient;
}

// =====================================================================
// OAuth2 PKCE 元数据
// =====================================================================

/// **OAuth2 授权请求参数**(Phase W15 仅提供参数模型,真实授权流在 infrastructure 层)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AuthRequest {
    /// 授权 endpoint
    pub auth_endpoint: String,
    /// client_id
    pub client_id: String,
    /// redirect_uri
    pub redirect_uri: String,
    /// scope
    pub scope: Vec<String>,
    /// state(防 CSRF)
    pub state: String,
    /// PKCE code_challenge(S256)
    pub code_challenge: String,
    /// PKCE method
    pub code_challenge_method: String,
}

/// **OAuth2 回调参数**(用户授权后由 callback endpoint 接收)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Callback {
    /// 授权码
    pub code: String,
    /// state(必须与请求一致)
    pub state: String,
    /// 关联的 credential_ref
    pub credential_ref: CredentialRefId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_ref_id_constructs() {
        let c = CredentialRefId::new();
        assert!(!c.as_uuid().is_nil());
    }

    #[test]
    fn auth_token_is_expired() {
        // 永不过期
        let t = AuthToken {
            access_token: "x".into(),
            refresh_token: None,
            expires_at: None,
            scope: "".into(),
            credential_ref: CredentialRefId::new(),
        };
        assert!(!t.is_expired());

        // 已过期
        let past = Utc::now() - chrono::Duration::seconds(60);
        let t2 = AuthToken {
            expires_at: Some(past),
            ..t.clone()
        };
        assert!(t2.is_expired());

        // 未来
        let future = Utc::now() + chrono::Duration::seconds(3600);
        let t3 = AuthToken {
            expires_at: Some(future),
            ..t
        };
        assert!(!t3.is_expired());
    }
}
