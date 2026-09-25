//! `aci-emit` CLI - 对标 Python `_lib_aci_emit.py` CLI 行为.
//!
//! 子命令:
//! - `emit`   构造一条 ACI assertion
//! - `schema` 打印 .aci.json schema 必填字段清单
//!
//! 退出码: 0 = OK, 2 = schema / emit 校验失败.
//!
//! 守门: #5 (no secret leak), #9 (subprocess 兼容 stdlib spawn).

use aci_emitter::{
    types::empty_scope, AciEmitError, AciEmitter, ExpectActual, ExpectValueType, Layer, ScopeMap,
    Severity, Status, ACI_VERSION,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "aci-emit",
    version,
    about = "ACI assertion emitter (per ULYS-191 §4.2.1 brief v0.1). 输出符合 .aci.json schema 的 LLM-可读断言 JSON."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// 打印 .aci.json schema 必填字段清单 (对标 Python `aci_summary.py schema`).
    Schema,

    /// 构造一条 ACI assertion (对标 Python `_lib_aci_emit.py emit`).
    Emit(EmitCmd),
}

#[derive(clap::Args, Debug)]
struct EmitCmd {
    /// 测试层 (ut/it/st/e2e).
    #[arg(long, value_enum)]
    layer: LayerArg,

    /// 唯一 ID, 格式 <module>:<id>.
    #[arg(long = "assertion-id")]
    assertion_id: String,

    /// Scope 维度 (可多个, k=v 形式).
    #[arg(long, num_args = 1.., value_name = "k=v")]
    scope: Vec<String>,

    /// expect.type (17 个 expect_value_types 之一).
    #[arg(long = "expect-type", value_enum)]
    expect_type: ExpectTypeArg,

    /// expect.value (类型由 --expect-type 决定).
    #[arg(long = "expect-value")]
    expect_value: String,

    /// expect.description (LLM 必读).
    #[arg(long = "expect-description")]
    expect_description: String,

    /// actual.type.
    #[arg(long = "actual-type", value_enum)]
    actual_type: ExpectTypeArg,

    /// actual.value.
    #[arg(long = "actual-value")]
    actual_value: String,

    /// actual.description.
    #[arg(long = "actual-description")]
    actual_description: String,

    /// 结果状态 (PASS/FAIL/WARN/SKIP).
    #[arg(long, value_enum)]
    status: StatusArg,

    /// 严重程度 (critical/high/medium/low/info).
    #[arg(long, value_enum)]
    severity: SeverityArg,

    /// 自然语言「为什么」(FAIL 时必填).
    #[arg(long)]
    reasoning: String,

    /// 自然语言「怎么修」(FAIL 时建议填).
    #[arg(long = "suggested-fix")]
    suggested_fix: Option<String>,

    /// 逗号分隔 tags (例: perf,kms,timeout).
    #[arg(long, value_delimiter = ',')]
    tags: Option<Vec<String>>,

    /// RFC3339 时间戳 (缺省: now UTC).
    #[arg(long = "captured-at")]
    captured_at: Option<String>,

    /// 输出文件路径 (JSON).
    #[arg(long)]
    output: PathBuf,
}

impl EmitCmd {
    fn status_str(&self) -> &'static str {
        match self.status {
            StatusArg::Pass => "PASS",
            StatusArg::Fail => "FAIL",
            StatusArg::Warn => "WARN",
            StatusArg::Skip => "SKIP",
        }
    }
    fn severity_str(&self) -> &'static str {
        match self.severity {
            SeverityArg::Critical => "critical",
            SeverityArg::High => "high",
            SeverityArg::Medium => "medium",
            SeverityArg::Low => "low",
            SeverityArg::Info => "info",
        }
    }
}

/// clap `ValueEnum` adapter: 4 层.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
enum LayerArg {
    Ut,
    It,
    St,
    #[clap(name = "e2e")]
    E2e,
}

impl From<LayerArg> for Layer {
    fn from(a: LayerArg) -> Self {
        match a {
            LayerArg::Ut => Layer::Ut,
            LayerArg::It => Layer::It,
            LayerArg::St => Layer::St,
            LayerArg::E2e => Layer::E2e,
        }
    }
}

/// clap `ValueEnum` adapter: 4 status.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum StatusArg {
    Pass,
    Fail,
    Warn,
    Skip,
}

impl From<StatusArg> for Status {
    fn from(a: StatusArg) -> Self {
        match a {
            StatusArg::Pass => Status::Pass,
            StatusArg::Fail => Status::Fail,
            StatusArg::Warn => Status::Warn,
            StatusArg::Skip => Status::Skip,
        }
    }
}

