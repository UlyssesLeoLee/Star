#!/usr/bin/env python3
"""arch-live 文档漂移门禁 + crate 列表生成器.

为什么存在
    docs/arch-live/*.html 里每个数字都带 data-fact="<key>". 本脚本重新扫描仓库,
    把真实值与页面里写的值逐条比对. 文档一旦漂移 (加 crate / 加表 / 改路由),
    这里立刻变红 —— 避免"文档比实现旧", 也避免"文档把没实现的写成已实现".

用法
    python scripts/automation/arch_live_doc_check.py            # 校验
    python scripts/automation/arch_live_doc_check.py --emit-chips > chips.html
    python scripts/automation/arch_live_doc_check.py --self-test

退出码 (fail-closed)
    0  全部一致
    1  有漂移 (页面数字 != 仓库真实值)
    2  无法解析 / 事实计算为空 (不是"0 违规", 是"没测到") —— 必须与 1 区分
    3  页面或仓库路径缺失

设计要点 (踩过的坑)
    * 解析结果为 0 一律 exit 2, 不当"干净"处理.
    * 每个事实配一个人工核对过的下限, 防止解析器静默退化后把所有数字刷成 0.
    * 变异自测: --self-test 会注入一个错误值, 断言它被判为漂移; 同时断言
      "未注入"的真实值被判为一致 —— 两者缺一, 无法区分"门禁有牙"和"门禁没跑".
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]
DOCS = REPO / "docs" / "arch-live"
CRATES = REPO / "crates"

# fact key -> (人类可读名, 下限, 计算函数名)
# 下限来源: 2026-10-04 commit c1ce1630 实测值, 留出新增余量, 但不允许"解析退化到 0"
FLOORS = {
    "crates_total": ("crate 总数", 100),
    "domain_crates": ("domain-* crate", 35),
    "star_crates": ("star-* crate", 25),
    "wasm_crates": ("*-wasm crate", 3),
    "rust_files": ("Rust 源文件", 500),
    "rust_loc": ("Rust 代码行", 150000),
    "migrations": ("DB migration", 40),
    "db_tables": ("CREATE TABLE", 140),
    "db_policies": ("CREATE POLICY", 400),
    "db_schemas": ("DB schema 数", 6),
    "db_multica": ("multica schema 表数", 40),
    "db_automation": ("automation schema 表数", 7),
    "db_canvas": ("canvas schema 表数", 5),
    "db_plugin": ("plugin schema 表数", 5),
    "db_permission": ("permission schema 表数", 3),
    "db_scm": ("scm schema 表数", 2),
    "rest_routes": ("REST 路由模板", 45),
    "mcp_tools": ("MCP tool", 16),
    "cli_commands": ("CLI 子命令", 13),
    "fe_pages": ("前端 page.tsx", 50),
    "fe_components": ("前端组件文件", 150),
    "ci_workflows": ("CI workflow", 10),
}

FAMILY_RULES = [
    ("domain-", "domain", "DDD bounded context"),
    ("star-", "star", "平台能力 crate"),
    ("wasm", "wasm", "浏览器 WASM 桥"),
    ("wc-", "canvas", "worktree-canvas 14 crate"),
]


def read_text(p: pathlib.Path) -> str:
    return p.read_text(encoding="utf-8", errors="replace")


def count_files(pat: str, root: pathlib.Path) -> int:
    return sum(1 for _ in root.rglob(pat))


def compute() -> dict[str, int]:
    f: dict[str, int] = {}
    crate_dirs = [d for d in CRATES.iterdir() if d.is_dir()]
    names = [d.name for d in crate_dirs]
    f["crates_total"] = len(names)
    f["domain_crates"] = sum(1 for n in names if n.startswith("domain-"))
    f["star_crates"] = sum(1 for n in names if n.startswith("star-"))
    f["wasm_crates"] = sum(1 for n in names if n.endswith("-wasm"))
    f["rust_files"] = count_files("*.rs", CRATES)
    f["rust_loc"] = 0
    for rs in CRATES.rglob("*.rs"):
        try:
            f["rust_loc"] += read_text(rs).count("\n") + 1
        except OSError:
            pass

    migs = sorted((REPO / "db" / "migrations").glob("*.sql"))
    f["migrations"] = len(migs)
    sql = "\n".join(read_text(m) for m in migs)
    f["db_tables"] = len(re.findall(r"CREATE TABLE", sql))
    f["db_policies"] = len(re.findall(r"CREATE POLICY", sql))
    f["db_schemas"] = len(set(re.findall(r"CREATE SCHEMA(?:\s+IF NOT EXISTS)?\s+([a-z_]+)", sql, re.I)))
    # 按 schema 统计: CREATE TABLE [IF NOT EXISTS] [schema.]name
    # 注意区分带前缀与不带前缀 (不带前缀的落在 search_path 默认 schema = public),
    # 这两类不能混算, 否则 150 张表会被少报成 60 多张。
    for sch in ("multica", "automation", "canvas", "plugin", "permission", "scm"):
        f[f"db_{sch}"] = len(re.findall(
            rf"CREATE TABLE\s+(?:IF NOT EXISTS\s+)?{sch}\.", sql, re.I))
    qualified = sum(f[f"db_{s}"] for s in ("multica", "automation", "canvas", "plugin", "permission", "scm"))
    f["db_public"] = f["db_tables"] - qualified

    # REST 路由: 只数生产路由组 (group_api/), 排除测试里的 UUID 字面量
    grp = list((REPO / "crates" / "star-api-rest" / "src" / "group_api").glob("*.rs"))
    gtxt = "\n".join(read_text(p) for p in grp)
    f["rest_routes"] = len([m for m in re.findall(r'"/api/v1/[^"]*"', gtxt) if "00000000-0000" not in m])

    tools = [p for p in (REPO / "crates" / "star-mcp" / "src" / "tools").glob("*.rs") if p.name != "mod.rs"]
    f["mcp_tools"] = len(tools)

    cli = read_text(REPO / "crates" / "star-cli" / "src" / "main.rs")
    f["cli_commands"] = len(set(re.findall(r"^\s{4}([A-Z][A-Za-z]+)\(", cli, re.M)))

    app = REPO / "frontend" / "src" / "app"
    f["fe_pages"] = count_files("page.tsx", app)
    f["fe_components"] = count_files("*.tsx", REPO / "frontend" / "src" / "components") + \
        count_files("*.ts", REPO / "frontend" / "src" / "components")
    f["ci_workflows"] = len(list((REPO / ".github" / "workflows").glob("*.yml")))
    return f


def extract_claims() -> dict[str, list[tuple[pathlib.Path, int]]]:
    """扫描所有页面, 取出 data-fact / data-count 声明.

    注意: data-count 与 data-fact 的先后顺序不固定 (手写 HTML 时两个方向都出现过),
    所以按"整标签"匹配再在标签内分别取属性, 不能用固定顺序的正则。
    """
    out: dict[str, list[tuple[pathlib.Path, int]]] = {}
    tag_re = re.compile(r"<[a-zA-Z][^>]*data-fact=[^>]*>")
    for html in sorted(DOCS.glob("*.html")):
        for tag in tag_re.findall(read_text(html)):
            key = re.search(r'data-fact="([a-z_0-9]+)"', tag)
            num = re.search(r'data-count="(\d+)"', tag)
            if key and num:
                out.setdefault(key.group(1), []).append((html, int(num.group(1))))
    return out


def check(claims: dict, facts: dict, verbose: bool) -> tuple[int, list[str]]:
    problems: list[str] = []
    # 1) 下限守卫 —— 解析退化检测
    for key, (label, floor) in FLOORS.items():
        got = facts.get(key, 0)
        if got == 0:
            problems.append(f"PARSE-FAIL {key} ({label}): 计算结果为 0, 无法区分'真的没有'与'解析失败'")
        elif got < floor:
            problems.append(f"BELOW-FLOOR {key} ({label}): {got} < 下限 {floor}, 疑似解析退化或仓库被裁剪")
    # 2) 页面声明 vs 真实值
    for key, entries in sorted(claims.items()):
        if key not in facts:
            problems.append(f"UNKNOWN-FACT {key}: 页面声明了脚本不认识的 fact key")
            continue
        for path, claimed in entries:
            if claimed != facts[key]:
                problems.append(
                    f"DRIFT {path.name} :: {key}: 页面写 {claimed}, 仓库实测 {facts[key]}")
            elif verbose:
                print(f"  ok  {path.name:<26} {key:<16} = {facts[key]}")
    # 3) 反向: 事实没被任何页面声明 (仅提示, 不算失败)
    unclaimed = [k for k in facts if k not in claims and k in FLOORS]
    if unclaimed and verbose:
        print("  --  未被任何页面引用的 fact: " + ", ".join(sorted(unclaimed)))
    return (1 if problems else 0), problems


def crate_families() -> list[tuple[str, str, list[tuple[str, int]]]]:
    """把 crate 按家族分组, 附 .rs 文件数."""
    wc14 = {"graph-core", "git-adapter", "git-observer", "worktree-service", "worktree-shared-dir",
            "risk-engine", "health-engine", "agent-bridge", "relationship-engine", "canvas-renderer",
            "layout-engine", "query-engine", "action-engine", "event-bus"}
    arg3 = {"arg", "arg-bridge", "arg-effect"}
    views = {"star-task", "star-game", "star-workflow", "star-canvas", "star-scheduler",
             "shared-task", "leads", "star-registry", "star-taskgraph", "agent-domain", "canvas-collab"}
    support = {"api", "application", "infrastructure"}
    surf = {"canvas-engine", "canvas-game", "canvas-realtime", "terminal-stack",
            "terminal-protocol", "ai-vault", "star-desktop"}

    buckets: dict[str, list[tuple[str, int]]] = {
        "domain": [], "star": [], "wasm": [], "canvas": [], "arg": [],
        "views": [], "support": [], "surface": [],
    }
    for d in sorted(p for p in CRATES.iterdir() if p.is_dir()):
        n = d.name
        nrs = sum(1 for _ in d.rglob("*.rs"))
        if n in wc14:
            buckets["canvas"].append((n, nrs))
        elif n in arg3:
            buckets["arg"].append((n, nrs))
        elif n in views:
            buckets["views"].append((n, nrs))
        elif n in support:
            buckets["support"].append((n, nrs))
        elif n in surf:
            buckets["surface"].append((n, nrs))
        elif n.endswith("-wasm"):
            buckets["wasm"].append((n, nrs))
        elif n.startswith("domain-"):
            buckets["domain"].append((n, nrs))
        elif n.startswith("star-"):
            buckets["star"].append((n, nrs))
        else:
            buckets["surface"].append((n, nrs))
    labels = [
        ("domain", "k-domain", "DDD bounded context · 40"),
        ("star", "k-star", "平台能力 star-* · 31"),
        ("canvas", "k-canvas", "worktree-canvas 14 crate"),
        ("views", "k-infra", "多视图 / 任务统一层"),
        ("arg", "k-wasm", "ARG 三层"),
        ("wasm", "k-wasm", "浏览器 WASM 桥 · 3"),
        ("support", "k-infra", "支撑层 (gateway / DTO / adapter)"),
        ("surface", "k-canvas", "终端 / Canvas / 桌面 进程面"),
    ]
    return [(key, cls, label, buckets[key]) for key, cls, label in labels if buckets[key]]


def emit_chips() -> str:
    out = []
    for i, (_key, cls, label, members) in enumerate(crate_families()):
        out.append(f'<div class="famgroup" data-family-group="{_key}">')
        out.append(f'<div class="famlabel"><span class="badge {cls}">{label}</span>'
                   f'<span class="sub"> · {len(members)} crate</span></div>')
        out.append('<div class="cratewall">')
        for name, nrs in members:
            out.append(
                f'<div class="chip {cls}" data-name="{name}" '
                f'title="{name} - {nrs} rust files" style="animation-delay:{i * 40}ms">'
                f'{name}<span style="opacity:.55"> ·{nrs}</span></div>')
        out.append("</div></div>")
    return "\n".join(out)


def self_test(facts: dict, claims: dict) -> int:
    ok = True
    # (a) 变异: 注入错误值, 必须被判为漂移
    mutated = dict(claims)
    first_key = next(iter(mutated), None)
    if first_key is None:
        print("SELF-TEST FAIL: 页面里没有任何 data-fact 声明, 无从变异", file=sys.stderr)
        return 2
    path, real = claims[first_key][0]
    mutated[first_key] = [(path, real + 1)]
    code, probs = check(mutated, facts, False)
    print(f"self-test (a) 注入 +1 到 {first_key}: exit={code} (期望 1)")
    ok &= (code == 1) and any("DRIFT" in x for x in probs)
    # (b) 对照: 不变异, 必须判为一致
    code2, probs2 = check(claims, facts, False)
    print(f"self-test (b) 未变异:            exit={code2} (期望 0)")
    ok &= (code2 == 0)
    # (c) 反例: 伪造"全部 0"的事实集, 必须被判为解析失败而非"干净"
    zeroed = {k: 0 for k in facts}
    code3, probs3 = check(claims, zeroed, False)
    print(f"self-test (c) 事实全 0:          exit={code3} (期望 1, 且报 PARSE-FAIL)")
    ok &= (code3 == 1) and any("PARSE-FAIL" in x for x in probs3)
    print("SELF-TEST " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 2


def emit_bars(top: int = 15) -> str:
    """实装度 Top-N: .rs 文件数最多的 crate. 用于 01 页的条形图."""
    rows: list[tuple[str, int, str]] = []
    for d in sorted(p for p in CRATES.iterdir() if p.is_dir()):
        nrs = sum(1 for _ in d.rglob("*.rs"))
        if nrs:
            rows.append((d.name, nrs, crate_kind(d.name)))
    rows.sort(key=lambda r: -r[1])
    rows = rows[:top]
    mx = rows[0][1] if rows else 1
    out = []
    for name, nrs, kind in rows:
        pct = max(2, round(nrs / mx * 100))
        out.append(
            f'<div class="row"><div class="rn"><span class="badge {kind}">{name}</span></div>'
            f'<div class="bar" data-fill="{pct}"><i></i></div>'
            f'<div class="rv">{nrs} .rs</div></div>')
    return "\n".join(out)


def crate_kind(name: str) -> str:
    if name.endswith("-wasm"):
        return "b-wasm"
    if name in {"graph-core", "git-adapter", "git-observer", "worktree-service", "worktree-shared-dir",
                "risk-engine", "health-engine", "agent-bridge", "relationship-engine", "canvas-renderer",
                "layout-engine", "query-engine", "action-engine", "event-bus", "worktree-canvas"}:
        return "b-domain"
    if name.startswith("domain-"):
        return "b-domain"
    if name.startswith("star-"):
        return "b-star"
    if name in {"api", "application", "infrastructure", "agent-bridge", "star-context", "star-dto"}:
        return "b-infra"
    return "b-mute"


DB_THEMES = [
    ("engineering_run", ["engineering_run", "engineering_directory_event"]),
    ("worktree", ["worktree_"]),
    ("task_lifecycle", ["task_lifecycle", "task_contract", "task_metadata", "task_run_outbox", "task_command_idempotency"]),
    ("execution_run", ["task_execution_"]),
    ("execution_profile", ["agent_execution_"]),
    ("resource", ["project_execution_resource"]),
    ("hook", ["hook_"]),
    ("group_chat", ["group_chat"]),
]


def all_tables() -> list[str]:
    sql = "\n".join(read_text(m) for m in sorted((REPO / "db" / "migrations").glob("*.sql")))
    return re.findall(r"CREATE TABLE\s+(?:IF NOT EXISTS\s+)?([a-z_][a-z_0-9]*(?:\.[a-z_][a-z_0-9]*)?)",
                      sql, re.I)


def emit_db() -> str:
    """multica schema 表按主题分组 (行数 = 实际表数, 条形宽度 = 相对最大值)."""
    tables = [t for t in all_tables() if t.lower().startswith("multica.")]
    short = [t.split(".", 1)[1] for t in tables]
    rows: list[tuple[str, int, list[str]]] = []
    used: set[str] = set()
    for theme, prefixes in DB_THEMES:
        hit = [s for s in short if any(s.startswith(p) for p in prefixes)]
        for h in hit:
            used.add(h)
        if hit:
            rows.append((theme, len(hit), hit))
    rest = [s for s in short if s not in used]
    if rest:
        rows.append(("other", len(rest), rest))
    mx = max((r[1] for r in rows), default=1)
    out = []
    for theme, n, members in rows:
        pct = max(4, round(n / mx * 100))
        sample = ", ".join(f"<code>{m}</code>" for m in sorted(members)[:4])
        more = f" <span class='sub'>+{len(members) - 4} 张</span>" if len(members) > 4 else ""
        out.append(
            f'<div class="row"><div class="rn"><span class="badge b-domain">{theme}</span></div>'
            f'<div class="bar" data-fill="{pct}"><i></i></div>'
            f'<div class="rv">{n} 张</div></div>'
            f'<div class="sub" style="margin:-2px 0 8px">{sample}{more}</div>')
    return "\n".join(out)


def extract_seeds() -> tuple[list[tuple[pathlib.Path, int, str]], list[str]]:
    """扫描所有页面, 取出 data-count 元素的"初始文本"(种子值).

    返回 (匹配到的列表, 解析异常列表)。
    为什么种子值也要管
        动画是装饰, 内容必须默认可见 (README 三条约定之一)。countUp() 会无条件
        覆写 textContent, 所以把初始文本写成真实数字, 对 JS 用户完全无影响,
        却让"禁用 JS / 脚本被拦截"的用户看到真实数字而不是一片 0。
        反过来, 如果种子值是 0, 无 JS 时数字墙就全是 0, 违反上述约定 —— 实测踩过。
        种子值和 data-count 是同一个数, 因此同属本门禁管辖, 不由人手改。
    为什么还要返回"异常列表"
        正则要求元素有闭合标签。若某处 data-count 写法不同(自闭合 / 未闭合),
        会被静默跳过, 门禁照样绿 —— 只防"一个都扫不到"不够, 扫描数必须与
        data-count 属性出现次数逐页相等, 否则就是解析退化 (实测踩过同类坑:
        表解析器漏了一种写法, 静默抽出 0 个, 输出与"仓库干净"无法区分)。
    """
    out: list[tuple[pathlib.Path, int, str]] = []
    bad: list[str] = []
    tag_re = re.compile(r'(<[a-zA-Z][^>]*data-count="(\d+)"[^>]*>)([^<]*)(</[a-zA-Z]+>)')
    for html in sorted(DOCS.glob("*.html")):
        src = read_text(html)
        matched = tag_re.findall(src)
        out.extend((html, int(n), t.strip()) for _o, n, t, _c in matched)
        total = len(re.findall(r'data-count="\d+"', src))
        if len(matched) != total:
            bad.append(f"{html.name}: 出现 {total} 处 data-count, 但只解析出 {len(matched)} 处 "
                       f"—— 有元素不是 <tag ...>text</tag> 形态, 门禁看不到它")
    return out, bad


def check_seeds(sync: bool = False) -> tuple[int, list[str]]:
    """校验 (或用 --sync-seed 修正) data-count 元素的初始文本."""
    problems: list[str] = []
    seeds, bad = extract_seeds()
    problems.extend(bad)
    if not seeds:
        # fail-closed: 一处都扫不到 = 解析退化, 不能当成"本来就干净"
        problems.append("PARSE-FAIL: 页面里扫不到任何 data-count 种子值, 解析可能已退化")
        return 2, problems
    by_file: dict[pathlib.Path, list[tuple[int, str]]] = {}
    for path, num, text in seeds:
        by_file.setdefault(path, []).append((num, text))
    for path, items in by_file.items():
        if not sync:
            for num, text in items:
                if text != str(num):
                    problems.append(
                        f"SEED {path.name}: data-count=\"{num}\" 的初始文本是 \"{text}\", "
                        f"应为 \"{num}\" (无 JS 时会显示这个值)")
            continue
        src = read_text(path)
        fixed, n = re.subn(
            r'(<[a-zA-Z][^>]*data-count="(\d+)"[^>]*>)([^<]*)(</[a-zA-Z]+>)',
            lambda m: m.group(1) + m.group(2) + m.group(4),
            src)
        if n:
            path.write_text(fixed, encoding="utf-8", newline="")
            print(f"  synced {path.name}: {n} 处种子值 -> data-count")
    if sync:
        print(f"PASS: 同步 {len(seeds)} 处种子值")
        return (2 if problems else 0), problems
    if problems:
        for p in problems:
            print(p, file=sys.stderr)
        print(f"\nSEED-FAIL: {len(problems)} 处种子值与 data-count 不一致 (exit="
              f"{2 if any('解析' in p or '只解析出' in p for p in problems) else 1})\n"
              f"  修法: python scripts/automation/arch_live_doc_check.py --sync-seed", file=sys.stderr)
        return (2 if any("只解析出" in p for p in problems) else 1), problems
    print(f"PASS: {len(seeds)} 处 data-count 种子值均等于目标值 (无 JS 也能读到真实数字)")
    return 0, []


BASELINE_RE = re.compile(r"数据基线\s*<code>([0-9a-f]{7,40})</code>|commit\s+([0-9a-f]{7,40})</code>")
README_RE = re.compile(r"\|\s*commit\s*\|\s*`([0-9a-f]{7,40})`")


def head_short() -> str:
    try:
        r = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=str(REPO),
                           capture_output=True, encoding="utf-8", errors="replace", timeout=30)
        return (r.stdout or "").strip()
    except Exception:
        return ""


def baseline_claims() -> tuple[list[tuple[pathlib.Path, str]], list[str]]:
    """取出各页页脚声明的基线 commit.

    基线是 provenance, 不是门禁信号: 数字一致才是真的对得上。所以这里只做
    "提取 + 告警", 不参与 exit code —— 否则任何无关提交都会把门禁弄红,
    久了就没人看它了。但提取失败必须 fail-closed: 扫不出基线串说明页脚格式变了,
    不能当成"基线没问题"。
    """
    claims: list[tuple[pathlib.Path, str]] = []
    bad: list[str] = []
    for html in sorted(DOCS.glob("*.html")):
        src = read_text(html)
        hits = [a or b for a, b in BASELINE_RE.findall(src)]
        if not hits:
            bad.append(f"{html.name}: 页脚里找不到 '数据基线 <code>xxxxxxx</code>', "
                       f"页脚格式可能变了, 无法核对基线")
            continue
        uniq = sorted(set(hits))
        if len(uniq) > 1:
            bad.append(f"{html.name}: 页脚出现多个不同基线 {uniq}, 应只有 1 个")
        claims.append((html, uniq[0]))
    return claims, bad


def rebase_baseline() -> int:
    """把全站基线串更新到当前 HEAD."""
    head = head_short()
    if not head:
        print("FAIL: 读不到 git HEAD (不在 git 仓库里?)", file=sys.stderr)
        return 2
    old = sorted({h for _p, h in baseline_claims()[0]})
    if not old:
        print("FAIL: 页面里找不到任何基线串, 无从替换 (fail-closed)", file=sys.stderr)
        return 2
    if old == [head]:
        print(f"PASS: 基线已是 {head}")
        return 0
    targets = sorted(DOCS.glob("*.html")) + [DOCS / "README.md"]
    total = 0
    for p in targets:
        if not p.exists():
            continue
        src = p.read_text(encoding="utf-8")
        n = 0
        for o in old:
            src, k = re.subn(r"(?<![0-9a-f])" + re.escape(o) + r"(?![0-9a-f])", head, src)
            n += k
        if n:
            p.write_text(src, encoding="utf-8", newline="")
            print(f"  rebased {p.name}: {n} 处 {old} -> {head}")
            total += n
    print(f"PASS: 基线 {old} -> {head} (共 {total} 处)")
    return 0


def check_assets() -> tuple[int, list[str]]:
    """静态校验共享资产: arch.js 必须能被解析, arch.css 关键门控规则必须在。

    为什么要有这条 (实测踩过)
        我在 arch.js 的块注释里写了行首锚点正则, 其中的星号斜杠序列当场结束了
        注释, 后面整段中文被当成 JS 解析 -> SyntaxError -> 整份脚本一行没跑,
        全站交互静默失效。更糟的是数字断言照样是绿的: 种子值已经是终值,
        "JS 跑没跑"在那个断言里根本不可区分。
        语法错误属于最便宜的一类失败, 不该等 Playwright 才发现 —— node --check
        一秒就能判。
    node 不在 PATH 上时只告警不失败: 那是"没测到", 交由交互门禁兜底, 不该让
    没装 node 的机器连数字门禁都跑不了。
    """
    problems: list[str] = []
    js = DOCS / "assets" / "arch.js"
    css = DOCS / "assets" / "arch.css"
    missing = [p.name for p in (js, css) if not p.exists()]
    if missing:
        print(f"FAIL: 共享资产缺失 {missing}", file=sys.stderr)
        return 1, problems

    try:
        r = subprocess.run(["node", "--check", str(js)], capture_output=True,
                           encoding="utf-8", errors="replace", timeout=60)
    except (FileNotFoundError, OSError):
        print("NOTICE: node 不可用, 跳过 arch.js 语法检查 (交由 arch_live_interact_check 兜底)")
        return 0, problems
    if r.returncode != 0:
        for line in (r.stderr or "").splitlines()[:6]:
            problems.append("ASSET-SYNTAX " + line.strip())
        print("FAIL: assets/arch.js 有语法错误 —— 整站脚本不会执行", file=sys.stderr)
        print("      常见原因: 块注释里写了含星号斜杠序列的行首锚点正则, 当场结束注释",
              file=sys.stderr)
        return 1, problems
    print("PASS: assets/arch.js 可被 node --check 解析")

    ctext = read_text(css)
    if "html.anim .rv" not in ctext:
        problems.append("ASSET-CSS: html.anim .rv 门控规则不见了 —— 无 JS 默认可见的约定失效")
    if "prefers-reduced-motion" not in ctext:
        problems.append("ASSET-CSS: prefers-reduced-motion 降级规则不见了")
    if not problems:
        print("PASS: assets/arch.css 的无 JS 可见性 + 动效降级规则都在")
    return 0, problems


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--emit-chips", action="store_true", help="输出 crate 列表 HTML (给 01-crate-map.html 用)")
    ap.add_argument("--emit-bars", action="store_true", help="输出实装度 Top-N 条形图 HTML")
    ap.add_argument("--emit-db", action="store_true", help="输出 multica 表按主题分组 HTML")
    ap.add_argument("--sync-seed", action="store_true", help="把 data-count 元素的初始文本同步为真实数字")
    ap.add_argument("--rebase-baseline", action="store_true", help="把全站基线 commit 串更新到当前 HEAD")
    ap.add_argument("--self-test", action="store_true", help="变异 + 对照 + 反例, 验证门禁有牙")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()

    if not CRATES.is_dir():
        print(f"FAIL: crates dir not found: {CRATES}", file=sys.stderr)
        return 3

    if args.emit_chips:
        print(emit_chips())
        return 0
    if args.emit_bars:
        print(emit_bars())
        return 0
    if args.emit_db:
        print(emit_db())
        return 0
    if args.sync_seed:
        code, problems = check_seeds(sync=True)
        if code:
            print("\n".join(problems), file=sys.stderr)
        return code
    if args.rebase_baseline:
        return rebase_baseline()

    # 资产静态校验最先跑: arch.js 一旦语法错误, 后面所有交互断言都失去意义
    acode, aproblems = check_assets()
    if acode:
        return acode

    facts = compute()
    claims = extract_claims()
    if not claims:
        print("FAIL: 页面里没有任何 data-fact 声明 —— 门禁等于没跑 (fail-closed)", file=sys.stderr)
        return 2

    if args.self_test:
        return self_test(facts, claims)

    print(f"repo facts @ {REPO}")
    for key, (label, _floor) in FLOORS.items():
        print(f"  {key:<15} = {facts.get(key, 0):<8} ({label})")
    print()
    code, problems = check(claims, facts, args.verbose)
    if code == 0:
        print(f"PASS: {sum(len(v) for v in claims.values())} 个页面声明全部与仓库一致")
    else:
        print("\n".join(problems), file=sys.stderr)
        print(f"\nFAIL: {len(problems)} 项 (exit={code})", file=sys.stderr)
        return code
    # 数字一致性过了再查种子值, 两道门禁一起才算过
    scode, sproblems = check_seeds()
    if scode:
        return scode

    # 基线只告警不改退出码 (见 baseline_claims 的说明)
    bl, bbad = baseline_claims()
    for b in bbad:
        print("BASELINE-PARSE-FAIL " + b, file=sys.stderr)
    if bbad:
        return 2
    head = head_short()
    claimed = sorted({h for _p, h in bl})
    if head and claimed != [head]:
        print()
        print(f"NOTICE: 页面基线 {', '.join(claimed)} != 当前 HEAD {head}。"
              f"数字门禁已通过, 说明这些计数在 HEAD 上依然成立;",
              file=sys.stderr)
        print(f"        只是'本图谱描述的是哪个 commit'这句已过期。", file=sys.stderr)
        print(f"        修法: python scripts/automation/arch_live_doc_check.py --rebase-baseline",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
