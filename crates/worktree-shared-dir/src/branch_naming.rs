//! `branch_naming.rs` — ULYS-158.5 / FR-ORCA-010 (Branch 命名优先级链).
//!
//! Per `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-010 +
//! ULYS-218 (ULYS-158.5) description §1.2:
//!
//! 优先级链 (Linear > GitHub PR > user input > workspace derived):
//! 1. `Linear` 提供的 branch 名 (e.g. `leedx/ULYS-218-pick-foundation`)
//! 2. `GitHub PR` 关联分支 (e.g. `feature/ulys-218`)
//! 3. 用户 Advanced drawer 输入 (e.g. `feat/my-work`)
//! 4. `workspace name` 派生 (e.g. `🚀 rocket-12345` → `rocket-12345`)
//!
//! ## 后处理
//!
//! - emoji shortcode 替换: `🚀` → `rocket`, 😀 → `grinning` (per spec §3
//!   行 230-232; 只覆盖 GitHub 官方 emoji shortcodes 集中常见的几个, 其它
//!   保留原 emoji 让 UI 显示, 不强行替换以免破坏 hash)
//! - 字符集规整: `[a-z0-9-_/]` 之外字符替换成 `-`
//! - 长度限制: 64 字符 (per git branch 最佳实践 + GitHub UI 截断点)
//! - trim 末尾 `-` + 合并连续 `-` 为单个
//!
//! ## 不依赖 Multica CLI
//!
//! 同 ULYS-177 路径 C: 本模块纯函数, 不接 Linear/GitHub API. 调用方
//! (REST/CLI/caller service) 负责填入 `BranchNamingInput` 的前 3 档.
//!
//! ## 测试
//!
//! 单元测试覆盖: 优先级链 4 档切换 / emoji shortcode / 长度截断 /
//! 字符集规整 / 空 fallback / 已有分支名校验.

use serde::{Deserialize, Serialize};

use crate::error::{SharedDirError, SharedDirResult};

// =====================================================================
// 输入/输出类型 (per brief §2.2)
// =====================================================================

/// Branch 命名输入 — 4 档优先级.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchNamingInput {
    /// 优先级 #1: Linear 提供的 branch 名 (e.g. `leedx/ULYS-218-...`).
    pub linear_branch_name: Option<String>,
    /// 优先级 #2: GitHub PR 关联分支 (e.g. `feature/ulys-218`).
    pub github_pr_branch: Option<String>,
    /// 优先级 #3: 用户 Advanced drawer 输入 (e.g. `feat/my-work`).
    pub user_input: Option<String>,
    /// 优先级 #4: workspace name 派生基础 (e.g. `🚀 rocket-12345`).
    pub workspace_name: String,
}

/// Branch 命名结果 + 来源.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchNamingResult {
    /// 最终 branch 名 (经过 shortcode + 长度 + 字符集后处理).
    pub branch_name: String,
    /// 来源 (per FR-ORCA-010 优先级链).
    pub source: BranchNamingSource,
}

/// Branch 名来源 (per FR-ORCA-010 优先级链).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchNamingSource {
    /// 优先级 #1: Linear 提供.
    Linear,
    /// 优先级 #2: GitHub PR 关联.
    GitHubPr,
    /// 优先级 #3: 用户输入.
    UserInput,
    /// 优先级 #4: workspace name 派生.
    WorkspaceDerived,
}

// =====================================================================
// Trait (per brief §2.2)
// =====================================================================

/// `BranchNamer` — 优先级链 + 后处理服务 (per FR-ORCA-010).
///
/// 默认实现 `DefaultBranchNamer` 覆盖 spec §3 行 230-232 的 emoji
/// shortcode 替换 + 64 字符截断 + `[a-z0-9-_/]` 字符集规整.
pub trait BranchNamer: Send + Sync {
    /// 给定输入, 派生 branch 名 + 来源.
    fn derive(&self, input: &BranchNamingInput) -> SharedDirResult<BranchNamingResult>;
}

// =====================================================================
// 常量
// =====================================================================

/// Branch 名最大长度 (per git branch 最佳实践 + GitHub UI 截断点).
pub const MAX_BRANCH_LENGTH: usize = 64;

// =====================================================================
// Emoji shortcode 表 (per spec §3 行 230-232)
// =====================================================================

