// SPDX-License-Identifier: MIT OR Apache-2.0
//! Local Git Provider — 本地 git CLI (per spec/vcs/05 §2 备选 — Phase F 倾向 Gitea, Local 推 Phase F+)

use super::*;
use async_trait::async_trait;
use tokio::process::Command;

pub struct LocalProvider;

#[async_trait]
impl Provider for LocalProvider {
    async fn list_repos(&self, _owner: &str) -> Result<Vec<Repo>, ProviderError> {
        Ok(vec![])
    }
    async fn get_repo(&self, _owner: &str, _name: &str) -> Result<Repo, ProviderError> {
        Err(ProviderError::NotFound("n/i".into()))
    }
    async fn list_branches(&self, _owner: &str, _repo: &str) -> Result<Vec<Branch>, ProviderError> {
        let out = Command::new("git")
            .args(["branch"])
            .output()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| Branch {
                name: l.trim().replace("* ", ""),
                sha: "".into(),
                protected: false,
            })
            .collect())
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
        Err(ProviderError::Other("local git has no PR".into()))
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
    #[tokio::test]
    async fn local_list_branches() {
        let p = LocalProvider;
        let r = p.list_branches("", "").await;
        assert!(r.is_ok());
    }
}
