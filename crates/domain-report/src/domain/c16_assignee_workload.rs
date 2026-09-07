//! C16 Assignee Workload 真实实现占位 (per docs/design/charts/c16-assignee-workload.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表): P2 阶段 worker
//! 子代理实装真实数据接入 (V2 接 `domain-work-item` 按 assignee 分组聚合).
//! 当前返回 `ReportError::not_implemented` 错误结构化 (per OPT-WORKER-01 模式).

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Assignee Workload Report
///
/// v0 phase 2 stub: 返回 NotImplemented 错误, 待 P2 阶段 V2 接 domain-work-item
/// 按 assignee_id 分组聚合 (e.g. SELECT assignee_id, COUNT(*) FROM work_item
/// WHERE status = 'open' GROUP BY assignee_id).
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C16_AssigneeWorkload",
        "V2 port to domain-work-item per守门 #4 (25 domain-* 真实数据接入), 12/25 → 25/25, \
         aggregate by assignee_id with open/closed status filter",
    ))
    // report_id 当前未使用, 占位以保持函数签名一致
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c16_returns_not_implemented() {
        // 同步测试: not_implemented 错误结构
        let err = ReportError::not_implemented(
            "ReportType::C16_AssigneeWorkload",
            "V2 port to domain-work-item per守门 #4",
        );
        assert_eq!(err.http_status(), 501);
        assert_eq!(err.code(), "REPORT_NOT_IMPLEMENTED");
        let s = format!("{err}");
        assert!(s.contains("C16_AssigneeWorkload"));
        assert!(s.contains("V2 port to domain-work-item"));
    }
}
