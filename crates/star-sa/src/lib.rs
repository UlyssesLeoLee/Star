// SPDX-License-Identifier: MIT OR Apache-2.0
//! crates/star-sa — Service Adapter (4 Git Provider)
//! per spec/vcs/05 + spec/services/01 (Phase F 真实数据源接入)

// Phase F 骨架: trait + error + from_env + 4 provider 落地, 远端 API 调用留给 Phase F+.
// 文档待 Phase F+ 实装 API 客户端时统一补 (per "缺标比错标安全" 偏好).
#![allow(missing_docs)]

pub mod provider_bitbucket;
pub mod provider_github;
pub mod provider_gitlab;
pub mod provider_gitea;
pub mod provider_local;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("auth: {0}")] Auth(String),
    #[error("not_found: {0}")] NotFound(String),
    #[error("rate_limit: retry_after {0}s")] RateLimit(u64),
    #[error("network: {0}")] Network(String),
    #[error("decode: {0}")] Decode(String),
    #[error("other: {0}")] Other(String),
}

impl ProviderError {
    pub fn code(&self) -> &'static str { "PROVIDER_ERROR" }
    pub fn retriable(&self) -> bool { matches!(self, Self::Network(_) | Self::RateLimit(_)) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo { pub owner: String, pub name: String, pub default_branch: String, pub url: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch { pub name: String, pub sha: String, pub protected: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit { pub sha: String, pub message: String, pub author: String, pub timestamp: i64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrArgs { pub title: String, pub head: String, pub base: String, pub body: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest { pub number: u32, pub title: String, pub state: String, pub url: String }

#[async_trait]
pub trait Provider: Send + Sync {
    async fn list_repos(&self, owner: &str) -> Result<Vec<Repo>, ProviderError>;
    async fn get_repo(&self, owner: &str, name: &str) -> Result<Repo, ProviderError>;
    async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>, ProviderError>;
    async fn get_branch(&self, owner: &str, repo: &str, branch: &str) -> Result<Branch, ProviderError>;
    async fn list_commits(&self, owner: &str, repo: &str, sha: Option<&str>, limit: u32) -> Result<Vec<Commit>, ProviderError>;
    async fn get_commit(&self, owner: &str, repo: &str, sha: &str) -> Result<Commit, ProviderError>;
    async fn create_pull_request(&self, owner: &str, repo: &str, args: CreatePrArgs) -> Result<PullRequest, ProviderError>;
    async fn get_pull_request(&self, owner: &str, repo: &str, number: u32) -> Result<PullRequest, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_error_codes() {
        assert_eq!(ProviderError::Auth("x".into()).code(), "PROVIDER_ERROR");
        assert!(ProviderError::Network("x".into()).retriable());
        assert!(!ProviderError::Auth("x".into()).retriable());
    }
}
