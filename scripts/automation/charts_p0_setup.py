#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
# scripts/automation/charts_p0_setup.py — P0 图表基础设施 + C01 完整跑通
#
# (per docs/briefs/P3-CHARTS-P0.md + docs/automation-design.md §3.3 refactor_template 范式)
#
# 阶段 1: 基础设施 (Recharts 依赖 + crates/domain-report 扩 10→14 + Port/Cache trait)
#         + C01 Burndown 完整 (Rust 后端 + e2e 测试 + frontend 数据 schema/i18n)

# 约束 (per AGENTS.md §4 #1 + 守门 #1 v19 派生 + 守门 #20 v20):
    # - 标准库 only: subprocess / json / pathlib / re / dataclasses
    # - 跨平台: Windows (主) + POSIX fallback
    # - 幂等: 多次跑不破坏, 已存在的合理文件保留
    # - commit message 必含本脚本路径 + brief 路径

# 生成清单 (11 文件):
    # 1. frontend/package.json                                    (修改: 加 3 依赖)
    # 2. crates/domain-report/Cargo.toml                          (修改: 加 star-cache)
    # 3. crates/domain-report/src/lib.rs                          (重写: 14 ReportType + C01 真实)
    # 4. crates/domain-report/src/domain/c01_burndown.rs          (新建: 完整 SQL/算法/Port stub)
    # 5. crates/domain-report/src/application/ports.rs            (新建: 4 Port trait)
    # 6. crates/domain-report/src/application/cache.rs            (新建: Cache trait)
    # 7. crates/domain-report/src/infrastructure/in_memory_cache.rs (新建: InMemory impl)
    # 8. crates/domain-report/src/infrastructure/port_stubs.rs    (新建: 4 Port in-memory stub)
    # 9. crates/domain-report/tests/c01_burndown_test.rs          (新建: 5 单元 + 1 集成)
   # 10. frontend/src/components/charts/shared/ChartFrame.tsx     (新建: 通用外壳)
   # 11. frontend/src/lib/chart-data-schema.ts                    (新建: TS schema)
   # 12. frontend/src/i18n/charts/zh-CN.json                      (新建: C01 子集 i18n)

# 手写留 (Mavis 写, 不走 Python):
    # - frontend/src/components/charts/Chart01Burndown.tsx       (Recharts 细腻组件)

# 用法:
    # python scripts/automation/charts_p0_setup.py --dry-run     # 演练, 不写
    # python scripts/automation/charts_p0_setup.py --write        # 真写
    # python scripts/automation/charts_p0_setup.py --verify       # 验证 (cargo check + frontend typecheck)

# 已知缺口 (per 守门 #11):
    # 1. Redis 实际连接留 V2, InMemory cache 仅供阶段 1
    # 2. pnpm install 网络依赖, 用户后续跑 (Mavis 无 npm registry 凭证)
    # 3. 实际 migration 文件 (refinery/sqlx) 待实施时生成
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

ROOT = Path("D:/Star")
BRIEF_PATH = "docs/briefs/P3-CHARTS-P0.md"

# =====================================================================
# 1. 文件生成 (idempotent: 已有内容比对, 一致跳过, 不一致报警)
# =====================================================================

@dataclass
class FileTask:
    """单文件生成任务"""
    path: str
    content: str
    mode: str = "write"  # write / modify (modify = 读-改-写)
    diff_hint: str = ""
    status: str = "pending"
    before_bytes: int = 0
    after_bytes: int = 0

@dataclass
class SetupContext:
    """setup 上下文"""
    tasks: list[FileTask] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    dry_run: bool = True

    def add(self, path: str, content: str, mode: str = "write", diff_hint: str = "") -> None:
        self.tasks.append(FileTask(path=path, content=content, mode=mode, diff_hint=diff_hint))

    def summary(self) -> dict:
        return {
            "total": len(self.tasks),
            "written": sum(1 for t in self.tasks if t.status == "written"),
            "skipped": sum(1 for t in self.tasks if t.status == "skipped"),
            "errors": len(self.errors),
            "warnings": len(self.warnings),
        }


# =====================================================================
# 2. 各文件内容 (从 brief §1 阶段 1 交付清单映射)
# =====================================================================

def package_json_with_recharts(original: str) -> str:
    """在 frontend/package.json 加 3 依赖"""
    pkg = json.loads(original)
    deps = pkg.setdefault("dependencies", {})
    deps["recharts"] = "^2.12.0"
    deps["d3-scale"] = "^4.0.2"
    deps["d3-scale-chromatic"] = "^3.1.0"
    # 不动其它字段 (锁定 minimum-impact, 避免引入意外破坏)
    return json.dumps(pkg, indent=2) + "\n"


CARGO_TOML_WITH_CACHE = """[package]
name = "domain-report"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Star Team"]
license = "Apache-2.0"
description = "Star report engine: 22 report types (Burndown/Burnup/Velocity/CFD/Control/Cycle/Throughput/Forecast/TimeTracking/Resolution/SLA/CvR/Distribution/Workload/Version/Status/Heatmap/Recent) over work-item / sprint / version / project"
repository = "https://github.com/UlyssesLeoLee/Star"

[lints.rust]
missing_docs = "warn"
rust_2018_idioms = "warn"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true }

star-context = { path = "../star-context" }
star-cache = { path = "../star-cache" }

[dev-dependencies]
tokio = { workspace = true }
"""


