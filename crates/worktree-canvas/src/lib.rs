//! `worktree-canvas` — AI Worktree Graph Canvas facade / aggregator crate.
//!
//! Per `docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md` §0.1 + §1.2 +
//! `docs/design/DD-WORKTREE-CANVAS-001.md` §1.1 + §24 推荐架构:
//! 14 个 Rust 模块实现 worktree-canvas 完整业务能力.
//!
//! ## 架构
//!
//! 实际实现为 **13 个 sibling crate** (`crates/graph-core/` .. `crates/event-bus/`),
//! 本 crate 作为 **facade / 聚合层** re-export 13 个子 crate 的公开 API,
//! 提供单 import 入口 (`worktree-canvas::*`) 而非 13 个独立 crate 路径.
//!
//! ## 修正背景 (per ULYS-57 自审整体审查 2026-09-19 v1.2)
//!
//! IMPL-PLAN §0.1 + SPEC §1.2 + DD §1.1 均声明 `crates/worktree-canvas/` 为
//! 14 crate 第一位; 实际工作树早期 commit (PR #52 #54) 落地 13 个 sibling crate
//! 后未建聚合 crate. v1.2 M-1 修正补建本 facade, 文档声明与目录布局对齐.
//!
//! ## 模块清单 (per DD §24 + IMPL-PLAN §1.2)
//!
//! | 子 crate | 模块职责 | DD 章节 | 不变量 |
//! |---|---|---|---|
//! | `graph-core` | GraphRepository Trait + Neo4j/Memory adapters + 11 Node + 13 Edge | DD §7/§8/§10 | INV-WC-04/05 |
//! | `git-adapter` | GitProvider Trait + libgit2 + CLI fallback | DD §10 | — |
//! | `git-observer` | fsnotify + 100ms 防抖 + 15 GitEvent | DD §11 | — |
//! | `worktree-service` | 7 状态机 + ahead/behind + dirty | DD §12 | INV-WC-01/02/03/08/09 |
//! | `risk-engine` | V1 Git + V2 Diff overlap + 11 Risk 类型 | DD §13 | INV-WC-03 |
//! | `health-engine` | 13 因素加权扣分制 0-100 | DD §14 | INV-WC-02 |
//! | `agent-bridge` | Multica adapter + 11 字段 AgentSession | DD §15 | INV-WC-10 |
//! | `relationship-engine` | 13 Edge ops + 跨树语义 | DD §13 | INV-WC-04 |
//! | `canvas-renderer` | CanvasRenderer Trait + React-Flow 11.x + LOD 3 + Viewport Virt | DD §11/§29/§30/§37 | INV-WC-05/06/07 |
//! | `layout-engine` | 3 Layout (dagre/d3-force/ELK) + visualDistance log(n+1)*30 | DD §16/§31/§36 | — |
//! | `query-engine` | DSL Parser + NL Translator + Cypher generation + cypher guard | DD §17/§28 | FR-SEARCH-001..006 |
//! | `action-engine` | 18 Action + 3 类别 + 二次确认 + Idempotency 24h + Audit SCD Type 2 + RBAC 5 角色 + Retry 3 档 | DD §18/§42/§43/§44 | — |
//! | `event-bus` | 15 Event + 4 消费者 + 100ms 防抖 + Redis Streams | DD §19/§20 | NFR-REL-002 |
//!
//! ## 守门
//!
//! - #1 v25 cargo test 单 crate 实证
//! - #3 docs 同步 (per IMPL-PLAN §0.1 v1.2 修正)
//! - #6 win+unix 路径 (per AGENTS.md §4)
//! - #14 v3 Mavis 接手代签 (5 域真人到位后切真人, per self-review m-7)
//! - #29 1 commit 多文件 (本 crate 创建与 docs 修正同 commit)

#![doc = "AI Worktree Graph Canvas 聚合入口. 见模块级 doc + DD/IMPL-PLAN."]

pub use action_engine;
pub use agent_bridge;
pub use canvas_renderer;
pub use event_bus;
pub use git_adapter;
pub use git_observer;
pub use graph_core;
pub use health_engine;
pub use layout_engine;
pub use query_engine;
pub use relationship_engine;
pub use risk_engine;
pub use worktree_service;

#[cfg(test)]
mod tests {
    /// Smoke test: 所有 13 个子 crate 都能通过 facade 访问.
    /// 不做深度测试, 仅验证 re-export 不破坏编译 + 13 个子 crate 都注册到 workspace.
    ///
    /// 守门 #1 v25: cargo test -p worktree-canvas --lib 实证.
    ///
    /// 测试设计: 通过 `pub use` 引入的 13 个子 crate 都在模块作用域里.
    /// 仅引用 crate 自身的 module path (不需要任何子 crate 暴露任何 API),
    /// 触发 Rust 解析每个 crate root. 13 个子 crate 任意一个不可达时本测试编译失败.
    #[test]
    fn facade_re_exports_resolve() {
        // 13 个子 crate 都通过 `pub use` 引入, 它们的名字 (`graph_core` .. `event_bus`)
        // 都是当前 crate 的有效标识符. 我们仅引用模块路径名 (`std::module_path!()`),
        // 不调用任何 API. 若任一子 crate 不可达, 这里的 `use` 不会失败
        // (因 `pub use` 已保证), 但 crate 内文档级别注释 + 编译期 linker 验证
        // 已经足够证明路径可达.
        //
        // 显式 use 让所有 13 个名字都进入作用域, 触发 rustc 的 unused 检查:
        // 若任一 crate 不存在, 此处 `use` 编译失败.
        #[allow(unused_imports)]
        use {
            crate::action_engine, crate::agent_bridge, crate::canvas_renderer, crate::event_bus,
            crate::git_adapter, crate::git_observer, crate::graph_core, crate::health_engine,
            crate::layout_engine, crate::query_engine, crate::relationship_engine,
            crate::risk_engine, crate::worktree_service,
        };
        // 通过 module_path!() 触发模块解析 (返回当前测试模块路径).
        // 13 个子 crate 已通过 `pub use` 在 crate root 暴露,
        // 这里仅占位证明测试可执行.
        assert!(!std::module_path!().is_empty());
    }
}
