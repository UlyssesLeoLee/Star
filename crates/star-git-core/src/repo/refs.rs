/*
@cypher schema=1 source_sha256=0e7db93ad8e24883a1b92a3d17f14583f878477106343a5c43b8cbf6a5bc91c0
MERGE (self:File {path:"crates/star-git-core/src/repo/refs.rs"})
MERGE (read_head:Symbol {id:"crates/star-git-core/src/repo/refs.rs::read_head",kind:"function"})
MERGE (read_ref:Symbol {id:"crates/star-git-core/src/repo/refs.rs::read_ref",kind:"function"})
MERGE (write_ref:Symbol {id:"crates/star-git-core/src/repo/refs.rs::write_ref",kind:"function"})
MERGE (self)-[:DEFINES]->(read_head)
MERGE (self)-[:DEFINES]->(read_ref)
MERGE (self)-[:DEFINES]->(write_ref)
@endcypher
*/
//! Ref read/write helpers for a bare repo.
//! 移植自 GitGit D:\GitGit\src\repo\refs.rs @ 69770e6 (2026-11-14, AGPL-3.0 —
//! 仅移植保留, Star 侧不做代码并入)
//!
//! Currently a stub. The smart-HTTP handlers do **not** read refs directly:
//! the `git upload-pack --advertise-refs` and `git receive-pack --advertise-refs`
//! subprocesses produce the canonical ref list. The helpers below are kept
//! for future debug / diagnostics endpoints and are intentionally a no-op
//! for the MVP to avoid surface area.

use std::path::Path;

use crate::error::{Result, StarGitError};

/// Read the symbolic target of `HEAD` in a bare repo (e.g. `refs/heads/main`).
#[allow(dead_code)]
pub fn read_head(repo: &Path) -> Result<String> {
    let head = repo.join("HEAD");
    let raw = std::fs::read_to_string(&head)?;
    let trimmed = raw.trim_end();
    let after = trimmed
        .strip_prefix("ref:")
        .ok_or_else(|| StarGitError::Http(format!("HEAD is detached: {trimmed}")))?;
    Ok(after.trim().to_string())
}

/// Resolve a ref like `main` or `refs/heads/main` to its full ref name and
/// return the resolved value if it exists.
#[allow(dead_code)]
pub fn read_ref(repo: &Path, name: &str) -> Result<Option<String>> {
    let full = if name.starts_with("refs/") {
        name.to_string()
    } else {
        format!("refs/heads/{name}")
    };
    let path = repo.join(&full);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(Some(raw.trim().to_string()))
}

/// Write a ref to a bare repo. Used by tests / diagnostics; not on the
/// receive-pack hot path (that goes through `git receive-pack`).
#[allow(dead_code)]
pub fn write_ref(repo: &Path, full_ref: &str, sha: &str) -> Result<()> {
    if !full_ref.starts_with("refs/") {
        return Err(StarGitError::Http(format!("not a ref: {full_ref}")));
    }
    let path = repo.join(full_ref);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, format!("{sha}\n"))?;
    Ok(())
}


/// List all branch names (refs/heads/*) in a bare repository.
pub fn list_branches(repo: &Path) -> Result<Vec<String>> {
    let heads = repo.join("refs").join("heads");
    if !heads.is_dir() {
        return Ok(Vec::new());
    }
    let mut branches = Vec::new();
    for entry in std::fs::read_dir(&heads)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                branches.push(name.to_string());
            }
        } else if path.is_dir() {
            // Recurse into nested directories (e.g., refs/heads/feature/xyz)
            let nested = list_branches_nested(&path, &heads)?;
            branches.extend(nested);
        }
    }
    branches.sort();
    Ok(branches)
}

