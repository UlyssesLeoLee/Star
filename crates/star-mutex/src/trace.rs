//! # TraceId Propagator
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.4.1
//!
//! 跨层 trace_id 传递 (UI → L0 → L1 → L3 → PG), 1 click 定位锁错乱
//!
//! 守门合规:
//! - 守门 #12 v2: 4 层 trace_id 必带, 锁错乱可 1 click 看到 4 层日志

use std::cell::RefCell;
use uuid::Uuid;

/// Trace ID 传播器 (per L0 派发层 + L1 SubAgent + L3 Domain 共享).
///
/// 用 thread-local 存储当前 actor 的 trace_id, 跨函数调用 + 跨层 RPC 传递.
#[derive(Debug, Default)]
pub struct TraceIdPropagator {
    inner: RefCell<Option<TraceContext>>,
}

/// Trace context (actor + trace_id + tenant_id).
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace UUID.
    pub trace_id: Uuid,
    /// Tenant UUID.
    pub tenant_id: Uuid,
    /// Actor UUID.
    pub actor_id: Uuid,
    /// Workspace UUID.
    pub workspace_id: Uuid,
}

impl TraceIdPropagator {
    /// 创建新 propagator.
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置当前 trace context.
    pub fn set(&self, ctx: TraceContext) {
        *self.inner.borrow_mut() = Some(ctx);
    }

    /// 获取当前 trace context.
    pub fn current(&self) -> Option<TraceContext> {
        self.inner.borrow().clone()
    }

    /// 获取当前 trace_id (None 表示未设置).
    pub fn current_trace_id(&self) -> Option<Uuid> {
        self.inner.borrow().as_ref().map(|c| c.trace_id)
    }

    /// 获取当前 tenant_id.
    pub fn current_tenant_id(&self) -> Option<Uuid> {
        self.inner.borrow().as_ref().map(|c| c.tenant_id)
    }

    /// 获取当前 actor_id.
    pub fn current_actor_id(&self) -> Option<Uuid> {
        self.inner.borrow().as_ref().map(|c| c.actor_id)
    }

    /// 获取当前 workspace_id.
    pub fn current_workspace_id(&self) -> Option<Uuid> {
        self.inner.borrow().as_ref().map(|c| c.workspace_id)
    }

    /// 清除当前 trace context.
    pub fn clear(&self) {
        *self.inner.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let prop = TraceIdPropagator::new();
        let trace = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let actor = Uuid::new_v4();
        let workspace = Uuid::new_v4();

        prop.set(TraceContext {
            trace_id: trace,
            tenant_id: tenant,
            actor_id: actor,
            workspace_id: workspace,
        });

        assert_eq!(prop.current_trace_id(), Some(trace));
        assert_eq!(prop.current_tenant_id(), Some(tenant));
        assert_eq!(prop.current_actor_id(), Some(actor));
        assert_eq!(prop.current_workspace_id(), Some(workspace));
    }

    #[test]
    fn test_clear() {
        let prop = TraceIdPropagator::new();
        prop.set(TraceContext {
            trace_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            actor_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
        });
        prop.clear();
        assert!(prop.current().is_none());
    }

    #[test]
    fn test_default_empty() {
        let prop = TraceIdPropagator::new();
        assert!(prop.current().is_none());
        assert_eq!(prop.current_trace_id(), None);
    }
}