/// GitHub 官方 emoji shortcode 表 (per spec §3 行 230-232).
///
/// 完整表见 <https://github.com/ikatyang/emoji-cheat-sheet>; 本表只覆盖
/// workspace 名称常见 emoji. 任何不在表内的 emoji **保留原字符** — 让
/// UI 显示, 不强行替换以免破坏分支名 hash.
pub const EMOJI_SHORTCODE_TABLE: &[(&str, &str)] = &[
    ("🚀", "rocket"),
    ("😀", "grinning"),
    ("😎", "sunglasses"),
    ("🐛", "bug"),
    ("✨", "sparkles"),
    ("🔥", "fire"),
    ("📦", "package"),
    ("🔧", "wrench"),
    ("📝", "memo"),
    ("🎉", "tada"),
    ("⚡", "zap"),
    ("🌟", "star2"),
    ("🍱", "bento"),
    ("🚧", "construction"),
    ("💡", "bulb"),
    ("🔖", "bookmark"),
];

// =====================================================================
// Pure helpers (testable)
// =====================================================================

/// 从 raw string 中替换 emoji 为 shortcode (per spec §3 行 230-232).
///
/// 任何不在 `EMOJI_SHORTCODE_TABLE` 的 emoji **保留原字符** (utf-8 字节
/// 序列原样通过). 这与 Orca v1.0 spec 一致 — 只有"workspace 名含 emoji"
/// 才走派生分支, 此时常用 emoji 在表内.
pub fn replace_emoji_shortcodes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        // 尝试匹配 4-byte UTF-8 emoji + 后续 VS16 (FE0F) 之类修饰符
        let mut matched = false;
        for (emoji, shortcode) in EMOJI_SHORTCODE_TABLE {
            if emoji.chars().next() == Some(c) {
                out.push_str(shortcode);
                matched = true;
                break;
            }
        }
        if !matched {
            out.push(c);
        }
    }
    out
}

/// 字符集规整: `[a-z0-9-_/]` 之外字符替换成 `-`, 合并连续 `-`,
/// trim 首尾 `-`.
///
/// 注意: `/` 保留 — branch 名允许 `/` 分隔 owner (e.g. `feature/x`).
pub fn sanitize_branch_chars(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_dash = false;
    let mut started = false; // 控制 trim 末尾
    for c in input.chars() {
        let valid = c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/';
        if valid {
            out.push(c);
            last_was_dash = c == '-';
            started = true;
        } else if started {
            // 合并连续 `-`
            if !last_was_dash {
                out.push('-');
                last_was_dash = true;
            }
        }
        // trim 首字符: started=false 时 invalid 字符直接丢弃
    }
    // trim 末尾 `-`
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// 长度截断到 `MAX_BRANCH_LENGTH` (默认 64).
///
/// 截断策略: 在最后一个 `/` 之后截断到 64 字符, 保证 prefix 完整; 没有
/// `/` 时直接截断. 这样 `feature/very-long-branch-name` 截断后
/// `feature/<...>` 形式, 而非 `featur` 这种半截前缀.
pub fn truncate_branch(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }
    // 找最后一个 `/`, 优先保留前缀完整
    let bytes = input.as_bytes();
    let cut_at = bytes[..max_len]
        .iter()
        .rposition(|&b| b == b'/')
        .map(|i| i + 1)
        .unwrap_or(max_len);
    input[..cut_at].to_string()
}

/// 把 raw input 转成 sanitize + truncate 后字符串.
///
/// 组合 `replace_emoji_shortcodes` → `sanitize_branch_chars` →
/// `truncate_branch`. 当 `sanitize_branch_chars` 输出空 (e.g. workspace
/// 名全是 emoji 但都不在表内) 时回退到 raw input 的 ASCII 摘要.
pub fn normalize_branch(input: &str) -> String {
    let shortcoded = replace_emoji_shortcodes(input);
    let sanitized = sanitize_branch_chars(&shortcoded);
    if sanitized.is_empty() {
        // fallback: 用 raw input 的第一个 ASCII 字符 + hash
        let mut fallback = String::from("branch-");
        for c in input.chars() {
            if c.is_ascii_alphanumeric() {
                fallback.push(c.to_ascii_lowercase());
                if fallback.len() >= 16 {
                    break;
                }
            }
        }
        return truncate_branch(&fallback, MAX_BRANCH_LENGTH);
    }
    truncate_branch(&sanitized, MAX_BRANCH_LENGTH)
}

// =====================================================================
// Default 实现
// =====================================================================

