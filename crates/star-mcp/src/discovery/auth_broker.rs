//! # AuthBroker - env_var passthrough 凭证注入 (per 守门 #5)
//!
//! per `ADR-0049 AI 工具自动扫描 + 链接` §2.2 (env_var passthrough)
//!
//! 守门合规:
//! - 守门 #5: 不打印 env 字段值, 仅引用
//! - 守门 #7: 0 unsafe
//! - 守门 #23: AI 凭证必须真实 (不 mock, per 拍板 C)

use std::collections::HashMap;
use std::process::Command;

/// AuthBroker - env_var passthrough (per 守门 #5).
pub(crate) struct AuthBroker {
    /// 已注入的 env vars (key only, value 不存储 per 守门 #5).
    injected_keys: HashMap<String, ()>,
}

impl AuthBroker {
    /// 创建新 broker.
    pub(crate) fn new() -> Self {
        Self {
            injected_keys: HashMap::new(),
        }
    }

    /// 注入 env var 到 command (passthrough, per 守门 #5).
    ///
    /// **不** 读取 env 字段值, 仅 key 列表 + process env 引用.
    /// 守门 #5: env 值永远不打印.
    pub(crate) fn inject(&mut self, command: &mut Command, env_keys: &[String]) {
        for key in env_keys {
            // passthrough: 用 std::env::var 引用, 不存值
            if let Ok(value) = std::env::var(key) {
                command.env(key, value);
            }
            // 记录已注入 key (per 守门 #5: 仅 key, 无 value)
            self.injected_keys.insert(key.clone(), ());
        }
    }

    /// 测用: 已注入 key 列表.
    pub(crate) fn injected_keys(&self) -> Vec<String> {
        self.injected_keys.keys().cloned().collect()
    }

    /// 测用: 注入数.
    pub(crate) fn len(&self) -> usize {
        self.injected_keys.len()
    }

    /// 测用: 是否空.
    pub(crate) fn is_empty(&self) -> bool {
        self.injected_keys.is_empty()
    }
}

impl Default for AuthBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_passes_through_existing_env() {
        // 守门 #5: 不打印 env value, 仅引用
        std::env::set_var("STAR_TEST_API_KEY", "secret-value-not-printed");

        let mut cmd = Command::new("dummy");
        let mut broker = AuthBroker::new();
        broker.inject(&mut cmd, &["STAR_TEST_API_KEY".to_string()]);

        assert_eq!(broker.len(), 1);
        assert_eq!(
            broker.injected_keys(),
            vec!["STAR_TEST_API_KEY".to_string()]
        );

        // 清理
        std::env::remove_var("STAR_TEST_API_KEY");
    }

    #[test]
    fn test_inject_skips_missing_env() {
        let mut cmd = Command::new("dummy");
        let mut broker = AuthBroker::new();
        broker.inject(&mut cmd, &["NONEXISTENT_VAR_XYZ".to_string()]);

        // env 不存在, key 仍记录 (用于可观测), 但不 inject
        assert_eq!(broker.len(), 1);
    }

    #[test]
    fn test_inject_multiple_keys() {
        std::env::set_var("STAR_TEST_KEY_A", "a");
        std::env::set_var("STAR_TEST_KEY_B", "b");

        let mut cmd = Command::new("dummy");
        let mut broker = AuthBroker::new();
        broker.inject(
            &mut cmd,
            &["STAR_TEST_KEY_A".to_string(), "STAR_TEST_KEY_B".to_string()],
        );

        assert_eq!(broker.len(), 2);

        std::env::remove_var("STAR_TEST_KEY_A");
        std::env::remove_var("STAR_TEST_KEY_B");
    }
}
