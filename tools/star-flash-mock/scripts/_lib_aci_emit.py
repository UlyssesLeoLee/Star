#!/usr/bin/env python3
# _lib_aci_emit.py - Star Mock Project ACI assertion emitter (per ULYS-191 §4.1.2 brief v0.1)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-191 v0.2 approved (reply 01a0cbf3 2026-09-23 10:48 JST) + brief v0.1
# 守门: #5 (no secret leak) + #6 (中文默认) + #7 (0 unsafe) + #10 (author=Ulysses) +
#       #11 (缺标比错标) + #13 (W/T/M 派生: assertion = Transaction audit) +
#       #14v4 (Mavis 审核) + #19v19 (Python 化) + #24 (vendor 中立 = stdlib only)
#
# 用途: 给 fixtures / 回归测试 / LLM agent 调, 产出 LLM-可读的断言 JSON。
# 对应 _lib_validate.py 风格, 纯 stdlib (json + argparse + datetime + uuid + sys)。
#
# 子命令:
#   emit   --layer L --scope k=v,k=v --expect-type T --expect-value V ...
#                 --status S --severity SEV --reasoning R --output FILE
#   schema (打印 .aci.json 必填字段清单)
#
# 输出: 0 exit = OK, 1+ exit = FAIL; 行格式 `LABEL=value` 给 bash 解析.

from __future__ import annotations

import argparse
import datetime as _dt
import json
import sys
import uuid
from pathlib import Path
from typing import Any, Iterable, Optional

# ---------------------------------------------------------------------------
# 常量: 跟 .aci.json schema 字段对齐
# ---------------------------------------------------------------------------

ACI_VERSION = "0.1.0-draft"
PROJECT = "star-flash-mock"

# Status / severity 取值 (与 .aci.json 中声明一致)
VALID_STATUSES = {"PASS", "FAIL", "WARN", "SKIP"}
VALID_SEVERITIES = {"critical", "high", "medium", "low", "info"}
VALID_LAYERS = {"ut", "it", "st", "e2e"}

# expect/actual value.type 取值 (per .aci.json expect_value_types)
VALID_EXPECT_TYPES = {
    "response_within_ms",
    "response_status_2xx",
    "response_status_4xx_5xx",
    "field_equals",
    "field_exists",
    "field_not_exists",
    "field_in_range",
    "field_greater_than",
    "field_less_than",
    "field_matches_regex",
    "list_length_at_least",
    "list_length_equals",
    "workflow_completes",
    "event_emitted",
    "log_contains",
    "metric_within_threshold",
    "no_resource_leak",
}

# Scope dimension 取值
VALID_SCOPE_DIMS = {
    "project", "module", "domain", "subdomain", "operation", "http_method",
}

REQUIRED_FIELDS = [
    "assertion_id",
    "aci_version",
    "layer",
    "scope",
    "expect",
    "actual",
    "status",
    "severity",
    "reasoning",
    "captured_at",
]


# ---------------------------------------------------------------------------
# 异常
# ---------------------------------------------------------------------------


class AciEmitError(Exception):
    """ACI emit 阶段错误 (用户输入 / schema 不符)."""


class AciValidationError(Exception):
    """ACI assertion 字段验证失败."""


# ---------------------------------------------------------------------------
# AciEmitter class
# ---------------------------------------------------------------------------


