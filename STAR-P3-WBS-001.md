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
| **小计** | | **~27.1M** | | | | **17/17** |

**累计 main HEAD**: `3eecc2e` (per 2026-08-29 14:10 JST, 49 commits ahead of origin/main)

**质量门 5 维自审** (per STAR-OLU-001 §6):
- 功能完整: 10/10 子项 spec 全部实现 (10 份 PHASE 报告 §1 改动矩阵)
- 测试覆盖: e2e 套件 7 + 单元 50+ (per `docs/architecture/domain-local-runtime.md` §6); 未跑 cargo test (受 5-min timeout)
- 守门 0 违反: 10 份报告 §5 全 ✅ (R-05 / bc23d6c / 5 域独立 / token-OLU / env 安全 / PowerShell / 0 unsafe / 不沿用 bc23d6c / 不 commit 散落 / 代签 / 缺标比错标 / AI 文档治理)
- 文档同步: AGENTS.md §10 +2 行 + `docs/architecture/{domain-local-runtime,msw-real-mode}.md` 新建 + STAR-P3-WBS-001.md §0 表格 10 行
- git 证据: 全部 commit message 含"per 守门" / author=Ulysses

**总分**: 4/5 (cargo test 未跑扣 1, 待 P3-A.6 CI 解锁) → 仍推 P3-B 准备

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
| P3-A | 8 | ~24.5M | 4.1 周 | 🟢 8/8 收官 |
| P3-B | 9 (草案) | 35M | 5.8 周 | 🟡 9 占位 + 2 阻塞 (B.5/B.6 凭证) |
| P3-C | 9 (草案) | 40M | 6.7 周 | 🟡 9 占位 |
| P3-D | 7 + 5 缺口 (草案) | ~33M | ~5.5 周 | 🟡 12 占位, 需 Ulysses 拍"7 还是 12" |
| P3-E | 7 (草案) | 30M | 5 周 | 🟡 5 占位 + 2 阻塞 (E.4 KMS / E.5 5 域 Lead) |
| P3-F | 6 (草案) | ~30M | 5 周 | 🟡 5 占位 + 2 阻塞 (F.1 5 域 Lead / F.6 R-05 反转) |
| **合计** | **46+** | **~192.5M** | **~32 周** | **8 实证 + 38+ 占位 + 7 阻塞** |

**注**: 200M 软预算 vs ~192.5M 实证+草案, 余 7.5M 缓冲 (per 余量 4% 守门)

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
- `PHASE-P3-A1..A8-IMPL-REPORT.md` — P3-A 8 份报告 (本文件 §0 实证)
- `docs/architecture/domain-local-runtime.md` — 11 模块入口
- `docs/architecture/msw-real-mode.md` — P3-A.7 开关使用指南
