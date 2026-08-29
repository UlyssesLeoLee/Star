# STAR-P3-D-DECISION-PACK P3-D 阶段 7 vs 12 范围拍板包 (per 2026-08-30 07:45 JST)

> **Status**: 🟡 Draft (P3-A + P3-B 7/9 子项收官落地, P3-C 拍板包准备中, P3-D 同步准备)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)
> **承接**: STAR-P3-WBS-001 §3 P3-D 占位表 (7 + 5 缺口 / ~33M / ~5.5 周) + AGENTS.md §4 守门 #12 v15 派生饱和

本文件是 P3-D 阶段 7 vs 12 范围拍板包. P3-D 当前 7 子项 + 5 高频缺口 = 12 占位, 累计 ~33M, 接近 35M 软预算. 实际范围需 Ulysses 拍板 "7 还是 12" (per WBS §3 备注).

---

## §0 背景

P3-D 阶段 7 子项原预算 35M, 但 7 + 5 = 12 项高频缺口累计 ~33M, 接近 35M 软预算.

P3-A + P3-B 阶段收官, 留下 7 个 P3-A 已知缺口 + 5 个高频缺口 (per STAR-P3-A-PHASE-CLOSEOUT-REPORT.md §3), 总 12 项 = P3-D 候选项.

---

## §1 P3-A 已知缺口 (7 子项核心 D.1-D.7)

> **D.1-D.7** 是 P3-A 阶段守门 0 违反项 + 已知缺口直接落到 P3-D, 优先级最高.

| # | 子项 | 标题(推荐) | 软预算 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|
| D.1 | D.1 | **w28 切 HubCliRuntime 入口** | 1M | A.4 | 🟡 占位 | per P3-A.4 缺口 #6, HubCliRuntime 已有, w28 切换入口 |
| D.2 | D.2 | **跨平台 e2e 矩阵 (windows/macos)** | 5M | A.6 | 🟡 占位 | per P3-A.6 缺口 #1/#2, ci e2e 扩 windows/macos runner |
| D.3 | D.3 | **frontend e2e (Playwright)** | 6M | 无 | 🟡 占位 | per P3-A.5 缺口 #3, Playwright 已有部分, 扩 e2e |
| D.4 | D.4 | **realFetch error wrapper** | 2M | A.7 | 🟡 占位 | per P3-A.7 缺口 #2, realFetch 统一错误处理 |
| D.5 | D.5 | **agents/analytics/inbox 3 handler real-mode** | 2M | A.7 | 🟡 占位 | per P3-A.7 缺口 #1, MSW handler 切换 |
| D.6 | D.6 | **markdownlint + cargo doc CI job** | 3M | A.6 | 🟡 占位 | per P3-A.8 缺口 #1/#2, 守门 #6 CI 完整实装 |
| D.7 | D.7 | **UserMenu 状态条 (real-mode 提示)** | 2M | D.5 | 🟡 占位 | per P3-A.7 缺口 #6, frontend 视觉提示 |
| **小计** | | | **21M** | | | **D.1-D.7 核心 7 子项** |

---

## §2 高频缺口 (5 子项 D.8-D.12, 可选)

> **D.8-D.12** 是 P3-A 阶段 P3-A 报告 §3 列出 9 个高频缺口里跟 P3-D 阶段相关的 5 项, 加 P3-A.5 缺口 #4 (性能 bench).

| # | 子项 | 标题(推荐) | 软预算 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|
| D.8 | D.8 | **性能 bench (criterion)** | 4M | 无 | 🟡 占位 | per P3-A.5 缺口 #4, Rust 性能 bench |
| D.9 | D.9 | **架构图 mermaid 化** | 2M | A.8 | 🟡 占位 | per P3-A.8 缺口 #3, docs 架构图 |
| D.10 | D.10 | **CHANGELOG.md 自动汇总** | 2M | A.8 | 🟡 占位 | per P3-A.8 缺口 #8, changelog auto-gen |
| D.11 | D.11 | **forwarder broadcast Closed finalizer** | 2M | A.4 | 🟡 占位 | per P3-A.4 缺口 #3, broadcast 资源清理 |
| D.12 | D.12 | **cancel_and_emit 集成 cancel** | 2M | A.4 | 🟡 占位 | per P3-A.4 缺口 #2, cancel token |
| **小计** | | | **12M** | | | **D.8-D.12 5 高频缺口** |

---

## §3 拍板选项

### 选项 1: 7 子项核心 (推荐, 21M / 3.5 周)

- 拍板 D.1-D.7 全部 7 子项, 共 21M tokens / 3.5 周
- 余 14M (35-21) 留给 P3-C 跨阶段 / P3-E 启动 / Buffer
- D.8-D.12 5 高频缺口留 P3-D 收官后增量拍

### 选项 2: 12 子项全部 (33M / 5.5 周, 接近软预算)

- 拍板 D.1-D.12 全部 12 子项, 共 33M tokens / 5.5 周
- 余 2M (35-33) 留给 Buffer, P3-D 阶段拉长
- 高频缺口一站式实装, 后续 P3-E 启动轻

### 选项 3: 7 + 2 选 (D.8 性能 + D.9 架构图, 27M / 4.5 周)

- D.1-D.7 + D.8 + D.9, 共 27M / 4.5 周
- 性能 + 架构图 优先, 其他高频缺口留

### 选项 4: 自定义

- 你给具体范围 (e.g. 7+1 选 D.8 / 7+3 选 D.8-D.10), 我按你的方案实装

---

## §4 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 7 vs 12 范围待 Ulysses 拍板 | 本决策包 |
| 2 | D.6 markdownlint + cargo doc CI 依赖 GitHub Actions runner 配置 (守门 #6 仍未实装) | P3-D 启动前 |
| 3 | D.3 Playwright e2e 跨平台 runner (windows/macos) 需 GitHub Actions 配置 | P3-D 启动前 |
| 4 | D.8 criterion bench 真实性能 baseline 未测 | P3-D 启动后首次跑 |

---

## §5 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- 拍板后 7 wt 并行启动 (per 10:58 JST 每子项 1 wt 决策) + 子代理 brief 写明"无证据叙事 = 禁止" (per AGENTS §1.2 派生规 4)

---

## §6 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作拍板草案 + 推荐, **不实施 P3-D 任何子项**, 等 Ulysses 拍板 | 2026-08-30 07:45 JST Mavis 接手代签 |
| 2 | 每推荐行标 🟡 占位, 拍板后行标 🟢 进行中 | 本文件 §1-§2 状态列 |
| 3 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 4 | 推进门槛是质量门禁 ≥4/5, 不是截止日期 | STAR-OLU-001 §0 |
| 5 | 守门 #12 commit-time 同步 (本文件 commit 即触发, 后续 docs 同步接 v15 派生饱和) | AGENTS §4.1 v15 |

---

## §7 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft; 7 vs 12 范围拍板包 (4 选项) + D.1-D.12 全列 + 软预算 21M / 27M / 33M |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: D.1-D.7 核心 7 子项 (21M) + D.8-D.12 高频缺口 5 子项 (12M) + 4 拍板选项 (7 / 12 / 7+2 / 自定义) + 已知缺口 4 项 | 2026-08-30 P3-B 7/9 子项收官 + P3-C 拍板包准备中, P3-D 同步准备 |
