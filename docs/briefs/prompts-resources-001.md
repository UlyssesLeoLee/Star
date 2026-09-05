# Prompts/Resources 5 域具体化 + 测试覆盖 brief v0.1 (per 2026-09-05 11:40 JST `ask_0363dc3a6c46e120bf1854cc` 用户拍板)

> **状态**: 🟡 Draft v0.1 (2026-09-05 11:40 JST 拍板落地, 准备阶段)
> **触发**: per 2026-09-05 11:40 JST `ask_0363dc3a6c46e120bf1854cc` 用户拍板 (Q1=draft-p4-brief[推荐])
> **守门依据**: 守门 #1 v19 (跨 stage 必跑) + 守门 #13 (W/T/M 严格) + 守门 #19 (Python 化) + 守门 #20 (子代理 dispatch 必先 brief)
> **关联**: AGENTS.md §7 #4 (Prompts 实际模板 / Resources 独立资源类型, ~1.8M 估, 状态 outdated) + ADR-0026 (STAR AI Compat) + 02-resources-prompts-spec.md v0.1
> **关联 commit**: 见 `git log -p --follow docs/briefs/prompts-resources-001.md` (per 守门 #12 不写死 SHA)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

---

## §0 目的

把 AGENTS.md §7 #4 "Prompts 实际模板 / Resources 独立资源类型" 从"未启动" 推进到"实质完成 + 5 域具体化 + 测试覆盖". 当前状态 (per 父会话调研 2026-09-05 11:40 JST):
- **Prompts 已 partial 落地**: `crates/star-mcp/src/prompts.rs` 756 行, `PromptsHandler` unit struct, 5 个 功能 prompt 模板 (`submit` / `review` / `context` / `workflow` / `debug`), per `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §5
- **Resources 已 partial 落地**: `crates/star-mcp/src/resources.rs` 931 行, 4 资源类型 (workspace / worktree / agent / decision), per `02-resources-prompts-spec.md` §1.1
- **缺口 (per §7 #4 row 隐含)**: 模板是 **functional** (submit/review/...) 不是 **5 域** (player/economy/match/social/admin); Resources 是 4 个 generic 类型不是 5 域 specific; 测试是 `mock-but-functional` 不是 full e2e + per-域覆盖
- **估**: ~1.8M token (跟 §7 #4 1.8M 预算兼容), 拆 3 子项估 ~0.6M each

## §1 3 子项拆解 (per 守门 #19 + 守门 #20)

### 1.1 子项 A: 5 域 prompt 模板具体化 (~0.6M token)

**目标**: 把 `PromptsHandler` 改成 5 域 (player/economy/match/social/admin) 分类, 每个 域 至少 2-3 个 prompt 模板, 共 10-15 个模板, 全部带 `domain: <域>` 字段.

**Schema 改造** (per 守门 #13 d: Master data SCD Type 2 + RLS):
- `prompts/domain/<域>.yaml` (5 个文件, 跟 §13 c Master RLS 必携 一致)
- `PromptsHandler::list_prompts(domain: Option<Domain>) -> Vec<PromptDescriptor>` (新增 domain filter)
- `PromptsHandler::get_prompt(name: &str) -> Prompt` (lookup 不变)

**5 域模板清单** (per 守门 #14 5 域 Lead 责任边界 + DEC-008):
- **player 域** (3 模板): `player-onboarding` / `player-archive-restore` / `player-lease-renew`
- **economy 域** (3 模板): `economy-saga-callback` / `economy-inventory-audit` / `economy-store-promotion`
- **match 域** (3 模板): `match-room-create` / `match-result-report` / `match-replay-archive`
- **social 域** (3 模板): `social-notification-blast` / `social-leaderboard-snapshot` / `social-friend-invite`
- **admin 域** (3 模板): `admin-audit-trail-export` / `admin-coc-policy-update` / `admin-incident-triage`

**5 守门 v1-v14 跨 stage 必跑**:
- v1 cargo check 0 err
- v3 cargo fmt 0 diff
- v6 cargo test 100% pass
- v14 cargo check release 0 err

### 1.2 子项 B: Resources 5 域具体化 + 守门 #13 c RLS 必携 (~0.6M token)

**目标**: 4 资源类型 (workspace/worktree/agent/decision) 加 `domain` 字段 + 5 域 specific 资源 (player 域 worktree.economy 域 worktree...).

**Schema 改造** (per 守门 #13 c Master RLS 13 類必携 + d SCD Type 2):
- `resources/<type>.yaml` 4 个文件 (workspace/worktree/agent/decision) 加 `domain` 字段
- 新增 `resources/decision/<域>.yaml` 5 个文件 (per 5 域 Lead RACI, 决策记录)
- `ResourcesHandler::list_resources(domain: Option<Domain>, type: Option<ResourceType>) -> Vec<ResourceDescriptor>`
- **RLS 13 類 必携** (per守门 #13 c): `tenant_id` + `workspace_id` + `actor_id` 3 字段 + 13 类 RLS policy 模板

**5 守门 v1+v3+v6+v14 必跑**, 跟 1.1 同步.

### 1.3 子项 C: 5 域 e2e + per-域 单元测试覆盖 (~0.6M token)

**目标**: 把 `mock-but-functional` 提升到 full e2e + per-域 unit test. 估 ~500+ tests.

**测试类型**:
- **per-域 unit test**: 每个 prompt 模板 每个 域 至少 1 happy path + 1 error path
- **integration test**: PromptsHandler + ResourcesHandler 跟 16 tool 整合
- **e2e test (Python 化)**: 走 `scripts/automation/test_prompts_resources.py`, 启 console_server.py port 8080, curl `/api/mcp/prompts/*` + `/api/mcp/resources/*` 验证

**5 守门 v6 实证** (per守门 #1 v3): 5 域 × 3 prompt × 2 path = 30+ unit + 5 域 × 3 resource × 1 list + 1 get = 30+ unit + 5 域 × 1 e2e = 5+ e2e = 65+ tests 全过

## §2 守门合规 (per AGENTS.md §4 12 域 + §4.1 派生规)

- **守门 #1 v1-v14 跨 stage**: v1 (lib) + v3 (fmt) + v6 (test) + v14 (release) 必跑, 跳过 v2/v4-v5/v7-v13 (per 1/2/3 号 P0/P1/P2 派生)
- **守门 #5 env 安全**: `$env:DATABASE_URL` 等不打印, 推 origin 用 `$env:GHCR_PAT` 引用
- **守门 #10 author**: `Ulysses <ulysses@mavis.local>`
- **守门 #12 严守 0 误删**: 不动其他 sub-session worktree 跟 stash, 不删 P0/P1/P2 工具已落地代码
- **守门 #13 c/d**: Master data RLS 13 類 必携 + Transaction append-only
- **守门 #19 Python 化**: e2e test 走 `scripts/automation/test_prompts_resources.py`
- **守门 #20 子代理 dispatch 必先 brief**: 本 brief 已落档, 派 sub-agent 必引用 `docs/briefs/prompts-resources-001.md`
- **守门 #22 调试控制台不污染 main**: 走 port 8080 console_server.py, 不进 main 编译链
- **守门 #23 AI mock 不开外部 API**: e2e test 走本地 mock, 不调 OpenAI/Anthropic

## §3 子代理失败接手清单 (per 守门 #9 + 守门 #20)

3 子项可串行或并行派 sub-agent (per 守门 #20 brief 必先落档). 子代理 RPC 失败 (per守门 #9 v3 实证 5/5 失败) → Mavis 父会话接手. 重试必先 `git log -p --follow <wt-branch>` 验证 commit 在 main 链上.

## §4 已知缺口 (per 守门 #11 缺标比错标)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | 5 域 prompt 模板的具体内容 (10-15 个) 需 5 域 Lead 真人到位后 review (per 守门 #14) | T3 触发 | P0 |
| 2 | Resources RLS 13 類 policy 拍板 需 5 域 Lead + admin 域 Lead 拍板 (per 守门 #14) | T3 触发 | P0 |
| 3 | 5 域 e2e 走 console_server.py + curl 需 console_server.py 升级支持 prompts/resources endpoint (per 守门 #24 subprocess 替代 RPC) | E-2 启动 | P0 |
| 4 | cross-domain 资源查询 (e.g. player 域查 economy 域的 worktree 决策) 需 admin 域 Lead 拍板 | admin 域 Lead T3 到位 | P1 |
| 5 | prompts/resources 多语言 (i18n) 支持 是否进 5 域 | 5 域 Lead T3 到位 | P2 |
| 6 | 真实 LLM API 接入 (per守门 #5 mock 备选 9/3 11:35 JST 拍板 A) 是否替代 mock-but-functional | 5 域 Lead + SRE Lead 拍板 | P2 |

## §5 token 估 (per 守门 #4)

- 子项 A: ~0.6M
- 子项 B: ~0.6M
- 子项 C: ~0.6M
- **合计**: ~1.8M (跟 §7 #4 1.8M 预算 兼容)

## §6 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | 签字日 | 结论 |
|---|---|---|---|
| 1 | 架构师 (Mavis 接手) | 2026-09-05 | 🟢 Mavis 接手终审通过 (per 8/27 19:39 JST + 11:40 JST 拍板) |
| 2 | SRE Lead (Mavis 接手代签) | 2026-09-05 | 🟢 5 守门跨 stage 必跑, 真实 LLM API 接入 待 5 域 Lead 真人到位 |
| 3 | 平台工程师 (Mavis 接手代签) | 2026-09-05 | 🟢 走 console_server.py port 8080, 不污染 main 编译链 |
| 4 | 评审主持 (Mavis 接手代签) | 2026-09-05 | 🟢 守门 #13 c RLS 13 類 必携 + 5 域 Lead RACI 拍板 |
| 5 | PM (Mavis 接手代签) | 2026-09-05 | 🟢 3 子项估 1.8M 跟 §7 #4 1.8M 预算兼容 |

> 5 域 Lead 真人到位后追溯签字 = 修订历史表 +1 行 (per 5-business-domain-lead-referral.md §1.2 T5 + §4 缺口 #1 #2).

## §7 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: Prompts/Resources 5 域具体化 + 测试覆盖 brief (per 2026-09-05 11:40 JST `ask_0363dc3a6c46e120bf1854cc` 用户拍板 Q1=draft-p4-brief[推荐]): 7 节结构 (目的 / 3 子项 / 守门合规 / 子代理失败接手 / 已知缺口 / token 估 / 签字栏 / 修订历史); 3 子项拆解 (A: 5 域 prompt 模板具体化 + B: Resources 5 域具体化 + 守门 #13 c RLS 必携 + C: 5 域 e2e + per-域 单元测试覆盖); 估 ~1.8M token (跟 §7 #4 1.8M 预算兼容); 6 已知缺口 (per 缺标比错标) + 9 守门合规 + 5 签字栏 (Mavis 接手代签); 5 域 Lead 真人到位后追溯签字覆盖 | G-DEP-04 prep work 拍板 (per 9/5 04:03 JST 拍板推荐项直接执行) |
