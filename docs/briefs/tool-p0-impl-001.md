# P0 Tool 真实接入 Brief (per 守门 #20)

> **落档日期**: 2026-09-05 06:34 JST
> **拍板**: 用户发 "1" (1 号 = 3 P0 工具实装, G-DEP-01 拆决, per 9/5 04:03 JST 拍板推荐项直接执行)
> **守门 #20 实证**: 本文件是 wt-tool-p0-impl 子代理 dispatch 前的 brief 落档, 必读
> **父文档**: [docs/briefs/deps-survey.md §3.2 12 mock 工具 + §3.3 P0 优先级](../briefs/deps-survey.md) · [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1 §3 G-DEP-01](../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) · [AGENTS.md §7 #2 16 tool 真实接入](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md)

## 0. 全局约束 (子代理必读)

1. **工作目录**: 子代理必须在 `D:\Star\.worktrees\wt-tool-p0-impl` worktree 工作, **不修改** `D:\Star` 主仓 working tree
2. **branch 来源**: `wt-tool-p0-impl` 从 `origin/main @ 6608d87` 拉 (TMO 7 节点已实装, PR #13 5e5b1c2 + PR #14 6608d87)
3. **commit author**: `Ulysses <ulysses@mavis.local>` (per 守门 #10), 用 `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
4. **commit message 格式**: `tool-p0-impl: <description>` (per 守门 #1 跨 stage 必跑, 必含 cargo check 实证 + 守门编号)
5. **实装路径**: **改 .rs** (`crates/star-mcp/src/tools/*.rs`), 跟 TMO 7 节点 (Python) 不同. 16 tool 接入必须 .rs 改
6. **守门 #9 实证**: 子代理 status ≠ 实际成功, Mavis 父会话必 `git log -p --follow wt-tool-p0-impl` 实证 worktree commit 在 branch 上
7. **守门 #10**: Mavis 接手代签, 5 角色签字栏 (per 19:39 + 21:59 JST 授权)
8. **守门 #12**: 禁回溯叙事, BAS 引用 git 实证, 缺标比错标
9. **守门 #1 v1**: 必跑 `cargo check --workspace --lib -j 4` (0 err)
10. **守门 #1 v2**: 必跑 `cargo check --workspace --all-targets -j 4` (0 err, 含 tests)
11. **守门 #1 v3**: 必跑 `cargo fmt --all --check` + `cargo clippy --workspace --lib -- -D warnings` (0 err)
12. **守门 #1 v6**: 必跑 `cargo test -p star-mcp` (0 fail)
13. **守门 #1 v14**: 必跑 `cargo check --workspace --all-targets -j 4` 在 release mode (per 守门 #15 ahead origin buffer)
14. **守门 #5**: env var 安全, 不打印 secret
15. **守门 #6**: PowerShell only, 不走 bash 子命令 (注释允许元描述)
16. **守门 #7**: 0 unsafe code
17. **守门 #20**: 本 brief 是 dispatch 前置, 必读

## 1. 任务: 实装 3 P0 工具 (mock → real)

**目标**: 拆决 G-DEP-01 (per deps-survey §3.3), 让 TMO-01/04/06 触发链路真实可用. 3 工具当前仍 mock (per `crates/star-mcp/src/tools/{create_merge_request,create_worktree,search_issues}.rs`).

### 1.1 create_merge_request (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/create_merge_request.rs:32`):
```rust
let body = json!({
    "mr": { "id": "MR-mock-001", "title": title, "status": "OPEN",
            "source_branch": head, "target_branch": base,
            "url": "https://example.invalid/mr/MR-mock-001".to_string() }
});
Ok(mock_response("create_merge_request", body))
```

**实装目标** (调 `domain_scm` real service):
```rust
// 调 domain_scm::InMemoryScmService::create_mr
use domain_scm::{InMemoryScmService, SCMError, CreateMRInput, MergeRequest};

let input = CreateMRInput {
    title: title.clone(),
    description: args.get("description").and_then(|v| v.as_str()).map(String::from),
    base: base.clone(),
    head: head.clone(),
    agent_session_id: args.get("agent_session_id").and_then(|v| v.as_str()).map(String::from),
};
let svc = InMemoryScmService::new();
let mr: MergeRequest = svc.create_mr(input).await.map_err(McpError::from)?;
let body = json!({ "mr": mr });
Ok(real_response("create_merge_request", body))
```

**守门验证**:
- `cargo test -p domain-scm` 已有 MR 集成测试 (per `feature/dev-domain-scm` commit `03e321c`)
- `cargo test -p star-mcp --test test_create_merge_request_real` (新增 integration test, mock_scm_service 注入, 验证 real path 走通)
- 输入参数验证: title + base + head 必填 (per守门 #12 Pydantic 风格)

### 1.2 create_worktree (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/create_worktree.rs:35`):
```rust
let body = json!({
    "worktree": { "id": wt_id, "path": format!("/repos/owner/repo/{wt_id}"),
                 "branch": branch, "head_commit": "0..0", "dirty": true,
                 "agent_session_id": "agent-mock", "ide_session_id": "ide-mock",
                 "created_at": "2026-08-27T00:00:00Z" }
});
Ok(mock_response("create_worktree", body))
```

**实装目标** (调 `domain_worktree` real service):
```rust
// 调 domain_worktree::InMemoryWorktreeService::create
use domain_worktree::{InMemoryWorktreeService, WorktreeError, CreateWorktreeInput, Worktree};

let input = CreateWorktreeInput {
    issue_id: issue_id.clone(),
    branch_name: branch.clone(),
    agent_session_id: args.get("agent_session_id").and_then(|v| v.as_str()).map(String::from),
};
let svc = InMemoryWorktreeService::new();
let wt: Worktree = svc.create(input).await.map_err(McpError::from)?;
let body = json!({ "worktree": wt });
Ok(real_response("create_worktree", body))
```

**守门验证**:
- `cargo test -p domain-worktree` 已有 worktree 集成测试 (per `feature/dev-track-b-agent` commit `214d964` 17 状态机 23/23 tests)
- `cargo test -p star-mcp --test test_create_worktree_real` (新增)
- TMO-06 reassign_node 真实 worktree_migration (per 5e5b1c2 PR #13 G-DEP-01 拆决) 依赖本工具

### 1.3 search_issues (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/search_issues.rs:38`):
```rust
let body = json!({
    "query": query,
    "total": 2,
    "issues": [
        { "id": "ISSUE-1", "title": format!("Mock match for '{query}' #1"),
          "status": "OPEN", "labels": ["mock"] },
        { "id": "ISSUE-2", "title": format!("Mock match for '{query}' #2"),
          "status": "IN_PROGRESS", "labels": ["mock"] }
    ]
});
Ok(mock_response("search_issues", body))
```

**实装目标** (调 `domain_work_item` real service):
```rust
// 调 domain_work_item::InMemoryWorkItemService::list + filter
use domain_work_item::{InMemoryWorkItemService, WorkItemError, WorkItemFilter, WorkItemStatus};

let filter = WorkItemFilter {
    query: Some(query.clone()),
    status: args.get("status").and_then(|v| v.as_str())
        .and_then(|s| match s { "OPEN" => Some(WorkItemStatus::Open),
                                "IN_PROGRESS" => Some(WorkItemStatus::InProgress),
                                "DONE" => Some(WorkItemStatus::Done),
                                _ => None }),
    project_id: args.get("project_id").and_then(|v| v.as_str()).map(String::from),
    limit: args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize),
};
let svc = InMemoryWorkItemService::new();
let issues: Vec<WorkItem> = svc.list_with_filter(filter).await.map_err(McpError::from)?;
let body = json!({
    "query": query,
    "total": issues.len(),
    "issues": issues,
});
Ok(real_response("search_issues", body))
```

**守门验证**:
- `cargo test -p domain-work-item` 已有 work-item 集成测试 (per `feature/dev-domain-work-item` commit `c5d96b1`)
- `cargo test -p star-mcp --test test_search_issues_real` (新增)
- TMO-04 bulk_node 真实 select (per 5e5b1c2 PR #13) 依赖本工具

## 2. 实装清单 (跟 TMO 7 节点 namespace 隔离, 跨 worktree merge 不冲突)

1. `crates/star-mcp/src/tools/create_merge_request.rs` (改 mock → real)
2. `crates/star-mcp/src/tools/create_worktree.rs` (改 mock → real)
3. `crates/star-mcp/src/tools/search_issues.rs` (改 mock → real)
4. `crates/star-mcp/src/tools/mod.rs` (改 mock_response import 保留 + 加 real_response helper, **注意**: 11 其他 tool 仍用 mock_response, 别动)
5. `crates/star-mcp/src/error.rs` (扩展 McpError::from impl, 加 `From<SCMError>` + `From<WorktreeError>` + `From<WorkItemError>`)
6. `crates/star-mcp/tests/test_p0_tools_real.rs` (新文件, 3 工具 real path 集成测试)

**注意命名空间隔离** (per G-TMO-04-04 派生): 你在 wt-tool-p0-impl worktree, 合并时:
- 3 工具 .rs 跟 HEAD 不冲突 (mock → real 完整改写)
- mod.rs / error.rs 可能跟 HEAD 冲突 (其他 sub-session 可能改了), 父会话手工 resolve
- test 文件新加, 不冲突

## 3. 完成标准 (全部满足才报 succeeded)

- 3 工具 .rs 改 mock → real, 调 domain_scm/worktree/work_item 真实 service
- 守门 #1 v1+v2+v3+v6+v14 全部 0 err (per守门 #1 跨 stage 必跑)
- `cargo test -p star-mcp` 0 fail (含新 real path integration test)
- `cargo test --workspace` 0 fail (不能破坏其他 12 mock 工具 + 22 domain crate)
- 所有 commit author = `Ulysses <ulysses@mavis.local>`
- `git log -p --follow wt-tool-p0-impl` 实证 commit 在 branch 上 (Mavis 父会话会做这步)

## 4. 守门验证 (你必跑, 守门 #1 v1-v14 跨 stage 必跑)

- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo check --workspace --lib -j 4 2>&1 | tail -5` 显示 `Finished` + 0 err
- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo check --workspace --all-targets -j 4 2>&1 | tail -5` 显示 `Finished` + 0 err
- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo fmt --all --check` 0 差异
- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo clippy --workspace --lib -- -D warnings 2>&1 | tail -5` 显示 0 错
- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo test -p star-mcp 2>&1 | tail -5` 显示 0 fail + 新增 real test 全过
- `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo test --workspace --no-fail-fast 2>&1 | tail -5` 显示 0 fail (含 22 domain crate)
- **release mode**: `cd D:\Star\.worktrees\wt-tool-p0-impl && cargo check --workspace --all-targets --release -j 4 2>&1 | tail -5` 显示 0 err (守门 #1 v14 实证)

## 5. 禁止

- 不推 origin (per 守门 #1 R-05; Mavis 父会话统一推)
- 不开 OpenAI / Anthropic API (per 守门 #23)
- 不直接修改 `D:\Star` 主仓 (在 wt-tool-p0-impl worktree 工作)
- 不写 0 unsafe code (per 守门 #7)
- 不读 `docs/architecture/2026-09-03-langgraph/3 份文档` 全文 (你必读的是本 brief + 现有 `crates/star-mcp/src/tools/{create_merge_request,create_worktree,search_issues}.rs` 跟 `domain_scm/worktree/work_item` 现有 API)
- 不动其他 12 mock 工具 (你只改 3 P0)

## 6. 汇报格式 (子代理 final report 必含)

- **status**: `succeeded` / `partial` / `failed` (per 守门 #9 实证)
- **branch**: `wt-tool-p0-impl`
- **commit 短码**: 实际 git commit hash 7 字符
- **新增文件**: 列表 (相对路径)
- **修改文件**: 列表 (相对路径, 排除 untracked)
- **守门实证**: 守门 #1 v1-v14 全部 0 err 跑通证据 (per cargo check 输出)
- **已知缺口**: G-TOOL-P0-01..N (per 缺标比错标)
- **token 估**: 实际消耗

完成后, 你**只读** `docs/briefs/tool-p0-impl-001.md` (本文件) + 此 prompt + 现有 .rs 工具 stub. 直接开工.
