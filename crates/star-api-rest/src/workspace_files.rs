/*
@cypher schema=1 source_sha256=47ba4e9306ccf112cb2b961660fe8968cbc68a0b87c87da91437271d5059f2bf
MERGE (self:File {path:"crates/star-api-rest/src/workspace_files.rs"})
MERGE (router:Function {name:"workspace_files::router", language:"rust", visibility:"pub"})
MERGE (state:Type {id:"workspace_files::WorkspaceFilesState",kind:"struct"})
MERGE (list:Function {name:"workspace_files::list_handler", language:"rust", visibility:"private"})
MERGE (read:Function {name:"workspace_files::read_handler", language:"rust", visibility:"private"})
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(list)
MERGE (self)-[:DEFINES]->(read)
@endcypher
*/
//! 本地文件夹桥接 (per 2026-11-11 用户发令: 网页版关联本地文件夹)。
//!
//! 受控只读文件 API, 安全边界:
//! - 白名单根目录由 `STAR_FILE_ROOTS` 环境变量配置 (分号分隔), 缺省
//!   `D:/IPA-Atomic-Skills;D:/Star` — 根目录之外一律 403。
//! - 路径规范化后必须落在白名单根内 (防 `..` 逃逸)。
//! - 只读 (无写端点), 单文件读取上限 4 MiB, 目录列举深度上限 6。
//! - 无认证要求但后端只绑定 127.0.0.1 (本地模式专属; 线上 Vercel 无此端点)。

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TREE_DEPTH: usize = 6;
const MAX_TREE_ENTRIES: usize = 2000;

#[derive(Clone)]
pub struct WorkspaceFilesState {
    roots: Arc<Vec<PathBuf>>,
}

impl WorkspaceFilesState {
    /// 从 `STAR_FILE_ROOTS` 构造 (分号分隔; 缺省 IPA 技能库 + Star 仓库)。
    pub fn from_env() -> Self {
        let raw = std::env::var("STAR_FILE_ROOTS")
            .unwrap_or_else(|_| "D:/IPA-Atomic-Skills;D:/Star".to_string());
        let roots: Vec<PathBuf> = raw
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect();
        Self {
            roots: Arc::new(roots),
        }
    }

    #[cfg(test)]
    fn from_roots(roots: Vec<PathBuf>) -> Self {
        Self {
            roots: Arc::new(roots),
        }
    }

    /// 规范化并校验 path 落在白名单根内。返回 (根索引, 相对路径)。
    /// 多根时优先返回 path 实际存在的根 (e.g. "frontend/src" 在 D:/Star 下
    /// 而不在 D:/IPA-Atomic-Skills 下 → 解析到 D:/Star); 全不存在则退回
    /// 第一个根, 让 list/read 以统一 NOT_FOUND 语义报错。
    fn resolve(&self, input: &str) -> Result<(usize, PathBuf), StatusCode> {
        let candidate = PathBuf::from(input);
        let mut normalized = PathBuf::new();
        for component in candidate.components() {
            match component {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(StatusCode::FORBIDDEN);
                }
                Component::CurDir => {}
                Component::Normal(part) => normalized.push(part),
            }
        }
        let mut fallback: Option<(usize, PathBuf)> = None;
        for (index, root) in self.roots.iter().enumerate() {
            let full = root.join(&normalized);
            if full.starts_with(root) {
                if fallback.is_none() {
                    fallback = Some((index, normalized.clone()));
                }
                if full.exists() {
                    return Ok((index, normalized));
                }
            }
        }
        fallback.ok_or(StatusCode::FORBIDDEN)
    }

    fn full_path(&self, index: usize, relative: &Path) -> PathBuf {
        self.roots[index].join(relative)
    }
}

#[derive(Deserialize)]
struct ListQuery {
    path: Option<String>,
}

#[derive(Deserialize)]
struct ReadQuery {
    path: String,
}

pub fn router(state: WorkspaceFilesState) -> Router {
    Router::new()
        .route(
            "/api/v1/workspace-files",
            get(list_handler).with_state(state.clone()),
        )
        .route(
            "/api/v1/workspace-files/read",
            get(read_handler).with_state(state),
        )
}

