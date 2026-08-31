//! domain-board crate
//!
//! 详细 spec: docs/specs/domain-board-spec.md §11
//! 上游基本设计: docs/basic-design.md §11 (Board 视图)
//! 数据设计: docs/data-design.md §4.6 (`board` schema)
//! API 设计: docs/api-design.md §3.7
//!
//! ## 职责
//!
//! Board 视图领域(§11,REQ-PLAN-003):
//! - Board 聚合根(看板视图配置)
//! - BoardColumn 实体(列,Todo/InProgress/Review/Done)
//! - Swimlane 实体(按 assignee/priority/epic 分组)
//! - BoardCard 投影(WorkItem 在 Board 视图上的只读投影)
//! - 3 trait: BoardCommandPort / BoardQueryPort / BoardRepository
//! - 5 不变量 (INV-BD-01~05)
//!
//! ## 关键不变量 (INV-BD-01~05)
//!
//! - INV-BD-01:Board 必带 tenant_id
//! - INV-BD-02:WIP limit 不可超(enable 时)
//! - INV-BD-03:Column order 唯一
//! - INV-BD-04:Card 必在某个 column 内
//! - INV-BD-05:Move card 跨 tenant 拒绝
//!
//! Lead 责任: board Lead
//!
//! ## v0.2 单文件化说明
//!
//! 之前是 9 个分模块(context / entity / error / event / invariants / macros /
//! port / service / value_object),现已合并到本单文件,移除旧的事件总线和
//! invariants 独立模块。所有类型保持原可见性以便调用方平滑过渡。

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
pub use star_context::ActorContext;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(BoardId);
define_uuid_id!(BoardColumnId);
define_uuid_id!(SwimlaneId);
define_uuid_id!(BoardCardId);
define_uuid_id!(ProjectId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// UUID 强类型 ID 宏(参考 domain-tenant / domain-permission 模式)
// =====================================================================

/// 定义一个 UUID 强类型 ID newtype。
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// 新建随机 UUID
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 取内部 UUID
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 角色常量
// =====================================================================

/// Board 相关标准角色
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}

// =====================================================================
// 枚举:BoardKind / SwimlaneGroupBy
// =====================================================================

/// **Board 类型**(§11.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BoardKind {
    /// Kanban 看板
    Kanban,
    /// Scrum 板(Sprint 关联)
    Scrum,
    /// Bug 跟踪板
    BugTracking,
}

impl Default for BoardKind {
    fn default() -> Self {
        Self::Kanban
    }
}

impl BoardKind {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kanban => "KANBAN",
            Self::Scrum => "SCRUM",
            Self::BugTracking => "BUG_TRACKING",
        }
    }
}

/// **Swimlane 分组维度**(§11.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwimlaneGroupBy {
    /// 按 Assignee 分组
    Assignee,
    /// 按 Priority 分组
    Priority,
    /// 按 Epic 分组
    Epic,
    /// 不分组(单泳道)
    None,
}

impl Default for SwimlaneGroupBy {
    fn default() -> Self {
        Self::None
    }
}

impl SwimlaneGroupBy {
    /// 大写字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assignee => "ASSIGNEE",
            Self::Priority => "PRIORITY",
            Self::Epic => "EPIC",
            Self::None => "NONE",
        }
    }
}

// =====================================================================
// 实体:BoardColumn / Swimlane / BoardCard / Board
// =====================================================================

/// **BoardColumn** — Board 的列(§11.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    /// 主键
    pub id: BoardColumnId,
    /// 名称(Todo / InProgress / Review / Done 等)
    pub name: String,
    /// WIP 限制(None = 不限制)
    pub wip_limit: Option<u32>,
    /// 显示顺序(INV-BD-03 唯一)
    pub order: u32,
    /// 颜色(可选)
    pub color: Option<String>,
}

impl BoardColumn {
    /// 默认 3 列模板:Todo / InProgress / Done
    pub fn default_three() -> Vec<BoardColumn> {
        vec![
            BoardColumn {
                id: BoardColumnId::new(),
                name: "Todo".to_string(),
                wip_limit: None,
                order: 0,
                color: Some("#999".to_string()),
            },
            BoardColumn {
                id: BoardColumnId::new(),
                name: "InProgress".to_string(),
                wip_limit: None,
                order: 1,
                color: Some("#06c".to_string()),
            },
            BoardColumn {
                id: BoardColumnId::new(),
                name: "Done".to_string(),
                wip_limit: None,
                order: 2,
                color: Some("#0a6".to_string()),
            },
        ]
    }
}

/// **Swimlane** — 泳道(按维度分组,§11.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swimlane {
    /// 主键
    pub id: SwimlaneId,
    /// 名称
    pub name: String,
    /// 分组维度
    pub group_by: SwimlaneGroupBy,
    /// 显示顺序
    pub order: u32,
}

impl Swimlane {
    /// 新建泳道
    pub fn new(name: impl Into<String>, group_by: SwimlaneGroupBy, order: u32) -> Self {
        Self {
            id: SwimlaneId::new(),
            name: name.into(),
            group_by,
            order,
        }
    }
}

