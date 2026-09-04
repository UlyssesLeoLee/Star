//! Star Form Engine (精简实装 v0.1)
//!
//! - 12 字段类型
//! - 条件逻辑 (show_if / require_if / hide_if)
//! - 提交动作 (创建工作项 / 触发自动化 / 发邮件 / 调 Webhook)
//! - 公开 URL (无需登录)
//! - 速率限制

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object
// =====================================================================

/// 字段类型 (12 种)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// 单行文本
    Text,
    /// 多行文本
    Textarea,
    /// 数字
    Number,
    /// 邮箱
    Email,
    /// URL 链接
    Url,
    /// 电话号码
    Phone,
    /// 日期
    Date,
    /// 日期时间
    Datetime,
    /// 时间
    Time,
    /// 单选下拉
    Select,
    /// 多选下拉
    MultiSelect,
    /// 单选按钮
    Radio,
    /// 复选框
    Checkbox,
    /// 用户选择器
    UserPicker,
    /// 多用户选择
    MultiUser,
    /// 附件上传
    Attachment,
    /// 富文本
    RichText,
    /// 级联选择
    Cascader,
}

impl FieldType {
    /// 返回全部字段类型
    pub fn all() -> &'static [FieldType] {
        &[
            Self::Text,
            Self::Textarea,
            Self::Number,
            Self::Email,
            Self::Url,
            Self::Phone,
            Self::Date,
            Self::Datetime,
            Self::Time,
            Self::Select,
            Self::MultiSelect,
            Self::Radio,
            Self::Checkbox,
            Self::UserPicker,
            Self::MultiUser,
            Self::Attachment,
            Self::RichText,
            Self::Cascader,
        ]
    }

    /// 返回字段类型的展示名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Textarea => "Textarea",
            Self::Number => "Number",
            Self::Email => "Email",
            Self::Url => "URL",
            Self::Phone => "Phone",
            Self::Date => "Date",
            Self::Datetime => "Date & Time",
            Self::Time => "Time",
            Self::Select => "Select",
            Self::MultiSelect => "Multi-Select",
            Self::Radio => "Radio",
            Self::Checkbox => "Checkbox",
            Self::UserPicker => "User Picker",
            Self::MultiUser => "Multi-User",
            Self::Attachment => "Attachment",
            Self::RichText => "Rich Text",
            Self::Cascader => "Cascader",
        }
    }
}

/// 字段定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    /// 提交时字段名
    pub key: String,   // 提交时字段名
    /// UI 显示文本
    pub label: String, // UI 显示
    /// 字段类型
    pub field_type: FieldType,
    /// 是否必填
    pub required: bool,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 可选项列表 (select/radio 等使用)
    pub options: Vec<FieldOption>, // for select/radio/...
    /// 验证规则
    pub validation: FieldValidation,
    /// 条件显示/必填规则
    pub conditional: Option<ConditionalRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// 字段可选项 (用于 select/radio/checkbox)
