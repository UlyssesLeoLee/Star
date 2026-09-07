// SPDX-License-Identifier: MIT OR Apache-2.0
//! 4 Git Provider stub 测试 — per OPT-WORKER-01 §3.5
//!
//! 验证 NotImplemented 错误结构 + 字段内容, 防止 stub 静默退化为 NotFound("n/i")。
//! 测试用 mock provider (LocalProvider) 是 Phase F 的真实路径, 4 远端 provider 暂未实装。

use star_sa::provider_bitbucket::BitbucketProvider;
use star_sa::provider_gitea::GiteaProvider;
use star_sa::provider_github::GitHubProvider;
use star_sa::provider_gitlab::GitLabProvider;
use star_sa::{Provider, ProviderError};

#[test]
fn github_get_repo_returns_not_implemented() {
    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    let p = GitHubProvider {
        token: "x".into(),
        base_url: "https://api.github.com".into(),
    };
    let result = rt.block_on(p.get_repo("owner", "name"));
    match result {
        Err(ProviderError::NotImplemented {
            feature,
            suggestion,
        }) => {
            assert_eq!(feature, "github_api_get_repo");
            assert!(
                suggestion.contains("P4-WBS Phase H.2"),
                "suggestion should reference P4 phase: {suggestion}"
            );
        }
        other => panic!("expected NotImplemented, got {other:?}"),
    }
}

#[test]
fn gitlab_get_repo_returns_not_implemented() {
    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    let p = GitLabProvider {
        token: "x".into(),
        base_url: "https://gitlab.com/api/v4".into(),
    };
    let result = rt.block_on(p.get_repo("owner", "name"));
    match result {
        Err(ProviderError::NotImplemented {
            feature,
            suggestion,
        }) => {
            assert_eq!(feature, "gitlab_api_get_repo");
            assert!(suggestion.contains("P4-WBS Phase H.2"));
        }
        other => panic!("expected NotImplemented, got {other:?}"),
    }
}

#[test]
fn bitbucket_get_repo_returns_not_implemented() {
    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    let p = BitbucketProvider {
        token: "x".into(),
        base_url: "https://api.bitbucket.org/2.0".into(),
    };
    let result = rt.block_on(p.get_repo("owner", "name"));
    match result {
        Err(ProviderError::NotImplemented {
            feature,
            suggestion,
        }) => {
            assert_eq!(feature, "bitbucket_api_get_repo");
            assert!(suggestion.contains("P4-WBS Phase H.2"));
        }
        other => panic!("expected NotImplemented, got {other:?}"),
    }
}

#[test]
fn gitea_get_repo_returns_not_implemented() {
    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    let p = GiteaProvider {
        token: "x".into(),
        base_url: "https://gitea.com/api/v1".into(),
    };
    let result = rt.block_on(p.get_repo("owner", "name"));
    match result {
        Err(ProviderError::NotImplemented {
            feature,
            suggestion,
        }) => {
            assert_eq!(feature, "gitea_api_get_repo");
            assert!(suggestion.contains("P4-WBS Phase H.2"));
        }
        other => panic!("expected NotImplemented, got {other:?}"),
    }
}

#[test]
fn provider_error_not_implemented_display() {
    let err = ProviderError::NotImplemented {
        feature: "github_api_get_repo".into(),
        suggestion: "use mock in test".into(),
    };
    let s = err.to_string();
    assert!(s.contains("not_implemented"));
    assert!(s.contains("github_api_get_repo"));
    assert!(s.contains("use mock in test"));
}
