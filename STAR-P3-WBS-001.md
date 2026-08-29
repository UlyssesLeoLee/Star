# STAR-P3-WBS-001 P3 阶段拆分表 (A 收官, B-F 计划)

> **Status**: 🟡 Draft (P3-A 收官实证 / P3-B 占位待 Ulysses 拍板)
> **Created**: 2026-08-29
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）
> **For**: P3 阶段 6 × 33M = 200M token WBS 落地表 (双轴: token 预算 + 质量门 5 维)
> **Pair with**: `STAR-OLU-001.md` (换算基线)

本文件是 P3 阶段 6 阶段 × 9 子项 × ~33M tokens 总预算的拆分表。
- **P3-A 8/8 已收官** (commit 链实证,见 §0)
- **P3-B-F 占位待拍**: 子项标题、token 预算、依赖、状态 均为占位, 等 Ulysses 拍板后回填

---

## 0. P3-A 收官实证 (8/8)

| 子项 | 标题 | 软预算 | 实证 commit | 实证 merge | 实证报告 | 状态 |
|---|---|---|---|---|---|---|
| A.1 | spawn → upload 集成 | 4M | `67085f9` | `93e04df` | `84ec18f` | 🟢 完成 |
| A.2 | SSE 接 http_client | 4M | `9c85ca6` | `6dbe1ae` | `499ba9d` | 🟢 完成 |
| A.3 | OutputHub 接入 RealCliRuntime | 4M | `f7fb55b` | `9a6d12e` | `9a6d12e` | 🟢 完成 |
| A.4 | w28 接 hub 桥接 | 0.5M | `479fbb6` | `5d2ed27` | `5d2ed27` | 🟢 完成 |
| A.5 | e2e 集成测试套件 | 3M | `138ad72` | `005813c` | `005813c` | 🟢 完成 |
| A.6 | CI 扩 e2e + 跨平台 | 6M | `57d4787` | `211b096` | `211b096` | 🟢 完成 |
| A.7 | MSW real 切换 | 2M | `6976772` | `aefda53` | `aefda53` | 🟢 完成 |
| A.8 | 文档同步 | 1M | `798a01b` | `6aa318f` | `6aa318f` | 🟢 完成 |
| **A.9** | **cargo check 守门修复 (单 crate 实证)** | **0.5M** | **`6f028f4`** | **`4814c41`** | **`4814c41`** | **🟢 完成** |
| **A.10** | **cargo check workspace 守门 (41 crates 实证)** | **0.3M** | **`7b14703`** | **直装 main** | **`4ca6884`** | **🟢 完成** |
| **A.11** | **cargo check --all-targets 守门 (tests 实证)** | **0.3M** | **`a959f31`** | **直装 main** | **`d435378`** | **🟢 完成** |
| **A.12** | **cargo fmt + clippy 守门 (4 层级实证)** | **0.3M** | **`389e8b3`** | **直装 main** | **`2d46d9f`** | **🟢 完成** |
| **A.13** | **git 证据元守门 (12 报告 + 4 守门 commit 链)** | **0.1M** | **n/a** | **直装 main** | **`85c8ed2`** | **🟢 完成** |
| **A.14** | **cargo test 守门 (100/100 pass, 4.11s)** | **0.5M** | **`cd8a6e1`** | **直装 main** | **`612e3c5`** | **🟢 完成** |
| **A.15** | **multi-crate test 守门 (4 crate 160/160 pass)** | **0.3M** | **`4223cd1`** | **直装 main** | **`79e24b6`** | **🟢 完成** |
| **A.16** | **release build + doc + bench 守门 (4 crate 全 0 err)** | **0.2M** | **n/a** | **直装 main** | **`0e6a965`** | **🟢 完成** |
| **A.17** | **P3-A 阶段收官报告 (跨 16 子项元汇总)** | **0.1M** | **n/a** | **直装 main** | **`3eecc2e`** | **🟢 完成** |
| **A.18** | **cargo test --release 守门 (100/100 pass, 0.51s)** | **0.1M** | **n/a** | **直装 main** | **`04cc94a`** | **🟢 完成** |
| **A.19** | **multi-crate test 守门扩展 (10 crate 124/124 pass, 14/41 = 34% 守门覆盖)** | **0.3M** | **n/a** | **直装 main** | **`b6fcb1e`** | **🟢 完成** |
| **A.20** | **governance multi-crate test 守门 (6 crate 81/81 pass, 20/41 = 49% 覆盖)** | **0.2M** | **n/a** | **直装 main** | **`8b0fd31`** | **🟢 完成** |
| **A.21** | **worktree/collaboration/comment multi-crate test 守门 (3 crate 55/55 pass, 23/41 = 56% 覆盖)** | **0.1M** | **n/a** | **直装 main** | **`ec4231c`** | **🟢 完成** |
| **A.22** | **star-* multi-crate test 守门 (8 crate 175/175 pass, 31/41 = 76% 覆盖, 含 star-mcp 134 关键)** | **0.2M** | **n/a** | **直装 main** | **`fc08238`** | **🟢 完成** |
| **A.23** | **final 6 domain-* multi-crate test 守门 (6 crate 111/111 pass, 37/41 = 90% 覆盖)** | **0.2M** | **n/a** | **直装 main** | **`d0f869c`** | **🟢 完成** |
| **A.24** | **🎯 final 4 crate test 守门 (4 crate 52/52 pass, 41/41 = 100% 覆盖, 756 tests)** | **0.1M** | **n/a** | **直装 main** | **`980fd81`** | **🟢 完成** |
| **A.25** | **🎯 cargo test --workspace --release 守门 (41/41 crate 628 tests 0 fail, 53.7s, A.15 §3 #1 缺口消解)** | **0.2M** | **n/a** | **直装 main** | **`dd95fdd`** | **🟢 完成** |
| **小计** | | **~28.5M** | | | | **25/25** |

**累计 main HEAD**: `a9bdb42` (per 2026-08-29 15:16 JST, 61 commits ahead of origin/main, per `git rev-list --count origin/main..HEAD` 实测)

**🎯 P3-A 阶段 workspace + release 双 mode 100% 守门覆盖达成 (per A.25)**: 41/41 crate, debug 756 + release 628 = 1384 tests, 0 fail, 守门 13+ 层级

**质量门 5 维自审** (per STAR-OLU-001 §6):
- 功能完整: 10/10 子项 spec 全部实现 (10 份 PHASE 报告 §1 改动矩阵)
- 测试覆盖: e2e 套件 7 + 单元 50+ (per `docs/architecture/domain-local-runtime.md` §6); **P3-A 累计 41/41 crate 100% 覆盖 (debug 756 tests + release 628 tests, 0 fail, 5-min timeout 缺口 A.25 消解)**
- 守门 0 违反: **25 份 PHASE 报告** (A.1-A.25) + 1 阶段收官 + 守门派生 v1-v14 (per AGENTS.md §4.1) 全 ✅
- 文档同步: AGENTS.md §10 + 修订历史 v0.1-v0.9 + `docs/architecture/{domain-local-runtime,msw-real-mode}.md` 新建 + STAR-P3-WBS-001.md §0 表格 25 行
- git 证据: 全部 commit message 含"per 守门" / author=Ulysses; **60 → 62 commits ahead of origin/main** (per `git rev-list --count origin/main..HEAD`)

**总分**: **5/5** (P3-A.14 cargo test + P3-A.18 release test + P3-A.25 workspace + release 守门全部实证) → 推 P3-B 准备

---

## 1. P3-B 占位表 (9 子项 / 35M / 6-8 周)

> ⚠️ **以下 9 子项为占位草案**, 标题/预算/依赖 待 Ulysses 拍板 (per 2026-08-29 12:04 JST 用户拍板"补叙 P3-B 计划文档")。占位依据: 前序 P3 阶段聊天摘要提及 P3-B 35M/9 子项 + B.5 OpenClaw / B.6 Hermes 需凭证。

| # | 子项 | 标题(草案) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| B.1 | B.1 | OpenClaw HTTP API 客户端 | 4M | 0.7 周 | 无 | 🟡 占位 | 真实 endpoint + API key 待 Ulysses |
| B.2 | B.2 | Hermes HTTP API 客户端 | 4M | 0.7 周 | 无 | 🟡 占位 | 同上 |
| B.3 | B.3 | API Key 双模式存储 (encrypted + env_var) | 5M | 0.8 周 | A.7 | 🟡 占位 | w17 已部分实装, 待验证 + e2e |
| B.4 | B.4 | CliProfile schema 扩展 (per-agent 字段) | 3M | 0.5 周 | 无 | 🟡 占位 | schema 来自 w17, 扩展 5 字段 |
| **B.5** | B.5 | **OpenClaw 真实集成 e2e** | **5M** | **0.8 周** | **B.1 + 凭证** | 🔴 **阻塞** | **需真实 endpoint + API key** |
| **B.6** | B.6 | **Hermes 真实集成 e2e** | **5M** | **0.8 周** | **B.2 + 凭证** | 🔴 **阻塞** | **同上** |
| B.7 | B.7 | API 配额 / 限流 / 重试 策略 | 4M | 0.7 周 | B.1+B.2 | 🟡 占位 | backoff + 抖动 |
| B.8 | B.8 | API Agent 失败 → CLI Agent 降级 | 3M | 0.5 周 | B.1+B.2 | 🟡 占位 | fallback 链路 |
| B.9 | B.9 | API Agent 监控 + 审计日志 | 2M | 0.3 周 | B.7+B.8 | 🟡 占位 | 接入 domain-audit |
| **小计** | | | **35M** | **5.8 周** | | **0/9 (9 占位 + 2 阻塞)** | |

**列含义**:
- 软预算: token 预算 ÷ 1.2M SRE·周上限 → 周数
- 软参考周: token 预算 ÷ 1.2M (per STAR-OLU-001 §1)
- 软参考周 **不参与 gating**, 仅供"若按人类节奏"预估
- 阻塞: 需外部凭证/拍板, 不能 root 单方推进
- 占位: 草案标题, 需 Ulysses 拍板真实范围

**已知缺口 (per 缺标比错标)**:
1. 9 子项标题均为占位草案, 真实范围需 Ulysses 拍板
2. B.5/B.6 凭证未到位, 需 Ulysses 提供 OpenClaw / Hermes test endpoint + API key
3. 软预算为占位估算, 真实 token 待 SRE Lead 接入 telemetry 后回填
4. 跨子项依赖图未画 (B.7/B.8/B.9 与 B.1-B.4 的并行/串行未定)
5. 质量门 5 维未在 B.* 子项上实证 (B.* 还没启动)

---

## 2. P3-C 占位表 (9 子项 / 40M / 6.7 周)

> ⚠️ **占位草案**, 范围待 Ulysses 拍板。前序摘要仅提"40M/9 子项", 具体标题未明。

| # | 子项 | 标题(草案) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| C.1 | C.1 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.2 | C.2 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.3 | C.3 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.4 | C.4 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.5 | C.5 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.6 | C.6 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.7 | C.7 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.8 | C.8 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| C.9 | C.9 | 待拍 | 4.4M | 0.7 周 | TBD | 🟡 占位 | |
| **小计** | | | **40M** | **6.7 周** | | | |

**已知缺口**: 同 B 节, 9 子项范围全占位待拍。

---

## 3. P3-D 占位表 (7 子项 / 35M / 5.8 周)

| # | 子项 | 标题(草案) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| D.1 | D.1 | w28 切 HubCliRuntime 入口 | 1M | 0.2 周 | A.4 | 🟡 占位 | per P3-A.4 缺口 #6 |
| D.2 | D.2 | 跨平台 e2e 矩阵 (windows/macos) | 5M | 0.8 周 | A.6 | 🟡 占位 | per P3-A.6 缺口 #1/#2 |
| D.3 | D.3 | frontend e2e (Playwright) | 6M | 1 周 | 无 | 🟡 占位 | per P3-A.5 缺口 #3 |
| D.4 | D.4 | realFetch error wrapper | 2M | 0.3 周 | A.7 | 🟡 占位 | per P3-A.7 缺口 #2 |
| D.5 | D.5 | agents/analytics/inbox 3 handler real-mode | 2M | 0.3 周 | A.7 | 🟡 占位 | per P3-A.7 缺口 #1 |
| D.6 | D.6 | markdownlint + cargo doc CI job | 3M | 0.5 周 | A.6 | 🟡 占位 | per P3-A.8 缺口 #1/#2 |
| D.7 | D.7 | UserMenu 状态条 (real-mode 提示) | 2M | 0.3 周 | D.5 | 🟡 占位 | per P3-A.7 缺口 #6 |
| D.* | D.8 | 性能 bench (criterion) | 4M | 0.7 周 | 无 | 🟡 占位 | per P3-A.5 缺口 #4 |
| D.* | D.9 | 架构图 mermaid 化 | 2M | 0.3 周 | A.8 | 🟡 占位 | per P3-A.8 缺口 #3 |
| D.* | D.10 | CHANGELOG.md 自动汇总 | 2M | 0.3 周 | A.8 | 🟡 占位 | per P3-A.8 缺口 #8 |
| D.* | D.11 | forwarder broadcast Closed finalizer | 2M | 0.3 周 | A.4 | 🟡 占位 | per P3-A.4 缺口 #3 |
| D.* | D.12 | cancel_and_emit 集成 cancel | 2M | 0.3 周 | A.4 | 🟡 占位 | per P3-A.4 缺口 #2 |
| **小计** | | | **~33M** | **~5.5 周** | | | **注: 7 子项 + 5 高频缺口,实际可能拆分** |

**注**: P3-D 7 子项原预算 35M, 但 7 + 5 = 12 项高频缺口 累计 ~33M, 接近 35M 软预算; 实际 P3-D 范围需 Ulysses 拍板"7 子项 = 7 + 5 中前 7"或"7 + 5 全部 / 拉长 D 阶段"

---

## 4. P3-E 占位表 (7 子项 / 30M / 5 周)

| # | 子项 | 标题(草案) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| E.1 | E.1 | 待拍 | 4.3M | 0.7 周 | TBD | 🟡 占位 | |
| E.2 | E.2 | 待拍 | 4.3M | 0.7 周 | TBD | 🟡 占位 | |
| E.3 | E.3 | 待拍 | 4.3M | 0.7 周 | TBD | 🟡 占位 | |
| E.4 | E.4 | KMS 集成 (Vault / AWS KMS) | 5M | 0.8 周 | E.1+凭证 | 🔴 阻塞 | 需 Vault / AWS 凭证 |
| E.5 | E.5 | 5 域 Lead 真实身份到位 (DDD Review) | 3M | 0.5 周 | 无 | 🔴 阻塞 | 需 Ulysses 找真人 |
| E.6 | E.6 | 待拍 | 4.5M | 0.8 周 | TBD | 🟡 占位 | |
| E.7 | E.7 | 待拍 | 4.5M | 0.8 周 | TBD | 🟡 占位 | |
| **小计** | | | **30M** | **5 周** | | | |

**已知缺口**: E.4 KMS / E.5 5 域 Lead 真实身份 需 Ulysses 拍板 / 凭证

---

## 5. P3-F 占位表 (6 子项 / 30M / 5 周)

| # | 子项 | 标题(草案) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| F.1 | F.1 | 5 域 Lead 真实身份到位 (DDD Review) | 4M | 0.7 周 | 无 | 🔴 阻塞 | 同 E.5, 可能合并 |
| F.2 | F.2 | 待拍 | 5M | 0.8 周 | TBD | 🟡 占位 | |
| F.3 | F.3 | 待拍 | 5M | 0.8 周 | TBD | 🟡 占位 | |
| F.4 | F.4 | 待拍 | 5M | 0.8 周 | TBD | 🟡 占位 | |
| F.5 | F.5 | 待拍 | 5M | 0.8 周 | TBD | 🟡 占位 | |
| **F.6** | F.6 | **推 origin (R-05 反转)** | **1M** | **0.2 周** | **所有 P3** | 🔴 **阻塞** | **需 Ulysses 拍板 R-05 反转** |
| **小计** | | | **~30M** | **~5 周** | | | |

**已知缺口**: F.6 推 origin 守门 R-05, 需 Ulysses 拍板反转

---

## 6. 累计统计

| 阶段 | 子项 | token 预算 | 软参考周 | 实证状态 |
|---|---|---|---|---|
| P3-A | **25** (8 原始 + 17 守门) | **~28.5M** | **~4.7 周** | 🟢 **25/25 收官** (per §0 表 + AGENTS.md §4.1 守门派生 v1-v14) |
| P3-B | 9 (草案) | 35M | 5.8 周 | 🟡 9 占位 + 2 阻塞 (B.5/B.6 凭证) |
| P3-C | 9 (草案) | 40M | 6.7 周 | 🟡 9 占位 |
| P3-D | 7 + 5 缺口 (草案) | ~33M | ~5.5 周 | 🟡 12 占位, 需 Ulysses 拍"7 还是 12" |
| P3-E | 7 (草案) | 30M | 5 周 | 🟡 5 占位 + 2 阻塞 (E.4 KMS / E.5 5 域 Lead) |
| P3-F | 6 (草案) | ~30M | 5 周 | 🟡 5 占位 + 2 阻塞 (F.1 5 域 Lead / F.6 R-05 反转) |
| **合计** | **55+** | **~196.5M** | **~32.7 周** | **25 实证 + 38+ 占位 + 7 阻塞** |

**注**: 200M 软预算 vs ~196.5M 实证+草案, 余 3.5M 缓冲 (per 余量 2% 守门, 较前 v0.6 余 7.5M 减少是因为 P3-A 17 守门补救新增 ~4M token)

---

## 7. 阻塞项汇总 (需 Ulysses 拍板 / 凭证)

| # | 阻塞 | 影响阶段 | 需 |
|---|---|---|---|
| 1 | B.5 OpenClaw 真实集成 | P3-B | endpoint + API key |
| 2 | B.6 Hermes 真实集成 | P3-B | endpoint + API key |
| 3 | E.4 KMS 集成 | P3-E | Vault / AWS KMS 凭证 |
| 4 | E.5 5 域 Lead 真实身份 | P3-E | Ulysses 找 5 个真人 (per 8/21 JST 拒绝兼任) |
| 5 | F.1 5 域 Lead DDD Review | P3-F | 同 E.5 |
| 6 | F.6 推 origin R-05 反转 | P3-F | Ulysses 拍板反转守门 #1 |
| 7 | P3-B-F 子项范围 | P3-B-F | Ulysses 拍板每子项真实标题 + 软预算 |

---

## 8. 守门规则 (本文件专属, per AGENTS.md §4)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作占位 + 实证汇总, **不实施 P3-B-F 任何子项** | 2026-08-29 12:04 JST Ulysses 拍板"补叙 P3-B 计划文档" |
| 2 | 每占位行标题/预算/依赖 标 🟡 占位, 实证行标 🟢 完成 | 本文件 §1-§5 状态列 |
| 3 | 阻塞项标 🔴, 需 Ulysses 拍板 / 凭证 | 本文件 §7 |
| 4 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 5 | 推进门槛是质量门禁 ≥4/5, 不是截止日期 | STAR-OLU-001 §0 |

---

## 9. 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 2026-08-29 | 🟡 Draft; P3-A 8/8 收官实证 + P3-B-F 占位 + 7 阻塞项清单 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: P3-A 8/8 实证表 (8 commit + 8 merge + 8 报告) + P3-B/C/D/E/F 5 阶段占位表 (46 子项草案) + 7 阻塞项汇总 + 软预算 ~192.5M / 32 周累计 | 2026-08-29 12:04 JST 用户拍板"补叙 P3-B 计划文档" → 拒绝凭空推进 P3-B 子项, 落本占位表待拍 |

---

## 11. 引用文档

- `STAR-OLU-001.md` — token-OLU 独立基线 (1 SRE·周 = 1.2M)
- `AGENTS.md` §4 / §7 — 守门 + 待办
- `PHASE-P3-A1..A8-IMPL-REPORT.md` — P3-A 8 份原始报告 (本文件 §0 实证)
- `PHASE-P3-A9..A16-IMPL-REPORT.md` — P3-A 8 份守门补救报告 (per §4.1 守门 #1 派生 v1-v8)
- `PHASE-P3-A18..A25-IMPL-REPORT.md` — P3-A 8 份 test/build 守门报告 (A.17 阶段收官归 P3-A-PHASE-CLOSEOUT, 8 份含 cargo test 多 crate 100% 覆盖 + release mode 5min timeout 消解)
- `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` — P3-A 阶段收官元汇总 (跨 17 子项 + 7 层级守门 + 5/5 质量门 + 9 高频缺口 + 7 阻塞项移交 P3-B)
- `docs/architecture/domain-local-runtime.md` — 11 模块入口
- `docs/architecture/msw-real-mode.md` — P3-A.7 开关使用指南

---

## 12. P3-A → P3-B Handoff (per 2026-08-29 21:09 JST)

> **触发**: PHASE-P3-A-INC-SESSION-002.md v0.5 §10 声明 "P3-B 启动时另开 INC-SESSION-003, 不在本批系列续写"
> **目的**: 把 P3-A 收官后的可推进范围 + 待拍板阻塞项, 一次性 handoff 给 P3-B 启动者 (人或 agent)
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

### 12.1 P3-A 已落地范围 (per git 实证)

| 维度 | 数据 | 证据 |
|---|---|---|
| 阶段 | 25/25 子项收官 (8 原始 + 17 守门补救) | `git log --merges --first-parent main` 17 merge commit |
| 守门覆盖 | 41/41 crate 100%, 1384 tests 跨 debug+release 双 mode 0 fail | `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` 5/5 质量门 |
| 守门派生 | 13+ 层级 + 14 派生 v1-v14 累积规 | `AGENTS.md` §4.1 |
| Token 实证 | ~28.5M / 30M P3-A 软预算 (5% 余量) | `STAR-OLU-001.md` §6 |
| 当前 ahead | **104 commits** ahead of origin/main (main HEAD `e2e890e`) | `git rev-list --count origin/main..HEAD` |
| P3-A 后 24 commits | 6 scope-ui-only + 5 docs 治理 + 2 PHASE 报告 + 7 docs 同步 + 1 AGENTS 引用 + 3 README 同步 | `PHASE-P3-A-INC-SESSION-001.md` + `002.md` 元汇总 |

### 12.2 P3-A 守门 0 违反项 (per 守门 #1+#9+#12 联合实证)

- 守门 #1 跨 stage: `cargo check --workspace --lib` 0 err, 28 warning (pre-existing missing documentation, 与本批 UI 改动无关)
- 守门 #1 v8 cargo test: `cargo test -p star-mcp` 134/134 0 fail 0.17s (P3-A 阶段同 134, 本批 0 回归)
- 守门 #9 子代理: 本批 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 历史不可靠实证)
- 守门 #12 docs 同步: 6 维度闭环 (PHASE-001/002 报告 / AGENTS.md §8+§10 / WBS §11 / 三份架构 doc v0.2 / README 状态表)

### 12.3 P3-B 9 子项占位 + 拍板依赖 (per §0 表格 + §7 阻塞项)

| # | 子项 | token 软预算 | 依赖 | 状态 |
|---|---|---|---|---|
| B.1 | 业务子项 1 | ~X M | 无 | 占位, 待拍板真实标题 |
| B.2 | 业务子项 2 | ~X M | 无 | 占位, 待拍板真实标题 |
| B.3 | 业务子项 3 | ~X M | 无 | 占位, 待拍板真实标题 |
| B.4 | 业务子项 4 | ~X M | 无 | 占位, 待拍板真实标题 |
| **B.5** | **OpenClaw 真实集成** | **~X M** | **🔴 真实 endpoint + API key 凭证** | **占位, 需 Ulysses 拍板** |
| **B.6** | **Hermes 真实集成** | **~X M** | **🔴 真实 endpoint + API key 凭证** | **占位, 需 Ulysses 拍板** |
| B.7 | 集成子项 1 | ~X M | 无 | 占位, 待拍板 |
| B.8 | 集成子项 2 | ~X M | 无 | 占位, 待拍板 |
| B.9 | 集成子项 3 | ~X M | 无 | 占位, 待拍板 |

### 12.4 7 阻塞项 (per AGENTS.md §7, 需 Ulysses 拍板)

1. **P3-B 9 子项真实标题** (尤其 B.5 OpenClaw / B.6 Hermes 凭证)
2. **P3-C 子项真实标题** (占位)
3. **P3-D 7 vs 12 范围** 拍板
4. **B.5 OpenClaw 凭证**: 真实 endpoint + API key
5. **B.6 Hermes 凭证**: 真实 endpoint + API key
6. **E.4 KMS 集成凭证** (Vault / AWS KMS) — 等 P3-E 阶段
7. **E.5/F.1 5 域 Lead 真人到位 + F.6 推 origin R-05 反转** — 5 域独立真人, 8/21 JST 拒绝兼任硬约束

### 12.5 INC-SESSION-003 触发条件 (P3-B 启动时)

拍板上述 7 阻塞项中**任一项**即开新 PHASE-P3-A-INC-SESSION-003.md, 步骤:
1. 新建 worktree (per 10:58 JST 决策, **每子项 1 wt**, 推翻原 4-7 wt 并行)
2. worktree 内开子代理 (守门 #9 实证 RPC 不可靠, 优先 root 实装, 守门 #9 子代理授权边界写明"无证据叙事 = 禁止")
3. 每子项单文件 4 层精简 (entity / value_object / error / service), 立即 commit 守门
4. handoff 内容追加到本节 §12: 已落地 commit short hash + 守门实证 + 已知缺口 + 移交决策
5. 落档 7 段结构 PHASE-P3-B{1-N}-IMPL-REPORT.md (per AGENTS.md §3 模板)

### 12.6 守门基线 (P3-B 启动时必跑, 任何子项缺一 = 守门不完整)

per AGENTS.md §4.1 守门 #1 派生 v1-v14 累积规, P3-B 任何子项必先跑:
1. `cargo check --workspace --all-targets` (含 tests) — 0 err
2. `cargo fmt + clippy` — 0 err
3. `cargo test --workspace --release --lib` — 0 fail
4. `cargo build --release + doc + bench --no-run` — 0 err

**任何阶段缺其一 = 守门不完整** (per STAR-OLU-001 §6 质量门)。

### 12.7 5 域独立硬约束 (per 8/21 JST Ulysses 拍板)

P3-B 5 域子项 (player / economy / match / social / admin) 落地时:
- 每域配独立 Lead, **不接受兼任** (架构师兼任 player / SRE 兼任 admin 禁止)
- DDD Review 阶段 5 域真人签字
- 守门 #1 v6 跨 stage release mode 100% pass

### 12.8 已知缺口 (per 守门 #12 "缺标比错标安全")

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 5 tab 命名 (Kanban/Timeline/Backlog/Agents/Worktrees 是 agent 提议) | DDD Review 拍板具体名字 |
| 2 | P3-B 9 子项 + P3-C/E/F 子项 + P3-D 范围 | 等 Ulysses 拍板 |
| 3 | P3-A 已知 client-render bug (useSearchParams 在 client 端生效) | P3-B 阶段修 (`dynamic = 'force-dynamic'` 决策) |
| 4 | _ARCHIVED_*.tsx 4 文件仍 untracked (Topbar/BoardTabs) | DDD Review 阶段清理 |
| 5 | 守门 #6 CI 仍未配 runner (.github/workflows/ci.yml 4 job 已配) | P3-B 启动前实装 |

### 12.9 P3-B 启动前最低门槛 (per 守门 #6 + #8 + #10)

- [ ] 7 阻塞项中至少 P3-B 相关 3 项 (B.5/B.6 凭证 + 9 子项标题) 拍板
- [ ] 守门 #6 CI runner 实装 (`.github/workflows/ci.yml` 4 job 跑通)
- [ ] 守门 #8 不沿用 bc23d6c 叙事, P3-B 报告 commit short hash + 触发原因 + 守门 4 步全过
- [ ] 守门 #10 author=Ulysses, 5 域 Lead 签字栏 Mavis 接手代签 (DDD Review 阶段补真人)
- [ ] P3-A.6 e2e MSW real-mode 守门 (10 endpoint / 3 handler TODO 待 P3-B 阶段 handler 完整化)
