#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Create 4 new crates: domain-task, domain-llm, domain-mcp, domain-tool
(per ADR-0048 D41-D44 + OPT-NEXT-08 brief sub-task 2)

Per守门 #19 (agent 交互 Python 化), this script lives in scripts/automation/.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

# Each crate has short doc fields only (single-line) to avoid encoding issues
CRATES = [
    {
        "name": "domain-task",
        "description": "Star L0 dispatch TaskQueue business layer (per ADR-0048 D41 + SRS-001 G-1)",
        "trait_name": "TaskQueue",
        "trait_doc": "L0 TaskQueue business-layer trait (per ADR-0048 D41)",
        "struct_name": "InMemoryTaskQueue",
        "struct_doc": "L0 TaskQueue business-layer stub (per ADR-0048 D41 v0.0.1)",
        "method1": "enqueue",
        "method1_doc": "Enqueue a business Task (state=Pending)",
        "method2": "next",
        "method2_doc": "Get next pending Task (state=Pending -> Running)",
    },
    {
        "name": "domain-llm",
        "description": "Star L2 LLM Pool business layer (per ADR-0048 D42 + SRS-001 G-4)",
        "trait_name": "LlmProvider",
        "trait_doc": "L2 LLM Provider business-layer trait (per ADR-0048 D42)",
        "struct_name": "LlmProviderRegistry",
        "struct_doc": "L2 LLM Provider Registry stub (per ADR-0048 D42 v0.0.1)",
        "method1": "register",
        "method1_doc": "Register an LLM Provider",
        "method2": "lookup",
        "method2_doc": "Lookup by provider name",
    },
    {
        "name": "domain-mcp",
        "description": "Star L2 MCP Pool business layer (per ADR-0048 D43 + SRS-001 G-4)",
        "trait_name": "McpServer",
        "trait_doc": "L2 MCP Server business-layer trait (per ADR-0048 D43)",
        "struct_name": "McpServerRegistry",
        "struct_doc": "L2 MCP Server Registry stub (per ADR-0048 D43 v0.0.1)",
        "method1": "register",
        "method1_doc": "Register an MCP Server",
        "method2": "list_tools",
        "method2_doc": "List tools provided by a server",
    },
    {
        "name": "domain-tool",
        "description": "Star L2 Tool Registry business layer (per ADR-0048 D44 + SRS-001 G-4)",
        "trait_name": "Tool",
        "trait_doc": "L2 Tool business-layer trait (per ADR-0048 D44)",
        "struct_name": "ToolRegistry",
        "struct_doc": "L2 Tool Registry stub (per ADR-0048 D44 v0.0.1)",
        "method1": "register",
        "method1_doc": "Register a Tool",
        "method2": "lookup",
        "method2_doc": "Lookup by tool name",
    },
]


def make_cargo_toml(crate: dict) -> str:
    """Generate Cargo.toml for a domain-* crate."""
    return (
        "[package]\n"
        f'name = "{crate["name"]}"\n'
        "version.workspace = true\n"
        "edition.workspace = true\n"
        "rust-version.workspace = true\n"
        "authors.workspace = true\n"
        "license.workspace = true\n"
        "repository.workspace = true\n"
        f'description = "{crate["description"]}"\n'
        "\n"
        "[dependencies]\n"
        "serde = { workspace = true }\n"
        "serde_json = { workspace = true }\n"
        "async-trait = { workspace = true }\n"
        "thiserror = { workspace = true }\n"
        "uuid = { workspace = true }\n"
        "chrono = { workspace = true }\n"
        "tokio = { workspace = true }\n"
        "\n"
        "[dev-dependencies]\n"
        "tokio = { workspace = true }\n"
        "\n"
        "[lints]\n"
        "workspace = true\n"
    )


