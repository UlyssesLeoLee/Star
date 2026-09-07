# Brief: OPT-WORKER-13 — 100% 覆盖 404 路径 + 交互预期 + 协作预期

**Agent**: worker
**Phase**: OPT-P4-UAT-100
**Created**: 2026-09-07 16:17 JST (per 9/7 16:15 JST 用户发令 "测试结果中是否存在404或者交互不符合预期，协作不符合预期，这些都要100%覆盖")
**Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)
**Status**: 🟡 Mavis 临时代签, 0 依赖, 立即启动

---

## 1. 任务目标 (3 件套 + 2 docs, 1 worker 1 worktree)

完成 100% 覆盖 3 类失败/异常, 估 1-2M token:

### 1.1 件套 1: Playwright 4 spec (100% 覆盖 3 类)

**位置**: `frontend/e2e/`

**当前 6 spec** (per OPT-WORKER-12): `worktree-creation-flow` / `five-domain-feedback-loop` / `tmo-merge-task-flow` / `streamable-http-reconnect` / `mcp-16-tool-coverage` / `uat-business-acceptance`

**新增 4 spec** (覆盖 3 类失败/异常):

| # | spec | 覆盖类 | 详细 |
|---|---|---|---|
| 1 | `error-404-path-coverage.spec.ts` | **404 路径** | (a) 3 incidents 能力 negative missing (per REQ-OPS-003 §30.6) + (b) 4 pre-existing star-mcp tools failed (find_references / get_code_context / get_symbol / search_code) + (c) 2 tsc err (src/app/agent-view/page.tsx:398 + src/lib/store.ts:562) + (d) 任何 404/NotFound/Capability not implemented 路径 |
| 2 | `interaction-expectation-coverage.spec.ts` | **交互不符合预期** | (a) 跨 session 异步响应时间 > 5s 警告 + (b) UI 组件 prop 类型不匹配 (per 2 tsc err) + (c) async race condition (5 域并发 update) + (d) MSW handler 跟真后端契约一致性 + (e) error boundary 兜底 + (f) 任何用户预期违反 |
| 3 | `collaboration-5d-lead-coordination.spec.ts` | **协作不符合预期** | (a) 5 域 Lead 跨域协调 (player/economy/match/social/admin) + (b) Mavis 临时代签 跟真人到位追溯签字一致性 + (c) 5 SA SA-01..SA-09 + SA-10 task-orchestrator 跨 sub-agent 协调 + (d) TMO 7 节点 (M-N1..M-N7) 跨域编排 + (e) 守门 #13 a L1↔L1 禁止 (L0 协调 实证) + (f) 跨 session 通信 + (g) worktree 跨域 resource contention |
| 4 | `uat-100pct-coverage-assertion.spec.ts` | **100% 覆盖 meta** | (a) 跑现有 6 spec + 新 3 spec, 全 pass 才算 100% 覆盖; (b) 失败 0 容忍; (c) timing assertion (每 spec < 30s); (d) 跨 spec state 隔离 |

**模式**: 复用现有 6 spec 模式 (vitest + Playwright + MSW handler), 引用 `frontend/src/mocks/handlers/incidents.ts` 3 NOT_IMPLEMENTED_404.

### 1.2 件套 2: UAT 10 新 fixture (S26-S35 10 业务场景)

**位置**: `tools/star-flash-mock/mock_data/uat/scenarios/`

**当前 25 场景** (per OPT-WORKER-12): S01-S25

**新增 10 场景** (S26-S35, 覆盖 3 类失败/异常):

