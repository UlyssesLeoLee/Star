#!/usr/bin/env python3
"""
inter_collab_refine.py
======================
为 24 份 domain spec (docs/specs/domain-*.md) 批量加 "与其他 domain 协作" 一节

数据源: docs/basic-design.md v0.16 §3.2.9 22 domain contact face 表
处理: 解析 basic-design §3.2.9 表格, 按 domain name 提取该 domain 涉及的接触面
输出: 每份 spec 末尾追加 "## 15. 与其他 domain 协作 (v0.16 协作细化新增)" 一节

Per: AGENTS.md §5 v0.6 + 2026-09-01 14:38 JST 模块间协作细化 (A + L3 + doc-only)
守门 #9: 脚本生成的 markdown 模板确定性 + 可 git log -p --follow 实证
"""
import re
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")
BASIC_DESIGN = REPO_ROOT / "docs/basic-design.md"
SPECS_DIR = REPO_ROOT / "docs/specs"

# 24 份 domain spec (per basic-design §6 + 排除 supporting crate)
DOMAIN_SPECS = [
    "identity", "tenant", "workspace", "project", "work-item", "workflow",
    "board", "planning", "permission", "comment", "relation", "collaboration",
    "automation", "integration", "scm", "development", "context", "worktree",
    "agent", "feedback", "validation", "audit", "search", "notification",
    "local-runtime",  # 25th (per AGENTS.md §6 22 logical + local-runtime + others = 25+)
]

# domain 名称到 spec 文件名 / 表格标记映射
DOMAIN_TABLE_MARKERS = {
    "identity": "**identity**", "tenant": "**tenant**", "workspace": "**workspace**",
    "project": "**project**", "work-item": "**work-item**",
    "workflow": "**workflow**", "board": "**board**", "planning": "**planning**",
    "permission": None,  # §3.2.8 横切综述
    "comment": "**comment**", "relation": "**relation**",
    "collaboration": "**collaboration**", "automation": "**automation**",
    "integration": "**integration**", "scm": "**scm**",
    "development": "**development**",
    "context": "**context**",  # 已在 §3.2.4 详表
    "worktree": "**worktree**",  # 已在 §3.2.2 详表
    "agent": "**agent**",  # 已在 §3.2.3 详表
    "feedback": "**feedback**",  # 已在 §3.2.5 详表
    "validation": "**validation**",  # 已在 §3.2.6 详表
    "audit": None,  # §3.2.8 横切综述
    "search": "**search**",  # 单独
    "notification": "**notification**",  # 单独
    "local-runtime": "**local-runtime**",
}

def parse_basic_design_contact_faces():
    """解析 basic-design.md §3.2.9 contact face 表,返回 list of dict"""
    text = BASIC_DESIGN.read_text(encoding="utf-8")
    # 找到 §3.2.9 起始位置
    start = text.find("#### 3.2.9")
    if start < 0:
        raise RuntimeError("未找到 basic-design §3.2.9")
    # 找到下一个 ## 或 ### (即 §3.3 起始)
    end = text.find("### 3.3", start)
    if end < 0:
        end = text.find("## 4.", start)
    section = text[start:end]

    rows = []
    # 匹配表格行: | **src** | **dst** | mode | point |
    for line in section.split("\n"):
        if not line.startswith("|"):
            continue
        # 跳过表头 / 分隔
        if "---" in line:
            continue
        if "Domain" in line and "源" in line:
            continue
        # split by '|', strip, remove ** markdown bold
        cells = [c.strip().replace("**", "") for c in line.split("|")]
        # cells = ['', 'src', 'dst', 'mode', 'point', '']
        if len(cells) < 5:
            continue
        src = cells[1].strip()
        dst = cells[2].strip()
        mode = cells[3].strip()
        point = cells[4].strip()
        if not src or not dst:
            continue
        # 去掉 src 末尾的 (单独) 脚注
        src = re.sub(r"\s*\(.*?\)\s*$", "", src).strip()
        rows.append({"src": src, "dst": dst, "mode": mode, "point": point})
    return rows


def filter_for_domain(rows, domain):
    """筛选出涉及该 domain 的接触面 (作为 src 或 dst)"""
    out = []
    for r in rows:
        if r["src"] == domain or r["dst"] == domain:
            out.append(r)
    return out


def render_collaboration_section(domain, rows):
    """生成 '与其他 domain 协作' markdown 章节"""
    if not rows:
        return None
    lines = []
    lines.append("")
    lines.append("## 15. 与其他 domain 协作 (v0.16 协作细化新增)")
    lines.append("")
    lines.append(f"per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md) + [spec/saga/01 v0.2 SagaCoordinationRole](../../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md),本节定义 `{domain}` 与 22 domain 中 {len(set(r['src'] for r in rows) | set(r['dst'] for r in rows) - {domain})} 个 domain 的显式接触面。")
    lines.append("")
    lines.append("| 源 Domain | 目标 Domain | 接触方式 | 接触点 |")
    lines.append("|---|---|---|---|")
    # 去重
    seen = set()
    for r in rows:
        key = (r["src"], r["dst"], r["mode"], r["point"])
        if key in seen:
            continue
        seen.add(key)
        lines.append(f"| {r['src']} | {r['dst']} | {r['mode']} | {r['point']} |")
    lines.append("")
    lines.append(f"**接触面统计**: {len(seen)} 条 (v0.16 新增,本 spec 由 `scripts/inter_collab_refine.py` 批量生成)")
    lines.append("")
    lines.append(f"**dual-use 警告** (per AGENTS.md §5 v0.6 + Q1-D 拍板): 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名,Star 仓不建立业务子域↔DDD 映射。本 spec 协作基于 22 domain crate,不通过 5 域绑定推导。")
    lines.append("")
    return "\n".join(lines)


def main():
    print(f"[inter_collab_refine] 解析 {BASIC_DESIGN} §3.2.9 ...")
    rows = parse_basic_design_contact_faces()
    print(f"[inter_collab_refine] 解析 {len(rows)} 行 contact face")

    summary = []
    for domain in DOMAIN_SPECS:
        spec_path = SPECS_DIR / f"domain-{domain}-spec.md"
        if not spec_path.exists():
            print(f"  [SKIP] {domain}: spec 不存在")
            continue
        domain_rows = filter_for_domain(rows, domain)
        section = render_collaboration_section(domain, domain_rows)
        if section is None:
            print(f"  [SKIP] {domain}: 无 contact face")
            continue
        # 追加到 spec 末尾
        existing = spec_path.read_text(encoding="utf-8")
        if "## 15. 与其他 domain 协作" in existing:
            print(f"  [SKIP] {domain}: 已包含 §15 协作节")
            continue
        new_content = existing.rstrip() + "\n" + section
        spec_path.write_text(new_content, encoding="utf-8")
        summary.append((domain, len(domain_rows), spec_path))
        print(f"  [OK]   {domain}: 加 {len(domain_rows)} 条 contact face")

    print(f"\n[inter_collab_refine] 完成: {len(summary)} 份 spec 已加 §15 协作节")
    print("\n=== Summary ===")
    for d, n, p in summary:
        print(f"  {d:20s}  {n:3d} contact face  →  {p.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
