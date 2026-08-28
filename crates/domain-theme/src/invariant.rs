//! 主题系统不变量

/// INV-THEME-01: 主题 ID 必须是 ThemeId enum 中已定义的合法值
pub fn inv_01_id_valid(id: &super::value_object::ThemeId) -> bool {
    // enum 自身已保证 — 这里校验 id 字符串非空
    !id.as_str().is_empty()
}

/// INV-THEME-02: 同一作用域下, 主题 ID 必须唯一
/// - Personal scope: (actor_id, theme_id) 唯一
/// - Tenant scope: (tenant_id, theme_id) 唯一
/// - Global scope: theme_id 全局唯一
pub fn inv_02_id_unique(
    scope: &super::value_object::ThemeScope,
    tenant_id: uuid::Uuid,
    actor_id: Option<uuid::Uuid>,
    existing: &[(uuid::Uuid, Option<uuid::Uuid>, super::value_object::ThemeId)],
    candidate: super::value_object::ThemeId,
) -> bool {
    !existing.iter().any(|(t, a, id)| {
        if id != &candidate { return false; }
        match scope {
            super::value_object::ThemeScope::Personal => a == &actor_id,
            super::value_object::ThemeScope::Tenant => t == &tenant_id,
            super::value_object::ThemeScope::Global => true,
        }
    })
}

/// INV-THEME-03: 主题定义必须包含全部必要字段
/// - 至少 1 个 color token
/// - 至少 1 个 spacing token
/// - 至少 1 个 radius token
/// - is_dark 必须与 ThemeId::is_dark() 一致
pub fn inv_03_definition_complete(theme: &super::value_object::ThemeDefinition) -> bool {
    if theme.colors.is_empty() { return false; }
    if theme.spacings.is_empty() { return false; }
    if theme.radii.is_empty() { return false; }
    theme.is_dark == theme.id.is_dark()
}

/// INV-THEME-04: 主题版本号必须单调递增
pub fn inv_04_version_monotonic(old: u32, new: u32) -> bool {
    new > old
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::*;

    #[test]
    fn test_inv_01_id_valid() {
        assert!(inv_01_id_valid(&ThemeId::Light));
        assert!(inv_01_id_valid(&ThemeId::Dark));
    }

    #[test]
    fn test_inv_02_id_unique_personal() {
        let t = uuid::Uuid::new_v4();
        let a1 = Some(uuid::Uuid::new_v4());
        let a2 = Some(uuid::Uuid::new_v4());
        let existing = vec![(t, a1, ThemeId::Dark)];
        // 同一用户重复注册 → 违规
        assert!(!inv_02_id_unique(&ThemeScope::Personal, t, a1, &existing, ThemeId::Dark));
        // 不同用户 → OK
        assert!(inv_02_id_unique(&ThemeScope::Personal, t, a2, &existing, ThemeId::Dark));
    }

    #[test]
    fn test_inv_03_definition_complete() {
        let ok = ThemeDefinition {
            id: ThemeId::Light,
            display_name: "Light".into(),
            is_dark: false,
            colors: vec![ColorToken::new("a", "#fff")],
            spacings: vec![SpacingToken { name: "a".into(), px: 4 }],
            radii: vec![RadiusToken { name: "a".into(), px: 4 }],
            version: 1,
        };
        assert!(inv_03_definition_complete(&ok));

        let empty = ThemeDefinition {
            colors: vec![],
            ..ok.clone()
        };
        assert!(!inv_03_definition_complete(&empty));

        let mismatch = ThemeDefinition { is_dark: true, ..ok };
        assert!(!inv_03_definition_complete(&mismatch));
    }

    #[test]
    fn test_inv_04_version_monotonic() {
        assert!(inv_04_version_monotonic(1, 2));
        assert!(!inv_04_version_monotonic(2, 1));
        assert!(!inv_04_version_monotonic(1, 1));
    }
}
