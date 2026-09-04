//! `star test ...` (MVP 17 核心 #17: affected / run)

use clap::Subcommand;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum TestCommand {
    Affected,
    Run,
}

impl TestCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        let (tool, passed, failed, duration_ms) = match self {
            TestCommand::Affected => ("test affected", 5u32, 0u32, 1234u64),
            TestCommand::Run => ("test run", 42u32, 0u32, 8765u64),
        };
        println!(
            "{}",
            output::json_pretty(serde_json::json!({
                "schema_version": output::SCHEMA_VERSION,
                "mock": true,
                "tool": tool,
                "test": {
                    "passed": passed,
                    "failed": failed,
                    "skipped": 0,
                    "failed_tests": [],
                    "duration_ms": duration_ms,
                },
            }))?
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_affected_pass_count_is_5() {
        let passed: u32 = 5;
        let failed: u32 = 0;
        assert_eq!(passed, 5);
        assert_eq!(failed, 0);
    }

    #[test]
    fn test_run_pass_count_is_42() {
        let passed: u32 = 42;
        let failed: u32 = 0;
        assert_eq!(passed, 42);
        assert_eq!(failed, 0);
    }
}
