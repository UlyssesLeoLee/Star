//! C20 Time in Status 真实实现占位 (per docs/design/charts/c20-time-in-status.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Time in Status Report
///
/// v0 phase 2 stub: P2 阶段 V2 接 domain-work-item 计算各状态停留时间 (work_item_history).
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    _report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C20_TimeInStatus",
        "V2 port to domain-work-item history (per F-027 状态变更审计), aggregate by status",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c20_returns_not_implemented() {
        let err = ReportError::not_implemented("C20", "V2 port");
        assert_eq!(err.http_status(), 501);
    }
}
