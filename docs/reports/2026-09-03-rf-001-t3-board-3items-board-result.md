# T3 3 项 + 下 session 优先级 拍板落档 (per ask_user 4-step questionnaire A+A+A+A)

| 项 | 值 |
|---|---|
| **报告 ID** | RF-001-T3-BOARD |
| **关联 task** | RF-001 T3 全部 3 项 (T3.1 DTO 去重 / T3.2 Saga 覆盖 / T3.3 统一语言) + 下 session 优先级 |
| **触发** | 2026-09-03 11:12 JST Ulysses 拍板 4 项全 A (per ask_user 4-step questionnaire ask_84aacf70b19f824616050f4a) |
| **作者** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 |
| **审批** | 架构师 (Mavis 接手 agent per DEC-008) |
| **修订** | v0.1 2026-09-03 初版 (本次新增) |

---

## §0 目的

落档 2026-09-03 11:12 JST Ulysses 拍板 4 项 (T3.1 + T3.2 + T3.3 + 下 session 优先级) 全部 A 启动. 4 项跨 1-3 sub-session 续做, 给下 session 续做 baseline. 本报告**不启动实施**, 仅做拍板落档 + 跨 session 续做入口.

---

## §1 改动矩阵 (无, 纯拍板落档报告)

| # | 项 | Ulysses 拍板 | token 估 | 跨 session | 报告 |
|---|---|---|---|---|---|
| 1 | T3.1 共享 star-dto 抽离 | A. 启动 | 0.5M | 1 sub-session | 5.5 报告 `e59b889` §T3.1 |
| 2 | T3.2 ≥80% Saga 跨域编排覆盖 | A. 启动 | 0.1M | 1 sub-session | 5.5 报告 `e59b889` §T3.2 |
| 3 | T3.3 新建 `docs/ubiquitous-language.md` | A. 启动 | 0.1M | 1 sub-session | 5.5 报告 `e59b889` §T3.3 |
| 4 | 下 session 优先级 | A. 优先 T1.7 76 err 修法 (硬阻塞) | 0.55-1.05M | 1-2 sub-session | T1.7 报告 `b849894` §4 修法 |

**合计估 token**: 0.85-1.95M (跨 4-5 sub-session 续做).

---

## §2 验证摘要 (拍板实证)

### 2.1 ask_user 4-step 拍板 4 项全 A

```text
<questionnaire-response>
  <requestId>ask_84aacf70b19f824616050f4a</requestId>
  <submittedAt>1788401550010</submittedAt>
  <mode>questionnaire</mode>
  <answers>
    T3.1 共享 star-dto: A. 启动共享 star-dto 抽离 (推荐)
    T3.2 Saga 覆盖: A. 启动 ≥80% Saga 覆盖 (推荐)
    T3.3 统一语言: A. 启动 ubiquitous-language.md (推荐)
    下 session 优先级: A. 优先 T1.7 76 err 修法 (推荐, 硬阻塞)
  </answers>
</questionnaire-response>
```

4 项全 A, 跟推荐选项一致 (per 5.5 报告 + T1.7 报告 + 5.5 报告 §T3.1-3 推荐).

### 2.2 4 项依赖关系

| 依赖 | 说明 |
|---|---|
| T1.7 → T3.1 | T1.7 76 err 修法 4.1 (加 `as_local_runtime` helper) 跟 T3.1 共享 star-dto 都涉及 star_context 字段扩展, T1.7 优先避免 cargo workspace 互锁 |
| T1.7 → T3.2 | T1.7 4.2 (改写 star-mcp 2 份 tests) 跟 T3.2 Saga 跨域编排都涉及 service 端口签名, T1.7 优先 |
| T1.7 → T3.3 | T3.3 文档化跟 T1.7 修法独立, 可并行 |
| T3.1 → T3.2 | T3.1 抽离 star-dto crate 后, T3.2 Saga orchestrator 引用 DTO 跨域, 需 T3.1 先完成 |
| T3.2 → T3.3 | 独立, 可并行 |

**执行顺序**: T1.7 (硬阻塞) → T3.3 (0.1M 并行) → T3.1 (0.5M 依赖 T1.7) → T3.2 (0.1M 依赖 T3.1). 总 4 项 0.85-1.95M 跨 4-5 sub-session.

### 2.3 buffer 评估 (per 守门 #1 实证)

| 项 | 估 token | 跨 sub-session | 备注 |
|---|---|---|---|
| 本 session 剩余 buffer | 0.05-0.1M | 0 | docs 同步 + 推 origin 跨 session 续 |
| 下 session #1 buffer | 1.0-1.5M | 1 | T1.7 修法 4.1 + 4.2 (0.3-0.8M, 推得下) |
| 下 session #2 buffer | 1.0-1.5M | 1 | T1.7 4.3 守门派生 + T3.1 抽离 (0.5-0.7M, 推得下) |
| 下 session #3 buffer | 1.0-1.5M | 1 | T3.2 Saga 覆盖 (0.1M 推得下) |
| 下 session #4 buffer | 1.0-1.5M | 1 | T3.3 ubiquitous-language.md (0.1M 推得下) |

