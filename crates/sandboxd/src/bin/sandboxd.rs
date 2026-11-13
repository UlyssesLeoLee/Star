/*
@cypher schema=1 source_sha256=53a3a9e815050661b2e4b123cd63778e623d0e3c744f18cab87c65b4aff8b7de
MERGE (self:File {path:"crates/sandboxd/src/bin/sandboxd.rs"})
@endcypher
*/
//! `sandboxd` binary 入口 (per FR-1.1, DD-SANDBOX-002 §1.1)
//!
//! 启动 daemon: 配置加载 → tracing → audit sink 组装 → orchestrator → gRPC + HTTP serve。
//!
//! 用法: `sandboxd [--config <path>]` (默认平台路径, `SANDBOXD_CONFIG` 环境变量优先)

use std::path::PathBuf;
use std::sync::Arc;

use sandboxd::audit::{AuditSink, CompositeAuditSink, JsonLinesAuditSink, PgAuditSink};
use sandboxd::backend::{BackendFactory, DefaultBackendFactory};
use sandboxd::config::SandboxdConfig;
use sandboxd::metrics::SandboxdMetrics;
use sandboxd::orchestrator::Orchestrator;

/// 平台默认配置路径 (per FR-1.1)
fn default_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| "C:/ProgramData".to_string()))
            .join("sandboxd")
            .join("config.yaml")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/sandboxd/config.yaml")
    }
}

/// 打印用法
fn print_usage() {
    println!(
        "sandboxd v{} — Sandbox-as-a-Service daemon (per SANDBOX-002)",
        sandboxd::VERSION
    );
    println!("用法: sandboxd [--config <path>]");
    println!(
        "  --config <path>  配置文件 (默认: {})",
        default_config_path().display()
    );
    println!("环境变量: SANDBOXD_CONFIG (优先于 --config), RUST_LOG (默认 info)");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("sandboxd {}", sandboxd::VERSION);
        return Ok(());
    }
    let config_path = match args.iter().position(|a| a == "--config") {
        Some(i) => args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| {
            eprintln!("--config 需要路径参数");
            print_usage();
            std::process::exit(2);
        }),
        None => std::env::var("SANDBOXD_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_config_path()),
    };

    sandboxd::logging::init("info");
    let config = SandboxdConfig::load_or_default(&config_path);
    tracing::info!(path = %config_path.display(), "config loaded");

    // audit sink 组装 (per FR-6.2 + FR-7.2 fail-closed)
    let mut sinks: Vec<Box<dyn AuditSink>> = Vec::new();
    if !config.observability.audit_log_file.is_empty() {
        let sink = JsonLinesAuditSink::new(&config.observability.audit_log_file)?;
        tracing::info!(file = %config.observability.audit_log_file, "audit jsonl sink ready");
        sinks.push(Box::new(sink));
    }
    if !config.observability.audit_log_pg_url.is_empty() {
        match PgAuditSink::connect(&config.observability.audit_log_pg_url).await {
            Ok(sink) => {
                tracing::info!("audit pg sink ready");
                sinks.push(Box::new(sink));
            }
            Err(e) => {
                if sinks.is_empty() {
                    // FR-7.2: 仅配置 PG 且连接失败 → 审计不可用 → fail-closed 拒绝启动
                    tracing::error!(error = %e, "audit pg sink failed and no file sink: refusing to start (fail-closed)");
                    return Err(anyhow::anyhow!("audit unavailable (fail-closed): {e}"));
                }
                tracing::warn!(error = %e, "audit pg sink failed, continuing with file sink only");
            }
        }
    }
    let audit: Arc<dyn AuditSink> = Arc::new(CompositeAuditSink::new(sinks));

    let metrics = Arc::new(SandboxdMetrics::new()?);
    let factory: Box<dyn BackendFactory> = Box::new(DefaultBackendFactory);
    let orchestrator = Arc::new(Orchestrator::new(
        config.server.max_concurrent_sessions,
        config.server.default_session_timeout_sec,
        audit,
        Arc::clone(&metrics),
        factory,
        config.actor.to_ids(),
    ));
    // FR-4.3: 30 min 默认 timeout 强制回收
    orchestrator.spawn_timeout_sweeper();

    sandboxd::server::serve(config, orchestrator, metrics).await?;
    Ok(())
}