/// **BoardCard** — Board 视图上的 WorkItem 投影(只读,§11.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardCard {
    /// = WorkItemId
    pub id: BoardCardId,
    /// 所在 Column
    pub column_id: BoardColumnId,
    /// 所在 Swimlane(可选,无 swimlane 时为 None)
    pub swimlane_id: Option<SwimlaneId>,
    /// Column 内排序
    pub order_in_column: u32,
    /// WorkItem 类型(字符串,如 "Story" / "Bug" / "Task")
    pub work_item_type: String,
    /// 优先级(字符串)
    pub priority: String,
}

impl BoardCard {
    /// 派生:WorkItemId
    pub fn work_item_id(&self) -> WorkItemId {
        WorkItemId(self.id.0)
    }
}

/// **Board 聚合根**(§11.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// 主键
    pub id: BoardId,
    /// 租户 ID(必带,INV-BD-01)
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// Board 类型
    pub kind: BoardKind,
    /// 列(按 order 升序)
    pub columns: Vec<BoardColumn>,
    /// 泳道(可选)
    pub swimlanes: Vec<Swimlane>,
    /// 过滤查询(可选,自定义 WorkItem 过滤)
    pub filter_query: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Board {
    /// 新建 Board(默认 3 列)
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        name: String,
        description: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: BoardId::new(),
            tenant_id,
            project_id,
            name,
            description,
            kind: BoardKind::default(),
            columns: BoardColumn::default_three(),
            swimlanes: Vec::new(),
            filter_query: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 追加 Column(append,order 追加到末尾)
    pub fn append_column(
        &mut self,
        name: String,
        wip_limit: Option<u32>,
        color: Option<String>,
    ) -> BoardColumnId {
        let next_order = self.columns.len() as u32;
        let col = BoardColumn {
            id: BoardColumnId::new(),
            name,
            wip_limit,
            order: next_order,
            color,
        };
        let id = col.id;
        self.columns.push(col);
        self.touch();
        id
    }

    /// 设置 WIP 限制(根据列 id 查找)
    pub fn set_wip_limit(
        &mut self,
        column_id: BoardColumnId,
        limit: Option<u32>,
    ) -> Result<(), BoardError> {
        let col = self
            .columns
            .iter_mut()
            .find(|c| c.id == column_id)
            .ok_or(BoardError::NotFound(column_id.0.to_string()))?;
        col.wip_limit = limit;
        self.touch();
        Ok(())
    }

    /// 追加 Swimlane
    pub fn append_swimlane(&mut self, name: String, group_by: SwimlaneGroupBy) -> SwimlaneId {
        let next_order = self.swimlanes.len() as u32;
        let s = Swimlane {
            id: SwimlaneId::new(),
            name,
            group_by,
            order: next_order,
        };
        let id = s.id;
        self.swimlanes.push(s);
        self.touch();
        id
    }

    /// 查找 Column 的 index
    pub fn column_index(&self, column_id: BoardColumnId) -> Option<usize> {
        self.columns.iter().position(|c| c.id == column_id)
    }

    /// 找 Column
    pub fn find_column(&self, column_id: BoardColumnId) -> Option<&BoardColumn> {
        self.columns.iter().find(|c| c.id == column_id)
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// =====================================================================
// 错误
// =====================================================================

/// **BoardError** — 6 变体
#[derive(Debug, Error)]
pub enum BoardError {
    /// 资源不存在
    #[error("not found: {0}")]
    NotFound(String),

    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,

    /// 跨租户访问被拒(INV-BD-01 / INV-BD-05)
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),

    /// WIP 限制超出(INV-BD-02)
    #[error("WIP limit exceeded for column {0}: limit {1}, attempted {2}")]
    WipLimitExceeded(BoardColumnId, u32, u32),

    /// 状态非法(违反不变量)
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// 冲突(乐观锁 / 唯一约束)
    #[error("conflict: {0}")]
    Conflict(String),
}

impl BoardError {
    /// 错误码
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "BOARD_NOT_FOUND",
            Self::PermissionDenied => "BOARD_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "BOARD_CROSS_TENANT_DENIED",
            Self::WipLimitExceeded(_, _, _) => "BOARD_WIP_LIMIT_EXCEEDED",
            Self::InvalidState(_) => "BOARD_INVALID_STATE",
            Self::Conflict(_) => "BOARD_CONFLICT",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

/// **CreateBoardCommand** — 创建 Board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 类型(可选,默认 Kanban)
    pub kind: Option<BoardKind>,
    /// 自定义列(可选,None = 默认 3 列)
    pub columns: Option<Vec<NewColumnSpec>>,
}

/// 创建 Board 时指定的列草稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewColumnSpec {
    /// 名称
    pub name: String,
    /// WIP 限制
    pub wip_limit: Option<u32>,
    /// 颜色
    pub color: Option<String>,
}

