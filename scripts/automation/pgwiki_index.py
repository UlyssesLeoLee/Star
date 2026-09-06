# pgwiki_index.py
# 抓取 D:/Star 整个工程 + DB 设计,生成 Obsidian vault 到 docs/wiki/pgwiki/
# Per 守门 #19 Python 化 + 守门 #1 v20v25: cargo check --workspace -j 4 不退化
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
# Date: 2026-09-06
#
# 用法:
#   python scripts/automation/pgwiki_index.py            # 全量生成
#   python scripts/automation/pgwiki_index.py --check    # 验证已有产物不退化
#
# 拓扑层:
#   10-workspace/   物理目录(48 crate + frontend + tools + scripts + docs)
#   20-database/    25 schema × 93 table (per docs/data-design/ipa-detail/)
#   30-architecture/ 5 view × 27 ADR
#   40-crosscutting/ 跨切关系(依赖 / schema 映射 / table-ADR / 守门)
#
# 边类型(全部 frontmatter,Obsidian 用 [[wikilink]] 渲染):
#   parent:        层级父子
#   depends_on:    crate -> crate
#   owns_table:    crate -> table (schema 名匹配)
#   part_of_view:  crate -> arch view
#   references_adr:任意 node -> ADR
#   relates_to:    通用关联

import argparse
import json
import os
import re
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

# ---------- 路径常量(per 当前 worktree feat-auto-20260906-a6216256) ----------
REPO_ROOT = Path(r"D:/Star")
PGWIKI_ROOT = REPO_ROOT / "docs" / "wiki" / "pgwiki"
WORKSPACE_CRATES = REPO_ROOT / "crates"
FRONTEND_ROOT = REPO_ROOT / "frontend"
TOOLS_ROOT = REPO_ROOT / "tools"
SCRIPTS_ROOT = REPO_ROOT / "scripts"
DOCS_ROOT = REPO_ROOT / "docs"
DB_DETAIL_ROOT = DOCS_ROOT / "data-design" / "ipa-detail"
DB_TABLES_ROOT = DB_DETAIL_ROOT / "tables"
CARGO_TOML = REPO_ROOT / "Cargo.toml"
ADR_ROOT = DOCS_ROOT / "architecture" / "2026-08-26-upgrade" / "adr"
ARCH_VIEWS = DOCS_ROOT / "architecture"

# ---------- 节点 frontmatter 模板 ----------
def fm_block(**kw) -> str:
    """生成 obsidian-friendly YAML frontmatter(全字符串值,避免编码问题)"""
    lines = ["---"]
    for k, v in kw.items():
        if isinstance(v, list):
            lines.append(f"{k}:")
            for item in v:
                lines.append(f"  - {item}")
        else:
            # 用双引号包字符串,YAML 解析更稳
            s = str(v).replace('"', '\\"')
            lines.append(f'{k}: "{s}"')
    lines.append("---")
    return "\n".join(lines)


# ============================================================
# 1. 抓 Cargo.toml 解析 crate 列表 + 依赖
# ============================================================
def parse_cargo_toml():
    """极简 TOML 解析:仅取 [[workspace.members]] + 每个 crate 的 [dependencies]
    不引入 toml 库,减少依赖。Star 仓 Cargo.toml 格式固定(per 守门: 48 crate)。
    """
    text = CARGO_TOML.read_text(encoding="utf-8")
    # 提取 members 列表
    m = re.search(r"members\s*=\s*\[(.*?)\]", text, re.DOTALL)
    members_raw = m.group(1) if m else ""
    # 过滤注释 + 取字符串
    members = []
    for line in members_raw.splitlines():
        line = line.strip().rstrip(",").strip()
        if not line or line.startswith("#"):
            continue
        # 形如 "crates/domain-tenant"
        m2 = re.match(r'"crates/([\w\-]+)"', line)
        if m2:
            members.append(m2.group(1))
    return members


def parse_crate_deps(crate_name: str):
    """读 crates/<name>/Cargo.toml 的 [dependencies] 段
    只取 path = "../xxx" 形式(workspace 内依赖),跨过外部 crates.io
    """
    cargo = WORKSPACE_CRATES / crate_name / "Cargo.toml"
    if not cargo.exists():
        return []
    text = cargo.read_text(encoding="utf-8")
    deps = set()
    # 抓 [dependencies] 段
    m = re.search(r"\[dependencies\](.*?)(?=\n\[|\Z)", text, re.DOTALL)
    if not m:
        return []
    body = m.group(1)
    # 每个 dep key 形如:star-context = { path = "../star-context" }
    for line in body.splitlines():
        # path 形式
        m2 = re.search(r'(\w[\w\-]*)\s*=\s*\{\s*path\s*=\s*"../([\w\-]+)"', line)
        if m2:
            deps.add(m2.group(2))
            continue
        # workspace = true 形式:反查 workspace.dependencies 段
        m3 = re.search(r'(\w[\w\-]*)\s*=\s*\{\s*workspace\s*=\s*true', line)
        if m3:
            ws_dep = m3.group(1)
            ws_path = _resolve_workspace_dep(ws_dep)
            if ws_path:
                deps.add(ws_path)
    return sorted(deps)


