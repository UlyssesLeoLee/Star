# P1 Tool 真实接入 Brief (per 守门 #20)

> **落档日期**: 2026-09-05 07:12 JST
> **拍板**: per 9/5 04:03 JST 拍板推荐项直接执行 (1 号完成, 2 号立即)
> **守门 #20 实证**: 本文件是 wt-tool-p1-impl 子代理 dispatch 前的 brief 落档, 必读
> **父文档**: [docs/briefs/deps-survey.md §3.2 12 mock 工具 + §3.3 P1 优先级](../briefs/deps-survey.md) · [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1 §3 G-DEP-02](../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) · [tool-p0-impl-001.md](tool-p0-impl-001.md) (1 号 brief 复用模式)

## 0. 全局约束 (子代理必读)

1. **工作目录**: 子代理必须在 `D:\Star\.worktrees\wt-tool-p1-impl` worktree 工作, **不修改** `D:\Star` 主仓 working tree
2. **branch 来源**: `wt-tool-p1-impl` 从 `origin/main @ 446a8e1` 拉 (1 号 P0 工具已实装 + merge + push)
3. **commit author**: `Ulysses <ulysses@mavis.local>` (per 守门 #10), 用 `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
4. **commit message 格式**: `tool-p1-impl: <description>` (含 cargo check 实证 + 守门编号)
5. **实装路径**: 改 .rs (`crates/star-mcp/src/tools/*.rs`), 跟 1 号 P0 工具同源
6. **守门 #9 实证**: 子代理 status ≠ 实际成功, Mavis 父会话必 `git log -p --follow wt-tool-p1-impl` 实证
7. **守门 #10**: Mavis 接手代签
8. **守门 #12**: **禁回溯叙事, BAS 引用 git 实证, 缺标比错标** (1 号子代理曾违规误删 -270 行, 2 号必须严守)
9. **守门 #1 v1**: 必跑 `cargo check --workspace --lib -j 4` (0 err)
10. **守门 #1 v2**: 必跑 `cargo check --workspace --all-targets -j 4` (0 err, 含 tests)
11. **守门 #1 v3**: 必跑 `cargo fmt --all --check` + `cargo clippy --workspace --lib -- -D warnings` (0 err)
12. **守门 #1 v6**: 必跑 `cargo test -p star-mcp` (0 fail, 19 pre-existing 失败不阻塞, 跟 1 号一致)
13. **守门 #1 v14**: 必跑 `cargo check --workspace --all-targets --release -j 4` (0 err)
14. **守门 #5/#6/#7**: env var 安全, PowerShell only, 0 unsafe
15. **守门 #20**: 本 brief 是 dispatch 前置, 必读

## 1. 任务: 实装 4 P1 工具 (mock → real)

**目标**: 拆决 G-DEP-02 (per deps-survey §3.3), 让 TMO-05 summarize_node + TMO-04 bulk_node 真实 context 汇总/批量 gate 可用. 4 工具当前仍 mock (per `crates/star-mcp/src/tools/{search_code,get_symbol,find_references,get_code_context}.rs`).

### 1.1 search_code (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/search_code.rs:25`):
```rust
let body = json!({
    "query": query,
    "total": 1,
    "results": [{ "file": "crates/star-cli/src/commands/agent.rs", "line": 1, "snippet": "..." }]
});
Ok(mock_response("search_code", body))
```

**实装目标** (调 `domain_search::InMemorySearchService::search`):
```rust
use domain_search::{InMemorySearchService, SearchQuery, SearchQueryPort, SearchError, ResourceType};

let query = SearchQuery {
    raw: query.clone(),
    project_id: args.get("project_id").and_then(|v| v.as_str()).map(String::from),
    resource_types: vec![ResourceType::Code],
    limit: args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize,
};
let svc = InMemorySearchService::new();
let results = svc.search(actor_context, query).await.map_err(McpError::from)?;
let body = json!({ "query": query, "total": results.len(), "results": results });
Ok(real_response("search_code", body))
```

**守门验证**: `cargo test -p domain-search` 已有 search unit test (per `1fdc9ae` 12/12 tests)

### 1.2 get_symbol (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/get_symbol.rs`): mock, 字段 `{file, symbol_name}`

**实装目标** (调 `domain_search::InMemorySearchService::get_index_by_resource` 或扩展新 method `get_symbol`):
- 子代理先看 lib.rs:632 SearchQueryPort 现有方法
- 如果没有 get_symbol 专用方法, 子代理在 `domain_search` 加 `async fn get_symbol(actor, file, symbol_name) -> Result<Vec<Symbol>, SearchError>` (守门 #12 子代理在 commit message 写明: 扩展 domain_search 新 API + 守门 #13 a 通过 L0 协调)
- 然后 star-mcp::tools::get_symbol 调 `svc.get_symbol(...)` 返回

### 1.3 find_references (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/find_references.rs`): mock, 字段 `{file, line, column}`

**实装目标**: 调 `domain_search::InMemorySearchService` 新 method `find_references(actor, file, line, column) -> Result<Vec<Reference>, SearchError>`
- 子代理在 domain_search 加新 method (跟 get_symbol 同源)
- 守门 #12 commit message 写明新 API

### 1.4 get_code_context (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/get_code_context.rs`): mock, 字段 `{file, line, radius}`

**实装目标**: 调 `domain_search::InMemorySearchService` 新 method `get_code_context(actor, file, line, radius) -> Result<CodeContext, SearchError>`
- 子代理在 domain_search 加新 method
- 返回 `{file, start_line, end_line, snippet}` 结构

## 2. 实装清单

1. `crates/star-mcp/src/tools/search_code.rs` (改 mock → real, 调 `domain_search::InMemorySearchService::search`)
2. `crates/star-mcp/src/tools/get_symbol.rs` (改 mock → real, 调新 method `domain_search::InMemorySearchService::get_symbol` 或扩展现有)
3. `crates/star-mcp/src/tools/find_references.rs` (改 mock → real, 调新 method)
4. `crates/star-mcp/src/tools/get_code_context.rs` (改 mock → real, 调新 method)
5. `crates/star-mcp/src/error.rs` (扩展 `From<SearchError>` impl, 1 号已加 `From<SCMError>` + `From<WorktreeError>` + `From<WorkItemError>`, 这次加 `From<SearchError>`)
6. `crates/star-mcp/Cargo.toml` (+1 行: 引入 `domain-search` 依赖, 1 号已引入 `domain-scm`)
7. **如果**需要新 method (get_symbol / find_references / get_code_context): `crates/domain-search/src/lib.rs` (+新 method 3 个, SearchQueryPort trait + InMemorySearchService impl + 守门 #13 a 实证 L0 协调)
8. `crates/star-mcp/src/tools/{get_symbol,find_references,get_code_context}.rs` (内联 `#[cfg(test)] mod tests` 跟 1 号 search_issues 模式一致)

**注意命名空间隔离** (per G-TOOL-P0-01 派生): 你在 wt-tool-p1-impl worktree, 合并时:
- 4 工具 .rs 跟 HEAD 不冲突 (mock → real 完整改写)
- error.rs 跟 HEAD 可能冲突 (1 号已加 From<SCMError> 等 3 个, 这次加 From<SearchError>, 父会话手工 resolve)
- Cargo.toml 跟 HEAD 不冲突 (1 号已加 domain-scm 依赖, 这次加 domain-search)
- domain-search 跟 HEAD 不冲突 (只有新加 method)
- 注意: **不要再误删无关文件** (1 号子代理违规 -270 行, 2 号必须严守守门 #12)

## 3. 完成标准 (全部满足才报 succeeded)

- 4 P1 工具 .rs 改 mock → real, 调 domain_search 真实 service
- 守门 #1 v1+v2+v3+v6+v14 全部 0 err (per守门 #1 跨 stage 必跑, 19 pre-existing 失败可接受)
- `cargo test -p star-mcp` 0 fail (新 fail 跟 P1 改动无关)
- `cargo test -p domain-search` 0 fail (新加 method 测试通过)
- `cargo test --workspace` 0 fail (新 fail 跟 P1 改动无关)
- 所有 commit author = `Ulysses <ulysses@mavis.local>`
- `git log -p --follow wt-tool-p1-impl` 实证 commit 在 branch (Mavis 父会话会做这步)
- **守门 #12 严守**: 0 误删无关文件, 0 回溯叙事, 0 无证据 commit (commit message 含 git 短码)

## 4. 守门验证 (你必跑, 守门 #1 v1-v14 跨 stage 必跑)

```bash
cd D:\Star\.worktrees\wt-tool-p1-impl
cargo check --workspace --lib -j 4 2>&1 | tail -5  # 守门 #1 v1
cargo check --workspace --all-targets -j 4 2>&1 | tail -5  # 守门 #1 v2
cargo fmt --all --check  # 守门 #1 v3
cargo clippy --workspace --lib -- -D warnings 2>&1 | tail -5  # 守门 #1 v3
cargo test -p star-mcp 2>&1 | tail -5  # 守门 #1 v6
cargo test -p domain-search 2>&1 | tail -5  # 守门 #1 v6 (新 method 测试)
cargo test --workspace --no-fail-fast 2>&1 | tail -5  # 守门 #1 v6
cargo check --workspace --all-targets --release -j 4 2>&1 | tail -5  # 守门 #1 v14
```

每个命令输出末尾必须显示 `Finished` + 0 `error[EXXXX]` + 0 `warning` (除 pre-existing missing_docs). 任何 1 个失败 = 子代理 partial, 必 fix 后重跑全部 8 步.

## 5. 禁止 (per 1 号守门 #12 违规教训)

- **不误删任何无关文件** (1 号子代理误删 -270 行, 父会话已 fix, 2 号必须 0 误删, 严守守门 #12)
- 不推 origin (per 守门 #1 R-05; Mavis 父会话统一推)
- 不开 OpenAI / Anthropic API (per 守门 #23)
- 不直接修改 `D:\Star` 主仓 (在 wt-tool-p1-impl worktree 工作)
- 不写 0 unsafe code (per 守门 #7)
- 不读 `docs/architecture/2026-09-03-langgraph/3 份文档` 全文
- 不动其他 12 mock 工具 (你只改 4 P1)
- 不动已经 real 的 3 P0 工具 (1 号已实装)

## 6. 汇报格式 (子代理 final report 必含)

- **status**: `succeeded` / `partial` / `failed` (per 守门 #9 实证)
- **branch**: `wt-tool-p1-impl`
- **commit 短码**: 实际 git commit hash 7 字符
- **新增文件**: 列表 (相对路径)
- **修改文件**: 列表 (相对路径, 排除 untracked)
- **守门实证**: 守门 #1 v1-v14 全部 0 err 跑通证据 (per cargo check 输出 tail)
- **已知缺口**: G-TOOL-P1-01..N (per 缺标比错标)
- **token 估**: 实际消耗

完成后, 你**只读** `docs/briefs/tool-p1-impl-001.md` (本文件) + 此 prompt. 直接开工.
