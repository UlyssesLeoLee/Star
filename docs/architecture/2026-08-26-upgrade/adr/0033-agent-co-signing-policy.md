# ADR-0033: Agent 代签规则反转 (Co-Signing Policy Reversal)

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-27
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 17:54 JST 发令"你自己 review 签你自己名字"，8/27 07:16 JST 代签规则反转授权）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) · [DEC-008 一人公司 12 角色](0033-...) · [AGENTS.md](../../AGENTS.md)
> **关联**：[PHASE-D2-CLI-IMPL-REPORT.md](../../../PHASE-D2-CLI-IMPL-REPORT.md) · [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) · [PHASE-D4-P1-FIX-REPORT.md](../../../PHASE-D4-P1-FIX-REPORT.md) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) · [STAR-UNTRACKED-CLEANUP-REPORT.md](../../../docs/reports/2026-08-27-untracked-cleanup/STAR-UNTRACKED-CLEANUP-REPORT.md) · [RGS-CROSS-REF-SYNC-REPORT.md](../../../RGS-CROSS-REF-SYNC-REPORT.md) · [RGS-MAVIS-AUDIT.md](../../../RGS-MAVIS-AUDIT.md) · [DTL-036 v1.4 hotfix commit](https://github.com/UlyssesLeoLee/Star/commit/DTL-036-v1.4-hotfix)

---

## 1. 背景与问题

AI 协作场景下，AI agent / 子代理在写文档和 commit 时，是否可以用"代签"形式填写实际责任人 Ulysses 的名字？经历了 3 阶段反转：

### 1.1 阶段 1：旧硬约束（2026-08-26 04:30 JST 确立）

per `DTL-036 v1.4 hotfix` 复盘（2026-08-26 08:40 JST 之前）：

- **不可代签是硬底线** — 修订历史"审批者"列必须 = `—`（待审批）
- **拒绝 AI 编造历史叙事** — 禁"per X 历史形态"回溯
- **引用 BAS 必须 git 实证** — `git log -p --follow RGS-BAS-NNN_*.md`
- **缺标比错标安全** — 显式列"已知缺口"清单
- **子代理授权加 git 实证约束**

**触发案例**（per 2026-08-26 DTL-036 v1.4 hotfix）：worker 编造 BAS-001 v1.1 历史形态出处，Ulysses 一审即发现 P1/P2/P3 三项违规，立即 hotfix。

### 1.2 阶段 2：反转（2026-08-27 07:16 JST 拍板）

per Ulysses 明确指令"今后所有文档允许代签"：

- 覆盖 04:30-08:40 旧"不可代签是硬底线"硬约束（生效窗口 4 小时，已废止）
- 允许 Mavis / 子代理 / worker 在修订历史"审批者"列直接填写"架构师（Mavis 接手 agent per DEC-008）"等真实责任署名
- 不再受"审批者 = —"硬约束
- 代签者 author = Ulysses（`git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`）

### 1.3 阶段 3：显式落 AGENTS.md（2026-08-27 17:36 JST）

per Ulysses 8/27 17:36 JST 发令"改成允许代签 Ulysses"：

- 显式落 `AGENTS.md` 仓库根作为 AI 协作硬约束入口
- 同步正式 ADR `0033-agent-co-signing-policy.md`（本文件）
- 保留派生约束（代签允许 ≠ 编造允许）

### 1.4 阶段 4：用户明确授权升级（2026-08-27 19:39 JST）

per Ulysses 19:39 JST 明确发令"**允许你代签**"：

- 强化阶段 2 + 阶段 3 规则：Mavis 接手默认代签 Ulysses，**无需再问**"我可以签吗？"
- 覆盖 17:54 之前"审批"列 ⏳ 待签硬约束
- 报告"签批"行直接写 🟢 Mavis 接手终审
- 适用所有 STAR / RGS 文档签字 / 修订 / commit / 报告审批
- 保留派生约束（代签允许 ≠ 编造允许）

---

## 2. 决策

**AI agent / 子代理在 STAR 项目（含 RGS 历史 200+ 份文档）中可以代签 Ulysses。Mavis 接手默认代签，无需再问（per 19:39 JST 用户授权）。**

### 2.1 允许代签

| 角色 | 可代签 Ulysses？ | 形式 |
|---|---|---|
| Mavis (root agent) | ✅ 允许 | commit author = `Ulysses <ulysses@mavis.local>`；报告"修订人"列 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` |
| 子代理 (worker / explorer / verifier) | ✅ 允许 | 同上 |
| Mavis 接手 agent | ✅ 允许 | 报告"审批者"列 = `架构师 (Mavis 接手 agent per DEC-008)` |

**覆盖范围**：
- 适用所有 RGS-* / STAR-* / DTL-* / SPEC-* / BAS-* / INTERFACE-REVIEW-* / REPORT-* / PHASE-* 文档
- 适用所有 git commit message + 修订历史表
- 适用 DDD Review 流程签字栏

### 2.2 不可代签底线（**仍然有效**）

代签允许 ≠ 编造允许。**派生约束**（per 8/26 04:30 旧规则保留项）：

| # | 禁止 | 反例 | 证据要求 |
|---|---|---|---|
| 1 | "per X 历史形态" / "per X 升版前/后" / "原本是" 等回溯叙事 | "per DTL-036 v1.3 历史形态..." | 必须 `git log -p --follow <file>` 实证 |
| 2 | 引用 BAS 文档缺 git 实证 | "per BAS-001 v1.1 历史..." | 引用前必须 `git log -p --follow RGS-BAS-NNN_*.md` |
| 3 | 隐性假设断链 | 标"已升版"但未列"已知缺口" | 显式列"已知缺口"清单 (DDD Review 必查) |
| 4 | 子代理授权无证据叙事约束 | 子代理 brief 写"自由改" | 授权边界要写明"无证据叙事 = 禁止" |

**违规案例**：DTL-036 v1.4 hotfix (2026-08-26) — worker 编造 BAS-001 v1.1 历史形态出处，Ulysses 一审即发现 P1/P2/P3 三项违规，立即 hotfix。

---

## 3. 实施清单

### 3.1 已完成（per 2026-08-27 17:36 JST）

- ✅ `AGENTS.md` 仓库根（9 549 字节）— AI 协作硬约束入口
- ✅ 本 ADR `0033-agent-co-signing-policy.md` — 正式决策记录
- ✅ 4 份 Phase D 报告 + STAR 清理报告 + RGS 跨引用报告签字栏应用新形式

### 3.2 4 commit author = Ulysses 实测（per 2026-08-27 17:01 JST）

| Commit | 仓 | Author | 报告审批 |
|---|---|---|---|
| `2a0a68c` fix(cli): P1-1 --json global + P1-2 mr named args (D.4) | STAR feature/ai-ide-compat | Ulysses | 架构师 (Mavis 接手 agent per DEC-008) |
| `1274725` chore(cleanup): STAR 8/26 untracked → .scratch/ | STAR feature/ai-ide-compat | Ulysses | 架构师 (Mavis 接手 agent per DEC-008) |
| `2857e6b` feat(phase-d5+): MCP Streamable HTTP + Resources + Prompts | STAR wt-phase-d5-impl | Ulysses | 架构师 (Mavis 接手 agent per DEC-008) |
| `3bff9c6` fix(rgs-cross-ref): 8 处跨文档引用 Mavis→Ulysses | RGS wt-plan-002-1-2week | Ulysses | 架构师 (Mavis 接手 agent per DEC-008) |

### 3.3 merge commit 链（per 2026-08-27 17:01 JST）

- `d0ed6d8` merge wt-phase-d5-impl → feature/ai-ide-compat (--no-ff)
- `6624417` merge feature/ai-ide-compat → main (--no-ff, 127 files / +16084)

### 3.4 待办

- ⏳ 4 份报告签字栏"审批"列正式签字（per DDD Review 阶段）
- ⏳ 9 wt (fix/acceptance-vcs-blockers / adr-0026-0032 / cli-mcp / api / flows / arch 等) 是否 merge 到 main
- ⏳ RGS 仓是否也建 `AGENTS.md`（per 一致性）
- ⏳ 推 origin (per R-05 反转决策待 Ulysses 拍板)

---

## 4. 影响

### 4.1 workflow 简化

反转前：
```
worker 写 → Ulysses 真实人工审批 → Ulysses 真实人工 commit
```

反转后：
```
worker 写 → Mavis 接手审 → Ulysses 代签 commit → Ulysses DDD Review 一次性终审
```

**省人工** 1 道（DDD Review 一次性审 + 代签 commit 一气呵成）。

### 4.2 历史文档保留

反转**不追溯**改写历史文档的"审批者 = —"项（per 8/27 07:16 JST 明确）。已写"审批 = —"的维持原样，新增文档按新形式写。

### 4.3 子代理授权边界

子代理 brief 模板增加"无证据叙事 = 禁止"硬约束（per 8/26 04:30 派生约束 #4）：

```
## 已知硬约束
- 禁"per X 历史形态"回溯叙事
- 引用 BAS 必须 `git log -p --follow` 实证
- 缺标比错标安全
- 子代理授权边界: 无证据叙事 = 禁止
```

---

## 5. 守门规则

| # | 规则 | 状态 |
|---|---|---|
| 1 | 代签规则应用（commit author = Ulysses, 报告审批 = 架构师 Mavis 接手） | ✅ 4 commit 实测 |
| 2 | 不沿用 bc23d6c 叙事 | ✅ 维持 |
| 3 | 禁"per X 历史形态"回溯叙事 | ✅ 维持 |
| 4 | BAS 引用全部 `git log --follow` 实证 | ✅ 维持 |
| 5 | 缺标比错标安全 | ✅ 维持 |
| 6 | 0 unsafe / 0 新外部依赖（D.5+ 例外，显式反转） | ✅ |
| 7 | R-05 不 push | ✅ 维持 |
| 8 | bc23d6c 保留 | ✅ 维持 |
| 9 | 5 域独立 Lead，不接受兼任 | ✅ 维持 |
| 10 | AI 协作 token-OLU 而非人天 | ✅ 维持 |
| 11 | 环境变量安全 (禁 env value 打印) | ✅ 维持 |
| 12 | PowerShell only (非 bash) | ✅ 维持 |
| 13 | 不 commit 散落子代理产出 (Mavis 终审后统一入库) | ✅ 维持 |
| 14 | 报告 7 段结构 (§0-§7) | ✅ 4 报告 + 2 清理报告齐 |
| 15 | 子代理授权写明"无证据叙事 = 禁止" | ✅ 维持 |

---

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-27 | 🟡 Draft v0.1; 代签规则反转硬约束 + 派生约束保留 + 4 commit author = Ulysses 实测 + merge → main |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手终审通过 (per 2026-08-27 17:54 JST 发令 "你自己 review 签你自己名字" + 8/27 07:16 JST 代签规则反转授权); ADR 0033 3 阶段反转记录 (8/26 04:30 → 8/27 07:16 → 8/27 17:36) + 4 commit 实测 (2a0a68c/1274725/2857e6b/3bff9c6) + 2 merge (d0ed6d8/6624417) 已自审 pass; 派生约束保留 4 项 (禁回溯/BAS 实证/缺标/子代理授权) |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM）| ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：3 阶段反转记录 + 允许代签硬约束 + 4 commit author = Ulysses 实测 + merge → main + 派生约束保留 (禁回溯 / BAS 实证 / 缺标 / 子代理授权) | 2026-08-27 17:36 JST 用户发令"改成允许代签 Ulysses", 显式落 AGENTS.md + 本 ADR |
| v0.2 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 终审签字: §0 签批改 🟢 Mavis 接手终审; §6 签字栏 #1.1 加 Mavis 接手审批行 (2026-08-27); 修订人 / 审批者代签按 8/27 07:16 JST 反转规则 | 2026-08-27 17:54 JST Ulysses 发令"你自己 review 签你自己名字" |
| v0.3 | 2026-08-27 | 架构师 (Mavis 接手 agent per DEC-008) | 用户授权升级: §1.4 新增阶段 4 显式落 19:39 JST 用户授权; §2 决策改"Mavis 接手默认代签无需再问" | 2026-08-27 19:39 JST Ulysses 明确发令"允许你代签" |

---

## 8. 引用文档

- `AGENTS.md` — 仓库根 AI 协作硬约束入口
- `RGS-MAVIS-AUDIT.md` — 子代理 D 灰区判定 (per RGS 历史扩量 commit 139b80a)
- `DTL-036 v1.4 hotfix` — 8/26 04:30 旧硬约束触发案例
- 4 份 Phase D 报告 + STAR 清理报告 + RGS 跨引用报告
- `README.md` — 8/26 升级 README
- [ADR-0021](0021-zero-vendor-cooperation.md) · [ADR-0026](0026-star-ai-compat.md) · [ADR-0027](0027-star-ide-gateway.md) · [ADR-0028](0028-gitgit-compat.md) · [ADR-0029](0029-universal-submit.md) · [ADR-0030](0030-agent-lease-heartbeat-resume.md) · [ADR-0031](0031-context-graph.md) · [ADR-0032](0032-mcp-transport-stdio.md)
