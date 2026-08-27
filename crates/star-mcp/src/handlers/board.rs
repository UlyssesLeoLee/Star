// crates/star-mcp/src/handlers/board.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! domain-board handler (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §6 #1 协作扩展)
//!
//! URI: `board://{id}` — 看板 (kanban board)
//! Cache TTL: 60s (协作类, 中频)
//! 真实数据源: TODO Phase H+ 接 `crates/domain-collaboration::board` (per spec/agents/02 §6 #1)
use crate::resources::{KeyBuilder, Resource, ResourceError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BoardData {
    pub board_id: String,
    pub name: String,
    pub columns: Vec<String>,
    pub created_at: i64,
}

pub(crate) struct BoardHandler;

#[async_trait]
impl Resource for BoardHandler {
    type Data = BoardData;
    fn uri_pattern(&self) -> &str {
        "board://{id}"
    }
    async fn read(&self, id: &str) -> Result<Option<Self::Data>, ResourceError> {
        // Phase H mock — TODO: 接入 crates/domain-collaboration::board 真实数据
        let _key = KeyBuilder::for_resource("board", id);
        Ok(Some(BoardData {
            board_id: id.into(),
            name: format!("Board {id} (mock)"),
            columns: vec!["Todo".into(), "Doing".into(), "Done".into()],
            created_at: 0,
        }))
    }
    fn cache_ttl_sec(&self) -> u32 {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn read_test() {
        let h = BoardHandler;
        let d = h.read("board-1").await.unwrap();
        assert_eq!(d.unwrap().board_id, "board-1");
    }
}
