//! crates/star-dispatcher/src/cross_repo_tests.rs
//!
//! H.2 LangGraph 跨仓 RPC (Star -> Physis) PoC e2e tests
//! per 守门 #19 [M] 拍板

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::cross_repo::{
        CrossRepoClient, DispatchTaskRequest, HealthCheckRequest, PhysisServer, PhysisTaskState,
        QueryStateRequest,
    };

    /// H.2 test 1: DispatchTask 跨仓派发 OK
    #[tokio::test]
    async fn h2_dispatch_task_cross_repo_ok() {
        let (server, _rx) = PhysisServer::new();
        let server = Arc::new(server);
        let client = CrossRepoClient::new(server.clone());

        let req = DispatchTaskRequest {
            task_id: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            kind: "physics_sim".into(),
            payload: serde_json::json!({"dt": 0.016}),
            idempotency_key: format!("idem-{}", Uuid::new_v4()),
        };
        let resp = client.dispatch_task(req).await.unwrap();
        assert!(resp.accepted);
        assert!(resp.task_id.starts_with("physis-"));
    }

    /// H.2 test 2: Idempotency 跨仓去重
    #[tokio::test]
    async fn h2_dispatch_idempotency_dedup() {
        let (server, _rx) = PhysisServer::new();
        let server = Arc::new(server);
        let client = CrossRepoClient::new(server.clone());

        let idem = format!("idem-{}", Uuid::new_v4());
        let req1 = DispatchTaskRequest {
            task_id: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            kind: "physics_sim".into(),
            payload: serde_json::json!({}),
            idempotency_key: idem.clone(),
        };
        let resp1 = client.dispatch_task(req1).await.unwrap();
        assert!(resp1.accepted);

        // 同一 idempotency_key 重复派发应被去重
        let req2 = DispatchTaskRequest {
            task_id: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            kind: "physics_sim".into(),
            payload: serde_json::json!({}),
            idempotency_key: idem.clone(),
        };
        let resp2 = client.dispatch_task(req2).await.unwrap();
        assert!(resp2.accepted);
        assert_eq!(
            resp1.task_id, resp2.task_id,
            "idempotency_key should deduplicate"
        );
    }

    /// H.2 test 3: QueryState 跨仓查询
    #[tokio::test]
    async fn h2_query_state_cross_repo() {
        let (server, _rx) = PhysisServer::new();
        let server = Arc::new(server);
        let client = CrossRepoClient::new(server.clone());

        // 先派发
        let idem = format!("idem-{}", Uuid::new_v4());
        let dispatch = DispatchTaskRequest {
            task_id: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            kind: "physics_sim".into(),
            payload: serde_json::json!({}),
            idempotency_key: idem.clone(),
        };
        let resp = client.dispatch_task(dispatch).await.unwrap();
        let physis_task_id = resp.task_id.clone();

        // 查询
        let query = QueryStateRequest {
            task_id: physis_task_id.clone(),
        };
        let state = client.query_state(query).await.unwrap();
        assert_eq!(state.task_id, physis_task_id);
        assert_eq!(state.state, PhysisTaskState::Completed);
    }

    /// H.2 test 4: HealthCheck 跨仓健康检查
    #[tokio::test]
    async fn h2_health_check_cross_repo() {
        let (server, _rx) = PhysisServer::new();
        let server = Arc::new(server);
        let client = CrossRepoClient::new(server.clone());

        let health = client.health_check().await.unwrap();
        assert!(health.healthy);
        assert!(health.version.starts_with("physis-"));
        assert!(health.latency_ms < 1000); // 跨仓 < 1s
    }
}