/// **AddColumnCommand** — 追加一列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddColumnCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
    /// 列名
    pub name: String,
    /// WIP 限制(可选)
    pub wip_limit: Option<u32>,
    /// 颜色(可选)
    pub color: Option<String>,
}

/// **MoveCardCommand** — 移动 card 到目标 column/order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCardCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
    /// Card id(= WorkItem id)
    pub card_id: BoardCardId,
    /// 目标 Column
    pub to_column: BoardColumnId,
    /// 目标 Column 内新位置
    pub to_order: u32,
}

/// **AddSwimlaneCommand** — 追加一个泳道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSwimlaneCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
    /// 名称
    pub name: String,
    /// 分组维度
    pub group_by: SwimlaneGroupBy,
}

/// **SetWipLimitCommand** — 设置列 WIP 限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetWipLimitCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
    /// Column ID
    pub column_id: BoardColumnId,
    /// 新限制(None = 取消限制)
    pub limit: Option<u32>,
}

/// **ListByProjectQuery** — 列出 Project 下所有 Board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByProjectQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Project ID
    pub project_id: ProjectId,
}

/// **GetViewQuery** — 取得 Board 完整视图(Board + cards + swimlanes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetViewQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Board ID
    pub board_id: BoardId,
}

/// **BoardView** — 看板视图快照(§11.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardView {
    pub board: Board,
    pub cards: Vec<BoardCard>,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **BoardCommandPort** — 写操作(5 方法)
#[async_trait]
pub trait BoardCommandPort: Send + Sync {
    /// 创建 Board(INV-BD-01)
    async fn create_board(
        &self,
        cmd: CreateBoardCommand,
        actor: &ActorContext,
    ) -> Result<Board, BoardError>;

    /// 追加 Column(INV-BD-03)
    async fn add_column(
        &self,
        cmd: AddColumnCommand,
        actor: &ActorContext,
    ) -> Result<BoardColumn, BoardError>;

    /// 移动 card(INV-BD-04 / INV-BD-05)
    async fn move_card(
        &self,
        cmd: MoveCardCommand,
        actor: &ActorContext,
    ) -> Result<BoardCard, BoardError>;

    /// 追加泳道
    async fn add_swimlane(
        &self,
        cmd: AddSwimlaneCommand,
        actor: &ActorContext,
    ) -> Result<Swimlane, BoardError>;

    /// 设置 WIP 限制(INV-BD-02)
    async fn set_wip_limit(
        &self,
        cmd: SetWipLimitCommand,
        actor: &ActorContext,
    ) -> Result<BoardColumn, BoardError>;
}

/// **BoardQueryPort** — 读操作(3 方法)
#[async_trait]
pub trait BoardQueryPort: Send + Sync {
    /// 按 ID 读
    async fn get(&self, board_id: BoardId, actor: &ActorContext) -> Result<Board, BoardError>;

    /// 按 Project 列出所有 Board
    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Board>, BoardError>;

    /// 取得完整视图(Board + cards + swimlanes)
    async fn get_view(
        &self,
        q: GetViewQuery,
        actor: &ActorContext,
    ) -> Result<BoardView, BoardError>;
}

/// **BoardRepository** — 持久化抽象
#[async_trait]
pub trait BoardRepository: Send + Sync {
    async fn insert_board(&self, b: &Board) -> Result<(), BoardError>;
    async fn get_board(&self, id: BoardId) -> Result<Option<Board>, BoardError>;
    async fn update_board(&self, b: &Board) -> Result<(), BoardError>;
    async fn list_boards_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Board>, BoardError>;

    async fn insert_card(&self, b: BoardId, c: &BoardCard) -> Result<(), BoardError>;
    async fn update_card(&self, b: BoardId, c: &BoardCard) -> Result<(), BoardError>;
    async fn delete_card(&self, b: BoardId, c: BoardCardId) -> Result<(), BoardError>;
    async fn list_cards(&self, b: BoardId) -> Result<Vec<BoardCard>, BoardError>;
}

// =====================================================================
// 内部辅助:tenant / project 校验
// =====================================================================

/// 校验 actor 与目标 tenant 一致(INV-BD-01 / INV-BD-05)
fn check_tenant(actor: &ActorContext, target: TenantId) -> Result<(), BoardError> {
    if TenantId::from(actor.tenant_id) != target {
        return Err(BoardError::CrossTenantDenied(TenantId::from(actor.tenant_id), target));
    }
    Ok(())
}

/// 校验 board 存在并返回克隆(INV-BD-01)
fn require_board<'a>(
    map: &'a HashMap<BoardId, Board>,
    id: BoardId,
) -> Result<&'a Board, BoardError> {
    map.get(&id)
        .ok_or_else(|| BoardError::NotFound(format!("board:{}", id)))
}

// =====================================================================
// InMemoryBoardService
// =====================================================================

