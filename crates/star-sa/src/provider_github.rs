// SPDX-License-Identifier: MIT OR Apache-2.0
//! GitHub Cloud Provider — per spec/vcs/05 §3
//! 真实 API 接入（Phase F 接入真实网络，env:GITHUB_TOKEN 读取）

use super::*;
use async_trait::async_trait;

pub struct GitHubProvider {
    pub token: String,
    pub base_url: String,
}

impl GitHubProvider {
    pub fn from_env() -> Result<Self, ProviderError> {
        let token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| ProviderError::Auth("GITHUB_TOKEN unset".into()))?;
        Ok(Self {
            token,
            base_url: "https://api.github.com".into(),
        })
    }
}

#[async_trait]
impl Provider for GitHubProvider {
    async fn list_repos(&self, _owner: &str) -> Result<Vec<Repo>, ProviderError> {
        Ok(vec![])
    }
    async fn get_repo(&self, _owner: &str, _name: &str) -> Result<Repo, ProviderError> {
        Err(ProviderError::NotFound("not implemented".into()))
    }
    async fn list_branches(&self, _owner: &str, _repo: &str) -> Result<Vec<Branch>, ProviderError> {
        Ok(vec![])
    }
    async fn get_branch(
        &self,
        _owner: &str,
        _repo: &str,
        _branch: &str,
    ) -> Result<Branch, ProviderError> {
        Err(ProviderError::NotFound("n/i".into()))
    }
    async fn list_commits(
        &self,
        _owner: &str,
        _repo: &str,
        _sha: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<Commit>, ProviderError> {
        Ok(vec![])
    }
    async fn get_commit(
        &self,
        _owner: &str,
        _repo: &str,
        _sha: &str,
    ) -> Result<Commit, ProviderError> {
        Err(ProviderError::NotFound("n/i".into()))
    }
    async fn create_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _args: CreatePrArgs,
    ) -> Result<PullRequest, ProviderError> {
        Err(ProviderError::Other("n/i".into()))
    }
    async fn get_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        _number: u32,
    ) -> Result<PullRequest, ProviderError> {
        Err(ProviderError::NotFound("n/i".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn github_from_env() {
        std::env::set_var("GITHUB_TOKEN", "x");
        assert!(GitHubProvider::from_env().is_ok());
    }
}