def make_lib_rs(crate: dict) -> str:
    """Generate src/lib.rs for a domain-* crate."""
    name = crate["name"]
    trait_name = crate["trait_name"]
    struct_name = crate["struct_name"]
    method1 = crate["method1"]
    method2 = crate["method2"]
    method1_doc = crate["method1_doc"]
    method2_doc = crate["method2_doc"]
    trait_doc = crate["trait_doc"]
    struct_doc = crate["struct_doc"]
    description = crate["description"]

    return f"""//! # {name}
//!
//! **{description}**
//!
//! **Gate (per ADR-0048 + AGENTS.md)**:
//! - Gate #1 v19: `cargo check --workspace --all-targets -j 4` 0 err
//! - Gate #4.2: concept->physical crate mapping satisfied
//! - Gate #10: author=Ulysses
//! - Gate #13: v0.0.1 stub does not touch DB schema, later v0.1+ W/T/M to be decided
//!
//! **Lead responsibility**: pending real 5-domain Lead signoff
//! (per AGENTS.md section 0 disclaimer gate #3)

#![allow(missing_docs)] // v0.0.1 stub, Phase 2 spec to complete docs later

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{{Deserialize, Serialize}};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// {trait_name} trait
// =====================================================================

/// **{trait_name}** -- {trait_doc}
#[async_trait]
pub trait {trait_name}: Send + Sync {{
    /// Initialize (e.g. load persistent state, start background workers)
    async fn init(&self) -> Result<(), {struct_name}Error>;

    /// Shutdown (e.g. flush pending state, release resources)
    async fn shutdown(&self) -> Result<(), {struct_name}Error>;

    /// Health check (return current backend state)
    async fn health_check(&self) -> Result<{struct_name}Health, {struct_name}Error>;
}}

// =====================================================================
// Error type
// =====================================================================

/// **{struct_name} error type** (per domain-* unified pattern)
#[derive(Debug, Error)]
pub enum {struct_name}Error {{
    /// Resource not found
    #[error("{{0}} not found")]
    NotFound(String),

    /// Backend not initialized
    #[error("backend not initialized")]
    NotInitialized,

    /// Resource exhausted
    #[error("{{resource}} exhausted: {{message}}")]
    Exhausted {{ resource: String, message: String }},

    /// Invalid operation
    #[error("invalid operation: {{0}}")]
    InvalidOperation(String),

    /// Backend error
    #[error("backend error: {{0}}")]
    Backend(String),
}}

// =====================================================================
// Health state
// =====================================================================

/// **{struct_name} health state**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct {struct_name}Health {{
    /// Current registered resource count
    pub count: u64,
    /// Backend identifier (e.g. "in-memory", "redis", "postgres")
    pub backend: String,
    /// Backend healthy flag
    pub healthy: bool,
}}

// =====================================================================
// {struct_name} stub (v0.0.1)
// =====================================================================

/// **{struct_name}** -- {struct_doc}
///
/// v0.0.1 stub: in-memory backend. Later v0.1+ will add persistence.
#[derive(Debug, Default, Clone)]
pub struct {struct_name} {{
    /// in-memory storage (key: Uuid, value: metadata)
    storage: HashMap<Uuid, String>,
    /// initialized flag
    initialized: bool,
}}

impl {struct_name} {{
    /// Create empty backend (uninitialized)
    pub fn new() -> Self {{
        Self {{
            storage: HashMap::new(),
            initialized: false,
        }}
    }}

    /// Mark backend as initialized (v0.0.1 stub helper for test)
    pub fn mark_initialized(&mut self) {{
        self.initialized = true;
    }}

    /// Current resource count
    pub fn count(&self) -> u64 {{
        self.storage.len() as u64
    }}

    /// {method1_doc} (per ADR-0048 stub)
    pub fn {method1}(&mut self, key: Uuid, value: String) -> Result<(), {struct_name}Error> {{
        if !self.initialized {{
            return Err({struct_name}Error::NotInitialized);
        }}
        self.storage.insert(key, value);
        Ok(())
    }}

    /// {method2_doc} (per ADR-0048 stub)
    pub fn {method2}(&self, key: &Uuid) -> Result<Option<&String>, {struct_name}Error> {{
        if !self.initialized {{
            return Err({struct_name}Error::NotInitialized);
        }}
        Ok(self.storage.get(key))
    }}
}}

#[async_trait]
impl {trait_name} for {struct_name} {{
    async fn init(&self) -> Result<(), {struct_name}Error> {{
        // v0.0.1 stub: no-op for in-memory backend.
        // Note: `initialized` flag is set via `mark_initialized()` for v0.0.1 stub testing;
        // later v0.1+ real backend will use interior mutability (e.g. Arc<Mutex<bool>>).
        Ok(())
    }}

    async fn shutdown(&self) -> Result<(), {struct_name}Error> {{
        // v0.0.1 stub: no-op for in-memory backend
        Ok(())
    }}

    async fn health_check(&self) -> Result<{struct_name}Health, {struct_name}Error> {{
        Ok({struct_name}Health {{
            count: self.count(),
            backend: "in-memory".to_string(),
            healthy: self.initialized,
        }})
    }}
}}

// =====================================================================
// Unit Tests (per Gate #1 v19 >= 1 test)
// =====================================================================

#[cfg(test)]
mod tests {{
    use super::*;

    /// **Test 1 (unit)**: {struct_name} new is empty
    #[test]
    fn {struct_name.lower()}_new_is_empty() {{
        let backend = {struct_name}::new();
        assert_eq!(backend.count(), 0);
        assert!(!backend.initialized);
    }}

    /// **Test 2 (unit)**: {method1} + {method2} roundtrip
    #[test]
    fn {struct_name.lower()}_{method1}_{method2}_roundtrip() {{
        let mut backend = {struct_name}::new();
        let key = Uuid::new_v4();
        let value = "test-value".to_string();

        // Before init: should return NotInitialized
        let err = backend.{method1}(key, value.clone()).unwrap_err();
        assert!(matches!(err, {struct_name}Error::NotInitialized));

        // After init: insert + lookup
        backend.mark_initialized();
        backend.{method1}(key, value.clone()).unwrap();
        let result = backend.{method2}(&key).unwrap();
        assert_eq!(result, Some(&value));
        assert_eq!(backend.count(), 1);
    }}

    /// **Test 3 (IT)**: trait async init + health_check
    #[tokio::test]
    async fn {struct_name.lower()}_init_and_health_check() {{
        let mut backend = {struct_name}::new();
        backend.mark_initialized();
        backend.init().await.unwrap();

        let health = backend.health_check().await.unwrap();
        assert_eq!(health.backend, "in-memory");
        assert_eq!(health.count, 0);
        assert!(health.healthy);
    }}

    /// **Test 4 (IT)**: trait async shutdown no-op
    #[tokio::test]
    async fn {struct_name.lower()}_shutdown_no_error() {{
        let backend = {struct_name}::new();
        backend.shutdown().await.unwrap();
    }}
}}
"""


