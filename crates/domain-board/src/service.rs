//! InMemoryBoardService:Phase 2 提供的内存实现
//!
//! 来源: spec/domain-board-spec.md §5(实施策略)

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{Board, Column, Swimlane};
use crate::error::BoardError;
use crate::event::{BoardEvent, BoardPatched, BoardReplaced, ColumnReordered, EventMeta};
use crate::invariants::{
    check_create_invariants, check_invariant_02_column_state_exists,
    check_invariant_03_display_order_unique,
};
use crate::port::{
    BoardCommandPort, BoardQueryPort, BoardRepository, ColumnDraft, ColumnOrderUpdate,
    ListColumnsQuery, ListSwimlanesQuery, PatchBoardCommand, ReplaceBoardCommand, SwimlaneDraft,
};
use crate::value_object::{
    BoardId, ColumnId, ProjectId, StateId, SwimlaneId, TenantId,
};

// =====================================================================
// InMemoryBoardService
// =====================================================================

/// **InMemory Board 命令/查询服务**(Phase 2 真实实现)
pub struct InMemoryBoardService {
    /// Board 存储
    boards: Arc<RwLock<HashMap<BoardId, Board>>>,
    /// Column 存储(按 Board 分组)
    columns: Arc<RwLock<HashMap<BoardId, HashMap<ColumnId, Column>>>>,
    /// Swimlane 存储
    swimlanes: Arc<RwLock<HashMap<BoardId, HashMap<SwimlaneId, Swimlane>>>>,
    /// 已使用 State 集合(用于 INV-B-02;实际生产中由 application 层从 workflow 查询)
    valid_states: Arc<RwLock<HashSet<StateId>>>,
    /// 事件发送器
    event_tx: mpsc::UnboundedSender<BoardEvent>,
}

