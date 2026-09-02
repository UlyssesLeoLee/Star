#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# scripts/automation/charts_p1.py — P1 批 7 图表共享部分生成
#
# (per docs/briefs/P3-CHARTS-P1.md v0.1 + 2026-09-02 15:28 JST Ulysses 拍板)
#
# 阶段 3: P1 7 图表 (C08-C12, C14-C15) 批量实装
# 复用 P0 阶段 2 (charts_p0_bulk.py) 模式
#
# 本脚本职责 (共享部分, 避免 token 爆炸):
#   1. domain/mod.rs (append 7 pub mod, 共 15)
#   2. lib.rs (在 generate_p0 加 7 match 分支, 共 15)
#   3. frontend chart-data-schema.ts (+7 interface)
#   4. frontend zh-CN.json (+7 chart.c0X.* keys)
#
# 7 图表 unique c0X.rs + Chart0X.tsx 留 Mavis 手写
#
# 用法:
#   python scripts/automation/charts_p1.py --write
#   python scripts/automation/charts_p1.py --verify

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

ROOT = Path("D:/Star")
BRIEF_PATH = "docs/briefs/P3-CHARTS-P1.md"

P1_CHARTS = [
    {"id": "C08", "name": "Throughput", "module": "c08_throughput", "data_type": "TimeSeriesWithMovingAvg"},
    {"id": "C09", "name": "Forecast", "module": "c09_forecast", "data_type": "ForecastWithConfidence"},
    {"id": "C10", "name": "TimeTracking", "module": "c10_time_tracking", "data_type": "TableWithProgress"},
    {"id": "C11", "name": "ResolutionTime", "module": "c11_resolution_time", "data_type": "GroupedBar"},
    {"id": "C12", "name": "Sla", "module": "c12_sla_compliance", "data_type": "TimeSeriesWithTarget"},
    {"id": "C14", "name": "IssueTypeDist", "module": "c14_issue_type_dist", "data_type": "PieWithCenter"},
    {"id": "C15", "name": "PriorityDist", "module": "c15_priority_dist", "data_type": "PieWithCenter"},
]


@dataclass
class FileTask:
    path: str
    content: str
    mode: str = "write"
    status: str = "pending"
    before_bytes: int = 0
    after_bytes: int = 0


@dataclass
class SetupContext:
    tasks: list[FileTask] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    dry_run: bool = True


def build_domain_mod_rs() -> str:
    # 已存在 P0 8 + P1 7 = 15 modules
    p0_modules = ["c01_burndown", "c02_burnup", "c03_velocity", "c04_sprint_report", "c05_cfd", "c06_control_chart", "c07_cycle_time", "c13_created_vs_resolved"]
    p1_modules = [c["module"] for c in P1_CHARTS]
    all_modules = p0_modules + p1_modules
    return "".join(f"pub mod {m};\n" for m in all_modules)


def build_lib_rs_p1_extension() -> str:
    """P1 7 match 分支 (无 sprint_port 依赖: C08-C15 大部分仅需 work_item_port)"""
    branches = []
    for c in P1_CHARTS:
        module = c["module"]
        # 所有 P1 仅需 work_item_port
        branches.append(f"""            ReportType::{c['name']} => {{
                domain::{module}::generate(
                    &*self.work_item_port,
                    filter,
                    report_id,
                ).await
            }}""")
    return "\n".join(branches)


def build_chart_data_schema_extension() -> str:
    interfaces = []
    for c in P1_CHARTS:
        cid = c["id"]
        name = c["name"]
        dt = c["data_type"]
        if dt == "TimeSeriesWithMovingAvg":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (per docs/design/charts/c{cid.lower()[1:]}-{name.lower()}.md) */
export interface {name}Data {{
  granularity: 'day' | 'week' | 'month';
  series: Array<{{ bucket: string; count: number }}>;
  moving_avg: Array<{{ bucket: string; avg: number }}>;
  stats: {{ total: number; avg: number; std_dev: number }};
}}""")
        elif dt == "ForecastWithConfidence":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  historical: {{
    sprints: Array<{{ name: string; completed_sp: number }}>;
    avg_velocity: number;
  }};
  forecast: {{
    method: 'simple_avg' | 'rolling_avg' | 'linear_regression';
    predicted_velocity: number;
    confidence_80: [number, number];
    confidence_95: [number, number];
    predicted_completion_date: string;
  }};
}}""")
        elif dt == "TableWithProgress":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  granularity: 'user' | 'project' | 'issue';
  rows: Array<{{
    id: string;
    name: string;
    original_seconds: number;
    spent_seconds: number;
    remaining_seconds: number;
    progress: number;
  }}>;
  summary: {{
    total_original: number;
    total_spent: number;
    total_remaining: number;
  }};
}}""")
        elif dt == "GroupedBar":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  group_by: 'priority' | 'type' | 'assignee';
  rows: Array<{{
    group: string;
    avg_days: number;
    median_days: number;
    count: number;
  }}>;
}}""")
        elif dt == "TimeSeriesWithTarget":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  series: Array<{{
    day: string;
    priorities: Record<string, {{ met: number; total: number; compliance: number }}>;
  }}>;
  summary: {{
    overall_compliance: number;
    by_priority: Record<string, number>;
    breaches: number;
  }};
  target_line: number;
}}""")
        elif dt == "PieWithCenter":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  slices: Array<{{
    key: string;
    count: number;
    percentage: number;
  }}>;
  total: number;
  status_filter?: 'all' | 'open' | 'closed';
}}""")
    return "\n".join(interfaces)


