//! crates/star-treesitter — 3 e2e tests
//! per 守门 #19 [M] 拍板

use super::*;

/// H.5 test 1: parse_rust 提取 function + struct + enum
#[test]
fn h5_parse_rust_extracts_symbols() {
    let source = r#"
            struct Foo {
                x: i32,
            }

            enum Color {
                Red,
                Blue,
            }

            fn bar() -> i32 {
                42
            }
        "#;
    let result = parse_rust(source).unwrap();
    assert_eq!(result.language, Language::Rust);
    assert!(!result.has_errors);
    let names: Vec<&str> = result.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"Foo"),
        "expected Foo in symbols, got {:?}",
        names
    );
    assert!(
        names.contains(&"Color"),
        "expected Color in symbols, got {:?}",
        names
    );
    assert!(
        names.contains(&"bar"),
        "expected bar in symbols, got {:?}",
        names
    );
    // 验证 SymbolKind
    assert!(result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Struct && s.name == "Foo"));
    assert!(result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Enum && s.name == "Color"));
    assert!(result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Function && s.name == "bar"));
}

/// H.5 test 2: parse_typescript 提取 function + interface
#[test]
fn h5_parse_typescript_extracts_symbols() {
    let source = r#"
            interface User {
                id: number;
                name: string;
            }

            function getUser(id: number): User {
                return { id, name: 'test' };
            }
        "#;
    let result = parse_typescript(source).unwrap();
    assert_eq!(result.language, Language::TypeScript);
    assert!(!result.has_errors);
    let names: Vec<&str> = result.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"User"),
        "expected User in symbols, got {:?}",
        names
    );
    assert!(
        names.contains(&"getUser"),
        "expected getUser in symbols, got {:?}",
        names
    );
    assert!(result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Interface && s.name == "User"));
    assert!(result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Function && s.name == "getUser"));
}

/// H.5 test 3: Language::from_str 验证 5 语言 + 错误处理
#[test]
fn h5_language_from_str_validation() {
    assert_eq!(Language::from_str("rust").unwrap(), Language::Rust);
    assert_eq!(
        Language::from_str("typescript").unwrap(),
        Language::TypeScript
    );
    assert_eq!(Language::from_str("python").unwrap(), Language::Python);
    assert_eq!(Language::from_str("go").unwrap(), Language::Go);
    assert_eq!(Language::from_str("json").unwrap(), Language::Json);
    // 不支持的语言
    let err = Language::from_str("cobol").unwrap_err();
    assert!(matches!(err, TreeSitterError::UnsupportedLanguage(_)));
}
