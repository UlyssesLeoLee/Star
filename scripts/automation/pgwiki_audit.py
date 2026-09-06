"""pgwiki_audit.py — 对照 docswiki vs pgwiki,扫"模块间问题"。

输出:
- 50-issues/00-orphan-schemas.md  schema 有表但 SCHEMA_TO_CRATE 缺失
- 50-issues/01-placeholder-schemas.md  schema 占位(无表)且对应 crate 也是空
- 50-issues/02-empty-crates.md  crate 在 workspace 但 src/ 不存在或 0 个 .rs
- 50-issues/03-broker-adr-refs.md  ADR 引用了但 workspace 不存在的 crate 段
- 50-issues/04-cargo-vs-arch.md  arch view (per docs/architecture/20266-*-*) 列了
                              但 workspace 没注册的 crate
- 50-issues/05-table-no-crate.md  表的 Module 字段(per docs/data-design) 指向不存在的 crate
- 50-issues/06-cargo-dep-anomaly.md  cargo 依赖图里的环 / 缺失
- 50-issues/07-docswiki-vs-pgwiki.md  docwiki 7 份文件 vs pgwiki 节点 对照表

设计原则 (per Ulysses 2026-09-06 11:47 JST 拍板):
- pgwiki = 程序实际拓扑的"事实"
- docswiki = DDD 视角的"叙事"
- 本 audit 工具 = 两边对照,暴露"叙事与事实不符"的地方
- 不建立 1:1 映射,不强行套 (per 守门 #3 + AGENTS.md §5 disclaimer)
"""
import re
import json
import sys
from collections import defaultdict
from pathlib import Path
from datetime import datetime, timezone

REPO = Path(r"D:/Star")
PGWIKI = REPO / "docs" / "wiki" / "pgwiki"
DOCSWIKI = REPO / "docs" / "wiki" / "docswiki"
DB_DETAIL = REPO / "docs" / "data-design" / "ipa-detail"
DB_TABLES = DB_DETAIL / "tables"
CARGO_TOML = REPO / "Cargo.toml"
ADR_DIR = REPO / "docs" / "architecture" / "2026-08-26-upgrade" / "adr"
ARCH_DIR = REPO / "docs" / "architecture"
CRATES = REPO / "crates"

# 跟 pgwiki_index.py 保持一致
SCHEMA_TO_CRATE = {
    "tenant": "domain-tenant", "permission": "domain-permission", "scm": "domain-scm",
    "agent": "domain-agent", "identity": "domain-identity", "validation": "domain-validation",
    "planning": "domain-planning", "local": "domain-local-runtime", "worktree": "domain-worktree",
    "comment": "domain-comment", "context": "domain-context", "project": "domain-project",
    "board": "domain-board", "audit": "domain-audit", "workflow": "domain-workflow",
    "feedback": "domain-feedback", "notification": "domain-notification",
    "integration": "domain-integration", "collaboration": "domain-collaboration",
    "relation": "domain-relation", "workspace": "domain-workspace", "search": "domain-search",
    "automation": "domain-automation", "kms": "domain-kms", "development": "domain-development",
}


def parse_cargo_members():
    text = CARGO_TOML.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"members\s*=\s*\[(.*?)\]", text, re.DOTALL)
    out = []
    for line in m.group(1).splitlines():
        line = line.strip().rstrip(",").strip()
        if not line or line.startswith("#"):
            continue
        m2 = re.match(r'"crates/([\w\-]+)"', line)
        if m2:
            out.append(m2.group(1))
    return out


def list_db_schemas():
    out = set()
    for f in DB_TABLES.glob("*.md"):
        s = f.stem.split("_", 1)[0]
        out.add(s)
    return out


def get_table_module(table_file: Path):
    """从 docs/data-design/ipa-detail/tables/<schema>_<table>.md 抽 Module 行"""
    text = table_file.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"\*\*Module\*\*\s*\|\s*`?([\w\-]+)`?", text)
    if m:
        return m.group(1)
    return None