| # | scenario | 覆盖类 | 详细 |
|---|---|---|---|
| 26 | S26-notimplemented-incident-probe | 404 路径 | GET /api/incidents/probe-production → 404 + Capability not implemented (per REQ-OPS-003 §30.6) |
| 27 | S27-notimplemented-incident-alert | 404 路径 | POST /api/incidents/process-alert → 404 + 同上 |
| 28 | S28-notimplemented-incident-rollback | 404 路径 | POST /api/incidents/:id/auto-rollback → 404 + 同上 |
| 29 | S29-mcp-tool-failed-empty-result | 404 路径 | 调用 find_references / get_code_context / get_symbol / search_code 4 tool, 验证空结果 + 跟期望一致 (per 9/5 报告 §3.7 + D.3 worker 已知缺口 #1) |
| 30 | S30-tsc-err-render-fallback | 交互预期 | 触发 src/app/agent-view/page.tsx:398 tsc err, 验证 error boundary 兜底不崩溃 |
| 31 | S31-async-timeout-5d-concurrent | 交互预期 | 5 域并发 update, 验证任意 > 5s 响应触发 warning + 协调一致 |
| 32 | S32-5d-cross-domain-saga-coordination | 协作预期 | 5 域 Lead 跨域 Saga 协调 (per D.2 T3.2 + 守门 #13 a L0 协调) |
| 33 | S33-mavis-proxy-real-person-signoff | 协作预期 | Mavis 临时代签 → 5 域 Lead 真人到位后追溯签字一致性 (per守门 #14 v2 + 守门 #1 禁回溯叙事) |
| 34 | S34-tmo-7node-orchestration-coordination | 协作预期 | TMO 7 节点 (M-N1..M-N7) 跨域编排 (per LangGraph 02 §2.6 + ADR-0046) |
| 35 | S35-l1-l1-prohibition-l0-coordination | 协作预期 | L1↔L1 禁止 (L0 协调, per 守门 #13 a 实证 TMO-03 4 类 cycle + O(V+E)) |

**每个 scenario 子目录**:
- `request.json` (业务场景输入)
- `expected_response.json` (预期响应 含 404 / 错误文案)
- `expected_db_state.json` (DB 状态变更)
- `expected_events.json` (事件流)
- `ac_mapping.md` (AC 跨引)

**守门 #13 W/T/M 分类** (per 9/1 18:30 JST 拍板):
- Master (SCD Type 2): S29 / S33 (long-term config)
- Transaction (append-only): S30 / S31 / S32 / S34 / S35 (event log)
- Work (短 TTL): S26 / S27 / S28 (404 negative result cache)

**脚本与 regression 实证**:
- 加 1 `_generate_uat_S26_S35.py` 脚本 (per守门 #19)
- 加 1 `regression/uat-s26-s35.sh` 脚本 (跑 10 场景, 5/5 pass)

### 1.3 件套 3: 2 docs 补充

**`docs/uat-design.md`** (增 §8):
- **§8 100% 覆盖断言** (per用户发令 9/7 16:15 JST):
  - §8.1 404 路径覆盖矩阵 (3 incidents + 4 mcp tools + 2 tsc err = 9 路径, 100% 覆盖)
  - §8.2 交互预期覆盖矩阵 (5 类 × N 场景)
  - §8.3 协作预期覆盖矩阵 (5 域 + 5 SA + TMO 7 节点 + 守门 #13 a, 100% 覆盖)
  - §8.4 跨 session 协调 (per 守门 #9 v2 + v3 实证)
  - §8.5 已知缺口 (per守门 #11 缺标比错标, 显式列)

**`docs/uat-runbook.md`** (增 §8):
- **§8 故障分类 + 排查手册**:
  - §8.1 404 错误分类 (3 类: NOT_IMPLEMENTED_404 / 4 mcp tools empty / 2 tsc err)
  - §8.2 交互预期违反分类 (5 类)
  - §8.3 协作预期违反分类 (5 类)
  - §8.4 跨 session 协调失败排查
  - §8.5 100% 覆盖状态仪表板 (实时同步 v0.91 升版)

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-uat-100pct D:/Star/.worktrees/impl/wt-opt-uat-100pct main
cd D:/Star/.worktrees/impl/wt-opt-uat-100pct
```

- **Worktree 路径**: `D:/Star/.worktrees/impl/wt-opt-uat-100pct`
- **Branch**: `feat/opt-uat-100pct` (基于 main HEAD `222d5b0`)

---

## 3. 关键约束

- **禁回溯叙事** (per守门 #12)
- **禁引用 RGS 仓** (per AGENTS §5)
- **PowerShell only** (per守门 #6)
- **守门 #5**: 禁打印 env secret
- **守门 #10**: commit author = `Ulysses <ulysses@mavis.local>`
- **守门 #13 W/T/M**: 10 新 fixture 跨域按 9/1 18:30 JST 拍板分类
- **守门 #14 v2**: 5 域 Lead CONTENT 4 维 (Mavis 临时代签, 真人 review 后追溯)
- **守门 #19**: 优先 Python 化
- **守门 #20**: 拆 commit 派生规 (Playwright 1+1 / mock 1+1 / docs 1 = 3 commit, 不沿用 v0.x 旧叙事)
- **守门 #11**: 100% 覆盖 → 0 容忍失败 (per用户发令)

---

## 4. 工作流程

```powershell
# Step 1: 创建 worktree
cd D:\Star
git worktree add -b feat/opt-uat-100pct D:/Star/.worktrees/impl/wt-opt-uat-100pct main
cd D:/Star/.worktrees/impl/wt-opt-uat-100pct

# Step 2: 件套 1 - Playwright 4 spec
# - frontend/e2e/error-404-path-coverage.spec.ts (200-300 行)
# - frontend/e2e/interaction-expectation-coverage.spec.ts (200-300 行)
# - frontend/e2e/collaboration-5d-lead-coordination.spec.ts (200-300 行)
# - frontend/e2e/uat-100pct-coverage-assertion.spec.ts (200-300 行)
# - 复用现有 6 spec 模式 + 引用 frontend/src/mocks/handlers/ 25 handler + tools/star-flash-mock/mock_data/ 17 + 125 fixture

# Step 3: 件套 2 - UAT 10 新 fixture
# - tools/star-flash-mock/mock_data/uat/scenarios/S26..S35/ 10 子目录 × 5 文件 = 50 fixture
# - tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_S26_S35.py
# - tools/star-flash-mock/mock_data/uat/regression/uat-s26-s35.sh

# Step 4: 件套 3 - 2 docs 补充
# - docs/uat-design.md 增 §8 (200-300 行)
# - docs/uat-runbook.md 增 §8 (100-200 行)

# Step 5: 守门实证
cargo check --workspace --all-targets -j 4     # 必须 0 err
cargo fmt --all
cd frontend && pnpm exec tsc --noEmit             # 必须 0 错 (per守门 #11 100% 覆盖)
bash tools/star-flash-mock/mock_data/uat/regression/uat-s26-s35.sh  # 5/5 pass

# Step 6: Commit (3 commit, per守门 #20)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add -A
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "..."
```

## 5. Commit 形式 (3 commit, 1 per 件套)

### Commit 1 - 件套 1 Playwright 4 spec
```
feat(e2e): 4 UAT spec 100% 覆盖 404 / 交互 / 协作 (per 9/7 16:15 JST)

4 spec 覆盖 3 类失败/异常 (per守门 #11 100% 覆盖硬约束):
- error-404-path-coverage: 3 incidents + 4 mcp tools + 2 tsc err = 9 路径
- interaction-expectation-coverage: 5 类 (响应时间 / prop / race / MSW / boundary)
- collaboration-5d-lead-coordination: 5 域 + 5 SA + TMO 7 节点 + 守门 #13 a
- uat-100pct-coverage-assertion: meta assertion (跨 spec 全 pass + 0 容忍 + 30s timing)

Refs: docs/briefs/OPT-WORKER-13-100pct-coverage.md
Refs: docs/test-design.md v0.8 §27.3 ST
Per守门 #11 100% 覆盖 0 容忍
Per守门 #14 v2 Mavis 临时代签 5 域 Lead
Per守门 #20 拆 commit 派生规
Per守门 #10 author=Ulysses
```

### Commit 2 - 件套 2 UAT 10 fixture
```
feat(mock): UAT 10 新 fixture S26-S35 (per 9/7 16:15 JST 100% 覆盖)

10 业务场景 (S26-S35) 覆盖 3 类失败/异常 (per守门 #11 100%):
- S26-S28: 3 incidents NOT_IMPLEMENTED_404 (per REQ-OPS-003 §30.6)
- S29: 4 mcp tools empty result (per 9/5 报告 §3.7 pre-existing)
- S30: 2 tsc err render fallback (per Worker 12 实证)
- S31: 5 域 async timeout coordination
- S32: 5 域跨域 Saga (per D.2 T3.2)
- S33: Mavis 临时代签 → 真人追溯签字
- S34: TMO 7 节点跨域编排 (per LangGraph 02 + ADR-0046)
- S35: L1↔L1 禁止 L0 协调 (per 守门 #13 a)

50 fixture (10 × 5) + 1 生成脚本 + 1 regression
守门 #13 W/T/M: 3 Master + 5 Transaction + 2 Work

Refs: docs/briefs/OPT-WORKER-13-100pct-coverage.md
Refs: docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.3
Per守门 #13 W/T/M 100% 25 + 10 = 35 场景
Per守门 #19 Python 化
Per守门 #10 author=Ulysses
```

### Commit 3 - 件套 3 docs
```
docs(uat): §8 100% 覆盖 + 故障分类 (per 9/7 16:15 JST 用户发令)

- docs/uat-design.md 增 §8 100% 覆盖断言 (200-300 行):
  §8.1 404 路径覆盖矩阵 (9 路径)
  §8.2 交互预期覆盖矩阵 (5 类 × N 场景)
  §8.3 协作预期覆盖矩阵 (5 域 + 5 SA + TMO 7 + 守门 #13 a)
  §8.4 跨 session 协调
  §8.5 已知缺口
- docs/uat-runbook.md 增 §8 故障分类 (100-200 行):
  §8.1 404 错误分类
  §8.2 交互预期违反分类
  §8.3 协作预期违反分类
  §8.4 跨 session 协调失败排查
  §8.5 100% 覆盖状态仪表板 (v0.91 升版实时同步)

Refs: docs/briefs/OPT-WORKER-13-100pct-coverage.md
Refs: docs/uat-design.md + docs/uat-runbook.md v0.91
Refs: docs/test-design.md v0.8
Per守门 #12 commit-time docs 同步 (新事件触发 = Commit 1+2 代码改动)
Per守门 #10 author=Ulysses
```

## 6. 实施注意

- **件套 1 Playwright**: 4 spec 覆盖 3 类失败, 复用现有 6 spec 模式 (vitest + Playwright + MSW)
- **件套 2 UAT mock**: 10 业务场景 S26-S35 跨域 + negative testing, 5 域 / TMO / Streamable / MCP 跨域协调
- **件套 3 docs**: 2 docs 增 §8, 触发由 Commit 1+2 代码改动
- **100% 覆盖** 含义: 9 404 路径 + 5 交互类 + 5 协作类 = 19 测试断言全过 = 0 失败 (per守门 #11)
- **跨 session 协调** 实证: per 守门 #9 v2 + v3 实证, sub-agent 跨 session 通信

## 7. 失败处理

- cargo check 不通过: 修复 + 重试, max 2 次
- pnpm tsc --noEmit 不通过: 修复 + 重试, max 2 次
- uat-s26-s35.sh regression 不 5/5 pass: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

## 8. 交付物 (per brief §6)

返回 final message 报告:
1. ✅/❌ worktree 创建
2. ✅/❌ 件套 1 Playwright 4 spec (file:line)
3. ✅/❌ 件套 2 UAT 10 fixture (file:line + 子目录)
4. ✅/❌ 件套 3 docs 2 docs 增 §8 (file:line)
5. ✅/❌ cargo check 0 err 实证
6. ✅/❌ pnpm tsc --noEmit 0 错 实证
7. ✅/❌ uat-s26-s35.sh 5/5 pass 实证
8. **3 commit hash 7 字符**
9. 任何超时 / 异常

## 9. 时间预算

≤ 2000K token (1 worker 跨 3 件套 + 2 docs, 估 1-2M)

## 10. 完成后

- 不要 merge / push
- 直接返回 final message 报告
