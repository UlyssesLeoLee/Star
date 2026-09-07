//! C17 Component Workload 真实实现占位 (per docs/design/charts/c17-component-workload.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Component Workload Report
///
/// v0 phase 2 stub: P2 阶段 V2 接 domain-work-item 按 component 分组聚合.
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    _report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C17_ComponentWorkload",
        "V2 port to domain-work-item per守门 #4, aggregate by component_id",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c17_returns_not_implemented() {
        let err = ReportError::not_implemented("C17", "V2 port");
        assert_eq!(err.http_status(), 501);
    }
}