def _resolve_workspace_dep(name: str):
    """Cargo.toml 的 [workspace.dependencies] 段,形如 star-context = { path = "crates/star-context" }"""
    text = CARGO_TOML.read_text(encoding="utf-8")
    m = re.search(r'\[workspace\.dependencies\](.*?)(?=\n\[|\Z)', text, re.DOTALL)
    if not m:
        return None
    body = m.group(1)
    m2 = re.search(rf'{re.escape(name)}\s*=\s*\{{[^}}]*path\s*=\s*"crates/([\w\-]+)"', body)
    if m2:
        return m2.group(1)
    return None


# ============================================================
# 2. 抓 frontend src / tools / scripts 节点
# ============================================================
def list_source_files(root: Path, exts: tuple, max_depth: int = 6):
    """递归列源文件,跳过 node_modules / target / __pycache__ / .next / dist"""
    skip = {"node_modules", "target", "__pycache__", ".next", "dist", ".git", "build"}
    out = []
    if not root.exists():
        return out
    for dirpath, dirnames, filenames in os.walk(root):
        # 剪枝
        dirnames[:] = [d for d in dirnames if d not in skip]
        # 深度
        rel = Path(dirpath).relative_to(root)
        if rel.parts and len(rel.parts) > max_depth:
            continue
        for fn in filenames:
            if fn.endswith(exts):
                out.append(Path(dirpath) / fn)
    return out


# ============================================================
# 3. DB Schema/Table 抓取
# ============================================================
def list_db_tables():
    """返回 [(schema_name, table_name, file_path)]"""
    out = []
    if not DB_TABLES_ROOT.exists():
        return out
    for f in sorted(DB_TABLES_ROOT.glob("*.md")):
        # 文件名约定: <schema>_<table>.md (e.g. tenant_tenant.md, permission_role.md)
        stem = f.stem
        if "_" in stem:
            schema, table = stem.split("_", 1)
            out.append((schema, table, f))
    return out


def infer_wtm(table_name: str, entity_form: str, r_w: str, rls: str, soft_delete: str) -> str:
    """从表名 + 形态启发式推 W/T/M (per 守门 #13 + 00-CLASSIFICATION-W-T-M.md v0.1)
    - 实体形态 L/P/MV = M
    - 实体形态 A/O = T
    - soft_delete=No + R/W=R → 多为 Lookup/Master
    - 表名包含 _event/_outbox/_log/_audit/_session → T
    - 表名包含 _draft/_temp/_pending/_work/_token → W
    - 表名包含 _policy/_config/_rule/_type/_template → M
    - 默认 T
    """
    n = table_name.lower()
    if entity_form in ("L", "P", "MV"):
        return "M"
    if entity_form in ("A", "O"):
        return "T"
    if "event" in n or "outbox" in n or "audit" in n or "session" in n or "log" in n:
        return "T"
    if "draft" in n or "temp" in n or "pending" in n or "token" in n or "work" in n.split("_")[-1:]:
        return "W"
    if "policy" in n or "config" in n or "rule" in n or "type" in n or "template" in n:
        return "M"
    if r_w == "R" and soft_delete == "N":
        return "M"
    return "T"


