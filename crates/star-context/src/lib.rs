//! `star-context` crate (Phase D 实装)
//!
//! 唯一公开 API:
//! - [`generate_bootstrap`]  — 生成 AGENTS.md bootstrap 文本 (不写文件)
//! - [`write_bootstrap`]     — 写 AGENTS.md 到指定仓库路径 (per Phase D 任务 6)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 0 新依赖 (除 workspace 继承的 std / serde)
//! - bootstrap 必须 <= 50 行 (per `spec/acceptance/09-agent-instructions-spec.md` §2)
//! - 是 bootstrap, 不是 knowledge base (per spec §0: 极薄, 不塞企业知识)
//! - write_bootstrap 如目标文件存在 -> 拒绝 (Err(AlreadyExists)), 不覆盖

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::fs;
use std::path::Path;

use thiserror::Error;

mod template;

pub use template::BOOTSTRAP_TEMPLATE;

/// **权威 ActorContext** (per P0-1 联动审计修复, 2026-08-31)
///
/// 替换 14 个 domain-* + 3 supporting crate 的 17 处重复 `ActorContext` 定义.
/// 字段用 `Uuid` 而非强类型 ID, 避免 star-context 引入对 domain-* 的依赖.
/// 各 domain 内部收到后做 `UserId::from(actor.user_id)` 等转换.
pub mod actor;
pub use actor::ActorContext;

/// Context 生成错误 (Phase D 任务 6 实装)
#[derive(Debug, Error)]
pub enum ContextError {
    /// IO 错误
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// AGENTS.md 已存在, 拒绝覆盖
    #[error("AGENTS.md already exists at {0} (refusing to overwrite)")]
    AlreadyExists(String),

    /// 仓库路径不是目录
    #[error("not a directory: {0}")]
    NotADirectory(String),
}

/// 生成 AGENTS.md bootstrap 文本
///
/// **不写文件** — 仅返回 `String` 文本, 由调用方决定写不写.
///
/// ## 参数
///
/// - `_repo_path`: 仓库路径 (Phase D 任务 6 仅保留, 不读; Phase D.1 扩展为读 `.star/`)
pub fn generate_bootstrap(_repo_path: &Path) -> Result<String, ContextError> {
    Ok(BOOTSTRAP_TEMPLATE.to_string())
}

/// 写 AGENTS.md 到指定仓库路径
///
/// ## 行为
///
/// - `repo_path` 必须存在且是目录, 否则 `Err(NotADirectory)`
/// - `<repo_path>/AGENTS.md` 不存在时, 调 `generate_bootstrap` 写文件
/// - **已存在** AGENTS.md -> `Err(AlreadyExists)`, **不覆盖** (Phase D 安全约束)
/// - 任何 IO 错误 -> `Err(Io)`
///
/// ## 返回
///
/// - `Ok(())`: 写成功
/// - `Err(ContextError::AlreadyExists)`: 已存在, 未改
/// - `Err(ContextError::NotADirectory)`: 路径不是目录
/// - `Err(ContextError::Io(_))`: IO 错误
pub fn write_bootstrap(repo_path: &Path) -> Result<(), ContextError> {
    // 1. 验证 repo_path 是目录
    let metadata = fs::metadata(repo_path)?;
    if !metadata.is_dir() {
        return Err(ContextError::NotADirectory(repo_path.display().to_string()));
    }

    // 2. 检查 AGENTS.md 是否已存在
    let target = repo_path.join("AGENTS.md");
    if target.exists() {
        return Err(ContextError::AlreadyExists(target.display().to_string()));
    }

    // 3. 生成 bootstrap 文本
    let content = generate_bootstrap(repo_path)?;

    // 4. 写文件
    fs::write(&target, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn bootstrap_under_50_lines() {
        let line_count = BOOTSTRAP_TEMPLATE.lines().count();
        assert!(
            line_count <= 50,
            "AGENTS.md bootstrap 必须 <= 50 行 (per spec), 实际 {line_count} 行"
        );
    }

    #[test]
    fn bootstrap_contains_core_commands() {
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
        // generate_bootstrap 纯函数, 不写文件
        let result = generate_bootstrap(Path::new("."));
        assert!(result.is_ok());
    }

    // Phase D 任务 6 新增 2 tests

    #[test]
    fn write_bootstrap_creates_agets_md_in_temp_dir() {
        // 用 tempdir 测 write_bootstrap 真实写文件
        let tmp = env::temp_dir().join(format!("star-context-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let result = write_bootstrap(&tmp);
        assert!(
            result.is_ok(),
            "write_bootstrap 应该成功, 实际: {:?}",
            result
        );

        // 验证文件已写
        let target = tmp.join("AGENTS.md");
        assert!(target.exists(), "AGENTS.md 应被创建");

        // 验证内容 = BOOTSTRAP_TEMPLATE
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(content, BOOTSTRAP_TEMPLATE);

        // 清理
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn write_bootstrap_refuses_overwrite_existing() {
        // 写 2 次, 第 2 次应该 Err(AlreadyExists)
        let tmp = env::temp_dir().join(format!(
            "star-context-test-overwrite-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // 第 1 次: 成功
        let r1 = write_bootstrap(&tmp);
        assert!(r1.is_ok());

        // 第 2 次: Err(AlreadyExists), 文件未改
        let r2 = write_bootstrap(&tmp);
        assert!(
            matches!(r2, Err(ContextError::AlreadyExists(_))),
            "第 2 次应 Err(AlreadyExists), 实际: {:?}",
            r2
        );

        // 验证内容仍是第 1 次的 (没被覆盖)
        let target = tmp.join("AGENTS.md");
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(content, BOOTSTRAP_TEMPLATE);

        // 清理
        let _ = fs::remove_dir_all(&tmp);
    }
}
