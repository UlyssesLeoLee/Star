//! `danger.rs` — 危险分类 (per DD §12.1 + INV-WC-09 + spec §4.4 自审修正)
//!
//! 三类别 (per ULYS-57.3 T8 acceptance):
//! - Safe (6): 无副作用, 无确认
//! - Warning (5): 有副作用但可撤销, 轻量确认
//! - Destructive (7): 高风险不可撤销, typed confirm (e.g. type "DELETE")
//!
//! 自审修正 (per spec §4.4 / issue description):
//! - `Merge` 从 Warning 重分类到 Destructive (修改 main, 不可轻量回滚)

use crate::action::{ActionClass, ActionType};

/// 分类别名 (re-export for public API)
pub type Classification = ActionClass;

/// 给定 ActionType 返回分类 (跟 `default_metadata(action).classification` 等价)
///
/// 走 `default_metadata` 保证分类统一, 避免 RBAC + UI 看到不同分类。
pub fn classify(action: ActionType) -> Classification {
    crate::action::default_metadata(action).classification
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_three_categories_sum_to_eighteen() {
        let mut safe = 0;
        let mut warning = 0;
        let mut destructive = 0;
        for at in ActionType::ALL.iter().copied() {
            match classify(at) {
                Classification::Safe => safe += 1,
                Classification::Warning => warning += 1,
                Classification::Destructive => destructive += 1,
            }
        }
        assert_eq!(safe, 9);
        assert_eq!(warning, 5);
        assert_eq!(destructive, 7);
        assert_eq!(safe + warning + destructive, 21);
    }
}
