# P2 Tool 真实接入 Brief (per 守门 #20)

> **落档日期**: 2026-09-05 07:56 JST
> **拍板**: 用户新目标 "P2 工具实装" (per archon_internal_context 2026-09-05 07:56)
> **守门 #20 实证**: 本文件是 wt-tool-p2-impl 子代理 dispatch 前的 brief 落档, 必读
> **父文档**: [docs/briefs/deps-survey.md §3.2 12 mock 工具 + §3.3 P2 优先级](../briefs/deps-survey.md) · [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.2 §1.2 16 tool 表 + §3.2 G-DEP-07](../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) · [tool-p0-impl-001.md](tool-p0-impl-001.md) (1 号 brief 复用模式) · [tool-p1-impl-001.md](tool-p1-impl-001.md) (2 号 brief 复用模式)

## 0. 全局约束 (子代理必读)

1. **工作目录**: 子代理必须在 `D:\Star\.worktrees\wt-tool-p2-impl` worktree 工作, **不修改** `D:\Star` 主仓 working tree
2. **branch 来源**: `wt-tool-p2-impl` 从 `origin/main @ eabdff3` 拉 (1 号 P0 + 2 号 P1 工具已实装 + 7 号 PHASE v0.2 升版落地)
3. **commit author**: `Ulysses <ulysses@mavis.local>` (per 守门 #10), 用 `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
4. **commit message 格式**: `tool-p2-impl: <description>` (含 cargo check 实证 + 守门编号)
5. **实装路径**: 改 .rs (`crates/star-mcp/src/tools/*.rs`), 跟 1 号 P0 + 2 号 P1 工具同源
6. **守门 #9 实证**: 子代理 status ≠ 实际成功, Mavis 父会话必 `git log -p --follow wt-tool-p2-impl` 实证
7. **守门 #10**: Mavis 接手代签
8. **守门 #12 严守**: **禁回溯叙事, BAS 引用 git 实证, 缺标比错标, 0 误删无关文件** (1 号子代理曾违规误删 -270 行, 父会话 fix; 2 号子代理 cargo fmt 自动 wrap 长 assert 字符串 create_worktree.rs + search_issues.rs 各 5+/-1, 接受)
9. **守门 #1 v1**: 必跑 `cargo check --workspace --lib -j 4` (0 err)
10. **守门 #1 v2**: 必跑 `cargo check --workspace --all-targets -j 4` (0 err, 含 tests)
11. **守门 #1 v3**: 必跑 `cargo fmt --all --check` + `cargo clippy --workspace --lib -- -D warnings` (0 err, clippy 260+ pre-existing 跳过 per 1 号 G-TOOL-P0-03)
12. **守门 #1 v6**: 必跑 `cargo test -p star-mcp` + `cargo test -p domain-validation` (0 fail, 19 + 4 = 23 pre-existing fail 跨 session 续 per 1/2 号 G-TOOL-P0-04)
13. **守门 #1 v14**: 必跑 `cargo check --workspace --all-targets --release -j 4` (0 err)
14. **守门 #5/#6/#7**: env var 安全, PowerShell only, 0 unsafe
15. **守门 #20**: 本 brief 是 dispatch 前置, 必读

## 1. 任务: 实装 5 P2 工具 (mock → real)

**目标**: 拆决 G-DEP-07 (per deps-survey §3.3), 16 tool 全部 REAL 化. 5 工具当前仍 mock (per `crates/star-mcp/src/tools/{get_context,get_pipeline_status,request_review,run_validation,submit}.rs`).

### 1.1 get_context (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/get_context.rs:25`):
```rust
let body = json!({
    "context": {
        "issue_id": issue_id,
        "linked_files": ["docs/architecture/.../05-universal-submit.md", ...],
        "linked_specs": ["arch/03-star-ai-compat-arch.md", ...]
    }
});
Ok(mock_response("get_context", body))
```

**实装目标** (调 `star_context` + multiple domain service):
```rust
use star_context::ContextService;
use domain_work_item::InMemoryWorkItemService;
use domain_search::InMemorySearchService;

let ctx_svc = ContextService::new();
let wi_svc = InMemoryWorkItemService::new();
let search_svc = InMemorySearchService::new();

// 1. 查 work_item 关联
let wi = wi_svc.get(actor, issue_id.parse()?).await.map_err(McpError::from)?;
// 2. 查相关 spec
let specs = search_svc.search(actor, SearchQuery { raw: issue_id, resource_types: vec![ResourceType::Spec], ..}).await.map_err(McpError::from)?;
// 3. 组装 context
let body = json!({
    "context": {
        "issue_id": issue_id,
        "work_item": wi,
        "linked_specs": specs,
    }
});
Ok(real_response("get_context", body))
```

**守门验证**: `cargo test -p star-context` 已有, `cargo test -p domain-work-item` 已有 (per 1aab37e P3-C + 5e5b1c2)

### 1.2 get_pipeline_status (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/get_pipeline_status.rs`): mock, 字段 `{worktree_id?}`

**实装目标** (调 `domain_kms::InMemoryKmsService` 跟 CI runner abstraction):
```rust
// CI runner 抽象: PipelineService (可能在 domain-kms 或新 domain-ci crate)
// 如果 CI runner 抽象已存在, 调 get_pipeline_status(worktree_id)
// 如果不存在, 子代理在 domain-kms 加 get_pipeline_status (P3-B D.2-D.6 GA runner 抽象, 守门 #12 commit message 写明)
```

**子代理自己判断** (P3-B D.2-D.6 GA runner 抽象可能没落地, 子代理可能需要扩展). commit message 写明扩展决策.

### 1.3 request_review (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/request_review.rs`): mock, 字段 `{worktree_id?, reviewer?}`

**实装目标** (调 `domain_development::InMemoryDevelopmentService` 或 `domain_scm` review):
- 子代理先看 `crates/domain-development/src/lib.rs` + `crates/domain-scm/src/lib.rs` 现有 review API
- 调真实 service, 不存在则扩展 (P3-B D.6 review 抽象, 守门 #12 commit message)

### 1.4 run_validation (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/run_validation.rs:22`):
```rust
let body = json!({
    "validation": { "passed": 0, "failed": 0, "skipped": 0, "failed_tests": [] }
});
Ok(mock_response("run_validation", body))
```

**实装目标** (调 `domain_validation::InMemoryValidationService`):
```rust
use domain_validation::{InMemoryValidationService, ValidationRunInput, ValidationKind};

let input = ValidationRunInput {
    worktree_id: args.get("worktree_id").and_then(|v| v.as_str()).map(String::from),
    kinds: vec![ValidationKind::Tests, ValidationKind::Lint, ValidationKind::Typecheck],
    actor: actor_context,
};
let svc = InMemoryValidationService::new();
let result = svc.run(input).await.map_err(McpError::from)?;
let body = json!({ "validation": result });
Ok(real_response("run_validation", body))
```

**守门验证**: `cargo test -p domain-validation` 已有 13/13 tests (per 37b4406 commit "Validation 7 类 + 5 状态")

### 1.5 submit (mock → real)

**当前状态** (per `crates/star-mcp/src/tools/submit.rs:25`):
```rust
let body = json!({
    "status": "OK",
    "commit_sha": "deadbeef...",
    "mr_id": "MR-mock-001",
    "pipeline_run_id": "PIPE-mock-001",
    "validation_passed": true,
    "policy_checked": true,
});
Ok(mock_response("submit", body))
```

**实装目标** (12 步 universal submit, per `flows/05-universal-submit.md`):
- 子代理先读 `docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md` 12 步
- 调 `domain_scm::InMemoryScmService::submit_changes` 或扩展新 method
- 返回 SubmitResult (status + commit_sha + mr_id + pipeline_run_id + validation_passed + policy_checked 6 字段)

**子代理自己判断** 12 步实装完整度 (mock 简化版 OK, 完整 12 步需多 service 协作).

## 2. 实装清单

1. `crates/star-mcp/src/tools/get_context.rs` (改 mock → real, 调 star_context + domain_work_item + domain_search)
2. `crates/star-mcp/src/tools/get_pipeline_status.rs` (改 mock → real, 调 domain_kms 或扩展)
3. `crates/star-mcp/src/tools/request_review.rs` (改 mock → real, 调 domain_development 或 domain_scm review)
4. `crates/star-mcp/src/tools/run_validation.rs` (改 mock → real, 调 domain_validation)
5. `crates/star-mcp/src/tools/submit.rs` (改 mock → real, 12 步 universal submit 调多 service)
6. `crates/star-mcp/src/error.rs` (扩展 `From<ValidationError>` + `From<KmsError>` 等 impl, 1 号已加 `From<SCMError/WorktreeError/WorkItemError>`, 2 号加 `From<SearchError>`)
7. `crates/star-mcp/Cargo.toml` (+若干行: 引入 `domain-validation` + `domain-kms` 等依赖)
8. **如果**需要新 method: `crates/domain-validation/src/lib.rs` (扩展) 或 `crates/domain-kms/src/lib.rs` (扩展) 或 `crates/domain-scm/src/lib.rs` (review 扩展)
9. `crates/star-mcp/src/tools/{get_context,get_pipeline_status,request_review,run_validation,submit}.rs` (内联 `#[cfg(test)] mod tests` 跟 1/2 号 search_issues / search_code 模式一致)

**注意命名空间隔离** (per G-TOOL-P0-01 派生): 你在 wt-tool-p2-impl worktree, 合并时:
- 5 工具 .rs 跟 HEAD 不冲突 (mock → real 完整改写)
- error.rs 跟 HEAD 可能冲突 (1 号已加 3 个 From, 2 号加 From<SearchError>, 这次加 2-3 个, 父会话手工 resolve)
- Cargo.toml 跟 HEAD 不冲突 (1/2 号已加 domain-scm + domain-search + domain-work-item + domain-worktree, 这次加 domain-validation + domain-kms 等)
- 注意: **不要再误删无关文件** (1/2 号教训)

## 3. 完成标准 (全部满足才报 succeeded)

- 5 P2 工具 .rs 改 mock → real, 调对应 domain service
- 守门 #1 v1+v2+v3+v6+v14 全部 0 err (per守门 #1 跨 stage 必跑, 19 + 4 = 23 pre-existing fail 可接受)
- `cargo test -p star-mcp` 0 fail (新 fail 跟 P2 改动无关)
- `cargo test -p domain-validation` 0 fail (新 method 测试通过)
- `cargo test --workspace` 0 fail (新 fail 跟 P2 改动无关)
- 所有 commit author = `Ulysses <ulysses@mavis.local>`
- `git log -p --follow wt-tool-p2-impl` 实证 commit 在 branch (Mavis 父会话会做这步)
- **守门 #12 严守**: 0 误删无关文件, 0 回溯叙事, commit message 含 git 短码

## 4. 守门验证 (你必跑, 守门 #1 v1-v14 跨 stage 必跑)

```bash
cd D:\Star\.worktrees\wt-tool-p2-impl
cargo check --workspace --lib -j 4 2>&1 | tail -5  # 守门 #1 v1
cargo check --workspace --all-targets -j 4 2>&1 | tail -5  # 守门 #1 v2
cargo fmt --all --check  # 守门 #1 v3
cargo test -p star-mcp 2>&1 | tail -5  # 守门 #1 v6
cargo test -p domain-validation 2>&1 | tail -5  # 守门 #1 v6 (run_validation 测试)
cargo test --workspace --no-fail-fast 2>&1 | tail -5  # 守门 #1 v6
cargo check --workspace --all-targets --release -j 4 2>&1 | tail -5  # 守门 #1 v14
```

(注: cargo clippy 跳过 per 1/2 号 G-TOOL-P0-03 派生, 260+ pre-existing err 跨 session 续)

每个命令输出末尾必须显示 `Finished` + 0 `error[EXXXX]` + 0 `warning` (除 pre-existing missing_docs). 任何 1 个失败 = 子代理 partial, 必 fix 后重跑全部 7 步.

## 5. 禁止 (per 1/2 号守门 #12 违规教训)

- **不误删任何无关文件** (1 号 -270 行, 2 号 cargo fmt 自动 wrap 1 号 P0 工具 assert 字符串, 3 号必须 0 误删)
- 不推 origin (per 守门 #1 R-05; Mavis 父会话统一推)
- 不开 OpenAI / Anthropic API (per 守门 #23)
- 不直接修改 `D:\Star` 主仓 (在 wt-tool-p2-impl worktree 工作)
- 不写 0 unsafe code (per 守门 #7)
- 不读 `docs/architecture/2026-09-03-langgraph/3 份文档` 全文
- 不动其他 11 REAL 工具 (你只改 5 P2)
- 不动已经 real 的 3 P0 工具 (1 号已实装) + 4 P1 工具 (2 号已实装)

## 6. 汇报格式 (子代理 final report 必含)

- **status**: `succeeded` / `partial` / `failed` (per 守门 #9 实证)
- **branch**: `wt-tool-p2-impl`
- **commit 短码**: 实际 git commit hash 7 字符
- **新增文件**: 列表 (相对路径)
- **修改文件**: 列表 (相对路径, 排除 untracked)
- **守门实证**: 守门 #1 v1-v14 全部 0 err 跑通证据 (per cargo check 输出 tail)
- **已知缺口**: G-TOOL-P2-01..N (per 缺标比错标, **特别声明 0 误删无关文件**)
- **token 估**: 实际消耗

完成后, 你**只读** `docs/briefs/tool-p2-impl-001.md` (本文件) + 此 prompt. 直接开工.
