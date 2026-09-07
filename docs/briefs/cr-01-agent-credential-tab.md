# Brief: CR-01 — Agent 界面凭证 tab TS 实施 (per ADR-0051)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_5ec955bc1bdbd590786c2039` 2 推荐项 (next-step = cr-01-impl + goal-finalize = mark-complete-with-blocked-items)
> **wt-branch**: `wt-cr-01-agent-credential-tab`
> **base**: `main @ 51884fc` (WBS v0.9 推 origin 完成)
> **触发**: per 2026-09-08 05:30 JST 用户发令"这些凭证ai相关的允许用户在agent界面自己填" + 5:38 JST 用户拍板选 CR-01 实施
> **关联**: [ADR-0051 凭证 UX 分类拍板 (v1.0)](../architecture/2026-08-26-upgrade/adr/0051-credential-ux-routing.md) · [V2-1 crates/star-credential (KMS 加密)](../../crates/star-credential/) · [TD-01 ADR-0049 (env_var passthrough)](../architecture/2026-08-26-upgrade/adr/0049-ai-tool-auto-discovery.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [守门 #5 env 安全](../../AGENTS.md) · [守门 #13 c RLS 13 类](../../AGENTS.md) · [守门 #14 v2 5 域 Lead 临时代签](../../AGENTS.md)

---

## 1. 目标 (Objective)

在 `frontend/src/lib/agent/` + `frontend/src/components/agent/` 落档 AI 凭证管理 (per-agent 持久化 + KMS 加密) 的 TS 实施, 跟现有 `crates/star-credential` + gm-console AppShell 集成, 实现 9 SA + SA-10 各自 agent 配置页加"凭证" tab (per-agent 持久化, 走 Agent 界面 UX 入口).

## 2. 范围 (Scope)

### 2.1 In-Scope (CR-01 精简版)

- `frontend/src/lib/agent/types.ts` (AI 凭证类型定义, ~1KB)
- `frontend/src/lib/agent/credentials.ts` (`AgentCredentialStore` + 5 个 API, ~3KB)
- `frontend/src/components/agent/CredentialTab.tsx` (凭证 tab 组件, ~2KB)
- `frontend/src/lib/agent/__tests__/credentials.test.ts` (4 UT, ~2KB)
- 跟现有 frontend 集成 (gm-console AppShell 引用 stub, 跨 session 续)

### 2.2 Out-of-Scope (跨 session 续)

- ❌ CR-02 设置界面凭证 section (TS 实施) — 跨 session 续
- ❌ CR-03 跟 TD-01 集成 (env_var passthrough 优先) — 跨 session 续
- ❌ CR-04 跟 V2-1 集成 (KMS 加密 + RLS 13 类) — 跨 session 续
- ❌ 9 SA + SA-10 实际 agent 页面 UI 改造 — 跨 session 续

## 3. 已知缺口 (per 守门 #11 缺标比错标)

- 本地 pnpm/vitest 环境受限 (per EX-05 实证), TypeScript 测试跨 session 续
- KMS 加密调用跨 session 续 (mock 备选)
- gm-console AppShell 9 SA + SA-10 agent 页面实际 UI 改造跨 session 续

## 4. 守门 (per 守门 #1 4 守门 + #5 + #7 + #10 + #13 + #14 v2)

1. **守门 #1 TypeScript 4 步**: pnpm tsc / eslint / test / build — 本机环境受限, 跨 session 续
2. **守门 #5 env 安全**: 不打印 value, 仅 key 引用 (跟 TD-01 D-03 一致)
3. **守门 #7 0 unsafe**: TypeScript 无 unsafe
4. **守门 #10 author=Ulysses**: 1 commit, author=`Ulysses <ulysses@mavis.local>`
5. **守门 #11 缺标比错标**: §3 显式列 3 缺口
6. **守门 #13 c RLS 13 类**: per-agent 隔离 (per V2-1)
7. **守门 #14 v2**: Mavis 临时代签 admin 域 (Agent 界面凭证管理)
8. **守门 #19 v19**: 不适用 (TS 实施, 非 Python agent 交互)
9. **守门 #9 v20**: brief 必先落档 (本 brief)
10. **守门 #15 饱和**: 用户发令=新事件, 符合

## 5. 依赖

### 5.1 上游

- Star-EI 8/8 wt 全部收官 + 推 origin (per v0.7)
- TD-01 ADR-0049 (4 源 scanner + env_var passthrough) 收官 (per `ca7971f`)
- ADR-0050 (5 Tab 命名) 收官 (per `cf5a95d`)
- ADR-0051 (凭证 UX 分类) 收官 (per `7880d70`)
- V2-1 crates/star-credential (KMS 加密) 已落地 (9/4 commit)

### 5.2 下游阻塞

- CR-02/03/04 跨 session 续
- gm-console AppShell 9 SA + SA-10 agent 页面实际 UI 改造跨 session 续

## 6. 交付物 (Deliverables)

| # | 路径 | 描述 | 大小 |
|---|---|---|---|
| 1 | `frontend/src/lib/agent/types.ts` | AI 凭证类型 (LLM / Code / Search / Embedding / Custom) | ~1KB |
| 2 | `frontend/src/lib/agent/credentials.ts` | `AgentCredentialStore` (CRUD + KMS 引用) | ~3KB |
| 3 | `frontend/src/components/agent/CredentialTab.tsx` | 凭证 tab 组件 (per-agent 持久化) | ~2KB |
| 4 | `frontend/src/lib/agent/__tests__/credentials.test.ts` | 4 UT (CRUD + 双键 dedup) | ~2KB |
| 5 | `docs/briefs/cr-01-agent-credential-tab.md` | 本 brief | ~6KB |
| **总** | | **5 文件, ~14KB raw** | |

## 7. 验收 (Acceptance Criteria)

### 7.1 守门实证

- [ ] `cargo check --workspace --lib -j 4` exit 0 (TS 不进 main 编译, 验证 0 错)
- [ ] `pnpm tsc --noEmit` exit 0 (跨 session 续, per EX-05 受限)
- [ ] `pnpm test --testPathPattern=agent/credentials` 100% pass (4/4 UT, 跨 session 续)
- [ ] `git log -p --follow frontend/src/lib/agent/credentials.ts` 实证类完整
- [ ] commit author = Ulysses per 守门 #10

### 7.2 4 想定シナリオ (S-CR-01..S-CR-04)

- [ ] **S-CR-01** Agent 界面填 OpenAI key, per-agent 持久化 (per V2-1 KMS 加密 stub)
- [ ] **S-CR-02** Agent 界面读 key, 不打印 value (per 守门 #5)
- [ ] **S-CR-03** Agent 界面删 key, 触发 KMS revoke (跨 session 续 V2-1)
- [ ] **S-CR-04** 双键 dedup (per EX-08 IT-02 RLS 13 类隔离, 跨 session 续)

## 8. 实施路径 (per 守门 #9 v3 fallback)

### 8.1 Mavis 直接落地 (per 守门 #9 v3 实证 5/5 subagent RPC 不可靠)

1. 创建 wt: `git worktree add ../.worktrees/wt-cr-01-agent-credential-tab -b wt-cr-01-agent-credential-tab main`
2. Mavis 在 wt 内写 5 文件 (per §6 交付物, brief 已落)
3. 实证守门 #1 TypeScript 4 步 (本机受限, cargo check 0 err 实证)
4. `git add` + `git commit -m "..."` author=Ulysses
5. 切回 main + `git merge --no-ff wt-cr-01-agent-credential-tab`
6. `git push origin main` (per 守门 #1 反转 9/7 21:08 JST)

### 8.2 失败接手 (per 守门 #9 v3 fallback)

- Mavis 直做失败 → 不再派 worker subagent (实证 5/5 RPC 不可靠, 简化)

## 9. 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| pnpm/vitest 环境受限 | 中 | 中 | 跨 session 续, 跟 EX-05 一致 |
| KMS 加密调用 | 低 | 中 | V2-1 已有, 复用, mock 备选 |
| AppShell 9 SA + SA-10 实际 UI | 中 | 中 | 跨 session 续, 简化版仅 1 个 component |

## 10. 签字 (5 角色 Mavis 临时代签 per 守门 #14 v2)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 05:38 JST | Ulysses — Mavis 接手 | 初稿, 5 文件 ~14KB, 10 守门 | per `ask_5ec955bc1bdbd590786c2039` 拍板 + ADR-0051 + 守门 #9 v20 |