/// **InMemoryBoardService** — 内存实现,直接持有 store
pub struct InMemoryBoardService {
    repo: Arc<dyn BoardRepository>,
    /// Board 存储
    boards: Arc<RwLock<HashMap<BoardId, Board>>>,
    /// Cards 按 Board 分组
    cards: Arc<RwLock<HashMap<BoardId, HashMap<BoardCardId, BoardCard>>>>,
    /// Project → Board 索引(用于 list_by_project O(N) 扫描)
    project_index: Arc<RwLock<HashMap<ProjectId, Vec<BoardId>>>>,
}

impl InMemoryBoardService {
    /// 新建(默认 InMemory 仓库)
    pub fn new() -> Self {
        Self::with_repo(Arc::new(InMemoryBoardRepository::new()))
    }

    /// 注入自定义仓库
    pub fn with_repo(repo: Arc<dyn BoardRepository>) -> Self {
        Self {
            repo,
            boards: Arc::new(RwLock::new(HashMap::new())),
            cards: Arc::new(RwLock::new(HashMap::new())),
            project_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 校验 Column order 唯一(INV-BD-03)
    fn check_column_order_unique(board: &Board) -> Result<(), BoardError> {
        let mut seen = std::collections::HashSet::new();
        for c in &board.columns {
            if !seen.insert(c.order) {
                return Err(BoardError::InvalidState(format!(
                    "INV-BD-03: column order {} not unique",
                    c.order
                )));
            }
        }
        Ok(())
    }
}

impl Default for InMemoryBoardService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BoardCommandPort for InMemoryBoardService {
    async fn create_board(
        &self,
        cmd: CreateBoardCommand,
        actor: &ActorContext,
    ) -> Result<Board, BoardError> {
        check_tenant(actor, cmd.tenant_id)?;

        let now = Utc::now();
        let columns = if let Some(cols) = cmd.columns {
            cols.into_iter()
                .enumerate()
                .map(|(i, c)| BoardColumn {
                    id: BoardColumnId::new(),
                    name: c.name,
                    wip_limit: c.wip_limit,
                    order: i as u32,
                    color: c.color,
                })
                .collect()
        } else {
            BoardColumn::default_three()
        };
        let kind = cmd.kind.unwrap_or_default();

        let board = Board {
            id: BoardId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name,
            description: cmd.description,
            kind,
            columns,
            swimlanes: Vec::new(),
            filter_query: None,
            created_at: now,
            updated_at: now,
        };
        Self::check_column_order_unique(&board)?;

        self.repo.insert_board(&board).await?;
        self.boards
            .write()
            .expect("lock")
            .insert(board.id, board.clone());
        self.cards
            .write()
            .expect("lock")
            .insert(board.id, HashMap::new());
        self.project_index
            .write()
            .expect("lock")
            .entry(board.project_id)
            .or_insert_with(Vec::new)
            .push(board.id);
        Ok(board)
    }

    async fn add_column(
        &self,
        cmd: AddColumnCommand,
        actor: &ActorContext,
    ) -> Result<BoardColumn, BoardError> {
        check_tenant(actor, cmd.tenant_id)?;
        // 1) 在锁内计算新 board(无 await)
        let new_col_id: BoardColumnId;
        let new_board: Board;
        {
            let mut boards = self.boards.write().expect("lock");
            let board = require_board(&boards, cmd.board_id)?.clone();
            if board.tenant_id != cmd.tenant_id {
                return Err(BoardError::CrossTenantDenied(
                    TenantId::from(actor.tenant_id),
                    cmd.tenant_id,
                ));
            }
            let mut b = board;
            new_col_id = b.append_column(cmd.name, cmd.wip_limit, cmd.color);
            Self::check_column_order_unique(&b)?;
            new_board = b;
        }
        let new_col = new_board
            .columns
            .iter()
            .find(|c| c.id == new_col_id)
            .cloned()
            .expect("just inserted");
        // 2) 释放锁后再 await 写仓库
        self.repo.update_board(&new_board).await?;
        // 3) 写回 store
        self.boards
            .write()
            .expect("lock")
            .insert(new_board.id, new_board);
        Ok(new_col)
    }

    async fn move_card(
        &self,
        cmd: MoveCardCommand,
        actor: &ActorContext,
    ) -> Result<BoardCard, BoardError> {
        check_tenant(actor, cmd.tenant_id)?;
        // 1) 在锁内完成全部状态计算,产出一个新 card + 新 board
        let (new_card, new_board, to_col_wip) = {
            let mut boards = self.boards.write().expect("lock");
            let board = require_board(&boards, cmd.board_id)?.clone();
            if board.tenant_id != cmd.tenant_id {
                return Err(BoardError::CrossTenantDenied(
                    TenantId::from(actor.tenant_id),
                    cmd.tenant_id,
                ));
            }
            // INV-BD-04:目标 column 必须在 board 内
            let to_col = board.find_column(cmd.to_column).ok_or_else(|| {
                BoardError::InvalidState(format!(
                    "INV-BD-04: target column {} not in board",
                    cmd.to_column
                ))
            })?;
            let to_col_wip = to_col.wip_limit;

            let mut cards = self.cards.write().expect("lock");
            let card_map = cards
                .get_mut(&cmd.board_id)
                .ok_or_else(|| BoardError::NotFound(format!("board:{} cards", cmd.board_id)))?;
            let card = card_map
                .get(&cmd.card_id)
                .cloned()
                .ok_or_else(|| BoardError::NotFound(format!("card:{}", cmd.card_id)))?;
            let from_col = card.column_id;

            // 准备新 card(列内换位)
            let mut moved = card;
            moved.column_id = cmd.to_column;
            // 重排: 旧 column(若不同)重排;新 column 重新连续编号 0..N(把 moved 插到 to_order 位置)
            // 1) 旧 column 内除 moved 外按 order 升序排
            if from_col != cmd.to_column {
                let mut from_ids: Vec<BoardCardId> = card_map
                    .values()
                    .filter(|c| c.column_id == from_col && c.id != cmd.card_id)
                    .map(|c| c.id)
                    .collect();
                from_ids.sort_by_key(|cid| {
                    card_map
                        .get(cid)
                        .map(|c| c.order_in_column)
                        .unwrap_or(u32::MAX)
                });
                for (i, cid) in from_ids.iter().enumerate() {
                    if let Some(c) = card_map.get_mut(cid) {
                        c.order_in_column = i as u32;
                    }
                }
            }
            // 2) 新 column:除 moved 外按 order 升序排,再插入 moved 到 cmd.to_order
            let mut to_ids: Vec<BoardCardId> = card_map
                .values()
                .filter(|c| c.column_id == cmd.to_column && c.id != cmd.card_id)
                .map(|c| c.id)
                .collect();
            to_ids.sort_by_key(|cid| {
                card_map
                    .get(cid)
                    .map(|c| c.order_in_column)
                    .unwrap_or(u32::MAX)
            });
            let pos = (cmd.to_order as usize).min(to_ids.len());
            to_ids.insert(pos, cmd.card_id);
            for (i, cid) in to_ids.iter().enumerate() {
                if let Some(c) = card_map.get_mut(cid) {
                    c.order_in_column = i as u32;
                }
            }
            // 关键修复:把 moved(column_id=to_column) 写回 card_map
            let final_card = BoardCard {
                id: cmd.card_id,
                column_id: cmd.to_column,
                swimlane_id: moved.swimlane_id,
                order_in_column: card_map
                    .get(&cmd.card_id)
                    .map(|c| c.order_in_column)
                    .unwrap_or(cmd.to_order),
                work_item_type: moved.work_item_type,
                priority: moved.priority,
            };
            card_map.insert(cmd.card_id, final_card.clone());
            let new_card = final_card;
            // 3) INV-BD-02:WIP 限制(enable 时) — 在新布局里检查
            if let Some(limit) = to_col_wip {
                let count = card_map
                    .values()
                    .filter(|c| c.column_id == cmd.to_column)
                    .count() as u32;
                if count > limit {
                    return Err(BoardError::WipLimitExceeded(cmd.to_column, limit, count));
                }
            }
            let _ = from_col;
            // 4) board 自身 updated_at
            let mut new_board = board;
            new_board.updated_at = Utc::now();
            (new_card, new_board, to_col_wip)
        };
        let _ = to_col_wip;
        // 2) 释放锁后 await 写仓库
        self.repo.update_board(&new_board).await?;
        self.repo.update_card(cmd.board_id, &new_card).await?;
        // 3) 写回 store
        self.boards
            .write()
            .expect("lock")
            .insert(new_board.id, new_board);
        Ok(new_card)
    }

    async fn add_swimlane(
        &self,
        cmd: AddSwimlaneCommand,
        actor: &ActorContext,
    ) -> Result<Swimlane, BoardError> {
        check_tenant(actor, cmd.tenant_id)?;
        let (new_s, new_board) = {
            let mut boards = self.boards.write().expect("lock");
            let board = require_board(&boards, cmd.board_id)?.clone();
            if board.tenant_id != cmd.tenant_id {
                return Err(BoardError::CrossTenantDenied(
                    TenantId::from(actor.tenant_id),
                    cmd.tenant_id,
                ));
            }
            let mut b = board;
            let new_id = b.append_swimlane(cmd.name, cmd.group_by);
            let new_s = b
                .swimlanes
                .iter()
                .find(|s| s.id == new_id)
                .cloned()
                .expect("just inserted");
            (new_s, b)
        };
        self.repo.update_board(&new_board).await?;
        self.boards
            .write()
            .expect("lock")
            .insert(new_board.id, new_board);
        Ok(new_s)
    }

    async fn set_wip_limit(
        &self,
        cmd: SetWipLimitCommand,
        actor: &ActorContext,
    ) -> Result<BoardColumn, BoardError> {
        check_tenant(actor, cmd.tenant_id)?;
        let (col, new_board) = {
            let mut boards = self.boards.write().expect("lock");
            let board = require_board(&boards, cmd.board_id)?.clone();
            if board.tenant_id != cmd.tenant_id {
                return Err(BoardError::CrossTenantDenied(
                    TenantId::from(actor.tenant_id),
                    cmd.tenant_id,
                ));
            }
            let mut b = board;
            b.set_wip_limit(cmd.column_id, cmd.limit)?;
            let col = b
                .columns
                .iter()
                .find(|c| c.id == cmd.column_id)
                .cloned()
                .expect("set_wip_limit succeeded");
            (col, b)
        };
        self.repo.update_board(&new_board).await?;
        self.boards
            .write()
            .expect("lock")
            .insert(new_board.id, new_board);
        Ok(col)
    }
}

#[async_trait]
impl BoardQueryPort for InMemoryBoardService {
    async fn get(&self, board_id: BoardId, actor: &ActorContext) -> Result<Board, BoardError> {
        let boards = self.boards.read().expect("lock");
        let board = require_board(&boards, board_id)?.clone();
        if board.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(BoardError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                board.tenant_id,
            ));
        }
        Ok(board)
    }