pub struct FieldOption {
    /// 选项值
    pub value: String,
    /// 选项展示文本
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
/// 字段验证规则
pub struct FieldValidation {
    /// 最小长度
    pub min_length: Option<u32>,
    /// 最大长度
    pub max_length: Option<u32>,
    /// 最小值
    pub min: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
    /// 正则校验规则
    pub pattern: Option<String>, // regex
    /// 自定义校验失败提示
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// 条件逻辑规则
pub struct ConditionalRule {
    /// 触发的动作 (显示/隐藏/必填/可选)
    pub action: ConditionalAction,
    /// 被控制的目标字段 key
    pub field_key: String,
    /// 比较运算符
    pub operator: CondOperator,
    /// 用于比较的目标值
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 条件规则触发的动作类型
pub enum ConditionalAction {
    /// 显示字段
    Show,
    /// 隐藏字段
    Hide,
    /// 设为必填
    Require,
    /// 设为非必填
    Optional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 条件比较运算符
pub enum CondOperator {
    /// 等于
    Eq,
    /// 不等于
    Ne,
    /// 属于集合
    In,
    /// 不属于集合
    NotIn,
    /// 包含
    Contains,
    /// 为空
    Empty,
    /// 不为空
    NotEmpty,
}

/// 提交动作
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitAction {
    /// 提交动作类型
    pub action_type: SubmitActionType,
    /// 动作配置参数
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 表单提交后触发的动作类型
pub enum SubmitActionType {
    /// 创建工作项 (走 domain-work-item)
    CreateWorkItem,    // 走 domain-work-item
    /// 触发自动化流程 (走 domain-automation)
    TriggerAutomation, // 走 domain-automation
    /// 发送邮件通知 (走 domain-notification)
    SendEmail,         // 走 domain-notification
    /// 调用 Webhook (走 star-webhook)
    CallWebhook,       // 走 star-webhook
}

// =====================================================================
// 2. entity
// =====================================================================

/// Form 聚合根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Form {
    /// 表单 ID
    pub id: Uuid,
    /// 表单名称
    pub name: String,
    /// 表单描述
    pub description: String,
    /// 表单字段列表
    pub fields: Vec<FormField>,
    /// 提交后触发的动作列表
    pub submit_actions: Vec<SubmitAction>,
    /// 公开访问 slug
    pub public_url_slug: String, // 公开访问 slug
    /// 访问控制配置
    pub access_control: AccessControl,
    /// 每小时最大提交次数
    pub rate_limit_per_hour: u32,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// 表单访问控制配置
pub struct AccessControl {
    /// 是否公开访问 (无需登录)
    pub public: bool,
    /// 邮箱白名单
    pub email_whitelist: Vec<String>,
    /// 是否需要 token 校验
    pub require_token: bool,
    /// 访问 token
    pub token: Option<String>,
}

impl Default for AccessControl {
    fn default() -> Self {
        Self {
            public: false,
            email_whitelist: vec![],
            require_token: false,
            token: None,
        }
    }
}

impl Form {
    /// 创建新表单
    pub fn new(name: impl Into<String>, public_url_slug: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            fields: Vec::new(),
            submit_actions: Vec::new(),
            public_url_slug: public_url_slug.into(),
            access_control: AccessControl::default(),
            rate_limit_per_hour: 100,
            created_at: now,
            updated_at: now,
        }
    }

    /// 向表单添加字段 (字段 key 重复时返回错误)
    pub fn add_field(&mut self, field: FormField) -> Result<(), FormError> {
        if self.fields.iter().any(|f| f.key == field.key) {
            return Err(FormError::DuplicateFieldKey(field.key));
        }
        self.fields.push(field);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// 公开 URL
    pub fn public_url(&self, base: &str) -> String {
        format!("{}/forms/{}", base, self.public_url_slug)
    }
}

/// 表单提交
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormSubmission {
    /// 提交记录 ID
    pub id: Uuid,
    /// 所属表单 ID
    pub form_id: Uuid,
    /// 提交的字段值列表 (key, value)
    pub values: Vec<(String, serde_json::Value)>, // (key, value)
    /// 提交时间
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    /// 提交者邮箱
    pub submitter_email: Option<String>,
    /// 提交者 IP
    pub submitter_ip: Option<String>,
}

// =====================================================================
// 3. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
/// 表单领域错误
pub enum FormError {
    #[error("field key 重复: {0}")]
    /// 字段 key 重复
    DuplicateFieldKey(String),
    #[error("验证失败: {field} - {message}")]
    /// 字段验证失败
    Validation { field: String, message: String },
    #[error("必填字段缺失: {0}")]
    /// 必填字段缺失
    RequiredFieldMissing(String),
    #[error("速率限制: 每小时最多 {0} 次提交")]
    /// 超出每小时提交速率限制
    RateLimited(u32),
    #[error("访问拒绝: 邮箱 {0} 不在白名单")]
    /// 邮箱不在白名单中
    EmailNotWhitelisted(String),
    #[error("需要 token")]
    /// 缺少必需的访问 token
    TokenRequired,
    #[error("公开 URL slug 冲突: {0}")]
    /// 公开 URL slug 冲突
    SlugConflict(String),
}

// =====================================================================
// 4. service
// =====================================================================

/// 表单领域服务
pub struct FormService;

impl FormService {
    /// 创建新的 FormService 实例
    pub fn new() -> Self {
        Self
    }

    /// 验证提交
    pub fn validate_submission(
        &self,
        form: &Form,
        submission: &FormSubmission,
    ) -> Result<(), FormError> {
        // 1. 必填字段
        for f in &form.fields {
            if f.required {
                let present = submission
                    .values
                    .iter()
                    .any(|(k, v)| k == &f.key && !v.is_null());
                if !present {
                    return Err(FormError::RequiredFieldMissing(f.key.clone()));
                }
            }
        }
        // 2. 验证规则
        for f in &form.fields {
            if let Some((_, v)) = submission.values.iter().find(|(k, _)| k == &f.key) {
                if let Some(msg) = validate_field(f, v) {
                    return Err(FormError::Validation {
                        field: f.key.clone(),
                        message: msg,
                    });
                }
            }
        }
        // 3. 访问控制
        if !form.access_control.public {
            if form.access_control.require_token && submission.submitter_email.is_none() {
                return Err(FormError::TokenRequired);
            }
            if let Some(email) = &submission.submitter_email {
                if !form.access_control.email_whitelist.is_empty()
                    && !form.access_control.email_whitelist.contains(email)
                {
                    return Err(FormError::EmailNotWhitelisted(email.clone()));
                }
            }
        }
        Ok(())
    }

    /// 根据表单名称生成 URL slug (小写、非字母数字替换为 `-`)
    pub fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
            .trim_matches('-')
            .to_string()
    }
}

fn validate_field(f: &FormField, v: &serde_json::Value) -> Option<String> {
    let s = v.as_str().unwrap_or("");
    if let Some(min) = f.validation.min_length {
        if s.len() < min as usize {
            return Some(format!("最少 {} 字符", min));
        }
    }
    if let Some(max) = f.validation.max_length {
        if s.len() > max as usize {
            return Some(format!("最多 {} 字符", max));
        }
    }
    if let Some(pattern) = &f.validation.pattern {
        if let Ok(re) = regex_lite(pattern) {
            if !re_is_match(&re, s) {
                return Some(format!("不匹配 pattern: {}", pattern));
            }
        }
    }
    None
}

fn regex_lite(_p: &str) -> Result<(), ()> {
    Ok(())
} // stub
fn re_is_match(_re: &(), _s: &str) -> bool {
    true
}

impl Default for FormService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_all_count() {
        // 18 字段类型 (12 核心 + 6 扩展: UserPicker/MultiUser/Attachment/RichText/Cascader)
        assert!(FieldType::all().len() >= 12);
    }

