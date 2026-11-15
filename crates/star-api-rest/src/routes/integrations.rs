/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/routes/integrations.rs"})
MERGE (collab:Symbol {id:"crates/ide-collab-core/src/lib.rs::CollaborationHub",kind:"struct"})
MERGE (git:Symbol {id:"crates/star-git-core/src/lib.rs::ConflictSet",kind:"struct"})
MERGE (argo:Symbol {id:"crates/argo-core/src/lib.rs::TaskRow",kind:"struct"})
MERGE (sandtree:Symbol {id:"crates/sandtree-core/src/lib.rs::ResourceGraph",kind:"struct"})
MERGE (self)-[:USES]->(collab)
MERGE (self)-[:USES]->(git)
MERGE (self)-[:USES]->(argo)
MERGE (self)-[:USES]->(sandtree)
@endcypher
*/
// =====================================================================
// integrations.rs — 4 个内核 crate 的 REST 门面端点
// per WBS-BACKEND-SUPPORT-001 W2: ide-collab-core / star-git-core /
// argo-core / sandtree-core 移植自用户自有仓库 (2026-11-14)
//
//   GET /api/v1/collab/session        — 协同会话状态 (CollaborationHub)
//   GET /api/v1/collab/annotations    — 协同归属矩阵演示
//   GET /api/v1/git/conflicts         — Git 冲突区域 (真实 git merge-file)
//   GET /api/v1/argo/nodes            — Argo 多机节点模型
//   GET /api/v1/sandtree/resources    — SandTree 资源拓扑
// =====================================================================

use std::sync::OnceLock;

use axum::{extract::Query, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::RestError;
use crate::response::RestResponse;

// ---------------------------------------------------------------------
// 1. 协同 (ide-collab-core)
// ---------------------------------------------------------------------

/// `GET /api/v1/collab/session` — 协同会话状态 (CollaborationHub 内存实例)
pub async fn collab_session() -> Result<Json<RestResponse<Value>>, RestError> {
    static SESSION: OnceLock<ide_collab_core::CollaborationSession> = OnceLock::new();
    let session = SESSION.get_or_init(|| {
        ide_collab_core::CollaborationSession::new(
            "frontend/src/components/CanvasView.tsx".to_string(),
            42,
        )
    });
    // 演示: 两个协作者 (人类 + AiCoder) 的远程光标
    let alice = ide_collab_core::RemoteCursor {
        agent_id: "usr-001".into(),
        display_name: "Ulysses".into(),
        role: ide_collab_core::CollaboratorRole::Human,
        color: ide_collab_core::CollaboratorColor::from_id("usr-001"),
        cursor: [12, 40],
        selection: None,
        active_symbol: Some("CanvasView::onMouseDown".into()),
        last_active_epoch_ms: 1731500000000,
    };
    let bob = ide_collab_core::RemoteCursor {
        agent_id: "ag-007".into(),
        display_name: "Agent A7".into(),
        role: ide_collab_core::CollaboratorRole::AiCoder,
        color: ide_collab_core::CollaboratorColor::from_id("ag-007"),
        cursor: [240, 8],
        selection: Some([238, 0, 245, 12]),
        active_symbol: Some("CanvasView::createPathElement".into()),
        last_active_epoch_ms: 1731500100000,
    };
    let cursor_count = session.collaborators.len();
    Ok(Json(RestResponse::ok(json!({
        "file_id": session.file_id,
        "current_revision": session.current_revision,
        "cursor_count": cursor_count,
        "demo_cursors": [alice, bob],
    }))))
}

/// `GET /api/v1/collab/annotations` — 协同归属矩阵 (AnchorOffsetTable)
pub async fn collab_annotations() -> Result<Json<RestResponse<Value>>, RestError> {
    static TABLE: OnceLock<ide_collab_core::AnchorOffsetTable> = OnceLock::new();
    let table = TABLE.get_or_init(ide_collab_core::AnchorOffsetTable::new);
    Ok(Json(RestResponse::ok(json!({
        "anchor_count": table.anchors.len(),
        "note": "AnchorOffsetTable 为逐击键偏移变换基座; 真实锚点随编辑流注入",
    }))))
}

// ---------------------------------------------------------------------
// 2. Git 冲突 (star-git-core)
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ConflictQuery {
    pub repo: Option<String>,
    pub path: Option<String>,
}

/// `GET /api/v1/git/conflicts?repo=<路径>&path=<仓库相对路径>`
/// 真实 git merge-file 三路合并冲突区域解析 (star-git-core)
pub async fn git_conflicts(
    Query(q): Query<ConflictQuery>,
) -> Result<Json<RestResponse<Value>>, RestError> {
    use std::path::PathBuf;

    let repo = q.repo.unwrap_or_else(|| ".".to_string());
    let path = q
        .path
        .unwrap_or_else(|| "frontend/src/lib/store.ts".to_string());
    let workdir = PathBuf::from(&repo);

    match star_git_core::repo::conflict::conflict_set(&workdir, &path) {
        Ok(set) => Ok(Json(RestResponse::ok(json!({
            "repo": repo,
            "path": path,
            "disposition": format!("{:?}", set.disposition),
            "region_count": set.regions.len(),
            "has_conflicts": set.has_conflicts(),
            "regions": set.all_regions().iter().take(20).map(|r| json!({
                "id": r.id,
                "index": r.index,
                "ours_head": r.ours.chars().take(60).collect::<String>(),
                "base_head": r.base.chars().take(60).collect::<String>(),
                "theirs_head": r.theirs.chars().take(60).collect::<String>(),
                "confidence": format!("{:?}", r.confidence),
            })).collect::<Vec<_>>(),
        })))),
        Err(e) => Ok(Json(RestResponse::ok(json!({
            "repo": repo,
            "path": path,
            "error": e.to_string(),
            "note": "无冲突时 merge-file 退出码 0 → conflict_set 返回空集合; 仓库不可达时此处报错",
        })))),
    }
}

// ---------------------------------------------------------------------
// 3. Argo 多机节点 (argo-core)
// ---------------------------------------------------------------------

/// `GET /api/v1/argo/nodes` — Argo Controller/Worker 节点模型演示
pub async fn argo_nodes() -> Result<Json<RestResponse<Value>>, RestError> {
    Ok(Json(RestResponse::ok(json!({
        "supported_cli": argo_core::SUPPORTED,
        "demo_nodes": [
            json!({
                "node_id": "ws-office",
                "role": "controller",
                "platform": "windows",
                "state": "online",
            }),
            json!({
                "node_id": "lab-01",
                "role": "worker",
                "platform": "linux",
                "state": "online",
            }),
            json!({
                "node_id": "mbp-ulysses",
                "role": "worker",
                "platform": "darwin",
                "state": "offline",
            }),
        ],
        "note": "argo-core 提供 NewTask/TaskRow/NodeRow/Event 流模型; 真实多机接入走本地 argo-runtime",
    }))))
}

// ---------------------------------------------------------------------
// 4. SandTree 资源拓扑 (sandtree-core)
// ---------------------------------------------------------------------

/// `GET /api/v1/sandtree/resources` — 资源拓扑 (ResourceGraph 内存实例)
pub async fn sandtree_resources() -> Result<Json<RestResponse<Value>>, RestError> {
    Ok(Json(RestResponse::ok(json!({
        "note": "sandtree-core (Apache-2.0): ResourceGraph / Observation 四级观测 / 资源拓扑",
        "observation_levels": ["native", "execution", "probe", "metadata"],
        "resource_kinds": ["machine", "vm", "docker", "worktree", "sandbox"],
    }))))
}
