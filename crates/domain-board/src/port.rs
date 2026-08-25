//! Board 端口(Port Traits)与命令/查询 DTO
//!
//! 来源:
//! - `docs/api-design.md` §3.7 (Board / Column / Swimlane 端点)
//! - `docs/specs/domain-board-spec.md` §4 (接口签名)
//!
//! **端口清单**:
//! - `BoardCommandPort`:3 方法(replace_board / patch_board / reorder_columns)
//! - `BoardQueryPort`:3 方法(get_by_project / list_columns / list_swimlanes)
//! - `BoardRepository`:基础设施层使用

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Board, Column, Swimlane};
use crate::error::BoardError;
use crate::value_object::{
    BoardId, BoardType, ColumnId, GroupByField, ProjectId, StateId, TenantId, UserId,
};

// =====================================================================
// 命令 DTO(写操作输入)
// =====================================================================

/// 单 Column 草稿(在 replace_board 时随 Board 整体提交)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDraft {
    /// 草稿 ID
    pub draft_id: uuid::Uuid,
    /// 列名
    pub name: String,
    /// 引用的 Workflow State
    pub state_id: StateId,
    /// 显示顺序
    pub display_order: u32,
    /// WIP 限制(可空)
    pub wip_limit: Option<u32>,
    /// 列颜色
    pub display_color: Option<String>,
}

/// 单 Swimlane 草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwimlaneDraft {
    /// 草稿 ID
    pub draft_id: uuid::Uuid,
    /// 名称
    pub name: String,
    /// group_by 字段
    pub group_by_field: GroupByField,
    /// 显示顺序
    pub display_order: u32,
}

/// `ReplaceBoardCommand`(整体替换 Board + Columns + Swimlanes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceBoardCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// Board 类型
    pub board_type: BoardType,
    /// Board 名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 过滤:仅看某分配人
    pub filter_assignee: Option<UserId>,
    /// 过滤:仅看某标签
    pub filter_label: Option<String>,
    /// 列草稿列表
    pub columns: Vec<ColumnDraft>,
    /// Swimlane 草稿列表
    pub swimlanes: Vec<SwimlaneDraft>,
    /// 期望乐观锁版本(0 表示新建,>0 表示更新)
    pub expected_version: u32,
}

/// `PatchBoardCommand`(部分更新)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchBoardCommand {
    /// Board ID
    pub board_id: BoardId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 期望乐观锁版本
    pub expected_version: u32,
    /// 新名称
    pub name: Option<String>,
    /// 新描述
    pub description: Option<Option<String>>,
    /// 新 Board 类型
    pub board_type: Option<BoardType>,
    /// 新过滤
    pub filter_assignee: Option<Option<UserId>>,
    /// 新过滤
    pub filter_label: Option<Option<String>>,
}

/// `ColumnOrderUpdate`(重新排序 column)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnOrderUpdate {
    /// Board ID
    pub board_id: BoardId,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 新顺序:column_id → display_order
    pub new_orders: Vec<(ColumnId, u32)>,
}

// =====================================================================
// 查询 DTO
// =====================================================================

/// `ListColumnsQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListColumnsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
}

/// `ListSwimlanesQuery`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSwimlanesQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
}

// =====================================================================
// 端口:BoardCommandPort(3 方法)
// =====================================================================

/// **Board 命令端口**(写操作 3 方法)
#[async_trait]
pub trait BoardCommandPort: Send + Sync {
    /// 整体替换 Board + Columns + Swimlanes(INV-B-01/02/03/04/05)
    async fn replace_board(
        &self,
        cmd: ReplaceBoardCommand,
        actor: ActorContext,
    ) -> Result<Board, BoardError>;

    /// 部分更新 Board
    async fn patch_board(
        &self,
        cmd: PatchBoardCommand,
        actor: ActorContext,
    ) -> Result<Board, BoardError>;

    /// 重排 Column(INV-B-03 UNIQUE 校验)
    async fn reorder_columns(
        &self,
        cmd: ColumnOrderUpdate,
        actor: ActorContext,
    ) -> Result<Vec<Column>, BoardError>;
}

// =====================================================================
// 端口:BoardQueryPort(3 方法)
// =====================================================================

/// **Board 查询端口**(读操作 3 方法)
#[async_trait]
pub trait BoardQueryPort: Send + Sync {
    /// 按 Project 取得 Board(每个 Project 1:1)
    async fn get_by_project(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Board, BoardError>;

    /// 列出 Board 下全部 Column(按 display_order ASC)
    async fn list_columns(
        &self,
        q: ListColumnsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Column>, BoardError>;

    /// 列出 Board 下全部 Swimlane(按 display_order ASC)
    async fn list_swimlanes(
        &self,
        q: ListSwimlanesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Swimlane>, BoardError>;
}

// =====================================================================
// 仓库端口(供 infrastructure crate 适配)
// =====================================================================

/// **Board 仓库端口**
#[async_trait]
pub trait BoardRepository: Send + Sync {
    /// 插入
    async fn insert(&self, board: &Board) -> Result<(), BoardError>;
    /// 按 ID 读
    async fn find_by_id(&self, id: BoardId) -> Result<Option<Board>, BoardError>;
    /// 更新(乐观锁)
    async fn update(&self, board: &Board) -> Result<(), BoardError>;
    /// 按 Project 读
    async fn find_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<Board>, BoardError>;

    /// 插入 Column
    async fn insert_column(&self, col: &Column) -> Result<(), BoardError>;
    /// 替换 Board 下所有 Column
    async fn replace_columns(
        &self,
        board_id: BoardId,
        cols: &[Column],
    ) -> Result<(), BoardError>;
    /// 列出 Board 下所有 Column
    async fn list_columns_raw(&self, board_id: BoardId) -> Result<Vec<Column>, BoardError>;

    /// 插入 Swimlane
    async fn insert_swimlane(&self, s: &Swimlane) -> Result<(), BoardError>;
    /// 替换 Board 下所有 Swimlane
    async fn replace_swimlanes(
        &self,
        board_id: BoardId,
        swimlanes: &[Swimlane],
    ) -> Result<(), BoardError>;
    /// 列出 Board 下所有 Swimlane
    async fn list_swimlanes_raw(&self, board_id: BoardId) -> Result<Vec<Swimlane>, BoardError>;
}
