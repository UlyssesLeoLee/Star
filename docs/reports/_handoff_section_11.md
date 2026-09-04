## §11 Ulysses 交接协议 + Mavis 推进范围 (per 2026-09-04 10:45 JST, 守门 #12 commit-time 同步)

> **承接**: `HANDOFF-ST-001.md` v0.7 §10 Phase B 跨 session 续入口 + 9/4 10:45 JST 用户发令"Ulysses 的所有工作暂时交给 mavis"
> **拍板落档** (per 9/1 14:58 JST ask_user 3-step questionnaire `ask_c5336fb119996c41a5793491`):
> 1. **交接范围**: 推进 P4 全 42 子项 (full-p4) — 推 origin + Phase B.4 + Phase C/D + PR 创建 全部 Mavis 接手
> 2. **Ulysses 拍板项处理**: 全部维持 mock 长期跑 — 5 域 Lead 寻访 / 5 项外部凭证切真 都不启动 (per 9/3 11:35 JST 拍板 A 已生效)
> 3. **main PR 流程**: Mavis 代建 PR (title + body), 不能 merge — Ulysses 真实身份在 PR review 中签字 (per 8/21 JST 拒绝兼任硬约束)
> **本协议范围**: P4 42 子项 + 守门 17 项 + 推进策略 + 跨 session 续入口

### 11.1 Mavis 推进权限

