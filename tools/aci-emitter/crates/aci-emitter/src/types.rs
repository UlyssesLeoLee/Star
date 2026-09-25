//! ACI emitter 核心类型 (per ULYS-191 §4.2.1, 对标 `tools/star-flash-mock/scripts/_lib_aci_emit.py`).
//!
//! 所有字段命名严格按 .aci.json schema 1:1 对应 (Python emitter 是规范).
//!
//! 设计要点:
//! - 纯 safe Rust (no `unsafe`, 守门 #7)
//! - stdlib-only 外部 deps (serde + serde_json + chrono + thiserror + uuid, 守门 #24)
//! - `serde::Serialize` 全字段, 与 Python `_lib_aci_emit.py` 序列化输出兼容
//!
//! Serde 命名规范:
//! - `Layer`: lowercase (`ut`/`it`/`st`/`e2e`)
//! - `Status`: SCREAMING_SNAKE_CASE (`PASS`/`FAIL`/`WARN`/`SKIP`)
//! - `Severity`: lowercase (`critical`/`high`/`medium`/`low`/`info`)
//! - `ExpectValueType`: 17 种 snake_case (含 `response_status_2xx` / `response_status_4xx_5xx` 等数字混合)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// ACI schema 版本 (与 Python emitter `ACI_VERSION = "0.1.0-draft"` 1:1 对应).
pub const ACI_VERSION: &str = "0.1.0-draft";

/// 测试层 (per .aci.json `supported_layers`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    /// 单元测试.
    Ut,
    /// 集成测试.
    It,
    /// 系统测试.
    St,
    /// 端到端测试.
    E2e,
}

impl Layer {
    /// 与 Python `_lib_aci_emit.py::VALID_LAYERS = {"ut", "it", "st", "e2e"}` 1:1.
    pub const ALL: [&'static str; 4] = ["ut", "it", "st", "e2e"];

    /// 序列化为 schema 字符串 (跟 serde 序列化结果一致).
    pub fn as_str(self) -> &'static str {
        match self {
            Layer::Ut => "ut",
            Layer::It => "it",
            Layer::St => "st",
            Layer::E2e => "e2e",
        }
    }

    /// 从 schema 字符串解析 (跟 serde 反序列化结果一致).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ut" => Some(Layer::Ut),
            "it" => Some(Layer::It),
            "st" => Some(Layer::St),
            "e2e" => Some(Layer::E2e),
            _ => None,
        }
    }
}

/// 断言结果状态 (per .aci.json `assertion_statuses`).
///
/// 序列化使用 SCREAMING_SNAKE_CASE (跟 Python emitter 1:1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    /// 通过.
    #[serde(rename = "PASS")]
    Pass,
    /// 失败 (LLM 必读 reasoning).
    #[serde(rename = "FAIL")]
    Fail,
    /// 警告.
    #[serde(rename = "WARN")]
    Warn,
    /// 跳过.
    #[serde(rename = "SKIP")]
    Skip,
}

impl Status {
    /// 与 Python `VALID_STATUSES = {"PASS", "FAIL", "WARN", "SKIP"}` 1:1.
    pub const ALL: [&'static str; 4] = ["PASS", "FAIL", "WARN", "SKIP"];

    /// 序列化为 schema 字符串.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Warn => "WARN",
            Status::Skip => "SKIP",
        }
    }

    /// 从 schema 字符串解析.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "PASS" => Some(Status::Pass),
            "FAIL" => Some(Status::Fail),
            "WARN" => Some(Status::Warn),
            "SKIP" => Some(Status::Skip),
            _ => None,
        }
    }
}

/// 严重程度 (per .aci.json `severity_levels`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 严重.
    Critical,
    /// 高.
    High,
    /// 中.
    Medium,
    /// 低.
    Low,
    /// 信息.
    Info,
}

impl Severity {
    /// 与 Python `VALID_SEVERITIES = {"critical", "high", "medium", "low", "info"}` 1:1.
    pub const ALL: [&'static str; 5] = ["critical", "high", "medium", "low", "info"];

    /// 序列化为 schema 字符串.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    /// 从 schema 字符串解析.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(Severity::Critical),
            "high" => Some(Severity::High),
            "medium" => Some(Severity::Medium),
            "low" => Some(Severity::Low),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }
}

