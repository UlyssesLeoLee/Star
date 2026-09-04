//! crates/star-treesitter/src/symbol_resolver.rs
//!
//! H.7 Tree-sitter Symbol Resolver 跨文件引用追踪 (per P4-H.7, 守门 #19 [P] 拍板)
//! per `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` §1.4
//!
//! 关键不变量 (per §1.4):
//! - INV-SR-01: 跨文件 symbol 引用必须可解析 (Foo::bar / module::Type)
//! - INV-SR-02: 解析失败返 None, 不 panic
//! - INV-SR-03: 引用关系有向图: source -> target (target 可能不存在于已知 symbols)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::{ParseResult, Symbol, SymbolKind};

#[derive(Debug, Error)]
pub enum SymbolResolverError {
    #[error("reference parse error: {0}")]
    ReferenceParseError(String),
    #[error("symbol not found: {0}")]
    SymbolNotFound(String),
}

/// 符号引用 (parsed from source)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolReference {
    pub raw: String,        // e.g. "domain_tenant::PlayerService::register"
    pub parts: Vec<String>, // split: ["domain_tenant", "PlayerService", "register"]
    pub line: usize,
    pub column: usize,
}

impl SymbolReference {
    /// 解析 "foo::bar::baz" -> ["foo", "bar", "baz"]
    pub fn parse(raw: &str) -> Self {
        let parts: Vec<String> = raw.split("::").map(|s| s.to_string()).collect();
        Self {
            raw: raw.to_string(),
            parts,
            line: 0,
            column: 0,
        }
    }

    /// 解析 + 行号 + 列号
    pub fn parse_at(raw: &str, line: usize, column: usize) -> Self {
        let mut r = Self::parse(raw);
        r.line = line;
        r.column = column;
        r
    }
}

/// 引用关系 (source -> target)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceEdge {
    pub source_file: String,
    pub source_ref: SymbolReference,
    pub target_file: String, // 推测的目标文件 (e.g. "domain_tenant.rs" for "domain_tenant::Foo")
    pub target_name: String, // 推测的目标 symbol
    pub resolved: bool,      // 是否在已知 symbols 中找到
}

/// Symbol index (跨文件 symbol 表)
pub struct SymbolIndex {
    /// file_name -> (symbol_name -> Symbol)
    symbols_by_file: HashMap<String, HashMap<String, Symbol>>,
    /// global symbol_name -> [file_name]
    symbols_by_name: HashMap<String, Vec<String>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols_by_file: HashMap::new(),
            symbols_by_name: HashMap::new(),
        }
    }

    /// 添加文件的 parse result 到 index
    pub fn add_file(&mut self, file_name: &str, result: &ParseResult) {
        let file_symbols = self
            .symbols_by_file
            .entry(file_name.to_string())
            .or_default();
        for symbol in &result.symbols {
            file_symbols.insert(symbol.name.clone(), symbol.clone());
            self.symbols_by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(file_name.to_string());
        }
    }

    /// 查找 symbol (按 file + name)
    pub fn lookup(&self, file_name: &str, name: &str) -> Option<&Symbol> {
        self.symbols_by_file.get(file_name)?.get(name)
    }

    /// 查找 symbol (按 name, 返回所有 file 命中)
    pub fn lookup_global(&self, name: &str) -> Vec<(String, &Symbol)> {
        self.symbols_by_name
            .get(name)
            .map(|files| {
                files
                    .iter()
                    .filter_map(|f| {
                        self.symbols_by_file
                            .get(f)?
                            .get(name)
                            .map(|s| (f.clone(), s))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn file_count(&self) -> usize {
        self.symbols_by_file.len()
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols_by_file.values().map(|m| m.len()).sum()
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol Resolver (跨文件引用追踪)
pub struct SymbolResolver {
    index: SymbolIndex,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
        }
    }

    pub fn from_index(index: SymbolIndex) -> Self {
        Self { index }
    }

    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }

    pub fn index_mut(&mut self) -> &mut SymbolIndex {
        &mut self.index
    }

    /// 解析 source 文件中的所有引用, 返回 ReferenceEdge 列表
    pub fn resolve_references(
        &self,
        source_file: &str,
        references: &[SymbolReference],
    ) -> Vec<ReferenceEdge> {
        let mut edges = vec![];
        for ref_item in references {
            // 简化: 提取最后一个 part 作为 symbol name, 倒数第二个作为 file hint
            let target_name = ref_item.parts.last().cloned().unwrap_or_default();
            let target_file = if ref_item.parts.len() >= 2 {
                format!("{}.rs", ref_item.parts[ref_item.parts.len() - 2])
            } else {
                "<unknown>".to_string()
            };

            // 检查是否在 index 中找到
            let resolved = !self.index.lookup_global(&target_name).is_empty();

            edges.push(ReferenceEdge {
                source_file: source_file.to_string(),
                source_ref: ref_item.clone(),
                target_file,
                target_name,
                resolved,
            });
        }
        edges
    }

    /// 跨文件查找: 给定 source file + name, 返回所有可能的 target file
    pub fn cross_file_lookup(&self, name: &str) -> Vec<String> {
        self.index
            .lookup_global(name)
            .into_iter()
            .map(|(f, _)| f)
            .collect()
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}
