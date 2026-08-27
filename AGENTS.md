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
> —— per 2026-08-27 07:16 JST 代签规则反转 + 2026-08-26 AI 协作文档治理规则保留

---

## 1. 代签规则（per 2026-08-27 07:16 JST 反转）

### 1.1 允许代签

| 角色 | 可代签 Ulysses？ | 形式 |
|---|---|---|
| Mavis (root) | ✅ 允许 | commit author = `Ulysses <ulysses@mavis.local>`；报告"修订人"列 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` |
| 子代理 (worker / explorer / verifier) | ✅ 允许 | 同上 |
| Mavis 接手 agent | ✅ 允许 | 报告"审批者"列 = `架构师 (Mavis 接手 agent per DEC-008)` |

**覆盖范围**（per 2026-08-27 07:16 JST 反转）：
- 覆盖 2026-08-26 04:30-08:40 旧"不可代签是硬底线"约束（生效窗口 4 小时，已废止）
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
| 4 | **AI 协作 token-OLU 而非人天** | 2026-08-21 JST | Ulysses 拍板 (1 SRE·周 ≈ 1M tokens, 1 人·天 ≈ 100-300K tokens) |
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

## 7. 待办 (per 当前 main HEAD `6624417`)

| # | 项 | 状态 | 优先级 |
|---|---|---|---|
| 1 | 4 份报告签字栏"审批"列 ⏳ 待 Ulysses DDD Review 终审 | pending | P0 |
| 2 | Streamable HTTP spec 完整实现 (session 重连 / server-push / Last-Event-ID / DELETE) | Phase D.6+ | P2 |
| 3 | Prompts 实际模板 / Resources 独立资源类型 | Phase D.6+ | P2 |
| 4 | 16 tool 真实数据源接入 (现 mock) | Phase D.6+ | P2 |
| 5 | 推 origin (R-05 不 push 反转决策) | 待 Ulysses 拍板 | P1 |
| 6 | 9 个 wt 是否 merge 到 main (acceptance-vcs-blockers / adr-0026-0032 / cli-mcp / api / flows / arch 等) | 已部分在 feature/ai-ide-compat | P1 |
| 7 | 25 domain-* crate 真实数据接入 (现 stub) | Phase 2+ | P3 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：代签规则反转 + 12 项守门 + 报告 7 段结构 + 仓库拓扑 + ADR 索引 + 待办清单 | 2026-08-27 17:36 JST 用户发令"改成允许代签 Ulysses", 显式落 AGENTS.md |

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-27 | 🟢 Active; 代签规则反转硬约束 + 12 项守门 + 报告 7 段结构 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM）| ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

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