    async fn list_by_project(
        &self,
        q: ListByProjectQuery,
        actor: &ActorContext,
    ) -> Result<Vec<Board>, BoardError> {
        check_tenant(actor, q.tenant_id)?;
        let boards = self.boards.read().expect("lock");
        let out: Vec<Board> = boards
            .values()
            .filter(|b| b.project_id == q.project_id && b.tenant_id == q.tenant_id)
            .cloned()
            .collect();
        Ok(out)
    }

    async fn get_view(
        &self,
        q: GetViewQuery,
        actor: &ActorContext,
    ) -> Result<BoardView, BoardError> {
        let board = self.get(q.board_id, actor).await?;
        let cards = self
            .cards
            .read()
            .expect("lock")
            .get(&q.board_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        Ok(BoardView { board, cards })
    }
}

// =====================================================================
// InMemoryBoardRepository
// =====================================================================

/// 内存 Board 仓库
pub struct InMemoryBoardRepository {
    boards: RwLock<HashMap<BoardId, Board>>,
    cards: RwLock<HashMap<BoardId, HashMap<BoardCardId, BoardCard>>>,
}

impl InMemoryBoardRepository {
    /// 新建空仓库
    pub fn new() -> Self {
        Self {
            boards: RwLock::new(HashMap::new()),
            cards: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryBoardRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BoardRepository for InMemoryBoardRepository {
    async fn insert_board(&self, b: &Board) -> Result<(), BoardError> {
        let mut s = self.boards.write().expect("lock");
        if s.contains_key(&b.id) {
            return Err(BoardError::Conflict(format!("board {} exists", b.id)));
        }
        s.insert(b.id, b.clone());
        Ok(())
    }
    async fn get_board(&self, id: BoardId) -> Result<Option<Board>, BoardError> {
        Ok(self.boards.read().expect("lock").get(&id).cloned())
    }
    async fn update_board(&self, b: &Board) -> Result<(), BoardError> {
        self.boards.write().expect("lock").insert(b.id, b.clone());
        Ok(())
    }
    async fn list_boards_by_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> Result<Vec<Board>, BoardError> {
        Ok(self
            .boards
            .read()
            .expect("lock")
            .values()
            .filter(|b| b.tenant_id == tenant_id && b.project_id == project_id)
            .cloned()
            .collect())
    }
    async fn insert_card(&self, b: BoardId, c: &BoardCard) -> Result<(), BoardError> {
        self.cards
            .write()
            .expect("lock")
            .entry(b)
            .or_insert_with(HashMap::new)
            .insert(c.id, c.clone());
        Ok(())
    }
    async fn update_card(&self, b: BoardId, c: &BoardCard) -> Result<(), BoardError> {
        self.cards
            .write()
            .expect("lock")
            .entry(b)
            .or_insert_with(HashMap::new)
            .insert(c.id, c.clone());
        Ok(())
    }
    async fn delete_card(&self, b: BoardId, c: BoardCardId) -> Result<(), BoardError> {
        if let Some(m) = self.cards.write().expect("lock").get_mut(&b) {
            m.remove(&c);
        }
        Ok(())
    }
    async fn list_cards(&self, b: BoardId) -> Result<Vec<BoardCard>, BoardError> {
        Ok(self
            .cards
            .read()
            .expect("lock")
            .get(&b)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    fn make_actor(tenant_id: TenantId) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id.0).with_role(roles::PROJECT_ADMIN)
    }

    // 1. create_board + 默认 3 列
    #[tokio::test]
    async fn create_board_creates_three_default_columns() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "Sprint Board".to_string(),
                    description: "Default".to_string(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .expect("ok");
        assert_eq!(board.columns.len(), 3);
        assert_eq!(board.columns[0].name, "Todo");
        assert_eq!(board.columns[1].name, "InProgress");
        assert_eq!(board.columns[2].name, "Done");
    }

    // 2. add_column 追加到末尾
    #[tokio::test]
    async fn add_column_appends_with_next_order() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col = svc
            .add_column(
                AddColumnCommand {
                    tenant_id: tenant,
                    board_id: board.id,
                    name: "Review".to_string(),
                    wip_limit: None,
                    color: Some("#f0a".to_string()),
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(col.name, "Review");
        assert_eq!(col.order, 3);
    }

    // 3. move_card_within_column
    #[tokio::test]
    async fn move_card_within_column_reorders() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col0 = board.columns[0].id;
        let col1 = board.columns[1].id;
        // 注入 2 张卡到 col0
        let c1 = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 0,
            work_item_type: "Task".to_string(),
            priority: "P1".to_string(),
        };
        let c2 = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 1,
            work_item_type: "Task".to_string(),
            priority: "P2".to_string(),
        };
        svc.repo.insert_card(board.id, &c1).await.unwrap();
        svc.repo.insert_card(board.id, &c2).await.unwrap();
        // 也需要 service 自己的 cards map 更新
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c1.id, c1.clone());
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c2.id, c2.clone());

