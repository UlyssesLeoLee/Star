/*
@cypher schema=1 source_sha256=0a0e2bb2fa4208a3aa4110231e2d80c4b2024b86ad5da3d676e997a1a1d21a49
MERGE (self:File {path:"crates/sandboxd/src/policy/fs.rs"})
@endcypher
*/
//! policy::fs — 文件系统维隔离策略 (per FR-3.3, 决策点 D-4 已拍板)
//!
//! 策略: **path allowlist 默认** (子代理需要 read worktree + 写 `$WORKTREE/.worktree-cache/`)。
//! - glob pattern 匹配 (`**` 任意深度 / `*` 无分隔符 / `?` 单字符)
//! - 每规则独立读 / 读写模式
//! - BLOCK 模式: 命中非 allowlist 路径 → audit log + EACCES (per FR-3.3)
//!
//! glob matcher 为 crate 内手写实现 (0 新依赖, per 守门 #11), 语义跟
//! 标准 glob (`**`/`*`/`?`) 对齐, 覆盖用例见 `tests/unit/test_policy_fs.rs`。

use serde::{Deserialize, Serialize};

use crate::error::SandboxdError;

/// fs 策略模式 (per BD §3.1 FsPolicy.Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FsMode {
    /// path allowlist (默认 per D-4)
    #[default]
    Allowlist,
    /// 只读根 (除 allowlist RW 外全只读, v0.1 策略层)
    ReadOnlyRoot,
    /// 开放 (不隔离, 审计用)
    Open,
}

/// 单路径规则模式 (per BD §3.1 FsPathRule.Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsPathMode {
    /// 只读
    Read,
    /// 读写
    ReadWrite,
}

/// 单条路径 allowlist 规则 (per BD §3.1 FsPathRule)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsPathRule {
    /// glob pattern (e.g. `/worktree/**`)
    pub path: String,
    /// 路径模式 (Read / ReadWrite)
    pub mode: FsPathMode,
}

impl FsPathRule {
    /// 构造规则
    pub fn new(path: impl Into<String>, mode: FsPathMode) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }
}

/// fs 维策略 (per FR-3.3)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsPolicy {
    /// 模式 (默认 Allowlist per D-4)
    pub mode: FsMode,
    /// path allowlist (glob pattern, per 决策点 D-4)
    pub allowlist: Vec<FsPathRule>,
}

impl Default for FsPolicy {
    fn default() -> Self {
        Self {
            mode: FsMode::Allowlist,
            // per SRS FR-3.3 D-4 默认 allowlist
            allowlist: vec![
                FsPathRule::new("/worktree/**", FsPathMode::Read),
                FsPathRule::new("/worktree/.worktree-cache/**", FsPathMode::ReadWrite),
                FsPathRule::new("/tmp/**", FsPathMode::ReadWrite),
            ],
        }
    }
}

/// fs 维判定结果 (per FR-3.3 BLOCK 模式派生)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsDecision {
    /// 放行读
    ReadAllowed,
    /// 放行读写
    WriteAllowed,
    /// 阻断
    Blocked {
        /// 阻断原因
        reason: String,
    },
}

impl FsPolicy {
    /// 构造 fs 策略
    pub fn new(mode: FsMode, allowlist: Vec<FsPathRule>) -> Self {
        Self { mode, allowlist }
    }

