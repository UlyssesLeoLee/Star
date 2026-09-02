# STAR-P3-F-DECISION-PACK P3-F 阶段 6 子项拍板包 (per 2026-08-30 07:47 JST)

> **Status**: 🟡 Draft (P3-A + P3-B + P3-C/P3-D/P3-E 决策包准备, P3-F 同步准备)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)
> **承接**: STAR-P3-WBS-001 §5 P3-F 占位表 (6 子项 / ~30M / ~5 周) + AGENTS.md §4 守门 #12 v15 派生饱和

本文件是 P3-F 阶段 6 子项的拍板包. P3-F 阶段 5 占位 + 2 阻塞 (F.1 5 域 Lead / F.6 推 origin R-05 反转), 累计 30M, 5 周. F.6 实际**已落地** (per `587b212` commit, 2026-08-30 07:09 JST), 决策包标 🟢 替换 🔴 阻塞.

---

## §0 背景

P3-F 阶段软预算 30M tokens / 6 子项 / 5 周 (per STAR-OLU-001 §1 1 SRE·周 = 1.2M).

6 子项当前 5 占位 (TBD) + F.1 阻塞 (5 域 Lead 真人). **F.6 推 origin 已落地** (per `587b212` R-05 反转 + 7 branch 推 https://github.com/UlyssesLeoLee/Star.git, 2026-08-30 07:09 JST), 决策包标 🟢 已完成.

---

## §1 P3-F 拍板包 (6 子项推荐标题)

> **推荐策略**: 5 域 DDD 边界落地 + 跨域集成测试 + 5 域 Lead 真人到位 + 推 origin 已完成. 跟 P3-E 5 域业务子域衔接. 5 域 (**Star 仓独立定义 per `a4b3cb7` RGS 边界硬约束**, player/economy/match/social/admin 不引用 RGS 仓 5 域).

| # | 子项 | 标题(推荐) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| **F.1** | F.1 | **5 域 Lead 真人到位 (DDD Review)** | 4M | 0.7 周 | 无 | 🔴 **阻塞** | **需 Ulysses 找 5 个真人** (per 8/21 JST 拒绝兼任硬约束), 跟 E.5 合并 (per WBS §5 备注) |
| F.2 | F.2 | **跨域集成测试 (5 域 E2E)** | 5M | 0.8 周 | P3-C 收官 | 🟡 占位 | 推荐: 5 域业务子域 (P3-C) 收官后, 跨域集成测试 |
| F.3 | F.3 | **CHANGELOG 跨域汇总** (per D.10 + 5 域 DDD 边界) | 5M | 0.8 周 | 无 | 🟡 占位 | 推荐: 5 域 DDD 边界标记 + CHANGELOG 跨域汇总 |
| F.4 | F.4 | **架构图 mermaid 化 (跨域)** (per D.9 + 5 域 DDD 边界) | 5M | 0.8 周 | 无 | 🟡 占位 | 推荐: 5 域 DDD 边界图 + 跨域 Saga 流程图 |
| F.5 | F.5 | **质量门 5 维全 5 实证** (P3-A 到 P3-F 全阶段) | 5M | 0.8 周 | F.2 + F.3 + F.4 | 🟡 占位 | 推荐: 5 维 (功能完整/测试覆盖/守门 0 违反/文档同步/git 证据) 全 5 阶段实证 |
| **F.6** | F.6 | **推 origin (R-05 反转)** | 1M | 0.2 周 | 所有 P3 | 🟢 **已落地** (per `587b212` 2026-08-30 07:09 JST) | 推 3 branch (main 116 ahead + feature/ai-ide-compat + 6 wt branch) 到 https://github.com/UlyssesLeoLee/Star.git, 守门 #1 v13 release 0 fail 27.2s + tsc exit 0 + author Ulysses + secret 扫描 全过 |
| **小计** | | | **30M** | **5 周** | | **4 占位 + 1 阻塞 + 1 已完成** | |

---

## §2 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 6 子项标题均为草案, 真实范围需 Ulysses 拍板 | 等 Ulysses 决策 |
| 2 | F.1 5 域 Lead 真人到位 (5 个真人 per 8/21 JST 拒绝兼任) | 跨 session 续, 拍板后启动找真人流程 |
| 3 | F.2/F.3/F.4/F.5 依赖 P3-C 收官 (5 域业务子域) | P3-C 启动 + 收官后推进 |
| 4 | F.6 已落地, 但 P3-B/P3-C/P3-D/P3-E 后续 commit 持续推送, 守门 #12 commit-time 同步需持续 | INC-SESSION-003 持续推进 |
| 5 | 软预算为占位估算, 真实 token 待 SRE Lead 接入 telemetry 后回填 | P3-F phase 2 续 |
| 6 | 跨子项依赖图未画 (F.2/F.3/F.4 并行, F.5 串行等 F.2+F.3+F.4) | 等 Ulysses 拍板后回填 |
| 7 | 质量门 5 维全 5 实证 (F.5) 需 P3-A + P3-B + P3-C + P3-D + P3-E 全 5 阶段收官, 当前 P3-A + P3-B 收官, P3-C/D/E 拍板中 | P3-F 启动前 P3-A/B/C/D/E 全收官 |

---

## §3 拍板选项 (Ulysses 一键决定)

### 选项 1: 批准推荐草案 (4 占位 + 1 阻塞 + 1 已完成, 25M / 4.2 周, 推荐)

- 4 占位 (F.2-F.5) 用推荐标题, F.1 阻塞, F.6 已完成
- 软预算 25M (扣 F.6 已落地 1M), 4.2 周
- 触发 4 wt 并行启动 (per 10:58 JST 每子项 1 wt 决策)

### 选项 2: 推迟 P3-F 启动, 等 P3-C/D/E 收官 (DDD Review 阶段全平推)

- 风险: P3-F 强依赖 P3-C 5 域业务子域 + E.5/F.1 5 域 Lead 真人
- 推迟到 P3-C/D/E 收官后启动, 1 phase 全推

### 选项 3: 折中, P3-F 只拍 F.2 跨域集成测试 (1 子项, 5M / 0.8 周)

- 风险: F.3/F.4/F.5 依赖 F.2 跨域测试, 推迟影响后续

### 选项 4: 自定义

- 你给 6 子项的真实标题 + 软预算分配

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- 拍板后 4 wt 并行启动 (F.2-F.5, F.1 阻塞, F.6 已完成) + 子代理 brief 写明"无证据叙事 = 禁止" (per AGENTS §1.2 派生规 4)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作拍板草案 + 推荐, **不实施 P3-F 任何子项**, 等 Ulysses 拍板 | 2026-08-30 07:47 JST Mavis 接手代签 |
| 2 | 每推荐行标 🟡 占位, 拍板后行标 🟢 进行中 | 本文件 §1 状态列 |
| 3 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 4 | 推进门槛是质量门禁 ≥4/5, 不是截止日期 | STAR-OLU-001 §0 |
| 5 | 守门 #12 commit-time 同步 (本文件 commit 即触发, 后续 docs 同步接 v15 派生饱和) | AGENTS §4.1 v15 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft; 6 子项推荐 (5 域 Lead 真人 / 跨域集成测试 / CHANGELOG / 架构图 / 质量门 5 维 / 推 origin 已落地) + 4 拍板选项 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 6 子项推荐 (F.1 5 域 Lead 真人 / F.2 跨域集成测试 / F.3 CHANGELOG / F.4 架构图 / F.5 质量门 5 维 / F.6 推 origin 已落地) + 4 拍板选项 + 已知缺口 7 项 | 2026-08-30 P3-B 7/9 子项收官 + P3-C/D/E 拍板包准备, P3-F 同步准备 |
