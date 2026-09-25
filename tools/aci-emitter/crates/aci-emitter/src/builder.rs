//! `AciEmitter` builder (对标 Python `class AciEmitter`).
//!
//! Usage (跟 Python emitter 同款):
//! ```no_run
//! use aci_emitter::{AciEmitter, ExpectActual, ExpectValueType, Layer, Severity, Status};
//! use aci_emitter::types::empty_scope;
//!
//! let em = AciEmitter::new(Layer::It);
//! let mut scope = empty_scope();
//! scope.insert("module".to_string(), "guards".to_string());
//! let assertion = em.build(
//!     "star-flash-mock:guards:g-1",
//!     scope,
//!     ExpectActual::new(ExpectValueType::ResponseWithinMs, 2000, "应 2s 内"),
//!     ExpectActual::new(ExpectValueType::ResponseWithinMs, 5, "实测 5ms"),
//!     Status::Pass,
//!     Severity::Info,
//!     "实测延迟远低于预期",
//! ).unwrap();
//! let s = em.to_json(&assertion);
//! ```

use crate::error::{AciEmitError, AciValidationError};
use crate::types::{Assertion, ExpectActual, Layer, ScopeMap, Severity, Status};
use crate::validators;
use std::fs;
use std::path::Path;

/// ACI emitter 构造器.
///
/// 持有一个 `Layer` (单 emitter 实例对应一个测试层), 跟 Python `__init__(layer)` 1:1.
#[derive(Debug, Clone)]
pub struct AciEmitter {
    layer: Layer,
    project: String,
}

impl AciEmitter {
    /// 新建 emitter, `layer` ∈ ut/it/st/e2e.
    pub fn new(layer: Layer) -> Self {
        Self {
            layer,
            project: "aci-emitter".to_string(),
        }
    }

    /// 自定义默认 project (默认 "aci-emitter", Python 默认 "star-flash-mock").
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = project.into();
        self
    }

    /// 当前测试层.
    pub fn layer(&self) -> Layer {
        self.layer
    }

    /// 当前默认 project.
    pub fn project(&self) -> &str {
        &self.project
    }

    /// 构造一条 assertion dict (跟 Python `build(**kwargs)` 1:1).
    ///
    /// `reasoning`: FAIL 时必填, 其他状态可选.
    /// `suggested_fix`: 可选, 任意状态都可填 (FAIL 时建议填).
    /// `tags`: 可选, 推荐填 (帮助 LLM 分类).
    /// `captured_at`: 可选, RFC3339 字符串; 缺省 = now UTC.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &self,
        assertion_id: &str,
        scope: ScopeMap,
        expect: ExpectActual,
        actual: ExpectActual,
        status: Status,
        severity: Severity,
        reasoning: &str,
    ) -> Result<Assertion, AciValidationError> {
        self.build_with(
            assertion_id,
            scope,
            expect,
            actual,
            status,
            severity,
            reasoning,
            None,
            None,
            None,
        )
    }

    /// `build` 的完整版 (含 suggested_fix / tags / captured_at).
    #[allow(clippy::too_many_arguments)]
    pub fn build_with(
        &self,
        assertion_id: &str,
        scope: ScopeMap,
        expect: ExpectActual,
        actual: ExpectActual,
        status: Status,
        severity: Severity,
        reasoning: &str,
        suggested_fix: Option<String>,
        tags: Option<Vec<String>>,
        captured_at: Option<String>,
    ) -> Result<Assertion, AciValidationError> {
        // ---- 字段级验证 (跟 Python 一致顺序) ----
        validators::validate_assertion_id(assertion_id)?;
        validators::validate_scope(&scope)?;
        validators::validate_expect_or_actual("expect", &expect)?;
        validators::validate_expect_or_actual("actual", &actual)?;
        validators::validate_layer(self.layer)?;
        validators::validate_status(status)?;
        validators::validate_severity(severity)?;

        // FAIL 必填 reasoning
        if status == Status::Fail && reasoning.trim().is_empty() {
            return Err(AciValidationError::FailRequiresReasoning);
        }

        let scope_with_default = validators::with_default_scope(&scope, &self.project);

        Ok(Assertion {
            assertion_id: assertion_id.to_string(),
            aci_version: validators::aci_version().to_string(),
            layer: self.layer,
            scope: scope_with_default,
            expect,
            actual,
            status,
            severity,
            reasoning: reasoning.to_string(),
            captured_at: captured_at.unwrap_or_else(validators::now_rfc3339),
            suggested_fix,
            tags,
        })
    }

    /// 便利方法: 给已构造的 assertion 加 tags (dedupe).
    ///
    /// 跟 Python `with_tags(assertion, tags)` 1:1.
    pub fn with_tags(&self, mut assertion: Assertion, tags: Vec<String>) -> Assertion {
        let existing = assertion.tags.unwrap_or_default();
        let mut merged = existing;
        for t in tags {
            if !merged.contains(&t) {
                merged.push(t);
            }
        }
        assertion.tags = if merged.is_empty() {
            None
        } else {
            Some(merged)
        };
        assertion
    }

    /// 序列化为 JSON 字符串 (sort_keys + ensure_ascii=false, 跟 Python `to_json` 1:1).
    pub fn to_json(&self, assertion: &Assertion) -> String {
        // serde_json::Value + BTreeMap scope 已保证有序
        // ensure_ascii=false: serde_json 默认是 false (会保留 UTF-8)
        serde_json::to_string_pretty(assertion).expect("Assertion Serialize derive 不应失败")
    }

    /// 序列化为 `serde_json::Value` (for programmatic 消费).
    pub fn to_value(&self, assertion: &Assertion) -> serde_json::Value {
        serde_json::to_value(assertion).expect("Assertion Serialize derive 不应失败")
    }

    /// 写入 JSON 到 `path` (UTF-8 + 末尾换行, 跟 Python `write` 1:1).
    pub fn write(&self, assertion: &Assertion, path: &Path) -> Result<(), AciEmitError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, self.to_json(assertion) + "\n")?;
        Ok(())
    }

    /// 从文件读取 + 解析断言 (跟 Python 暂无直接对标, 但 `from_file` 在 brief §1.1 中点名要求).
    ///
    /// 仅做 JSON parse + 字段必填检查 (不重跑 build 的全部 validator,
    /// 因为读取的是已通过 schema 校验的产物).
    pub fn from_file(path: &Path) -> Result<Assertion, AciEmitError> {
        let raw = fs::read_to_string(path)?;
        let assertion: Assertion = serde_json::from_str(&raw)?;
        Ok(assertion)
    }
}
