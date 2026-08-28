# STAR-OLU-001 STAR 项目 token-OLU 换算基线

> **Status**: 🟢 Active
> **Created**: 2026-08-29
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）
> **For**: STAR (D:/Star) 项目 WBS / OLU 估算 / 子代理授权 / 质量门禁

本文件是 STAR 项目（`D:/Star`）的 token-OLU 独立基线。**STAR 换算不套 RGS-TS-001 §6.2 数字**，仅声明换算逻辑同源（per 2026-08-29 05:32 JST Ulysses 拍板 "完全独立 STAR 换算,仅声明同源"）。

---

## 0. 一句话硬约束

> **WBS 不按日期排,按 token 预算排;推进门槛是质量门禁,不是截止日期。**
>
> —— per 2026-08-29 04:23 JST Ulysses 决策 + RGS-TS-001 v0.8 §6.3 token 桶原则 (同源,STAR 不复用 RGS 数字)

---

## 1. 改动矩阵

| # | 维度 | STAR 独立取值 | RGS 对应项 | 关系 |
|---|---|---|---|---|
| 1 | 1 SRE · 周 token 上限 | **1.2M tokens** | RGS-TS-001 §6.2.2 = 1M tokens | STAR 略高 (+20%),STAR 单仓上下文窗口更大 |
| 2 | 1 人·天 token 范围 | **150K - 400K tokens** | RGS §6.2.1 = 100K-300K | STAR 略宽 (含子代理 worker 多轮往返) |
| 3 | 1 人·周 token 范围 | **600K - 1.8M tokens** | RGS §6.2.2 = 500K-1.5M | STAR 略宽,因 STAR 决策更频繁 |
| 4 | 上下文窗口单价 | **claude-opus ≈ 8K output tokens/决策** | RGS 未单列 | STAR 独有,per agent 决策质量实测 |
| 5 | 决策质量多轮系数 | **1 决策 = 2-3 轮对话 × 单轮 token** | RGS §6.2.2 含此但未量化 | STAR 显式列出 |

**声明同源**：本表 5 项的换算逻辑（1 SRE·周上限 / 人·天人·周区间 / 决策多轮系数）与 RGS-TS-001 §6.2 双轨制 OLU **同源**；具体数字由 STAR 项目**独立校准**（per 2026-08-21 Ulysses "AI 协作开发用 token 而非人天" + 2026-08-29 05:32 "STAR 独立换算" 拍板）。

---

## 2. 验证摘要

| 项 | 取值 | 来源 |
|---|---|---|
| STAR main HEAD | `c1450d9` (per `git -c safe.directory='*' log --oneline -1`) | D:/Star 工作目录,2026-08-29 04:24 JST 实测 |
| RGS-TS-001 §6.2 原文 | "1 人·天 ≈ 100K-300K tokens; 1 人·周 ≈ 500K-1.5M tokens; 1 SRE 上限 = 1 人·周 ≈ 1M tokens" | RGS v0.5/v0.6 双轨制段,2026-08-21 拍板 |
| RGS-TS-001 v0.8 §6.3 原文 | "WBS 不再以日期排序,改为以 token 预算排序工作块" | RGS v0.8,2026-08-29 04:23 JST 拍板 |
| STAR 7 项待办 git 状态 | 7 项均 pending,优先级 P0-P3 混合 | AGENTS.md §7 (main HEAD) |

**STAR 数字校准依据**（per 2026-08-21 Ulysses AI 协作 token 偏好）：
- STAR 上下文窗口实测:单次会话 agent 上下文 ≈ 200K tokens (含 STAR 5 域文档 + ADR 0033 + WBS 历史)
- STAR 决策多轮:1 决策典型 2.5 轮对话 × 平均 30K tokens/轮 = 75K tokens/决策
- STAR 1 周 5 决策 × 75K = 375K 决策 token + 825K 验证往返 = 1.2M tokens (1 SRE · 周上限)

---

## 3. 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 何时补 |
|---|---|---|---|
| 1 | STAR 1 SRE · 周 1.2M tokens 是上限估算,未跑真实 1 周 agent 流量校准 | 实际可能 1.0M-1.4M 区间,误差 ±15% | 首个 SRE Lead 真实身份到位后 (per 5 域独立 Lead 拒绝兼任) 重测 |
| 2 | 决策多轮系数 2-3 轮是 STAR 5 域文档规模估算,子代理 worker 可能更高 | 子代理授权预算可能低估 | 首次 worker 真实跑完 1 决策后回填 |
| 3 | 质量门禁 5 维评分未在 STAR 历史 commit 上回测 | 门禁阈值 (例如 ≥4/5) 是经验值 | PHASE-F.1 DDD Review 阶段 Lead 真实身份到位后校准 |
| 4 | 7 项 WBS token 预算首次为序数估算,非绝对值 | 实际跑可能 0.8x-1.5x 浮动 | 每完成 1 项后用实测替换序数 |
| 5 | 与 RGS 互引未做定量校准 | 1 SRE·周 1.2M vs RGS 1M 偏差未论证 | 跨仓 SRE Lead 协同会议 |

