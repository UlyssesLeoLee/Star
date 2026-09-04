//! crates/star-treesitter — H.7 Symbol Resolver 3 e2e tests

use super::*;
use crate::symbol_resolver::{SymbolIndex, SymbolReference, SymbolResolver};

/// H.7 test 1: SymbolReference::parse 拆 "foo::bar::baz"
#[test]
fn h7_symbol_reference_parse() {
    let r = SymbolReference::parse("foo::bar::baz");
    assert_eq!(r.parts, vec!["foo", "bar", "baz"]);
    assert_eq!(r.raw, "foo::bar::baz");

    let r2 = SymbolReference::parse_at("a::b", 10, 5);
    assert_eq!(r2.line, 10);
    assert_eq!(r2.column, 5);
}

/// H.7 test 2: SymbolIndex.add_file + lookup
#[test]
fn h7_symbol_index_add_and_lookup() {
    let mut index = SymbolIndex::new();
    let result = parse_rust("struct Foo { x: i32 } fn bar() {}").unwrap();
    index.add_file("domain_tenant.rs", &result);

    // lookup by file + name
    let foo = index.lookup("domain_tenant.rs", "Foo");
    assert!(foo.is_some());
    assert_eq!(foo.unwrap().name, "Foo");

    // lookup_global
    let all_bar = index.lookup_global("bar");
    assert_eq!(all_bar.len(), 1);
    assert_eq!(all_bar[0].0, "domain_tenant.rs");

    assert_eq!(index.file_count(), 1);
    assert!(index.symbol_count() >= 2);
}

/// H.7 test 3: SymbolResolver.resolve_references 跨文件解析
#[test]
fn h7_resolve_references_cross_file() {
    let mut resolver = SymbolResolver::new();
    // 1. 添加 domain_tenant.rs 的 symbols
    let r1 = parse_rust("struct PlayerService { id: i32 }").unwrap();
    resolver.index_mut().add_file("domain_tenant.rs", &r1);

    // 2. 添加 domain_project.rs 的 symbols
    let r2 = parse_rust("struct Project { name: String }").unwrap();
    resolver.index_mut().add_file("domain_project.rs", &r2);

    // 3. 解析 source.rs 的引用
    let refs = vec![
        SymbolReference::parse_at("domain_tenant::PlayerService", 1, 1),
        SymbolReference::parse_at("domain_project::Project", 2, 1),
        SymbolReference::parse_at("unknown_module::Missing", 3, 1), // 未解析
    ];
    let edges = resolver.resolve_references("source.rs", &refs);

    assert_eq!(edges.len(), 3);

    // 第一条: PlayerService 应解析成功
    assert!(edges[0].resolved);
    assert_eq!(edges[0].target_name, "PlayerService");
    assert_eq!(edges[0].target_file, "domain_tenant.rs");

    // 第二条: Project 应解析成功
    assert!(edges[1].resolved);
    assert_eq!(edges[1].target_name, "Project");
    assert_eq!(edges[1].target_file, "domain_project.rs");

    // 第三条: Missing 未解析
    assert!(!edges[2].resolved);
    assert_eq!(edges[2].target_name, "Missing");
}

/// H.7 test 4: cross_file_lookup 跨文件查询
#[test]
fn h7_cross_file_lookup() {
    let mut resolver = SymbolResolver::new();
    resolver
        .index_mut()
        .add_file("a.rs", &parse_rust("fn shared() {}").unwrap());
    resolver
        .index_mut()
        .add_file("b.rs", &parse_rust("fn shared() {}").unwrap());
    resolver
        .index_mut()
        .add_file("c.rs", &parse_rust("struct Unique {}").unwrap());

    let shared_files = resolver.cross_file_lookup("shared");
    assert_eq!(shared_files.len(), 2);
    assert!(shared_files.contains(&"a.rs".to_string()));
    assert!(shared_files.contains(&"b.rs".to_string()));

    let unique_files = resolver.cross_file_lookup("Unique");
    assert_eq!(unique_files.len(), 1);
    assert_eq!(unique_files[0], "c.rs");
}
