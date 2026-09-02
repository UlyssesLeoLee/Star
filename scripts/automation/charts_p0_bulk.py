#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# scripts/automation/charts_p0_bulk.py — P0 阶段 2: 7 图表共享部分生成
#
# (per docs/briefs/P3-CHARTS-P0-BULK.md v0.1 + 2026-09-02 14:06 JST Ulysses 拍板 A+I+i)
#
# 阶段 2: 复用 C01 模板, 7 图表 (C02-C05, C06, C07, C13) 批量实装
#
# 本脚本职责 (共享部分, 避免 token 爆炸):
#   1. crates/domain-report/src/domain/mod.rs         (加 7 pub mod)
#   2. crates/domain-report/src/lib.rs                (在 generate_p0 加 7 match 分支, call c0X::generate)
#   3. frontend/src/lib/chart-data-schema.ts          (加 7 chart data interface)
#   4. frontend/src/i18n/charts/zh-CN.json            (加 7 chart.c0X.* keys)
#
# 7 图表 unique 部分 (c0X.rs + test.rs + Chart0X.tsx) 由 Mavis Edit 工具手写 (per 守门 #19 v19 子代理不可靠 + token 控制)
#
# 用法:
#   python scripts/automation/charts_p0_bulk.py --dry-run     # 演练
#   python scripts/automation/charts_p0_bulk.py --write        # 真写
#   python scripts/automation/charts_p0_bulk.py --verify       # 守门 (cargo check)
#
# 已知缺口 (per 守门 #11):
#   1. 7 图表 unique 算法 (c0X.rs 算法 + Modified Z-Score) 留 Mavis Edit
#   2. 7 图表 Recharts 组件 (Chart0X.tsx) 留 Mavis 手写
#   3. Redis 实际连接留 V2 (C01 阶段 1 决策)
#   4. pnpm install 网络依赖, 用户后续跑

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

ROOT = Path("D:/Star")
BRIEF_PATH = "docs/briefs/P3-CHARTS-P0-BULK.md"

# 7 图表元数据 (per 22 详细设计)
P0_BULK_CHARTS = [
    {
        "id": "C02", "name": "Burnup", "module": "c02_burnup",
        "chart_id_const": "C02",
        "scope_default": "Sprint",
        "data_type": "TimeSeriesCumulative",
        "reuse_pct": 70,
        "test_count": 6,
    },
    {
        "id": "C03", "name": "Velocity", "module": "c03_velocity",
        "chart_id_const": "C03",
        "scope_default": "Project",
        "data_type": "BarWithAvgLine",
        "reuse_pct": 50,
        "test_count": 6,
    },
    {
        "id": "C04", "name": "SprintReport", "module": "c04_sprint_report",
        "chart_id_const": "C04",
        "scope_default": "Sprint",
        "data_type": "TableWithSummary",
        "reuse_pct": 30,  # 表格新模板
        "test_count": 6,
    },
    {
        "id": "C05", "name": "Cfd", "module": "c05_cfd",
        "chart_id_const": "C05",
        "scope_default": "Project",
        "data_type": "StackedArea",
        "reuse_pct": 60,
        "test_count": 6,
    },
    {
        "id": "C06", "name": "ControlChart", "module": "c06_control_chart",
        "chart_id_const": "C06",
        "scope_default": "IssueFilter",
        "data_type": "ScatterWithRefLines",
        "reuse_pct": 50,  # Modified Z-Score 算法独立
        "test_count": 6,
    },
    {
        "id": "C07", "name": "CycleTime", "module": "c07_cycle_time",
        "chart_id_const": "C07",
        "scope_default": "IssueFilter",
        "data_type": "HistogramWithPercentiles",
        "reuse_pct": 70,
        "test_count": 6,
    },
    {
        "id": "C13", "name": "CreatedVsResolved", "module": "c13_created_vs_resolved",
        "chart_id_const": "C13",
        "scope_default": "Project",
        "data_type": "DualLine",
        "reuse_pct": 80,
        "test_count": 6,
    },
]


@dataclass
class FileTask:
    path: str
    content: str
    mode: str = "modify"  # modify (默认) / write
    status: str = "pending"
    before_bytes: int = 0
    after_bytes: int = 0


@dataclass
class SetupContext:
    tasks: list[FileTask] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    dry_run: bool = True

    def summary(self) -> dict:
        return {
            "total": len(self.tasks),
            "written": sum(1 for t in self.tasks if t.status == "written"),
            "skipped": sum(1 for t in self.tasks if t.status == "skipped"),
            "errors": len(self.errors),
            "warnings": len(self.warnings),
        }


# =====================================================================
# 1. domain/mod.rs (7 pub mod)
# =====================================================================

def build_domain_mod_rs() -> str:
    modules = ["c01_burndown"] + [c["module"] for c in P0_BULK_CHARTS]
    lines = [f"pub mod {m};\n" for m in modules]
    return "".join(lines)


# =====================================================================
# 2. lib.rs generate_p0 扩展 (在现有 match ReportType::Burndown 后加 7 分支)
# =====================================================================

