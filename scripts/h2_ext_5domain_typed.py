#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""scripts/h2_ext_5domain_typed.py — H2-EXT 5 domain (comment/identity/project/tenant/work-item)
as_uuid Copy 语义一致性验证 + 字段扩展验证 (per WBS §14.17 H2-EXT final + brief v0.55).

per 守门 #1 v25 (CI 单 crate 模式) + 守门 #9 v19 (Mavis 自驱) + 守门 #19 (agent 交互 Python 化) +
守门 #9 v20 (子代理 dispatch 必先 brief, 跨域字段扩展参考 HANDOFF-ST-001 v0.5 §5.1).

跟 H3 commit `e44c934` 同模式: 5 domain 强类型 ID 宏 `as_uuid` 返回 Uuid (Copy) + 跨域字段
(workspace_ids/tenant_policy_id) 在 star-context/actor.rs 已扩展.

本脚本是 idempotent 验证脚本: 跟 H3 `scripts/h3_as_uuid_fix.py` 同 pattern, 跑一次输出验证结果,
不修改源码 (因 H2-EXT 5 domain 改造已在 main 落地 per commit 83b02ce + 8958302 + 9d08f80 + b6f6e2a + 7f611b0).

检查项 (5 域 × 2):
  A. `define_uuid_id!` 宏里 `as_uuid` 返回 `uuid::Uuid` (Copy), 不可 `&uuid::Uuid`
  B. 跨域字段扩展:
     - star-context/actor.rs: tenant_policy_id: Option<Uuid> + workspace_ids: Vec<Uuid> 字段
     - star-context/actor.rs: is_in_workspace(workspace_id) + has_tenant_policy() helper

用法:
    python scripts/h2_ext_5domain_typed.py
    # exit 0 = 5 域全过; exit 1 = 有 issue

约束 (per 守门 #1 v25 单 crate 模式):
  - 不修改源码 (只 verify)
  - 标准库 only (re / pathlib / sys)
  - 5 域验证都跑, 任一不通过 -> exit 1

已知缺口 (per docs/automation-design.md §7):
  1. domain-identity / domain-project / domain-tenant 的 entity struct 内部仍用强类型 ID
     (UserId/DeviceId/TenantId 等), 不在本脚本验证范围 (per HANDOFF v0.5 §5.1 #5 已实证
     "device_id String 删掉 context.rs + port/service dead import 0.05M 跨 session 续做")
  2. 跨 domain service.rs 改造 (feedback/validation/integration ~150+ call sites) 跨 session 续做
     (per §5 已知缺口 + WBS §14.16 H2-3-SVC 维持 v0.26 done 状态)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(r"D:\Star")

# 5 域 (per HANDOFF v0.5 §5.1)
DOMAINS = [
    "domain-comment",
    "domain-identity",
    "domain-project",
    "domain-tenant",
    "domain-work-item",
]

# H3 pattern: 不可出现 `&uuid::Uuid` (会触发 copy not Copy)
BAD_OLD_PATTERN = re.compile(
    r"pub fn as_uuid\(&self\) -> &uuid::Uuid\s*\{\s*&self\.0\s*\}"
)

# H3 修正后: 返回 Uuid (Copy), 兼容 `uuid::Uuid` 跟 `Uuid` 两种 import path
GOOD_NEW_PATTERN = re.compile(
    r"pub fn as_uuid\(&self\) -> (?:uuid::)?Uuid\s*\{\s*self\.0\s*\}"
)

# star-context 跨域字段 (per HANDOFF v0.5 §5.1)
ACTOR_RS = ROOT / "crates" / "star-context" / "src" / "actor.rs"
EXPECTED_ACTOR_FIELDS = [
    "tenant_policy_id",
    "workspace_ids",
]
EXPECTED_ACTOR_HELPERS = [
    "is_in_workspace",
    "has_tenant_policy",
]


def check_domain_as_uuid(domain: str) -> tuple[bool, str]:
    """检查 domain 的 as_uuid 宏是否已统一为返回 Uuid (Copy).

    Returns: (ok, detail)
    """
    src_dir = ROOT / "crates" / domain / "src"
    candidates = [src_dir / "macros.rs", src_dir / "lib.rs"]
    found_files = []
    for f in candidates:
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        # 检查旧模式 (不能存在)
        bad_matches = BAD_OLD_PATTERN.findall(text)
        # 检查新模式 (至少 1 处)
        good_matches = GOOD_NEW_PATTERN.findall(text)
        # 检查 define_uuid_id! 宏 (至少 1 处)
        has_macro = "define_uuid_id!" in text
        found_files.append((f, has_macro, len(bad_matches), len(good_matches)))

    if not found_files:
        return False, f"{domain}: no macros.rs/lib.rs found"

    if any(bad > 0 for _, _, bad, _ in found_files):
        bad_detail = ", ".join(
            f"{f.name}({bad} bad)" for f, _, bad, _ in found_files if bad > 0
        )
        return False, f"{domain}: BAD old `&uuid::Uuid` pattern still present in {bad_detail}"

    macros_found = sum(1 for _, has_macro, _, _ in found_files if has_macro)
    if macros_found == 0:
        return False, f"{domain}: no `define_uuid_id!` macro found"

    good_total = sum(good for _, _, _, good in found_files)
    if good_total == 0:
        return False, f"{domain}: no new `as_uuid() -> uuid::Uuid` pattern found"

    return True, f"{domain}: {macros_found} macro file(s), {good_total} as_uuid() match (Copy)"


def check_actor_context() -> tuple[bool, str]:
    """检查 star-context/actor.rs 跨域字段 + helper."""
    if not ACTOR_RS.exists():
        return False, f"{ACTOR_RS} not found"

    text = ACTOR_RS.read_text(encoding="utf-8")
    missing_fields = [f for f in EXPECTED_ACTOR_FIELDS if f not in text]
    missing_helpers = [h for h in EXPECTED_ACTOR_HELPERS if h not in text]

    if missing_fields or missing_helpers:
        return False, (
            f"actor.rs: missing fields={missing_fields}, "
            f"missing helpers={missing_helpers}"
        )

    return True, f"actor.rs: {len(EXPECTED_ACTOR_FIELDS)} fields + {len(EXPECTED_ACTOR_HELPERS)} helpers OK"


def main() -> int:
    print("=" * 60)
    print("H2-EXT 5 domain as_uuid Copy 语义一致性 + 跨域字段验证")
    print("per WBS §14.17 H2-EXT final + brief v0.55")
    print("=" * 60)
    print()

    domain_results = []
    for d in DOMAINS:
        ok, detail = check_domain_as_uuid(d)
        domain_results.append((d, ok, detail))
        marker = "OK  " if ok else "FAIL"
        print(f"  [{marker}] {detail}")

    print()
    actor_ok, actor_detail = check_actor_context()
    actor_marker = "OK  " if actor_ok else "FAIL"
    print(f"  [{actor_marker}] {actor_detail}")
    print()

    # 总结
    all_ok = all(ok for _, ok, _ in domain_results) and actor_ok
    total = len(domain_results) + 1
    passed = sum(1 for _, ok, _ in domain_results if ok) + (1 if actor_ok else 0)

    print("=" * 60)
    if all_ok:
        print(f"RESULT: ALL OK ({passed}/{total} verified)")
        print("H2-EXT 5 域 + star-context 跨域字段 全部就绪")
        return 0
    else:
        print(f"RESULT: FAIL ({passed}/{total} passed)")
        failed = [d for d, ok, _ in domain_results if not ok]
        if not actor_ok:
            failed.append("actor.rs")
        print(f"failed: {failed}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
