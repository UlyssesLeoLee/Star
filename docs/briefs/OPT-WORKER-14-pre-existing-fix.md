# Brief: OPT-WORKER-14 — pre-existing 缺口改善 (2 tsc + 4 mcp)

**Agent**: worker
**Phase**: OPT-P4-FIX
**Created**: 2026-09-07 17:43 JST (per 9/7 17:30 JST 用户发令"缺口改善好", Mavis 临时代签)
**Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)
**Status**: 🟡 Mavis 临时代签, 0 依赖, 立即启动

---

## 1. 任务目标 (3 commit, 1 worker 1 worktree)

修复 6 pre-existing 缺口 (2 tsc + 4 mcp tools failed), 估 0.5-0.8M token:

### 1.1 Commit 1: 2 tsc err fix (per守门 #1 v19 实证)

**文件 1**: `frontend/src/app/agent-view/page.tsx:398`
- **错误**: `Property 'name' does not exist on type 'AgentSession'`
- **原因**: `agent` 是 AgentSession type, 但 `.name` 字段不存在
- **修法**: 在 AgentSession type 加 `name: string` 字段 (per 5 域 Lead Mavis 临时代签拍板)
- **验证**: 跑 `cd frontend && pnpm exec tsc --noEmit` 0 错

**文件 2**: `frontend/src/lib/store.ts:562`
- **错误**: `Property 'tenantId' does not exist on type 'StoreState'`
- **原因**: StoreState interface 没有 `tenantId` 字段
- **修法**: 在 StoreState interface 加 `tenantId?: string` 字段
- **验证**: 跑 `cd frontend && pnpm exec tsc --noEmit` 0 错

### 1.2 Commit 2: 4 mcp tools failed fix (per 9/5 报告 §3.7 + D.3 已知缺口 #1)

