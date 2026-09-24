#!/usr/bin/env python3
# _lib_mock_switch.py - Star Mock Project L1 cluster_switch reader (per ULYS-190 §4.1 brief v0.1)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-190 v0.2 approved (reply 01a0d073 2026-09-23 22:46 JST 「a」) + §4.1 brief v0.1
#       + reply 01a0d081 2026-09-23 23:02 JST 「推进」派工触發 T1
# 守门: #5 (no secret leak) + #6 (中文默认) + #7 (0 unsafe) + #10 (author=Ulysses) +
#       #11 (缺标比错标) + #13 (W/T/M 派生: cluster_switch = Master SCD-2) +
#       #14v4 (Mavis 审核) + #15 (1 sub-agent 1 切点) + #19v19 (Python 化) +
#       #20 (sub-agent brief 落档) + #24 (vendor 中立 = stdlib only)
#
# 用途: 给 fixtures / 回归测试 / LLM agent 调, 读 .mock-cluster.json + 校验 .aci.json 对齐 +
#       拼接 mock_switch_trace 字段给 ACI emit (per ULYS-190 §3.4 + design-analysis §3.1).
#
# 子命令:
#   is-enabled           --cluster-config <path>             (exit 0=enabled, 1=disabled, 2=error)
#   get-mode             --cluster-config <path>             (stdout: offline|passthrough|proxy)
#   trace                --cluster-config <path>             (stdout: mock_switch_trace 字符串)
#   validate-compat      --cluster-config <path> --aci-config <path> (exit 0=OK, 3=mismatch)
#
# 输出: 0 exit = OK, 1+ exit = FAIL; 行格式 `LABEL=value` 给 bash 解析.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Optional

# ---------------------------------------------------------------------------
# 常量
# ---------------------------------------------------------------------------

MOCK_CLUSTER_VERSION = "0.1.0-draft"
DEFAULT_PROJECT = "star-flash-mock"
VALID_MODES = {"offline", "passthrough", "proxy"}

# Required fields per design-analysis §3.1
CLUSTER_REQUIRED_FIELDS = (
    "cluster_version",
    "project",
    "cluster_id",
    "enabled",
    "mode",
    "aci_compat_version",
)


class MockSwitchError(Exception):
    """Mock switch reader 通用错误."""


class ClusterConfigError(MockSwitchError):
    """.mock-cluster.json 读取 / 解析错误."""


class AciCompatVersionMismatch(MockSwitchError):
    """.aci.json 跟 .mock-cluster.json aci_compat_version 不一致."""


# ---------------------------------------------------------------------------
# Reader class
# ---------------------------------------------------------------------------


class MockSwitchReader:
    """读 .mock-cluster.json + 校验 .aci.json 对齐 + 拼接 mock_switch_trace.

    Usage::

        r = MockSwitchReader(Path("tools/star-flash-mock/.mock-cluster.json"))
        if r.is_cluster_enabled():
            trace = r.build_trace()  # e.g. "cluster.enabled=true, cluster.mode=offline"
        r.validate_aci_compat(Path("tools/star-flash-mock/.aci.json"))
    """

    def __init__(self, cluster_config_path: Path) -> None:
        self.cluster_config_path = cluster_config_path
        self._config = _load_cluster_config(cluster_config_path)
        _validate_cluster_config(self._config, cluster_config_path)

    # ----- accessors --------------------------------------------------------

    @property
    def config(self) -> dict[str, Any]:
        return dict(self._config)

    def is_cluster_enabled(self) -> bool:
        return bool(self._config["enabled"])

    def get_mode(self) -> str:
        mode = self._config["mode"]
        if mode not in VALID_MODES:
            raise ClusterConfigError(
                f"mode 必须是 {sorted(VALID_MODES)} 之一, 收到: {mode!r} "
                f"(path={self.cluster_config_path})"
            )
        return mode

    def get_aci_compat_version(self) -> str:
        return str(self._config["aci_compat_version"])

    # ----- trace builder ----------------------------------------------------

    def build_trace(self) -> str:
        """拼接 mock_switch_trace 字符串 (per ULYS-190 §3.4).

        v0.1 简化版: 只含 cluster.enabled + cluster.mode + cluster.aci_compat_version.
        plugins / modules 部分留 placeholder, 待 §4.4 module_switch 扩展补完整.
        """
        parts = [
            f"cluster.enabled={str(self.is_cluster_enabled()).lower()}",
            f"cluster.mode={self.get_mode()}",
            f"cluster.aci_compat_version={self.get_aci_compat_version()}",
            "[TBD plugins section 落地後擴充]",
        ]
        return ", ".join(parts)

    # ----- ACI compat validation -------------------------------------------

    def validate_aci_compat(self, aci_config_path: Path) -> None:
        """校验 .aci.json 的 aci_version 跟 .mock-cluster.json 的 aci_compat_version 一致.

        Raises AciCompatVersionMismatch if inconsistent.
        """
        try:
            aci = json.loads(aci_config_path.read_text(encoding="utf-8"))
        except Exception as e:
            raise ClusterConfigError(
                f"读取 .aci.json 失败: {e} (path={aci_config_path})"
            ) from e
        aci_version = aci.get("aci_version")
        cluster_compat = self.get_aci_compat_version()
        if aci_version != cluster_compat:
            raise AciCompatVersionMismatch(
                f".aci.json aci_version={aci_version!r} 跟 .mock-cluster.json "
                f"aci_compat_version={cluster_compat!r} 不一致 "
                f"(aci_path={aci_config_path}, cluster_path={self.cluster_config_path})"
            )


