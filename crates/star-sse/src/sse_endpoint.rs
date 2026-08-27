// SPDX-License-Identifier: MIT OR Apache-2.0
//! SseEndpoint — SSE HTTP handler 框架 (per spec/services/02 §4)
//!
//! Phase F 阶段：提供鉴权 + heartbeat 30s 框架；
//! Phase F+ 接 axum/hyper SSE 完整实现 (chunked transfer + 鉴权中间件)。

use super::{EventRouter, SseError};

/// SSE 端点
///
/// 持有一个 EventRouter 引用,在 `handle_connect` 中执行鉴权后开始推送循环。
/// Phase F stub 不实际写 socket,仅保留 API 形状 + 鉴权/heartbeat 常量。
pub struct SseEndpoint {
    /// 共享的事件路由器
    pub router: EventRouter,
}

impl SseEndpoint {
    /// 创建新端点
    pub fn new(router: EventRouter) -> Self {
        Self { router }
    }

    /// 处理 SSE 连接 (Phase F stub)
    ///
    /// 鉴权 token:Phase F+ 接入 OIDC/JWT,当前仅检查非空。
    /// 真实实现应启动 chunked response + 事件循环 + heartbeat timer。
    pub async fn handle_connect(&self, auth_token: &str) -> Result<(), SseError> {
        if auth_token.is_empty() {
            return Err(SseError::Auth("missing".into()));
        }
        // Phase F+: 这里启动 SSE chunked response + tokio::select! 推送循环
        Ok(())
    }

    /// heartbeat 间隔 (秒) — per spec/services/02 §4
    pub async fn heartbeat_interval_sec() -> u64 {
        30
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_rejects_empty_token() {
        let e = SseEndpoint::new(EventRouter::new());
        let r = e.handle_connect("").await;
        assert!(matches!(r, Err(SseError::Auth(_))));
    }

    #[tokio::test]
    async fn handle_accepts_token() {
        let e = SseEndpoint::new(EventRouter::new());
        assert!(e.handle_connect("valid-token").await.is_ok());
    }

    #[tokio::test]
    async fn heartbeat_is_30s() {
        assert_eq!(SseEndpoint::heartbeat_interval_sec().await, 30);
    }

    #[tokio::test]
    async fn shares_router() {
        let r = EventRouter::new();
        let e = SseEndpoint::new(r.clone());
        // 同一 router 引用,可双向访问
        e.router
            .publish(crate::Event {
                id: String::new(),
                event_type: crate::EventType::Pipeline,
                source: "t".into(),
                timestamp: 0,
                data: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(r.len().await, 1);
    }
}
