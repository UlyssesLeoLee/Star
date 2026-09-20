// SPDX-License-Identifier: MIT OR Apache-2.0
//! `domain-llm/src/provider` — W2 (ULYS-98-W2) real LLM provider implementations.
//!
//! Per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 2.1":
//! - `anthropic.rs` — Claude provider (POST https://api.anthropic.com/v1/messages)
//! - `openai.rs`    — OpenAI Compatible provider (POST https://api.openai.com/v1/chat/completions)
//! - `mock.rs`      — CI / no-key stub (deterministic reply, no network)
//! - `registry.rs`  — provider selection by model name
//!
//! **v0.0.2 (W2)**: real reqwest wiring + `no_network_mode` (守门 #25 v25 模式, CI 默认不联网).
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #5 v2 + 守门 #7 + 守门 #11 + 守门 #14 v2 + 守门 #25 v25):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"`).
//! - API key 不入 log / 不 println (仅校验是否非空, 失败返 `Backend`).
//! - 失败统一映射到 [`LlmProviderRegistryError::Backend`] 或 `Unimplemented`, 不 panic.
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

pub mod anthropic;
pub mod mock;
pub mod openai;
pub mod registry;

pub use anthropic::{AnthropicProvider, ANTHROPIC_DEFAULT_BASE_URL};
pub use mock::MockProvider;
pub use openai::{OpenAiProvider, OPENAI_DEFAULT_BASE_URL};
pub use registry::{DispatchProvider, ProviderRegistry, ProviderRegistryError, ProviderSelector};