def build_zh_cn_extension() -> dict:
    extensions = {}
    title_zh = {
        "Throughput": "吞吐量报告", "Forecast": "预测", "TimeTracking": "时间跟踪报告",
        "ResolutionTime": "解决时间报告", "Sla": "SLA 合规报告",
        "IssueTypeDist": "问题类型分布", "PriorityDist": "优先级分布",
    }
    for c in P1_CHARTS:
        cid_lower = c["id"].lower()
        name_en = c["name"]
        prefix = f"chart.{cid_lower}."
        extensions[f"{prefix}title"] = title_zh.get(name_en, name_en)
        extensions[f"{prefix}x_axis"] = "日期"
        extensions[f"{prefix}y_axis"] = "数值"
        extensions[f"{prefix}empty.no_data"] = "无数据"
        extensions[f"{prefix}error.loading"] = "图表加载失败"
        extensions[f"{prefix}export.csv"] = "导出 CSV"
        extensions[f"{prefix}export.png"] = "导出 PNG"
        extensions[f"{prefix}subscribe"] = "订阅此报告"
    return extensions


def write_file(ctx: SetupContext, path: str, content: str) -> None:
    fp = ROOT / path
    fp.parent.mkdir(parents=True, exist_ok=True)
    if fp.exists() and fp.read_text(encoding="utf-8") == content:
        ctx.warnings.append(f"skipped (unchanged): {path}")
        return
    before_bytes = fp.stat().st_size if fp.exists() else 0
    if ctx.dry_run:
        ctx.warnings.append(f"[DRY-RUN] would write: {path} ({before_bytes} -> {len(content.encode('utf-8'))} bytes)")
        return
    fp.write_text(content, encoding="utf-8")
    ctx.tasks.append(FileTask(path=path, content=content, status="written", before_bytes=before_bytes, after_bytes=len(content.encode("utf-8"))))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    if not (args.dry_run or args.write or args.verify):
        args.dry_run = True
    ctx = SetupContext(dry_run=args.dry_run)

    print(f"P1 7 图表: " + ", ".join(c["id"] for c in P1_CHARTS))
    print(f"模式: {'DRY-RUN' if ctx.dry_run else 'WRITE'}")

    write_file(ctx, "crates/domain-report/src/domain/mod.rs", build_domain_mod_rs())

    # lib.rs / chart-data-schema / zh-CN 留上层 caller (Mavis Edit)
    lib_ext = build_lib_rs_p1_extension()
    Path("/tmp/lib_p1_extension.rs").write_text(lib_ext, encoding="utf-8")
    ctx.warnings.append(f"[INFO] lib.rs P1 extension saved to /tmp/lib_p1_extension.rs")

    schema_ext = build_chart_data_schema_extension()
    Path("/tmp/chart_data_schema_p1_extension.ts").write_text(schema_ext, encoding="utf-8")
    ctx.warnings.append(f"[INFO] chart-data-schema.ts P1 extension saved")

    zh_ext = build_zh_cn_extension()
    Path("/tmp/zh_cn_p1_extension.json").write_text(json.dumps(zh_ext, indent=2, ensure_ascii=False), encoding="utf-8")
    ctx.warnings.append(f"[INFO] zh-CN.json P1 extension saved")

    print()
    for t in ctx.tasks:
        print(f"  [written] {t.path:60} {t.before_bytes:>6} -> {t.after_bytes:>6} bytes")
    for w in ctx.warnings:
        print(f"  [info   ] {w}")
    for e in ctx.errors:
        print(f"  [ERROR  ] {e}")
    print()
    print(f"汇总: {len(ctx.tasks)} 文件, {len(ctx.warnings)} info, {len(ctx.errors)} errors")
    return 0 if not ctx.errors else 1


if __name__ == "__main__":
    sys.exit(main())