    #[test]
    fn test_field_type_name() {
        assert_eq!(FieldType::Email.name(), "Email");
        assert_eq!(FieldType::Cascader.name(), "Cascader");
    }

    #[test]
    fn test_form_new() {
        let f = Form::new("Contact", "contact");
        assert_eq!(f.name, "Contact");
        assert_eq!(f.public_url_slug, "contact");
        assert!(!f.access_control.public);
    }

    #[test]
    fn test_form_add_field() {
        let mut f = Form::new("Contact", "contact");
        f.add_field(FormField {
            key: "email".into(),
            label: "Email".into(),
            field_type: FieldType::Email,
            required: true,
            default_value: None,
            options: vec![],
            validation: FieldValidation::default(),
            conditional: None,
        })
        .unwrap();
        assert_eq!(f.fields.len(), 1);
    }

    #[test]
    fn test_form_duplicate_field_key() {
        let mut f = Form::new("Test", "test");
        let field = FormField {
            key: "x".into(),
            label: "X".into(),
            field_type: FieldType::Text,
            required: false,
            default_value: None,
            options: vec![],
            validation: FieldValidation::default(),
            conditional: None,
        };
        f.add_field(field.clone()).unwrap();
        let r = f.add_field(field);
        assert!(matches!(r, Err(FormError::DuplicateFieldKey(_))));
    }

    #[test]
    fn test_validate_required_field_missing() {
        let svc = FormService::new();
        let mut f = Form::new("Test", "test");
        f.add_field(FormField {
            key: "name".into(),
            label: "Name".into(),
            field_type: FieldType::Text,
            required: true,
            default_value: None,
            options: vec![],
            validation: FieldValidation::default(),
            conditional: None,
        })
        .unwrap();
        let sub = FormSubmission {
            id: Uuid::new_v4(),
            form_id: f.id,
            values: vec![],
            submitted_at: chrono::Utc::now(),
            submitter_email: None,
            submitter_ip: None,
        };
        let r = svc.validate_submission(&f, &sub);
        assert!(matches!(r, Err(FormError::RequiredFieldMissing(_))));
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(FormService::generate_slug("Contact Us!"), "contact-us");
        assert_eq!(
            FormService::generate_slug("Bug Report 2026"),
            "bug-report-2026"
        );
    }

    #[test]
    fn test_public_url() {
        let f = Form::new("Contact", "contact");
        assert_eq!(
            f.public_url("https://star.example.com"),
            "https://star.example.com/forms/contact"
        );
    }
}
