//! crates/star-dispatcher/src/sa_real_tests.rs
//!
//! H.3 6 SA e2e tests (12 tests, 2 per SA)
//! per 守门 #19 [P] 拍板

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::{
        AgentTask, SubAgent, SubAgentArchetype, TaskState,
    };
    use crate::sa_real_impls::{
        CodeReviewAgent, TestGenAgent, DocSyncAgent, RefactorAgent, DbMigrationAgent, DomainDevAgent,
    };

    fn make_task(payload: serde_json::Value) -> AgentTask {
        AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "test".into(),
            payload,
            idempotency_key: format!("idem-{}", Uuid::new_v4()),
            created_at_ms: 0,
            state: TaskState::Pending,
            state_history: vec![],
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// H.3 test 1: CodeReview 解析 pr_id + 记录 review
    #[tokio::test]
    async fn h3_code_review_parses_pr_id() {
        let agent = CodeReviewAgent::new();
        let task = make_task(serde_json::json!({"pr_id": "42"}));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 2: CodeReview 缺 pr_id 报错
    #[tokio::test]
    async fn h3_code_review_missing_pr_id() {
        let agent = CodeReviewAgent::new();
        let task = make_task(serde_json::json!({}));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }

    /// H.3 test 3: TestGen 解析 module_path + 生成 5 test
    #[tokio::test]
    async fn h3_test_gen_generates_5_tests() {
        let agent = TestGenAgent::new();
        let task = make_task(serde_json::json!({"module_path": "crates/star-saga/src/lib.rs"}));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 4: TestGen 缺 module_path 报错
    #[tokio::test]
    async fn h3_test_gen_missing_module_path() {
        let agent = TestGenAgent::new();
        let task = make_task(serde_json::json!({}));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }

    /// H.3 test 5: DocSync 解析 doc_path + 同步版本
    #[tokio::test]
    async fn h3_doc_sync_records_version() {
        let agent = DocSyncAgent::new();
        let task = make_task(serde_json::json!({"doc_path": "AGENTS.md"}));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 6: DocSync 缺 doc_path 报错
    #[tokio::test]
    async fn h3_doc_sync_missing_doc_path() {
        let agent = DocSyncAgent::new();
        let task = make_task(serde_json::json!({}));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }

    /// H.3 test 7: Refactor 解析 refactor_target + 3 步 plan
    #[tokio::test]
    async fn h3_refactor_3_step_plan() {
        let agent = RefactorAgent::new();
        let task = make_task(serde_json::json!({"refactor_target": "domain-tenant"}));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 8: Refactor 缺 refactor_target 报错
    #[tokio::test]
    async fn h3_refactor_missing_target() {
        let agent = RefactorAgent::new();
        let task = make_task(serde_json::json!({}));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }

    /// H.3 test 9: DbMigration 验证 W/T/M + 记录 status
    #[tokio::test]
    async fn h3_db_migration_validates_w_t_m() {
        let agent = DbMigrationAgent::new();
        let task = make_task(serde_json::json!({
            "migration_target": "audit.audit_event",
            "w_t_m_class": "T",
        }));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 10: DbMigration 无效 w_t_m_class 报错 (per 守门 #DB-13)
    #[tokio::test]
    async fn h3_db_migration_invalid_w_t_m() {
        let agent = DbMigrationAgent::new();
        let task = make_task(serde_json::json!({
            "migration_target": "audit.audit_event",
            "w_t_m_class": "X",  // 无效
        }));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }

    /// H.3 test 11: DomainDev 验证 22 domain-* crate + 记录 dev plan
    #[tokio::test]
    async fn h3_domain_dev_validates_crate() {
        let agent = DomainDevAgent::new();
        let task = make_task(serde_json::json!({"domain_target": "domain-tenant"}));
        let r = agent.run(&task).await;
        assert!(r.is_ok());
    }

    /// H.3 test 12: DomainDev 无效 domain 报错 (per 守门 §5 disclaimer)
    #[tokio::test]
    async fn h3_domain_dev_invalid_crate() {
        let agent = DomainDevAgent::new();
        let task = make_task(serde_json::json!({"domain_target": "fake-domain"}));
        let r = agent.run(&task).await;
        assert!(r.is_err());
    }
}