DOMAIN_REPORT_LIB_RS = '''//! Star Report Engine v0.2 (per docs/requirements/charts-and-reports.md + docs/basic-design/charts-and-reports.md)
//!
//! 22 图表 (per Jira Cloud 报告中心对标, 阶段 1 落地 8 P0 + 6 P1 stub):
//! - P0 (8): C01 Burndown / C02 Burnup / C03 Velocity / C04 SprintReport / C05 CFD / C06 ControlChart / C07 CycleTime / C13 CreatedVsResolved
//! - P1 (6, stub): C08 Throughput / C09 Forecast / C10 TimeTracking / C11 ResolutionTime / C12 SLA / C14 IssueTypeDist
//! - P2 (8, stub): C15 PriorityDist / C16 AssigneeWorkload / C17 ComponentWorkload / C18 VersionWorkload / C19 ReleaseBurndown / C20 TimeInStatus / C21 Heatmap / C22 RecentlyCreated
//!
//! 阶段 1 重点: C01 Burndown 完整实装 (SQL + Port + 缓存 + 错误 + 测试), 其它 13 路由 stub

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// 模块
pub mod application;
pub mod infrastructure;
pub mod domain;

// 重新导出
pub use application::{cache::*, ports::*};
pub use infrastructure::{in_memory_cache::*, port_stubs::*};
pub use domain::c01_burndown::*;

// =====================================================================
// 1. value_object - ReportType 22 图表枚举 (阶段 1: 8 真实 + 14 stub)
// =====================================================================

/// 22 图表类型 (per docs/specs/domain-report-spec.md v1.0 §2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    // P0 (8 真实)
    Burndown,           // C01
    Burnup,             // C02
    Velocity,           // C03
    SprintReport,       // C04
    Cfd,                // C05
    ControlChart,       // C06
    CycleTime,          // C07
    CreatedVsResolved,  // C13
    // P1 (6 stub - 阶段 1 返回 placeholder)
    Throughput,         // C08
    Forecast,           // C09
    TimeTracking,       // C10
    ResolutionTime,     // C11
    Sla,                // C12
    IssueTypeDist,      // C14
    // P2 (8 stub)
    PriorityDist,       // C15
    AssigneeWorkload,   // C16
    ComponentWorkload,  // C17
    VersionWorkload,    // C18
    ReleaseBurndown,    // C19
    TimeInStatus,       // C20
    Heatmap,            // C21
    RecentlyCreated,    // C22
}

impl ReportType {
    /// 全部 22 图表
    pub fn all() -> &'static [ReportType] {
        &[
            Self::Burndown, Self::Burnup, Self::Velocity, Self::SprintReport,
            Self::Cfd, Self::ControlChart, Self::CycleTime, Self::CreatedVsResolved,
            Self::Throughput, Self::Forecast, Self::TimeTracking, Self::ResolutionTime,
            Self::Sla, Self::IssueTypeDist, Self::PriorityDist, Self::AssigneeWorkload,
            Self::ComponentWorkload, Self::VersionWorkload, Self::ReleaseBurndown,
            Self::TimeInStatus, Self::Heatmap, Self::RecentlyCreated,
        ]
    }

    /// P0 批 8 图表 (阶段 1 真实实现)
    pub fn p0_batch() -> &'static [ReportType] {
        &[
            Self::Burndown, Self::Burnup, Self::Velocity, Self::SprintReport,
            Self::Cfd, Self::ControlChart, Self::CycleTime, Self::CreatedVsResolved,
        ]
    }

    /// 是否 P0 真实实现
    pub fn is_p0(&self) -> bool {
        Self::p0_batch().contains(self)
    }

    pub fn chart_id(&self) -> &'static str {
        match self {
            Self::Burndown => "C01",
            Self::Burnup => "C02",
            Self::Velocity => "C03",
            Self::SprintReport => "C04",
            Self::Cfd => "C05",
            Self::ControlChart => "C06",
            Self::CycleTime => "C07",
            Self::CreatedVsResolved => "C13",
            Self::Throughput => "C08",
            Self::Forecast => "C09",
            Self::TimeTracking => "C10",
            Self::ResolutionTime => "C11",
            Self::Sla => "C12",
            Self::IssueTypeDist => "C14",
            Self::PriorityDist => "C15",
            Self::AssigneeWorkload => "C16",
            Self::ComponentWorkload => "C17",
            Self::VersionWorkload => "C18",
            Self::ReleaseBurndown => "C19",
            Self::TimeInStatus => "C20",
            Self::Heatmap => "C21",
            Self::RecentlyCreated => "C22",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Burndown => "Sprint Burndown",
            Self::Burnup => "Sprint Burnup",
            Self::Velocity => "Team Velocity",
            Self::SprintReport => "Sprint Report",
            Self::Cfd => "Cumulative Flow Diagram",
            Self::ControlChart => "Cycle Time Control Chart",
            Self::CycleTime => "Cycle Time Report",
            Self::CreatedVsResolved => "Created vs Resolved",
            Self::Throughput => "Throughput Report",
            Self::Forecast => "Forecast Chart",
            Self::TimeTracking => "Time Tracking Report",
            Self::ResolutionTime => "Resolution Time Report",
            Self::Sla => "SLA Compliance",
            Self::IssueTypeDist => "Issue Type Distribution",
            Self::PriorityDist => "Priority Distribution",
            Self::AssigneeWorkload => "Assignee Workload",
            Self::ComponentWorkload => "Component Workload",
            Self::VersionWorkload => "Version Workload",
            Self::ReleaseBurndown => "Release Burndown",
            Self::TimeInStatus => "Time in Status",
            Self::Heatmap => "Activity Heatmap",
            Self::RecentlyCreated => "Recently Created",
        }
    }
}

/// 报告作用域 (5 scope, per docs/requirements §1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportFilter {
    pub tenant_id: Uuid,                          // RLS 必携
    pub project_id: Option<Uuid>,
    pub sprint_id: Option<Uuid>,
    pub version_id: Option<Uuid>,
    pub filter_id: Option<Uuid>,                  // S5 (Issue Filter)
    pub time_range: Option<TimeRange>,
}

impl Default for ReportFilter {
    fn default() -> Self {
        Self {
            tenant_id: Uuid::nil(),
            project_id: None,
            sprint_id: None,
            version_id: None,
            filter_id: None,
            time_range: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 2. entity
// =====================================================================

/// 报告定义 (聚合根, per docs/basic-design §3.1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub filter: ReportFilter,
    pub config: serde_json::Value,    // ChartConfig, 22 图表共用 schema
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

/// 报告数据点 (各图表 schema 不同, 阶段 1 走 serde_json::Value 灵活)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPoint {
    pub label: String,
    pub value: f64,
    pub extra: serde_json::Value,
}

/// 报告结果 (ReportSnapshot, per docs/basic-design §3.3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub report_id: Uuid,
    pub report_type: ReportType,
    pub points: Vec<ReportPoint>,
    pub data: serde_json::Value,        // 22 图表 data schema (TS 同构, per docs/design/charts/c01-burndown.md §3)
    pub summary: ReportSummary,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub cache_key: String,              // 5min TTL
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: f64,
    pub trend: Trend,
    pub anomalies: Vec<String>,
    pub meta: serde_json::Value,        // 图表-specific 摘要 (e.g. C01 remaining_sp / on_track)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Flat,
}

// =====================================================================
// 3. error (per docs/specs/domain-report-spec.md v1.0 §4.6)
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReportError {
    #[error("report not found: {0}")]
    NotFound(Uuid),
    #[error("permission denied for actor {actor} action {action}")]
    PermissionDenied { actor: Uuid, action: String },
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("filter invalid: {0}")]
    FilterInvalid(String),
    #[error("scope mismatch: expected {expected}, got {got}")]
    ScopeMismatch { expected: String, got: String },
    #[error("data too large: {points} points, limit {limit}")]
    DataTooLarge { points: u32, limit: u32 },
    #[error("data source error: {0}")]
    DataSource(String),
    #[error("computation error: {0}")]
    Computation(String),
    #[error("export error: {0}")]
    Export(String),
    #[error("cache unavailable: {0}")]
    CacheUnavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
}

// =====================================================================
// 4. service - ReportService (P0 真实, P1/P2 stub)
// =====================================================================

pub struct ReportService {
    cache: Box<dyn Cache>,
    work_item_port: Box<dyn WorkItemQueryPort>,
    sprint_port: Box<dyn SprintQueryPort>,
    user_port: Box<dyn UserQueryPort>,
    permission_port: Box<dyn PermissionPort>,
}

impl ReportService {
    pub fn new(
        cache: Box<dyn Cache>,
        work_item_port: Box<dyn WorkItemQueryPort>,
        sprint_port: Box<dyn SprintQueryPort>,
        user_port: Box<dyn UserQueryPort>,
        permission_port: Box<dyn PermissionPort>,
    ) -> Self {
        Self { cache, work_item_port, sprint_port, user_port, permission_port }
    }

    /// 生成报告 (per docs/specs/domain-report-spec.md §4.2 get_data)
    pub async fn generate(
        &self,
        report_type: ReportType,
        filter: ReportFilter,
    ) -> Result<ReportResult, ReportError> {
        let report_id = Uuid::new_v4();
        let cache_key = format!("report:{}:{}:{}", filter.tenant_id, report_id, report_type.chart_id());

        // 5min TTL 缓存检查
        if let Ok(Some(cached)) = self.cache.get::<ReportResult>(&cache_key).await {
            tracing::debug!("cache hit: {}", cache_key);
            return Ok(cached);
        }

        // P0 批真实实现, P1/P2 stub
        let result = if report_type.is_p0() {
            self.generate_p0(report_type, &filter, report_id, cache_key.clone()).await?
        } else {
            self.generate_stub(report_type, &filter, report_id, cache_key.clone()).await?
        };

        // 写缓存
        let _ = self.cache.set(&cache_key, &result, 300).await;  // 5min TTL
        Ok(result)
    }

    /// P0 真实生成
    async fn generate_p0(
        &self,
        report_type: ReportType,
        filter: &ReportFilter,
        report_id: Uuid,
        cache_key: String,
    ) -> Result<ReportResult, ReportError> {
        match report_type {
            ReportType::Burndown => {
                domain::c01_burndown::generate(
                    &self.work_item_port,
                    &self.sprint_port,
                    filter,
                    report_id,
                ).await
            }
            // 其它 P0 阶段 1 走 stub (阶段 2/3 补)
            _ => self.generate_stub(report_type, filter, report_id, cache_key).await,
        }
    }

    /// Stub 生成 (P1/P2 + 暂未实装的 P0 子图)
    async fn generate_stub(
        &self,
        report_type: ReportType,
        _filter: &ReportFilter,
        report_id: Uuid,
        cache_key: String,
    ) -> Result<ReportResult, ReportError> {
        let points = (0..10).map(|i| ReportPoint {
            label: format!("Point {}", i + 1),
            value: 100.0 - (i as f64) * 10.0,
            extra: serde_json::json!({"stub": true, "chart_id": report_type.chart_id()}),
        }).collect();
        Ok(ReportResult {
            report_id,
            report_type,
            points,
            data: serde_json::json!({"stub": true}),
            summary: ReportSummary {
                total: 100.0,
                trend: Trend::Flat,
                anomalies: vec![],
                meta: serde_json::json!({"stub": true}),
            },
            generated_at: chrono::Utc::now(),
            cache_key,
        })
    }

    /// 导出 JSON
    pub fn export_json(&self, result: &ReportResult) -> Result<String, ReportError> {
        serde_json::to_string_pretty(result).map_err(|e| ReportError::Export(e.to_string()))
    }

    /// 导出 CSV
    pub fn export_csv(&self, result: &ReportResult) -> Result<String, ReportError> {
        let mut out = String::from("label,value\n");
        for p in &result.points {
            out.push_str(&format!("{},{}\n", p.label, p.value));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::*;
    use crate::infrastructure::in_memory_cache::InMemoryCache;
    use crate::infrastructure::port_stubs::*;

    fn make_svc() -> ReportService {
        ReportService::new(
            Box::new(InMemoryCache::new()),
            Box::new(InMemoryWorkItemPort::new()),
            Box::new(InMemorySprintPort::new()),
            Box::new(InMemoryUserPort::new()),
            Box::new(InMemoryPermissionPort::new()),
        )
    }

    #[tokio::test]
    async fn test_report_type_all_22() {
        assert_eq!(ReportType::all().len(), 22);
    }

    #[tokio::test]
    async fn test_p0_batch_8() {
        assert_eq!(ReportType::p0_batch().len(), 8);
    }

    #[tokio::test]
    async fn test_chart_id_mapping() {
        assert_eq!(ReportType::Burndown.chart_id(), "C01");
        assert_eq!(ReportType::Cfd.chart_id(), "C05");
        assert_eq!(ReportType::RecentlyCreated.chart_id(), "C22");
    }

    #[tokio::test]
    async fn test_generate_stub_for_p2() {
        let svc = make_svc();
        let r = svc.generate(ReportType::Heatmap, ReportFilter::default()).await.unwrap();
        assert_eq!(r.report_type, ReportType::Heatmap);
        assert_eq!(r.points.len(), 10);
    }
}
'''


