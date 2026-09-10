//! A2.4 pipeline connector (per DD-CANVAS-AGENT-001 §3.1 + §4.14.1 + 守门 #13 a RLS + 守门 #19 v19 累积规).
//!
//! `PipelineConnector` 表示 agent 流水线 (A → B → C 直连, per BD-CANVAS-AGENT-001 §3 A2.4).
//! 主要业务方法: 创建/添加 step/查询 pipeline 列表, 0 外部 I/O.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` pipeline 错误 (per DD-AGENT §4.7 + 守门 #13 a RLS 13 类).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineError {
    /// pipeline step 列表为空 (append_step 前)
    EmptyPipeline,
    /// append_step 的 agent_id 已存在 pipeline 中 (duplicate step 禁止)
    DuplicateStep,
    /// tenant_id 未设置 (守门 #13 a RLS 13 类)
    TenantIdRequired,
}

/// A2.4 agent pipeline connector (per BD-CANVAS-AGENT-001 §3 A2.4 A → B → C 直连).
///
/// 字段: id + agent_ids (按顺序, A→B→C) + tenant_id (守门 #13 a) + created_at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineConnector {
    /// pipeline ID
    pub id: Uuid,
    /// pipeline step agent_ids (按顺序, A→B→C, ≥ 1)
    pub agent_ids: Vec<Uuid>,
    /// tenant_id (守门 #13 a 100% RLS 13 类 必携)
    pub tenant_id: Uuid,
    /// 创建时间 (UTC)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PipelineConnector {
    /// A2.4 业务方法: 创建 pipeline (空 step, 后续 append).
    pub fn new(id: Uuid, tenant_id: Uuid) -> Result<Self, PipelineError> {
        if tenant_id.is_nil() {
            return Err(PipelineError::TenantIdRequired);
        }
        Ok(Self {
            id,
            agent_ids: Vec::new(),
            tenant_id,
            created_at: chrono::Utc::now(),
        })
    }

    /// A2.4 业务方法: 添加 pipeline step (按顺序).
    pub fn append_step(&mut self, agent_id: Uuid) -> Result<(), PipelineError> {
        if self.agent_ids.contains(&agent_id) {
            return Err(PipelineError::DuplicateStep);
        }
        self.agent_ids.push(agent_id);
        Ok(())
    }

    /// A2.4 业务方法: pipeline step 数量.
    pub fn step_count(&self) -> usize {
        self.agent_ids.len()
    }

    /// A2.4 业务方法: 验证 pipeline 是否有效 (≥ 1 step + tenant != nil).
    pub fn is_valid(&self) -> bool {
        !self.agent_ids.is_empty() && !self.tenant_id.is_nil()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new_ok() {
        let id = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let p = PipelineConnector::new(id, tenant).expect("valid");
        assert_eq!(p.id, id);
        assert_eq!(p.tenant_id, tenant);
        assert_eq!(p.step_count(), 0);
        assert!(!p.is_valid()); // 0 step
    }

    #[test]
    fn test_pipeline_new_tenant_required() {
        let id = Uuid::new_v4();
        let result = PipelineConnector::new(id, Uuid::nil());
        assert_eq!(result.err(), Some(PipelineError::TenantIdRequired));
    }

    #[test]
    fn test_pipeline_append_step_ok() {
        let tenant = Uuid::new_v4();
        let mut p = PipelineConnector::new(Uuid::new_v4(), tenant).unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        p.append_step(a).unwrap();
        p.append_step(b).unwrap();
        p.append_step(c).unwrap();
        assert_eq!(p.step_count(), 3);
        assert!(p.is_valid());
        assert_eq!(p.agent_ids, vec![a, b, c]);
    }

    #[test]
    fn test_pipeline_append_duplicate() {
        let tenant = Uuid::new_v4();
        let mut p = PipelineConnector::new(Uuid::new_v4(), tenant).unwrap();
        let a = Uuid::new_v4();
        p.append_step(a).unwrap();
        let result = p.append_step(a);
        assert_eq!(result.err(), Some(PipelineError::DuplicateStep));
        assert_eq!(p.step_count(), 1);
    }
}
