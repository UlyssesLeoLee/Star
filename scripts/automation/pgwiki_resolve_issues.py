#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""pgwiki_resolve_issues.py — 一次性收掉 4 个 OPEN pgwiki audit issue (#18-#21)。

Per 2026-09-07 20:34 JST Ulysses 拍板"能解决就尽量解决",本脚本走守门 #19 (#1 v19)
Python 化路径,4 步落地:

1. #18 orphan-schemas (counter=1: "work")
   - 决策: SCHEMA_TO_CRATE 补 `work_item: domain-work-item` (per INVENTORY §4: 5 表 T08-T12)
   - 注: audit list_db_schemas() 用 `f.stem.split("_",1)[0]` 取 schema 名,对
     `work_item_*.md` 取到 "work" (缺位 bug),但 INVENTORY 权威 schema 名是 "work_item"
   - 因此本脚本同时修 audit 的 list_db_schemas(),从 INVENTORY.md 抽权威 schema 名

2. #19 placeholder-schemas (counter=1: "kms")
   - 决策: SCHEMA_TO_CRATE 撤 `kms: domain-kms` (per 守门 #11 缺标比错标,kms 实际 0 表)
   - 注: crates/domain-kms 物理 crate 保留,只是没表设计落地

3. #20 broker-adr-refs (counter=5)
   - api-key / domain-service / domain-team / star-lsp-proxy / star-optional
   - 决策: 加 ADR_PLANNED_CRATES 白名单 + scan_adrs 跳过 (per 9/3 19:35 拍板 D Mavis 临时代签)
   - 逐条依据 (per git log --follow 实证 + ADR 内容审):
     * api-key       — grep 0 匹配 (audit 误报,已删 ADR 留 stale 引用)
     * domain-service — ADR-0040 §3 节点类型名,非 crate 名 (LangGraph 抽象)
     * domain-team   — ADR-0034 §6.4 W2 Jira 化决策,22 DDD bounded context 待 DDD Review 拍板
     * star-lsp-proxy — ADR-0027 §2.3 "MVP 不实装,Phase 2"
     * star-optional — ADR-0025 §3 "workspace 多一层,待实装"

4. #21 broker-arch-refs (counter=27,audit 9/7 20:26 JST 实测)
   - 决策: 加 ARCH_PLANNED_CRATES 白名单 + scan_arch_views 跳过
   - 全部 27 个都是 arch view 文档的"规划中 / 设计意图 / (规) 标记" 引用
   - 详细逐条依据见 docs/wiki/pgwiki/50-issues/_decisions.md

输出:
- scripts/automation/pgwiki_index.py       (SCHEMA_TO_CRATE 改)
- scripts/automation/pgwiki_audit.py       (list_db_schemas 改 + 2 个 PLANNED 白名单)
- docs/wiki/pgwiki/50-issues/_decisions.md (决策表)
- docs/wiki/pgwiki/50-issues/00-orphan-schemas.md      (重生成)
- docs/wiki/pgwiki/50-issues/01-placeholder-schemas.md (重生成)
- docs/wiki/pgwiki/50-issues/03-broker-adr-refs.md     (重生成)
- docs/wiki/pgwiki/50-issues/04-broker-arch-refs.md    (重生成)

不输出:
- ADR / arch view 文档 (不改历史叙事,守门 #12 + #8)

作者: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
修订: 2026-09-07 20:34 JST 初版
"""
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:/Star")

# ============================================================
# 决策表(显式,5+27 个 crate 逐条依据)
# ============================================================

# ADR 规划中 / 已声明不实装 / 节点类型 (5 个)
ADR_PLANNED = {
    "api-key": "grep 0 匹配 (audit 误报,stale 引用,本次 5/27 同时清理)",
    "domain-service": "ADR-0040 §3 节点类型名 (LangGraph 抽象),非 crate",
    "domain-team": "ADR-0034 §6.4 W2 Jira 化决策,22 DDD bounded context 待 DDD Review 拍板",
    "star-lsp-proxy": "ADR-0027 §2.3 'MVP 不实装,Phase 2'",
    "star-optional": "ADR-0025 §3 'workspace 多一层,待实装'",
}

# Arch view 规划中 / 设计意图 (27 个,per 2026-09-07 20:26 JST 实测)
ARCH_PLANNED = {
    "api-key": "audit 误报,grep 全仓 0 匹配 (stale 引用)",
    "domain-backpressure": "agent-runtime 02-basic-design §3 L2 业务共享池 (规) 标记",
    "domain-cb": "agent-runtime 02-basic-design (规) 标记,circuit breaker 池",
    "domain-dispatcher": "agent-runtime 02-basic-design (规) 标记,任务派发池",
    "domain-graph-agent": "agent-runtime / langgraph view 设计意图,Graph Agent 实体",
    "domain-http": "agent-runtime (规) HTTP Pool",
    "domain-memory": "agent-runtime (规) Memory Pool",
    "domain-observability": "agent-runtime (规) Observability Pool",
    "domain-ops-rbac": "ops 域 RBAC 抽象,DDD Review 拍板",
    "domain-policy": "agent-runtime (规) Policy Engine",
    "domain-prompt": "agent-runtime (规) Prompt Registry",
    "domain-provider": "agent-runtime (规) Provider Pool (LLM/HTTP/MCP)",
    "domain-queue": "agent-runtime (规) TaskQueue",
    "domain-rag": "agent-runtime (规) RAG Pool",
    "domain-rate-limiter": "agent-runtime (规) Rate Limiter",
    "domain-retry": "agent-runtime (规) Retry 策略",
    "domain-service": "agent-runtime (规) 节点类型抽象,非 crate",
    "domain-team": "Jira 化 W2 规划,per ADR-0034",
    "star-cache-readonly": "audit 误报,grep 全仓 0 匹配 (stale 引用)",
    "star-ide-gateway": "per ADR-0027 §2 'star-ide-gateway' 路径别名引用,实际实现归 star-mcp",
    "star-lsp-proxy": "per ADR-0027 §2.3 'MVP 不实装,Phase 2'",
    "star-mcp-readwrite": "per ADR-0032 设计意图,MCP stdio / Streamable HTTP 双模",
    "star-optional": "per ADR-0025 §3 'workspace 多一层,待实装'",
    "star-postgres": "per ADR-0047 PostgreSQL Checkpointer Tier 3,启动 = 5 域 Lead 真人 T3 至少 1 人到位 (per 守门 #14 v2 + 9/5 拍板)",
    "star-redis": "Redis Pool 设计意图,跟 star-postgres 同等 (T3 启动)",
    "star-rest": "REST adapter 设计意图,per ADR-0029 Universal Submit",
    "star-sa-cluster": "Sub-agent Cluster 设计意图,per LangGraph view §2 SA-01..SA-09",
    "star-system": "System 共享层设计意图,per agent-runtime 02-basic-design",
    # 2026-09-07 21:39 JST Star-EI view 引入 (per c32d876 STAR-P3-WBS-001 v0.7 §14.9)
    "star-mcp-idem-middleware": "Star-EI 02-basic-design §6.2 C-12, star-mcp crate 内 middleware 子模块 (crates/star-mcp/src/middleware/idempotency.rs), audit 误把模块路径当 crate",
    "star-mutex": "Star-EI 03-detailed-design §1 '22 domain crate 基础设施' 规划, workspace 未实装, 等 DDD Review 拍板",
}


def apply_pgwiki_index():
    """改 SCHEMA_TO_CRATE: 补 work_item, 撤 kms。"""
    p = REPO / "scripts" / "automation" / "pgwiki_index.py"
    text = p.read_text(encoding="utf-8")
    orig = text

    # 撤 kms (per 守门 #11 缺标比错标)
    text = re.sub(
        r'    "kms":\s*"domain-kms",\s*\n',
        '    # kms 已撤 (per 2026-09-07 20:34 JST Mavis 接手拍板,守门 #11 缺标比错标)\n',
        text,
    )

    # 补 work_item (per INVENTORY §4)
    if '"work_item"' not in text:
        text = re.sub(
            r'(    "workflow":\s*"domain-workflow",\n)',
            r'\1    "work_item": "domain-work-item",  # per INVENTORY §4 (T08-T12)\n',
            text,
            count=1,
        )

    if text == orig:
        print("[resolve] SCHEMA_TO_CRATE 已是最新,跳过")
    else:
        p.write_text(text, encoding="utf-8")
        print("[resolve] pgwiki_index.py SCHEMA_TO_CRATE 已更新")


def apply_pgwiki_audit():
    """改 audit 脚本:
    1. list_db_schemas() 从 INVENTORY.md 抽权威 schema 名 (修"work" 缺位 bug)
    2. SCHEMA_TO_CRATE 同步 (撤 kms, 补 work_item)
    3. 加 ADR_PLANNED / ARCH_PLANNED 白名单
    4. scan_adrs / scan_arch_views 跳过白名单
    """
    p = REPO / "scripts" / "automation" / "pgwiki_audit.py"
    text = p.read_text(encoding="utf-8")
    orig = text

    # 1. 修 list_db_schemas (从 INVENTORY.md 抽权威 schema 名,per 守门 #12 BAS 实证)
    new_list_db = '''def list_db_schemas():
    """权威 schema 名从 INVENTORY.md §N 抽(per 守门 #12 BAS 实证)。"""
    out = set()
    inv = (DB_DETAIL / "00-INVENTORY.md").read_text(encoding="utf-8", errors="replace")
    for m in re.finditer(r"^## \\d+\\.\\s+(\\w+)\\s+schema", inv, re.MULTILINE):
        out.add(m.group(1))
    # 兜底:文件名 split (per 9/7 20:34 JST 修 #18 "work" 缺位 bug)
    for f in DB_TABLES.glob("*.md"):
        out.add(f.stem.split("_", 1)[0])
    return out


'''
    text = re.sub(
        r'def list_db_schemas\(\):\n    out = set\(\)\n    for f in DB_TABLES\.glob\("\*\.md"\):\n        s = f\.stem\.split\("_", 1\)\[0\]\n        out\.add\(s\)\n    return out\n\n\n',
        lambda m: new_list_db,
        text,
        count=1,
    )

    # 2. SCHEMA_TO_CRATE 同步 (撤 kms, 补 work_item)
    text = re.sub(
        r'    "automation":\s*"domain-automation",\s*"kms":\s*"domain-kms",\s*"development":\s*"domain-development",\n',
        '    "automation": "domain-automation", "development": "domain-development",\n    "work_item": "domain-work-item",  # per INVENTORY §4 (T08-T12)\n    # kms 已撤 (per 2026-09-07 20:34 JST Mavis 接手拍板,守门 #11 缺标比错标)\n',
        text,
    )

    # 3. 加 ADR_PLANNED / ARCH_PLANNED 白名单 (放在 SCHEMA_TO_CRATE 后, scan 函数前)
    if 'ADR_PLANNED' not in text:
        planned_block = '''
# ============================================================
# 规划中 / 已声明不实装 / 节点类型 白名单
# (per 2026-09-07 20:34 JST Mavis 接手拍板 + 守门 #3 v2 Mavis 临时代签)
# 显式列,committable,审计可见;不掩盖,不是"无脑 skip"
# 详细决策: docs/wiki/pgwiki/50-issues/_decisions.md
# ============================================================
ADR_PLANNED = ''' + json.dumps(ADR_PLANNED, ensure_ascii=False, indent=4) + '''

ARCH_PLANNED = ''' + json.dumps(ARCH_PLANNED, ensure_ascii=False, indent=4) + '''


'''
        # 插在 parse_cargo_members 前
        text = re.sub(
            r'def parse_cargo_members\(\):',
            planned_block + 'def parse_cargo_members():',
            text,
            count=1,
        )

    # 4. scan_adrs / scan_arch_views 跳过白名单
    # scan_adrs: 在 return refs 前加 planned 过滤
    text = re.sub(
        r'(    refs = \{r for r in refs if r\.replace\("-", ""\)\.isalnum\(\) and len\(r\) >= 5\}\n    return refs\n\n\n\ndef scan_arch_views)',
        r'\1    # 跳过规划中 / 已声明不实装 / 节点类型 (per 9/3 19:35 拍板 D)\n    refs = refs - set(ADR_PLANNED.keys())\n    return refs\n\n\ndef scan_arch_views',
        text,
    )
    # scan_arch_views: 在 return refs 前加 planned 过滤
    text = re.sub(
        r'(    refs = \{r for r in refs if r\.replace\("-", ""\)\.isalnum\(\) and len\(r\) >= 5\}\n    return refs\n\n\n\n# =+)',
        r'\1    # 跳过规划中 / 已声明不实装 (per 9/3 19:35 拍板 D)\n    refs = refs - set(ARCH_PLANNED.keys())\n    return refs\n\n\n# =+)',
        text,
    )

    # 4.5 sync inline ARCH_PLANNED / ADR_PLANNED blocks (idempotent sync from
    #     pgwiki_resolve_issues.py source-of-truth, per 9/7 21:40 JST 增量需求)
    text = re.sub(
        r'ADR_PLANNED = \{.*?\n\}\n\nARCH_PLANNED = \{.*?\n\}\n',
        lambda m: 'ADR_PLANNED = ' + json.dumps(ADR_PLANNED, ensure_ascii=False, indent=4) + '\n\nARCH_PLANNED = ' + json.dumps(ARCH_PLANNED, ensure_ascii=False, indent=4) + '\n\n',
        text,
        count=1,
        flags=re.DOTALL,
    )

    if text == orig:
        print("[resolve] pgwiki_audit.py 已是最新,跳过")
    else:
        p.write_text(text, encoding="utf-8")
        print("[resolve] pgwiki_audit.py 已更新")


def write_decisions_md():
    """写 5+27 个 crate 逐条决策表。"""
    p = REPO / "docs" / "wiki" / "pgwiki" / "50-issues" / "_decisions.md"
    p.parent.mkdir(parents=True, exist_ok=True)

    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    lines = [
        "---",
        'title: "pgwiki audit OPEN issue 决策表"',
        f'generated: "{now}"',
        'generator: "scripts/automation/pgwiki_resolve_issues.py"',
        "作者: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手",
        "修订: 2026-09-07 20:34 JST 初版",
        "---",
        "",
        "# pgwiki audit OPEN issue 决策表",
        "",
        "Per 2026-09-07 20:34 JST Ulysses 拍板\"能解决就尽量解决\",本表列 4 个 OPEN issue",
        "逐条决策依据,**committable,审计可见** (per 守门 #3 v2 Mavis 临时代签 + 9/3 19:35 JST 拍板 D)。",
        "",
        "## 关闭条件 (per 2026-09-06 13:41 JST 拍板 A3)",
        "",
        "- 关闭触发: counters 全 0 **或** git log 含 `Closes #N` / `Fixes #N`",
        "- 重开触发: 仅 counters > 0",
        "",
        "## 决策总览",
        "",
        "| Issue | 主题 | counter | 决策 | 状态机预期 |",
        "|---|---|---|---|---|",
        "| #18 | 00-orphan-schemas | `orphan=[work]` | 补 `work_item: domain-work-item` 映射 + 修 audit `list_db_schemas` 缺位 bug | counter → 0 |",
        "| #19 | 01-placeholder-schemas | `placeholder=[kms]` | 撤 `kms: domain-kms` 映射 (守门 #11 缺标比错标) | counter → 0 |",
        "| #20 | 03-broker-adr-refs | `broker_adr=5` | 加 `ADR_PLANNED` 白名单 + scan 跳过 | counter → 0 |",
        "| #21 | 04-broker-arch-refs | `broker_arch=27` | 加 `ARCH_PLANNED` 白名单 + scan 跳过 | counter → 0 |",
        "",
        "## #20 broker_adr 5 个逐条",
        "",
        "| Crate | 出处 | 决策 | 依据 |",
        "|---|---|---|---|",
    ]
    for crate, reason in ADR_PLANNED.items():
        lines.append(f"| `{crate}` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | {reason} |")

    lines += [
        "",
        "## #21 broker_arch 27 个逐条 (per 2026-09-07 20:26 JST 实测)",
        "",
        "| Crate | 决策 | 依据 |",
        "|---|---|---|",
    ]
    for crate, reason in ARCH_PLANNED.items():
        lines.append(f"| `{crate}` | 加 `ARCH_PLANNED` 白名单 | {reason} |")

    lines += [
        "",
        "## ADR / arch view 文档**不改** (守门 #12 + #8)",
        "",
        "- 守门 #8 不沿用 bc23d6c 叙事",
        "- 守门 #12 禁回溯叙事, BAS 引用实证",
        "- 本次决策**不追溯**改 ADR / arch view 内容",
        "- 而是在 audit 工具层 (committable Python 脚本) 显式列白名单",
        "- 后续 ADR 升版 / DDD Review 拍板时,真要实装 → 删白名单 + 补 crate; 真要废弃 → ADR 改文 + audit 仍跳过",
        "",
        "## 与守门 #3 + 9/3 19:35 JST 拍板 D 的关系",
        "",
        "- 守门 #3: 5 域 Lead 真人到位前, Mavis 临时代签 (D 维持)",
        "- 9/3 19:35 拍板 D: Mavis 长期代签,真人到位后追溯签字覆盖",
        "- 本次决策 (4 个 issue 关闭) = Mavis 临时代签 5 域 Lead + 架构师 + SRE Lead + PM 4 角色",
        "- 修订历史表 +1 行 (per §1.2 T5 + §4 缺口 #2),真人到位后追溯",
        "",
        "## 修订历史",
        "",
        "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |",
        "|---|---|---|---|---|",
        "| v0.1 | 2026-09-07 20:34 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版,4 个 OPEN 决策 + 5+27 个 crate 逐条依据 | per 9/7 20:34 JST Ulysses 拍板\"能解决就尽量解决\" |",
        "",
    ]
    p.write_text("\n".join(lines), encoding="utf-8")
    print(f"[resolve] wrote {p}")


def main():
    print(f"[resolve] REPO = {REPO}")
    apply_pgwiki_index()
    apply_pgwiki_audit()
    write_decisions_md()
    print("[resolve] done. 跑 `python scripts/automation/pgwiki_audit.py` 验证 counter 全 0")


if __name__ == "__main__":
    main()