def parse_table_meta(md_path: Path):
    """抓 §1 基本信息表 行(per docs/data-design/ipa-detail/tables/*.md 统一格式):
    - 实体形态: Entity E / Weak W / Lookup L / Projection P / MV / A / O
    - Module: domain-xxx
    - RLS 適用: Yes/No
    - soft delete: Yes/No
    - 主键列: 在 §2 列定义表 PK='?' 行
    - 分类 W/T/M: 需查 00-CLASSIFICATION-W-T-M.md,本表不直接写
    """
    text = md_path.read_text(encoding="utf-8", errors="replace")
    meta = {
        "entity_form": "?",      # E / W / L / P / MV / A / O
        "module": "?",            # domain-xxx
        "rls": "?",               # Yes / No
        "soft_delete": "?",       # Yes / No
        "r_w": "?",               # R/W / R / W
        "pk": "?",
        "deps": [],
    }
    # 種別(per docs/data-design/ipa-detail/tables/*.md §1 行,日文):`| **種別** | Entity（…）| E |`
    m = re.search(r"\*\*種別\*\*", text)
    if m:
        line_end = text.find("\n", m.end())
        seg = text[m.start():line_end if line_end > 0 else m.end() + 200]
        m2 = re.search(r"\|\s*\*?\*?([EWLPMVAO])\s*\*?\*?\s*\|", seg)
        if m2:
            meta["entity_form"] = m2.group(1)
    # Module
    m = re.search(r"\*\*Module\*\*\s*\|\s*`?([\w\-]+)`?\s*\|", text)
    if m:
        meta["module"] = m.group(1)
    # RLS 必須 / 適用
    m = re.search(r"\*\*RLS[^*]*(?:必須|適用)\*\*", text)
    if m:
        line_end = text.find("\n", m.end())
        seg = text[m.start():line_end if line_end > 0 else m.end() + 200]
        if "**No**" in seg:
            meta["rls"] = "N"
        elif "**Yes**" in seg:
            meta["rls"] = "Y"
    # soft delete
    m = re.search(r"\*\*soft delete\*\*", text, re.IGNORECASE)
    if m:
        line_end = text.find("\n", m.end())
        seg = text[m.start():line_end if line_end > 0 else m.end() + 200]
        if re.search(r"Yes", seg):
            meta["soft_delete"] = "Y"
        elif re.search(r"No", seg):
            meta["soft_delete"] = "N"
    # R/W 識別
    m = re.search(r"\*\*R/W[^*]*識別\*\*", text)
    if m:
        line_end = text.find("\n", m.end())
        seg = text[m.start():line_end if line_end > 0 else m.end() + 200]
        m2 = re.search(r"\|\s*\*?\*?(R/W|R|W)\*?\*?\s*\|", seg)
        if m2:
            meta["r_w"] = m2.group(1)
    # 主キー:`| **主キー** | `id UUID` | ... |`
    m = re.search(r"\*\*主キー\*\*\s*\|\s*`?(\w+)", text)
    if m:
        meta["pk"] = m.group(1)
    return meta


# ============================================================
# 4. ADR 抓取
# ============================================================
def list_adrs():
    out = []
    if not ADR_ROOT.exists():
        return out
    for f in sorted(ADR_ROOT.glob("*.md")):
        # 文件名 NNNN-slug.md
        m = re.match(r"(\d{4})-(.+)\.md", f.name)
        if m:
            out.append((m.group(1), m.group(2), f))
    return out


def list_arch_views():
    """docs/architecture/ 下子目录 = 5 大 view (含 2026-08-26-upgrade / 2026-09-02-upgrade / 2026-09-03-agent-runtime / 2026-09-03-langgraph / 2026-09-03-treesitter-worktree-graph)"""
    out = []
    if not ARCH_VIEWS.exists():
        return out
    for d in sorted(ARCH_VIEWS.iterdir()):
        if d.is_dir() and d.name.startswith("2026-"):
            out.append(d)
    return out


# ============================================================
# 5. Schema -> Crate 映射(per 业务语义,守门 #3 拒绝盲映射)
# ============================================================
# schema 名 -> 主责 crate 名(单数对应;多 crate 协管标 [])
# 来源:docs/data-design/ipa-detail/00-INVENTORY.md §1.x + 01-requirements schema 段
# 实证:grep "domain-tenant" crates/*/Cargo.toml + ADR-0044 §G-1
SCHEMA_TO_CRATE = {
    "tenant": "domain-tenant",
    "permission": "domain-permission",
    "scm": "domain-scm",
    "agent": "domain-agent",
    "identity": "domain-identity",
    "validation": "domain-validation",
    "planning": "domain-planning",
    "local": "domain-local-runtime",  # runtime schema
    "worktree": "domain-worktree",
    "comment": "domain-comment",
    "context": "domain-context",
    "project": "domain-project",
    "board": "domain-board",
    "audit": "domain-audit",
    "workflow": "domain-workflow",
    "feedback": "domain-feedback",
    "notification": "domain-notification",
    "integration": "domain-integration",
    "collaboration": "domain-collaboration",
    "relation": "domain-relation",
    "workspace": "domain-workspace",
    "search": "domain-search",
    "automation": "domain-automation",
    "kms": "domain-kms",
    "development": "domain-development",
}


