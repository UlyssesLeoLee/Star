//! C21 Activity Heatmap 真实实现占位 (per docs/design/charts/c21-heatmap.md v1.0)
//!
//! **v0 phase 2 stub** (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter};
use uuid::Uuid;

/// 公开入口: 异步生成 Activity Heatmap Report
///
/// v0 phase 2 stub: P2 阶段 V2 接 domain-audit 按 (user_id, hour) × (weekday) 二维分布.
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    _report_id: Uuid,
) -> Result<crate::ReportResult, ReportError> {
    Err(ReportError::not_implemented(
        "ReportType::C21_Heatmap",
        "V2 port to domain-audit (per守门 #13 T-class 审计表), 2D distribution (user × time)",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c21_returns_not_implemented() {
        let err = ReportError::not_implemented("C21", "V2 port");
        assert_eq!(err.http_status(), 501);
    }
}
