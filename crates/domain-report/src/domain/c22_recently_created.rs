//! C22 Recently Created 真实实现占位 (per docs/design/charts/c22-recently-created.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Recently Created Report
///
/// v0 phase 2 stub: P2 阶段 V2 接 domain-work-item 查 created_at DESC LIMIT N.
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    _report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C22_RecentlyCreated",
        "V2 port to domain-work-item per守门 #4, query by created_at DESC with limit",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c22_returns_not_implemented() {
        let err = ReportError::not_implemented("C22", "V2 port");
        assert_eq!(err.http_status(), 501);
    }
}
