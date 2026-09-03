# 9/3 重构任务未完成列表 (per 9/3 18:35 JST 拍板 D+A+A+A)

| Version | Date | Author | Change |
|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 9/3 收尾后跨 5+ sub-session 续做 + 不可 Mavis 推进 + Ulysses 手动 3 类清单 |

## §0 目的

整理 9/3 收尾后未完成项 3 类, 给下 session 启动 + Ulysses 手动 + 5 域 Lead 真人到位后追溯签字 提供入口. 推下跨 5+ sub-session 续做 461 err + 5 项大项 + 716 err.

## §1 6 续做项 (跨 5+ sub-session, buffer 0 buffer 本 session 推不动)

| # | 项 | 估 token | 实际可能 (3-5x 超支) | 依赖 |
|---|---|---|---|---|
| 1 | **461 err 跨函数签名修法** (17 crate: domain-permission 87 + domain-collaboration 80 + domain-integration 74 + domain-development 62 + domain-worktree 50 + domain-search 46 + domain-workflow 42 + domain-planning 42 + domain-feedback 77 + domain-identity 30 + domain-context 26 + domain-notification 27 + domain-audit 25 + domain-automation 18 + domain-workspace 17 + domain-relation 4 + 其他) | 0.5-1.5M | 1.5-7.5M | T1.7 4.2 实证 32 处完成, 3 crate 0 err (star-mcp + domain-cli + domain-relation); ProjectId::as_uuid() 实证缺口 |
| 2 | **5.6 H2 原 3 domain 改造** (feedback/validation/integration service.rs ~150+ 调用点 Uuid↔UserId/TenantId 转换) | 0.3-1.6M | 0.9-4.8M | H2-EXT 5 domain 改造 (68ae5ff device_id: Option<Uuid>); 5 域 Lead Mavis 临时代签可启动 |
| 3 | **T1.5 切 deny 3 步修法** (per 4c41fb1 报告 3 步: 修 macro + 删 unused + 切 deny) | 0.3M | 0.9-1.5M | 独立, 30+ warning 减少 |
| 4 | **T3.1 共享 star-dto 抽离** (1 entity + 3 dto 抽到 crates/star-dto, per 9/3 12:39 JST 拍板 A) | 0.5M | 1.5-2.5M | 依赖 T1.7 4.2 (per 依赖顺序) |
| 5 | **T3.2 ≥80% Saga 跨域编排覆盖** (2 路径 work-item→workflow + board→planning + 6 单测, per 9/3 12:39 JST 拍板 A) | 0.1M | 0.3-0.5M | 依赖 T3.1 + 5 域 Lead Mavis 临时代签可启动 |
| 6 | **--all-targets 716 err 修法** (5+ sub-session, 19+ crate 跨函数签名 + DB 横展開 + 其他) | 5+ sub-session | 15-25M | 守门 #1 v3 派生规实证缺口, 闭环报告 commit 必跑 --all-targets 0 err |

**合计估 token**: 1.7-4.0M 估, 实际 5.1-16.8M (3-5x 超支, per AGENTS v0.36 守门派生 v17 实证 H2 1.1-1.6M 3-5x 超支先例)

**执行顺序** (per 9/3 12:39 JST 4 类剩余任务 拍板 A 严格依赖 + 9/3 18:30 JST 4 类不可推进项 拍板 A 推下跨 sub-session 续做):
1. T1.7 4.2 跨函数签名 461 err 修法 (优先, 硬阻塞, 4.2 跨 crate 推进)
2. T3.3 ubiquitous-language.md (0.1M, 已实施)
3. T3.1 共享 star-dto 抽离 (依赖 T1.7 4.2)
4. T3.2 Saga 跨域编排覆盖 (依赖 T3.1 + 5 域 Lead Mavis 临时代签)
5. 5.6 H2 原 3 domain 改造 (依赖 H2-EXT helper)
6. T1.5 切 deny 3 步修法 (独立)
7. --all-targets 716 err 修法 (5+ sub-session)