/// 期望 / 实测值类型 (per .aci.json `expect_value_types`, 17 种).
///
/// 数字混合变种 (`2xx` / `4xx5xx`) 用 `#[serde(rename = ...)]` 显式标注,
/// 因 `rename_all = "snake_case"` 不会在数字前后插 `_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpectValueType {
    /// 响应在 N 毫秒内.
    #[serde(rename = "response_within_ms")]
    ResponseWithinMs,
    /// 响应状态码 2xx.
    #[serde(rename = "response_status_2xx")]
    ResponseStatus2xx,
    /// 响应状态码 4xx/5xx.
    #[serde(rename = "response_status_4xx_5xx")]
    ResponseStatus4xx5xx,
    /// 字段等于某值.
    #[serde(rename = "field_equals")]
    FieldEquals,
    /// 字段存在.
    #[serde(rename = "field_exists")]
    FieldExists,
    /// 字段不存在.
    #[serde(rename = "field_not_exists")]
    FieldNotExists,
    /// 字段在范围内.
    #[serde(rename = "field_in_range")]
    FieldInRange,
    /// 字段大于某值.
    #[serde(rename = "field_greater_than")]
    FieldGreaterThan,
    /// 字段小于某值.
    #[serde(rename = "field_less_than")]
    FieldLessThan,
    /// 字段匹配正则.
    #[serde(rename = "field_matches_regex")]
    FieldMatchesRegex,
    /// 列表长度至少 N.
    #[serde(rename = "list_length_at_least")]
    ListLengthAtLeast,
    /// 列表长度等于 N.
    #[serde(rename = "list_length_equals")]
    ListLengthEquals,
    /// 工作流完成.
    #[serde(rename = "workflow_completes")]
    WorkflowCompletes,
    /// 事件触发.
    #[serde(rename = "event_emitted")]
    EventEmitted,
    /// 日志包含.
    #[serde(rename = "log_contains")]
    LogContains,
    /// 指标在阈值内.
    #[serde(rename = "metric_within_threshold")]
    MetricWithinThreshold,
    /// 无资源泄漏.
    #[serde(rename = "no_resource_leak")]
    NoResourceLeak,
}

impl ExpectValueType {
    /// 与 Python `VALID_EXPECT_TYPES` 1:1 (17 个).
    pub const ALL: [&'static str; 17] = [
        "response_within_ms",
        "response_status_2xx",
        "response_status_4xx_5xx",
        "field_equals",
        "field_exists",
        "field_not_exists",
        "field_in_range",
        "field_greater_than",
        "field_less_than",
        "field_matches_regex",
        "list_length_at_least",
        "list_length_equals",
        "workflow_completes",
        "event_emitted",
        "log_contains",
        "metric_within_threshold",
        "no_resource_leak",
    ];

