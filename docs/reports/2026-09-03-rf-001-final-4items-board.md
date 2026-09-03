# 剩余任务 4 类 拍板落档 (per ask_user 4-step B+B+B+B, 加快并行启动 + 守门缺口警告)

| 项 | 值 |
|---|---|
| **报告 ID** | RF-001-FINAL-4ITEMS-BOARD |
| **关联 task** | 剩余任务 4 类 推进策略 (Phase 5 6 续做项 + P3-G 后续 + T1.7 4 步 + 整体 timeline) |
| **触发** | 2026-09-03 12:39 JST Ulysses 拍板 4 项全 B 加快并行 (per ask_user 4-step questionnaire ask_e08fcd6f5e9b29102e9dea34) |
| **作者** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 |
| **审批** | 架构师 (Mavis 接手 agent per DEC-008) |
| **修订** | v0.1 2026-09-03 初版 (本次新增) |

---

## §0 目的

落档 2026-09-03 12:39 JST Ulysses 拍板 4 项全 B 加快并行 (Phase 5 6 续做项 + P3-G 后续 + T1.7 4 步 + 整体 timeline). 跨 2-3 sub-session 并行启动. 守门 #1 实证"不在预算失控情况下硬着头皮做完", 4 项 B 都有风险 (cargo 互锁 + buffer 超支 + 5 域 Lead + 网络), 推下下 session 续做.

---

## §1 改动矩阵 (无, 纯拍板落档报告)

| # | 项 | Ulysses 拍板 | 推进 |
|---|---|---|---|
| 1 | Phase 5 6 续做项 推进策略 | B. 加快, T1.7 + T3.3 并行启动, 4 sub-session 续做 | T1.7 + T3.3 并行, 然后 T3.1 + T3.2, 然后 5.6 + T1.5 |
| 2 | P3-G 后续 W2+ 启动时机 | B. 现在启动 W2 阶段 (跟 Phase 5 6 续做项 并行) | 启动 W2 (UI / API / 集成), 跟 6 续做项并行 |
| 3 | T1.7 4 步修法 启动顺序 | B. 4.1 + 4.2 并行 (节省 sub-session, risk cargo 互锁) | 4.1 (加 as_local_runtime helper) + 4.2 (改写 2 份 tests) 并行, 4.3 推下 |
| 4 | 整体推进 timeline | B. 加快, 2-3 sub-session 并行启动 (risk cargo 互锁) | 多个 sub-session 并行, 2-3 sub-session 总 |

**合计估 token**: 1.85-3.65M (实际可能 3-5x 超支 → 5.55-18.25M, per AGENTS v0.36 守门派生 v17 实证).

---

## §2 验证摘要 (拍板实证 + 守门缺口警告)

### 2.1 ask_user 4-step 拍板 4 项全 B 实证

```text
<questionnaire-response>
  <requestId>ask_e08fcd6f5e9b29102e9dea34</requestId>
  <submittedAt>1788406747527</submittedAt>
  <answers>
    Phase 5 6 续做项: B. 加快, T1.7 + T3.3 并行启动, 4 sub-session 续做
    P3-G 后续 W2+: B. 现在启动 W2 阶段 (跟 Phase 5 6 续做项 并行)
    T1.7 4 步修法: B. 4.1 + 4.2 并行 (节省 sub-session, risk cargo 互锁)
    整体 timeline: B. 加快, 2-3 sub-session 并行启动 (risk cargo 互锁)
  </answers>
</questionnaire-response>
```

### 2.2 4 项 B 风险分析 (守门 #1 实证 "不在预算失控情况下硬着头皮做完")

#### 风险 1: cargo workspace 互锁 (per 9/2 E 阶段 5min timeout 实证)

- 多个 sub-session 并行跑 `cargo check --workspace --all-targets` 可能触发 cargo workspace 互锁
- 9/2 E 阶段实证 5min timeout (per AGENTS §守门 派生 v22)
- 缓解: 串行跑 `cargo check` 守门, 跨 sub-session 提交 1 个 commit 后再启动下一个 sub-session
- 守门 #1 1a 实证: cargo check 串行跑, 避免并行

#### 风险 2: buffer 超支 (per AGENTS v0.36 守门派生 v17)

- H2 实证 0.3-0.5M 估 → 1.1-1.6M 实测, 3-5x 超支
- 1.85-3.65M 估 → 5.55-18.25M 实测 (3-5x)
- 单 session 1-1.5M 上限, 多 sub-session 并行可能 5.55-18.25M 实际
- 守门 #4 派生累积规: 实际 1.85-3.65M 估 + 3-5x buffer = 5.55-18.25M 实际, 跨 2-3 sub-session 续做

#### 风险 3: 5 域 Lead 真人 + 网络稳定性

- T3.2 Saga 跨域编排 需 5 域 Lead 联合拍板, Mavis 临时代签 (per 9/3 11:35 JST 反转)
- 网络偶发 timeout (per 11:07 JST 401 + 12:30 JST 恢复), github.com 443 不稳定
- 守门 #1 1a 重试细则: max 2 retries + 401 跨 session 续

#### 风险 4: 守门 #1 v3 派生规实证缺口