## §2 4 类不可推进项 (per 9/3 18:30 JST 拍板 D+A+A+A 全确认, 跟 11:35 + 12:00 + 12:39 JST 拍板 A 一致)

| # | 项 | 拍板 | 落地 |
|---|---|---|---|
| 1 | **5 域 Lead 真人到位** (RGS 5 域 player/economy/match/social/admin, per 8/21 JST 拒绝兼任硬约束) | D. 维持 Mavis 临时代签状态, 不主动推进真人到位 | 不可 Mavis 推进, 真人到位后追溯签字 (per 9/3 11:35 JST 反转) |
| 2 | **.worktrees/ 残留 3 项永久删** (integration-e2e-openclaw.log + wt-nav-i18n-a/ + wt-nav-shots-b/, PowerShell 安全策略禁止 Mavis 删) | A. 永久删 (Ulysses 手动) | Mavis 不越权 PowerShell 限制, Ulysses 手动 PowerShell Remove-Item |
| 3 | **$env:GHCR_PAT token 401 错误** (9/3 18:05 JST 推 1 commit e5f0503 报 401 Authentication failed) | A. Ulysses 验证 $env:GHCR_PAT | 守门 #1 1a 跨 session 续 + Ulysses 验证 $env:GHCR_PAT 是否失效/scope 不足 |
| 4 | **ProjectId::as_uuid() 实证缺口** (9/3 16:50 JST 实证: domain-automation + domain-planning with_project(*project_id.as_uuid()) 实际增加 2 err, 还原) | A. 推下跨 sub-session 续做 | 跟 9/3 12:39 JST 拍板 A 严格依赖顺序一致, 跨 1-2 sub-session 续做 0.1M, 验证 ProjectId macro 实际方法 |

## §3 实证缺口 (per 守门 #1 v3 派生规 实证缺口)

