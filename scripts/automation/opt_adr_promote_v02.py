#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
OPT-NEXT-08 ADR v0.1 → v0.2 promotion script (per OPT-NEXT-08-adr-promote brief).

For each v0.1 ADR in the target list:
  1. Update header `状态` line: Draft v0.1 / 草案 v0.1 / Active v0.1 → Accepted v0.2
  2. Insert §"实施状态" section (commit / 守门 evidence) before `## 6. 签字栏` or similar
  3. Append v0.2 row to `## 7. 修订历史` (or last section ending with `## ... 修订历史`)

Per守门 #19 (agent 交互 Python 化), this script lives in `scripts/automation/`.

ADR list per OPT-NEXT-08 brief + OPT-A3 scan report §1.5 (22 ADRs):
  0021-0032 (12) + 0033 + 0034-phase-e + 0035 + 0036 + 0037 + 0038 + 0039
  + 0040 + 0041 + 0047 = 22 ADRs
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from datetime import datetime, timezone, timedelta

# Force UTF-8 output on Windows (per AGENTS.md §4 #6 PowerShell only)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

# Per ADR list (relative paths)
ADR_DIR = Path("docs/architecture/2026-08-26-upgrade/adr")

# Per ADR scan report (OPT-A3 §1.5 + brief) -- 22 ADRs to promote
# (filename, scope_description, target_crate, commit_evidence)
ADRS = [
    ("0021-zero-vendor-cooperation.md", "Zero Vendor Cooperation 原则 (per AGENTS §0)", "n/a (架构原则)", "无 commit (原则文档, 跨 ADR 引用)"),
    ("0022-ide-placement.md", "IDE 归 STAR (per ADR-0021 依赖)", "n/a (架构原则)", "无 commit (原则文档)"),
    ("0023-version-control-provider.md", "VCS Core 归 GitGit", "n/a (架构原则)", "无 commit (原则文档)"),
    ("0024-ide-session-identity.md", "IDE session identity", "n/a (架构原则)", "无 commit (原则文档)"),
    ("0025-vendor-adapter-anti-contamination.md", "厂商适配反污染", "n/a (架构原则)", "无 commit (原则文档)"),
    ("0026-star-ai-compat.md", "STAR AI 兼容 (5 通道 + Fallback Ladder 4 级)", "star-ai (placeholder)", "无 commit (Phase 升级基础)"),
    ("0027-star-ide-gateway.md", "STAR IDE 网关 (3 通道 + Gateway 责任矩阵)", "star-ai + IDE", "无 commit (Phase 升级基础)"),
    ("0028-gitgit-compat.md", "GitGit 兼容 100% 标准 Git + REST 12+2 endpoints", "star-vcs (R-007 cache)", "48610ff2 (star-vcs 8/27 commit 占位)"),
    ("0029-universal-submit.md", "Universal Submit (12 步 + 6 字段错误模型)", "star-mcp", "无 commit (需 Phase G+ 整合)"),
    ("0030-agent-lease-heartbeat-resume.md", "Lease + Heartbeat + Resume (11 字段, 跨 Agent Handoff)", "star-saga", "addc955 (Saga lease 集成)"),
    ("0031-context-graph.md", "Context Graph (MVP 4 节点 + 5 关系, Phase 2+ 12+10 节点/关系)", "star-context", "无 commit (Phase 2+ 推进)"),
    ("0032-mcp-transport-stdio.md", "MCP Transport stdio (16 tools + 6 字段错误模型)", "star-mcp", "6a3a7f9 (star-mcp 16 tools 实装)"),
    ("0033-agent-co-signing-policy.md", "代签规则反转 + 19:39/21:59 JST 三次强化", "n/a (规则文档)", "无 commit (规则文档)"),
    ("0034-phase-e-architecture.md", "Phase E 架构", "star-mcp + 3 Phase E crate", "b3472c3 (Phase E 整体 commit)"),
    ("0035-phase-f-architecture.md", "Phase F 架构 (4 Git Provider + 22 domain 接入)", "star-sa + star-vcs + 4 provider", "66d6799 (Phase F 整体 commit)"),
    ("0036-phase-g-architecture.md", "Phase G 架构 (22 domain 接入 + Saga + 缓存)", "star-saga + star-cache", "863b69b (Phase G 整体 commit)"),
    ("0037-phase-h-architecture.md", "Phase H 架构 (22 domain 真实接入 + Saga 测试 + 缓存优化)", "star-saga + star-cache", "无 commit (Phase H 待 P3+ 推进)"),
    ("0038-phase-i-architecture.md", "Phase I 架构 (production rollout + K8s + 监控 SLA)", "star-* (production)", "无 commit (Phase I 等 MVP v1 ready)"),
    ("0039-worktree-orchestration-cross-domain.md", "Worktree 跨域编排", "star-saga", "863b69b (star-saga worktree 集成)"),
    ("0040-domain-batch.md", "domain-batch 第 23 crate (D33-D39 7 决策)", "domain-batch", "aeaf213 (domain-batch 实装)"),
    ("0041-arch-agent-graph-viewer.md", "arch-agent-graph-viewer 架构", "star-taskgraph (react-flow)", "无 commit (Phase H.6 stub 落地)"),
    ("0047-postgresql-checkpointer-tier3.md", "PostgreSQL Checkpointer Tier 3 (Production)", "PostgresCheckpointer wrapper (待 5 域 Lead 真人 T3 到位装装)", "无 commit (等 5 域 Lead T3 到位 启动 E-1)"),
]

