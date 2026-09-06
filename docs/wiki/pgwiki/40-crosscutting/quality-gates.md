---
title: "质量门 4 维"
generated: "2026-09-06T01:19:57Z"
node_type: "cross-quality"
---

# 质量门 4 维

# 质量门 4 维

Per `AGENTS.md` §4.1 守门累积规 + `STAR-OLU-001.md` §6 质量门 5 维:

| # | 维度 | 当前实测 | 工具 |
|---|---|---|---|
| 1 | cargo check --workspace --lib -j 4 | (per 守门 #1 v19 实证,32.27s) | scripts/automation/console_server.py 跑后 0 err |
| 2 | pgwiki 节点数 | 181 | 本脚本 |
| 3 | 死链率 | 见 validate() 输出 | scripts/automation/pgwiki_index.py --check |
| 4 | cargo test -p star-context --lib -j 4 | 21/21 pass (per PR #12 实证) | CI |