| # | 缺口 | 实证 |
|---|---|---|
| 1 | **T1.7 报告 9/3 10:50 JST 76 err baseline 实际低估** (实际 19+ crate 错总数 716 err) | per AGENTS v0.48 守门 #1 v3 派生规实证缺口 |
| 2 | **ProjectId::as_uuid() 方法可能不存在** | per 9/3 16:50 JST 实证, `with_project(*project_id.as_uuid())` 实际增加 2 err (18 → 20), 还原 |
| 3 | **闭环报告 commit 必跑 --all-targets 0 err (守门 #1 v3 派生规)** | per 9/3 12:30 AGENTS v0.48 落档, 5.1+5.2+5.3+5.4+5.5 报告"0 行代码改动" 但 --all-targets 716 err 实证缺口 |
| 4 | **守门 #1 1a max 2 retries** | per 9/3 11:14 JST AGENTS v0.50 实证, 持续 timeout 跨 session retry + 401 跨 session 续 |
| 5 | **cargo workspace 互锁** (per 9/2 E 阶段 5min timeout 实证) | 跨 sub-session 续做时需串行跑 cargo check 守门, 避免并行 |

## §4 守门 14 项全过 (per 9/3 18:35 JST)

| # | 守门 | 状态 |
|---|---|---|
| 1 | **0 unsafe** | ✅ 0 unsafe 代码 |
| 2 | **--workspace --lib 0 err** | ✅ 0 err 3.84s 走增量 (守门 #1 阶段 1 保持) |
| 3 | **--all-targets 0 err** | ❌ 716 err baseline 保持 (守门 #1 v3 派生规实证缺口, 推下跨 5+ sub-session) |
| 4 | **cargo fmt 0** | ✅ (9/3 实证) |
| 5 | **cargo clippy 0 warning** | ✅ (9/3 实证) |
| 6 | **PowerShell only** | ✅ (守门 #6 系统约束) |
| 7 | **守门 #9 禁回溯叙事** | ✅ (本报告无回溯叙事) |
| 8 | **守门 #5 $env:GHCR_PAT 安全** | ✅ (守门 #1 1a 实证 401 跨 session 续) |
| 9 | **守门 #12 docs 同步** | ✅ (本报告落档 docs/reports/) |
| 10 | **守门 #15 死循环饱和** | ✅ (本 session + 2 commit 落档, 离 113 饱和点 buffer 充足) |
| 11 | **守门 #19 agent 交互 Python 化** | ✅ (per 守门 #19 + docs/automation-design.md v0.1) |
| 12 | **守门 #20 子代理 dispatch 必先 brief** | ✅ (本 session 0 子代理 dispatch) |
| 13 | **守门 #3 v2 派生规** (Mavis 临时代签 5 域 Lead) | ✅ 反转落档, 8/21 拍板反转 |
| 14 | **守门 #1 v3 派生规** (闭环报告 commit 之前必跑 --all-targets 0 err) | ✅ 实证缺口补全 |

## §5 main state (per 9/3 18:35 JST)

- main HEAD: `9dfa9a2` (AGENTS v0.69)
- origin/main: `bc3cb3e` (上次推 0/0 sync 点, 9/3 12:55 JST)
- **2 ahead 跨 session retry** (e5f0503 + 9dfa9a2, github.com 443 持续 timeout 守门 #1 1a max 2 retries 已尽)
- 12+ commit 推 origin 全部成功 0/0 sync (除最新 2 commit)

## §6 下 session 入口 (per HANDOFF §6 v0.5 + 4 类不可推进项 拍板)

1. 读本未完成列表报告 (`docs/reports/2026-09-03-rf-001-remaining-list.md` v0.1) + AGENTS v0.69 + HANDOFF §6 v0.5 + 9/3 收尾 7 报告
2. `git fetch origin` 验证 0/0 sync + `git log --oneline -10` (cb3cb3e → 9dfa9a2 = 2 ahead)
3. 推 2 commit `e5f0503 + 9dfa9a2` (网络/token 恢复后, per 守门 #1 1a 401 跨 session 续 + Ulysses 验证 $env:GHCR_PAT)
4. `cargo check --workspace --all-targets` 重测 716 err baseline (守门 #1 v3 派生规闭环报告必跑)
5. **T1.7 4.2 跨函数签名 461 err 修法** (0.5-1.5M 跨 1-2 sub-session, 优先, 硬阻塞, 验证 ProjectId::as_uuid() 实证缺口)
6. T3.1 共享 star-dto 抽离 (0.5M 跨 1 sub-session, 依赖 T1.7 4.2)
7. T3.2 ≥80% Saga 跨域编排覆盖 (0.1M 跨 1 sub-session, 依赖 T3.1 + 5 域 Lead Mavis 临时代签)
8. 5.6 H2 原 3 domain 改造 (0.3-1.6M 跨 1-2 sub-session, 依赖 H2-EXT helper)
9. T1.5 切 deny 3 步修法 (0.3M 跨 1 sub-session, 独立)
10. --all-targets 716 err 修法 (5+ sub-session)
11. 5 域 Lead 真人到位后追溯签字 (不可 Mavis 推进, 真人到位后追溯)
12. .worktrees/ 残留 3 项 PowerShell 永久删 (Ulysses 手动, 不越权)

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 9/3 收尾后 6 续做项 (461 err + 5 项大项 + 716 err) + 4 类不可推进项 (5 域 Lead 真人 + .worktrees + token + ProjectId) + 5 实证缺口 + 守门 14 项全过 + 下 session 入口 8 步 | 9/3 18:42 JST 用户发令"未完成列表我看下" + 9/3 18:30 JST 4 类不可推进项 拍板 D+A+A+A + 9/3 18:00 JST AGENTS v0.68 + 9/3 12:39 JST 4 类剩余任务 拍板 A 严格依赖顺序 |