def build_lib_rs_p0_extension() -> str:
    """返回要插入 generate_p0 match 块的 7 个分支"""
    lines = []
    for c in P0_BULK_CHARTS:
        chart_id = c["id"]
        module = c["module"]
        lines.append(f"""            ReportType::{c['name']} => {{
                domain::{module}::generate(
                    &self.work_item_port,
                    &self.sprint_port,
                    filter,
                    report_id,
                ).await
            }}""")
    return "\n".join(lines)


# =====================================================================
# 3. frontend/src/lib/chart-data-schema.ts (7 chart data interface)
# =====================================================================

def build_chart_data_schema_extension() -> str:
    """返回 7 个 chart data interface, 追加到 chart-data-schema.ts"""
    interfaces = []
    for c in P0_BULK_CHARTS:
        cid = c["id"]
        name = c["name"]
        dt = c["data_type"]
        if dt == "TimeSeriesCumulative":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (per docs/design/charts/c{cid.lower()[1:]}-{name.lower()}.md) */
export interface {name}Data {{
  sprint?: SprintMeta;
  series: {{
    actual: TimeSeriesPoint[];
    scope?: TimeSeriesPoint[];     // 范围阶梯 (C02)
  }};
  scope_changes?: ScopeChange[];
  summary: {{
    completed_sp: number;
    total_sp: number;
    completion_ratio: number;
  }};
}}""")
        elif dt == "BarWithAvgLine":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema */
export interface {name}Data {{
  sprints: Array<{{
    sprint_id: string;
    name: string;
    committed_sp: number;
    completed_sp: number | null;
  }}>;
  average_completed_sp: number;
  trend: 'increasing' | 'decreasing' | 'stable';
}}""")
        elif dt == "TableWithSummary":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (表格) */
export interface {name}Data {{
  sprint: {{ sprint_id: string; name: string }};
  groups: {{
    completed: IssueRow[];
    carry_over: IssueRow[];
    incomplete: IssueRow[];
  }};
  summary: {{
    completed_count: number;
    carry_over_count: number;
    incomplete_count: number;
    completed_sp: number;
  }};
}}

export interface IssueRow {{
  key: string;
  title: string;
  type: string;
  priority: string;
  assignee?: {{ name: string; avatar_url: string }};
  completed_at?: string;
  story_points?: number;
}}""")
        elif dt == "StackedArea":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (累积流图) */
export interface {name}Data {{
  date_range: {{ start: string; end: string }};
  status_categories: string[];
  series: Array<{{
    day: string;
    counts: Record<string, number>;
  }}>;
  total: number;
}}""")
        elif dt == "ScatterWithRefLines":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (控制图) */
export interface {name}Data {{
  data_points: Array<{{
    workitem_id: string;
    key: string;
    cycle_time_days: number;
    completed_at: string;
    anomaly: boolean;
    z_score: number;
  }}>;
  reference_lines: Array<{{
    y_value: number;
    label: string;
    style: 'solid' | 'dashed' | 'dotted';
  }}>;
  stats: {{
    median: number;
    p70: number;
    p85: number;
    p95: number;
    mean: number;
    std_dev: number;
  }};
}}""")
        elif dt == "HistogramWithPercentiles":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (直方图) */
export interface {name}Data {{
  buckets: Array<{{
    range_start: number;
    range_end: number;
    count: number;
    label: string;
  }}>;
  percentiles: {{ p50: number; p85: number; p95: number }};
  stats: {{
    total_count: number;
    median: number;
    mean: number;
  }};
  bucket_size: number;
}}""")
        elif dt == "DualLine":
            interfaces.append(f"""
/** C{cid[1:]} {name} data schema (双线对比) */
export interface {name}Data {{
  series: Array<{{
    day: string;
    created: number;
    resolved: number;
  }}>;
  summary: {{
    total_created: number;
    total_resolved: number;
    net_change: number;
    backlog_trend: 'growing' | 'shrinking' | 'stable';
  }};
  time_granularity: 'day' | 'week' | 'month';
}}""")
    return "\n".join(interfaces)


# =====================================================================
# 4. frontend/src/i18n/charts/zh-CN.json (7 chart.c0X.* keys)
# =====================================================================

def build_zh_cn_extension() -> dict:
    """返回 7 图表 i18n keys dict"""
    extensions = {}
    for c in P0_BULK_CHARTS:
        cid_lower = c["id"].lower()
        name_en = c["name"]
        title_zh = {
            "Burnup": "燃起图", "Velocity": "速度图", "SprintReport": "Sprint 报告",
            "Cfd": "累积流图 (CFD)", "ControlChart": "控制图",
            "CycleTime": "周期时间报告", "CreatedVsResolved": "新建 vs 解决",
        }.get(name_en, name_en)
        prefix = f"chart.{cid_lower}."
        extensions[f"{prefix}title"] = title_zh
        extensions[f"{prefix}x_axis"] = "日期"
        extensions[f"{prefix}y_axis"] = "数值"
        extensions[f"{prefix}empty.no_data"] = "无数据"
        extensions[f"{prefix}error.loading"] = "图表加载失败"
        extensions[f"{prefix}export.csv"] = "导出 CSV"
        extensions[f"{prefix}export.png"] = "导出 PNG"
        extensions[f"{prefix}subscribe"] = "订阅此报告"
    return extensions


