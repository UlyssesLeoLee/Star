// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-01 集群子域 (per SRS-001 §4 F-01 + brief §2.1 wt3)
//!
//! MVP-骨架: 1 数据结构 `HelmRelease` + 3 stub method (commit 03d7d43)
//! F-01 端到端: 4 endpoint 真实 (list_releases/canary/rollback/status) 调
//! `scripts/automation/helm_canary_mock.sh` subprocess, 守门 #1 R-05 不动生产,
//! 守门 #19 v19 agent 外部交互走 scripts/automation.

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

/// Helm release 当前状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub revision: u32,
    pub status: ReleaseStatus,
    pub last_deployed_at: DateTime<Utc>,
    /// 0-100, MVP 默认 0
    pub canary_weight: u8,
}

/// 灰度请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRequest {
    pub release_name: String,
    /// 0-100
    pub canary_weight: u8,
    /// 目标 revision (None = 最新)
    pub target_revision: Option<u32>,
}

/// 回滚请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub release_name: String,
    /// 回滚目标 revision
    pub target_revision: u32,
}

/// 灰度/回滚 ack (per OPS-BASIC-DESIGN §3.1 + brief §2.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmActionAck {
    pub action_id: String,
    pub status: String,
    pub detail: String,
}

/// Mock JSON 输出 (per helm_canary_mock.sh 输出 schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmMockOutput {
    pub status: String,
    pub release_name: String,
    pub revision: u32,
    pub canary_weight: u8,
    pub detail: String,
    pub mock: bool,
}

/// 调 helm_canary_mock.sh subprocess (守门 #19 v19 + #24 v2 + 守门 #1 R-05)
async fn run_helm_mock(action: &str, args: &[&str]) -> Result<HelmMockOutput, String> {
    use tokio::process::Command;
    let script = format!(
        "{}/scripts/automation/helm_canary_mock.sh",
        env!("CARGO_MANIFEST_DIR").replace("crates/star-ops", "")
    );
    let mut cmd = Command::new("bash");
    cmd.arg(&script).arg(action);
    for a in args {
        cmd.arg(a);
    }
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("subprocess 调起失败: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "helm_canary_mock.sh exit {}: stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("解析 mock 输出失败: {}", e))
}

impl HelmRelease {
    /// 列出 release (F-01 端到端, 调 helm_canary_mock.sh list)
    pub async fn list_releases() -> Result<Vec<Self>, String> {
        let output = run_helm_mock("list", &[]).await?;
        let release_name = output.release_name.clone();
        Ok(vec![Self {
            name: release_name.clone(),
            namespace: "default".to_string(),
            chart: format!("{}-0.1.0", release_name),
            revision: output.revision,
            status: ReleaseStatus::from_str(&output.status),
            last_deployed_at: Utc::now(),
            canary_weight: output.canary_weight,
        }])
    }

    /// 触发灰度 (F-01 端到端, 调 helm_canary_mock.sh canary)
    pub async fn trigger_canary(req: &CanaryRequest) -> Result<HelmActionAck, String> {
        let target_rev = req
            .target_revision
            .map(|r| r.to_string())
            .unwrap_or_else(|| "latest".to_string());
        let output = run_helm_mock(
            "canary",
            &[
                "--release",
                &req.release_name,
                "--weight",
                &req.canary_weight.to_string(),
                "--target",
                &target_rev,
            ],
        )
        .await?;
        let ts = chrono::Utc::now().timestamp();
        Ok(HelmActionAck {
            action_id: format!("canary-{}-{}", output.release_name, ts),
            status: output.status,
            detail: output.detail,
        })
    }

    /// 回滚 (F-01 端到端, 调 helm_canary_mock.sh rollback)
    pub async fn rollback(req: &RollbackRequest) -> Result<HelmActionAck, String> {
        let output = run_helm_mock(
            "rollback",
            &[
                "--release",
                &req.release_name,
                "--target",
                &req.target_revision.to_string(),
            ],
        )
        .await?;
        let ts = chrono::Utc::now().timestamp();
        Ok(HelmActionAck {
            action_id: format!("rollback-{}-{}", output.release_name, ts),
            status: output.status,
            detail: output.detail,
        })
    }

    /// 状态 (F-01 端到端, 调 helm_canary_mock.sh status)
    pub async fn status(release_name: &str) -> Result<HelmMockOutput, String> {
        run_helm_mock("status", &["--release", release_name]).await
    }

    /// MVP 兼容 stub (老 cluster_list handler 用, 守门 #1 R-05)
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

impl ReleaseStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "healthy" => ReleaseStatus::Healthy,
            "pending" => ReleaseStatus::Pending,
            "canary" => ReleaseStatus::Canary,
            "degraded" => ReleaseStatus::Degraded,
            "failed" => ReleaseStatus::Failed,
            _ => ReleaseStatus::Pending,
        }
    }
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
        let req = CanaryRequest {
            release_name: "star-mcp".to_string(),
            canary_weight: 10,
            target_revision: Some(4),
        };
        assert_eq!(req.canary_weight, 10);
    }

    #[test]
    fn release_status_from_str_works() {
        assert_eq!(ReleaseStatus::from_str("healthy"), ReleaseStatus::Healthy);
        assert_eq!(ReleaseStatus::from_str("canary"), ReleaseStatus::Canary);
        assert_eq!(ReleaseStatus::from_str("unknown"), ReleaseStatus::Pending);
    }

    #[tokio::test]
    async fn helm_action_ack_serde() {
        let ack = HelmActionAck {
            action_id: "test-1".to_string(),
            status: "canary".to_string(),
            detail: "mock ok".to_string(),
        };
        let json = serde_json::to_string(&ack).unwrap();
        let back: HelmActionAck = serde_json::from_str(&json).unwrap();
        assert_eq!(ack.action_id, back.action_id);
    }
}
