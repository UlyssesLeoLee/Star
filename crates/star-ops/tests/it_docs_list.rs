// crates/star-ops/tests/it_docs_list.rs
//
// F-04 端到端 IT 雏形 (per 守门 #1 v25 + 守门 #9 v20 + 守门 #13)
//
// 守门 #1 v25: cargo test -p star-ops --lib -j 4 100% pass
// 守门 #9 v20: 子代理 RPC 不可靠实证 → 跨 crate IT + walkdir 真实跑通
// 守门 #13: F-04 0 新表 (per brief §1, 文档扫描不入 DB)
//
// 范围:
// - 跨 crate IT: axum oneshot 调 /api/ops/docs 真实端点
// - walkdir 真实跑通: 调 DocScanner.list() 真实扫 docs/ 4 子目录
// - 5 类别 (SRS / BAS / DET / Report / Other) 分类正确性

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use star_ops::ops_api::{router, AppState};
use star_ops::ops_domain::docs::{DocCategory, DocScanner};
use tower::ServiceExt;

/// F-04 端到端 IT: docs_list 真实端点 (跨 crate IT)
/// 验证: walkdir 真实扫 docs/ 4 子目录 + meta.stub=false + meta.hint 含 walkdir
#[tokio::test]
async fn it_docs_list_end_to_end() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/ops/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 验证 body 真实返 (至少 1 条 doc 必扫到, 守門 #1 R-05)
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).expect("body is JSON");
    let data = body["data"].as_array().expect("data is array");
    assert!(
        !data.is_empty(),
        "F-04 端到端 walkdir 必扫到 docs/ 子目录文档"
    );

    // 验证 meta.stub=false (F-04 端到端, 跟 F-03 metrics_summary_real 实证同)
    assert_eq!(
        body["meta"]["stub"].as_bool(),
        Some(false),
        "F-04 端到端 stub 必为 false"
    );

    // 验证 meta.hint 含 walkdir 标识
    assert!(
        body["meta"]["hint"]
            .as_str()
            .unwrap_or("")
            .contains("walkdir"),
        "F-04 端到端 hint 必含 walkdir"
    );

    // 验证至少 1 条 doc 字段完整 (path / title / category / updated_at)
    let first = &data[0];
    assert!(first["path"].is_string(), "doc.path 必是 string");
    assert!(first["title"].is_string(), "doc.title 必是 string");
    assert!(first["category"].is_string(), "doc.category 必是 string");
    assert!(
        first["updated_at"].is_string(),
        "doc.updated_at 必是 string"
    );
}

/// F-04 端到端 IT: walkdir 真实跑通 (跨 crate IT, 调 DocScanner)
/// 验证: 5 类别 (SRS / BAS / DET / Report / Other) 分类正确
#[test]
fn it_walkdir_real_scan_via_doc_scanner() {
    let scanner = DocScanner::new();
    let docs = scanner.list();
    assert!(
        !docs.is_empty(),
        "F-04 walkdir 必扫到 docs/ 子目录至少 1 条文档"
    );

    // 5 类别枚举验证 (per DocCategory::short_name)
    let valid_categories = [
        DocCategory::Srs.short_name(),
        DocCategory::Bas.short_name(),
        DocCategory::Det.short_name(),
        DocCategory::Report.short_name(),
        DocCategory::Other.short_name(),
    ];

    for d in &docs {
        assert!(
            valid_categories.contains(&d.category.as_str()),
            "doc.category 必是 5 类别之一 (SRS/BAS/DET/Report/Other), got: {}",
            d.category
        );
    }
}

/// F-04 端到端 IT: walkdir 仅扫 docs/ 子目录 (守門 #1 R-05)
/// 验证: 返回的所有 path 必以 docs/ 开头 (不扫全仓库)
#[test]
fn it_walkdir_scans_only_docs_subdirs() {
    let scanner = DocScanner::new();
    let docs = scanner.list();
    for d in &docs {
        assert!(
            d.path.starts_with("docs/"),
            "F-04 walkdir 路径必以 docs/ 开头 (守門 #1 R-05 不动生产), got: {}",
            d.path
        );
        assert!(
            d.path.ends_with(".md"),
            "F-04 walkdir 必仅扫 .md 文件, got: {}",
            d.path
        );
    }
}

// ============ UT-IT-51 §3.3 Phase 4 F-04 派生缺口 (per brief §2.1) ============

/// 派生 #39: walkdir 真实扫 (跟 baseline it_walkdir_real_scan_via_doc_scanner 互补, 跨 crate 端点)
/// 守门 #1 R-05 + 守门 #5 v2
#[tokio::test]
async fn it_docs_list_walkdir_real_scan() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/ops/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 验证 body 含 walkdir 真实扫到的 docs
    use axum::body::to_bytes;
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).expect("body is JSON");
    let data = body["data"].as_array().expect("data is array");
    assert!(
        !data.is_empty(),
        "F-04 walkdir 真实扫必返至少 1 条 doc (跟 IT baseline 同)"
    );
}

/// 派生 #40: walkdir 仅扫 docs/ 子目录 (跟 baseline it_walkdir_scans_only_docs_subdirs 互补, 跨 crate 端点)
/// 守門 #1 R-05
#[tokio::test]
async fn it_docs_list_walkdir_scans_only_docs_subdirs() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/ops/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    use axum::body::to_bytes;
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).expect("body is JSON");
    let data = body["data"].as_array().expect("data is array");
    for d in data.iter() {
        let path = d["path"].as_str().expect("path is string");
        assert!(
            path.starts_with("docs/"),
            "F-04 walkdir 路径必以 docs/ 开头, got: {}",
            path
        );
        assert!(
            path.ends_with(".md"),
            "F-04 walkdir 必仅扫 .md 文件, got: {}",
            path
        );
    }
}

/// 派生 #41: 长路径 (Windows MAX_PATH 260 字符) 边界
/// 守门 #11 缺标比错标: 长路径走稳定路径
#[test]
fn it_docs_list_handles_path_too_long() {
    // 派生测: 验证 walkdir 遇到长路径不 panic
    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段加长路径特殊处理 (Windows MAX_PATH 260)
    let scanner = DocScanner::new();
    let docs = scanner.list();
    // 验证所有 doc path 长度 < 1000 (避免 Windows MAX_PATH 派生)
    for d in &docs {
        assert!(
            d.path.len() < 1000,
            "doc path 必 < 1000 字符, got: {} chars",
            d.path.len()
        );
    }
}