class AciEmitter:
    """生成 LLM-可读 ACI assertion 字典 (per .aci.json schema).

    Usage::

        em = AciEmitter(layer="it")
        a = em.build(
            assertion_id="star-flash-mock:guards:g-1",
            scope={"project": "star-flash-mock", "module": "guards", "domain": "admin"},
            expect={"type": "response_within_ms", "value": 2000, "description": "..."},
            actual={"type": "response_within_ms", "value": 5, "description": "..."},
            status="PASS",
            severity="info",
            reasoning="actual 5ms < 2000ms 预期",
        )
        a = em.with_tags(a, tags=["perf", "smoke"])
        em.write(a, Path("/tmp/out.aci.json"))
    """

    def __init__(self, layer: str, project: str = PROJECT) -> None:
        if layer not in VALID_LAYERS:
            raise AciEmitError(
                f"layer 必须是 {sorted(VALID_LAYERS)} 之一, 收到: {layer!r}"
            )
        self.layer = layer
        self.project = project

    # ----- builders --------------------------------------------------------

    def build(
        self,
        *,
        assertion_id: str,
        scope: dict[str, str],
        expect: dict[str, Any],
        actual: dict[str, Any],
        status: str,
        severity: str,
        reasoning: str,
        suggested_fix: Optional[str] = None,
        tags: Optional[list[str]] = None,
        captured_at: Optional[str] = None,
    ) -> dict[str, Any]:
        """构造一条 ACI assertion dict (符合 .aci.json 必填字段)."""
        _validate_assertion_id(assertion_id)
        _validate_scope(scope)
        _validate_expect_or_actual("expect", expect)
        _validate_expect_or_actual("actual", actual)
        _validate_status(status)
        _validate_severity(severity)

        if status == "FAIL" and not reasoning.strip():
            raise AciValidationError(
                "status=FAIL 时 reasoning 必填 (LLM 必读「为什么 fail」)"
            )

        if suggested_fix is not None and status != "FAIL":
            # 软警告: 非 FAIL 状态也允许 suggested_fix (例如 WARN)
            pass

        assertion = {
            "assertion_id": assertion_id,
            "aci_version": ACI_VERSION,
            "layer": self.layer,
            "scope": _with_default_scope(scope, self.project),
            "expect": expect,
            "actual": actual,
            "status": status,
            "severity": severity,
            "reasoning": reasoning,
            "captured_at": captured_at or _now_rfc3339(),
        }
        if suggested_fix is not None:
            assertion["suggested_fix"] = suggested_fix
        if tags:
            assertion["tags"] = list(tags)
        return assertion

    def with_tags(self, assertion: dict[str, Any], tags: list[str]) -> dict[str, Any]:
        """便利方法: 给已构造的 assertion 加 tags."""
        assertion = dict(assertion)
        existing = list(assertion.get("tags", []))
        for t in tags:
            if t not in existing:
                existing.append(t)
        if existing:
            assertion["tags"] = existing
        return assertion

    # ----- serialization ----------------------------------------------------

    def to_json(self, assertion: dict[str, Any], *, indent: int = 2) -> str:
        """序列化 assertion 为 JSON 字符串 (sort_keys=True 便于 diff)."""
        return json.dumps(assertion, indent=indent, sort_keys=True, ensure_ascii=False)

    def write(self, assertion: dict[str, Any], path: Path) -> None:
        """写入 assertion 到 path (UTF-8, ensure_ascii=False, sort_keys)."""
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(self.to_json(assertion) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------------
# Validators
# ---------------------------------------------------------------------------


def _validate_assertion_id(aid: str) -> None:
    if not isinstance(aid, str) or not aid.strip():
        raise AciValidationError("assertion_id 必填且为非空字符串")
    if ":" not in aid:
        raise AciValidationError(
            f"assertion_id 格式应为 <module>:<id>, 收到: {aid!r} "
            "(per .aci.json schema_required_fields + .aci.json 隐含约定)"
        )


def _validate_scope(scope: dict[str, Any]) -> None:
    if not isinstance(scope, dict) or not scope:
        raise AciValidationError("scope 必填且为非空 dict (至少含 1 个 dimension)")
    unknown = set(scope.keys()) - VALID_SCOPE_DIMS
    if unknown:
        raise AciValidationError(
            f"scope 含未声明 dimension: {sorted(unknown)} "
            f"(VALID_SCOPE_DIMS = {sorted(VALID_SCOPE_DIMS)})"
        )
    for k, v in scope.items():
        if not isinstance(v, str) or not v.strip():
            raise AciValidationError(f"scope[{k!r}] 必填且为非空字符串")


def _validate_expect_or_actual(label: str, obj: dict[str, Any]) -> None:
    if not isinstance(obj, dict) or not obj:
        raise AciValidationError(f"{label} 必填且为非空 dict")
    if "type" not in obj:
        raise AciValidationError(f"{label}.type 必填 (expect_value_types 之一)")
    if obj["type"] not in VALID_EXPECT_TYPES:
        raise AciValidationError(
            f"{label}.type={obj['type']!r} 不在 VALID_EXPECT_TYPES 中"
        )
    if "value" not in obj:
        raise AciValidationError(f"{label}.value 必填 (类型由 type 决定)")
    if "description" not in obj or not str(obj["description"]).strip():
        raise AciValidationError(f"{label}.description 必填 (LLM 必读)")


def _validate_status(status: str) -> None:
    if status not in VALID_STATUSES:
        raise AciValidationError(
            f"status={status!r} 不在 VALID_STATUSES={sorted(VALID_STATUSES)}"
        )


def _validate_severity(severity: str) -> None:
    if severity not in VALID_SEVERITIES:
        raise AciValidationError(
            f"severity={severity!r} 不在 VALID_SEVERITIES={sorted(VALID_SEVERITIES)}"
        )


def _with_default_scope(scope: dict[str, str], project: str) -> dict[str, str]:
    """给 scope 默认补 project 字段 (避免每个调用方都重复)."""
    out = dict(scope)
    out.setdefault("project", project)
    return out


def _now_rfc3339() -> str:
    """RFC3339 UTC 时间戳 (per .aci.json field_semantics.captured_at)."""
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _parse_kv_pairs(items: Iterable[str]) -> dict[str, str]:
    """解析 ['k=v', 'k2=v2'] → {'k': 'v', 'k2': 'v2'}.

    支持 k=v 形式 (v 不含 =). 重复 key 抛错.
    """
    out: dict[str, str] = {}
    for item in items:
        if "=" not in item:
            raise AciEmitError(f"scope 形式应为 k=v, 收到: {item!r}")
        k, v = item.split("=", 1)
        if k in out:
            raise AciEmitError(f"scope 重复 key: {k!r}")
        out[k] = v
    return out


def _build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="_lib_aci_emit.py",
        description=(
            "Star Mock Project ACI assertion emitter (per ULYS-191 §4.1.2 brief v0.1). "
            "生成符合 .aci.json schema 的 LLM-可读断言 JSON。"
        ),
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    # schema 子命令
    sub.add_parser("schema", help="打印 .aci.json schema 必填字段清单")

    # emit 子命令
    e = sub.add_parser("emit", help="构造一条 ACI assertion")
    e.add_argument("--layer", required=True, choices=sorted(VALID_LAYERS),
                    help="测试层 (ut/it/st/e2e)")
    e.add_argument("--assertion-id", required=True,
                    help="唯一 ID, 格式 <module>:<id>")
    e.add_argument("--scope", nargs="+", required=True,
                    metavar="k=v", help="scope 维度 (可多个, k=v 形式)")
    e.add_argument("--expect-type", required=True, choices=sorted(VALID_EXPECT_TYPES))
    e.add_argument("--expect-value", required=True,
                    help="expect.value (类型由 --expect-type 决定)")
    e.add_argument("--expect-description", required=True,
                    help="expect.description (LLM 必读)")
    e.add_argument("--actual-type", required=True, choices=sorted(VALID_EXPECT_TYPES))
    e.add_argument("--actual-value", required=True)
    e.add_argument("--actual-description", required=True)
    e.add_argument("--status", required=True, choices=sorted(VALID_STATUSES))
    e.add_argument("--severity", required=True, choices=sorted(VALID_SEVERITIES))
    e.add_argument("--reasoning", required=True,
                    help="自然语言「为什么」(FAIL 时必填)")
    e.add_argument("--suggested-fix", default=None,
                    help="自然语言「怎么修」(FAIL 时建议填)")
    e.add_argument("--tags", default=None,
                    help="逗号分隔 tags (例: perf,kms,timeout)")
    e.add_argument("--captured-at", default=None,
                    help="RFC3339 时间戳 (缺省: now UTC)")
    e.add_argument("--output", required=True, type=Path,
                    help="输出文件路径 (JSON)")

    return p


def cmd_schema(_args: argparse.Namespace) -> int:
    """打印 schema 必填字段."""
    out = {
        "aci_version": ACI_VERSION,
        "project": PROJECT,
        "required_fields": REQUIRED_FIELDS,
        "valid_layers": sorted(VALID_LAYERS),
        "valid_statuses": sorted(VALID_STATUSES),
        "valid_severities": sorted(VALID_SEVERITIES),
        "valid_expect_types": sorted(VALID_EXPECT_TYPES),
        "valid_scope_dims": sorted(VALID_SCOPE_DIMS),
    }
    print(json.dumps(out, indent=2, ensure_ascii=False))
    return 0


def cmd_emit(args: argparse.Namespace) -> int:
    """emit 子命令实现."""
    try:
        scope = _parse_kv_pairs(args.scope)
        tags: Optional[list[str]] = (
            [t.strip() for t in args.tags.split(",") if t.strip()]
            if args.tags else None
        )
        em = AciEmitter(layer=args.layer)
        assertion = em.build(
            assertion_id=args.assertion_id,
            scope=scope,
            expect={
                "type": args.expect_type,
                "value": _coerce_value(args.expect_value),
                "description": args.expect_description,
            },
            actual={
                "type": args.actual_type,
                "value": _coerce_value(args.actual_value),
                "description": args.actual_description,
            },
            status=args.status,
            severity=args.severity,
            reasoning=args.reasoning,
            suggested_fix=args.suggested_fix,
            tags=tags,
            captured_at=args.captured_at,
        )
        em.write(assertion, args.output)
    except (AciEmitError, AciValidationError) as exc:
        print(f"FAIL={exc}", file=sys.stderr)
        return 2
    print(f"OK status={args.status} severity={args.severity} output={args.output}")
    return 0


def _coerce_value(raw: str) -> Any:
    """尽量把 CLI 字符串值解析成 int / float / bool / str (LLM-可读优先)."""
    if raw.lower() in ("true", "false"):
        return raw.lower() == "true"
    try:
        return int(raw)
    except ValueError:
        pass
    try:
        return float(raw)
    except ValueError:
        pass
    return raw


def main(argv: Optional[list[str]] = None) -> int:
    parser = _build_argparser()
    args = parser.parse_args(argv)
    if args.cmd == "schema":
        return cmd_schema(args)
    if args.cmd == "emit":
        return cmd_emit(args)
    parser.error(f"unknown cmd: {args.cmd}")
    return 2  # unreachable


if __name__ == "__main__":
    sys.exit(main())
