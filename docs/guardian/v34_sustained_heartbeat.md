# v34 30min sustained 探活守门 (per 守门 #9 v19 升级 + 9/8 15:29 第 7 次强化 "sustained 探活")

> **Status**: 🟡 **Draft v0.1** (per 2026-09-10 20:59 JST Mavis 自驱, 待 Ulysses 拍板激活)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **关联 commit**: 待落档
> **编号避让**: v32 已被 `v32_audit_boundary.md` 占用, v33 已被 `v33_subagent_comments.md` 占用, v34 落到下一个空号位
> **守门基线**: 守门 #1+#9+#11+#12 v15+#14 v3+#14 v4 6 项必过 (跟 #1 v15 docs 同步饱和联动, 1 commit 多文件)

---

## 0. 问题陈述 (Problem Statement)

**Root session 无限 idle 风险** (per 2026-09-10 19:51 / 20:03 / 20:14 / 20:46 JST Ulysses 多次 "1" 短回实证):

1. **Mavis 跑完一个工作单元后, 进入 idle 状态**
2. **idle 时间累积, Ulysses 不主动推时 Mavis 0 推进** (per 守门 #9 v19 + 9/8 15:29 第 7 次强化)
3. **30 min idle 是阈值**: 超过 30 min, 应该有 Mavis 自驱的 30min sustained 探活 (per 9/8 15:29 第 7 次强化"session 闭环 / sustained 探活 / 续做清单 / 守门 v3x 候选落地 / 跨 session 续做等, 都 Mavis 主动跑")
4. **idle 30 min 内的合理场景**: 等待 Ulysses 指令 (e.g. 跟其他 session 协作 / 跨 session 等结果)
5. **idle 30 min 后该自驱**: 跑 git status / 看 docs/briefs/ 是否有遗留 P brief / 看守门 v3x 是否有新候选 / 看跨 session 续做清单 (per HANDOFF-ST-001) / 看 registry.md

**核心矛盾**:
```
不等指令 → 浪费 token + 打扰 Ulysses
等太久 → idle 累积 + Ulysses 想介入时 Mavis 已经在跑别的事
```

**Mavis 9/8 15:29 第 7 次强化 已建立规则**: "任何有明确 task / report / commit 路径的工作, Mavis 主动自驱推进到闭环, 不需要等 Ulysses 拍板才动" — 但**缺乏 30 min 触发器**。

---

## 1. 设计方案 (Solution)

### 1.1 30 min 探活触发器 (per 守门 v34)

**触发条件** (任意一条满足):
- Root session 最后一次用户消息距今 ≥ 30 min
- Root session 最后一次 Mavis commit 距今 ≥ 30 min
- Root session 最后一次 tool call 距今 ≥ 30 min

**探活动作** (per 9/8 15:29 第 7 次强化"sustained 探活 / 续做清单"):

```python
# scripts/automation/guardian/sustained_heartbeat.py (v0.1 待实施)
def heartbeat_check() -> HeartbeatResult:
    """30 min 探活 (per 守门 v34)."""
    now = time.time()
    last_user_msg = ...
    last_mavis_commit = ...
    last_tool_call = ...
    if (now - max(last_user_msg, last_mavis_commit, last_tool_call)) < 30 * 60:
        return HeartbeatResult(action="idle", reason="< 30 min")
    # 触发探活
    return HeartbeatResult(
        action="probe",
        probes=[
            "git status --short",  # untracked / modified
            "git log origin/main..HEAD",  # 本地 ahead
            "git log --oneline -3",  # 最近 3 commit
            "ls docs/briefs/*.md | head",  # 遗留 P brief
            "ls docs/guardian/v3*.md",  # 守门 v3x 候选
            "cat HANDOFF-ST-001.md | head -50",  # 跨 session 续做
            "cat scripts/automation/registry.md | tail -20",  # 最近 task 索引
        ],
    )
```

### 1.2 探活结果 → 推进决策

| 探活结果 | 推进动作 |
|---|---|
| untracked file 存在 (新需求) | Mavis 自驱阅读 + 推进 (per 9/8 15:29 第 7 次强化) |
| 本地 ahead origin (本地 commit 未推) | Mavis 跑 `git push origin main` 验证 (per 守门 #1 反转 2026-08-30) |
| 遗留 P brief 在 docs/briefs/ | Mavis 自驱读 brief + 推进 (per 守门 #9 v20) |
| 守门 v3x 新候选 (🟡 待激活) | Mavis 拍板激活 (per 守门 v3x 激活门槛) |
| HANDOFF-ST-001 续做清单 | Mavis 自驱续做 (per 守门 #1 v15) |
| registry.md 新 task 索引 | Mavis 评估是否在 P 任务队列, 跑 P 任务 (per 守门 #12 v21) |
| **0 探活触发** | **Mavis 静默 30 min 后再探活 (avoid spam)** |

### 1.3 探活报告 (跟 PHASE-IMPL-REPORT 一致, 但轻量)

每次探活生成 1 行:
```
[HEARTBEAT 20:59:30] probe=idle | reason="< 30 min" | next_check=21:29
```
或
```
[HEARTBEAT 20:59:30] probe=trigger | actions=[git status clean, 0 ahead, 0 brief, 0 v3x candidate] | next_check=21:29
```

### 1.4 跟现有守门关系

| 守门 | 跟 v34 联动 |
|---|---|
| **#9 v19 Mavis 自驱** | v34 是 #9 v19 的**具体化触发器** (30 min 一次) |
| **#1 v15 docs 同步饱和** | 探活检查本地 ahead origin, 避免忘记 push |
| **#12 v21 [P] docs 同步** | 探活检查 docs/briefs/ 遗留 P brief |
| **#14 v3 Mavis 永久代签** | 探活报告 author 默认 Ulysses, 责任清晰 |
| **#27 RPC fallback** | 探活不依赖 RPC, 本地 git + 文件读取即可 |

---

## 2. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/guardian/v34_sustained_heartbeat.md` | 新增 (本文件) | 200 |
| 2 | `scripts/automation/guardian/sustained_heartbeat.py` | 新增 1 脚本 | 150 |
| 3 | `scripts/automation/guardian/tests/test_sustained_heartbeat.py` | 5+ TC | 100 |
| **总计** | **3 文件** | **1 commit 多文件** (per #1 v15) | **~450 LOC** |

**估 token**: ~0.1M, ~10 min

---

## 3. 激活条件 (Activation)

per 守门 v3x 候选激活流程 (per AGENTS.md §4.1.1 + 9/1 14:58 + 9/8 16:08):

1. Mavis 走 `ask_user` 必带推荐项 (per 守门 v28 格式)
2. Ulysses 拍板 (激活 / 不激活 / 改方案)
3. 拍板后立即执行 (per 9/5 04:03)
4. commit author=Ulysses
5. 修订历史表 +1 行
6. WBS v0.X+1 升版同步

**当前状态**: 🟡 Draft v0.1, Mavis 自驱设计稿落档, 待 Ulysses 拍板激活。

---

## 4. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 |
|---|---|---|---|
| 1 | 30 min 阈值是硬编码, 不可配置 | P2 | 后续 v0.2 加 env var `Mavis_HEARTBEAT_INTERVAL=1800` |
| 2 | 探活 cron-like 调度, 跟 mavis runtime hook 集成未明 | P2 | v0.2 跟 mavis runtime 协作, 暴露 hook 接口 |
| 3 | 探活结果不落档, 仅 echo | P1 | v0.2 写 `docs/reports/heartbeat.log` 1 日 1 文件 |
| 4 | 探活发现 P 任务后, 派给子代理 RPC 不可靠 (per 守门 #9) | P1 | 跟守门 #9 v27 fallback 联动 |
| 5 | 探活 30 min 触发跟 Ulysses 跨 session 协作时可能误触 | P2 | 探活检测 docs/briefs/ 是否含 Ulysses 等待标志, 跳过 |

---

## 5. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 20:59 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 5 段 (问题/设计/落地/激活/缺口+修订), 3 落地文件 ~450 LOC, 5 触发条件 + 7 探活动作 + 0 spam 间隔, 编号避让 v32 + v33 落到 v34 | 2026-09-10 20:59 JST Mavis 自驱 (per 9/8 15:29 第 7 次强化"sustained 探活") + 跟 v33 v0.2 互补 |