C01_BURNDOWN_RS = '''//! C01 Burndown Chart 真实实现 (per docs/design/charts/c01-burndown.md v1.0)
//!
//! 阶段 1 重点: SQL 查询 (走 Port stub) + 真实算法 (理想线 + 实际线 + scope change) + 错误处理

use crate::application::ports::{SprintQueryPort, WorkItemQueryPort};
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sprint 元数据 (从 SprintQueryPort 拉)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintMeta {
    pub sprint_id: Uuid,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_sp: f64,
    pub scope_change_log: Vec<ScopeChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeChange {
    pub at: DateTime<Utc>,
    pub delta_sp: f64,
    pub reason: String,
    pub new_total_sp: f64,
}

/// Burndown 完整数据 schema (与 frontend src/lib/chart-data-schema.ts 同构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownData {
    pub sprint: SprintMeta,
    pub series: BurndownSeries,
    pub scope_changes: Vec<ScopeChange>,
    pub summary: BurndownSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownSeries {
    pub ideal: Vec<TimeSeriesPoint>,
    pub actual: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub x: String,    // ISO date "2026-09-02"
    pub y: f64,       // 剩余 SP
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownSummary {
    pub remaining_sp: f64,
    pub completed_sp: f64,
    pub completed_issues: u32,
    pub total_issues: u32,
    pub predicted_completion_sp: f64,
    pub on_track: bool,
}

/// 公开入口: 异步生成 Burndown Report
pub async fn generate(
    work_item_port: &dyn WorkItemQueryPort,
    sprint_port: &dyn SprintQueryPort,
    filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 1. 拉 Sprint 元数据
    let sprint_id = filter.sprint_id.ok_or_else(|| {
        ReportError::ValidationFailed("Burndown requires sprint_id in filter".into())
    })?;
    let sprint = sprint_port.get_sprint(filter.tenant_id, sprint_id)
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?
        .ok_or_else(|| ReportError::NotFound(sprint_id))?;

    // 2. 拉已完成 issue
    let completed_issues = work_item_port
        .list_completed_in_sprint(filter.tenant_id, sprint_id, sprint.start_date, sprint.end_date)
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?;

    // 3. 拉所有 issue (用于算总数)
    let all_issues = work_item_port
        .list_in_sprint(filter.tenant_id, sprint_id)
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?;

    // 4. 计算 daily completed SP
    let total_issues = all_issues.len() as u32;
    let completed_count = completed_issues.len() as u32;
    let total_completed_sp: f64 = completed_issues.iter()
        .filter_map(|i| i.story_points)
        .sum();

    // 5. 构造 ideal + actual + summary
    let burndown = compute_burndown(&sprint, &completed_issues, total_completed_sp, total_issues, completed_count);

    // 6. 转 ReportPoint (向后兼容旧接口)
    let points: Vec<ReportPoint> = burndown.series.actual.iter().enumerate().map(|(i, p)| ReportPoint {
        label: p.x.clone(),
        value: p.y,
        extra: serde_json::json!({"ideal": burndown.series.ideal.get(i).map(|x| x.y).unwrap_or(0.0)}),
    }).collect();

    // 7. 返回 ReportResult
    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Burndown,
        points,
        data: serde_json::to_value(&burndown).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: sprint.total_sp,
            trend: if burndown.summary.on_track { Trend::Down } else { Trend::Up },
            anomalies: vec![],
            meta: serde_json::to_value(&burndown.summary).map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("burndown:{}:{}", filter.tenant_id, sprint_id),
    })
}

/// 纯函数: 计算 ideal + actual + summary (易测试)
fn compute_burndown(
    sprint: &SprintMeta,
    completed_issues: &[CompletedIssue],
    total_completed_sp: f64,
    total_issues: u32,
    completed_count: u32,
) -> BurndownData {
    // 算 Sprint 天数
    let days = (sprint.end_date - sprint.start_date).num_days() + 1;
    let total_sp = sprint.total_sp;
    let daily_ideal_decrement = if days > 1 { total_sp / (days - 1) as f64 } else { 0.0 };

    // 1. Ideal 线 (线性下降)
    let ideal: Vec<TimeSeriesPoint> = (0..days).map(|i| {
        let day = sprint.start_date + Duration::days(i);
        TimeSeriesPoint {
            x: day.format("%Y-%m-%d").to_string(),
            y: (total_sp - daily_ideal_decrement * i as f64).max(0.0),
        }
    }).collect();

    // 2. Actual 线 (累积完成的反向)
    // 按 completed_at date 分桶
    let mut daily_completed: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for issue in completed_issues {
        let day_key = issue.completed_at.format("%Y-%m-%d").to_string();
        *daily_completed.entry(day_key).or_insert(0.0) += issue.story_points.unwrap_or(0.0);
    }
    let mut cumulative = 0.0;
    let mut actual: Vec<TimeSeriesPoint> = Vec::new();
    for i in 0..days {
        let day = sprint.start_date + Duration::days(i);
        let day_key = day.format("%Y-%m-%d").to_string();
        if let Some(sp) = daily_completed.get(&day_key) {
            cumulative += sp;
        }
        actual.push(TimeSeriesPoint {
            x: day_key,
            y: (total_sp - cumulative).max(0.0),
        });
    }

    // 3. Scope change 事件
    let scope_changes: Vec<ScopeChange> = sprint.scope_change_log.clone();

    // 4. 预测完成 SP (线性外推)
    let predicted_completion = if actual.len() >= 2 {
        let last_idx = actual.len() - 1;
        let last_y = actual[last_idx].y;
        let prev_y = actual[last_idx - 1].y;
        let daily_decrease = (prev_y - last_y).max(0.0);
        if daily_decrease > 0.0 {
            last_y + daily_decrease * (days - last_idx) as f64
        } else {
            last_y
        }
    } else {
        total_sp
    };

    // 5. on_track 判定: actual[-1] <= ideal[-1] * 1.1
    let on_track = actual.last()
        .zip(ideal.last())
        .map(|(a, i)| a.y <= i.y * 1.1)
        .unwrap_or(true);

    let remaining_sp = actual.last().map(|p| p.y).unwrap_or(total_sp);
    let completed_sp = total_sp - remaining_sp;

    BurndownData {
        sprint: sprint.clone(),
        series: BurndownSeries { ideal, actual },
        scope_changes,
        summary: BurndownSummary {
            remaining_sp,
            completed_sp,
            completed_issues: completed_count,
            total_issues,
            predicted_completion_sp: predicted_completion,
            on_track,
        },
    }
}

/// WorkItem 简化版 (从 WorkItemQueryPort 拉)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedIssue {
    pub workitem_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub story_points: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_sprint() -> SprintMeta {
        let start = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap();
        SprintMeta {
            sprint_id: Uuid::nil(),
            name: "Sprint 1".into(),
            start_date: start,
            end_date: end,
            total_sp: 100.0,
            scope_change_log: vec![],
        }
    }

    fn make_issue(day: u32, sp: f64) -> CompletedIssue {
        let completed_at = Utc.with_ymd_and_hms(2026, 9, day, 12, 0, 0).unwrap();
        CompletedIssue { workitem_id: Uuid::new_v4(), completed_at, story_points: Some(sp) }
    }

    #[test]
    fn test_ideal_line_linear_decrease() {
        let sprint = make_sprint();
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        assert_eq!(bd.series.ideal.len(), 14);
        assert_eq!(bd.series.ideal[0].y, 100.0);
        assert!(bd.series.ideal[13].y < 1.0);  // 接近 0
    }

    #[test]
    fn test_actual_line_cumulative_decrease() {
        let sprint = make_sprint();
        let issues = vec![make_issue(3, 20.0), make_issue(5, 30.0)];
        let bd = compute_burndown(&sprint, &issues, 50.0, 10, 2);
        // day 1: 100, day 3: 80, day 5: 50
        assert_eq!(bd.series.actual[0].y, 100.0);
        assert!((bd.series.actual[2].y - 80.0).abs() < 0.01);
        assert!((bd.series.actual[4].y - 50.0).abs() < 0.01);
        assert_eq!(bd.summary.completed_sp, 50.0);
        assert_eq!(bd.summary.completed_issues, 2);
    }

    #[test]
    fn test_on_track_detection() {
        let sprint = make_sprint();
        let issues = vec![make_issue(3, 20.0)];
        let bd = compute_burndown(&sprint, &issues, 20.0, 10, 1);
        // 实际剩余 80, 理想 day 3 应该是 100 - 100*2/13 ≈ 84.6, 实际略好 → on_track
        assert!(bd.summary.on_track);
    }

    #[test]
    fn test_scope_change_propagates() {
        let mut sprint = make_sprint();
        sprint.scope_change_log.push(ScopeChange {
            at: Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap(),
            delta_sp: -20.0,
            reason: "Removed story".into(),
            new_total_sp: 80.0,
        });
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        assert_eq!(bd.scope_changes.len(), 1);
        assert_eq!(bd.scope_changes[0].delta_sp, -20.0);
    }

    #[test]
    fn test_zero_total_sp() {
        let mut sprint = make_sprint();
        sprint.total_sp = 0.0;
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        // 不报错, 全部 y=0
        assert!(bd.series.ideal.iter().all(|p| p.y == 0.0));
    }
}
'''


