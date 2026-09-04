//! `star code ...` (MVP 17 核心 #7-#9: search / symbol / references)

use clap::Subcommand;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

#[derive(Debug, Subcommand)]
pub(crate) enum CodeCommand {
    Search { query: String },
    Symbol { name: String },
    References { name: String },
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeMatch {
    file: String,
    line: u32,
    snippet: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeSearchResult {
    query: String,
    matches: Vec<CodeMatch>,
    total: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct SymbolResult {
    name: String,
    kind: String,
    file: String,
    line: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReferencesResult {
    name: String,
    references: Vec<CodeMatch>,
    total: u32,
}

impl CodeCommand {
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            CodeCommand::Search { query } => {
                let result = CodeSearchResult {
                    query: query.clone(),
                    matches: vec![CodeMatch {
                        file: "crates/star-cli/src/commands/code.rs".to_string(),
                        line: 1,
                        snippet: format!("// search: {query}"),
                    }],
                    total: 1,
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "code search",
                        "result": result,
                    }))?
                );
                Ok(())
            }
            CodeCommand::Symbol { name } => {
                let result = SymbolResult {
                    name: name.clone(),
                    kind: "function".to_string(),
                    file: format!("src/{name}.rs"),
                    line: 1,
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "code symbol",
                        "symbol": result,
                    }))?
                );
                Ok(())
            }
            CodeCommand::References { name } => {
                let result = ReferencesResult {
                    name: name.clone(),
                    references: vec![CodeMatch {
                        file: "src/main.rs".to_string(),
                        line: 1,
                        snippet: format!("use {name}"),
                    }],
                    total: 1,
                };
                println!(
                    "{}",
                    output::json_pretty(serde_json::json!({
                        "schema_version": output::SCHEMA_VERSION,
                        "mock": true,
                        "tool": "code references",
                        "result": result,
                    }))?
                );
                Ok(())
            }
        }
    }
}