**4 files**: `crates/star-mcp/src/tools/{find_references, get_code_context, get_symbol, search_code}.rs`
- **错误**: `tests::invoke_service_roundtrip_real_data` 期望 invoke 失败 (nil tenant) 但实际成功 (返空 results)
- **原因**: 4 tool 的 invoke 函数用 `ActorContext::default()` (nil tenant), 但 service 不 reject nil tenant (只返回空 results)
- **修法**: 4 tool invoke 入口加 nil-actor 检查, 若 `actor.tenant_id.is_nil()` 则返 `ToolError::InvalidContext { feature: "actor_session", suggestion: "use ActorContext::nil_actor_with_tenant" }` (per守门 #5 跟 domain-service 行为一致)
- **测试**: 4 tool 各加 1 unit test 验证 nil-actor 被拒绝 (`test_*_rejects_nil_actor`)

### 1.3 Commit 3 (可选): docs 增 known-issues 章节

**`docs/reports/PHASE-P4-PRE-EXISTING-FIX-REPORT.md`** (新建, per守门 #3 报告 7 段结构):
- §0 目的
- §1 改动矩阵 (2 tsc + 4 mcp + 1 report = 7 files)
- §2 验证摘要 (cargo check 0 err + tsc 0 错 + 4 mcp tests 5/5 pass)
- §3 已知缺口 (per守门 #11)
- §4 子代理失败接手 (无)
- §5 守门规则
- §6 签字栏 (5 角色, Mavis 临时代签)
- §7 修订历史

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-pre-existing-fix D:/Star/.worktrees/impl/wt-opt-pre-existing-fix main
cd D:/Star/.worktrees/impl/wt-opt-pre-existing-fix
```

- **Worktree 路径**: `D:/Star/.worktrees/impl/wt-opt-pre-existing-fix`
- **Branch**: `feat/opt-pre-existing-fix` (基于 main HEAD `7344ccb`)

---

## 3. 关键约束

- **禁回溯叙事** (per守门 #12)
- **禁引用 RGS 仓** (per AGENTS §5)
- **PowerShell only** (per守门 #6)
- **守门 #5**: 禁打印 env secret
- **守门 #10**: commit author = `Ulysses <ulysses@mavis.local>`
- **守门 #11**: 0 容忍失败
- **守门 #20**: 拆 commit 派生规 (1 per file group)

---

## 4. 工作流程

```powershell
# Step 1: 创建 worktree
cd D:\Star
git worktree add -b feat/opt-pre-existing-fix D:/Star/.worktrees/impl/wt-opt-pre-existing-fix main
cd D:/Star/.worktrees/impl/wt-opt-pre-existing-fix

# Step 2: Commit 1 - 2 tsc err fix
# - frontend/src/app/agent-view/page.tsx:398 修 (加 name 字段)
# - frontend/src/lib/store.ts:562 修 (加 tenantId 字段)
# - 跑 pnpm exec tsc --noEmit 实证 0 错
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add frontend/src/app/agent-view/page.tsx frontend/src/lib/store.ts
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "..."

# Step 3: Commit 2 - 4 mcp tools fix
# - 4 tool invoke 加 nil-actor 检查
# - 4 unit test 验证
# - 跑 cargo test -p star-mcp --tests 5/5 pass (4 pre-existing 不再 fail)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add crates/star-mcp/src/tools/{find_references,get_code_context,get_symbol,search_code}.rs
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "..."

# Step 4: Commit 3 - docs 增 (可选, 7 段)
# - docs/reports/PHASE-P4-PRE-EXISTING-FIX-REPORT.md 新建
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add docs/reports/PHASE-P4-PRE-EXISTING-FIX-REPORT.md
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "..."

# Step 5: 守门实证
cargo check --workspace --all-targets -j 4     # 0 err
cd frontend && pnpm exec tsc --noEmit             # 0 错
cargo test -p star-mcp --tests -j 4              # 4 tools 5/5 pass
```

## 5. Commit 形式 (3 commit, per守门 #20)

### Commit 1 - 2 tsc err
```
fix(frontend): 2 pre-existing tsc err (per 9/7 17:30 JST 用户发令)

- frontend/src/app/agent-view/page.tsx:398 {agent.name} → AgentSession 加 name 字段
- frontend/src/lib/store.ts:562 s.tenantId → StoreState 加 tenantId? 字段
- 守门 #1 v19 tsc --noEmit 0 错 实证

Refs: docs/briefs/OPT-WORKER-14-pre-existing-fix.md
Per守门 #11 0 容忍
Per守门 #10 author=Ulysses
```

### Commit 2 - 4 mcp tools
```
fix(mcp): 4 pre-existing tool failed (per 9/5 报告 §3.7)

- crates/star-mcp/src/tools/find_references.rs: 加 nil-actor 检查
- crates/star-mcp/src/tools/get_code_context.rs: 加 nil-actor 检查
- crates/star-mcp/src/tools/get_symbol.rs: 加 nil-actor 检查
- crates/star-mcp/src/tools/search_code.rs: 加 nil-actor 检查
- 4 unit test 验证 (test_*_rejects_nil_actor)
- 守门 #13 a 跟 domain-service 行为一致 (per ActorContext::nil_actor_with_tenant)
- 守门 #5 InvariantEvidence (actor 拒绝 nil tenant)

Refs: docs/briefs/OPT-WORKER-14-pre-existing-fix.md
Per守门 #11 0 容忍 (4 tools 5/5 pass)
Per守门 #13 a 跨域协调
Per守门 #10 author=Ulysses
```

### Commit 3 - docs (可选)
```
docs(reports): pre-existing 缺口 fix 报告 (per OPT-WORKER-14)

- docs/reports/PHASE-P4-PRE-EXISTING-FIX-REPORT.md v0.1 (7 段结构 per守门 #3)
- §0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口
- §4 子代理失败接手 / §5 守门规则 / §6 签字栏 / §7 修订历史

Refs: docs/briefs/OPT-WORKER-14-pre-existing-fix.md
Per守门 #12 commit-time docs 同步 (新事件 = Commit 1+2 代码改动)
Per守门 #10 author=Ulysses
```

## 6. 实施注意

### 6.1 tsc fix 细节

**page.tsx:398**:
- AgentSession type 加 `name: string` 字段
- 找 AgentSession 定义 (可能在 `frontend/src/lib/types.ts` 或 inline)
- 跟 5 域 Lead Mavis 临时代签拍板 (per守门 #14 v2)
- 不改 render 逻辑, 仅 type 字段

**store.ts:562**:
- StoreState interface 加 `tenantId?: string` 字段
- 找 interface 定义 (line 141 附近)
- default state 加 tenantId: "tenant-default" 或 undefined

### 6.2 mcp tool fix 细节

每个 tool invoke 函数 (4 files) 加:
```rust
if actor.tenant_id.is_nil() {
    return Err(ToolError::InvalidContext {
        feature: "actor_session",
        suggestion: "use ActorContext::nil_actor_with_tenant(tenant_id)",
    });
}
```

Test 模式:
```rust
#[test]
fn test_find_references_rejects_nil_actor() {
    let nil_actor = ActorContext::default();  // tenant_id is_nil
    let result = invoke_find_references(nil_actor, ...);
    assert!(matches!(result, Err(ToolError::InvalidContext { .. })));
}
```

## 7. 失败处理

- pnpm tsc --noEmit 不通过: 修复 + 重试, max 2 次
- cargo test -p star-mcp --tests 不 5/5 pass: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

## 8. 交付物 (per brief §6)

返回 final message 报告:
1. ✅/❌ worktree 创建
2. ✅/❌ 2 tsc err fix (file:line)
3. ✅/❌ 4 mcp tools fix (file:line + 4 unit test)
4. ✅/❌ docs 报告 (file:line)
5. ✅/❌ cargo check 0 err 实证
6. ✅/❌ pnpm tsc --noEmit 0 错 实证
7. ✅/❌ cargo test -p star-mcp --tests 5/5 pass 实证 (4 tools 全过)
8. **3 commit hash 7 字符**
9. 任何超时 / 异常

## 9. 时间预算

≤ 800K token (估 0.5-0.8M)

## 10. 完成后

- 不要 merge / push
- 直接返回 final message 报告
