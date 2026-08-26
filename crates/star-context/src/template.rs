//! AGENTS.md bootstrap 模板(静态资源)
//!
//! 模板内容 per `docs/architecture/2026-08-26-upgrade/spec/acceptance/09-agent-instructions-spec.md` §1
//! 上限 50 行(per §2),硬约束。

#![warn(missing_docs)]

/// AGENTS.md bootstrap 模板(per `spec/acceptance/09-agent-instructions-spec.md` §1)
///
/// 静态嵌入,无运行时模板引擎。Phase D 骨架不引入 handlebars / tera 等依赖。
pub const BOOTSTRAP_TEMPLATE: &str = r#"# This repository is managed by STAR.

Discover available capabilities:
    star agent capabilities

Retrieve your current task:
    star task current --json

Retrieve relevant context:
    star context current --json

Search code:
    star code search "your query" --json

Before submitting:
    star test affected

Submit:
    star submit
"#;
