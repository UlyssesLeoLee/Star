# 18. Code Navigation Architecture

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/context/02-code-intelligence-arch.md](02-code-intelligence-arch.md)

## 1. 能力清单

| 能力 | MCP tool / CLI 命令 | MVP | Phase 2 |
|---|---|---|---|
| Go to Definition | `get_symbol` + `code symbol` | ⚠️ 文件名 + 行号 | ✅ LSP |
| Find References | `find_references` + `code references` | ⚠️ grep | ✅ LSP |
| Document Symbols | `get_symbol` | ⚠️ 简单解析 | ✅ AST |
| Hover / Type Info | n/a (MCP) | ❌ | ✅ LSP |
| Call Hierarchy | n/a (Phase 3) | ❌ | ❌ |
| Workspace Symbol | `search_code` | ⚠️ 文件名匹配 | ✅ LSP |
| Semantic Search | n/a (Phase 3) | ❌ | ❌ |

## 2. MVP 实现（fallback to grep）

```rust
// crates/star-code-intelligence/src/grep.rs
pub fn find_references_approx(name: &str, scope: &Path) -> Vec<Reference> {
    // 1. ripgrep 全文搜索
    // 2. 简单去重
    // 3. 返回近似结果（带"⚠️ not semantic"标记）
}
```

## 3. Phase 2 实现

```rust
// crates/star-code-intelligence/src/lsp.rs
pub async fn find_references_lsp(name: &str, file: &Path) -> Result<Vec<Reference>> {
    // 1. spawn rust-analyzer / ty / 等 LSP server
    // 2. textDocument/references 请求
    // 3. 返回精确结果
}
```

## 4. 实施位置

- `crates/star-code-intelligence/src/navigation.rs`
- `crates/star-code-intelligence/src/grep.rs` (MVP)
- `crates/star-code-intelligence/src/lsp.rs` (Phase 2)

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