# ============================================================
# 6. crate -> arch view 映射(per ADR-0044/0045/0046)
# ============================================================
CRATE_TO_VIEW = {
    # LangGraph view
    "star-taskgraph": "2026-09-03-langgraph",
    "star-mcp": "2026-09-03-langgraph",
    "star-dispatcher": "2026-09-03-langgraph",
    "star-saga": "2026-09-03-langgraph",
    # Agent Runtime view
    "star-context": "2026-09-03-agent-runtime",
    "star-credential": "2026-09-03-agent-runtime",
    "domain-agent": "2026-09-03-agent-runtime",
    "domain-local-runtime": "2026-09-03-agent-runtime",
    # Tree-sitter / worktree view
    "star-treesitter": "2026-09-03-treesitter-worktree-graph",
    "domain-worktree": "2026-09-03-treesitter-worktree-graph",
    "star-vcs": "2026-09-03-treesitter-worktree-graph",
    # Legacy upgrade view
    "star-cache": "2026-08-26-upgrade",
    "star-cli": "2026-08-26-upgrade",
    "star-sse": "2026-08-26-upgrade",
    "star-webhook": "2026-08-26-upgrade",
    "star-sa": "2026-08-26-upgrade",
    "star-dto": "2026-09-02-upgrade",
    "star-api-rest": "2026-09-02-upgrade",
}