# =====================================================================
# 5. 文件写入 (幂等)
# =====================================================================

def modify_or_write(ctx: SetupContext, path: str, new_content: str, mode: str = "modify") -> None:
    """modify = 替换文件中特定 marker 之间的内容; write = 全替换"""
    fp = ROOT / path
    fp.parent.mkdir(parents=True, exist_ok=True)

    if mode == "write":
        if fp.exists() and fp.read_text(encoding="utf-8") == new_content:
            ctx.warnings.append(f"skipped (unchanged): {path}")
            return
        before_bytes = fp.stat().st_size if fp.exists() else 0
        if ctx.dry_run:
            ctx.warnings.append(f"[DRY-RUN] would write: {path} ({before_bytes} -> {len(new_content.encode('utf-8'))} bytes)")
            return
        fp.write_text(new_content, encoding="utf-8")
        ctx.tasks.append(FileTask(
            path=path, content=new_content, mode="write", status="written",
            before_bytes=before_bytes, after_bytes=len(new_content.encode("utf-8")),
        ))
    else:
        # modify mode: 留给上层 caller 用 regex 替换
        pass


def main() -> int:
    parser = argparse.ArgumentParser(description="P0 阶段 2 共享部分生成")
    parser.add_argument("--dry-run", action="store_true", help="演练")
    parser.add_argument("--write", action="store_true", help="真写")
    parser.add_argument("--verify", action="store_true", help="cargo check 验证")
    args = parser.parse_args()
    if not (args.dry_run or args.write or args.verify):
        args.dry_run = True

    ctx = SetupContext(dry_run=args.dry_run)
    print("=" * 70)
    print(f"P0 阶段 2 共享部分生成 (per docs/briefs/P3-CHARTS-P0-BULK.md v0.1)")
    print(f"7 图表: " + ", ".join(c["id"] for c in P0_BULK_CHARTS))
    print("=" * 70)
    print(f"模式: {'DRY-RUN' if ctx.dry_run else 'WRITE'}")
    print()

    # 1. domain/mod.rs (全替换)
    modify_or_write(ctx, "crates/domain-report/src/domain/mod.rs", build_domain_mod_rs(), "write")

    # 2. lib.rs (modify: 在 generate_p0 match 块加 7 分支)
    # 留上层 caller (我手写 Edit 调用)
    lib_extension = build_lib_rs_p0_extension()
    lib_extension_marker_path = "/tmp/lib_p0_extension.rs"  # 暂存
    Path("/tmp").mkdir(exist_ok=True)
    Path(lib_extension_marker_path).write_text(lib_extension, encoding="utf-8")
    ctx.warnings.append(f"[INFO] lib.rs extension (7 分支) saved to {lib_extension_marker_path}")
    ctx.warnings.append(f"[INFO] 用 Edit 工具粘贴到 crates/domain-report/src/lib.rs generate_p0 match 块")

    # 3. chart-data-schema.ts (modify: 追加 7 interface)
    schema_extension = build_chart_data_schema_extension()
    schema_ext_path = "/tmp/chart_data_schema_extension.ts"
    Path(schema_ext_path).write_text(schema_extension, encoding="utf-8")
    ctx.warnings.append(f"[INFO] chart-data-schema.ts extension saved to {schema_ext_path}")

    # 4. zh-CN.json (modify: 合并 7 keys)
    zh_ext = build_zh_cn_extension()
    zh_ext_path = "/tmp/zh_cn_extension.json"
    Path(zh_ext_path).write_text(json.dumps(zh_ext, indent=2, ensure_ascii=False), encoding="utf-8")
    ctx.warnings.append(f"[INFO] zh-CN.json extension saved to {zh_ext_path}")

    # 5. frontend/src/components/charts/shared/ChartFrame.tsx 不变 (已通用)

    # 6. 总结
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
    print()
    print("⚠️  本脚本只生成 1 个文件 (domain/mod.rs)")
    print("⚠️  其他 3 个文件 (lib.rs / chart-data-schema.ts / zh-CN.json) 需用 Edit 工具手写")
    print("⚠️  7 图表 unique c0X.rs + test.rs + Chart0X.tsx 留 Mavis 7 commits 手写")
    print("=" * 70)

    if ctx.errors:
        return 1

    if args.verify and not ctx.dry_run:
        print()
        print("=" * 70)
        print("守门: cargo check")
        print("=" * 70)
        try:
            r = subprocess.run(
                ["cargo", "check", "--workspace", "--lib", "-p", "domain-report"],
                cwd=str(ROOT), capture_output=True, text=True, timeout=300,
            )
            print(f"exit: {r.returncode}")
            if r.returncode != 0:
                print(r.stderr[-2000:])
                return 1
        except subprocess.TimeoutExpired:
            print("timeout (300s)")
            return 1
        except FileNotFoundError:
            print("cargo not found, skip")

    return 0


if __name__ == "__main__":
    sys.exit(main())