def update_workspace_members(cargo_toml_path: Path, new_crates: list[str]) -> bool:
    """Add new crate names to workspace Cargo.toml members."""
    text = cargo_toml_path.read_text(encoding="utf-8")
    orig = text
    comment = "    # OPT-NEXT-08 (2026-09-07 14:30 JST) added 4 crates (per ADR-0048 D41-D44)\n"
    new_entries = "".join(f'    "{c}",\n' for c in new_crates)

    m = re.search(r"(members\s*=\s*\[)([\s\S]*?)(\n\])", text)
    if not m:
        return False

    members_body = m.group(2)
    new_body = members_body + "\n" + comment + new_entries
    new_text = text[: m.start(2)] + new_body + text[m.end(2) :]
    if new_text != orig:
        cargo_toml_path.write_text(new_text, encoding="utf-8")
        return True
    return False


def main() -> int:
    base = Path.cwd()
    crates_dir = base / "crates"
    if not crates_dir.exists():
        print(f"ERROR: crates/ not found at {crates_dir}", file=sys.stderr)
        return 1

    cargo_toml = base / "Cargo.toml"
    if not cargo_toml.exists():
        print(f"ERROR: Cargo.toml not found at {cargo_toml}", file=sys.stderr)
        return 1

    # First clean up any existing crate dirs (if re-running)
    new_crate_names = []
    for crate in CRATES:
        name = crate["name"]
        crate_dir = crates_dir / name
        if crate_dir.exists():
            import shutil
            shutil.rmtree(crate_dir)
        src_dir = crate_dir / "src"
        src_dir.mkdir(parents=True, exist_ok=True)

        cargo_path = crate_dir / "Cargo.toml"
        cargo_path.write_text(make_cargo_toml(crate), encoding="utf-8")
        print(f"  OK: {name}/Cargo.toml")

        lib_path = src_dir / "lib.rs"
        lib_path.write_text(make_lib_rs(crate), encoding="utf-8")
        print(f"  OK: {name}/src/lib.rs")

        new_crate_names.append(f"crates/{name}")

    # Update workspace Cargo.toml (only if not already updated)
    text = cargo_toml.read_text(encoding="utf-8")
    if "ADR-0048" in text:
        print("  SKIP: Cargo.toml already has ADR-0048 members")
    elif update_workspace_members(cargo_toml, new_crate_names):
        print(f"  OK: Cargo.toml (workspace members added: 4 new crates)")
    else:
        print(f"  ERROR: Cargo.toml update failed", file=sys.stderr)
        return 1

    print(f"\nResult: 4 new crates created + workspace members updated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