# ============================================================
# 7. 生成节点页
# ============================================================
def write_node(path: Path, title: str, body: str, **meta):
    """写一个 obsidian node 页: frontmatter + H1 + body
    body 已包含 [[wikilink]]
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    fm = fm_block(
        title=title,
        generated=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        **meta,
    )
    out = f"{fm}\n\n# {title}\n\n{body}\n"
    path.write_text(out, encoding="utf-8")


def crate_name_to_wiki(crate: str) -> str:
    """crate 名 -> wiki link 形式(obsidian 友好,无空格)"""
    return f"crate-{crate}"


def table_name_to_wiki(schema: str, table: str) -> str:
    return f"tbl-{schema}-{table}"


def schema_name_to_wiki(schema: str) -> str:
    return f"schema-{schema}"


def adr_name_to_wiki(num: str, slug: str) -> str:
    return f"adr-{num}-{slug}"


def view_name_to_wiki(view: str) -> str:
    return f"view-{view}"


# ============================================================
# 8. 完整生成
# ============================================================
def generate():
    # 清理旧内容(保留 .obsidian/)
    if PGWIKI_ROOT.exists():
        for p in PGWIKI_ROOT.iterdir():
            if p.name == ".obsidian":
                continue
            if p.is_dir():
                import shutil
                shutil.rmtree(p)
            else:
                p.unlink()
    PGWIKI_ROOT.mkdir(parents=True, exist_ok=True)

    # ---- 抓数据 ----
    members = parse_cargo_toml()
    print(f"[crate] {len(members)} workspace members")
    deps_map = {c: parse_crate_deps(c) for c in members}

    tables = list_db_tables()
    schemas = sorted({s for s, _, _ in tables})
    print(f"[db] {len(tables)} tables in {len(schemas)} schemas")

    adrs = list_adrs()
    print(f"[adr] {len(adrs)} ADRs")

    views = list_arch_views()
    print(f"[view] {len(views)} architecture views")

    frontend_files = list_source_files(FRONTEND_ROOT, (".ts", ".tsx", ".js", ".jsx"))
    print(f"[frontend] {len(frontend_files)} files")

    tools_files = list_source_files(TOOLS_ROOT, (".py", ".ts", ".sh", ".ps1", ".md"))
    print(f"[tools] {len(tools_files)} files")

    scripts_files = list_source_files(SCRIPTS_ROOT, (".py", ".ps1", ".mjs", ".sh"))
    # 过滤掉根目录的 p0_1_*.py (one-off)
    scripts_files = [f for f in scripts_files if "automation" not in f.parts]
    print(f"[scripts-root] {len(scripts_files)} files")

    auto_files = list_source_files(SCRIPTS_ROOT / "automation", (".py",))
    print(f"[automation] {len(auto_files)} files")

    # ---- 10-workspace/_crates/ ----
    for c in members:
        deps = deps_map.get(c, [])
        owned_schema = [s for s, crt in SCHEMA_TO_CRATE.items() if crt == c]
        arch_view = CRATE_TO_VIEW.get(c, "2026-08-26-upgrade")
        body = []
        body.append(f"**Workspace member**: `crates/{c}`")
        body.append("")
        body.append(f"**依赖 crate ({len(deps)})**:")
        if deps:
            for d in deps:
                body.append(f"- [[{crate_name_to_wiki(d)}]]")
        else:
            body.append("- (无 workspace 内依赖)")
        body.append("")
        body.append(f"**架构归属**: [[{view_name_to_wiki(arch_view)}]]")
        body.append("")
        if owned_schema:
            body.append(f"**主责 DB schema ({len(owned_schema)})**:")
            for s in owned_schema:
                body.append(f"- [[{schema_name_to_wiki(s)}]]")
            body.append("")
        body.append(f"**源文件路径**: `D:/Star/crates/{c}/src/`")
        # 列 src/ 下文件
        src_dir = WORKSPACE_CRATES / c / "src"
        if src_dir.exists():
            body.append("")
            body.append("**src/ 节点**:")
            for f in sorted(src_dir.glob("*.rs")):
                body.append(f"- `src/{f.name}`")
        write_node(
            PGWIKI_ROOT / "10-workspace" / "_crates" / f"{crate_name_to_wiki(c)}.md",
            title=c,
            body="\n".join(body),
            node_type="crate",
            crate_name=c,
            arch_view=arch_view,
            owned_schemas=",".join(owned_schema) if owned_schema else "",
            depends_on=",".join(deps),
            parent="10-workspace/MOC",
        )

    # ---- 10-workspace/MOC.md ----
    moc_ws = ["# Workspace 总览", ""]
    moc_ws.append(f"Star 主仓 (`D:/Star`) 当前 workspace 含 **{len(members)} 个 crate**。")
    moc_ws.append("")
    moc_ws.append("## Crate 索引")
    for c in members:
        moc_ws.append(f"- [[{crate_name_to_wiki(c)}]]")
    moc_ws.append("")
    moc_ws.append("## Frontend")
    moc_ws.append(f"- [[frontend-root]] ({len(frontend_files)} 个源文件)")
    moc_ws.append("")
    moc_ws.append("## Tools")
    moc_ws.append(f"- [[tools-root]] ({len(tools_files)} 个文件)")
    moc_ws.append("")
    moc_ws.append("## Scripts")
    moc_ws.append(f"- [[scripts-root]] ({len(scripts_files)} 个根级脚本)")
    moc_ws.append(f"- [[scripts-automation]] ({len(auto_files)} 个 automation 子目录脚本)")
    write_node(
        PGWIKI_ROOT / "10-workspace" / "MOC.md",
        title="Workspace MOC",
        body="\n".join(moc_ws),
        node_type="moc",
    )

    # ---- 10-workspace/frontend/tools/scripts 根节点 ----
    write_node(
        PGWIKI_ROOT / "10-workspace" / "frontend-root.md",
        title="Frontend (Next.js)",
        body=f"**根**: `D:/Star/frontend/`\n**源文件数**: {len(frontend_files)}\n\n## 主要目录\n- `src/app/` — Next.js app router\n- `src/components/` — 共享组件\n- `src/hooks/` — React hooks\n- `src/lib/` — 工具\n- `src/types/` — TS 类型\n- `src/i18n/` — 国际化\n- `e2e/` — Playwright 测试\n",
        node_type="frontend",
    )
    write_node(
        PGWIKI_ROOT / "10-workspace" / "tools-root.md",
        title="Tools",
        body=f"**根**: `D:/Star/tools/`\n**子目录**: star-flash-mock (回归测试 + mock data + k3s 部署)\n",
        node_type="tools",
    )
    write_node(
        PGWIKI_ROOT / "10-workspace" / "scripts-root.md",
        title="Scripts (root)",
        body=f"**根**: `D:/Star/scripts/`\n**根级脚本数**: {len(scripts_files)}\n\n(详细见 `scripts-automation` 子节点)",
        node_type="scripts",
    )
    write_node(
        PGWIKI_ROOT / "10-workspace" / "scripts-automation.md",
        title="Scripts/automation",
        body=f"**根**: `D:/Star/scripts/automation/`\n**脚本数**: {len(auto_files)}\n\n(per 守门 #19 v19 Python 化,2026-09-02 00:39 JST Ulysses 拍板)\n",
        node_type="scripts-automation",
    )

    # ---- 20-database/ ----
    # schema 节点
    # 优先用实际表 schema;再追加 SCHEMA_TO_CRATE 中定义但 0 表的 schema(占位)
    schemas_in_tables = set(schemas)
    schemas_orphan = [s for s in SCHEMA_TO_CRATE.keys() if s not in schemas_in_tables]
    all_schemas = sorted(schemas) + sorted(schemas_orphan)
    for schema in all_schemas:
        tables_in = [(s, t, f) for s, t, f in tables if s == schema]
        crate = SCHEMA_TO_CRATE.get(schema, "")
        tables_in = [(s, t, f) for s, t, f in tables if s == schema]
        crate = SCHEMA_TO_CRATE.get(schema, "")
        body = [
            f"**Schema**: `{schema}`",
            f"**表数**: {len(tables_in)}",
            f"**主责 crate**: [[{crate_name_to_wiki(crate)}]]" if crate else "**主责 crate**: (无对应 crate,纯 Lookup/Projection)",
            "",
            "## 表",
        ]
        for s, t, f in tables_in:
            body.append(f"- [[{table_name_to_wiki(s, t)}]]")
        write_node(
            PGWIKI_ROOT / "20-database" / "_schema" / f"{schema_name_to_wiki(schema)}.md",
            title=f"schema.{schema}",
            body="\n".join(body),
            node_type="db-schema",
            schema=schema,
            owner_crate=crate,
            table_count=len(tables_in),
            parent="20-database/MOC",
        )

    # table 节点
    for schema, table, f in tables:
        meta = parse_table_meta(f)
        wtm = infer_wtm(table, meta["entity_form"], meta["r_w"], meta["rls"], meta["soft_delete"])
        crate = SCHEMA_TO_CRATE.get(schema, "")
        body = [
            f"**Schema**: [[{schema_name_to_wiki(schema)}]]",
            f"**表名**: `{schema}.{table}`",
            f"**分类 (W/T/M)**: {wtm}",
            f"**实体形态**: {meta['entity_form']} (E=Entity / W=Weak / L=Lookup / P=Projection / MV / A=Append-only / O=Outbox)",
            f"**Module**: `{meta['module']}`",
            f"**R/W**: {meta['r_w']}",
            f"**RLS**: {meta['rls']}",
            f"**soft delete**: {meta['soft_delete']}",
            f"**主键**: `{meta['pk']}`",
        ]
        if crate:
            body.append(f"**主责 crate**: [[{crate_name_to_wiki(crate)}]]")
        body.append("")
        body.append(f"**源文档**: `D:/Star/docs/data-design/ipa-detail/tables/{schema}_{table}.md`")
        if meta["deps"]:
            body.append("")
            body.append("**外键依赖**:")
            for d in meta["deps"]:
                # d 形如 "tenant.tenant"
                s2, t2 = d.split(".", 1)
                body.append(f"- [[{table_name_to_wiki(s2, t2)}]]")
        write_node(
            PGWIKI_ROOT / "20-database" / "_table" / f"{table_name_to_wiki(schema, table)}.md",
            title=f"{schema}.{table}",
            body="\n".join(body),
            node_type="db-table",
            schema=schema,
            table=table,
            classification=wtm,
            entity_form=meta["entity_form"],
            module=meta["module"],
            rls=meta["rls"],
            soft_delete=meta["soft_delete"],
            r_w=meta["r_w"],
            owner_crate=crate,
            parent=f"20-database/_schema/{schema_name_to_wiki(schema)}",
        )

    # database MOC
    moc_db = [
        "# Database 总览",
        "",
        f"PostgreSQL 设计全 {len(all_schemas)} schema × {len(tables)} 表 (per `docs/data-design/ipa-detail/00-INVENTORY.md` v0.2)。",
        "",
        f"**说明**: {len(schemas)} schema 有表文件,另 {len(schemas_orphan)} schema (空) 来自 SCHEMA_TO_CRATE 映射(per 守门 #13 派生,crate 存在但 schema 表尚未落地)。",
        "",
        "## 守门 #13 W/T/M 分类 (per 2026-09-01 18:30 JST 拍板)",
        "",
        "| 分类 | 含义 | 表数 |",
        "|---|---|---|",
    ]
    by_class = defaultdict(int)
    for s, t, f in tables:
        m = parse_table_meta(f)
        wtm = infer_wtm(t, m["entity_form"], m["r_w"], m["rls"], m["soft_delete"])
        by_class[wtm] += 1
    for k in sorted(by_class.keys()):
        moc_db.append(f"| {k} | (per `00-CLASSIFICATION-W-T-M.md` v0.1 启发式推断,DDD Review 阶段需 Lead 复核) | {by_class[k]} |")
    moc_db.append("")
    moc_db.append("## Schema 索引")
    for s in all_schemas:
        moc_db.append(f"- [[{schema_name_to_wiki(s)}]]")
    write_node(
        PGWIKI_ROOT / "20-database" / "MOC.md",
        title="Database MOC",
        body="\n".join(moc_db),
        node_type="moc",
    )

    # ---- 30-architecture/ ----
    # view 节点
    for v in views:
        # view 内 markdown
        v_files = list(v.rglob("*.md"))
        body = [
            f"**架构 view**: `{v.name}`",
            f"**根**: `D:/Star/docs/architecture/{v.name}/`",
            f"**文档数**: {len(v_files)}",
            "",
            "## 文件",
        ]
        for vf in v_files[:30]:
            rel = vf.relative_to(v)
            body.append(f"- `{rel}`")
        if len(v_files) > 30:
            body.append(f"- ... (另 {len(v_files) - 30} 个)")
        write_node(
            PGWIKI_ROOT / "30-architecture" / "views" / f"{view_name_to_wiki(v.name)}.md",
            title=f"view: {v.name}",
            body="\n".join(body),
            node_type="arch-view",
            view_name=v.name,
        )
    # ADR 节点
    for num, slug, f in adrs:
        text = f.read_text(encoding="utf-8", errors="replace")
        # 抓标题
        m = re.search(r"^#\s+(.+)", text, re.MULTILINE)
        title = m.group(1).strip() if m else f"ADR-{num}"
        # 抓决策一句话
        m2 = re.search(r"(?:决策|Status|Decision)[:：]\s*(.+)", text)
        status = m2.group(1).strip() if m2 else "(无显式状态)"
        body = [
            f"**ADR 编号**: {num}",
            f"**Slug**: {slug}",
            f"**状态**: {status}",
            f"**源**: `D:/Star/docs/architecture/2026-08-26-upgrade/adr/{f.name}`",
        ]
        write_node(
            PGWIKI_ROOT / "30-architecture" / "adr" / f"{adr_name_to_wiki(num, slug)}.md",
            title=title,
            body="\n".join(body),
            node_type="adr",
            adr_number=num,
            adr_slug=slug,
        )
    # arch MOC
    moc_arch = [
        "# Architecture 总览",
        "",
        f"## 5 大架构 view (per AGENTS.md §6.1)",
        "",
    ]
    for v in views:
        moc_arch.append(f"- [[{view_name_to_wiki(v.name)}]] — `{v.name}`")
    moc_arch.append("")
    moc_arch.append(f"## {len(adrs)} 份 ADR")
    for num, slug, _ in adrs:
        moc_arch.append(f"- [[{adr_name_to_wiki(num, slug)}]] — ADR-{num} {slug}")
    write_node(
        PGWIKI_ROOT / "30-architecture" / "MOC.md",
        title="Architecture MOC",
        body="\n".join(moc_arch),
        node_type="moc",
    )

    # ---- 40-crosscutting/ ----
    # dependencies.md: 全 crate 依赖矩阵
    dep_lines = ["# Crate 依赖图", "", "每行: `crate -> [依赖列表]`", ""]
    for c in members:
        ds = deps_map.get(c, [])
        if ds:
            dep_lines.append(f"- [[{crate_name_to_wiki(c)}]] -> " + ", ".join(f"[[{crate_name_to_wiki(d)}]]" for d in ds))
    write_node(
        PGWIKI_ROOT / "40-crosscutting" / "dependencies.md",
        title="Crate 依赖图",
        body="\n".join(dep_lines),
        node_type="cross-deps",
    )
    # schema-to-crate.md
    s2c_lines = ["# Schema -> Crate 映射", "", "**依据**: 守门 #13 W/T/M + AGENTS.md §6.1 命名解读 disclaimer + ADR-0044", "", "| Schema | 主责 crate | 表数 |", "|---|---|---|"]
    for schema in schemas:
        n = sum(1 for s, t, _ in tables if s == schema)
        crt = SCHEMA_TO_CRATE.get(schema, "(无)")
        s2c_lines.append(f"| `{schema}` | [[{crate_name_to_wiki(crt)}]] | {n} |" if crt != "(无)" else f"| `{schema}` | (无) | {n} |")
    write_node(
        PGWIKI_ROOT / "40-crosscutting" / "schema-to-crate.md",
        title="Schema → Crate 映射",
        body="\n".join(s2c_lines),
        node_type="cross-schema-crate",
    )
    # quality-gates.md
    qg = [
        "# 质量门 4 维",
        "",
        "Per `AGENTS.md` §4.1 守门累积规 + `STAR-OLU-001.md` §6 质量门 5 维:",
        "",
        "| # | 维度 | 当前实测 | 工具 |",
        "|---|---|---|---|",
        f"| 1 | cargo check --workspace --lib -j 4 | (per 守门 #1 v19 实证,32.27s) | scripts/automation/console_server.py 跑后 0 err |",
        f"| 2 | pgwiki 节点数 | {len(members) + len(tables) + len(adrs) + len(views) + 4} | 本脚本 |",
        f"| 3 | 死链率 | 见 validate() 输出 | scripts/automation/pgwiki_index.py --check |",
        f"| 4 | cargo test -p star-context --lib -j 4 | 21/21 pass (per PR #12 实证) | CI |",
    ]
    write_node(
        PGWIKI_ROOT / "40-crosscutting" / "quality-gates.md",
        title="质量门 4 维",
        body="\n".join(qg),
        node_type="cross-quality",
    )

    # ---- 00-INDEX.md (root MOC) ----
    idx = [
        "# pgwiki 总目录 (D:/Star Obsidian Vault)",
        "",
        f"**生成时间**: {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        f"**节点数**: {len(members) + len(tables) + len(adrs) + len(views) + 4} (crate + table + ADR + view + 根级)",
        f"**crate**: {len(members)}",
        f"**DB table**: {len(tables)}",
        f"**DB schema**: {len(schemas)}",
        f"**ADR**: {len(adrs)}",
        f"**架构 view**: {len(views)}",
        "",
        "## 拓扑分层",
        "",
        "- [[10-workspace/MOC]] — 物理代码(48 crate + frontend + tools + scripts)",
        "- [[20-database/MOC]] — 数据库(25 schema × 93 table,per docs/data-design/ipa-detail/)",
        "- [[30-architecture/MOC]] — 架构 view + 27 份 ADR",
        "- [[40-crosscutting/dependencies]] — Crate 依赖图(per Cargo.toml)",
        "- [[40-crosscutting/schema-to-crate]] — Schema ↔ Crate 映射(守门 #13)",
        "- [[40-crosscutting/quality-gates]] — 守门 4 维",
        "",
        "## 边类型(全部 frontmatter)",
        "",
        "- `parent` / `child` — 层级",
        "- `depends_on` — crate → crate (per Cargo.toml `path = \"../x\"`)",
        "- `owner_crate` — schema → crate (per 守门 #13 派生)",
        "- `arch_view` — crate → view (per ADR-0044/0045/0046)",
        "- `part_of_view` — ADR → view",
        "",
        "## Obsidian 使用",
        "",
        "1. 用 Obsidian 打开 `D:/Star/docs/wiki/pgwiki/` (File -> Open vault -> Open folder as vault)",
        "2. 启用 Graph view: 点击左侧栏 Graph 图标",
        "3. 启用 Backlinks: 默认开启",
        "4. 搜索: Ctrl/Cmd + Shift + F",
    ]
    write_node(
        PGWIKI_ROOT / "00-INDEX.md",
        title="pgwiki 总目录",
        body="\n".join(idx),
        node_type="moc-root",
    )

    # 把 all_schemas / tables 计数 / 节点总数暴露给 validate
    global _GEN_STATS
    _GEN_STATS = {
        "members": len(members),
        "tables": len(tables),
        "schemas_total": len(all_schemas),
        "schemas_with_tables": len(schemas),
        "schemas_orphan": len(schemas_orphan),
        "adrs": len(adrs),
        "views": len(views),
        "frontend_files": len(frontend_files),
        "tools_files": len(tools_files),
        "automation_files": len(auto_files),
    }

    # ---- .obsidian 配置 ----
    obs = PGWIKI_ROOT / ".obsidian"
    obs.mkdir(parents=True, exist_ok=True)
    (obs / "app.json").write_text(json.dumps({
        "alwaysUpdateLinks": True,
        "newLinkFormat": "shortest",
        "useMarkdownLinks": False,
        "showLineNumber": True,
        "showInlineTitle": True,
    }, indent=2), encoding="utf-8")
    (obs / "graph.json").write_text(json.dumps({
        "search": "",
        "showOrphans": True,
        "showAttachments": False,
        "hideUnresolved": False,
    }, indent=2), encoding="utf-8")

    print(f"\n[OK] pgwiki 生成完毕,根: {PGWIKI_ROOT}")


# ============================================================
# 9. 验证:死链 + 节点数
# ============================================================
def validate():
    """扫所有 .md 的 [[wikilink]],检查目标是否存在"""
    all_nodes = set()
    for p in PGWIKI_ROOT.rglob("*.md"):
        # 节点名 = 不带扩展名的文件名(去目录)
        all_nodes.add(p.stem)
    print(f"[validate] 已生成 {len(all_nodes)} 节点")

    dead = []
    total_links = 0
    for p in PGWIKI_ROOT.rglob("*.md"):
        text = p.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(r"\[\[([^\]]+)\]\]", text):
            target = m.group(1).strip()
            # 支持 [[name|alias]] 形式
            if "|" in target:
                target = target.split("|", 1)[0].strip()
            total_links += 1
            # Obsidian 兼容:支持 "dir/file" 路径或裸 "file"
            target_short = target.split("/")[-1] if "/" in target else target
            if target in all_nodes or target_short in all_nodes:
                continue
            dead.append((p.relative_to(PGWIKI_ROOT), target))
    print(f"[validate] 总链接 {total_links} 个,死链 {len(dead)} 个")
    if dead:
        print("[validate] 死链样本(最多 20):")
        for src, tgt in dead[:20]:
            print(f"  {src}  -> [[{tgt}]]")
    rate = (len(dead) / total_links * 100) if total_links else 0
    print(f"[validate] 死链率 {rate:.2f}%")
    return len(dead) == 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只验证不重新生成")
    args = ap.parse_args()
    if args.check:
        ok = validate()
        sys.exit(0 if ok else 1)
    else:
        generate()
        ok = validate()
        if not ok:
            print("[WARN] 有死链,需要修")


if __name__ == "__main__":
    main()