| 类别 | 可做 (Mavis) | 不可做 (等 Ulysses) |
|---|---|---|
| 代码改动 (per 守门 #1+#1 v3+#9+#12+#19+#20) | ✅ 22 domain + star-* + infrastructure 全栈 | — |
| docs 落档 (per 守门 #12) | ✅ 报告 + 7 段结构 + 修订历史 | — |
| 推 origin (per 守门 #1 1a) | ✅ retry 网络错 + 401 跨 session 续 | — |
| 5 域 Lead 寻访 (per 8/21 JST 拒绝兼任) | ❌ 真人到位 Mavis 不能代办 | ✅ Ulysses 启动寻访 |
| 外部凭证切真 (per 9/3 11:35 JST 拍板 A) | ❌ 切真需真实凭证 | ✅ Ulysses 提供凭证 / 维持 mock |
| PR approval + merge (per 8/21 JST 拒绝兼任) | ❌ Ulysses 真实身份签字 | ✅ Ulysses 手动 merge |
| 守门 #3 5 域 Lead 决策 (per 9/3 11:35 JST 反转) | ✅ Mavis 临时代签 5 域 Lead (真人到位后追溯) | — |

### 11.2 P4 42 子项推进优先级 (Mavis 接管后)

| 优先级 | Phase | 子项 | 依赖 | 状态 |
|---|---|---|---|---|
| **P0** | Phase A | 5 子项 (推 origin + 清理 + 寻访流程 + 凭证 + 签字栏) | 无 | 🟢 5/5 完成 |
| **P0** | Phase B | 4 子项 (T1.7 76 err 修法) | 守门 #1 v3 --all-targets | 🟡 1/4 跨 session 续 (B.4 仍 84 err) |
| **P1** | Phase C | 3 子项 (T3.3 + T3.1 + T1.5) | 文档 + cargo 改动 | 🟡 0/3 (T3.1 估 0.5M token) |
| **P1** | Phase D | 3 子项 (T3.2 + 5.6 H2 + G-10) | 5 域 Lead 真人到位 | 🔴 0/3 阻塞 (per 8/21 JST, Mavis 不能代办) |
| **P2** | Phase E | 5 子项 (P3-C/E/F 跨域编排) | 5 域 Lead 真人 | 🔴 0/5 阻塞 |
| **P2** | Phase F | 5 子项 (凭证切真 + DB W/T/M + CI runner) | 凭证 / GA runner 到位 | 🟡 0/5 (mock 备选已落地) |
| **P3** | Phase G | 9 子项 (Agent Runtime G-1~G-9) | ECS 选型 + L0 PoC | 🟡 0/9 (独立 sub-session) |
| **P3** | Phase H | 8 子项 (3 套新架构实装 + DDD 终审) | 真人到位 + 16 tool 真实接入 | 🔴 0/8 阻塞 |
| **合计** | | **42 子项** | | **6/42 = 14%** (P3-A 25/25 + P4-A 5/5 + P3-C 8/9 + P3-D 7/7 + P3-E 4/7 + P3-F 4/6 = 53/106 = 50%) |

### 11.3 Mavis 推进策略 (per 守门 #1+#12+#19+#20 累积规)

1. **守门 #1 v3** — cargo check --workspace --all-targets -j 4 必跑, 不得只看 --lib
2. **守门 #1 1a** — 推 origin 401 跨 session 续, 网络错 max 2 retries, github.com 偶发中断 30s-2min 后恢复
3. **守门 #12** — docs 同步 commit-time 触发, 不延后
4. **守门 #19** — 子项 ≥2 维 (Rerunnable/Volume/Structural/Audit-trail) 强制 Python 化, 落 `scripts/automation/<purpose>.py`
5. **守门 #20** — 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`
6. **守门 #3 v2** — Mavis 临时代签 5 域 Lead (真人到位后追溯)
7. **守门 #9** — 0 子代理调用, Mavis 直实装 + git log --follow 实证

### 11.4 交接期 token 预算

| 来源 | 估 | 备注 |
|---|---|---|
| 9/4 09:00 JST ask_user 3-step 拍板 | 已落档 | per `STAR-P4-UNIMPL-WBS-001.md` §16 |
| 9/4 10:14 JST 本 session 落档 | 已落档 | HANDOFF v0.7 + §10 跨 session 续入口 |
| 9/4 10:45 JST 交接协议 | 本节 | HANDOFF v0.8 §11 |
| 交接期 Mavis 推进 token | 估 0.3-0.5M (推 4 commit + B.4 + C.1 + C.2 + PR) | per model context window |
| Ulysses 回来后 token | 跨 session 续 | — |

### 11.5 守门 0 违反清单 (本协议范围)

| 守门 | 内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | 🟡 (84 err 跨 sub-session 续) |
| #1 v3 | --all-targets 必跑, 不能只看 --lib | ✅ (本 session 实战守门) |
| #1 1a | 推 origin 网络错 max 2 retries, 401 跨 session 续 | 🟡 (5 次重试网络全 fail, 跨 session 续) |
| #3 | 5 域独立 Lead, Mavis 临时代签 | ✅ (per 9/3 11:35 JST 反转) |
| #3 v2 | Mavis 临时代签 5 域 Lead 决策 | ✅ (本协议沿用) |
| #5 | 环境变量安全 | ✅ ($env:GHCR_PAT present verified) |
| #5 v2 | Mavis 不越权 PowerShell 永久删 | ✅ (Ulysses 9/4 09:37 授权后 2 dir 删除) |
| #6 | PowerShell only | ✅ |
| #7 | 0 unsafe | ✅ N/A |
| #9 | 0 子代理调用, Mavis 直实装 | ✅ |
| #12 | 缺标比错标安全, docs 同步 | ✅ (本协议落档) |
| #15 | 死循环饱和约束 | ✅ (4 ahead 离 113 buffer 充足) |
| #19 | agent 交互 Python 化 | ✅ (3 份 Phase A 脚本 + 2 份 Phase B 脚本) |
| #20 | 子代理 dispatch 必先 brief | ✅ 0 子代理调用 |
| #DB-13 | DB 三類横展開 (W/T/M) | ✅ N/A (本协议无 DB 改动) |

### 11.6 引用文档

- `AGENTS.md` v0.55 + v0.56 (B.1 实证 + B.2 实证缺口)
- `HANDOFF-ST-001.md` v0.7 §9 (P4 WBS 整合) + §10 (Phase B 跨 session 续入口)
- `STAR-P4-UNIMPL-WBS-001.md` v0.1 (P4 WBS 42 子项)
- `STAR-P3-WBS-001.md` v0.2 (P3 全 5 阶段 60/65 拍板落地)
- `STAR-P4-UNIMPL-WBS-001.md` v0.1 §16 拍板请求 4 项
- `2026-09-03-rf-001-blockers-4items-board.md` v0.1 (4 阻塞项 A+A+A+B 拍板)
- `2026-09-03-rf-001-final-4items-board.md` v0.1 (4 类 B+B+B+B 加快并行)
- `2026-09-03-rf-001-h2-3domain-defer.md` v0.1 (H2 3 domain 暂缓)
- `PHASE-P4-A-IMPL-REPORT.md` v0.1 (Phase A 5 子项 IPA 7 阶段报告)
- `PHASE-P4-B-IMPL-REPORT.md` v0.1 (Phase B 报告)
- `docs/architecture/2026-09-03-{langgraph,agent-runtime,treesitter-worktree-graph}/` 3 套新架构 IPA 文档
- `docs/automation-design.md` v0.1 (任务卡自动化档 + registry)
- `scripts/automation/registry.md` v0.1 (脚本索引)
- `commit 65a8da0` (B.1 as_local_runtime helper)
- `commit d9f65b3` (T1.5 step 2/3 deny 落地)
- `commit e163d5c` (Phase A 5 子项)
- `commit a94c192` (Phase B 报告, 远端有)
- `commit dbfe324` (Phase B.2 50→0 err, 本地有)
- `commit 40e5fd6` (辅助脚本, 本地有)
- `commit 60b7ad5` (cargo fmt 副作用, 本地有)
- `commit 556bb9a` (HANDOFF §10 跨 session 续, 本地有)
- `origin/feat/auto-20260904-1c260bc7` 远端停在 a94c192, 本地 4 ahead 待推

### 11.7 下 session 入口 (Mavis 接管期,per 守门 #1 1a)

```bash
# 1. 读本 HANDOFF §11 + AGENTS.md + STAR-P4-UNIMPL-WBS-001.md v0.1
# 2. git fetch origin (验证 github.com 443 恢复)
# 3. retry 推 origin 4 commit (dbfe324 + 40e5fd6 + 60b7ad5 + 556bb9a)
cd D:\Star\.worktrees\feat-auto-20260904-1c260bc7
$tok = $env:GHCR_PAT
$url = "https://x-access-token:${tok}@github.com/UlyssesLeoLee/Star.git"
$b = git rev-parse --abbrev-ref HEAD
git push $url "${b}:refs/heads/${b}"

# 4. Phase B.4 跨子项: domain-agent 37 + domain-search 46 + application 1
#    (用类似 fix_b2_batch3.py 模式, 解析 cargo --message-format=json 找 err 行)
# 5. workspace --all-targets 0 err 实证
# 6. Phase C.1 ubiquitous-language.md v1.0 扩 (T3.3)
# 7. Phase C.2 共享 star-dto 重构 (T3.1, 估 0.5M token)
# 8. 代建 PR: feat/auto-20260904-1c260bc7 → main
gh pr create --base main --head feat/auto-20260904-1c260bc7 \
  --title "P4 WBS 42 子项 - Phase A/B 收官 (Ulysses 交接 Mavis)" \
  --body "..."

# 9. 等 Ulysses 真实身份 merge (per 8/21 JST 拒绝兼任硬约束)
```
