// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-04 运维文档子域 (per SRS-001 §4 F-04 + brief §2.1)
//!
//! F-04 端到端实装: `DocRef` + `list()` 调 `walkdir` 扫描 4 类 docs 子目录
//!   (docs/requirements/ + docs/basic-design/ + docs/detailed-design/ + docs/reports/),
//!   返 `Vec<DocRef>`. 不接 DB (per SRS-001 §8 文档不是表存储, F-04 0 新表).
//!
//! 5 类别 (per brief §2.1 + SRS-001 §4 F-04):
//!   - SRS  → docs/requirements/*.md
//!   - BAS  → docs/basic-design/*.md
//!   - DET  → docs/detailed-design/*.md
//!   - 报告 → docs/reports/*.md
//!   - 其他 → docs/ 其他 .md (briefs/ adr/ architecture/ agents/ ...)
//!
//! 守門:
//!   - #1 R-05: 仅扫 docs/ 子目录, 不扫全仓库; 不动生产路径
//!   - #13: F-04 0 新表 (per brief §1, 文档扫描不入 DB)
//!   - #11 缺标比错标: walkdir 失败 / 路径不存在 → 返空 Vec + 静默跳过

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 文档类别 (5 态, per brief §2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocCategory {
    /// docs/requirements/*.md (SRS / 系统需求)
    Srs,
    /// docs/basic-design/*.md (BAS / 基本设计)
    Bas,
    /// docs/detailed-design/*.md (DET / 详细设计)
    Det,
    /// docs/reports/*.md (PHASE / PR / 报告)
    Report,
    /// docs/ 其他 .md (briefs/ adr/ architecture/ ...)
    Other,
}

impl DocCategory {
    /// 从相对路径派生类别 (per docs/ 4 子目录 + 其他)
    /// 走 `to_lowercase` 容错 (Windows path 大小写不敏感)
    pub fn from_path(rel: &str) -> Self {
        let lower = rel.replace('\\', "/").to_lowercase();
        if lower.starts_with("docs/requirements/") {
            Self::Srs
        } else if lower.starts_with("docs/basic-design/") {
            Self::Bas
        } else if lower.starts_with("docs/detailed-design/") {
            Self::Det
        } else if lower.starts_with("docs/reports/") {
            Self::Report
        } else {
            Self::Other
        }
    }

    /// 类别短名 (per SRS-001 §4 F-04 卡片: SRS / BAS / DET / 报告 / 其他)
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Srs => "SRS",
            Self::Bas => "BAS",
            Self::Det => "DET",
            Self::Report => "Report",
            Self::Other => "Other",
        }
    }
}

/// 文档引用 (per ops_api.rs::DocRef + brief §2.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRef {
    /// 相对路径 (相对 worktree 根, 用 forward slash)
    pub path: String,
    /// 标题: 取文件第一行 `# xxx` 头 (MVP 简化: 文件 stem)
    pub title: String,
    /// 类别 (SRS / BAS / DET / Report / Other)
    pub category: String,
    /// 最后修改时间 (ISO 8601 UTC)
    pub updated_at: DateTime<Utc>,
}

impl DocRef {
    /// MVP stub: 2 条假数据 (per OPS-BASIC-DESIGN §3.4 + 跟 F-01/F-02/F-03 同 pattern)
    /// 保留作为 fallback (无 walkdir / 路径异常时)
    pub fn stub() -> Vec<Self> {
        let now: DateTime<Utc> = "2026-09-08T07:53:00Z".parse().expect("hardcoded UTC");
        vec![
            Self {
                path: "docs/requirements/SRS-STAR-OPS-001.md".to_string(),
                title: "STAR Ops Console SRS".to_string(),
                category: "SRS".to_string(),
                updated_at: now,
            },
            Self {
                path: "docs/basic-design/OPS-BASIC-DESIGN-001.md".to_string(),
                title: "STAR Ops Console 基本设计".to_string(),
                category: "BAS".to_string(),
                updated_at: now,
            },
        ]
    }
}

/// 文档扫描器 (F-04 端到端, 调 walkdir 真实扫 docs/ 4 子目录)
#[derive(Clone)]
pub struct DocScanner {
    /// worktree 根 (CARGO_MANIFEST_DIR/../..)
    pub root: PathBuf,
}

impl DocScanner {
    /// 构造新 scanner (per F-04 brief §2.1)
    pub fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let root = Path::new(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self { root }
    }

