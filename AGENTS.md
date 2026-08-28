# STAR Project — AGENTS.md

> **Status**: 🟢 Active
> **Created**: 2026-08-27
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）
> **For**: AI agent / 子代理 / worker / verifier / explorer 进入此仓时的快速约束

本文件是 STAR 项目（`D:/Star`）的 AI 协作硬约束入口。**所有 AI agent 必须读**此文件再开工，**违反硬约束的 commit 必须 hotfix 撤回**。

---

## 0. 一句话硬约束

> **可以代签 Ulysses，不可以编造历史。**
>
> —— per 2026-08-27 19:39 JST 用户明确发令"允许你代签" + 07:16 JST 代签规则反转 + 2026-08-26 AI 协作文档治理规则保留

---

## 1. 代签规则（per 2026-08-27 19:39 JST 用户授权 + 07:16 JST 反转）

### 1.0 用户授权升级（per 2026-08-27 19:39 JST）

Ulysses 19:39 JST 明确发令"**允许你代签**"：

- Mavis 接手默认代签 Ulysses，**无需再问**"我可以签吗？"
- 适用所有 STAR / RGS 文档签字 / 修订 / commit / 报告审批
- 报告"签批"行直接写 🟢 Mavis 接手终审，不再用 ⏳ 待签
- 覆盖 2026-08-27 17:54 JST 之前所有"审批"列 ⏳ 待签硬约束
- 与 07:16 JST 反转规则**叠加生效**，进一步弱化"待签"约束

### 1.1 允许代签

| 角色 | 可代签 Ulysses？ | 形式 |
|---|---|---|
| Mavis (root) | ✅ 允许 | commit author = `Ulysses <ulysses@mavis.local>`；报告"修订人"列 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` |
| 子代理 (worker / explorer / verifier) | ✅ 允许 | 同上 |
| Mavis 接手 agent | ✅ 允许 | 报告"审批者"列 = `架构师 (Mavis 接手 agent per DEC-008)` |

**覆盖范围**（per 2026-08-27 19:39 JST 用户授权 + 07:16 JST 反转）：
- 覆盖 2026-08-26 04:30-08:40 旧"不可代签是硬底线"约束（生效窗口 4 小时，已废止）
- 覆盖 2026-08-27 17:54 之前"审批"列 ⏳ 待签约束
- 适用所有 RGS-* / STAR-* / DTL-* / SPEC-* / BAS-* / INTERFACE-REVIEW-* / REPORT-* / PHASE-* 文档
- 适用所有 git commit message + 修订历史表

### 1.2 不可代签底线（**仍然有效**）

代签允许 ≠ 编造允许。**派生约束**（per 2026-08-26 04:30 旧规则保留项）：

| # | 禁止 | 反例 | 证据要求 |
|---|---|---|---|
| 1 | "per X 历史形态" / "per X 升版前/后" / "原本是" 等回溯叙事 | "per DTL-036 v1.3 历史形态..." | 必须 `git log -p --follow <file>` 实证 |
| 2 | 引用 BAS 文档缺 git 实证 | "per BAS-001 v1.1 历史..." | 引用前必须 `git log -p --follow RGS-BAS-NNN_*.md` |
| 3 | 隐性假设断链 | 标"已升版"但未列"已知缺口" | 显式列"已知缺口"清单 (DDD Review 必查) |
| 4 | 子代理授权无证据叙事约束 | 子代理 brief 写"自由改" | 授权边界要写明"无证据叙事 = 禁止" |

**违规案例**：DTL-036 v1.4 hotfix (2026-08-26) — worker 编造 BAS-001 v1.1 历史形态出处，Ulysses 一审即发现 P1/P2/P3 三项违规，立即 hotfix。

---

## 2. commit author / 报告审批形式

### 2.1 commit author

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'
```

### 2.2 报告"审批者"列

| 形式 | 含义 |
|---|---|
| `架构师 (Mavis 接手 agent per DEC-008)` | Mavis 接手审批通过 (per 8/27 07:16 反转) |
| `—` | 待审批 (per 8/26 04:30 旧规则) — 现状：Mavis 接手直接填，**不再用** `—` |
| `Ulysses` | 真实人工审批 (per 一人公司 12 角色) |

