//! Git adapter types — git-adapter
//!
//! Per DD-WORKTREE-CANVAS-001 §10:
//! - `RepositoryPath` / `BranchName` / `CommitHash` / `GitRef` 4 类型

use serde::{Deserialize, Serialize};

/// Repository path (per DD §10 `RepositoryNode.path`)
pub type RepositoryPath = std::path::PathBuf;

/// Branch name (per DD §10 `BranchNode.name`)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName(pub String);

impl BranchName {
    /// Create a new branch name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for BranchName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for BranchName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Commit hash (per DD §10 `CommitNode.hash`)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitHash(pub String);

impl CommitHash {
    /// Create a new commit hash.
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CommitHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CommitHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Git reference (branch/tag/commit)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "name")]
pub enum GitRef {
    /// Branch reference
    Branch(BranchName),
    /// Tag reference
    Tag(String),
    /// Commit reference
    Commit(CommitHash),
}

impl GitRef {
    /// Get the name as string.
    pub fn name(&self) -> &str {
        match self {
            GitRef::Branch(b) => b.as_str(),
            GitRef::Tag(t) => t,
            GitRef::Commit(c) => c.as_str(),
        }
    }
}

impl From<BranchName> for GitRef {
    fn from(b: BranchName) -> Self {
        GitRef::Branch(b)
    }
}

impl From<CommitHash> for GitRef {
    fn from(c: CommitHash) -> Self {
        GitRef::Commit(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_name_new() {
        let name = BranchName::new("main");
        assert_eq!(name.as_str(), "main");
    }

    #[test]
    fn commit_hash_new() {
        let hash = CommitHash::new("abc123");
        assert_eq!(hash.as_str(), "abc123");
    }

    #[test]
    fn git_ref_branch() {
        let r: GitRef = BranchName::new("feature").into();
        assert!(matches!(r, GitRef::Branch(_)));
    }
}