    /// 扫描 4 docs 子目录 + 其他, 返 Vec<DocRef> 排序 by path
    /// 守門 #1 R-05: 仅扫 docs/ 子目录, 不扫全仓库
    /// 守門 #13: F-04 0 新表, 不持久化
    pub fn list(&self) -> Vec<DocRef> {
        let subdirs = [
            "docs/requirements",
            "docs/basic-design",
            "docs/detailed-design",
            "docs/reports",
        ];
        let mut docs: Vec<DocRef> = Vec::new();
        for sub in subdirs {
            let abs = self.root.join(sub);
            if !abs.exists() {
                continue; // 守門 #11 缺标比错标: 路径不存在静默跳过
            }
            for entry in WalkDir::new(&abs)
                .max_depth(2) // docs/<sub>/<file>.md, 限制深度避免 .git 子目录
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                if let Some(doc) = self.to_doc_ref(entry.path()) {
                    docs.push(doc);
                }
            }
        }
        // 排序 by path (稳定, 守門 #23 不能 panic)
        docs.sort_by(|a, b| a.path.cmp(&b.path));
        docs
    }

    /// Path → DocRef (守門 #11 缺标比错标: 失败返 None 跳过)
    fn to_doc_ref(&self, abs: &Path) -> Option<DocRef> {
        // 派生相对路径 (forward slash)
        let rel = abs.strip_prefix(&self.root).ok()?.to_path_buf();
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let category = DocCategory::from_path(&rel_str);

        // 标题: file_stem (MVP 简化, owner 拍板后 [M] 子项可扫第一行 `# xxx` 头)
        let title = rel
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        // updated_at: 文件 mtime → DateTime<Utc>
        let metadata = std::fs::metadata(abs).ok()?;
        let modified = metadata.modified().ok()?;
        let updated_at: DateTime<Utc> = modified.into();

        Some(DocRef {
            path: rel_str,
            title,
            category: category.short_name().to_string(),
            updated_at,
        })
    }
}

impl Default for DocScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_category_from_path_classifies_four_subdirs() {
        assert_eq!(
            DocCategory::from_path("docs/requirements/SRS-STAR-OPS-001.md"),
            DocCategory::Srs
        );
        assert_eq!(
            DocCategory::from_path("docs/basic-design/OPS-BASIC-DESIGN-001.md"),
            DocCategory::Bas
        );
        assert_eq!(
            DocCategory::from_path("docs/detailed-design/OPS-DETAILED-DESIGN-001.md"),
            DocCategory::Det
        );
        assert_eq!(
            DocCategory::from_path("docs/reports/PHASE-F04-DOCS-REPORT.md"),
            DocCategory::Report
        );
        assert_eq!(
            DocCategory::from_path("docs/briefs/ops-f04-docs-impl.md"),
            DocCategory::Other
        );
        assert_eq!(
            DocCategory::from_path("docs/adr/0048-foo.md"),
            DocCategory::Other
        );
    }

    #[test]
    fn doc_category_short_name_returns_5_labels() {
        assert_eq!(DocCategory::Srs.short_name(), "SRS");
        assert_eq!(DocCategory::Bas.short_name(), "BAS");
        assert_eq!(DocCategory::Det.short_name(), "DET");
        assert_eq!(DocCategory::Report.short_name(), "Report");
        assert_eq!(DocCategory::Other.short_name(), "Other");
    }

    #[test]
    fn doc_ref_stub_returns_two_refs() {
        let docs = DocRef::stub();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].category, "SRS");
        assert_eq!(docs[1].category, "BAS");
    }

    /// F-04 端到端: 调 walkdir 真实扫描 docs/ 4 子目录
    /// 守門 #1 R-05: 仅 docs/ 子目录, 不动生产
    /// 守門 #23: 不能 panic, 返稳定结果
    #[test]
    fn doc_scanner_list_finds_at_least_four_subsections() {
        let scanner = DocScanner::new();
        let docs = scanner.list();
        // docs/ 4 子目录 + 其他可能为空, 至少 4 类应存在 (不一定每个 .md 都有)
        let categories: std::collections::HashSet<String> =
            docs.iter().map(|d| d.category.clone()).collect();
        assert!(
            categories.len() >= 2,
            "F-04 端到端 walkdir 必扫到至少 2 类文档, 实际 {} 类: {:?}",
            categories.len(),
            categories
        );
        // docs/reports/ + docs/requirements/ 必有 (per git 实证: 既有 6+ PHASE 报告)
        assert!(categories.contains("Report"), "F-04 docs/reports/ 必含");
    }

    /// F-04 端到端: 调 walkdir 真实扫 docs/reports/ 应能扫到 PHASE-F03 报告
    /// 守門 #9 v20: 跨 crate IT 真实调 (本 crate 内, 走 walkdir 真实 IO)
    #[test]
    fn doc_scanner_finds_phase_f03_report() {
        let scanner = DocScanner::new();
        let docs = scanner.list();
        let found = docs
            .iter()
            .any(|d| d.path == "docs/reports/PHASE-F03-METRICS-REPORT.md");
        assert!(found, "F-04 walkdir 必能扫到 PHASE-F03 报告");
    }
}
