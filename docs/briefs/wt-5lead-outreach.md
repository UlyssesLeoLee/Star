# Brief wt-5lead-outreach: 5 域 Lead 真人寻访 Ulysses 内推启动 (per docs/recruitment/5-business-domain-lead-referral.md v0.1)

> **Status**: 🟡 Active (per 2026-09-09 22:31 JST 用户发令"开子代理和worktree并行处理" + ask_4b06eee1bba60b2727e8bccb 拍板 4 个 wt 并行 + 逐个 rebase + ff merge, 推荐项)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **Worktree**: `wt-5lead-outreach` (新, 轻量)
> **依赖**: docs/recruitment/5-business-domain-lead-referral.md v0.1 (9.5KB, 5 域各 1 份 Ulysses 内推话术 + token-OLU 11-15 SRE·周估 + 6 周满员 timeline T0-T5) + per 2026-09-05 10:43 JST `ask_409cbd32edc309d71a083e2a` 用户拍板 Q1=Ulysses 内推[推荐]+Q2=立即启动[推荐]
> **跨 session 续**: per 守门 #20 v20 + 守门 #27 v27

---

## 0. 任务目标 (Objective)

在 `wt-5lead-outreach` worktree 启动 5 域 Lead 真人寻访流程, **Ulysses 内推 = 信任度最高 / 1-1 沟通 / 强契合 / ~2 周可到位 (per 候选 1) / 备选 Freelance + 开源社区招募 兜底**, 覆盖 5 域: player / economy / match / social / admin, 立即启动 (per Q2 拍板).