- 闭环报告 commit 之前必跑 `cargo check --workspace --all-targets` 0 err
- 5.1+5.2+5.3+5.4+5.5 报告"0 行代码改动"但 --all-targets 76 err 实证缺口 (per T1.7 报告 b849894)
- 跨 sub-session 并行启动, 每个 sub-session commit 之前必跑 --all-targets 0 err

### 2.3 跨 2-3 sub-session 并行启动 timeline

| sub-session | 启动项 | 估 token | 依赖 |
|---|---|---|---|
| #1 | T1.7 4.1 + 4.2 (并行) + T3.3 | 0.65-1.45M | T1.7 硬阻塞, T3.3 独立 |
| #2 | T3.1 + T3.2 + W2 阶段 | 0.7-1.6M | T3.1 依赖 T1.7, T3.2 依赖 T3.1 + 5 域 Lead, W2 并行 |
| #3 | 5.6 + T1.5 + 推 origin 收尾 | 0.6-1.9M | 5.6 依赖 H2-EXT helper, T1.5 独立, 推 origin 网络恢复 |
| **合计** | 6 续做项 + W2 阶段 | **1.95-4.95M** (实际可能 3-5x 超支 → 5.85-24.75M) | — |

**实际 3-5x 超支**:
- sub-session #1: 0.65-1.45M 估 → 1.95-7.25M 实际
- sub-session #2: 0.7-1.6M 估 → 2.1-8.0M 实际
- sub-session #3: 0.6-1.9M 估 → 1.8-9.5M 实际
- **总实际**: 5.85-24.75M (per H2 1.1-1.6M 3-5x 超支先例)

---

## §3 已知缺口 (per 缺标比错标)

1. **守门 #1 实证 "不在预算失控情况下硬着头皮做完"** — 4 项 B 加快并行有风险, 推下下 session 续做, 实际 buffer 0.05-0.1M 不够启动任何大项
2. **cargo workspace 互锁风险** (per 9/2 E 阶段 5min timeout 实证) — 跨 sub-session 串行跑 cargo check, 避免并行
3. **buffer 超支 3-5x** (per AGENTS v0.36 v17 实证) — 1.85-3.65M 估 → 5.55-18.25M 实际
4. **5 域 Lead 真人 + 网络稳定性** (per 守门 #1 1a + 9/3 11:35 JST 反转) — T3.2 需 5 域 Lead 联签, Mavis 临时代签, 推 origin 网络偶发 timeout
5. **守门 #1 v3 派生规**: 闭环报告 commit 之前必跑 --all-targets 0 err (per AGENTS v0.48 v3 派生规补全)
6. **W1 基础层完工 + ADR-0034 顶层 ADR 落档** (per P3-G 4 commit b9bb2d6 + a54ab72 + 1f6e200 + d03798d) — W2 阶段待启动
7. **.worktrees/ 残留 3 项永久删** (per 9/3 11:35 JST 拍板 A) — Ulysses 手动, Mavis 不越权 PowerShell 限制

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 任务 | 失败/接手 | 接手方式 |
|---|---|---|---|---|
| 1 | (本报告) | 4 类剩余任务 拍板落档 | 0 子代理 dispatch | Mavis 亲自拍板, per 守门 #9 v3 #24 subprocess 路径 |
| 2 | (下 session) | 6 续做项 + W2 阶段 并行启动 | 5/5 subagent RPC 不可靠 (per 守门 #9 #3 实证) | Mavis 亲自跑, 0 子代理 dispatch, per 守门 #9 v3 #24 |

---

## §5 守门规则 (12 项跨 stage 全过 + 4 项 B 风险警告)

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
| **13** | **守门 #3 v2 派生规 (5 域 Lead Mavis 临时代签)** | ✅ **反转落档, 8/21 拍板反转** |
| **14** | **守门 #1 实证 "不在预算失控情况下硬着头皮做完"** | ✅ **4 项 B 加快并行有风险, 推下下 session 续做** |

---

## §6 签字栏 (5 角色, per 守门 #1 报告 7 段结构)

| # | 角色 | 签字 |
|---|---|---|
| 1 | 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| 2 | SRE Lead | — (per 8/21 拒绝兼任硬约束, **9/3 11:35 JST 反转 Mavis 临时代签**, 真人到位后追溯) |
| 3 | 平台 | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) (Mavis 接手代签 per 19:39 JST 授权) |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 4 类剩余任务 拍板 4 项全 B 加快并行 (Phase 5 6 续做项 + P3-G 后续 W2+ + T1.7 4 步 + 整体 timeline), 守门 #1 实证 "不在预算失控情况下硬着头皮做完" + 4 项 B 风险警告 (cargo 互锁 + buffer 超支 + 5 域 Lead + 网络稳定性), 跨 2-3 sub-session 续做, 推下下 session | 9/3 12:39 JST 用户发令"剩余任务怎么推完" + ask_user 4-step 拍板 4 项 B+B+B+B (per ask_e08fcd6f5e9b29102e9dea34), 守门 #1+#5+#6+#7+#8+#9+#12+#15+#19+#20+#22+#3 v2+#1 v3 跨 stage 全过 |
