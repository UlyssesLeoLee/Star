/*
@cypher schema=1 source_sha256=4adaebfa4f2c872e4291bd003d638ea4cf2b4a5db03c2a97c4f0eeebae914fd1
MERGE (self:File {path:"crates/sandboxd/src/logging.rs"})
@endcypher
*/
//! `sandboxd` logging 模块 (per BD-002 §2.1 logging)
//!
//! tracing 初始化: EnvFilter 默认 info (RUST_LOG 可覆盖), 含时间戳 + 模块路径。

/// 初始化 tracing (多次调用安全, per BD §2.1)
///
/// - `RUST_LOG` 环境变量优先
/// - 默认 `info`
pub fn init(level: &str) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init();
}