def scan_adrs_for_crate_refs():
    """扫 ADR 文档,找引用的 crate 名(可能不存在)。
    启发式: ADR 里引用的 'star-xxx' / 'domain-xxx' 在工作区 52 个 member 之外。
    排除: ADR/标题后缀(arch/spec/-spec/-design 等) + RFC/Phase 内部代号。
    """
    text = ""
    for f in sorted(ADR_DIR.glob("*.md")):
        text += f.read_text(encoding="utf-8", errors="replace") + "\n"
    # 抓 `xxx` 反引号内的(代码形式最可靠)
    refs = set()
    for m in re.finditer(r"`(star|domain|api|application|infrastructure)[\-_]([\w\-]+)`", text):
        refs.add(f"{m.group(1)}-{m.group(2)}")
    # 过滤明显的非 crate(后缀:arch/spec/XX/-spec/-design/-crate / -srs)
    bad_suffix = ("-arch", "-spec", "-design", "-crate", "-srs", "-compatibility",
                  "-gateway-arch", "compat-arch", "-runtime-srs", "compat")
    refs = {r for r in refs if not any(r.endswith(suf) or suf in r for suf in bad_suffix)}
    # 再过滤带特殊字符(像 \u00a0 等)+ 长度<5 的怪引用
    refs = {r for r in refs if r.replace("-", "").isalnum() and len(r) >= 5}
    return refs


def scan_arch_views_for_crate_refs():
    """扫 5 大 arch view 目录的 .md,找反引号内引用的 crate 段。"""
    refs = set()
    for d in sorted(ARCH_DIR.iterdir()):
        if d.is_dir() and d.name.startswith("2026-"):
            for f in d.rglob("*.md"):
                text = f.read_text(encoding="utf-8", errors="replace")
                for m in re.finditer(r"`(star|domain|api|application|infrastructure)[\-_]([\w\-]+)`", text):
                    refs.add(f"{m.group(1)}-{m.group(2)}")
    bad_suffix = ("-arch", "-spec", "-design", "-crate", "-srs", "-compatibility",
                  "-gateway-arch", "compat-arch", "-runtime-srs", "compat")
    refs = {r for r in refs if not any(r.endswith(suf) or suf in r for suf in bad_suffix)}
    refs = {r for r in refs if r.replace("-", "").isalnum() and len(r) >= 5}
    return refs


