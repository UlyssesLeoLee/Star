#!/usr/bin/env python3
# mock_switch_validate.py - 跨项目 Mock Cluster Switch + ACI 校验脚本 (per ULYS-190 §4.5)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-190 v0.2 approved + §4.1 Stage 1 ship + reply 01a0d0e1 「完成所有后续工作」 T1 派工
# 守门: #5+#7+#10+#11+#14v4+#15+#19v19+#24 (vendor 中立 = stdlib only)
#
# 用途: 跨项目 (Star / IDE1.0 / RGS / CATs / IM1.0 / GitGit / Ada) 跑校验:
#   1. 每个项目根存在 .mock-cluster.json (L1 cluster_switch 契约)
#   2. 每个项目 .mock-cluster.json.aci_compat_version 跟 .aci.json.aci_version 一致 (if .aci.json 存在)
#   3. 报告: 跨项目开/关/模式/兼容版本状态聚合
#
# 子命令:
#   validate-all  --projects <json>  (跑全部项目校验, 输出聚合报告)
#   validate-one  --cluster-config <path> [--aci-config <path>]  (单项目校验)
#   report        --projects <json> --output <path>  (生成 markdown 报告)
#
# 输出: 0 exit = 全部 PASS, 1+ exit = 有 FAIL; 行格式 `LABEL=value` 给 bash 解析.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Optional

# ---------------------------------------------------------------------------
# 常量: 7 个项目 (per ULYS-190 v0.2 approved 7 项目, 排除 Xiaoshuo per ULYS-191)
# ---------------------------------------------------------------------------

SUPPORTED_PROJECTS = ("star-flash-mock", "ide1", "rgs", "cats", "im1", "gitgit", "ada")

# 每个项目根路径 (跟 ULYS-191 v0.3 G-ACI-01 盤點结论一致; Ada G-MS-01 已解决)
DEFAULT_PROJECT_ROOTS = {
    "star-flash-mock": "D:/Star/tools/star-flash-mock",
    "ide1": "E:/IDE1.0",
    "rgs": "D:/RustGameServer/tools/rgs-flash-mock",
    "cats": "D:/CATs/crates/cats-mock",
    "im1": "D:/IM1.0/crates/im-testkit",
    "gitgit": "D:/GitGit/apps/gm-console/src/mocks",
    "ada": "D:/Ada/crates/ada-mock",
}

CLUSTER_REQUIRED = (
    "cluster_version",
    "project",
    "cluster_id",
    "enabled",
    "mode",
    "aci_compat_version",
)
VALID_MODES = {"offline", "passthrough", "proxy"}


class ValidateError(Exception):
    """mock-switch-validate 通用错误."""


# ---------------------------------------------------------------------------
# 核心: per-project 校验
# ---------------------------------------------------------------------------


def validate_cluster_config(cluster_path: Path) -> dict[str, Any]:
    """读取 .mock-cluster.json, 返回 {ok, error?, project?, enabled?, mode?, cluster_version?, aci_compat_version?, cluster_path}."""
    result: dict[str, Any] = {"ok": False, "cluster_path": str(cluster_path)}
    if not cluster_path.exists():
        result["error"] = f"cluster_config 不存在: {cluster_path}"
        return result
    try:
        cfg = json.loads(cluster_path.read_text(encoding="utf-8"))
    except Exception as e:
        result["error"] = f"JSON 解析失败: {e}"
        return result
    if not isinstance(cfg, dict):
        result["error"] = "cluster_config 必须是 JSON object"
        return result
    missing = [f for f in CLUSTER_REQUIRED if f not in cfg]
    if missing:
        result["error"] = f"缺必填字段: {missing}"
        return result
    if not isinstance(cfg["enabled"], bool):
        result["error"] = f"enabled 必须是 bool, 收到: {type(cfg['enabled']).__name__}"
        return result
    if cfg["mode"] not in VALID_MODES:
        result["error"] = f"mode 必须是 {sorted(VALID_MODES)} 之一, 收到: {cfg['mode']!r}"
        return result
    result["ok"] = True
    result["project"] = cfg.get("project")
    result["enabled"] = cfg.get("enabled")
    result["mode"] = cfg.get("mode")
    result["cluster_version"] = cfg.get("cluster_version")
    result["aci_compat_version"] = cfg.get("aci_compat_version")
    result["cluster_id"] = cfg.get("cluster_id")
    return result


def validate_aci_compat(cluster_cfg: dict[str, Any], aci_path: Path) -> dict[str, Any]:
    """如果 .aci.json 存在, 校验 aci_version 跟 cluster.aci_compat_version 一致."""
    result: dict[str, Any] = {"aci_path": str(aci_path), "checked": False}
    if not aci_path.exists():
        result["status"] = "SKIP"
        result["reason"] = ".aci.json 不存在 (本项目可能尚未落地 ULYS-191 ACI Stage 1)"
        return result
    try:
        aci = json.loads(aci_path.read_text(encoding="utf-8"))
    except Exception as e:
        result["status"] = "FAIL"
        result["reason"] = f".aci.json JSON 解析失败: {e}"
        return result
    result["checked"] = True
    cluster_compat = cluster_cfg.get("aci_compat_version")
    aci_version = aci.get("aci_version")
    if aci_version != cluster_compat:
        result["status"] = "FAIL"
        result["reason"] = f"aci_version={aci_version!r} != cluster.aci_compat_version={cluster_compat!r}"
    else:
        result["status"] = "OK"
        result["reason"] = f"aci_version={aci_version!r} == cluster.aci_compat_version={cluster_compat!r}"
    return result