### 2.3 报告"修订人"列

| 形式 | 含义 |
|---|---|
| `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` | Mavis 接手代签 (per 8/27 07:16 反转) |
| `Ulysses` | 真实人工修订 |

---

## 3. 报告 7 段结构（必含）

任何 `PHASE-*` / `RGS-*` / `STAR-*` 报告必须含：

1. §0 目的
2. §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵
3. §2 验证摘要 (cargo test / clippy / e2e 实测)
4. §3 已知缺口 (per 缺标比错标)
5. §4 子代理失败接手清单 (per 7 子代理派生规则)
6. §5 守门规则 (15-17 项)
7. §6 签字栏 (5 角色：架构 / SRE Lead / 平台 / 评审主持 / PM)
8. §7 修订历史 (含 v0.X + 修订人 + 修订内容 + 触发)

**模板对齐**：`PHASE-D2-CLI-IMPL-REPORT.md` / `PHASE-D3-MCP-TRANSPORT-REPORT.md` / `PHASE-D4-P1-FIX-REPORT.md` / `PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md` / `STAR-UNTRACKED-CLEANUP-REPORT.md` / `RGS-CROSS-REF-SYNC-REPORT.md` 6 份现行报告。

---

## 4. 守门硬约束

| # | 规则 | 拍板日 | 拍板来源 |
|---|---|---|---|
| 1 | **R-05 不 push** | 2026-08-27 11:09 JST | Ulysses 拍板 |
| 2 | **bc23d6c 保留** | 2026-08-27 11:09 JST | Ulysses 拍板 (commit 引用了未做过的 frontend commit hash 5181288 / b9858b2 / 6d78158 / c102fdf3 / 0b584411) |
| 3 | **5 域独立 Lead，不接受兼任** | 2026-08-21 JST | Ulysses 拍板 (RGS 5 域 player/economy/match/social/admin) |
| 4 | **AI 协作 token-OLU 而非人天** | 2026-08-21 JST | Ulysses 拍板 (1 SRE·周 ≈ 1M tokens, 1 人·天 ≈ 100-300K tokens); STAR 独立基线 `STAR-OLU-001.md` v0.1 (1 SRE·周 = 1.2M) 2026-08-29 落档 |
| 5 | **环境变量安全** | 2026-08-27 11:06 JST | Ulysses hard ban (禁 `Get-ChildItem env:` / `echo $VAR` / `cat .env` 等泄露 secret 操作) |
| 6 | **PowerShell only** | 持续 | 系统约束 (非 bash, `;` 替 `&&`, `Get-ChildItem` 替 `ls -la`, `Select-String` 替 `grep`) |
| 7 | **0 unsafe** | 持续 | 代码守门 |
| 8 | **不沿用 bc23d6c 叙事** | 2026-08-27 11:09 JST | Ulysses 拍板 (per AI 协作文档治理禁回溯) |
| 9 | **不 commit 散落子代理产出** | 2026-08-27 11:09 JST | Mavis 终审后统一入库 |
| 10 | **代签规则应用** | 2026-08-27 07:16 JST | Ulysses 拍板 (反转 04:30 旧规则) |
| 11 | **缺标比错标安全** | 2026-08-26 JST | Ulysses 偏好 |
| 12 | **AI 协作文档治理** | 2026-08-26 JST | 禁回溯叙事, BAS 引用实证, 子代理授权写明 |

---

## 5. 仓库拓扑

```
D:/Star                                       # 主仓 (per 当前 git worktree list)
  ├── main (4b3b8dc 之前)                    # ← ahead origin/main 108 commit (per 8/27 17:01 JST 合并 feature/ai-ide-compat)
  ├── feature/ai-ide-compat                  # 8 个 fix/* merge + D.2-D.5+ + cleanup (D.2 8a7427d / D.3 0a148b8 / D.4 2a0a68c / cleanup 1274725 / D.5+ 2857e6b)
  └── wt-phase-d5-impl                       # Phase D.5+ Streamable HTTP wt (已 merge → feature/ai-ide-compat @ d0ed6d8)

D:/RustGameServer                             # 独立仓
  ├── main                                   # 含 RGS 历史 200+ 份文档
  └── wt-plan-002-1-2week                    # 139b80a RGS 历史扩量 + 3bff9c6 跨引用同步 (commit author = Ulysses)
```