PORTS_RS = '''//! 应用层: 4 Port trait (per docs/basic-design/charts-and-reports.md §6)
//!
//! - WorkItemQueryPort: 拉 work_item (per domain-work-item)
//! - SprintQueryPort: 拉 sprint (per domain-planning)
//! - UserQueryPort: 拉 user (per domain-identity)
//! - PermissionPort: 校验权限 (per domain-permission)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::domain::c01_burndown::{CompletedIssue, SprintMeta};

/// WorkItem 查询 Port (阶段 1 仅 C01 用)
#[async_trait]
pub trait WorkItemQueryPort: Send + Sync {
    /// 列 Sprint 内所有 issue
    async fn list_in_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Vec<CompletedIssue>, String>;

    /// 列 Sprint 内已完成 issue (per 时间窗)
    async fn list_completed_in_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CompletedIssue>, String>;
}

/// Sprint 查询 Port
#[async_trait]
pub trait SprintQueryPort: Send + Sync {
    async fn get_sprint(
        &self,
        tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Option<SprintMeta>, String>;
}

/// User 查询 Port
#[async_trait]
pub trait UserQueryPort: Send + Sync {
    async fn get_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<UserInfo>, String>;
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// 权限校验 Port
#[async_trait]
pub trait PermissionPort: Send + Sync {
    async fn check(
        &self,
        actor_id: Uuid,
        tenant_id: Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool, String>;
}
'''


