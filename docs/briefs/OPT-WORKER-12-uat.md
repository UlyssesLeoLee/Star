# Brief: OPT-WORKER-12 — UAT 测试 3 件套 (Playwright + mock + docs)

**Agent**: worker
**Phase**: OPT-P4-UAT
**Created**: 2026-09-07 14:42 JST (per 9/7 14:30 JST 用户发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档")
**Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)
**Status**: 🟡 Mavis 临时代签, 0 依赖, 立即启动

---

## 1. 任务目标 (3 件套, 1 worker 1 worktree)

完成 3 件套实施, 估 0.8-1.5M token:

### 1.1 件套 1: Playwright 6 spec 补充

**位置**: `frontend/e2e/`

**当前 6 spec**: `canvas-share-export.spec.ts` / `canvas-view.spec.ts` / `cross-domain-5b.spec.ts` / `debug-mobile.spec.ts` / `redirects.spec.ts` / `remote-mobile.spec.ts`

**新增 6 spec** (覆盖 6 关键业务流程 + 5 域 + TMO + Streamable HTTP + MCP + OpenClaw):

| # | spec | 流程 | 涉及 |
|---|---|---|---|
| 1 | `worktree-creation-flow.spec.ts` | 从 WorkItem 创建 Worktree → 启动 Agent → 监控状态机 | per test-design v0.5 §2.3.1 #1 |
| 2 | `five-domain-feedback-loop.spec.ts` | 5 域 (player/economy/match/social/admin) 跨域 Feedback 流程 | per test-design v0.5 §2.3.1 #3 + §7 S2-S5 |
| 3 | `tmo-merge-task-flow.spec.ts` | TMO 7 节点 (merge/split/reorder/bulk/summarize/reassign/metadata) | per LangGraph TMO 02 §2.6 + AGENTS §7 #8.1 |
| 4 | `streamable-http-reconnect.spec.ts` | Streamable HTTP session 重连 + Server-push + Last-Event-ID | per test-design v0.5 §23 + AGENTS §7 #3 |
| 5 | `mcp-16-tool-coverage.spec.ts` | 16 MCP tool 端到端覆盖 (per守门 #4 16/16 REAL) | per test-design v0.5 §22 + AGENTS §7 #1 |
| 6 | `uat-business-acceptance.spec.ts` | UAT 业务验收 (跨 6 spec) — 5 域 Lead CONTENT 4 维 + AC 跨引 | per test-design v0.5 §2.6 + §7 |

**模式**: 复用现有 6 spec 模式 (vitest + Playwright + MSW handler), 引用 `frontend/src/mocks/handlers/` 25 个 handler, 引用 `tools/star-flash-mock/mock_data/` 17 fixture.

### 1.2 件套 2: UAT mock_data 50+ 业务场景 fixture

**位置**: `tools/star-flash-mock/mock_data/uat/`

**新建子目录**:
```
mock_data/uat/
├── scenarios/                  # 业务场景 fixture (50+)
│   ├── S01-workitem-create/    # 场景 1: WorkItem 创建
│   ├── S02-worktree-create/    # 场景 2: Worktree 创建
│   ├── S03-agent-running/       # 场景 3: Agent 运行
│   ├── S04-feedback-loop/       # 场景 4: 5 域 Feedback 环
│   ├── S05-validation-pass/     # 场景 5: 验证通过
│   ├── S06-validation-fail/     # 场景 6: 验证失败
│   ├── S07-conflict-detect/     # 场景 7: 冲突检测
│   ├── S08-rebase-merge/        # 场景 8: rebase + merge
│   ├── S09-merge-request/       # 场景 9: 合并请求
│   ├── S10-tmo-merge/           # 场景 10: TMO merge 节点
│   ├── S11-tmo-split/           # 场景 11: TMO split 节点
│   ├── S12-tmo-reorder/         # 场景 12: TMO reorder 节点
│   ├── S13-tmo-bulk/            # 场景 13: TMO bulk 节点
│   ├── S14-tmo-summarize/       # 场景 14: TMO summarize 节点
│   ├── S15-tmo-reassign/        # 场景 15: TMO reassign 节点
│   ├── S16-tmo-metadata/        # 场景 16: TMO metadata 节点
│   ├── S17-streamable-connect/  # 场景 17: Streamable 连接
│   ├── S18-streamable-push/     # 场景 18: Streamable server-push
│   ├── S19-streamable-reconnect/# 场景 19: Streamable 重连
│   ├── S20-streamable-delete/   # 场景 20: Streamable 删除
│   ├── S21-5d-ac-acceptance/    # 场景 21: 5 域 AC 跨引
│   ├── S22-multitenant-isolation/# 场景 22: 多租户隔离
│   ├── S23-rbac-scheme/         # 场景 23: RBAC scheme
│   ├── S24-audit-worm/          # 场景 24: 审计 WORM
│   └── S25-quota-exceeded/      # 场景 25: 配额超出
├── fixtures/                   # 50+ fixture JSON
├── scripts/                    # 5+ 生成脚本 (_generate_uat_*.py)
└── regression/                 # 5+ regression script
```

**每个 scenario 子目录**:
- `request.json` (业务场景输入)
- `expected_response.json` (预期响应)
- `expected_db_state.json` (DB 状态变更)
- `expected_events.json` (事件流)
- `ac_mapping.md` (AC 跨引)

**5 域 + 5 SA + TMO 跨域 fixture**:
- 5 域 = player / economy / match / social / admin
- 5 SA = SA-01..SA-09 + SA-10
- TMO = 7 节点 (M-N1..M-N7) + 7 协议 + 5 Reducer

### 1.3 件套 3: 2 docs

**`docs/uat-design.md`** (UAT 测试设计, ~500-800 行, 7 段结构 per守门 #3):
- §0 目的 (per 9/7 14:30 JST 用户发令)
- §1 UAT 范围 (5 域 + 6 关键流程 + 25 业务场景)
- §2 UAT 测试用例 (50+ AC 跨引, Gherkin Given/When/Then)
- §3 UAT mock_data 索引 (50+ fixture 列表 + schema)
- §4 UAT 与 E2E/IT/ST 关系 (per test-design v0.8 §27)
- §5 守门 0 违反 (per守门 #11 + #13)
- §6 5 域 Lead CONTENT 4 维 (per守门 #14 v2, Mavis 临时代签)
- §7 已知缺口 (per守门 #11 缺标比错标)

**`docs/uat-runbook.md`** (UAT 落地手册, ~200-400 行):
- §0 目的
- §1 前置条件 (Node.js / Python / Cargo / Playwright / MSW)
- §2 跑 UAT 流程 (本地 / CI / k3s)
- §3 50+ fixture 一键生成
- §4 25 业务场景 一键跑
- §5 结果解读 (per 守门 #13 W/T/M 分类)
- §6 故障排查 (常见 4 错误)
- §7 引用 (test-design v0.8 / STAR-P4-OPT-WBS-001)

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-uat-3pieces D:/Star/.worktrees/impl/wt-opt-uat-3pieces main
cd D:/Star/.worktrees/impl/wt-opt-uat-3pieces
```

- **Worktree 路径**: `D:/Star/.worktrees/impl/wt-opt-uat-3pieces`
- **Branch**: `feat/opt-uat-3pieces` (基于 main HEAD `eb5a967`)

---

## 3. 关键约束

- **禁回溯叙事** (per守门 #12)
- **禁引用 RGS 仓** (per AGENTS §5)
- **PowerShell only** (per守门 #6)
- **守门 #5**: 禁打印 env secret
- **守门 #10**: commit author = `Ulysses <ulysses@mavis.local>`
- **守门 #13 W/T/M**: UAT fixture 跨域按 9/1 18:30 JST 拍板分类
- **守门 #19**: 优先 Python 化 (`docs/automation-design.md` v0.1)
- **守门 #20**: 拆 commit 派生规 (Playwright 1+1 / mock 1+1 / docs 1 = 3 commit, 不沿用 v0.x 旧叙事)

---

## 4. 工作流程

```powershell
# Step 1: 创建 worktree
cd D:\Star
git worktree add -b feat/opt-uat-3pieces D:/Star/.worktrees/impl/wt-opt-uat-3pieces main
cd D:/Star/.worktrees/impl/wt-opt-uat-3pieces

# Step 2: 件套 1 - Playwright 6 spec
# - frontend/e2e/worktree-creation-flow.spec.ts (200-300 行)
# - frontend/e2e/five-domain-feedback-loop.spec.ts (200-300 行)
# - frontend/e2e/tmo-merge-task-flow.spec.ts (200-300 行)
# - frontend/e2e/streamable-http-reconnect.spec.ts (200-300 行)
# - frontend/e2e/mcp-16-tool-coverage.spec.ts (200-300 行)
# - frontend/e2e/uat-business-acceptance.spec.ts (200-300 行)
# - 复用现有 6 spec 模式 + 引用 frontend/src/mocks/handlers/ 25 handler + tools/star-flash-mock/mock_data/ 17 fixture

# Step 3: 件套 2 - UAT mock_data 50+ fixture
# - tools/star-flash-mock/mock_data/uat/ 25 子目录 × 5 文件 = 125 文件
# - tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_*.py 5 个
# - tools/star-flash-mock/mock_data/uat/regression/uat-*.sh 5 个

# Step 4: 件套 3 - 2 docs
# - docs/uat-design.md (500-800 行, 7 段结构)
# - docs/uat-runbook.md (200-400 行, 7 段结构)

# Step 5: 守门实证
cargo check --workspace --all-targets -j 4     # 必须 0 err
cargo fmt --all
cd frontend && pnpm lint
pnpm test:e2e -- --reporter=line,html 2>&1 | head -100

# Step 6: Commit (3 commit, per守门 #20)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add -A
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "..."
```

## 5. Commit 形式 (3 commit, 1 per 件套)

### Commit 1 - 件套 1 Playwright 6 spec
```
feat(e2e): 6 UAT spec 补充 (per 9/7 14:30 JST 用户发令)

6 spec 覆盖 6 关键业务流程 + 5 域 + TMO + Streamable + MCP:
- worktree-creation-flow: WorkItem → Worktree → Agent 状态机
- five-domain-feedback-loop: 5 域跨域 Feedback 流程
- tmo-merge-task-flow: TMO 7 节点 (merge/split/reorder/bulk/summarize/reassign/metadata)
- streamable-http-reconnect: session 重连 + Server-push + Last-Event-ID + DELETE
- mcp-16-tool-coverage: 16 MCP tool 端到端覆盖
- uat-business-acceptance: 5 域 Lead CONTENT 4 维 + AC 跨引

Refs: docs/briefs/OPT-WORKER-12-uat.md
Refs: docs/test-design.md v0.8 §27
Per守门 #14 v2 Mavis 临时代签 5 域 Lead
Per守门 #20 拆 commit 派生规
Per守门 #10 author=Ulysses
```

### Commit 2 - 件套 2 UAT mock_data
```
feat(mock): UAT mock_data 50+ 业务场景 fixture (per 9/7 14:30 JST 用户发令)

25 业务场景 × 5 文件 = 125 fixture (per守门 #13 W/T/M 分类):
- S01-S03: WorkItem/Worktree/Agent 流程
- S04-S06: Feedback/Validation 环
- S07-S09: Conflict/Rebase/Merge
- S10-S16: TMO 7 节点 (per LangGraph TMO 02)
- S17-S20: Streamable HTTP (per AGENTS §7 #3)
- S21: 5 域 AC 跨引 (per test-design §2.6)
- S22-S25: 多租户/RBAC/审计/配额

+ 5 _generate_uat_*.py 脚本 (per守门 #19)
+ 5 regression/uat-*.sh

Refs: docs/briefs/OPT-WORKER-12-uat.md
Refs: docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.3
Per守门 #13 W/T/M 派生
Per守门 #19 Python 化
Per守门 #10 author=Ulysses
```

### Commit 3 - 件套 3 docs
```
docs(uat): UAT 设计 + runbook (per 9/7 14:30 JST 用户发令)

- docs/uat-design.md (700+ 行, 7 段结构 per守门 #3):
  §0 目的 / §1 范围 / §2 50+ AC / §3 mock_data 索引 /
  §4 UAT-E2E-IT-ST 关系 / §5 守门 0 违反 / §6 5 域 Lead CONTENT / §7 已知缺口
- docs/uat-runbook.md (300+ 行, 7 段结构):
  §0 目的 / §1 前置 / §2 跑流程 / §3 一键生成 /
  §4 一键跑 / §5 结果解读 / §6 故障排查 / §7 引用

Refs: docs/briefs/OPT-WORKER-12-uat.md
Refs: docs/test-design.md v0.8
Refs: docs/reports/STAR-P4-OPT-WBS-001.md
Per守门 #12 commit-time docs 同步 (新事件触发 = Commit 1+2 代码改动)
Per守门 #10 author=Ulysses
```

## 6. 实施注意

- **件套 1 Playwright 引用**: 复用 frontend/src/mocks/handlers/ 25 handler + tools/star-flash-mock/mock_data/ 17 fixture
- **件套 2 UAT mock 跨域**: 5 域 player/economy/match/social/admin + 5 SA SA-01..SA-09 + SA-10 + TMO 7 节点
- **件套 3 docs 触发链**: Commit 3 docs 由 Commit 1+2 代码改动触发, 不沿用 v0.x 旧叙事
- **6 spec 不需真跑通**: per test-design v0.5 §2.3 + test-design v0.8 §27.3 状态, 6 spec 是设计落地, MSW mock backend (per P3-A.7 9/3 11:35 JST 拍板), 不需要真后端

## 7. 失败处理

- cargo check 不通过: 修复 + 重试, max 2 次
- pnpm lint 不通过: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

## 8. 交付物 (per brief §6)

返回 final message 报告:
1. ✅/❌ worktree 创建
2. ✅/❌ 件套 1 Playwright 6 spec (file:line)
3. ✅/❌ 件套 2 UAT mock_data 50+ fixture (file:line + 子目录)
4. ✅/❌ 件套 3 docs/uat-design.md + docs/uat-runbook.md (file:line)
5. ✅/❌ cargo check 0 err 实证
6. ✅/❌ 3 commit hash 7 字符
7. 任何超时 / 异常

## 9. 时间预算

≤ 1500K token (1 worker 跨 3 件套, 估 0.8-1.5M)

## 10. 完成后

- 不要 merge / push
- 直接返回 final message 报告