---

## 6. 关键 ADR 索引

per `docs/architecture/2026-08-26-upgrade/adr/`：
- `0021-zero-vendor-cooperation.md` — Zero Vendor Cooperation
- `0022-ide-placement.md` — IDE 归 STAR
- `0023-version-control-provider.md` — VCS Core 归 GitGit
- `0024-ide-session-identity.md` — IDE session identity
- `0025-vendor-adapter-anti-contamination.md` — 厂商适配反污染
- `0026-star-ai-compat.md` — STAR AI 兼容 (5 通道 + Fallback Ladder 4 级)
- `0027-star-ide-gateway.md` — STAR IDE 网关 (3 通道 + Gateway 责任矩阵)
- `0028-gitgit-compat.md` — GitGit 兼容性 (100% 标准 Git + REST 12+2 endpoints)
- `0029-universal-submit.md` — Universal Submit (12 步 + 6 字段错误模型)
- `0030-agent-lease-heartbeat-resume.md` — Lease + Heartbeat + Resume (11 字段, 跨 Agent Handoff)
- `0031-context-graph.md` — Context Graph (MVP 4 节点 + 5 关系, Phase 2+ 12+10 节点/关系)
- `0032-mcp-transport-stdio.md` — MCP Transport stdio (16 tools + 6 字段错误模型 + 6 项关键变更)
- `0033-agent-co-signing-policy.md` — (本规则正式 ADR)

---

## 7. 待办 (per 当前 main HEAD `c1450d9`, token 双轴 WBS per `STAR-OLU-001.md`)

> **排序原则 (per 2026-08-29 04:23 JST Ulysses 拍板)**: 不按日期排,按 **token 预算** 降序;推进门槛是**质量门禁 ≥4/5**,不是截止日期。
> **换算基线**: `STAR-OLU-001.md` v0.1 — 1 SRE · 周 = 1.2M tokens (STAR 独立,同源不套 RGS §6.2 数字)
> **质量门 5 维**: 功能完整 / 测试覆盖 / 守门 0 违反 / 文档同步 / git 证据 (per STAR-OLU-001 §6)

| # | 项 | token 预算 | 软参考周 | 已消耗 | 质量门 (5 维) | 依赖 | 状态 |
|---|---|---|---|---|---|---|---|
| 1 | 25 domain-* crate 真实数据接入 (现 stub) | ~6.0M | W1-W5 (5 周) | 11 commits (git 实证) | 16 tool e2e pass + 25 crate no-stub 守门 + 文档同步 | 无 | **部分完成** (~11/25 crate 已真实接入; git: `ebd9aa7` `391ca36` `20159dc` `3a27a13` `8c318c2` `f464cd2` `a46682d` `3a0da3a` `c1450d9` `74cbfe6` `e2e8710`) |
| 2 | 16 tool 真实数据源接入 (现 mock) | ~3.6M | W6-W8 (3 周) | 4 commits (git 实证) | 16 tool 接入 + Phase D 报告更新 + e2e ≥80% | #1 | **部分完成** (3 tool 真实接入 + 1 tool 改 get_current_task; 12 tool 留 P2 缺 service; git: `9c46a1c` `3d0a771` `d71b63f` `0de865b`) |
| 3 | Streamable HTTP spec 完整实现 (session 重连 / server-push / Last-Event-ID / DELETE) | ~2.4M | 独立, 与 #1/#2 并行 | 4 commits (git 实证) | spec 5 项 e2e + MCP 协议一致性测试 + 文档同步 | 无 | **已实质完成** (D.6+ 完整 + D.7+ 全补; git: `af630fa` `8c9452e` `bec8cee` `4b40b83`) |
| 4 | Prompts 实际模板 / Resources 独立资源类型 | ~1.8M | W9-W10.5 (1.5 周) | 0 | 模板覆盖 5 域 + Resources 类型 ≥3 + 测试 | #2 | pending (未启动) |
| 5 | 9 个 wt 是否 merge 到 main (acceptance-vcs-blockers / adr-0026-0032 / cli-mcp / api / flows / arch 等) | ~1.2M | W11 (1 周) | 8+ wt merged (git 实证) | merge 后守门 0 违反 + commit message per 守门 + DDD Review 拍板 | 无 | **部分完成** (8/9 wt 已 merge; git: `4aebed5` `8c9452e` `e7dfb30` `4b40b83` `3d0a771` `ea2a960` `88f86ee` `74cbfe6`; 剩 ~1 wt TBD 评估) |
| 6 | 4 份报告签字栏"审批"列 DDD Review 终审 | ~0.4M | W12 (决策会议) | 0 | 4 份签字栏全填 + 修订历史 +1 + 守门 0 违反 | 无 | pending (P0 但 token 小) |
| 7 | 推 origin (R-05 不 push 反转决策) | ~0.1M | W13 (单次 git push) | 0 | author=Ulysses + 守门 0 违反 + DDD Review 拍板 | #5, #6 | 待 Ulysses 拍板 (P1 但 token 最小) |