    /// 校验 (空 path 非法)
    pub fn validate(&self) -> Result<(), SandboxdError> {
        for rule in &self.allowlist {
            if rule.path.is_empty() {
                return Err(SandboxdError::InvalidPolicy(
                    "fs allowlist rule path must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// 路径判定: 给定路径 + 是否写访问
    ///
    /// - `Open` → 读写全放行
    /// - `Allowlist` → 找最佳匹配规则 (最长 pattern 优先), 写访问要求 ReadWrite
    /// - `ReadOnlyRoot` → allowlist 内规则照常, allowlist 外读放行 / 写阻断
    pub fn check_path(&self, path: &str, write: bool) -> FsDecision {
        match self.mode {
            FsMode::Open => {
                if write {
                    FsDecision::WriteAllowed
                } else {
                    FsDecision::ReadAllowed
                }
            }
            FsMode::ReadOnlyRoot => match self.best_match(path) {
                Some(rule) if rule.mode == FsPathMode::ReadWrite && write => {
                    FsDecision::WriteAllowed
                }
                Some(_) | None if !write => FsDecision::ReadAllowed,
                Some(_) | None => FsDecision::Blocked {
                    reason: format!("write to '{path}' not in RW allowlist (READ_ONLY_ROOT)"),
                },
            },
            FsMode::Allowlist => match self.best_match(path) {
                Some(rule) if rule.mode == FsPathMode::ReadWrite || !write => {
                    if write && rule.mode == FsPathMode::ReadWrite {
                        FsDecision::WriteAllowed
                    } else {
                        FsDecision::ReadAllowed
                    }
                }
                Some(rule) => FsDecision::Blocked {
                    reason: format!(
                        "write to '{path}' requires READWRITE rule (matched '{}' = READ)",
                        rule.path
                    ),
                },
                None => FsDecision::Blocked {
                    reason: format!("path '{path}' not in fs allowlist"),
                },
            },
        }
    }

    /// 最长 pattern 优先的最佳匹配规则
    fn best_match(&self, path: &str) -> Option<&FsPathRule> {
        self.allowlist
            .iter()
            .filter(|rule| glob_match(&rule.path, path))
            .max_by_key(|rule| rule.path.len())
    }
}

/// glob 匹配: `**` 任意深度 (含 0 段, 可跨分隔符) / `*` 单段内任意 (不跨分隔符) / `?` 单字符
///
/// 路径分隔符跨平台处理: `/` 与 `\` 归一为 `/` (Windows 后端同样受益)。
/// 语义补充 (minimatch 惯例): pattern 以 `/**` 结尾时, 同时匹配父目录本身
/// (e.g. `/worktree/**` 匹配 `/worktree`), 保证目录遍历 (read dir) 不被误判。
/// 最坏情况复杂度指数级, 但策略 pattern / 路径均短 (< 1KB), 实测安全。
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern = pattern.replace('\\', "/");
    let path = path.replace('\\', "/");
    if glob_match_impl(pattern.as_bytes(), path.as_bytes()) {
        return true;
    }
    // 尾随 `/**` → 父目录本身也算命中
    if let Some(parent) = pattern.strip_suffix("/**") {
        return glob_match_impl(parent.as_bytes(), path.as_bytes());
    }
    false
}

fn glob_match_impl(pat: &[u8], text: &[u8]) -> bool {
    if pat.is_empty() {
        return text.is_empty();
    }
    match pat[0] {
        b'*' => {
            // 连续 `*` 折叠; 计数决定单段 (`*`) vs 任意深度 (`**`)
            let mut consumed = 0;
            while consumed < pat.len() && pat[consumed] == b'*' {
                consumed += 1;
            }
            let rest = &pat[consumed..];
            if consumed >= 2 {
                // `**`: 任意长度序列, 可跨 `/`
                (0..=text.len()).any(|k| glob_match_impl(rest, &text[k..]))
            } else {
                // 单 `*`: 仅本段内 (不含 `/`)
                let seg_end = text.iter().position(|&c| c == b'/').unwrap_or(text.len());
                (0..=seg_end).any(|k| glob_match_impl(rest, &text[k..]))
            }
        }
        b'?' => !text.is_empty() && glob_match_impl(&pat[1..], &text[1..]),
        c => !text.is_empty() && text[0] == c && glob_match_impl(&pat[1..], &text[1..]),
    }
}

// ===== proto 互转 (per BD-002 §3.1 FsPolicy / FsPathRule message) =====

impl From<&FsPolicy> for crate::proto::FsPolicy {
    fn from(p: &FsPolicy) -> Self {
        let mode = match p.mode {
            FsMode::Allowlist => crate::proto::fs_policy::Mode::Allowlist as i32,
            FsMode::ReadOnlyRoot => crate::proto::fs_policy::Mode::ReadOnlyRoot as i32,
            FsMode::Open => crate::proto::fs_policy::Mode::Open as i32,
        };
        Self {
            mode,
            allowlist: p.allowlist.iter().map(Into::into).collect(),
        }
    }
}

impl From<&crate::proto::FsPolicy> for FsPolicy {
    fn from(p: &crate::proto::FsPolicy) -> Self {
        use crate::proto::fs_policy::Mode;
        let mode = match Mode::try_from(p.mode).unwrap_or(Mode::Allowlist) {
            Mode::ReadOnlyRoot => FsMode::ReadOnlyRoot,
            Mode::Open => FsMode::Open,
            Mode::Allowlist => FsMode::Allowlist,
        };
        Self {
            mode,
            allowlist: p.allowlist.iter().map(Into::into).collect(),
        }
    }
}

impl From<&FsPathRule> for crate::proto::FsPathRule {
    fn from(r: &FsPathRule) -> Self {
        let mode = match r.mode {
            FsPathMode::Read => crate::proto::fs_path_rule::Mode::Read as i32,
            FsPathMode::ReadWrite => crate::proto::fs_path_rule::Mode::Readwrite as i32,
        };
        Self {
            path: r.path.clone(),
            mode,
        }
    }
}

impl From<&crate::proto::FsPathRule> for FsPathRule {
    fn from(r: &crate::proto::FsPathRule) -> Self {
        use crate::proto::fs_path_rule::Mode;
        let mode = match Mode::try_from(r.mode).unwrap_or(Mode::Read) {
            Mode::Readwrite => FsPathMode::ReadWrite,
            Mode::Read => FsPathMode::Read,
        };
        Self {
            path: r.path.clone(),
            mode,
        }
    }
}