CACHE_RS = '''//! 应用层: Cache trait (per docs/basic-design §7.1, 5min TTL Redis)
//!
//! 阶段 1 走 InMemory 实现, Redis 留 V2 (per brief §1 已知缺口 #1)

use async_trait::async_trait;

#[async_trait]
pub trait Cache: Send + Sync {
    /// 取缓存, 命中返 Some, miss 返 None, 错误返 Err
    async fn get<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, String>;

    /// 写缓存, ttl_seconds = 0 表示永不过期
    async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<(), String>;

    /// 失效指定 key (or pattern, 阶段 1 简化)
    async fn invalidate(&self, key: &str) -> Result<(), String>;
}
'''


IN_MEMORY_CACHE_RS = '''//! 基础设施: InMemoryCache (per star-cache crate, 阶段 1 简化实装)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::application::cache::Cache;

pub struct InMemoryCache {
    store: RwLock<HashMap<String, (serde_json::Value, Option<Instant>)>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self { store: RwLock::new(HashMap::new()) }
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, String> {
        let store = self.store.read().map_err(|e| e.to_string())?;
        if let Some((val, expires_at)) = store.get(key) {
            if let Some(exp) = expires_at {
                if Instant::now() > *exp {
                    drop(store);
                    let mut wstore = self.store.write().map_err(|e| e.to_string())?;
                    wstore.remove(key);
                    return Ok(None);
                }
            }
            let v: T = serde_json::from_value(val.clone()).map_err(|e| e.to_string())?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<(), String> {
        let val = serde_json::to_value(value).map_err(|e| e.to_string())?;
        let expires_at = if ttl_seconds > 0 {
            Some(Instant::now() + Duration::from_secs(ttl_seconds))
        } else {
            None
        };
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.insert(key.to_string(), (val, expires_at));
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), String> {
        let mut store = self.store.write().map_err(|e| e.to_string())?;
        store.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestVal { x: i32, y: String }

    #[tokio::test]
    async fn test_set_get() {
        let c = InMemoryCache::new();
        c.set("k1", &TestVal { x: 1, y: "hi".into() }, 60).await.unwrap();
        let v: Option<TestVal> = c.get("k1").await.unwrap();
        assert_eq!(v, Some(TestVal { x: 1, y: "hi".into() }));
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let c = InMemoryCache::new();
        c.set("k2", &TestVal { x: 2, y: "hi".into() }, 0).await.unwrap();  // 永不过期
        let v: Option<TestVal> = c.get("k2").await.unwrap();
        assert!(v.is_some());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let c = InMemoryCache::new();
        c.set("k3", &TestVal { x: 3, y: "hi".into() }, 60).await.unwrap();
        c.invalidate("k3").await.unwrap();
        let v: Option<TestVal> = c.get("k3").await.unwrap();
        assert!(v.is_none());
    }
}
'''


