# 5 Domain Leads - Subagent Dispatch Protocol

> **Status**: 🟡 Draft v0.1 (2026-09-04 18:30 JST 拍板, 守门 #3 反转 + 守门 #14 修订)
> **承接**: 2026-09-04 18:28 JST 用户拍板"5 域 Lead 真人寻访 → 全部由子代理兼任"
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签

---

## §0 拍板背景

### 9/4 18:28 JST 用户拍板
> "5 域 Lead 真人寻访 → 全部由子代理兼任"

### 守门 #3 反转 (per 8/21 JST 拍板 → 9/4 18:30 JST 反转)
- 之前 (8/21 JST): "5 域独立 Lead 拒绝兼任" (拒绝架构师/SRE 兼任 5 域 Lead, 5 真人)
- 现在 (9/4 18:30 JST): 5 域 Lead **可由子代理兼任** (5 子代理, 各 1 域)
- 真人到位后追溯签字 (per 守门 #1 禁回溯叙事 + 守门 #10)

### 守门 #14 修订 (per 9/3 19:43 JST 5 域 Lead CONTENT 4 维)
- 决策 scope: 5 域各子代理独立决策自己域
- RACI: R+A+C+I 完整责任
- 到位 timeline: 真人寻访待启动, 子代理代签期间覆盖
- Mavis 代签边界: 跨域协调仍 Mavis (守门 #9 v3 实证 5/5 subagent RPC 不可靠, Mavis 实际执行)

---

## §1 5 域 Lead 角色

per AGENTS.md §5 仓库拓扑 disclaimer:
- "5 域"是**历史治理命名**, **不**是 DDD bounded context, **不** = Star 仓 22 domain-* crate
- 22 domain-* crate 跟 5 域**不建立业务子域↔DDD 映射** (per 守门 §5 disclaimer)

| # | 5 域 (历史命名) | 决策 scope | 子代理 ID |
|---|---|---|---|
| 1 | **player** (玩家域) | 玩家生命周期 / 账户 / 角色 / 存档 / 在线状态 | `player-lead` |
| 2 | **economy** (经济域) | 货币 / 交易 / 商店 / 付费 / 跨域 Saga 协调 (per Q-003) | `economy-lead` |
| 3 | **match** (对战域) | 匹配 / 房间 / 对战逻辑 / 战报 / 跨域协调 | `match-lead` |
| 4 | **social** (社交域) | 好友 / 公会 / 聊天 / 排行榜 / 通知 | `social-lead` |
| 5 | **admin** (管理域) | COC 控制面 / 审计 / 合规 / 监控 / RBAC | `admin-lead` |

---

## §2 5 子代理 brief 模板

per 守门 #9 v3 (子代理 dispatch 必先落地 brief, per 9/2 00:39 JST 拍板)

每子代理 dispatch 前, 必须先创 brief 落档 `docs/briefs/5-leads/{domain}.md`:
```yaml
# {Domain} Lead Subagent Brief

## 任务
{本子代理需要做的具体决策/任务}

## 上下文
- 5 域 (player / economy / match / social / admin) 是历史治理命名 (per AGENTS.md §5)
- 决策 scope: {本域内所有技术决策}
- RACI: R+A+C+I 完整 (per 守门 #14 5 域 Lead CONTENT 4 维)

## 不变量
- 守门 #1 禁回溯叙事
- 守门 #5 env 安全
- 守门 #10 代签 author=Ulysses
- 守门 #12 commit-time docs 同步

## 已知缺口
- 守门 #9 v3 实证 5/5 subagent RPC 不可靠 (per AGENTS.md §4 #9 实证)
  → 子代理 dispatch 失败时, Mavis 实际执行决策 + 标注"代签" 触发条件

## 决策权
- 域内决策: 子代理可自主
- 跨域决策: 必须 Mavis 协调 (避免 subagent 间冲突)
- 真人间隔后追溯: per 守门 #1 禁回溯叙事 + 守门 #10 author=Ulysses

## 输出
- commit author=Ulysses (per 守门 #10)
- docs commit (per 守门 #12)
- 1 PHASE-* 报告 (per WBS §3 报告 7 段结构)
```

---

## §3 dispatch orchestrator (per 守门 #19 [M])

`scripts/automation/dispatch_5_leads.py` (PoC, 守门 #19 派生):

```python
# 5 子代理并行 dispatch (实际不可靠, per 守门 #9 v3)
# PoC: 串行 + 失败 fallback Mavis
LEADS = ["player", "economy", "match", "social", "admin"]
for domain in LEADS:
    brief = load_brief(f"docs/briefs/5-leads/{domain}.md")
    result = dispatch_subagent(f"{domain}-lead", brief)
    if result.status != "succeeded":
        # 守门 #9 v3 fallback: Mavis 实际执行
        mavis_decide(domain, brief, reason="subagent RPC failed")
```

---

## §4 当前状态 (per 9/4 18:30 JST 拍板)

| 域 | Lead 角色 | 真人到位 | 子代理代签 | Mavis 跨域协调 |
|---|---|---|---|---|
| player | `player-lead` | ❌ 待寻访 | ✅ 已代签 | ✅ |
| economy | `economy-lead` | ❌ 待寻访 | ✅ 已代签 | ✅ |
| match | `match-lead` | ❌ 待寻访 | ✅ 已代签 | ✅ |
| social | `social-lead` | ❌ 待寻访 | ✅ 已代签 | ✅ |
| admin | `admin-lead` | ❌ 待寻访 | ✅ 已代签 | ✅ |

---

## §5 守门规则应用

| # | 守门 | 9/4 18:30 JST 修订 |
|---|---|---|
| 1 | 禁回溯叙事 | ✅ 历史 commit 不 revert, 真人到位用新 commit 覆盖 |
| 3 | 5 域独立 Lead 拒绝兼任 | **🔄 反转**: 5 域 Lead 可由子代理兼任 (5 子代理, 各 1 域) |
| 5 | env 安全 | ✅ 继续遵守 |
| 9 | 子代理 dispatch 必先 brief | ✅ 5 子代理 dispatch 前必先创 brief (per §2) |
| 10 | 代签 author=Ulysses | ✅ 5 子代理代签 commit author=Ulysses |
| 12 | commit-time docs 同步 | ✅ 本 docs 同步落档 |
| 14 | 5 域 Lead CONTENT 4 维 | **🔄 修订**: 决策 scope 5 子代理独立, RACI R+A+C+I, 到位 timeline 待寻访, Mavis 跨域协调 |

---

## §6 已知缺口

| # | 缺口 | 后续阶段 |
|---|---|---|
| 1 | 5 子代理 dispatch 实际不可靠 (per 守门 #9 v3 实证 5/5 RPC 失败) | 守门 #9 改进 |
| 2 | 5 域 Lead 真人寻访流程未启动 | V2.6 启动 |
| 3 | 跨域协调仍 Mavis 主导 (子代理不能互相 dispatch) | V2.6 改进 |
| 4 | 真人到位后追溯签字流程 (per 守门 #1+#10) | 待 5 域 Lead 真人到位 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 5 域 Lead 全部由子代理兼任 (守门 #3 反转 + 守门 #14 修订) | 9/4 18:28 JST 用户拍板"全部由子代理兼任" |