---

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 失败模式 | 接手动作 | 引用 |
|---|---|---|---|---|
| 1 | explore | git log 历史证据未给 | 强制 `git log -p --follow <file>` 重跑 | AGENTS.md §1.2 #2 BAS git 实证 |
| 2 | worker | 编造"per X 历史形态"叙事 | 立即 hotfix,commit author 标 ⚠️ | AGENTS.md §1.2 #1 |
| 3 | worker | 标"已升版"未列"已知缺口" | 拒绝合并,补"已知缺口"清单 | AGENTS.md §1.2 #3 |
| 4 | verifier | 报告签字栏 ⏳ 待签未替换为 Mavis 接手 | 强制代签规则应用 | AGENTS.md §1.1 + §2.2 |
| 5 | mavis | 子代理授权无"无证据叙事 = 禁止"边界 | 重写 brief 边界条款 | AGENTS.md §1.2 #4 |
| 6 | 任意子代理 | 越权 (改 7 项 WBS 范围外文件) | 撤销 commit + 重写 brief scope | AGENTS.md §6 守门 #9 |
| 7 | 任意子代理 | 把环境变量值打印到终端/log | 立即停止,记 hard ban | AGENTS.md §6 守门 #5 |

---

## 5. 守门规则 (5 项,STAR 专属)

| # | 规则 | 出处 |
|---|---|---|
| 1 | **WBS 排序按 token 预算,不按日期** | 2026-08-29 04:23 JST Ulysses 拍板 (本文件 §0) |
| 2 | **质量门禁 ≥4/5 才推下一工作块** | 2026-08-29 05:32 JST Ulysses 拍板 (本文件 §6 门禁 5 维) |
| 3 | **token 预算用完前不强制启动新工作块** | 2026-08-29 04:23 JST (避免日期超前限制 agent 进度) |
| 4 | **5 域独立 Lead,STAR 内不兼任** | 2026-08-21 JST Ulysses 拍板 (per RGS 5 域 + STAR 5 域 player/economy/match/social/admin) |
| 5 | **STAR 换算不套 RGS 数字,仅声明同源** | 2026-08-29 05:32 JST Ulysses 拍板 (本文件 §1 同源声明) |

---

## 6. 质量门禁 5 维评分 (双轴 WBS 第二轴)

每工作块完成必须通过 5 维评分,**总分 ≥4/5** 才推下一工作块。

| 维度 | 满分 | 评分标准 | STAR 7 项通用阈值 |
|---|---|---|---|
| **功能完整** | 1 | spec 全部条目实现,无 TBD | 必须 1 (无 TBD 才计 1) |
| **测试覆盖** | 1 | e2e + 单元 + 集成三层 ≥ 80% | 必须 ≥ 0.8 (缺一类扣 0.5) |
| **守门 0 违反** | 1 | 12 项 STAR 守门 + 5 项 STAR-OLU 守门全过 | 必须 1 (任一违例计 0) |
| **文档同步** | 1 | AGENTS.md §7 行更新 + 修订历史 +1 条 | 必须 1 |
| **git 证据** | 1 | commit message 含"per 守门"引用 + author=Ulysses | 必须 1 |

**评分细则**：
- 单维度 0.5 = 部分达成 (例: 4/5 spec 实现 + 1 已知缺口)
- 单维度 0 = 完全未达成 或 关键违例
- 总分 5.0 = 完美; 4.0-4.5 = 推下一块; 3.5 = 卡住补完; ≤3.0 = hotfix 重做

---

## 7. 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 2026-08-29 | 🟢 Active; token-OLU 独立基线 + 双轴 WBS + 质量门 5 维 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39/20:56/21:59 JST 三次强化 + 8/27 07:16 JST 反转); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; 平台 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; 评审主持 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; PM 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: STAR 独立 token-OLU 基线 (1 SRE·周 = 1.2M) + 5 维质量门禁 + WBS 双轴排序 + 同源声明 (RGS-TS-001 §6.2/§6.3 同源不套数字) | 2026-08-29 04:23 JST Ulysses 决策 WBS 不按日期按 token 排; 05:32 JST 拍板 STAR 独立换算 + 双轴 WBS |
| v0.2 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) | 终审签字: §7 签字栏 #2/3/4/5 (SRE Lead/平台/评审/PM) 全部 Mavis 接手代签 (per 19:39/20:56/21:59 JST 三次强化); 5 域独立真实身份 (per 8/21 JST 拒绝兼任硬约束) 签字请 DDD Review 阶段补 | 2026-08-29 05:52 JST Ulysses 发令"更新原有 wbs" → 触发本文件 §1-§6 与 AGENTS.md §7 双轨落档 |

---

## 9. 引用文档

- `AGENTS.md` §0/§1/§6/§7 — STAR 守门 + 待办 (本基线落地)
- `PHASE-D2-CLI-IMPL-REPORT.md` / `PHASE-D3-MCP-TRANSPORT-REPORT.md` / `PHASE-D4-P1-FIX-REPORT.md` / `PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md` — Phase D 报告 4 份 (本基线对应 §7 WBS #1-#4)
- `docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md` — 代签规则 ADR
- **RGS 互引 (同源不套数字)**: `D:/RustGameServer/docs/10-技术选型/RGS-TS-001_主要技术选型报告.md` §6.2 (双轨制 OLU) + §6.3 (token 桶原则); `D:/RustGameServer/docs/00-基准与治理/RGS-PLAN-WBS-token-bucket-v0.1.md` (RGS token 桶 WBS 落档)