def main():
    members = parse_cargo_members()
    members_set = set(members)
    print(f"[audit] cargo workspace members: {len(members)}")

    schemas = list_db_schemas()
    print(f"[audit] DB schemas (有表): {len(schemas)}")

    # ---- Issue 1: schema 有表但 SCHEMA_TO_CRATE 没列(orphans) ----
    orphan_schemas = sorted(schemas - set(SCHEMA_TO_CRATE.keys()))

    # ---- Issue 2: schema 占位(在 SCHEMA_TO_CRATE 但无表) ----
    placeholder_schemas = sorted(set(SCHEMA_TO_CRATE.keys()) - schemas)

    # ---- Issue 3: crate 在 workspace 但 src/ 不存在或 0 个 .rs ----
    empty_crates = []
    for c in members:
        src = CRATES / c / "src"
        if not src.exists():
            empty_crates.append((c, "no src/"))
        else:
            rs_count = sum(1 for _ in src.glob("*.rs"))
            if rs_count == 0:
                empty_crates.append((c, "0 .rs in src/"))

    # ---- Issue 4: ADR 引用但 workspace 不存在 ----
    adr_refs = scan_adrs_for_crate_refs()
    broker_adr = sorted(adr_refs - members_set)
    # 过滤:可能 ADR 引用的是 "star-cli" 但实际是 "star-cli"的别名 — 不过这些名应该都一致
    # 真有 "context-graph" 这类可能不是 crate

    # ---- Issue 5: arch view 引用但 workspace 不存在 ----
    arch_refs = scan_arch_views_for_crate_refs()
    broker_arch = sorted(arch_refs - members_set)

    # ---- Issue 6: 表 Module 字段指向不存在的 crate ----
    table_module_orphan = []
    for f in DB_TABLES.glob("*.md"):
        m = get_table_module(f)
        if m and m not in members_set:
            table_module_orphan.append((f.stem, m))

    # ---- Issue 7: docwiki 7 份 vs pgwiki 节点 对照 ----
    docswiki_files = sorted(DOCSWIKI.glob("*.md"))
    docswiki_topics = {
        "00-design-topology.md": "工程拓扑总览(DDD 视角)",
        "01-ui-agent-view.md": "UI Agent view",
        "02-orchestration-langgraph.md": "LangGraph 编排",
        "03-runtime-ecs.md": "ECS Runtime",
        "04-domain-crates.md": "DDD domain crate 划分",
        "05-persistence-checkpoint.md": "持久化 + checkpoint",
        "06-data-flow.md": "数据流",
    }
    # 每个 docwiki 主题对应 pgwiki 哪些节点(粗匹配)
    docswiki_vs_pgwiki = []
    for fname, topic in docswiki_topics.items():
        f = DOCSWIKI / fname
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8", errors="replace")
        # 抓 §1 之类的标题
        m = re.search(r"^#\s+(.+)", text, re.MULTILINE)
        title = m.group(1).strip() if m else fname
        # 抓提到的 crate / schema / view
        crates_mentioned = set()
        for m2 in re.finditer(r"\b(star|domain|api|application|infrastructure)[\-_]([\w\-]+)", text):
            crates_mentioned.add(f"{m2.group(1)}-{m2.group(2)}")
        docswiki_vs_pgwiki.append({
            "file": fname,
            "title": title,
            "topic": topic,
            "crates_mentioned": sorted(crates_mentioned),
        })

    # ---- Issue 8: cargo 依赖里环 ----
    # 简单:不抓环(workspace 必然 DAG),只抓引用了不在 workspace 的"伪依赖"
    fake_deps = []
    for c in members:
        cargo = CRATES / c / "Cargo.toml"
        if not cargo.exists():
            continue
        text = cargo.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(r'(\w[\w\-]*)\s*=\s*\{\s*path\s*=\s*"\.\./([\w\-]+)"', text):
            dep = m.group(2)
            if dep not in members_set:
                fake_deps.append((c, dep))

    # ---- 写 50-issues/ ----
    out_dir = PGWIKI / "50-issues"
    out_dir.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    def write_issue(name, title, body):
        path = out_dir / name
        fm = f"---\ntitle: \"{title}\"\ngenerated: \"{now}\"\nnode_type: \"audit-issue\"\n---\n\n# {title}\n\n{body}\n"
        path.write_text(fm, encoding="utf-8")
        print(f"[audit] wrote {path.relative_to(PGWIKI)}")

    # Issue 1
    if orphan_schemas:
        body = "## Orphan Schemas(有表但 SCHEMA_TO_CRATE 缺失)\n\n"
        body += "**含义**: DB 里有表,但 `scripts/automation/pgwiki_index.py` 的 `SCHEMA_TO_CRATE` 映射表没列。\n"
        body += "**风险**: pgwiki 不会给这些 schema 建 schema 节点 / crate 关联边,docwiki 对照时会缺一段。\n\n"
        body += "| Schema | DB 文件数 | pgwiki 节点 | 应主责 crate(需 DDD Review 拍板)|\n|---|---|---|---|\n"
        for s in orphan_schemas:
            n = sum(1 for f in DB_TABLES.glob(f"{s}_*.md"))
            body += f"| `{s}` | {n} | 无 | ?(DDD Review 阶段拍)|\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 1\n"
        body += f"\n## Counters\n\n```json\n{json.dumps({'orphan_schemas': len(orphan_schemas), 'orphan_list': orphan_schemas}, ensure_ascii=False)}\n```\n"
        write_issue("00-orphan-schemas.md", "Orphan Schemas(有表无 crate 映射)", body)
    else:
        print("[audit] 0 orphan schemas")

    # Issue 2
    body = "## Placeholder Schemas(占位:无表 + SCHEMA_TO_CRATE 列出)\n\n"
    body += "**含义**: crate 已注册在 workspace,`SCHEMA_TO_CRATE` 显式列出 schema 名,但 `docs/data-design/ipa-detail/tables/` 没表文件。\n"
    body += "**风险**: 该 crate 的持久化层未落地 / schema 设计在另一份文档,可能跟 docs/architecture 描述的 EFS 不一致。\n\n"
    body += "| Schema | 占位 crate | 实际表数 | 建议 |\n|---|---|---|---|\n"
    for s in placeholder_schemas:
        crt = SCHEMA_TO_CRATE[s]
        body += f"| `{s}` | `{crt}` | 0 | DDD Review 阶段确认是否落地 schema 设计,或撤映射|\n"
    body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 2\n"
    body += f"\n## Counters\n\n```json\n{json.dumps({'placeholder_schemas': len(placeholder_schemas), 'placeholder_list': placeholder_schemas}, ensure_ascii=False)}\n```\n"
    write_issue("01-placeholder-schemas.md", "Placeholder Schemas(crate 在,schema 表未落地)", body)

    # Issue 3
    if empty_crates:
        body = "## Empty Crates(workspace 注册但 src/ 空)\n\n"
        body += "**含义**: `Cargo.toml` 的 `members` 列出 crate,但 `crates/<name>/src/` 缺失或 0 个 .rs 文件。\n"
        body += "**风险**: 这些 crate 是骨架 / 早期 placeholder,可能 `cargo check` 通过但实际无逻辑,或文档/SRS 描述超出实际。\n\n"
        body += "| Crate | 问题 |\n|---|---|\n"
        for c, why in empty_crates:
            body += f"| `{c}` | {why} |\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 3\n"
        body += f"\n## Counters\n\n```json\n{json.dumps({'empty_crates': len(empty_crates), 'empty_list': empty_crates}, ensure_ascii=False)}\n```\n"
        write_issue("02-empty-crates.md", "Empty Crates(workspace 列出但 src 空)", body)
    else:
        print("[audit] 0 empty crates")

    # Issue 4
    if broker_adr:
        body = "## Broker ADR Refs(ADR 引用但 workspace 无)\n\n"
        body += "**含义**: 27 份 ADR 文档中提到这些 crate 段,但 `Cargo.toml` 没注册。\n"
        body += "**风险**: ADR 是历史决策,可能(1)crate 已重命名(2)crate 被废弃(3)ADR 描述超前(4)措辞非 crate 名。\n"
        body += "**行动**: 逐条 git log --follow 实证,或在 DDD Review 阶段删 ADR 引用 / 补 crate。\n\n"
        body += "| 引用名 | 出现 ADR | 实证片段(前 80 字符) |\n|---|---|---|\n"
        for c in broker_adr:
            evidences = []
            for f in sorted(ADR_DIR.glob("*.md")):
                text = f.read_text(encoding="utf-8", errors="replace")
                if f"`{c}`" in text:
                    for line in text.split("\n"):
                        if f"`{c}`" in line:
                            # 截到 80 字符(避免长 line)
                            snippet = line.strip()[:120]
                            evidences.append((f.name, snippet))
                            break
            adr_list = ", ".join(set(e[0] for e in evidences))
            snippet_str = evidences[0][1] if evidences else "(无)"
            body += f"| `{c}` | {adr_list} | `{snippet_str}` |\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 4\n"
        body += f"\n## Counters\n\n```json\n{json.dumps({'broker_adr': len(broker_adr), 'broker_adr_list': broker_adr}, ensure_ascii=False)}\n```\n"
        write_issue("03-broker-adr-refs.md", "Broker ADR Refs(ADR 引用但 crate 不存在)", body)
    else:
        print("[audit] 0 broker ADR refs")

    # Issue 5
    if broker_arch:
        body = "## Broker Arch View Refs(arch view 引用但 workspace 无)\n\n"
        body += "**含义**: 5 大架构 view 文档提到这些 crate 段,但 `Cargo.toml` 没注册。\n"
        body += "**行动**: DDD Review / arch view 文档升版时复核。\n\n"
        body += "| 引用名 |\n|---|\n"
        for c in broker_arch:
            body += f"| `{c}` |\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 5\n"
        body += f"\n## Counters\n\n```json\n{json.dumps({'broker_arch': len(broker_arch), 'broker_arch_list': broker_arch}, ensure_ascii=False)}\n```\n"
        write_issue("04-broker-arch-refs.md", "Broker Arch View Refs", body)

    # Issue 6
    if table_module_orphan:
        body = "## Table Module → Broken Crate(表的 Module 字段指向不存在 crate)\n\n"
        body += "**含义**: `docs/data-design/ipa-detail/tables/*.md §1 基本情報` 的 `Module` 行指向不存在的 crate。\n"
        body += "**风险**: 文档级 schema 跟实际 Rust crate 不一致,实装时找不到 module。\n\n"
        body += "| 表(文档) | 指向 Module |\n|---|---|\n"
        for tname, mod in table_module_orphan:
            body += f"| `{tname}` | `{mod}` |\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 6\n"
        write_issue("05-table-module-broken.md", "Table Module → Broken Crate", body)

    # Issue 7: docswiki vs pgwiki 对照表
    body = "## docswiki vs pgwiki 对照表\n\n"
    body += "**设计原则** (per 2026-09-06 11:47 JST 拍板):\n"
    body += "- **docswiki** = DDD 视角的叙事(7 份设计主题)\n"
    body += "- **pgwiki** = 程序实际拓扑(212 节点,事实)\n"
    body += "- **不**建立 1:1 映射(per 守门 #3 + AGENTS.md §5 disclaimer)\n"
    body += "- 对照目的:暴露\"叙事与事实\"的差异 = **问题**\n\n"
    body += "| docwiki 文件 | 主题 | 提到的 crate 段 | pgwiki 对应层 | 差异/缺口 |\n|---|---|---|---|---|\n"
    layer_map = {
        "00-design-topology.md": "30-architecture/MOC + 40-crosscutting/dependencies",
        "01-ui-agent-view.md": "10-workspace/frontend-root + crate-star-api-rest",
        "02-orchestration-langgraph.md": "30-architecture/views/view-2026-09-03-langgraph",
        "03-runtime-ecs.md": "crate-star-dispatcher + crate-domain-agent",
        "04-domain-crates.md": "10-workspace/_crates (35 domain-*)",
        "05-persistence-checkpoint.md": "20-database/MOC + adr-0047-postgresql-checkpointer-tier3",
        "06-data-flow.md": "40-crosscutting/dependencies",
    }
    # 实际 crate 名表(只在 main 里用,前面已经准备好 members_set)
    actual_crates = members_set
    for entry in docswiki_vs_pgwiki:
        layer = layer_map.get(entry["file"], "?")
        # 拆"存在 vs 设计意图"两类
        existing = [c for c in entry["crates_mentioned"] if c in actual_crates]
        designed_only = [c for c in entry["crates_mentioned"] if c not in actual_crates]
        existing_wiki = ", ".join(f"[[crate-{c}]]" for c in existing[:6])
        designed_str = ", ".join(f"`{c}`" for c in designed_only[:6])
        if len(designed_only) > 6:
            designed_str += f" ... (+{len(designed_only)-6})"
        existing_display = existing_wiki if existing_wiki else "(无)"
        # layer 是"X + Y"形式,只把第一个 token 当 wikilink
        layer_first = layer.split(" ")[0] if layer else "?"
        body += f"| [[docswiki-{entry['file'].replace('.md','')}]] | {entry['topic']} | {existing_display} | [[{layer_first}]] | 已实装 {len(existing)} / 设计意图 {len(designed_only)} |\n"
        if designed_str:
            body += f"|  |  | **设计意图(workspace 未实装)**: {designed_str} |  |  |\n"
    body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 7\n"
    body += f"\n## Counters\n\n```json\n{json.dumps({'docswiki_contrast': True, 'topic_count': len(docswiki_vs_pgwiki)}, ensure_ascii=False)}\n```\n"
    write_issue("07-docswiki-vs-pgwiki.md", "docswiki vs pgwiki 对照表", body)

    # Issue 8: fake deps
    if fake_deps:
        body = "## Cargo Fake Deps(Cargo.toml path 引用 workspace 之外的 crate)\n\n"
        body += "**含义**: 某 crate 的 `Cargo.toml` `path = \"../xxx\"` 引用,但 xxx 不在 workspace members。\n"
        body += "**风险**: `cargo build` 会失败(找不到 crate)或引用了一个外部项目。\n\n"
        body += "| 来源 crate | 引用 crate |\n|---|---|\n"
        for src, dep in fake_deps:
            body += f"| `{src}` | `{dep}` |\n"
        body += f"\n**生成时间**: {now}\n**来源**: `pgwiki_audit.py` Issue 8\n"
        write_issue("06-cargo-fake-deps.md", "Cargo Fake Deps", body)

    # ---- 50-issues/MOC.md ----
    issue_files = sorted(out_dir.glob("*.md"))
    moc = f"# Issues 总览\n\n"
    moc += f"**生成时间**: {now}\n\n"
    moc += "**对照目的** (per 2026-09-06 11:47 JST Ulysses 拍板):\n"
    moc += "- pgwiki = 程序实际拓扑(事实)\n"
    moc += "- docswiki = DDD 视角叙事\n"
    moc += "- 本目录 = 两者对照暴露的\"模块间问题\"\n\n"
    moc += f"## 统计\n\n"
    moc += f"| 维度 | 数 |\n|---|---|\n"
    moc += f"| Cargo workspace members | {len(members)} |\n"
    moc += f"| DB schema (有表) | {len(schemas)} |\n"
    moc += f"| Orphan schema (有表无映射) | {len(orphan_schemas)} |\n"
    moc += f"| Placeholder schema (无表) | {len(placeholder_schemas)} |\n"
    moc += f"| Empty crate (src 空) | {len(empty_crates)} |\n"
    moc += f"| Broker ADR refs | {len(broker_adr)} |\n"
    moc += f"| Broker Arch refs | {len(broker_arch)} |\n"
    moc += f"| Table Module broken | {len(table_module_orphan)} |\n"
    moc += f"| Cargo fake deps | {len(fake_deps)} |\n"
    moc += f"| docswiki 文件 | {len(docswiki_files)} |\n\n"
    moc += "## Issue 索引\n\n"
    for f in issue_files:
        if f.name == "MOC.md":
            continue
        # 从 frontmatter 读 title(避免把整个文件塞进 list)
        text = f.read_text(encoding="utf-8", errors="replace")
        m = re.search(r'^title:\s*"(.+?)"', text, re.MULTILINE)
        title = m.group(1) if m else f.stem
        moc += f"- [[{f.stem}]] — {title}\n"
    write_issue_path = out_dir / "MOC.md"
    fm = f"---\ntitle: \"Issues MOC\"\ngenerated: \"{now}\"\nnode_type: \"moc\"\n---\n\n"
    write_issue_path.write_text(fm + moc, encoding="utf-8")
    print(f"[audit] wrote {write_issue_path.relative_to(PGWIKI)}")

    # ---- 同时写一个摘要给 stdout,方便回显 ----
    summary = {
        "members": len(members),
        "schemas": len(schemas),
        "orphan_schemas": orphan_schemas,
        "placeholder_schemas": placeholder_schemas,
        "empty_crates": empty_crates,
        "broker_adr": broker_adr,
        "broker_arch": broker_arch,
        "table_module_orphan": table_module_orphan,
        "fake_deps": fake_deps,
    }
    print("\n[audit] SUMMARY:")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
