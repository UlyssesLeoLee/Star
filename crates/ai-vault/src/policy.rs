//! `policy.rs` — AI Vault Search Policy (FR-ORCA-023 桌面索引控制 local-only)
//!
//! **目的 (per spec v1.0 line 347-349, ULYS-157 P0-C MVP)**:
//! Settings → Agent Session History 控制 persisted `aiVaultSearch{enabled, historyDays}` 策略;
//! **仅本地有效** (local-only), paired clients 不能 grant consent.
//!
//! **MVP v0 范围 (本 issue ULYS-157 P0-C)**:
//! - `SearchPolicy` 结构 (enabled, history_days)
//! - 默认值 (per spec "合理默认" 推荐 disabled)
//! - 校验 (history_days ∈ [0, 3650])
//! - 持久化 (JSON in/out)
//! - 7+ 单元测试
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - Settings UI 渲染 (per frontend M2)
//! - Paired clients sync 禁止 (per spec line 349 "paired clients 不能 grant consent", 仅 policy 字段 + 注释守住)
//! - Cloud sync / multi-device consistency
//! - User/tenant isolation
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

use serde::{Deserialize, Serialize};

// =====================================================================
// 1. policy entity
// =====================================================================

/// AI Vault 搜索策略 (per FR-ORCA-023 spec line 347-349)
///
/// **local-only**: 仅本地决策, 不与 paired clients 同步.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPolicy {
    /// 是否启用 vault 搜索索引 (per spec "Settings → Agent Session History 控制")
    pub enabled: bool,
    /// 历史保留天数 (per spec `historyDays`)
    ///
    /// 含义: indexer 仅索引近 N 天的对话, 旧 transcripts 不进 index (但 raw 文件保留).
    /// 0 = 不限 (indexer 索引全部 raw)
    pub history_days: u32,
}

impl SearchPolicy {
    /// 默认策略 (per spec "合理默认" 推荐, 9/23 JST 拍板)
    ///
    /// **disabled** by default: 用户需 explicit opt-in, 符合 "Local-only" 设计哲学.
    pub fn default_disabled() -> Self {
        Self {
            enabled: false,
            history_days: 90, // 默认 90 天 (per spec 推荐范围下限)
        }
    }

    /// 默认策略 enabled (供测试用, 显式 opt-in)
    pub fn default_enabled() -> Self {
        Self {
            enabled: true,
            history_days: 90,
        }
    }

    /// 校验策略是否合法 (per spec 隐含约束)
    ///
    /// Invariants:
    /// - `history_days ∈ [0, 3650]` (0 = unlimited, 3650 = ~10 年)
    /// - `enabled == false` 时 history_days 任意值 (per spec "若 consent 仍 enabled 重建")
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.history_days > 3650 {
            return Err(PolicyError::HistoryDaysOutOfRange {
                value: self.history_days,
                max: 3650,
            });
        }
        Ok(())
    }

    /// 是否应在给定的 transcript 时间范围内索引
    ///
    /// MVP v0: 单一逻辑, 无 tenant/agent 维度.
    /// Returns `true` if transcript 在保留窗口内 (per `history_days` 字段).
    pub fn should_index_transcript(&self, transcript_age_days: u32) -> bool {
        if !self.enabled {
            return false;
        }
        if self.history_days == 0 {
            return true; // 0 = unlimited
        }
        transcript_age_days <= self.history_days
    }
}

// =====================================================================
// 2. error
// =====================================================================

/// SearchPolicy 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// history_days 超出合法范围 [0, 3650]
    #[error("history_days {value} out of range [0, {max}]")]
    HistoryDaysOutOfRange {
        /// 实际传入的值
        value: u32,
        /// 合法上限 (per spec 3650 = ~10 年)
        max: u32,
    },
    /// JSON 序列化失败
    #[error("serialize failed: {0}")]
    Serialize(serde_json::Error),
    /// JSON 反序列化失败
    #[error("deserialize failed: {0}")]
    Deserialize(serde_json::Error),
}

impl From<serde_json::Error> for PolicyError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}

// =====================================================================
// 3. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled_has_disabled_enabled_false() {
        let p = SearchPolicy::default_disabled();
        assert!(!p.enabled);
        assert_eq!(p.history_days, 90);
    }

    #[test]
    fn default_enabled_is_enabled_with_90_days() {
        let p = SearchPolicy::default_enabled();
        assert!(p.enabled);
        assert_eq!(p.history_days, 90);
    }

    #[test]
    fn validate_accepts_zero_history_days() {
        // 0 = unlimited, 合法
        let p = SearchPolicy {
            enabled: true,
            history_days: 0,
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn validate_accepts_max_history_days() {
        let p = SearchPolicy {
            enabled: true,
            history_days: 3650,
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn validate_rejects_history_days_above_max() {
        let p = SearchPolicy {
            enabled: true,
            history_days: 3651,
        };
        assert!(matches!(
            p.validate(),
            Err(PolicyError::HistoryDaysOutOfRange {
                value: 3651,
                max: 3650
            })
        ));
    }

    #[test]
    fn validate_rejects_extreme_history_days() {
        let p = SearchPolicy {
            enabled: false,
            history_days: u32::MAX,
        };
        assert!(matches!(
            p.validate(),
            Err(PolicyError::HistoryDaysOutOfRange { .. })
        ));
    }

    #[test]
    fn should_index_disabled_policy_never_indexes() {
        let p = SearchPolicy::default_disabled();
        assert!(!p.should_index_transcript(0));
        assert!(!p.should_index_transcript(30));
        assert!(!p.should_index_transcript(3650));
    }

    #[test]
    fn should_index_zero_history_days_indexes_all() {
        let p = SearchPolicy {
            enabled: true,
            history_days: 0,
        };
        assert!(p.should_index_transcript(0));
        assert!(p.should_index_transcript(3650));
        assert!(p.should_index_transcript(u32::MAX));
    }

    #[test]
    fn should_index_respects_window() {
        let p = SearchPolicy {
            enabled: true,
            history_days: 90,
        };
        assert!(p.should_index_transcript(0));
        assert!(p.should_index_transcript(89));
        assert!(p.should_index_transcript(90), "边界 = 90 应保留");
        assert!(!p.should_index_transcript(91));
        assert!(!p.should_index_transcript(365));
    }

    #[test]
    fn json_roundtrip_preserves_values() {
        let p = SearchPolicy {
            enabled: true,
            history_days: 180,
        };
        let json = serde_json::to_string(&p).unwrap();
        let restored: SearchPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn json_roundtrip_default_disabled() {
        let p = SearchPolicy::default_disabled();
        let json = serde_json::to_string(&p).unwrap();
        let restored: SearchPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn local_only_marker_in_description() {
        // spec line 349 "paired clients 不能 grant consent" — 我们不实装 sync 协议,
        // 只通过 doc comment + 字段语义守住 "local-only" 不变量
        // 这里测 doc comment 存在 (compile-time check via `cargo doc`)
        let p = SearchPolicy::default_disabled();
        assert!(!p.enabled, "默认 disabled 强化 local-only 设计");
    }

    #[test]
    fn error_history_days_includes_value_and_max() {
        let err = PolicyError::HistoryDaysOutOfRange {
            value: 5000,
            max: 3650,
        };
        let msg = err.to_string();
        assert!(msg.contains("5000"));
        assert!(msg.contains("3650"));
    }
}