# ---------------------------------------------------------------------------
# Module-level helpers
# ---------------------------------------------------------------------------


def _load_cluster_config(path: Path) -> dict[str, Any]:
    """读取 + 解析 .mock-cluster.json; 不存在 / 解析失败 raise ClusterConfigError."""
    if not path.exists():
        raise ClusterConfigError(f"cluster_config 文件不存在: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        raise ClusterConfigError(
            f"cluster_config JSON 解析失败: {e} (path={path})"
        ) from e
    if not isinstance(data, dict):
        raise ClusterConfigError(
            f"cluster_config 必须是 JSON object, 收到: {type(data).__name__} (path={path})"
        )
    return data


def _validate_cluster_config(cfg: dict[str, Any], path: Path) -> None:
    """校验必填字段 + 类型; 失败 raise ClusterConfigError."""
    for f in CLUSTER_REQUIRED_FIELDS:
        if f not in cfg:
            raise ClusterConfigError(
                f"cluster_config 缺必填字段: {f!r} (path={path})"
            )
    if not isinstance(cfg["enabled"], bool):
        raise ClusterConfigError(
            f"enabled 必须是 bool, 收到: {type(cfg['enabled']).__name__} (path={path})"
        )
    if cfg["mode"] not in VALID_MODES:
        raise ClusterConfigError(
            f"mode 必须是 {sorted(VALID_MODES)} 之一, 收到: {cfg['mode']!r} (path={path})"
        )


# ---------------------------------------------------------------------------
# CLI subcommands
# ---------------------------------------------------------------------------


def cmd_is_enabled(args: argparse.Namespace) -> int:
    r = MockSwitchReader(Path(args.cluster_config))
    print(f"CLUSTER_ENABLED={str(r.is_cluster_enabled()).lower()}")
    return 0 if r.is_cluster_enabled() else 1


def cmd_get_mode(args: argparse.Namespace) -> int:
    r = MockSwitchReader(Path(args.cluster_config))
    print(f"CLUSTER_MODE={r.get_mode()}")
    return 0


def cmd_trace(args: argparse.Namespace) -> int:
    r = MockSwitchReader(Path(args.cluster_config))
    print(r.build_trace())
    return 0


def cmd_validate_compat(args: argparse.Namespace) -> int:
    r = MockSwitchReader(Path(args.cluster_config))
    try:
        r.validate_aci_compat(Path(args.aci_config))
    except AciCompatVersionMismatch as e:
        print(f"ACI_COMPAT=MISMATCH: {e}", file=sys.stderr)
        return 3
    print(f"ACI_COMPAT=OK (cluster={r.get_aci_compat_version()})")
    return 0


def _build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="_lib_mock_switch.py",
        description="Star Mock L1 cluster_switch reader (per ULYS-190 §4.1)",
    )
    sub = p.add_subparsers(dest="subcommand", required=True)

    common = argparse.ArgumentParser(add_help=False)
    common.add_argument(
        "--cluster-config",
        required=True,
        help="path to .mock-cluster.json (e.g. tools/star-flash-mock/.mock-cluster.json)",
    )

    sp = sub.add_parser("is-enabled", parents=[common], help="exit 0=enabled, 1=disabled")
    sp.set_defaults(func=cmd_is_enabled)

    sp = sub.add_parser("get-mode", parents=[common], help="print cluster mode")
    sp.set_defaults(func=cmd_get_mode)

    sp = sub.add_parser("trace", parents=[common], help="print mock_switch_trace string")
    sp.set_defaults(func=cmd_trace)

    sp = sub.add_parser(
        "validate-compat",
        parents=[common],
        help="validate .aci.json aci_version matches .mock-cluster.json aci_compat_version",
    )
    sp.add_argument(
        "--aci-config",
        required=True,
        help="path to .aci.json (e.g. tools/star-flash-mock/.aci.json)",
    )
    sp.set_defaults(func=cmd_validate_compat)

    return p


def main(argv: Optional[list[str]] = None) -> int:
    parser = _build_argparser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except (ClusterConfigError, MockSwitchError) as e:
        print(f"ERROR={type(e).__name__}: {e}", file=sys.stderr)
        return 2
    except Exception as e:  # pragma: no cover (defensive)
        print(f"ERROR=Unexpected: {type(e).__name__}: {e}", file=sys.stderr)
        return 99


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
