//! # Version CAS 乐观锁
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.3.1
//!
//! 多 agent 并发改同一行时, 走 version CAS (Compare-And-Swap) 防数据覆盖
//!
//! 守门合规:
//! - 守门 #13 a L0 协调: DomainMutex 业务层乐观锁, 跨 L1 SubAgent 互改
//! - 守门 #11 缺标比错标: 不匹配返回 VersionMismatch, 由调用方决定重试

use thiserror::Error;

/// version CAS 不匹配错误.
#[derive(Debug, Error)]
#[error("version mismatch: expected {expected}, actual {actual}")]
pub struct VersionMismatchError {
    /// 期望 version.
    pub expected: u64,
    /// 实际 current version.
    pub actual: u64,
}

impl VersionMismatchError {
    /// 创建新错误.
    pub fn new(expected: u64, actual: u64) -> Self {
        Self { expected, actual }
    }
}

/// Version CAS 工具.
pub struct VersionCAS;

impl VersionCAS {
    /// 校验 `expected_version == current_version`.
    ///
    /// 不匹配返回 `VersionMismatchError`.
    pub fn check(expected: u64, current: u64) -> Result<(), VersionMismatchError> {
        if expected == current {
            Ok(())
        } else {
            Err(VersionMismatchError::new(expected, current))
        }
    }

    /// 校验新 version 单调递增 (旧 version + 1).
    pub fn check_monotonic(old: u64, new: u64) -> Result<(), VersionMismatchError> {
        if new == old + 1 {
            Ok(())
        } else {
            Err(VersionMismatchError::new(old + 1, new))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_pass() {
        assert!(VersionCAS::check(5, 5).is_ok());
    }

    #[test]
    fn test_check_fail() {
        let err = VersionCAS::check(5, 6).unwrap_err();
        assert_eq!(err.expected, 5);
        assert_eq!(err.actual, 6);
    }

    #[test]
    fn test_check_monotonic_pass() {
        assert!(VersionCAS::check_monotonic(5, 6).is_ok());
        // 允许从 0 到 1
        assert!(VersionCAS::check_monotonic(0, 1).is_ok());
    }

    #[test]
    fn test_check_monotonic_fail_skip() {
        // 跳过 version (5 → 7) 失败
        let err = VersionCAS::check_monotonic(5, 7).unwrap_err();
        assert_eq!(err.expected, 6);
        assert_eq!(err.actual, 7);
    }

    #[test]
    fn test_check_monotonic_fail_revert() {
        // 回退 (5 → 4) 失败
        let err = VersionCAS::check_monotonic(5, 4).unwrap_err();
        assert_eq!(err.expected, 6);
        assert_eq!(err.actual, 4);
    }
}