/// `DefaultBranchNamer` — 标准实现, 严格按 spec §3 优先级链.
#[derive(Debug, Clone, Default)]
pub struct DefaultBranchNamer;

impl BranchNamer for DefaultBranchNamer {
    fn derive(&self, input: &BranchNamingInput) -> SharedDirResult<BranchNamingResult> {
        // 优先级 #1: linear
        if let Some(name) = input.linear_branch_name.as_deref() {
            let normalized = normalize_branch(name);
            if !normalized.is_empty() {
                return Ok(BranchNamingResult {
                    branch_name: normalized,
                    source: BranchNamingSource::Linear,
                });
            }
        }
        // 优先级 #2: github_pr
        if let Some(name) = input.github_pr_branch.as_deref() {
            let normalized = normalize_branch(name);
            if !normalized.is_empty() {
                return Ok(BranchNamingResult {
                    branch_name: normalized,
                    source: BranchNamingSource::GitHubPr,
                });
            }
        }
        // 优先级 #3: user_input
        if let Some(name) = input.user_input.as_deref() {
            let normalized = normalize_branch(name);
            if !normalized.is_empty() {
                return Ok(BranchNamingResult {
                    branch_name: normalized,
                    source: BranchNamingSource::UserInput,
                });
            }
        }
        // 优先级 #4: workspace_name 派生
        let derived = normalize_branch(&input.workspace_name);
        if derived.is_empty() {
            return Err(SharedDirError::new(
                "WSD.BRANCH_NAME_DERIVE_FAIL",
                format!(
                    "cannot derive branch from workspace_name={:?}: empty after sanitize",
                    input.workspace_name
                ),
                "wsd-branch-naming-default-trace",
            ));
        }
        Ok(BranchNamingResult {
            branch_name: derived,
            source: BranchNamingSource::WorkspaceDerived,
        })
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- emoji shortcode 表 ----

    #[test]
    fn replace_emoji_rocket_to_rocket_shortcode() {
        assert_eq!(replace_emoji_shortcodes("🚀 deploy"), "rocket deploy");
    }

    #[test]
    fn replace_emoji_grinning_to_shortcode() {
        assert_eq!(replace_emoji_shortcodes("😀 fast"), "grinning fast");
    }

    #[test]
    fn replace_emoji_unknown_preserved() {
        // 不在表内的 emoji (e.g. 🥑) 保留原字符
        assert_eq!(
            replace_emoji_shortcodes("🥑 avocado"),
            "🥑 avocado"
        );
    }

    #[test]
    fn replace_emoji_multiple_in_one_string() {
        assert_eq!(
            replace_emoji_shortcodes("🚀 fire 🔥"),
            "rocket fire fire"
        );
    }

    #[test]
    fn replace_emoji_passthrough_ascii() {
        assert_eq!(replace_emoji_shortcodes("plain ascii"), "plain ascii");
    }

    // ---- 字符集规整 ----

    #[test]
    fn sanitize_lowercases_preserved_via_input() {
        // sanitize 不做 lowercase (Orca v1.0 不要求, branch 大小写不敏感
        // 但保留原样便于人工读)
        assert_eq!(sanitize_branch_chars("Feature-X"), "Feature-X");
    }

    #[test]
    fn sanitize_replaces_invalid_chars_with_dash() {
        assert_eq!(
            sanitize_branch_chars("feat my work"),
            "feat-my-work"
        );
    }

    #[test]
    fn sanitize_collapses_consecutive_dashes() {
        assert_eq!(
            sanitize_branch_chars("feat---x---y"),
            "feat-x-y"
        );
    }

    #[test]
    fn sanitize_trims_leading_and_trailing_dashes() {
        assert_eq!(sanitize_branch_chars("---feat-x---"), "feat-x");
    }

    #[test]
    fn sanitize_preserves_slash() {
        assert_eq!(sanitize_branch_chars("feature/x"), "feature/x");
    }

    #[test]
    fn sanitize_all_invalid_returns_empty() {
        assert_eq!(sanitize_branch_chars("🚀 😀 ✨"), "");
    }

    // ---- 长度截断 ----

    #[test]
    fn truncate_under_limit_passthrough() {
        assert_eq!(truncate_branch("feat-x", MAX_BRANCH_LENGTH), "feat-x");
    }

    #[test]
    fn truncate_over_limit_cuts_at_last_slash() {
        let long = format!("feature/{}", "a".repeat(100));
        let truncated = truncate_branch(&long, MAX_BRANCH_LENGTH);
        assert!(truncated.len() <= MAX_BRANCH_LENGTH);
        assert!(truncated.starts_with("feature/"));
    }

    #[test]
    fn truncate_over_limit_no_slash_cuts_at_max() {
        let long = "a".repeat(100);
        let truncated = truncate_branch(&long, MAX_BRANCH_LENGTH);
        assert_eq!(truncated.len(), MAX_BRANCH_LENGTH);
    }

    // ---- normalize 组合 ----

    #[test]
    fn normalize_emoji_workspace_name_rocket() {
        // per spec §3 行 230-232: 🚀 → rocket
        assert_eq!(normalize_branch("🚀 deploy"), "rocket-deploy");
    }

    #[test]
    fn normalize_lowercase_already() {
        assert_eq!(normalize_branch("feat-x"), "feat-x");
    }

    #[test]
    fn normalize_handles_spaces() {
        assert_eq!(normalize_branch("my new feature"), "my-new-feature");
    }

    #[test]
    fn normalize_trims_length() {
        let long_input = format!("feat-{}", "x".repeat(100));
        let normalized = normalize_branch(&long_input);
        assert!(normalized.len() <= MAX_BRANCH_LENGTH);
    }

    #[test]
    fn normalize_empty_fallback_to_ascii() {
        // 全部 emoji 不在表内 → fallback "branch-" + ASCII 摘要
        let normalized = normalize_branch("🥑🥑🥑");
        assert!(normalized.starts_with("branch-"));
    }

    // ---- 优先级链 ----

    #[test]
    fn derive_priority_linear_wins() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: Some("linear/ULYS-218".into()),
            github_pr_branch: Some("gh-pr".into()),
            user_input: Some("user-input".into()),
            workspace_name: "🚀".into(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.branch_name, "linear/ULYS-218");
        assert_eq!(r.source, BranchNamingSource::Linear);
    }