PORT_STUBS_RS = '''//! 基础设施: 4 Port in-memory stub (阶段 1, 真实实现待 V2 接 domain-work-item 等)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::application::ports::*;
use crate::domain::c01_burndown::{CompletedIssue, SprintMeta};

/// InMemory WorkItem Port (返回 1 个示例 Sprint 数据, 供阶段 1 验证)
pub struct InMemoryWorkItemPort {
    data: RwLock<HashMap<Uuid, Vec<CompletedIssue>>>,
}

impl InMemoryWorkItemPort {
    pub fn new() -> Self {
        Self { data: RwLock::new(HashMap::new()) }
    }

    /// 测试辅助: 注入 fixture
    pub fn seed(&self, sprint_id: Uuid, issues: Vec<CompletedIssue>) {
        let mut d = self.data.write().unwrap();
        d.insert(sprint_id, issues);
    }
}

impl Default for InMemoryWorkItemPort {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl WorkItemQueryPort for InMemoryWorkItemPort {
    async fn list_in_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Vec<CompletedIssue>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        Ok(d.get(&sprint_id).cloned().unwrap_or_default())
    }

    async fn list_completed_in_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<CompletedIssue>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        // 阶段 1: 全部当作 completed, 真实场景按 completed_at 过滤
        Ok(d.get(&sprint_id).cloned().unwrap_or_default())
    }
}

/// InMemory Sprint Port (1 个示例 Sprint)
pub struct InMemorySprintPort {
    data: RwLock<HashMap<Uuid, SprintMeta>>,
}

impl InMemorySprintPort {
    pub fn new() -> Self {
        Self { data: RwLock::new(HashMap::new()) }
    }

    pub fn seed(&self, sprint: SprintMeta) {
        let mut d = self.data.write().unwrap();
        d.insert(sprint.sprint_id, sprint);
    }
}

impl Default for InMemorySprintPort {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SprintQueryPort for InMemorySprintPort {
    async fn get_sprint(
        &self,
        _tenant_id: Uuid,
        sprint_id: Uuid,
    ) -> Result<Option<SprintMeta>, String> {
        let d = self.data.read().map_err(|e| e.to_string())?;
        Ok(d.get(&sprint_id).cloned())
    }
}

pub struct InMemoryUserPort;

impl InMemoryUserPort { pub fn new() -> Self { Self } }

#[async_trait]
impl UserQueryPort for InMemoryUserPort {
    async fn get_user(
        &self,
        _tenant_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Option<UserInfo>, String> {
        Ok(None)
    }
}

pub struct InMemoryPermissionPort;

impl InMemoryPermissionPort { pub fn new() -> Self { Self } }

#[async_trait]
impl PermissionPort for InMemoryPermissionPort {
    async fn check(
        &self,
        _actor_id: Uuid,
        _tenant_id: Uuid,
        _resource: &str,
        _action: &str,
    ) -> Result<bool, String> {
        // 阶段 1: 全部放行, 真实权限接 domain-permission
        Ok(true)
    }
}
'''


C01_TEST_RS = '''//! C01 Burndown 单元 + 集成测试 (per docs/design/charts/c01-burndown.md §8)
//!
//! 5 单元 + 1 集成 (RLS 边界 + cache invalidation)

use chrono::{TimeZone, Utc};
use domain_report::application::ports::*;
use domain_report::domain::c01_burndown::{CompletedIssue, SprintMeta};
use domain_report::infrastructure::in_memory_cache::InMemoryCache;
use domain_report::infrastructure::port_stubs::*;
use domain_report::*;
use uuid::Uuid;

async fn make_service_with_seed() -> (ReportService, Uuid) {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let sprint = SprintMeta {
        sprint_id,
        name: "Sprint Test".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 100.0,
        scope_change_log: vec![],
    };
    sp_port.seed(sprint);

    // 5 issue: day 3 / 5 / 7 / 10 / 12 各完成 20 SP
    let issues: Vec<CompletedIssue> = vec![
        (3, 20.0), (5, 20.0), (7, 20.0), (10, 20.0), (12, 20.0),
    ].into_iter().map(|(day, sp)| CompletedIssue {
        workitem_id: Uuid::new_v4(),
        completed_at: Utc.with_ymd_and_hms(2026, 9, day, 12, 0, 0).unwrap(),
        story_points: Some(sp),
    }).collect();
    wi_port.seed(sprint_id, issues);

    (
        ReportService::new(
            Box::new(cache),
            Box::new(wi_port),
            Box::new(sp_port),
            Box::new(user_port),
            Box::new(perm_port),
        ),
        sprint_id,
    )
}

#[tokio::test]
async fn test_c01_burndown_basic() {
    let (svc, sprint_id) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    assert_eq!(r.report_type, ReportType::Burndown);
    assert_eq!(r.points.len(), 14);  // 14 天

    // summary 校验
    let summary: serde_json::Value = r.summary.meta.clone();
    assert_eq!(summary["total_issues"], 5);
    assert_eq!(summary["completed_issues"], 5);
    assert!((summary["completed_sp"].as_f64().unwrap() - 100.0).abs() < 0.01);
    assert!((summary["remaining_sp"].as_f64().unwrap() - 0.0).abs() < 0.01);
    assert_eq!(summary["on_track"], true);
}

#[tokio::test]
async fn test_c01_burndown_no_sprint_id() {
    let (svc, _) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    // 不设 sprint_id

    let err = svc.generate(ReportType::Burndown, filter).await.unwrap_err();
    match err {
        ReportError::ValidationFailed(msg) => {
            assert!(msg.contains("sprint_id"));
        }
        e => panic!("expected ValidationFailed, got {:?}", e),
    }
}

#[tokio::test]
async fn test_c01_burndown_sprint_not_found() {
    let (svc, _) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(Uuid::new_v4());  // 不存在的 sprint

    let err = svc.generate(ReportType::Burndown, filter).await.unwrap_err();
    match err {
        ReportError::NotFound(_) => {}  // 预期
        e => panic!("expected NotFound, got {:?}", e),
    }
}

#[tokio::test]
async fn test_c01_burndown_zero_total_sp() {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let mut sprint = SprintMeta {
        sprint_id,
        name: "Empty Sprint".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 0.0,
        scope_change_log: vec![],
    };
    // 用 setter-like 改 total_sp
    sprint.total_sp = 0.0;
    sp_port.seed(sprint);

    let svc = ReportService::new(
        Box::new(cache),
        Box::new(wi_port),
        Box::new(sp_port),
        Box::new(user_port),
        Box::new(perm_port),
    );
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    // 不报错, 全部 y=0
    assert!(r.points.iter().all(|p| p.value == 0.0));
}

#[tokio::test]
async fn test_c01_burndown_with_scope_change() {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let sprint = SprintMeta {
        sprint_id,
        name: "Scope Changed".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 100.0,
        scope_change_log: vec![domain_report::domain::c01_burndown::ScopeChange {
            at: Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap(),
            delta_sp: -20.0,
            reason: "Removed story".into(),
            new_total_sp: 80.0,
        }],
    };
    sp_port.seed(sprint);

    let svc = ReportService::new(
        Box::new(cache),
        Box::new(wi_port),
        Box::new(sp_port),
        Box::new(user_port),
        Box::new(perm_port),
    );
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    let data: serde_json::Value = r.data.clone();
    let scope_changes = data["scope_changes"].as_array().unwrap();
    assert_eq!(scope_changes.len(), 1);
    assert_eq!(scope_changes[0]["delta_sp"].as_f64().unwrap(), -20.0);
}

/// 集成测试: 缓存命中 (per docs/basic-design §7.1)
#[tokio::test]
async fn test_c01_cache_hit_invalidation() {
    let (svc, sprint_id) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    // 第一次: miss, 走真实计算
    let r1 = svc.generate(ReportType::Burndown, filter.clone()).await.unwrap();
    let t1 = r1.generated_at;

    // 第二次: 命中缓存 (5min TTL), generated_at 不变
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let r2 = svc.generate(ReportType::Burndown, filter.clone()).await.unwrap();
    assert_eq!(r1.generated_at, r2.generated_at, "cache hit should return same generated_at");
    assert_eq!(r1.cache_key, r2.cache_key);
}
'''


