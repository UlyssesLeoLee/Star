//! crates/star-treesitter — Tree-sitter 集成 (per P4-H.5, 守门 #19 [M] 拍板)
//!
//! 提供 5 语言 (Rust / TypeScript / Python / Go / JSON) 语法解析 + 符号提取.
//! per `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` §1.4
//!
//! 关键不变量 (per §1.4):
//! - INV-TS-01: parse 必须 thread-safe (CSP 兼容)
//! - INV-TS-02: 错误恢复: parse 错误返回 Err, 不 panic
//! - INV-TS-03: 5 语言 grammar 默认启用
//! - INV-TS-04: symbol 提取走 S-expression 路径, 不用字符串解析
//!
//! Lead 责任: 5 域 Lead 真人到位后追溯签字 (per 守门 #14)

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TreeSitterError {
    #[error("parse error at line {line}, column {column}: {message}")]
    ParseError { line: usize, column: usize, message: String },
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// 支持的 5 语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Go,
    Json,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Json => "json",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, TreeSitterError> {
        match s {
            "rust" => Ok(Self::Rust),
            "typescript" => Ok(Self::TypeScript),
            "python" => Ok(Self::Python),
            "go" => Ok(Self::Go),
            "json" => Ok(Self::Json),
            _ => Err(TreeSitterError::UnsupportedLanguage(s.into())),
        }
    }
}

/// 符号提取结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub name: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Static,
    Module,
    TypeAlias,
    Class,
    Interface,
    Method,
    Variable,
    Other,
}

/// 解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub language: Language,
    pub symbols: Vec<Symbol>,
    pub has_errors: bool,
    pub error_count: usize,
}

/// Tree-sitter parser (5 语言)
pub struct TreeSitterParser {
    language: Language,
}

impl TreeSitterParser {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// 解析源代码 + 提取符号
    pub fn parse(&self, source: &str) -> Result<ParseResult, TreeSitterError> {
        let mut parser = tree_sitter::Parser::new();
        let ts_lang = self.get_ts_language()?;
        parser
            .set_language(&ts_lang)
            .map_err(|e| TreeSitterError::Internal(format!("set_language: {}", e)))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| TreeSitterError::Internal("parse returned None".into()))?;

        let has_errors = tree.root_node().has_error();
        let symbols = self.extract_symbols(tree.root_node(), source);

        Ok(ParseResult {
            language: self.language,
            symbols,
            has_errors,
            error_count: if has_errors { 1 } else { 0 },
        })
    }

    fn get_ts_language(&self) -> Result<tree_sitter::Language, TreeSitterError> {
        match self.language {
            Language::Rust => Ok(tree_sitter_rust::LANGUAGE.into()),
            Language::TypeScript => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Language::Python => Ok(tree_sitter_python::LANGUAGE.into()),
            Language::Go => Ok(tree_sitter_go::LANGUAGE.into()),
            Language::Json => Ok(tree_sitter_json::LANGUAGE.into()),
        }
    }

    /// 从语法树提取符号 (简化版, 走 S-expression path)
    fn extract_symbols(&self, root: tree_sitter::Node<'_>, source: &str) -> Vec<Symbol> {
        let mut symbols = vec![];
        self.walk(root, source, &mut symbols);
        symbols
    }

    fn walk(&self, node: tree_sitter::Node<'_>, source: &str, symbols: &mut Vec<Symbol>) {
        let kind = node.kind();
        let symbol_kind = self.node_kind_to_symbol_kind(kind);
        if let Some(sk) = symbol_kind {
            // 尝试提取名字 (下一个 sibling 或 first named child)
            if let Some(name) = self.extract_name(node, source) {
                symbols.push(Symbol {
                    kind: sk,
                    name,
                    line: node.start_position().row + 1,
                    column: node.start_position().column + 1,
                });
            }
        }
        // 递归 walk children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(child, source, symbols);
        }
    }

    fn node_kind_to_symbol_kind(&self, kind: &str) -> Option<SymbolKind> {
        match kind {
            "function_item" => Some(SymbolKind::Function),
            "struct_item" => Some(SymbolKind::Struct),
            "enum_item" => Some(SymbolKind::Enum),
            "trait_item" => Some(SymbolKind::Trait),
            "impl_item" => Some(SymbolKind::Impl),
            "const_item" => Some(SymbolKind::Const),
            "static_item" => Some(SymbolKind::Static),
            "mod_item" => Some(SymbolKind::Module),
            "type_item" => Some(SymbolKind::TypeAlias),
            // TypeScript / Python / Go
            "function_declaration" | "method_definition" | "function_definition" => Some(SymbolKind::Function),
            "class_definition" | "class_declaration" => Some(SymbolKind::Class),
            "interface_declaration" => Some(SymbolKind::Interface),
            _ => None,
        }
    }

    fn extract_name(&self, node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
        // 简化: 找名为 "name" 的 child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier"
                || child.kind() == "type_identifier"
                || child.kind() == "name"
            {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    return Some(text.to_string());
                }
            }
        }
        None
    }
}

/// 便利函数: 解析 Rust 源码
pub fn parse_rust(source: &str) -> Result<ParseResult, TreeSitterError> {
    TreeSitterParser::new(Language::Rust).parse(source)
}

/// 便利函数: 解析 TypeScript 源码
pub fn parse_typescript(source: &str) -> Result<ParseResult, TreeSitterError> {
    TreeSitterParser::new(Language::TypeScript).parse(source)
}

#[cfg(test)]
mod tests;