    /// 序列化为 schema 字符串.
    pub fn as_str(self) -> &'static str {
        match self {
            ExpectValueType::ResponseWithinMs => "response_within_ms",
            ExpectValueType::ResponseStatus2xx => "response_status_2xx",
            ExpectValueType::ResponseStatus4xx5xx => "response_status_4xx_5xx",
            ExpectValueType::FieldEquals => "field_equals",
            ExpectValueType::FieldExists => "field_exists",
            ExpectValueType::FieldNotExists => "field_not_exists",
            ExpectValueType::FieldInRange => "field_in_range",
            ExpectValueType::FieldGreaterThan => "field_greater_than",
            ExpectValueType::FieldLessThan => "field_less_than",
            ExpectValueType::FieldMatchesRegex => "field_matches_regex",
            ExpectValueType::ListLengthAtLeast => "list_length_at_least",
            ExpectValueType::ListLengthEquals => "list_length_equals",
            ExpectValueType::WorkflowCompletes => "workflow_completes",
            ExpectValueType::EventEmitted => "event_emitted",
            ExpectValueType::LogContains => "log_contains",
            ExpectValueType::MetricWithinThreshold => "metric_within_threshold",
            ExpectValueType::NoResourceLeak => "no_resource_leak",
        }
    }

    /// 从 schema 字符串解析.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "response_within_ms" => Some(ExpectValueType::ResponseWithinMs),
            "response_status_2xx" => Some(ExpectValueType::ResponseStatus2xx),
            "response_status_4xx_5xx" => Some(ExpectValueType::ResponseStatus4xx5xx),
            "field_equals" => Some(ExpectValueType::FieldEquals),
            "field_exists" => Some(ExpectValueType::FieldExists),
            "field_not_exists" => Some(ExpectValueType::FieldNotExists),
            "field_in_range" => Some(ExpectValueType::FieldInRange),
            "field_greater_than" => Some(ExpectValueType::FieldGreaterThan),
            "field_less_than" => Some(ExpectValueType::FieldLessThan),
            "field_matches_regex" => Some(ExpectValueType::FieldMatchesRegex),
            "list_length_at_least" => Some(ExpectValueType::ListLengthAtLeast),
            "list_length_equals" => Some(ExpectValueType::ListLengthEquals),
            "workflow_completes" => Some(ExpectValueType::WorkflowCompletes),
            "event_emitted" => Some(ExpectValueType::EventEmitted),
            "log_contains" => Some(ExpectValueType::LogContains),
            "metric_within_threshold" => Some(ExpectValueType::MetricWithinThreshold),
            "no_resource_leak" => Some(ExpectValueType::NoResourceLeak),
            _ => None,
        }
    }
}

/// Scope 维度 (per .aci.json `scope_dimensions`, 6 dims).
///
/// 用 `BTreeMap` 保证序列化顺序稳定 (便于 diff), 跟 Python `dict[str, str]` 行为兼容
/// (Python 3.7+ dict 是有序插入; sort_keys=True 保证跨语言排序一致).
pub type ScopeMap = BTreeMap<String, String>;

/// 创建空 scope (后续用 `BTreeMap::insert` 或 `Scope::set(k, v)` API).
pub fn empty_scope() -> ScopeMap {
    BTreeMap::new()
}

/// Expect / Actual 字段 (per .aci.json `expect_actual_format`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectActual {
    /// 值类型 (17 个 enum 之一).
    #[serde(rename = "type")]
    pub ty: ExpectValueType,
    /// 值 (类型由 `ty` 决定; 用 `serde_json::Value` 兼容 int/float/bool/string).
    pub value: serde_json::Value,
    /// 自然语言说明 (LLM 必读).
    pub description: String,
}

impl ExpectActual {
    /// 创建 expect / actual 字段 (便捷构造器).
    pub fn new(
        ty: ExpectValueType,
        value: impl Into<serde_json::Value>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            ty,
            value: value.into(),
            description: description.into(),
        }
    }
}

/// ACI assertion (per .aci.json `schema_required_fields` 10 必填 + 可选).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    /// 唯一 ID (格式 `<module>:<id>`).
    pub assertion_id: String,
    /// ACI schema 版本 (固定 `"0.1.0-draft"`).
    pub aci_version: String,
    /// 测试层 (ut/it/st/e2e).
    pub layer: Layer,
    /// Scope 维度 (≥1 个 dim).
    pub scope: ScopeMap,
    /// 预期 (LLM 必读).
    pub expect: ExpectActual,
    /// 实测.
    pub actual: ExpectActual,
    /// 结果状态 (PASS/FAIL/WARN/SKIP).
    pub status: Status,
    /// 严重程度 (critical/high/medium/low/info).
    pub severity: Severity,
    /// 自然语言「为什么」(FAIL 时必填).
    pub reasoning: String,
    /// RFC3339 UTC 时间戳.
    pub captured_at: String,
    /// 自然语言「怎么修」(FAIL 时建议填).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub suggested_fix: Option<String>,
    /// Tags (推荐填, 帮助 LLM 分类).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tags: Option<Vec<String>>,
}