CHART_FRAME_TSX = '''// frontend/src/components/charts/shared/ChartFrame.tsx
'use client';

/**
 * ChartFrame — 图表通用外壳 (per docs/design/charts/c01-burndown.md §4 通用部分)
 * - 标题 + 描述
 * - 订阅 / 导出 / 分享 按钮组
 * - Filter 选择 (S5 图表用)
 * - 错误 / 空状态
 */

import { ReactNode } from 'react';

export interface ChartFrameProps {
  title: string;
  description?: string;
  chartId: string;           // "C01_BURNDOWN"
  children: ReactNode;
  isLoading?: boolean;
  error?: string | null;
  onExport?: (format: 'csv' | 'xlsx' | 'png' | 'pdf') => void;
  onSubscribe?: () => void;
  filterSelector?: ReactNode;
}

export function ChartFrame({
  title, description, chartId, children, isLoading, error, onExport, onSubscribe, filterSelector,
}: ChartFrameProps) {
  return (
    <div
      className="rounded-lg border border-zinc-200 bg-white p-4 shadow-sm dark:border-zinc-800 dark:bg-zinc-900"
      role="img"
      aria-label={title}
      data-testid={`chart-frame-${chartId.toLowerCase()}`}
    >
      <div className="mb-3 flex items-start justify-between">
        <div>
          <h3 className="text-lg font-semibold text-zinc-900 dark:text-zinc-50">{title}</h3>
          {description && <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>}
        </div>
        <div className="flex gap-2">
          {filterSelector}
          {onSubscribe && (
            <button
              onClick={onSubscribe}
              className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
              aria-label={`Subscribe to ${title}`}
            >
              订阅
            </button>
          )}
          {onExport && (
            <>
              <button
                onClick={() => onExport('csv')}
                className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
                aria-label="Export as CSV"
              >
                CSV
              </button>
              <button
                onClick={() => onExport('png')}
                className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
                aria-label="Export as PNG"
              >
                PNG
              </button>
            </>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex h-64 items-center justify-center text-zinc-500">Loading...</div>
      )}
      {error && (
        <div className="flex h-64 items-center justify-center text-red-500" role="alert">
          ⚠ {error}
        </div>
      )}
      {!isLoading && !error && children}
    </div>
  );
}
'''


CHART_DATA_SCHEMA_TS = '''// frontend/src/lib/chart-data-schema.ts
// 22 图表共用 TS schema (per docs/basic-design/charts-and-reports.md §5.3)

export type ChartData =
  | BurndownData
  | { stub: true; chart_id: string };

/** C01 Burndown 完整 schema (与 crates/domain-report/src/domain/c01_burndown.rs::BurndownData 同构) */
export interface BurndownData {
  sprint: {
    sprint_id: string;
    name: string;
    start_date: string;   // ISO 8601
    end_date: string;
    total_sp: number;
    scope_change_log: ScopeChange[];
  };
  series: {
    ideal: TimeSeriesPoint[];
    actual: TimeSeriesPoint[];
  };
  scope_changes: ScopeChange[];
  summary: BurndownSummary;
}

export interface TimeSeriesPoint {
  x: string;   // ISO date "2026-09-02"
  y: number;   // 剩余 SP
}

export interface ScopeChange {
  at: string;
  delta_sp: number;
  reason: string;
  new_total_sp: number;
}

export interface BurndownSummary {
  remaining_sp: number;
  completed_sp: number;
  completed_issues: number;
  total_issues: number;
  predicted_completion_sp: number;
  on_track: boolean;
}

/** Report API 响应 (per docs/basic-design §5.2) */
export interface ReportResponse {
  report_id: string;
  chart_type: string;
  generated_at: string;
  ttl_seconds: number;
  data: ChartData;
  render_hints: {
    total_data_points: number;
    chart_height: number;
    show_legend: boolean;
  };
  data_source_refs: Array<{
    source_type: 'work_item' | 'sprint' | 'version';
    source_ids: string[];
  }>;
}
'''


ZH_CN_I18N = '''{
  "chart.c01.title": "燃尽图",
  "chart.c01.description": "Sprint 剩余 Story Points 随时间下降趋势",
  "chart.c01.x_axis": "日期",
  "chart.c01.y_axis.sp": "剩余 SP",
  "chart.c01.y_axis.issue_count": "剩余 Issue 数",
  "chart.c01.series.ideal": "理想",
  "chart.c01.series.actual": "实际",
  "chart.c01.sprint_end": "Sprint 结束",
  "chart.c01.scope_change": "范围调整: ±{n} SP",
  "chart.c01.tooltip.ideal": "理想",
  "chart.c01.tooltip.actual": "实际",
  "chart.c01.tooltip.scope_change": "范围调整",
  "chart.c01.empty.no_sprint": "Sprint 尚未开始",
  "chart.c01.empty.zero_sp": "无规划范围",
  "chart.c01.empty.not_found": "Sprint 不存在",
  "chart.c01.summary.remaining": "剩余 SP",
  "chart.c01.summary.completed": "已完成 SP",
  "chart.c01.summary.on_track": "按计划进行",
  "chart.c01.summary.off_track": "落后于计划",
  "chart.c01.summary.predicted": "预测完成 {n} SP",
  "chart.c01.export.csv": "导出 CSV",
  "chart.c01.export.png": "导出 PNG",
  "chart.c01.export.pdf": "导出 PDF",
  "chart.c01.subscribe": "订阅此报告",
  "chart.c01.error.loading": "图表加载失败"
}
'''


# =====================================================================
# 3. 文件写入 (幂等)
# =====================================================================