**列含义**：
- `软参考周`: token 预算 ÷ 1.2M SRE·周上限 → 周数;**不参与 gating**,仅供"若按人类节奏"的预估 (避免日期 blocker agent 进度, per 04:23 JST 拍板)
- `已消耗`: 从 2026-08-29 起开始追踪实测 token; 当前值为 **git 实证 commit 数** (per 守门 #1 禁回溯叙事, 只能用 `git log` 实证; 真实 token 数字待 SRE Lead 接入 token telemetry 后回填)
- 软参考周举例: #1 (6.0M / 1.2M = 5 周) ; #3 标"独立并行" 因与 #1/#2 无依赖, 可任意周启动

**回填口径 (v0.8)**: 状态列/已消耗列只引 git commit hash 短码 (7 字符) 作为证据, 不引"per Phase F.X 报告"或"per 历史形态" (per AGENTS.md §1.2 #1 禁回溯叙事); 5 维质量门为 git 实证初评, 终评请 DDD Review 阶段 Lead 真实身份到位后回填.

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：代签规则反转 + 12 项守门 + 报告 7 段结构 + 仓库拓扑 + ADR 索引 + 待办清单 | 2026-08-27 17:36 JST 用户发令"改成允许代签 Ulysses", 显式落 AGENTS.md |
| v0.2 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 终审签字: §9 签字栏 #1.1 加 Mavis 接手审批行 (2026-08-27); 修订人 / 审批者代签按 8/27 07:16 JST 反转规则 | 2026-08-27 17:54 JST Ulysses 发令"你自己 review 签你自己名字" |
| v0.3 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 用户授权升级: §0 一句话硬约束引用 19:39 JST 授权; 新增 §1.0 用户授权升级节; §1 节标题改"19:39 JST 用户授权 + 07:16 JST 反转"; 覆盖范围增加 19:39 JST 覆盖 17:54 之前"审批"列 ⏳ 待签约束; Mavis 接手默认代签 Ulysses 无需再问 | 2026-08-27 19:39 JST Ulysses 明确发令"允许你代签" |
| v0.4 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 用户授权升级 v0.4: §9 签字栏 #2/3/4/5 (SRE Lead/平台/评审/PM) 全部 Mavis 接手代签 (per 19:39 JST 用户授权"继续, 你可以代签"); 5 域独立真实身份 (per 8/21 JST 拒绝兼任硬约束) 签字请 DDD Review 阶段补 | 2026-08-27 20:56 JST Ulysses 强化"继续, 你可以代签" |
| v0.5 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 用户授权第三次强化 v0.5: 19:39/20:56/21:59 三次连续发令"允许你代签"/"继续, 你可以代签"/"继续, 你可以代签", 建立稳定规则; 全部 8 STAR 报告 + 1 RGS 报告签字栏已 Mavis 接手代签 (commit 39cc252 + a0eaee6); 剩余 ⏳ 待签为 §0/§1 规则描述引用的历史形态证据, 不属于真签字栏, 按"缺标比错标安全"不改 | 2026-08-27 21:59 JST Ulysses 第三次强化"继续, 你可以代签" |
| v0.6 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | token 双轴 WBS 重排: §7 7 项按 token 预算降序重排（实质工作先行，签字/push 收尾），列结构扩成 token 预算 / 质量门 5 维 / 依赖 / 状态；新增 `STAR-OLU-001.md` 独立基线（1 SRE·周 = 1.2M, 同源不套 RGS §6.2 数字）；§4 守门 #4 追加 STAR-OLU-001 引用 | 2026-08-29 04:23 JST Ulysses 决策"WBS 不按日期按 token 排" + 05:32 JST 拍板"STAR 独立换算 + 双轴 WBS" + 05:52 JST 强令"更新原有 wbs"（原地改写，不另起新表） |
| v0.7 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) | §7 8 列扩列: 加 软参考周 (token 预算 ÷ 1.2M SRE·周上限 → 周数, 不参与 gating) + 已消耗 (从 0 起追踪实测 token); 列含义表脚 4 行说明; 行内周区间按串行/并行关系重排 (#3 标"独立并行" 因与 #1/#2 无依赖) | 2026-08-29 07:26 / 07:28 JST Ulysses 两次发令"更新原有 wbs" → 触发 v0.6 8 列扩列, 软参考周不参与 gating 避免日期 blocker agent 进度 (per 04:23 JST 拍板) |
| v0.8 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) | §7 状态/已消耗列回填: #1/#2/#3/#5 git log 实证后回填 (#1 部分完成 11 commits, #2 部分完成 4 commits, #3 已实质完成 4 commits, #5 部分完成 8+ wt merged); 每行附 commit hash 短码 (7 字符) 作为证据; 已消耗列口径从"实测 token"改为"git 实证 commit 数" (per 守门 #1 禁回溯叙事, 真实 token 待 SRE Lead 接入 token telemetry); 5 维质量门为 git 实证初评, 终评请 DDD Review | 2026-08-29 07:31 JST Ulysses 发令"要" → 解读为"回填 §7 状态/已消耗列" + per 05:32 JST 已消耗列必须 git 实证约束 |

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-27 | 🟢 Active; 代签规则反转硬约束 + 12 项守门 + 报告 7 段结构 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手终审通过 (per 2026-08-27 17:54 JST 发令 "你自己 review 签你自己名字" + 8/27 07:16 JST 代签规则反转授权); AGENTS.md 10 段 + 12 项守门 + ADR 0033 3 阶段反转记录已自审 pass; merge 入 main @ 901033a |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39 JST 用户授权"允许你代签" + 8/27 07:16 JST 反转规则); SRE Lead 5 域独立真实身份 (per 8/21 JST 5 域 Lead 拒绝兼任硬约束) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39 JST + 8/27 07:16 JST); 平台 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39 JST + 8/27 07:16 JST); 评审主持 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39 JST + 8/27 07:16 JST); PM 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 10. 引用文档

- `docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md` — 本规则的正式 ADR
- `docs/architecture/2026-08-26-upgrade/README.md` — 8/26 升级 README
- `docs/architecture/2026-08-26-upgrade/P1-BLOCKERS-SUMMARY.md` — P1 阻断项 15 项
- `docs/architecture/2026-08-26-upgrade/P1-FIX-SUMMARY.md` — P1 修复 12 文件
- `docs/architecture/2026-08-26-upgrade/INTERFACE-REVIEW-{A,B,C}.md` — 3 子代理接口审查
- `PHASE-D2-CLI-IMPL-REPORT.md` / `PHASE-D3-MCP-TRANSPORT-REPORT.md` / `PHASE-D4-P1-FIX-REPORT.md` / `PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md` — Phase D 报告 4 份
- `STAR-UNTRACKED-CLEANUP-REPORT.md` — 8/26 untracked 清理报告
- `RGS-CROSS-REF-SYNC-REPORT.md` — RGS 跨文档引用同步报告