fn list_branches_nested(base: &Path, heads_root: &Path) -> Result<Vec<String>> {
    let mut branches = Vec::new();
    for entry in std::fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(_name) = path.file_name().and_then(|n| n.to_str()) {
                // Build full branch name relative to refs/heads/
                if let Ok(rel) = path.strip_prefix(&heads_root) {
                    if let Some(s) = rel.to_str() {
                        branches.push(s.replace("\\", "/"));
                    }
                }
            }
        } else if path.is_dir() {
            let nested = list_branches_nested(&path, heads_root)?;
            branches.extend(nested);
        }
    }
    Ok(branches)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn unique_temp(label: &str) -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!(
            "gitgit-test-{label}-{pid}-{n}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn lay_bare_with_head(dir: &Path, head_target: &str) {
        std::fs::write(dir.join("HEAD"), format!("ref: {head_target}
")).unwrap();
    }



    #[test]
    fn read_head_returns_symbolic_target() {
        let dir = unique_temp("refs-head");
        lay_bare_with_head(&dir, "refs/heads/main");
        let got = read_head(&dir).unwrap();
        assert_eq!(got, "refs/heads/main");
    }



    #[test]
    fn read_head_rejects_detached() {
        let dir = unique_temp("refs-detached");
        std::fs::write(
            dir.join("HEAD"),
            b"0123456789abcdef0123456789abcdef01234567\n",
        )
        .unwrap();
        let err = read_head(&dir).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("HEAD is detached"), "got: {msg}");
    }



    #[test]
    fn read_ref_resolves_short_name() {
        let dir = unique_temp("refs-short");
        lay_bare_with_head(&dir, "refs/heads/main");
        let sha = "1111111111111111111111111111111111111111";
        write_ref(&dir, "refs/heads/main", sha).unwrap();
        let got = read_ref(&dir, "main").unwrap();
        assert_eq!(got.as_deref(), Some(sha));
    }



    #[test]
    fn read_ref_passes_through_full_name() {
        let dir = unique_temp("refs-full");
        lay_bare_with_head(&dir, "refs/heads/main");
        let sha = "2222222222222222222222222222222222222222";
        write_ref(&dir, "refs/heads/feature/x", sha).unwrap();
        let got = read_ref(&dir, "refs/heads/feature/x").unwrap();
        assert_eq!(got.as_deref(), Some(sha));
    }



    #[test]
    fn read_ref_missing_returns_none() {
        let dir = unique_temp("refs-missing");
        lay_bare_with_head(&dir, "refs/heads/main");
        let got = read_ref(&dir, "nope").unwrap();
        assert!(got.is_none());
    }



    #[test]
    fn write_ref_rejects_non_refs_prefix() {
        let dir = unique_temp("refs-bad");
        lay_bare_with_head(&dir, "refs/heads/main");
        let err = write_ref(&dir, "notrefs/x", "abc").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("not a ref"), "got: {msg}");
    }



    #[test]
    fn write_ref_creates_nested_dirs() {
        let dir = unique_temp("refs-nested");
        lay_bare_with_head(&dir, "refs/heads/main");
        let sha = "3333333333333333333333333333333333333333";
        write_ref(&dir, "refs/heads/feature/deep/nested", sha).unwrap();
        // File must now be readable on disk via read_ref.
        let got = read_ref(&dir, "refs/heads/feature/deep/nested").unwrap();
        assert_eq!(got.as_deref(), Some(sha));
    }



    #[test]
    fn list_branches_single() {
        let dir = unique_temp("list-single");
        lay_bare_with_head(&dir, "refs/heads/main");
        std::fs::create_dir_all(dir.join("refs").join("heads")).unwrap();
        std::fs::write(dir.join("refs").join("heads").join("main"), "abc123
").unwrap();
        let branches = list_branches(&dir).unwrap();
        assert_eq!(branches, vec!["main"]);
    }



    #[test]
    fn list_branches_multiple() {
        let dir = unique_temp("list-multi");
        lay_bare_with_head(&dir, "refs/heads/main");
        let heads = dir.join("refs").join("heads");
        std::fs::create_dir_all(&heads).unwrap();
        std::fs::write(heads.join("main"), "abc123
").unwrap();
        std::fs::write(heads.join("develop"), "def456
").unwrap();
        std::fs::create_dir_all(heads.join("feature")).unwrap();
        std::fs::write(heads.join("feature").join("xyz"), "ghi789
").unwrap();
        let branches = list_branches(&dir).unwrap();
        assert_eq!(branches, vec!["develop", "feature/xyz", "main"]);
    

}

}