    #[test]
    fn derive_priority_github_when_no_linear() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: Some("feature/ulys-218".into()),
            user_input: Some("user-input".into()),
            workspace_name: "🚀".into(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.branch_name, "feature/ulys-218");
        assert_eq!(r.source, BranchNamingSource::GitHubPr);
    }

    #[test]
    fn derive_priority_user_when_no_linear_no_github() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: None,
            user_input: Some("feat/my-work".into()),
            workspace_name: "🚀".into(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.branch_name, "feat/my-work");
        assert_eq!(r.source, BranchNamingSource::UserInput);
    }

    #[test]
    fn derive_falls_back_to_workspace_derived() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: None,
            user_input: None,
            workspace_name: "🚀 rocket-launch".into(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.branch_name, "rocket-launch");
        assert_eq!(r.source, BranchNamingSource::WorkspaceDerived);
    }

    #[test]
    fn derive_empty_user_input_falls_through() {
        // user_input = "" 应该跳过, 不当作有效名
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: None,
            user_input: Some(String::new()),
            workspace_name: "🚀".into(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.source, BranchNamingSource::WorkspaceDerived);
    }

    #[test]
    fn derive_all_empty_inputs_errors() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: None,
            user_input: None,
            workspace_name: "🥑🥑🥑".into(), // 全是不在表的 emoji
        };
        // sanitize 出空 → fallback "branch-" + ASCII → 不空, 走 WorkspaceDerived
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.source, BranchNamingSource::WorkspaceDerived);
        assert!(r.branch_name.starts_with("branch-"));
    }

    #[test]
    fn derive_all_empty_inputs_truly_empty_errors() {
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: None,
            github_pr_branch: None,
            user_input: None,
            workspace_name: String::new(),
        };
        let err = namer.derive(&input).unwrap_err();
        assert_eq!(err.code, "WSD.BRANCH_NAME_DERIVE_FAIL");
    }

    #[test]
    fn derive_preserves_owner_slash_for_linear() {
        // Linear 习惯 `owner/name`, sanitize 应保留
        let namer = DefaultBranchNamer;
        let input = BranchNamingInput {
            linear_branch_name: Some("leedx/feat-x".into()),
            github_pr_branch: None,
            user_input: None,
            workspace_name: String::new(),
        };
        let r = namer.derive(&input).unwrap();
        assert_eq!(r.branch_name, "leedx/feat-x");
    }
}