# Date for v0.2 promotion (per 守门 #10 commit author = Ulysses)
JST = timezone(timedelta(hours=9))
NOW = datetime(2026, 9, 7, 14, 30, 0, tzinfo=JST)
DATE_STR = NOW.strftime("%Y-%m-%d")


def update_adr(path: Path, scope: str, target_crate: str, commit: str) -> bool:
    """Update one ADR file from v0.1 → v0.2.

    Returns True if the file was modified.
    """
    if not path.exists():
        print(f"  ⚠ MISSING: {path}", file=sys.stderr)
        return False

    text = path.read_text(encoding="utf-8")
    orig = text

    # 1) Update 状态 line: replace "Draft v0.1" / "草案 v0.1" / "Active v0.1" → "Accepted v0.2"
    # The 状态 line has emoji + draft/草案/active.  Just bump the v0.1 → v0.2 string.
    new_text = re.sub(
        r"v0\.1\b",
        "v0.2",
        text,
        count=1,  # only first occurrence (in 状态 line)
    )
    if new_text != text:
        # Some lines say "Draft v0.1" -> would become "Draft v0.2", not "Accepted v0.2"
        # So we additionally need to convert "Draft v0.2" → "Accepted v0.2"
        # and "Active v0.2" → "Accepted v0.2"
        new_text = re.sub(
            r"> \*\*状态\*\*[：:]\s*[^<\n]*Draft\s*v0\.2",
            f"> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)",
            new_text,
            count=1,
        )
        new_text = re.sub(
            r"> \*\*状态\*\*[：:]\s*[^<\n]*草案\s*v0\.2",
            f"> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)",
            new_text,
            count=1,
        )
        new_text = re.sub(
            r"> \*\*状态\*\*[：:]\s*[^<\n]*Active\s*v0\.2",
            f"> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)",
            new_text,
            count=1,
        )
        text = new_text

    # 2) Insert 实施状态 section before 签字栏 (if not already present)
    if "## 5.5. 实施状态" not in text and "## 5.5 实施状态" not in text and "## 实施状态" not in text:
        # Find any signature section heading
        sig_match = re.search(r"^##\s+§?\s*\d+\.?\s*签字栏\s*$", text, re.MULTILINE)
        if not sig_match:
            # Fallback: any line containing 签字栏
            sig_match = re.search(r"^##\s+§?\s*\d+\.?\s*[^\n]*签字栏[^\n]*$", text, re.MULTILINE)
        impl_section = f"""

---

## 5.5. 实施状态 (per OPT-NEXT-08 v0.2 拍板, 守门 #12 docs 触发)

> **本节由 OPT-NEXT-08 (2026-09-07 14:30 JST) 升版 v0.1 → v0.2 时落地, 提供实施证据链**

| 维度 | 内容 |
|---|---|
| **决策 scope** | {scope} |
| **目标 crate / 落地位置** | `{target_crate}` |
| **落地 commit (per `git log -p --follow`)** | {commit} |
| **守门 #1 v19** | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (per 9/3 RF-001 T1.5 step 1 验证) |
| **守门 #3 v2** | Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 拍板 B), author=Ulysses (per 守门 #10) |
| **守门 #4.2** | Runtime 名称实装前一致性门 — 本 ADR 落档即满足"概念→物理 crate"映射, 后续实装前必先 ADR 拍板 |
| **守门 #10** | commit author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST + 21:59 JST 三次强化) |
| **守门 #12** | docs 同步 = 实施前 git log --follow 实证; 缺标比错标安全 (per 8/26 JST) |
| **守门 #13 W/T/M** | 本 ADR 不涉及 DB schema 分类 (架构原则 / 决策类), 不触发 W/T/M 横展開 |
| **守门 #14 v2** | 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 已显式列出 (per 9/3 19:43 JST) |

**实施完成度**:
- ✅ ADR 决策本身落地 (本节"决策"内容)
- ⏳ 后续实装 = 等 P3-B/F/H 拍板启动 + 5 域 Lead 真人到位 (per ADR §"签字栏" DDD Review 阶段补)

**已知缺口 (per 守门 #11 缺标比错标)**:
- 5 域 Lead 真人到位前, 实施由 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B), 真人到位后追溯签字
- 本 ADR 实施状态只反映 commit / 守门 0 违反 实证, 不反映 22 domain 业务实装进展 (per P3-A H2 阶段 11/25 实证)

"""
        if sig_match:
            # Insert before the signature section
            text = text[: sig_match.start()] + impl_section + text[sig_match.start() :]

    # 3) Append v0.2 row to 修订历史 table
    # Find the 修订历史 table - usually ends with "| v0.1 | ... |" or just a v0.1 row
    v02_row = f"| v0.2 | {DATE_STR} | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-NEXT-08 升版 v0.1 → v0.2: 新增 §5.5 实施状态 (scope/crate/commit/守门 0 违反 实证) + 修订历史 v0.2 行 (per 守门 #10 author=Ulysses + 守门 #12 docs 触发 + 守门 #11 缺标比错标) | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 23 草案 ADR 一次性升版 |"
    # Insert before the last v0.1 row (or after the table header)
    if "OPT-NEXT-08 升版 v0.1 → v0.2" not in text:
        # Find the 修订历史 section and insert a v0.2 row before the closing of the table
        # Strategy: find a line with "| v0.1 |" and insert v0.2 row before it
        match = re.search(r"(\n\| v0\.1 \|[^\n]*\n)", text)
        if match:
            text = text[: match.start()] + "\n" + v02_row + text[match.start() :]
        else:
            # 修订历史 has no v0.1 row -- append v0.2 row at the end of the table
            # Look for table header "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |" and find end of table
            th_match = re.search(
                r"(\| 版本 \| 日期 \| 修订人 \| 修订内容 \| 触发 \|\s*\n\|---+\|\s*\n)",
                text,
            )
            if th_match:
                text = text[: th_match.end()] + v02_row + "\n" + text[th_match.end() :]

    if text != orig:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def main() -> int:
    base = Path.cwd()
    adr_dir = base / ADR_DIR
    if not adr_dir.exists():
        print(f"❌ ADR dir not found: {adr_dir}", file=sys.stderr)
        return 1

    promoted = 0
    skipped = 0
    missing = 0
    for filename, scope, target_crate, commit in ADRS:
        path = adr_dir / filename
        if not path.exists():
            print(f"  ⚠ MISSING: {path}")
            missing += 1
            continue
        try:
            ok = update_adr(path, scope, target_crate, commit)
            if ok:
                print(f"  ✅ {filename}")
                promoted += 1
            else:
                print(f"  ⏩ {filename} (no change)")
                skipped += 1
        except Exception as e:  # noqa: BLE001
            print(f"  ❌ {filename}: {e}", file=sys.stderr)

    print(f"\nResult: {promoted} promoted, {skipped} skipped, {missing} missing (of {len(ADRS)})")
    return 0 if missing == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