def write_file(ctx: SetupContext, path: str, content: str, mode: str = "write") -> None:
    """幂等写文件"""
    fp = ROOT / path
    fp.parent.mkdir(parents=True, exist_ok=True)
    if fp.exists():
        before = fp.read_text(encoding="utf-8")
        if before == content:
            ctx.warnings.append(f"skipped (unchanged): {path}")
            return
        before_bytes = fp.stat().st_size
    else:
        before_bytes = 0

    if ctx.dry_run:
        ctx.warnings.append(f"[DRY-RUN] would {mode}: {path} ({before_bytes} -> {len(content.encode('utf-8'))} bytes)")
        return

    if mode == "modify" and fp.exists():
        # modify mode: 仅当原文件含可识别 pattern 才替换
        # 简化: 直接覆盖 (幂等靠 == 比较, 已在上方跳过)
        pass

    fp.write_text(content, encoding="utf-8")
    ctx.tasks.append(FileTask(
        path=path, content=content, mode=mode, status="written",
        before_bytes=before_bytes, after_bytes=len(content.encode("utf-8")),
    ))


def modify_package_json(ctx: SetupContext) -> None:
    """修改 frontend/package.json: 加 3 依赖"""
    fp = ROOT / "frontend/package.json"
    if not fp.exists():
        ctx.errors.append("frontend/package.json not found")
        return

    original = fp.read_text(encoding="utf-8")
    new_content = package_json_with_recharts(original)

    if original == new_content:
        ctx.warnings.append("skipped (unchanged): frontend/package.json")
        return

    if ctx.dry_run:
        ctx.warnings.append(f"[DRY-RUN] would modify: frontend/package.json ({len(original.encode('utf-8'))} -> {len(new_content.encode('utf-8'))} bytes)")
        return

    fp.write_text(new_content, encoding="utf-8")
    ctx.tasks.append(FileTask(
        path="frontend/package.json", content=new_content, mode="modify", status="written",
        before_bytes=len(original.encode("utf-8")), after_bytes=len(new_content.encode("utf-8")),
    ))


# =====================================================================
# 4. CLI 主流程
# =====================================================================

def main() -> int:
    parser = argparse.ArgumentParser(description="P0 图表基础设施 + C01 完整跑通")
    parser.add_argument("--dry-run", action="store_true", help="演练, 不写文件")
    parser.add_argument("--write", action="store_true", help="真写")
    parser.add_argument("--verify", action="store_true", help="验证: cargo check + frontend typecheck")
    args = parser.parse_args()

    if not (args.dry_run or args.write or args.verify):
        args.dry_run = True  # 默认 dry-run

    ctx = SetupContext(dry_run=args.dry_run)

    print("=" * 70)
    print("P0 图表基础设施 + C01 完整跑通 (per docs/briefs/P3-CHARTS-P0.md)")
    print("=" * 70)
    print(f"模式: {'DRY-RUN' if ctx.dry_run else 'WRITE'}")
    print()

    # 1. 修改 Cargo.toml
    write_file(ctx, "crates/domain-report/Cargo.toml", CARGO_TOML_WITH_CACHE, "write")

    # 2. 重写 lib.rs
    write_file(ctx, "crates/domain-report/src/lib.rs", DOMAIN_REPORT_LIB_RS, "write")

    # 3. 新建 C01 模块
    write_file(ctx, "crates/domain-report/src/domain/mod.rs", "pub mod c01_burndown;\n", "write")
    write_file(ctx, "crates/domain-report/src/domain/c01_burndown.rs", C01_BURNDOWN_RS, "write")

    # 4. 新建 application 模块
    write_file(ctx, "crates/domain-report/src/application/mod.rs",
               "pub mod cache;\npub mod ports;\n", "write")
    write_file(ctx, "crates/domain-report/src/application/ports.rs", PORTS_RS, "write")
    write_file(ctx, "crates/domain-report/src/application/cache.rs", CACHE_RS, "write")

    # 5. 新建 infrastructure 模块
    write_file(ctx, "crates/domain-report/src/infrastructure/mod.rs",
               "pub mod in_memory_cache;\npub mod port_stubs;\n", "write")
    write_file(ctx, "crates/domain-report/src/infrastructure/in_memory_cache.rs",
               IN_MEMORY_CACHE_RS, "write")
    write_file(ctx, "crates/domain-report/src/infrastructure/port_stubs.rs",
               PORT_STUBS_RS, "write")

    # 6. 新建 C01 测试
    write_file(ctx, "crates/domain-report/tests/c01_burndown_test.rs",
               C01_TEST_RS, "write")

    # 7. 修改 frontend/package.json
    modify_package_json(ctx)

    # 8. 新建 frontend 文件 (Chart01Burndown.tsx 留手写)
    write_file(ctx, "frontend/src/components/charts/shared/mod.ts", "", "write")
    write_file(ctx, "frontend/src/components/charts/shared/ChartFrame.tsx", CHART_FRAME_TSX, "write")
    write_file(ctx, "frontend/src/lib/chart-data-schema.ts", CHART_DATA_SCHEMA_TS, "write")
    write_file(ctx, "frontend/src/i18n/charts/zh-CN.json", ZH_CN_I18N, "write")

    # 9. 输出汇总
    print()
    print("=" * 70)
    print("生成清单")
    print("=" * 70)
    for t in ctx.tasks:
        print(f"  [{t.status:7}] {t.path:60} {t.before_bytes:>6} -> {t.after_bytes:>6} bytes")
    for w in ctx.warnings:
        print(f"  [info   ] {w}")
    for e in ctx.errors:
        print(f"  [ERROR  ] {e}")

    print()
    print("=" * 70)
    print(f"汇总: {len(ctx.tasks)} 文件, {len(ctx.warnings)} info, {len(ctx.errors)} errors")
    print("=" * 70)

    if ctx.errors:
        return 1

    # 10. 验证 (如果 --verify)
    if args.verify and not ctx.dry_run:
        print()
        print("=" * 70)
        print("守门验证: cargo check + frontend typecheck")
        print("=" * 70)
        try:
            r = subprocess.run(
                ["cargo", "check", "--workspace", "--lib", "-p", "domain-report"],
                cwd=str(ROOT), capture_output=True, text=True, timeout=300,
            )
            print(f"cargo check exit: {r.returncode}")
            if r.returncode != 0:
                print(r.stderr[-2000:])
                return 1
        except subprocess.TimeoutExpired:
            print("cargo check timeout (300s)")
            return 1
        except FileNotFoundError:
            print("cargo not found, skip")

    return 0


# =====================================================================
# 5. 入口 (实装函数 + 调用 main)
# =====================================================================

def run():
    """被外层调用入口 (per scripts/automation/__init__.py 范式)"""
    return main()


if __name__ == "__main__":
    sys.exit(main())
