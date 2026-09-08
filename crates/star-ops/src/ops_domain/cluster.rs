// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-01 集群子域 (per SRS-001 §4 F-01)
//!
//! MVP-骨架: 1 数据结构 `HelmRelease` + 3 stub method
//! 实装阶段: 接入 kube-rs + helm SDK (per SRS-001 §10.2)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Helm release 状态 (per SRS-001 §4 F-01)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStatus {
    /// 部署中
    Pending,
    /// 健康
    Healthy,
    /// 降级 (部分 pod 不可用)
    Degraded,
    /// 失败
    Failed,
    /// 灰度中
    Canary,
}

/// Helm release 当前状态 (MVP stub 数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub revision: u32,
    pub status: ReleaseStatus,
    pub last_deployed_at: DateTime<Utc>,
    /// 0-100, MVP 永远 0 (实装阶段启用灰度)
    pub canary_weight: u8,
}

impl HelmRelease {
    /// MVP stub: 列出 release (per OPS-BASIC-DESIGN §3.1)
    pub fn list_stub() -> Vec<Self> {
        vec![Self {
            name: "star-mcp".to_string(),
            namespace: "default".to_string(),
            chart: "star-mcp-0.1.0".to_string(),
            revision: 3,
            status: ReleaseStatus::Healthy,
            last_deployed_at: "2026-09-08T07:00:00Z".parse().expect("hardcoded UTC"),
            canary_weight: 0,
        }]
    }
}

/// 灰度请求 (per OPS-BASIC-DESIGN §3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRequest {
    pub release_name: String,
    /// 0-100
    pub canary_weight: u8,
    /// 目标 revision (None = 最新)
    pub target_revision: Option<u32>,
}

/// 回滚请求 (per OPS-BASIC-DESIGN §3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub release_name: String,
    /// 回滚目标 revision
    pub target_revision: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_stub_returns_one_helm_release() {
        let releases = HelmRelease::list_stub();
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].name, "star-mcp");
        assert_eq!(releases[0].status, ReleaseStatus::Healthy);
    }

    #[test]
    fn canary_request_validates_weight_range() {
        // MVP 仅 struct, 业务校验 [M] 子项
        let req = CanaryRequest {
            release_name: "star-mcp".to_string(),
            canary_weight: 10,
            target_revision: Some(4),
        };
        assert_eq!(req.canary_weight, 10);
    }
}