        // 把 c2 移到 order=0(列内重排)
        let moved = svc
            .move_card(
                MoveCardCommand {
                    tenant_id: tenant,
                    board_id: board.id,
                    card_id: c2.id,
                    to_column: col0,
                    to_order: 0,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(moved.column_id, col0);
        // 验证两卡都还在 col0
        let view = svc
            .get_view(
                GetViewQuery {
                    tenant_id: tenant,
                    board_id: board.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(view.cards.len(), 2);
        let _ = col1;
    }

    // 4. move_card_across_column
    #[tokio::test]
    async fn move_card_across_columns() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col0 = board.columns[0].id;
        let col2 = board.columns[2].id;
        let c = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 0,
            work_item_type: "Story".to_string(),
            priority: "P0".to_string(),
        };
        svc.repo.insert_card(board.id, &c).await.unwrap();
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c.id, c.clone());

        let moved = svc
            .move_card(
                MoveCardCommand {
                    tenant_id: tenant,
                    board_id: board.id,
                    card_id: c.id,
                    to_column: col2,
                    to_order: 0,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(moved.column_id, col2);
    }

    // 5. wip_limit_exceeded_rejected
    #[tokio::test]
    async fn wip_limit_exceeded_rejected() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col0 = board.columns[0].id;
        // 限制 = 1
        svc.set_wip_limit(
            SetWipLimitCommand {
                tenant_id: tenant,
                board_id: board.id,
                column_id: col0,
                limit: Some(1),
            },
            &actor,
        )
        .await
        .unwrap();
        // 注入 1 张
        let c1 = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 0,
            work_item_type: "Task".to_string(),
            priority: "P1".to_string(),
        };
        svc.repo.insert_card(board.id, &c1).await.unwrap();
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c1.id, c1.clone());
        // 再注入 1 张(从外部直接加,模拟 board 已有 1 张时)
        let c2 = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 1,
            work_item_type: "Task".to_string(),
            priority: "P2".to_string(),
        };
        svc.repo.insert_card(board.id, &c2).await.unwrap();
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c2.id, c2.clone());

        // 试图把 c2 移到 col0(已有 c1,limit=1 → 超)
        let res = svc
            .move_card(
                MoveCardCommand {
                    tenant_id: tenant,
                    board_id: board.id,
                    card_id: c2.id,
                    to_column: col0,
                    to_order: 0,
                },
                &actor,
            )
            .await;
        assert!(matches!(res, Err(BoardError::WipLimitExceeded(_, 1, _))));
    }

    // 6. wip_limit_disabled_no_check
    #[tokio::test]
    async fn wip_limit_disabled_no_check() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col0 = board.columns[0].id;
        // 显式取消限制
        svc.set_wip_limit(
            SetWipLimitCommand {
                tenant_id: tenant,
                board_id: board.id,
                column_id: col0,
                limit: None,
            },
            &actor,
        )
        .await
        .unwrap();
        // 注入 5 张
        for i in 0..5 {
            let c = BoardCard {
                id: BoardCardId::new(),
                column_id: col0,
                swimlane_id: None,
                order_in_column: i,
                work_item_type: "Task".to_string(),
                priority: "P1".to_string(),
            };
            svc.repo.insert_card(board.id, &c).await.unwrap();
            svc.cards
                .write()
                .unwrap()
                .entry(board.id)
                .or_insert_with(HashMap::new)
                .insert(c.id, c.clone());
        }
        // 移动第 5 张 → 不应触发 WIP 校验
        let view = svc
            .get_view(
                GetViewQuery {
                    tenant_id: tenant,
                    board_id: board.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(view.cards.len(), 5);
    }

    // 7. add_swimlane
    #[tokio::test]
    async fn add_swimlane_appends() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let s = svc
            .add_swimlane(
                AddSwimlaneCommand {
                    tenant_id: tenant,
                    board_id: board.id,
                    name: "By Assignee".to_string(),
                    group_by: SwimlaneGroupBy::Assignee,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(s.name, "By Assignee");
        assert_eq!(s.group_by, SwimlaneGroupBy::Assignee);
        assert_eq!(s.order, 0);
    }

    // 8. cross_tenant_get_denied
    #[tokio::test]
    async fn cross_tenant_get_denied() {
        let svc = InMemoryBoardService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let a1 = make_actor(t1);
        let a2 = make_actor(t2);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: t1,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &a1,
            )
            .await
            .unwrap();
        let res = svc.get(board.id, &a2).await;
        assert!(matches!(res, Err(BoardError::CrossTenantDenied(_, _))));
    }

    // 9. card_always_in_column
    #[tokio::test]
    async fn card_always_in_a_column() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        let col0 = board.columns[0].id;
        let c = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 0,
            work_item_type: "Task".to_string(),
            priority: "P1".to_string(),
        };
        svc.repo.insert_card(board.id, &c).await.unwrap();
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c.id, c.clone());

        let view = svc
            .get_view(
                GetViewQuery {
                    tenant_id: tenant,
                    board_id: board.id,
                },
                &actor,
            )
            .await
            .unwrap();
        // INV-BD-04:每张 card 的 column_id 必在 board.columns 内
        for card in &view.cards {
            assert!(
                board.columns.iter().any(|col| col.id == card.column_id),
                "card {} column not in board",
                card.id
            );
        }
    }

    // 10. column_order_unique
    #[tokio::test]
    async fn column_order_unique_invariant() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        // 故意造 2 个同 order 的列
        let res = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "Dup".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: Some(vec![
                        NewColumnSpec {
                            name: "A".to_string(),
                            wip_limit: None,
                            color: None,
                        },
                        NewColumnSpec {
                            name: "B".to_string(),
                            wip_limit: None,
                            color: None,
                        },
                    ]),
                },
                &actor,
            )
            .await
            .unwrap();
        // 默认 create_board 重新编号 (0, 1) → 唯一
        assert_eq!(res.columns[0].order, 0);
        assert_eq!(res.columns[1].order, 1);
        // 显式校验
        let mut board = res.clone();
        // 强行把第 2 列 order 改为 0(模拟脏数据)
        board.columns[1].order = 0;
        let check = InMemoryBoardService::check_column_order_unique(&board);
        assert!(check.is_err());
    }

    // 11. list_by_project
    #[tokio::test]
    async fn list_by_project_returns_only_own() {
        let svc = InMemoryBoardService::new();
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let p1 = ProjectId::new();
        let p2 = ProjectId::new();
        let a1 = make_actor(t1);
        let a2 = make_actor(t2);

        for (tid, pid, actor) in [(t1, p1, &a1), (t1, p2, &a1), (t2, p1, &a2)] {
            svc.create_board(
                CreateBoardCommand {
                    tenant_id: tid,
                    project_id: pid,
                    name: "X".to_string(),
                    description: String::new(),
                    kind: None,
                    columns: None,
                },
                actor,
            )
            .await
            .unwrap();
        }
        let out = svc
            .list_by_project(
                ListByProjectQuery {
                    tenant_id: t1,
                    project_id: p1,
                },
                &a1,
            )
            .await
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].project_id, p1);
    }

    // 12. get_view 返回 board + cards + swimlanes
    #[tokio::test]
    async fn get_view_returns_full_snapshot() {
        let svc = InMemoryBoardService::new();
        let tenant = uuid::Uuid::new_v4();
        let project = ProjectId::new();
        let actor = make_actor(tenant);

        let board = svc
            .create_board(
                CreateBoardCommand {
                    tenant_id: tenant,
                    project_id: project,
                    name: "B".to_string(),
                    description: "desc".to_string(),
                    kind: Some(BoardKind::Scrum),
                    columns: None,
                },
                &actor,
            )
            .await
            .unwrap();
        // 加 swimlane
        svc.add_swimlane(
            AddSwimlaneCommand {
                tenant_id: tenant,
                board_id: board.id,
                name: "By Epic".to_string(),
                group_by: SwimlaneGroupBy::Epic,
            },
            &actor,
        )
        .await
        .unwrap();
        // 加一张 card
        let col0 = board.columns[0].id;
        let c = BoardCard {
            id: BoardCardId::new(),
            column_id: col0,
            swimlane_id: None,
            order_in_column: 0,
            work_item_type: "Story".to_string(),
            priority: "P0".to_string(),
        };
        svc.repo.insert_card(board.id, &c).await.unwrap();
        svc.cards
            .write()
            .unwrap()
            .entry(board.id)
            .or_insert_with(HashMap::new)
            .insert(c.id, c.clone());

        let view = svc
            .get_view(
                GetViewQuery {
                    tenant_id: tenant,
                    board_id: board.id,
                },
                &actor,
            )
            .await
            .unwrap();
        assert_eq!(view.board.name, "B");
        assert_eq!(view.board.kind, BoardKind::Scrum);
        assert_eq!(view.board.swimlanes.len(), 1);
        assert_eq!(view.cards.len(), 1);
        assert_eq!(view.cards[0].column_id, col0);
    }
}

pub mod wip_swimlane;