**关键**: 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D), 真人到位后追溯签字覆盖修订历史 (per 守门 #14 v3).

## 1. 已知事实 (Known Facts)

- **recruitment/5-business-domain-lead-referral.md v0.1 已落档** (commit 在 v0.76 AGENTS.md, 9.5KB):
  - 8 节结构 + 5 域各 1 份 Ulysses 内推话术
  - token-OLU 11-15 SRE·周估
  - 6 周满员 timeline T0-T5
- **5 域**: player / economy / match / social / admin (per AGENTS.md §4 #3 5 域独立 Lead, 拒绝兼任)
- **拍板 D**: Mavis 长期代签, 真人到位后追溯签字覆盖修订历史 (per 9/3 19:35 JST + 9/5 10:43 JST)
- **5 域 Lead 内推 brief 模板** (per守门 #14 v25 e): docs/recruitment/5-business-domain-lead-referral.md
- **G-DEP-08 PostgreSQL checkpointer Tier 3 启动 = 真人到位后 T3 至少 1 人到位触发**
- **5 域 Lead Subagent dispatch brief** (`docs/briefs/5-leads/{id}.md`) 跟真人 Lead 责任边界不清, 待 DDD Review 拍板 (per §4 缺口 #3 P1)
- **scripts/automation/lead_outreach.py 6.8KB 已落档** (per 9/2 commit) — 5 域内推话术模板
- **守门 #5 env 安全** (per 8/27 11:06 JST hard ban): 5 域 Lead 联系信息 (email / phone) 不打印到日志

## 2. 路径 (Ruled-out Paths)

- ❌ **公开招聘** (守门 #14 v25 a 推荐 Ulysses 内推, 不走公开招聘)
- ❌ **Freelance 兜底** (per 内推 brief, 备选, 暂不启动)
- ❌ **开源社区招募** (per 内推 brief, 备选, 暂不启动)
- ❌ **直接派 5 域 Lead 子代理** (per §4 缺口 #3 P1, 真人 Lead 责任边界不清, 待 DDD Review)
- ❌ **打印 5 域 Lead 联系方式到终端** (守门 #5 env 安全 hard ban)

## 3. 范围 (Exact Scope)

### 3.1 在 worktree `wt-5lead-outreach` 新建 / 修改

```
docs/recruitment/
├── 5-business-domain-lead-referral.md v0.1   # 已有 (per v0.76 commit)
├── 5-business-domain-lead-referral.md v0.2   # 🆕 启动 + 内推话术准备 + 6 周 timeline 表
├── outreach_log/                              # 🆕 内推 contact log (private, 不入 git)
│   ├── README.md                              # 目录说明 (committed)
│   └── .gitignore                             # private 屏蔽 (committed)
└── status/
    ├── player.md                              # 🆕 player 域寻访状态 (per 周更新)
    ├── economy.md                             # 🆕 economy 域寻访状态
    ├── match.md                               # 🆕 match 域寻访状态
    ├── social.md                              # 🆕 social 域寻访状态
    └── admin.md                               # 🆕 admin 域寻访状态

scripts/automation/lead_outreach.py v0.2       # 🆕 5 域内推 + contact log + 邮件草稿 (per 守门 #19 v19)

docs/briefs/5-leads/                            # 🆕 5 域 Lead Subagent dispatch brief placeholder
├── README.md                                  # 真人 Lead 责任边界 (待 DDD Review 拍板)
├── player.md                                   # placeholder
├── economy.md                                  # placeholder
├── match.md                                    # placeholder
├── social.md                                   # placeholder
└── admin.md                                    # placeholder
```

### 3.2 5 域 Lead 内推话术 (per docs/recruitment v0.1 §3)

**5 域 × 1 份话术** (每域独立):

| 域 | 关键职责 | token-OLU 估 | timeline |
|---|---|---|---|
| **player** | 玩家域 DDD bounded context + DDD Review Lead | ~3 SRE·周 (~3.6M tokens) | T0-T2 立即启动 |
| **economy** | 经济域 token 计量 + 守门 #4 token-OLU Lead | ~3 SRE·周 | T0-T2 |
| **match** | 匹配域 saga orchestrator + ECS Lead | ~2 SRE·周 | T1-T3 |
| **social** | 社交域 event bus + EventBus Lead | ~2 SRE·周 | T1-T3 |
| **admin** | 管理域 5 域 RACI + 评审主持 Lead | ~2 SRE·周 | T2-T4 |

**总估**: 11-15 SRE·周, 1.5-2 月满员 (T0-T5, per 9/5 10:43 JST 拍板 D + 内推 brief v0.1).

### 3.3 内推启动动作 (本次 wt 内可实装)

1. **5 域 × 1 份话术** 复制到 `docs/recruitment/outreach_log/{domain}/` (committed placeholder)
2. **`scripts/automation/lead_outreach.py v0.2`** 升级: 邮件草稿生成 + contact log 写入 (per 守门 #19 v19 Python 化)
3. **`docs/briefs/5-leads/{domain}.md`** placeholder 落档 (待 DDD Review 拍板)
4. **`docs/recruitment/status/{domain}.md`** 寻访状态表 (每域 1 行 status)

## 4. 验收 (Acceptance Criteria)

### 4.1 守门合规

| # | 守门 | 验证 |
|---|---|---|
| 1 | 5 域 × 1 份话术落档 (committed placeholder) | commit 后 |
| 2 | scripts/automation/lead_outreach.py v0.2 升级 (per 守门 #19 v19) | 实装后 |
| 3 | docs/briefs/5-leads/ 5 placeholder 落档 (待 DDD Review) | commit 后 |
| 4 | 5 域 Lead 联系方式 (email / phone) 不打印到终端 (守门 #5 env 安全) | 实装后审计 |
| 5 | Mavis 临时代签 (5 域 Lead 决策), 真人到位后追溯签字 (per 守门 #14 v3) | commit author=Ulysses |

### 4.2 跨 session 续做 (per 守门 #14 v3 + 9/5 10:43 JST 拍板 D)

- **本次 session (mvs_0649e004aebc41259a4ec93a093e9e3)**: worktree + brief (本 brief) + lead_outreach.py v0.2 升级 + 5 placeholder 落档 (轻量 ~0.05M tokens)
- **下周 session T1**: Ulysses 跑 5 域 × 1 份内推 (1-1 沟通)
- **T2-T5 (2-6 周)**: 5 域 Lead 真人陆续到位 (per 内推 brief timeline)
- **真人到位后**: 5 域 Lead Subagent dispatch brief (`docs/briefs/5-leads/`) 跟真人 Lead 责任边界 → DDD Review 拍板 + 修订历史 +1 行追溯签字 (per 守门 #14 v2 缺口 #2 + 守门 #14 v3 Mavis 永久代签)

### 4.3 commit message 引用 brief 路径 (per 守门 #21 v21)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(recruitment): 5 域 Lead 内推启动 + lead_outreach.py v0.2 升级

  per docs/briefs/wt-5lead-outreach.md (5 域 Lead 寻访 brief) + docs/recruitment/5-business-domain-lead-referral.md v0.1 (内推 brief 范式) + ask_409cbd32edc309d71a083e2a 拍板 (Q1=Ulysses 内推+ Q2=立即启动).
  ..."
```

## 5. 守门硬约束

- 守门 #5 env 安全 (8/27 11:06 JST hard ban): 5 域 Lead 联系方式 (email / phone) 不打印到终端/log
- 守门 #9 v3: 调试控制台走 subprocess 替代 RPC
- 守门 #9 v19: agent 交互 Python 化 (走 scripts/automation/lead_outreach.py)
- 守门 #12: 缺标比错标, 4 缺口 (G-DEP-03/08/09/11) 显式列
- 守门 #14 v2: 5 域 Lead 内推 brief 模板 + timeline 拍板落地
- 守门 #14 v3: Mavis 永久代签全部签字栏
- 守门 #19 v19: agent 交互 Python 化
- 守门 #20 v20: 子代理 dispatch 必先 brief
- 守门 #21 v21: [P] docs 同步必更新 §4 + registry
- 守门 #27 v27 候选: 子代理 RPC 失败 fallback

## 6. 引用 (References)

- [docs/recruitment/5-business-domain-lead-referral.md v0.1](../../docs/recruitment/5-business-domain-lead-referral.md) — 内推 brief 范式
- [automation-design §4.18 ARG.4](../../docs/automation-design.md) — 5 域 RACI 表关联
- [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.3.1 §3.3 line 174](../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) — G-DEP-03 已拍板 ✅
- [scripts/automation/lead_outreach.py v0.1](../../scripts/automation/lead_outreach.py) — 6.8KB 现状
- [AGENTS.md §4 #14 5 域 Lead 独立真实身份 + 真人到位追溯](../../AGENTS.md)
- [AGENTS.md §4.1 v25 守门 #14 v25 5 域 Lead 内推 + timeline 拍板](../../AGENTS.md)

---

**per 守门 #14 v3 Mavis 永久代签**: author = Ulysses <ulysses@mavis.local>, 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008).