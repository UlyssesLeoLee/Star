//! C19 Release Burndown 真实实现占位 (per docs/design/charts/c19-release-burndown.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Release Burndown Report
///
/// v0 phase 2 stub: P2 阶段 V2 接 domain-work-item 按 release_date 累计 burndown.
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    _report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C19_ReleaseBurndown",
        "V2 port to domain-work-item per守门 #4, cumulative burndown by release_date",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c19_returns_not_implemented() {
        let err = ReportError::not_implemented("C19", "V2 port");
        assert_eq!(err.http_status(), 501);
    }
}
