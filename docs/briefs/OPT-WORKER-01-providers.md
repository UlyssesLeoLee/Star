# Brief: OPT-WORKER-01 — 7 个 provider stub 实装 (per OPT-CODE-69..73)

**Agent**: worker
**Phase**: OPT-P4
**Created**: 2026-09-07 12:04 JST
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)

---

## 1. 任务目标

实装 7 个 stub 位置, 改进错误信息 + 加 unit test 覆盖 + #[deprecated] 标注 (per守门 #4 v18 派生规)。

**目标文件 (7 处)**:
1. `crates/star-sa/src/provider_github.rs:30` — `Err(ProviderError::NotFound("not implemented"))`
2. `crates/star-sa/src/provider_gitlab.rs:30` — 同上
3. `crates/star-sa/src/provider_bitbucket.rs:30` — 同上
4. `crates/star-sa/src/provider_gitea.rs:30` — 同上
5. `crates/domain-local-runtime/src/http_client.rs:355` — `"CLI spawn in RealHttpRuntime not implemented; use DefaultLocalRuntime::with_real_processes() in Phase 2"`
6. `crates/star-cache/src/in_memory_backend.rs:83` — `"incr not implemented for in-memory"`

**改进形式 (每个 stub 统一)**:
```rust
// 改进前
Err(ProviderError::NotFound("not implemented"))

// 改进后
#[deprecated(note = "provider 暂未实装, per P3-A 阶段 11/25 git 实证; 建议 P4-WBS Phase H.2 拍板启动 (per docs/reports/STAR-P4-OPT-WBS-001.md §3.2 #4)")]
Err(ProviderError::NotImplemented {
    feature: "provider_real_call",
    suggestion: "use mock provider in test, real implementation pending P4-WBS Phase H.2",
})
```

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-providers D:/Star/.worktrees/wt-opt-providers main
cd D:/Star/.worktrees/wt-opt-providers
```

- **Worktree 路径**: `D:/Star/.worktrees/wt-opt-providers`
- **Branch**: `feat/opt-providers` (基于 main HEAD `bfb0bca`)

---

## 3. 实装要求

### 3.1 ProviderError 变体扩展

在 `crates/star-sa/src/lib.rs` 加 `NotImplemented { feature: String, suggestion: String }` 变体 (per `thiserror` derive):

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("not implemented: feature={feature}, suggestion={suggestion}")]
    NotImplemented { feature: String, suggestion: String },
    
    #[error("not found: {0}")]
    NotFound(String),
    // ... 现有变体
}
```

如果已有 `NotImplemented` 变体, 直接用, 不重复定义。

### 3.2 4 个 provider_*.rs 改动

每个文件 line 30 附近, 把:
```rust
Err(ProviderError::NotFound("not implemented".into()))
```
改成:
```rust
#[deprecated(note = "...")]
Err(ProviderError::NotImplemented {
    feature: "github_api_call".into(),  // 各自不同
    suggestion: "use mock in test, real impl pending P4 Phase H.2".into(),
})
```

### 3.3 http_client.rs 改动

`crates/domain-local-runtime/src/http_client.rs:355`:
```rust
// 改进前
return Err("CLI spawn in RealHttpRuntime not implemented; ...".into());

// 改进后
#[deprecated(note = "...")]
return Err(format!(
    "feature=cli_spawn_in_real_runtime, suggestion=use DefaultLocalRuntime::with_real_processes() in Phase 2, current_phase=P3, p4_phase=H.2"
));
```

### 3.4 in_memory_backend.rs 改动

`crates/star-cache/src/in_memory_backend.rs:83`:
```rust
// 改进前
return Err(NotImplemented("incr not implemented for in-memory".into()));

// 改进后
#[deprecated(note = "...")]
return Err(NotImplemented {
    feature: "in_memory_incr".into(),
    suggestion: "use redis backend for incr support, current_phase=P3, p4_phase=G.4".into(),
});
```

### 3.5 Unit test 覆盖

每个 provider 加至少 1 个 test:
- `crates/star-sa/tests/provider_stub_test.rs` (新建) — 4 个 provider 测 NotImplemented 错误
- `crates/domain-local-runtime/tests/http_client_stub_test.rs` (新建, 或加到现有 test 文件)
- `crates/star-cache/tests/in_memory_stub_test.rs` (新建, 或加到现有 test 文件)

Test 模式:
```rust
#[test]
fn test_provider_github_returns_not_implemented() {
    let provider = GitHubProvider::new();
    let result = provider.some_method();
    assert!(matches!(result, Err(ProviderError::NotImplemented { .. })));
}
```

---

## 4. 守门硬约束 (per AGENTS §4 + §4.1 + §4.2)

1. **`cargo check --workspace --all-targets -j 4` 必须 0 err** (per守门 #1 v19)
2. **`cargo fmt --all -- --check` 必须 0 diff** (per守门 #6 enforced)
3. **`cargo clippy --workspace --all-targets -j 4` 必须 0 err** (per守门 #7 v3 advisory, 尽力)
4. **`cargo test -p star-sa --lib -j 4` + `cargo test -p domain-local-runtime --lib -j 4 -- --skip e2e_integration` + `cargo test -p star-cache --lib -j 4` 全 pass**
5. **禁回溯叙事** (per守门 #12)
6. **禁引用 RGS 仓** (per AGENTS §5 硬约束)
7. **禁打印 env secret** (per守门 #5)
8. **PowerShell only** (per守门 #6)

---

## 5. Commit 形式

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' add -A
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(stub): improve 7 provider/http_client/in_memory_backend error info + tests

- 4 star-sa provider (GitHub/GitLab/Bitbucket/Gitea) NotFound→NotImplemented
- 1 domain-local-runtime http_client CLI spawn 改进错误
- 1 star-cache in_memory_backend incr 改进错误
- 加 #[deprecated(note)] 标注 (per守门 #4 v18 派生规)
- 加 unit test 覆盖 (3 new test files)

Refs: docs/briefs/OPT-WORKER-01-providers.md
Refs: docs/reports/STAR-P4-OPT-WBS-001.md §3.2 #4 (OPT-CODE-69..73)
Per守门 #10 author=Ulysses
Per守门 #12 commit-time docs 同步 (新事件 = 代码改动)
"
```

---

## 6. 交付物

返回时报告:
1. worktree 路径: `D:/Star/.worktrees/wt-opt-providers`
2. branch 名称: `feat/opt-providers`
3. **commit hash 7 字符**
4. **修改文件列表** (file:line)
5. **`cargo check --workspace --all-targets -j 4` 输出 (必须 0 err)**
6. **`cargo test` 3 个 crate 输出 (必须全 pass)**
7. **任何 0 重试 > 2 次的失败 (per PowerShell fail-fast)**

---

## 7. 失败处理

- 如果 `cargo check` 不通过: 修复后再 check, **最多 2 次重试**, 仍失败则报告具体错误, 不 commit
- 如果 `cargo test` 不通过: 同上
- 0 文件超时 (per brief §7)

---

## 8. 工作目录

`D:/Star/.worktrees/wt-opt-providers` (创建后切到这个目录)

## 9. 时间预算

≤ 300K token (单 sub-session worker 上限)