/// clap `ValueEnum` adapter: 5 severity.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
enum SeverityArg {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl From<SeverityArg> for Severity {
    fn from(a: SeverityArg) -> Self {
        match a {
            SeverityArg::Critical => Severity::Critical,
            SeverityArg::High => Severity::High,
            SeverityArg::Medium => Severity::Medium,
            SeverityArg::Low => Severity::Low,
            SeverityArg::Info => Severity::Info,
        }
    }
}

/// clap `ValueEnum` adapter: 17 expect_value_types.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "snake_case")]
enum ExpectTypeArg {
    ResponseWithinMs,
    ResponseStatus2xx,
    ResponseStatus4xx5xx,
    FieldEquals,
    FieldExists,
    FieldNotExists,
    FieldInRange,
    FieldGreaterThan,
    FieldLessThan,
    FieldMatchesRegex,
    ListLengthAtLeast,
    ListLengthEquals,
    WorkflowCompletes,
    EventEmitted,
    LogContains,
    MetricWithinThreshold,
    NoResourceLeak,
}

impl From<ExpectTypeArg> for ExpectValueType {
    fn from(a: ExpectTypeArg) -> Self {
        match a {
            ExpectTypeArg::ResponseWithinMs => ExpectValueType::ResponseWithinMs,
            ExpectTypeArg::ResponseStatus2xx => ExpectValueType::ResponseStatus2xx,
            ExpectTypeArg::ResponseStatus4xx5xx => ExpectValueType::ResponseStatus4xx5xx,
            ExpectTypeArg::FieldEquals => ExpectValueType::FieldEquals,
            ExpectTypeArg::FieldExists => ExpectValueType::FieldExists,
            ExpectTypeArg::FieldNotExists => ExpectValueType::FieldNotExists,
            ExpectTypeArg::FieldInRange => ExpectValueType::FieldInRange,
            ExpectTypeArg::FieldGreaterThan => ExpectValueType::FieldGreaterThan,
            ExpectTypeArg::FieldLessThan => ExpectValueType::FieldLessThan,
            ExpectTypeArg::FieldMatchesRegex => ExpectValueType::FieldMatchesRegex,
            ExpectTypeArg::ListLengthAtLeast => ExpectValueType::ListLengthAtLeast,
            ExpectTypeArg::ListLengthEquals => ExpectValueType::ListLengthEquals,
            ExpectTypeArg::WorkflowCompletes => ExpectValueType::WorkflowCompletes,
            ExpectTypeArg::EventEmitted => ExpectValueType::EventEmitted,
            ExpectTypeArg::LogContains => ExpectValueType::LogContains,
            ExpectTypeArg::MetricWithinThreshold => ExpectValueType::MetricWithinThreshold,
            ExpectTypeArg::NoResourceLeak => ExpectValueType::NoResourceLeak,
        }
    }
}

/// 解析 CLI `--scope k=v ...` → ScopeMap (跟 Python `_parse_kv_pairs` 1:1).
fn parse_scope_kv(items: Vec<String>) -> Result<ScopeMap, AciEmitError> {
    let mut out = empty_scope();
    for item in items {
        if !item.contains('=') {
            return Err(AciEmitError::InvalidScopeKv(item));
        }
        let (k, v) = item.split_once('=').expect("checked contains '='");
        if out.contains_key(k) {
            return Err(AciEmitError::DuplicateScopeKey(k.to_string()));
        }
        out.insert(k.to_string(), v.to_string());
    }
    Ok(out)
}

/// CLI 字符串值 → serde_json::Value (跟 Python `_coerce_value` 1:1).
fn coerce_value(raw: &str) -> serde_json::Value {
    aci_emitter::util::coerce_value(raw)
}

/// `schema` 子命令实现.
fn cmd_schema() -> ExitCode {
    let out = serde_json::json!({
        "aci_version": ACI_VERSION,
        "required_fields": [
            "assertion_id", "aci_version", "layer", "scope",
            "expect", "actual", "status", "severity",
            "reasoning", "captured_at",
        ],
        "valid_layers": Layer::ALL.to_vec(),
        "valid_statuses": Status::ALL.to_vec(),
        "valid_severities": Severity::ALL.to_vec(),
        "valid_expect_types": ExpectValueType::ALL.to_vec(),
        "valid_scope_dims": aci_emitter::validators::VALID_SCOPE_DIMS.to_vec(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).expect("json serialize")
    );
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_lines)]
fn cmd_emit(c: EmitCmd) -> ExitCode {
    let layer: Layer = c.layer.into();
    let status: Status = c.status.into();
    let severity: Severity = c.severity.into();
    let expect_type: ExpectValueType = c.expect_type.into();
    let actual_type: ExpectValueType = c.actual_type.into();

    let status_str = c.status_str();
    let severity_str = c.severity_str();
    let output = c.output.clone();

    let scope = match parse_scope_kv(c.scope) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("FAIL={e}");
            return ExitCode::from(2);
        }
    };

    let em = AciEmitter::new(layer);
    let result = em.build_with(
        &c.assertion_id,
        scope,
        ExpectActual::new(
            expect_type,
            coerce_value(&c.expect_value),
            &c.expect_description,
        ),
        ExpectActual::new(
            actual_type,
            coerce_value(&c.actual_value),
            &c.actual_description,
        ),
        status,
        severity,
        &c.reasoning,
        c.suggested_fix,
        c.tags,
        c.captured_at,
    );

    let assertion = match result {
        Ok(a) => a,
        Err(e) => {
            eprintln!("FAIL={e}");
            return ExitCode::from(2);
        }
    };

    match em.write(&assertion, &output) {
        Ok(()) => {
            println!(
                "OK status={status_str} severity={severity_str} output={}",
                output.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("FAIL={e}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Schema => cmd_schema(),
        Cmd::Emit(c) => cmd_emit(c),
    }
}