# ---------------------------------------------------------------------------
# CLI 子命令
# ---------------------------------------------------------------------------


def cmd_validate_one(args: argparse.Namespace) -> int:
    cluster_path = Path(args.cluster_config)
    cfg = validate_cluster_config(cluster_path)
    if not cfg["ok"]:
        print(f"CLUSTER=FAIL error={cfg.get('error')}", file=sys.stderr)
        return 2
    print(f"CLUSTER=OK project={cfg['project']} enabled={cfg['enabled']} mode={cfg['mode']} cluster_version={cfg['cluster_version']} aci_compat_version={cfg['aci_compat_version']}")
    if args.aci_config:
        aci = validate_aci_compat(cfg, Path(args.aci_config))
        print(f"ACI={aci['status']} {aci['reason']}")
        if aci["status"] == "FAIL":
            return 3
    return 0


def cmd_validate_all(args: argparse.Namespace) -> int:
    roots = json.loads(args.roots_json) if args.roots_json else DEFAULT_PROJECT_ROOTS
    results: list[dict[str, Any]] = []
    fail_count = 0
    for project, root in roots.items():
        cluster_path = Path(root) / ".mock-cluster.json"
        cfg = validate_cluster_config(cluster_path)
        row: dict[str, Any] = {"project": project, "root": root, "cluster_ok": cfg["ok"]}
        if cfg["ok"]:
            row["enabled"] = cfg["enabled"]
            row["mode"] = cfg["mode"]
            row["cluster_version"] = cfg["cluster_version"]
            row["aci_compat_version"] = cfg["aci_compat_version"]
            # 也校验 .aci.json (per Stage 1 §4.5)
            aci_path = Path(root) / ".aci.json"
            aci = validate_aci_compat(cfg, aci_path)
            row["aci_status"] = aci["status"]
            row["aci_reason"] = aci["reason"]
            if aci["status"] == "FAIL":
                fail_count += 1
        else:
            row["cluster_error"] = cfg.get("error")
            fail_count += 1
        results.append(row)
    print(f"TOTAL={len(results)} FAIL={fail_count}")
    for r in results:
        line = f"PROJECT={r['project']} cluster_ok={r['cluster_ok']}"
        if r["cluster_ok"]:
            line += f" enabled={r.get('enabled')} mode={r.get('mode')} cluster_version={r.get('cluster_version')} aci_compat_version={r.get('aci_compat_version')} aci_status={r.get('aci_status')}"
        else:
            line += f" error={r.get('cluster_error')}"
        print(line)
    return 0 if fail_count == 0 else 1


def cmd_report(args: argparse.Namespace) -> int:
    roots = json.loads(args.roots_json) if args.roots_json else DEFAULT_PROJECT_ROOTS
    rows: list[str] = []
    rows.append("# Mock Cluster Switch 跨项目状态报告 (per ULYS-190 §4.5)")
    rows.append("")
    rows.append("| 项目 | 根路径 | cluster_ok | enabled | mode | cluster_version | aci_compat_version | aci_status |")
    rows.append("|---|---|---|---|---|---|---|---|")
    fail = 0
    for project, root in roots.items():
        cluster_path = Path(root) / ".mock-cluster.json"
        cfg = validate_cluster_config(cluster_path)
        if cfg["ok"]:
            aci = validate_aci_compat(cfg, Path(root) / ".aci.json")
            rows.append(f"| {project} | `{root}` | ✅ | {cfg['enabled']} | {cfg['mode']} | {cfg['cluster_version']} | {cfg['aci_compat_version']} | {aci['status']} |")
            if aci["status"] == "FAIL":
                fail += 1
        else:
            rows.append(f"| {project} | `{root}` | ❌ | — | — | — | — | {cfg.get('error')} |")
            fail += 1
    rows.append("")
    rows.append(f"**TOTAL**: {len(roots)} 个项目, **FAIL**: {fail} 个")
    out = "\n".join(rows) + "\n"
    Path(args.output).write_text(out, encoding="utf-8")
    print(f"REPORT_WRITTEN={args.output} fail={fail}")
    return 0 if fail == 0 else 1


def _build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="mock-switch-validate.py",
        description="跨项目 Mock Cluster Switch + ACI 校验 (per ULYS-190 §4.5)",
    )
    sub = p.add_subparsers(dest="subcommand", required=True)

    sp = sub.add_parser("validate-one", help="单项目校验")
    sp.add_argument("--cluster-config", required=True)
    sp.add_argument("--aci-config", default=None)
    sp.set_defaults(func=cmd_validate_one)

    sp = sub.add_parser("validate-all", help="跑全部 7 项目校验")
    sp.add_argument("--roots-json", default=None,
                    help="JSON dict {project: root}, 缺省用 DEFAULT_PROJECT_ROOTS")
    sp.set_defaults(func=cmd_validate_all)

    sp = sub.add_parser("report", help="生成 markdown 报告")
    sp.add_argument("--roots-json", default=None)
    sp.add_argument("--output", required=True)
    sp.set_defaults(func=cmd_report)

    return p


def main(argv: Optional[list[str]] = None) -> int:
    parser = _build_argparser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except ValidateError as e:
        print(f"ERROR=ValidateError: {e}", file=sys.stderr)
        return 2
    except Exception as e:  # pragma: no cover (defensive)
        print(f"ERROR=Unexpected: {type(e).__name__}: {e}", file=sys.stderr)
        return 99


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