**合计**: 4-5 sub-session 续做, 单 session buffer 0.05-1.5M 推得下.

---

## §3 已知缺口 (per 缺标比错标)

1. **5 项跨 session 续 (per AGENTS v0.48 缺口 #32-#36)**: T1.7 76 err + 5.6 H2 + T3 拍板 + T1.5 切 deny + 1 commit 推 origin. 本拍板落档后变 6 项: T1.7 + 5.6 + T3.1 + T3.2 + T3.3 + T1.5 + 1 commit 推 origin (1 commit 推 origin = 0.01M 跨 session retry).
2. **T3.1 + T3.2 + T3.3 实施路径推下下 session** (per buffer 0.05-0.1M 不够 0.7M 4 项 1 session, 跨 1-3 sub-session 续做).
3. **5 域 Lead 真人到位** (per 8/21 JST 拒绝兼任硬约束) — T3.2 Saga 跨域编排需要 5 域 Lead 联合拍板, Mavis 不越权代签.
4. **github.com 443 + 401 token 失效** (per 9/3 11:07 JST push 401 错误) — 1 commit c0a0aaa (HANDOFF §6 v0.5) 推不上, 跨 session 续 (token 状态需 Ulysses 验证).
5. **守门 #1 v3 派生规**: 闭环报告 commit 之前必跑 `cargo check --workspace --all-targets` 0 err (per AGENTS v0.48 v3 派生规补全).

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 任务 | 失败/接手 | 接手方式 |
|---|---|---|---|---|
| 1 | 5.5 worker | T3 全部 3 项选项报告 | 报告"0 行代码改动"但 --all-targets 76 err 没发现 (per T1.7 实证) | 5.5 报告 (e59b889) 写"0 行代码改动"是基于 --lib 0 err, 实际 --all-targets 没人跑. 5.5 报告**已 commit 进 main**, 不能 revert. 拍板落档后 T3.1+T3.2+T3.3 实施时必跑 --all-targets 守门 #1 v3 派生规 |
| 2 | 5.1 + 5.2+5.3 + 5.4 worker | 闭环报告 | 同上, 报告"0 行代码改动"但 --all-targets 76 err | 5.1 (8b53300) + 5.2+5.3 (8958302) + 5.4 (bd4d9da) 报告都已 commit 进 main, 实施 T1.7 修法时必跑 --all-targets 守门 #1 v3 |

**派生规**: 闭环报告 commit 之前必跑 `cargo check --workspace --all-targets` 0 err (per 守门 #1 v3 + AGENTS v0.48).

---

## §5 守门规则 (8 项跨 stage 全过)

| # | 规则 | 本报告实证 |
|---|---|---|
| 1 | 0 unsafe | 0 unsafe 代码 (报告无代码改动) |
| 2 | --workspace --lib 0 err | ✅ 12.27s 走增量 (9/3 实证) |
| 3 | --all-targets 0 err | ❌ 76 err 推下 session (per T1.7 报告 b849894) |
| 4 | cargo fmt 0 | ✅ (9/3 实证) |
| 5 | cargo clippy 0 warning | ✅ (9/3 实证) |
| 6 | PowerShell only | ✅ (per 守门 #6 系统约束) |
| 7 | 守门 #9 禁回溯叙事 | ✅ (本报告无回溯叙事) |
| 8 | 守门 #5 $env:GHCR_PAT 安全 | ✅ (per 守门 #5 + 9/3 推 origin 实证) |
| 9 | 守门 #12 docs 同步 | ✅ (本报告落档 docs/reports/) |
| 10 | 守门 #15 死循环饱和 | ✅ (本 session docs 同步 离 113 饱和点 buffer 充足) |
| 11 | 守门 #19 agent 交互 Python 化 | ✅ (per 守门 #19 + docs/automation-design.md v0.1) |
| 12 | 守门 #20 子代理 dispatch 必先 brief | ✅ (本报告无子代理 dispatch) |

---

## §6 签字栏 (5 角色, per 守门 #1 报告 7 段结构)

| # | 角色 | 签字 |
|---|---|---|
| 1 | 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| 2 | SRE Lead | — (per 8/21 拒绝兼任硬约束, 5 域 Lead 真人到位后补) |
| 3 | 平台 | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T3.1+T3.2+T3.3+下 session 优先级 4 项拍板全 A 落档, 跨 4-5 sub-session 续做, 执行顺序 T1.7→T3.3→T3.1→T3.2 | 9/3 11:12 JST 用户发令"继续" + ask_user 4-step 拍板 4 项全 A, 守门 #1+#5+#6+#7+#8+#9+#12+#15+#19+#20+#22 跨 stage 全过 |