impl InMemoryBoardService {
    /// 创建新服务
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<BoardEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            boards: Arc::new(RwLock::new(HashMap::new())),
            columns: Arc::new(RwLock::new(HashMap::new())),
            swimlanes: Arc::new(RwLock::new(HashMap::new())),
            valid_states: Arc::new(RwLock::new(HashSet::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    /// 仅创建服务(事件接收端丢弃)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    /// 当前 Board 数量
    pub async fn count(&self) -> usize {
        self.boards.read().expect("lock").len()
    }

    /// 校验 actor 与命令的 tenant_id 一致
    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), BoardError> {
        if actor.tenant_id != expected {
            return Err(BoardError::PermissionDenied);
        }
        Ok(())
    }

    /// 注册合法 State IDs(由 application 层从 workflow 同步过来,INV-B-02)
    pub async fn register_valid_states(&self, state_ids: Vec<StateId>) {
        let mut s = self.valid_states.write().expect("lock");
        for sid in state_ids {
            s.insert(sid);
        }
    }

    /// 清空合法 State IDs
    pub async fn clear_valid_states(&self) {
        self.valid_states.write().expect("lock").clear();
    }
}

impl Default for InMemoryBoardService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryBoardService {
    fn clone(&self) -> Self {
        Self {
            boards: self.boards.clone(),
            columns: self.columns.clone(),
            swimlanes: self.swimlanes.clone(),
            valid_states: self.valid_states.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// BoardCommandPort 实现
// =====================================================================

#[async_trait]
impl BoardCommandPort for InMemoryBoardService {
    async fn replace_board(
        &self,
        cmd: ReplaceBoardCommand,
        actor: ActorContext,
    ) -> Result<Board, BoardError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // 1. 检查 Project 唯一(INV-B-01 同 Project 1:1)
        if cmd.expected_version == 0 {
            // 创建路径
            let boards = self.boards.read().expect("lock");
            if boards.values().any(|b| b.project_id == cmd.project_id) {
                return Err(BoardError::Conflict(
                    "INV-B-01: Project 已有 Board(同 Project 一对一)".to_string(),
                ));
            }
        }

        // 2. 构造 Column 草稿
        let now = chrono::Utc::now();
        let draft_to_col: HashMap<uuid::Uuid, ColumnId> = cmd
            .columns
            .iter()
            .map(|d| (d.draft_id, ColumnId::new()))
            .collect();
        let columns: Vec<Column> = cmd
            .columns
            .iter()
            .map(|d: &ColumnDraft| Column {
                id: draft_to_col[&d.draft_id],
                board_id: BoardId::from_uuid(uuid::Uuid::nil()), // 稍后覆盖
                tenant_id: cmd.tenant_id,
                name: d.name.clone(),
                state_id: d.state_id,
                display_order: d.display_order,
                wip_limit: d.wip_limit,
                display_color: d.display_color.clone(),
                created_at: now,
                updated_at: now,
            })
            .collect();

        // 3. 构造 Swimlane 草稿
        let draft_to_swim: HashMap<uuid::Uuid, SwimlaneId> = cmd
            .swimlanes
            .iter()
            .map(|d| (d.draft_id, SwimlaneId::new()))
            .collect();
        let swimlanes: Vec<Swimlane> = cmd
            .swimlanes
            .iter()
            .map(|d: &SwimlaneDraft| Swimlane {
                id: draft_to_swim[&d.draft_id],
                board_id: BoardId::from_uuid(uuid::Uuid::nil()),
                tenant_id: cmd.tenant_id,
                name: d.name.clone(),
                group_by_field: d.group_by_field,
                display_order: d.display_order,
                created_at: now,
                updated_at: now,
            })
            .collect();

        // 4. 不变量校验(INV-B-02 引用,INV-B-03 UNIQUE,INV-B-04/05)
        let valid_states: Vec<StateId> =
            self.valid_states.read().expect("lock").iter().copied().collect();
        check_invariant_02_column_state_exists(&columns, &valid_states)?;

        // 5. 创建/更新 Board 实体
        let board = if cmd.expected_version == 0 {
            // 新建
            let new_board = Board {
                id: BoardId::new(),
                tenant_id: cmd.tenant_id,
                project_id: cmd.project_id,
                board_type: cmd.board_type,
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                filter_assignee: cmd.filter_assignee,
                filter_label: cmd.filter_label.clone(),
                created_at: now,
                updated_at: now,
                lock_version: 1,
            };
            check_create_invariants(&new_board, &columns, &swimlanes)?;
            new_board
        } else {
            // 更新:取出原 board 校验 + 应用变更
            let mut boards = self.boards.write().expect("lock");
            let existing = boards
                .values()
                .find(|b| b.project_id == cmd.project_id && b.tenant_id == cmd.tenant_id)
                .cloned()
                .ok_or_else(|| {
                    BoardError::NotFound(BoardId::from_uuid(uuid::Uuid::nil()))
                })?;
            if existing.lock_version != cmd.expected_version {
                return Err(BoardError::Conflict(format!(
                    "lock_version mismatch: expected={}, actual={}",
                    cmd.expected_version, existing.lock_version
                )));
            }
            let mut new_board = existing;
            new_board.board_type = cmd.board_type;
            new_board.name = cmd.name.clone();
            new_board.description = cmd.description.clone();
            new_board.filter_assignee = cmd.filter_assignee;
            new_board.filter_label = cmd.filter_label.clone();
            new_board.bump_version();
            check_create_invariants(&new_board, &columns, &swimlanes)?;
            boards.insert(new_board.id, new_board.clone());
            drop(boards);
            new_board
        };

        // 6. 覆盖 Column / Swimlane 的 board_id
        let final_columns: Vec<Column> = columns
            .into_iter()
            .map(|mut c| {
                c.board_id = board.id;
                c
            })
            .collect();
        let final_swimlanes: Vec<Swimlane> = swimlanes
            .into_iter()
            .map(|mut s| {
                s.board_id = board.id;
                s
            })
            .collect();

        // 7. 替换 Column / Swimlane
        let mut col_map: HashMap<ColumnId, Column> = HashMap::new();
        for c in final_columns {
            col_map.insert(c.id, c);
        }
        self.columns
            .write()
            .expect("lock")
            .insert(board.id, col_map);
        let mut swim_map: HashMap<SwimlaneId, Swimlane> = HashMap::new();
        for s in final_swimlanes {
            swim_map.insert(s.id, s);
        }
        self.swimlanes
            .write()
            .expect("lock")
            .insert(board.id, swim_map);

        // 8. 保存 Board(若新建)
        if cmd.expected_version == 0 {
            self.boards
                .write()
                .expect("lock")
                .insert(board.id, board.clone());
        }

        // 9. 事件
        let event = BoardEvent::Replaced(BoardReplaced {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            board_id: board.id,
            project_id: board.project_id,
            version: board.lock_version,
        });
        let _ = self.event_tx.send(event);

        Ok(board)
    }

    async fn patch_board(
        &self,
        cmd: PatchBoardCommand,
        actor: ActorContext,
    ) -> Result<Board, BoardError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut boards = self.boards.write().expect("lock");
        let board = boards
            .get_mut(&cmd.board_id)
            .ok_or(BoardError::NotFound(cmd.board_id))?;
        if board.tenant_id != cmd.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        if board.lock_version != cmd.expected_version {
            return Err(BoardError::Conflict(format!(
                "lock_version mismatch: expected={}, actual={}",
                cmd.expected_version, board.lock_version
            )));
        }
        let mut patched_fields = Vec::new();
        if let Some(n) = cmd.name {
            board.name = n;
            patched_fields.push("name".to_string());
        }
        if let Some(d) = cmd.description {
            board.description = d;
            patched_fields.push("description".to_string());
        }
        if let Some(bt) = cmd.board_type {
            board.board_type = bt;
            patched_fields.push("board_type".to_string());
        }
        if let Some(a) = cmd.filter_assignee {
            board.filter_assignee = a;
            patched_fields.push("filter_assignee".to_string());
        }
        if let Some(l) = cmd.filter_label {
            board.filter_label = l;
            patched_fields.push("filter_label".to_string());
        }
        board.bump_version();

        let event = BoardEvent::Patched(BoardPatched {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            board_id: board.id,
            patched_fields,
        });
        let _ = self.event_tx.send(event);

        Ok(board.clone())
    }

    async fn reorder_columns(
        &self,
        cmd: ColumnOrderUpdate,
        actor: ActorContext,
    ) -> Result<Vec<Column>, BoardError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let boards = self.boards.read().expect("lock");
        let board = boards
            .get(&cmd.board_id)
            .ok_or(BoardError::NotFound(cmd.board_id))?;
        if board.tenant_id != cmd.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        drop(boards);

        // 校验所有 column_id 属于此 board
        let mut cols_map = self.columns.write().expect("lock");
        let map = cols_map
            .get_mut(&cmd.board_id)
            .ok_or_else(|| BoardError::InvalidState("Board 无 Column".to_string()))?;

        // 应用新顺序
        for (cid, new_order) in &cmd.new_orders {
            if let Some(c) = map.get_mut(cid) {
                c.display_order = *new_order;
            } else {
                return Err(BoardError::InvalidState(format!(
                    "Column {cid} 不属于此 Board"
                )));
            }
        }
        // 校验 UNIQUE
        let all: Vec<Column> = map.values().cloned().collect();
        check_invariant_03_display_order_unique(&all)?;

        let event = BoardEvent::ColumnReordered(ColumnReordered {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            board_id: cmd.board_id,
            new_order: cmd.new_orders.iter().map(|(cid, _)| *cid).collect(),
            actor_user_id: actor.user_id,
        });
        let _ = self.event_tx.send(event);

        let mut out = all;
        out.sort_by_key(|c| c.display_order);
        Ok(out)
    }
}

// =====================================================================
// BoardQueryPort 实现
// =====================================================================

#[async_trait]
impl BoardQueryPort for InMemoryBoardService {
    async fn get_by_project(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Board, BoardError> {
        let boards = self.boards.read().expect("lock");
        let board = boards
            .values()
            .find(|b| b.project_id == project_id)
            .cloned()
            .ok_or_else(|| BoardError::NotFound(BoardId::from_uuid(uuid::Uuid::nil())))?;
        if board.tenant_id != viewer.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        Ok(board)
    }

    async fn list_columns(
        &self,
        q: ListColumnsQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Column>, BoardError> {
        let boards = self.boards.read().expect("lock");
        let board = boards
            .get(&q.board_id)
            .ok_or(BoardError::NotFound(q.board_id))?;
        if board.tenant_id != q.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        if board.tenant_id != viewer.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        drop(boards);
        let cols = self.columns.read().expect("lock");
        let mut out: Vec<Column> = cols
            .get(&q.board_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        out.sort_by_key(|c| c.display_order);
        Ok(out)
    }

    async fn list_swimlanes(
        &self,
        q: ListSwimlanesQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Swimlane>, BoardError> {
        let boards = self.boards.read().expect("lock");
        let board = boards
            .get(&q.board_id)
            .ok_or(BoardError::NotFound(q.board_id))?;
        if board.tenant_id != q.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        if board.tenant_id != viewer.tenant_id {
            return Err(BoardError::PermissionDenied);
        }
        drop(boards);
        let swim = self.swimlanes.read().expect("lock");
        let mut out: Vec<Swimlane> = swim
            .get(&q.board_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        out.sort_by_key(|s| s.display_order);
        Ok(out)
    }
}

// =====================================================================
// BoardRepository 实现
// =====================================================================

#[async_trait]
impl BoardRepository for InMemoryBoardService {
    async fn insert(&self, board: &Board) -> Result<(), BoardError> {
        self.boards.write().expect("lock").insert(board.id, board.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: BoardId) -> Result<Option<Board>, BoardError> {
        Ok(self.boards.read().expect("lock").get(&id).cloned())
    }

    async fn update(&self, board: &Board) -> Result<(), BoardError> {
        self.boards.write().expect("lock").insert(board.id, board.clone());
        Ok(())
    }

    async fn find_by_project(&self, project_id: ProjectId) -> Result<Option<Board>, BoardError> {
        Ok(self
            .boards
            .read()
            .expect("lock")
            .values()
            .find(|b| b.project_id == project_id)
            .cloned())
    }

    async fn insert_column(&self, col: &Column) -> Result<(), BoardError> {
        self.columns
            .write()
            .expect("lock")
            .entry(col.board_id)
            .or_insert_with(HashMap::new)
            .insert(col.id, col.clone());
        Ok(())
    }

    async fn replace_columns(
        &self,
        board_id: BoardId,
        cols: &[Column],
    ) -> Result<(), BoardError> {
        let map: HashMap<ColumnId, Column> = cols.iter().map(|c| (c.id, c.clone())).collect();
        self.columns.write().expect("lock").insert(board_id, map);
        Ok(())
    }

    async fn list_columns_raw(&self, board_id: BoardId) -> Result<Vec<Column>, BoardError> {
        Ok(self
            .columns
            .read()
            .expect("lock")
            .get(&board_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn insert_swimlane(&self, s: &Swimlane) -> Result<(), BoardError> {
        self.swimlanes
            .write()
            .expect("lock")
            .entry(s.board_id)
            .or_insert_with(HashMap::new)
            .insert(s.id, s.clone());
        Ok(())
    }

    async fn replace_swimlanes(
        &self,
        board_id: BoardId,
        swimlanes: &[Swimlane],
    ) -> Result<(), BoardError> {
        let map: HashMap<SwimlaneId, Swimlane> = swimlanes.iter().map(|s| (s.id, s.clone())).collect();
        self.swimlanes.write().expect("lock").insert(board_id, map);
        Ok(())
    }

    async fn list_swimlanes_raw(&self, board_id: BoardId) -> Result<Vec<Swimlane>, BoardError> {
        Ok(self
            .swimlanes
            .read()
            .expect("lock")
            .get(&board_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default())
    }
}