async fn list_handler(
    State(state): State<WorkspaceFilesState>,
    Query(query): Query<ListQuery>,
) -> Response {
    let relative = query.path.as_deref().unwrap_or("");
    let (root_index, rel) = match state.resolve(relative) {
        Ok(resolved) => resolved,
        Err(status) => return status.into_response(),
    };
    let dir = state.full_path(root_index, &rel);
    let Ok(meta) = std::fs::metadata(&dir) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "PATH_NOT_FOUND" })),
        )
            .into_response();
    };
    if !meta.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "NOT_A_DIRECTORY" })),
        )
            .into_response();
    }
    let depth = rel.components().count();
    if depth > MAX_TREE_DEPTH {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "TREE_DEPTH_EXCEEDED" })),
        )
            .into_response();
    }

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "READ_DIR_FAILED" })),
        )
            .into_response();
    };
    for entry in read_dir.flatten() {
        if entries.len() >= MAX_TREE_ENTRIES {
            break;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = file_type.is_dir();
        let size = if is_dir {
            None
        } else {
            entry.metadata().map(|m| m.len()).ok()
        };
        entries.push(json!({
            "name": name,
            "is_dir": is_dir,
            "size_bytes": size,
        }));
    }
    entries.sort_by(|a, b| {
        let (a_dir, b_dir) = (
            a["is_dir"].as_bool() == Some(true),
            b["is_dir"].as_bool() == Some(true),
        );
        match (a_dir, b_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a["name"].as_str().cmp(&b["name"].as_str()),
        }
    });

    Json(json!({
        "root": state.roots[root_index].to_string_lossy(),
        "path": relative,
        "entries": entries,
    }))
    .into_response()
}

async fn read_handler(
    State(state): State<WorkspaceFilesState>,
    Query(query): Query<ReadQuery>,
) -> Response {
    let (root_index, rel) = match state.resolve(&query.path) {
        Ok(resolved) => resolved,
        Err(status) => return status.into_response(),
    };
    let full = state.full_path(root_index, &rel);
    let Ok(meta) = std::fs::metadata(&full) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "PATH_NOT_FOUND" })),
        )
            .into_response();
    };
    if !meta.is_file() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "NOT_A_FILE" })),
        )
            .into_response();
    }
    if meta.len() > MAX_FILE_BYTES {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({ "error": "FILE_TOO_LARGE", "max_bytes": MAX_FILE_BYTES })),
        )
            .into_response();
    }
    match std::fs::read(&full) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => Json(json!({ "path": query.path, "text": text })).into_response(),
            Err(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(json!({ "error": "BINARY_FILE_NOT_SERVED" })),
            )
                .into_response(),
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "READ_FAILED" })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("star-workspace-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp root");
        dir
    }

    #[test]
    fn resolve_blocks_parent_escape() {
        let root = temp_root();
        let state = WorkspaceFilesState::from_roots(vec![root]);
        assert_eq!(state.resolve("ok/file.md").unwrap().0, 0);
        assert!(state.resolve("../secret").is_err());
        assert!(state.resolve("a/../../b").is_err());
        assert!(state.resolve("C:/Windows/system32").is_err());
    }

    #[test]
    fn resolve_rejects_path_outside_roots() {
        let root = temp_root();
        let state = WorkspaceFilesState::from_roots(vec![root]);
        assert_eq!(state.resolve("inside.md").unwrap().0, 0);
        // 相对路径永远从白名单根解析, 不存在"根外"相对路径; 根外检测主要靠 RootDir/Prefix 拒绝
        assert!(state.resolve("D:/elsewhere/x.md").is_err());
    }

    #[test]
    fn resolve_picks_the_root_where_the_path_exists() {
        let root_a = temp_root().join("a");
        let root_b = temp_root().join("b");
        std::fs::create_dir_all(root_a.clone()).expect("create root a");
        std::fs::create_dir_all(root_b.join("frontend/src")).expect("create nested dir");
        let state = WorkspaceFilesState::from_roots(vec![root_a, root_b]);
        // "frontend/src" 在 root_a 下不存在, 在 root_b 下存在 → 解析到 root_b
        let (index, _rel) = state.resolve("frontend/src").unwrap();
        assert_eq!(index, 1);
        // 全部不存在 → 退回第一个根 (404 语义而非 403)
        let (index, _rel) = state.resolve("no/such/path.md").unwrap();
        assert_eq!(index, 0);
    }
}
