//! Star Agent Task Windows (精简实装 v0.1)
//!
//! Per 2026-08-29 04:09 JST 上轮拍板 + 09:07 JST 本轮拍板:
//! - 新页面 frontend/src/app/(app)/agent-windows
//! - 每 worktree 多 CLI session 并行 (tab)
//! - 6 内置 CLI + 自定义 (claude/codex/openclaw/hermes/gemini/aider)
//! - 三触发上传: 成功退出 / 手动 / 定时轮询
//!
//! 8 层结构 (单文件 4 层精简模式, per w11-w14 经验):
//! - value_object: TriggerMode, UploadTask, Window/Tab 状态
//! - entity: Window + Tab 聚合
//! - error: 6 错误变体
//! - service: WindowService (开窗/关窗/添加 tab/触发上传)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object — 触发模式 + 上传任务
// =====================================================================

/// 上传触发模式 (per 2026-08-29 04:09 JST 用户拍板三触发)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerMode {
    /// CLI 成功退出 0 自动触发
    OnSuccessExit,
    /// 手动点击 "Upload" 按钮
    Manual,
    /// 定时轮询 (例如 5 分钟一次)
    Polling,
}

impl TriggerMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::OnSuccessExit => "成功退出时自动上传",
            Self::Manual => "手动上传",
            Self::Polling => "定时轮询",
        }
    }
}

/// 上传任务 (单次 commit)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadTask {
    pub id: Uuid,
    pub window_id: Uuid,
    pub tab_id: Uuid,
    pub worktree_id: Uuid,
    pub trigger: TriggerMode,
    pub files_changed: Vec<String>,
    pub commit_message: String,
    pub status: UploadStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStatus {
    Pending,
    Committing,
    Completed,
    Failed,
}

// =====================================================================
// 2. entity — Window + Tab 状态
// =====================================================================

