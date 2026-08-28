//! 主题系统的值对象层
//!
//! 定义: 主题 ID (三元组 enum + 可插拔), 设计令牌 (颜色 / 圆角 / 间距), 作用域 (三层).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 主题唯一标识 (三元组 enum, 可插拔扩展)
///
/// - 内置 2 个: Light + Dark (per 2026-08-29 04:09 JST 用户拍板)
/// - 预留扩展: HighContrast, Solarized, IndigoDark 等
/// - 第三方 / 租户自定义主题可追加 enum variant (需通过 INV-THEME-02 id 唯一校验)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeId {
    /// 亮色主题 (默认)
    Light,
    /// 暗色主题
    Dark,
    /// 扩展位 1: 高对比度 (无障碍)
    #[serde(skip)]
    HighContrast,
    /// 扩展位 2: Solarized 配色
    #[serde(skip)]
    Solarized,
}

impl ThemeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeId::Light => "light",
            ThemeId::Dark => "dark",
            ThemeId::HighContrast => "high-contrast",
            ThemeId::Solarized => "solarized",
        }
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, ThemeId::Dark | ThemeId::HighContrast | ThemeId::Solarized)
    }

    pub fn all_builtin() -> &'static [ThemeId] {
        &[ThemeId::Light, ThemeId::Dark]
    }
}

/// 主题作用域 (三层解析优先级, per 2026-08-29 04:09 JST 拍板)
///
/// 解析顺序: Personal > Tenant > Global
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeScope {
    /// 个人偏好 (localStorage / user API)
    Personal,
    /// 租户默认 (tenant API, 企业白标)
    Tenant,
    /// 平台全局默认 (admin API)
    Global,
}

impl ThemeScope {
    pub fn priority(&self) -> u8 {
        match self {
            ThemeScope::Personal => 3, // 最高
            ThemeScope::Tenant => 2,
            ThemeScope::Global => 1,
        }
    }
}

/// 设计令牌 — 颜色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorToken {
    pub name: String,   // 例: "primary", "surface", "text"
    pub hex: String,    // 例: "#5B5BD6"
    pub alpha: Option<f32>, // 0.0 - 1.0, None = 不透明
}

impl ColorToken {
    pub fn new(name: impl Into<String>, hex: impl Into<String>) -> Self {
        Self { name: name.into(), hex: hex.into(), alpha: None }
    }
}

/// 设计令牌 — 间距 (4px 基础栅格)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacingToken {
    pub name: String,   // 例: "space-4", "space-8"
    pub px: u32,        // 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64
}

/// 设计令牌 — 圆角 (3 档)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusToken {
    pub name: String,   // 例: "sm", "md", "lg"
    pub px: u32,        // 4 / 8 / 12
}

/// 完整主题定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub id: ThemeId,
    pub display_name: String,
    pub is_dark: bool,
    pub colors: Vec<ColorToken>,
    pub spacings: Vec<SpacingToken>,
    pub radii: Vec<RadiusToken>,
    pub version: u32, // INV-THEME-04: 主题升版
}

impl ThemeDefinition {
    pub fn to_css_variables(&self) -> String {
        let mut css = String::new();
        for color in &self.colors {
            let val = if let Some(a) = color.alpha {
                format!("{} {}; ", color.name, hex_to_rgba(&color.hex, a))
            } else {
                format!("{} {}; ", color.name, color.hex)
            };
            css.push_str(&val);
        }
        for s in &self.spacings {
            css.push_str(&format!("{} {}px; ", s.name, s.px));
        }
        for r in &self.radii {
            css.push_str(&format!("{} {}px; ", r.name, r.px));
        }
        css
    }

    pub fn to_css_variables_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for c in &self.colors {
            m.insert(c.name.clone(), c.hex.clone());
        }
        for s in &self.spacings {
            m.insert(s.name.clone(), format!("{}px", s.px));
        }
        for r in &self.radii {
            m.insert(r.name.clone(), format!("{}px", r.px));
        }
        m
    }
}

fn hex_to_rgba(hex: &str, alpha: f32) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return hex.to_string();
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!("rgba({}, {}, {}, {})", r, g, b, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_id_as_str() {
        assert_eq!(ThemeId::Light.as_str(), "light");
        assert_eq!(ThemeId::Dark.as_str(), "dark");
    }

    #[test]
    fn test_theme_id_is_dark() {
        assert!(!ThemeId::Light.is_dark());
        assert!(ThemeId::Dark.is_dark());
    }

    #[test]
    fn test_theme_scope_priority() {
        assert!(ThemeScope::Personal.priority() > ThemeScope::Tenant.priority());
        assert!(ThemeScope::Tenant.priority() > ThemeScope::Global.priority());
    }

    #[test]
    fn test_builtin_count() {
        assert_eq!(ThemeId::all_builtin().len(), 2);
    }

    #[test]
    fn test_hex_to_rgba() {
        assert_eq!(hex_to_rgba("#FF0000", 0.5), "rgba(255, 0, 0, 0.5)");
        assert_eq!(hex_to_rgba("#000000", 1.0), "rgba(0, 0, 0, 1)");
    }

    #[test]
    fn test_to_css_variables() {
        let theme = ThemeDefinition {
            id: ThemeId::Light,
            display_name: "Light".to_string(),
            is_dark: false,
            colors: vec![ColorToken::new("--color-primary", "#5B5BD6")],
            spacings: vec![SpacingToken { name: "--space-4".into(), px: 4 }],
            radii: vec![RadiusToken { name: "--radius-sm".into(), px: 4 }],
            version: 1,
        };
        let css = theme.to_css_variables();
        assert!(css.contains("--color-primary #5B5BD6"));
        assert!(css.contains("--space-4 4px"));
    }
}
