//! `star-context` — STAR AGENTS.md bootstrap 生成器骨架(Phase D)
//!
//! 唯一对外 API: [`generate_bootstrap`]
//!
//! ## 设计原则
//!
//! - **不写文件** — 只返回 `String`,调用方决定写不写
//! - **bootstrap,不是 knowledge base** — 模板硬上限 50 行(per `spec/acceptance/09-agent-instructions-spec.md` §2)
//! - **薄模板** — 不塞企业知识,不依赖 LLM
//! - **静态资源** — 模板用 `include_str!` 嵌入,无运行时模板引擎
//!
//! 完整 Project / Task 级 context(per `spec/acceptance/09-agent-instructions-spec.md` §5)
//! 待 Phase D.1 增量补齐(当前 crate 不包含 Project / Task 上下文 API)。

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::path::Path;

use thiserror::Error;

mod template;

pub use template::BOOTSTRAP_TEMPLATE;

/// Context 生成错误(Phase D 骨架只暴露 IO / 不支持两类)
#[derive(Debug, Error)]
pub enum ContextError {
    /// 输入路径不可访问
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// 仓库不被 STAR 管理(Phase D 暂不实现检测,留 stub)
    #[error("not a STAR-managed repository: {0}")]
    NotStarManaged(String),
}

/// 生成 AGENTS.md bootstrap 内容
///
/// **不写文件** — 仅返回 `String`,调用方决定写不写。
///
/// ## 参数
///
/// - `repo_path`: 仓库根路径。Phase D 骨架只用作未来扩展 hook(例如检测 `.star/`
///   是否存在);当前实现不读 `repo_path` 任何文件。
///
/// ## 返回
///
/// - `Ok(String)`:bootstrap 内容(per `spec/acceptance/09-agent-instructions-spec.md` §1)
/// - `Err(ContextError::NotStarManaged)`:Phase D 永不返回(留 stub)
/// - `Err(ContextError::Io(_))`:Phase D 永不返回(留 stub)
pub fn generate_bootstrap(_repo_path: &Path) -> Result<String, ContextError> {
    // Phase D 骨架:静态模板,无运行时分支
    Ok(BOOTSTRAP_TEMPLATE.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_under_50_lines() {
        // 验证 bootstrap 模板不超 50 行(per spec §2)
        let line_count = BOOTSTRAP_TEMPLATE.lines().count();
        assert!(
            line_count <= 50,
            "AGENTS.md bootstrap 上限 50 行(per spec),实际 {line_count} 行"
        );
    }

    #[test]
    fn bootstrap_contains_core_commands() {
        // 验证模板包含 6 个核心 star 命令(per spec §1)
        let s = BOOTSTRAP_TEMPLATE;
        assert!(s.contains("star agent capabilities"));
        assert!(s.contains("star task current"));
        assert!(s.contains("star context current"));
        assert!(s.contains("star code search"));
        assert!(s.contains("star test affected"));
        assert!(s.contains("star submit"));
    }

    #[test]
    fn generate_bootstrap_does_not_write_files() {
        // 验证 generate_bootstrap 是纯函数(返回 String,不写盘)
        // Phase D 骨架不实现任何写文件路径,这里 smoke-test 一下
        let result = generate_bootstrap(Path::new("."));
        assert!(result.is_ok());
    }
}