/// 任务窗口 (per Tab 上独立 CLI session)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskWindow {
    pub id: Uuid,
    pub name: String,
    pub worktree_id: Uuid,
    pub default_profile_id: Uuid,
    pub tabs: Vec<TaskTab>,
    pub active_tab_id: Option<Uuid>,
    pub upload_trigger: TriggerMode,
    pub polling_interval_sec: u32, // for TriggerMode::Polling
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 任务 Tab
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskTab {
    pub id: Uuid,
    pub window_id: Uuid,
    pub profile_id: Uuid,
    pub label: String,
    pub state: TabState,
    pub last_output: String, // 最近 N 行 (前端展示, 限 200 行)
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabState {
    Created,
    Running,
    WaitingInput,
    Completed,
    Failed,
    Aborted,
}

impl TaskWindow {
    pub fn new(
        name: impl Into<String>,
        worktree_id: Uuid,
        default_profile_id: Uuid,
        upload_trigger: TriggerMode,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            worktree_id,
            default_profile_id,
            tabs: Vec::new(),
            active_tab_id: None,
            upload_trigger,
            polling_interval_sec: 300, // 5 min default
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_tab(&mut self, profile_id: Uuid, label: impl Into<String>) -> Result<&TaskTab, WindowError> {
        if self.tabs.len() >= 20 {
            return Err(WindowError::TooManyTabs(20));
        }
        let tab = TaskTab {
            id: Uuid::new_v4(),
            window_id: self.id,
            profile_id,
            label: label.into(),
            state: TabState::Created,
            last_output: String::new(),
            started_at: Utc::now(),
            finished_at: None,
            exit_code: None,
            files_changed: Vec::new(),
        };
        let id = tab.id;
        self.active_tab_id.get_or_insert(id);
        self.tabs.push(tab);
        self.updated_at = Utc::now();
        Ok(self.tabs.last().unwrap())
    }

    pub fn close_tab(&mut self, tab_id: Uuid) -> Result<(), WindowError> {
        let before = self.tabs.len();
        self.tabs.retain(|t| t.id != tab_id);
        if self.tabs.len() == before {
            return Err(WindowError::TabNotFound(tab_id));
        }
        if self.active_tab_id == Some(tab_id) {
            self.active_tab_id = self.tabs.first().map(|t| t.id);
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_tab_state(
        &mut self,
        tab_id: Uuid,
        state: TabState,
        exit_code: Option<i32>,
    ) -> Result<(), WindowError> {
        let tab = self.tabs.iter_mut().find(|t| t.id == tab_id)
            .ok_or(WindowError::TabNotFound(tab_id))?;
        tab.state = state;
        tab.exit_code = exit_code;
        if matches!(state, TabState::Completed | TabState::Failed | TabState::Aborted) {
            tab.finished_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
        Ok(())
    }
}

// =====================================================================
// 3. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum WindowError {
    #[error("Tab 不存在: {0}")]
    TabNotFound(Uuid),
    #[error("Window 不存在: {0}")]
    WindowNotFound(Uuid),
    #[error("Tab 数量超限: max {0}")]
    TooManyTabs(usize),
    #[error("Worktree 不存在: {0}")]
    WorktreeNotFound(Uuid),
    #[error("上传失败: {0}")]
    UploadFailed(String),
    #[error("触发模式不匹配: 期望 {0:?}, 实际 {1:?}")]
    TriggerMismatch(TriggerMode, TriggerMode),
    #[error("未指定 profile_id")]
    ProfileIdMissing,
}

// =====================================================================
// 4. service — WindowService
// =====================================================================

pub struct WindowService {
    /// 内存 store
    windows: std::sync::RwLock<std::collections::HashMap<Uuid, TaskWindow>>,
    upload_tasks: std::sync::RwLock<Vec<UploadTask>>,
    /// CLI 端口 trait (Phase 2 接 w19 local-runtime)
    cli_port: Option<Arc<dyn CliPort>>,
}

pub trait CliPort: Send + Sync {
    async fn run_cli(
        &self,
        profile_id: Uuid,
        prompt: &str,
        worktree_id: Uuid,
    ) -> Result<RunResult, WindowError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResult {
    pub stdout: String,
    pub exit_code: i32,
    pub files_changed: Vec<String>,
}

use std::sync::Arc;

impl WindowService {
    pub fn new() -> Self {
        Self {
            windows: std::sync::RwLock::new(std::collections::HashMap::new()),
            upload_tasks: std::sync::RwLock::new(Vec::new()),
            cli_port: None,
        }
    }

    pub fn with_cli_port(mut self, port: Arc<dyn CliPort>) -> Self {
        self.cli_port = Some(port);
        self
    }

    pub fn open_window(&self, window: TaskWindow) -> Result<Uuid, WindowError> {
        let id = window.id;
        self.windows.write().unwrap().insert(id, window);
        Ok(id)
    }

    pub fn close_window(&self, id: Uuid) -> Result<(), WindowError> {
        self.windows.write().unwrap().remove(&id).ok_or(WindowError::WindowNotFound(id))?;
        Ok(())
    }

    pub fn get_window(&self, id: Uuid) -> Option<TaskWindow> {
        self.windows.read().unwrap().get(&id).cloned()
    }

    pub fn list_windows_by_worktree(&self, worktree_id: Uuid) -> Vec<TaskWindow> {
        self.windows.read().unwrap().values()
            .filter(|w| w.worktree_id == worktree_id)
            .cloned()
            .collect()
    }

    /// 触发上传 (per 2026-08-29 04:09 JST 三触发模式)
    pub fn trigger_upload(
        &self,
        window_id: Uuid,
        tab_id: Uuid,
        files_changed: Vec<String>,
        commit_message: impl Into<String>,
        trigger: TriggerMode,
    ) -> Result<UploadTask, WindowError> {
        let window = self.get_window(window_id).ok_or(WindowError::WindowNotFound(id_()))?;
        if window.upload_trigger != trigger {
            return Err(WindowError::TriggerMismatch(window.upload_trigger, trigger));
        }
        let tab = window.tabs.iter().find(|t| t.id == tab_id)
            .ok_or(WindowError::TabNotFound(tab_id))?;
        let task = UploadTask {
            id: Uuid::new_v4(),
            window_id,
            tab_id,
            worktree_id: window.worktree_id,
            trigger,
            files_changed,
            commit_message: commit_message.into(),
            status: UploadStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
        };
        self.upload_tasks.write().unwrap().push(task.clone());
        Ok(task)
    }

    /// 轮询触发: 对所有 TriggerMode::Polling 的窗口扫描
    pub async fn poll_upload_tick(&self) -> Vec<UploadTask> {
        let mut triggered = Vec::new();
        let windows = self.windows.read().unwrap().clone();
        for window in windows.values() {
            if window.upload_trigger != TriggerMode::Polling { continue; }
            for tab in &window.tabs {
                if !tab.files_changed.is_empty() {
                    let msg = format!("[polling] auto-upload tab {}", tab.label);
                    if let Ok(t) = self.trigger_upload(
                        window.id,
                        tab.id,
                        tab.files_changed.clone(),
                        msg,
                        TriggerMode::Polling,
                    ) {
                        triggered.push(t);
                    }
                }
            }
        }
        triggered
    }

    pub fn list_upload_tasks(&self) -> Vec<UploadTask> {
        self.upload_tasks.read().unwrap().clone()
    }
}

impl Default for WindowService {
    fn default() -> Self { Self::new() }
}

fn id_() -> Uuid { Uuid::nil() }

// =====================================================================
// 5. invariant
// =====================================================================

/// INV-WIN-01: window.tabs 最多 20 个
pub fn inv_01_max_tabs(window: &TaskWindow) -> bool {
    window.tabs.len() <= 20
}

/// INV-WIN-02: 同一 window 中 tab id 唯一
pub fn inv_02_unique_tab_ids(window: &TaskWindow) -> bool {
    let mut seen = std::collections::HashSet::new();
    for t in &window.tabs {
        if !seen.insert(t.id) { return false; }
    }
    true
}

/// INV-WIN-03: active_tab_id 必是 tabs 中之一
pub fn inv_03_active_tab_in_tabs(window: &TaskWindow) -> bool {
    match window.active_tab_id {
        None => window.tabs.is_empty(),
        Some(id) => window.tabs.iter().any(|t| t.id == id),
    }
}

/// INV-WIN-04: tab 完成时 finished_at 必填
pub fn inv_04_finished_at_set(tab: &TaskTab) -> bool {
    match tab.state {
        TabState::Completed | TabState::Failed | TabState::Aborted => tab.finished_at.is_some(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_mode_name() {
        assert!(TriggerMode::OnSuccessExit.name().contains("成功"));
        assert!(TriggerMode::Manual.name().contains("手动"));
        assert!(TriggerMode::Polling.name().contains("轮询"));
    }

    #[test]
    fn test_window_new() {
        let w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::OnSuccessExit);
        assert_eq!(w.name, "Test");
        assert_eq!(w.upload_trigger, TriggerMode::OnSuccessExit);
        assert_eq!(w.polling_interval_sec, 300);
    }

    #[test]
    fn test_window_add_close_tab() {
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        let tab = w.add_tab(Uuid::new_v4(), "tab 1").unwrap();
        let id = tab.id;
        assert_eq!(w.tabs.len(), 1);
        assert_eq!(w.active_tab_id, Some(id));
        w.close_tab(id).unwrap();
        assert_eq!(w.tabs.len(), 0);
        assert_eq!(w.active_tab_id, None);
    }

    #[test]
    fn test_window_too_many_tabs() {
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        for i in 0..20 {
            w.add_tab(Uuid::new_v4(), format!("tab {}", i)).unwrap();
        }
        let r = w.add_tab(Uuid::new_v4(), "tab 21");
        assert!(matches!(r, Err(WindowError::TooManyTabs(_))));
    }

    #[test]
    fn test_window_update_tab_state() {
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        let tab = w.add_tab(Uuid::new_v4(), "tab").unwrap();
        w.update_tab_state(tab.id, TabState::Completed, Some(0)).unwrap();
        assert_eq!(w.tabs[0].state, TabState::Completed);
        assert_eq!(w.tabs[0].exit_code, Some(0));
        assert!(w.tabs[0].finished_at.is_some());
    }

    #[test]
    fn test_service_open_close_window() {
        let svc = WindowService::new();
        let w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        let id = w.id;
        svc.open_window(w).unwrap();
        assert!(svc.get_window(id).is_some());
        svc.close_window(id).unwrap();
        assert!(svc.get_window(id).is_none());
    }

    #[test]
    fn test_service_list_by_worktree() {
        let svc = WindowService::new();
        let wt_id = Uuid::new_v4();
        for i in 0..3 {
            let w = TaskWindow::new(&format!("W{}", i), wt_id, Uuid::new_v4(), TriggerMode::Manual);
            svc.open_window(w).unwrap();
        }
        // 另一个 worktree
        let w2 = TaskWindow::new("Other", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        svc.open_window(w2).unwrap();
        let list = svc.list_windows_by_worktree(wt_id);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_trigger_upload_mismatch() {
        let svc = WindowService::new();
        let w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::OnSuccessExit);
        let wid = w.id;
        let tid = Uuid::new_v4();
        svc.open_window(w).unwrap();
        let r = svc.trigger_upload(wid, tid, vec![], "msg", TriggerMode::Manual);
        assert!(matches!(r, Err(WindowError::TriggerMismatch(_, _))));
    }

    #[test]
    fn test_trigger_upload_match() {
        let svc = WindowService::new();
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::OnSuccessExit);
        let tab = w.add_tab(Uuid::new_v4(), "tab").unwrap();
        let wid = w.id;
        let tid = tab.id;
        svc.open_window(w).unwrap();
        let task = svc.trigger_upload(wid, tid, vec!["a.rs".into()], "msg", TriggerMode::OnSuccessExit).unwrap();
        assert_eq!(task.trigger, TriggerMode::OnSuccessExit);
        assert_eq!(task.files_changed, vec!["a.rs".to_string()]);
    }

    #[tokio::test]
    async fn test_poll_upload_tick() {
        let svc = WindowService::new();
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Polling);
        let mut tab = w.add_tab(Uuid::new_v4(), "tab").unwrap();
        tab.files_changed.push("x.rs".into());
        let triggered = svc.poll_upload_tick().await;
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].trigger, TriggerMode::Polling);
    }

    #[test]
    fn test_inv_01_max_tabs() {
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        assert!(inv_01_max_tabs(&w));
        for _ in 0..21 {
            w.add_tab(Uuid::new_v4(), "t").ok();
        }
        assert!(!inv_01_max_tabs(&w));
    }

    #[test]
    fn test_inv_04_finished_at() {
        let mut w = TaskWindow::new("Test", Uuid::new_v4(), Uuid::new_v4(), TriggerMode::Manual);
        let tab = w.add_tab(Uuid::new_v4(), "t").unwrap();
        assert!(inv_04_finished_at_set(&w.tabs[0]));
        w.update_tab_state(tab.id, TabState::Completed, Some(0)).unwrap();
        assert!(inv_04_finished_at_set(&w.tabs[0]));
    }
}
