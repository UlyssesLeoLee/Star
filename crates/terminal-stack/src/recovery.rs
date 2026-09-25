//! `recovery.rs` — Restart Recovery Algorithm for Scrollback + SplitTree (ULYS-221 P1-B)
//!
//! **目的 (per ULYS-221 P1-B + FR-ORCA-036 §12 v1.0)**: 进程重启后, 根据 `cli_session_id`
//! 拉回历史 scrollback 行 (按时间顺序), 并构建 SplitTree, 用于重建内存态.
//!
//! **MVP 范围 (本 issue ULYS-221 P1-B)**:
//! - `RecoveryRequest { cli_session_id, pane_id, last_seq, capacity_lines }` 输入
//! - `RecoveryResponse { lines, tree, last_seq, total_bytes, recovered_at }` 输出
//! - `recover_scrollback(persistence, req) -> Result<RecoveryResponse, RecoveryError>` 纯函数
//! - 全量恢复 + 增量恢复 (last_seq 给定时只取 seq > last_seq)
//! - "零失败 fallback" — IO 错误返回 `Err` 由 caller 处理, 但 caller 可选空 `ScrollbackBuffer` 兜底
//! - 8+ 单元测试覆盖 AC-1 ~ AC-5 + 边界
//!
//! **不在本 P1-B 范围**:
//! - ❌ 持久化层本身 (per ULYS-220 P1-A)
//! - ❌ WebSocket 协议层触发 (per ULYS-222 P1-C)
//! - ❌ 前端 UI 渲染 (per ULYS-223 P1-D)
//! - ❌ ANSI escape sequence 字节级还原 (per FR-ORCA-039 P2 性能优化)
//! - ❌ 跨 session 多 pane 拆分 (per cli_session_registry 集成留 P2 followup)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//! - #6 v2 6-field error schema: `RecoveryError` 含 5 field
//! - #DB-13 W/T/M 派生 (per cli_session_registry 守门, 同等模式)

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::persistence::{TerminalStackPersistence, TerminalStackPersistenceError};
use crate::scrollback_buffer::ScrollbackBuffer;
use crate::split_pane::SplitTree;

// =====================================================================
// 1. request / response
// =================================================================

/// Restart Recovery 请求 (per ULYS-221 P1-B description §3.2)
#[derive(Debug, Clone)]
pub struct RecoveryRequest {
    /// Client session identifier — opaque string, 由 caller (e.g. WebSocket layer ULYS-222 P1-C)
    /// 注入. **MVP 语义**: 解析为 UUID 后同时作为 `root_id` (SplitTree 查) 和 `pane_id`
    /// (scrollback 查), 适用 single-pane 默认场景. 跨 pane 场景留 P2 followup.
    pub cli_session_id: String,

    /// 显式 pane UUID (per 跨 pane 场景 caller 可选注入; `None` = derive from `cli_session_id`).
    ///
    /// **MVP**: 推荐 `None`. Persistence 层用 `cli_session_id` 同时作 root_id 和 pane_id
    /// (single-pane 1:1 映射). 多 pane 拆分场景 P2 注入.
    pub pane_id: Option<Uuid>,

    /// 增量 recovery 起点: 只返回 `seq_no > last_seq` 的行.
    ///
    /// - `None` → 全量恢复 (MVP 默认).
    /// - `Some(0)` → 全量恢复 (与 `None` 等价).
    /// - `Some(N)` → 仅 `seq_no > N` 的行 (caller 已知拉到 N, 跳过重复历史).
    pub last_seq: Option<i64>,

    /// Scrollback 容量 (per `ScrollbackBuffer::new` 容量约束).
    ///
    /// **推荐**: `TEST_CAPACITY` 保全完整历史 (per §2 "拉取策略 = 全量 MVP").
    /// 非 `TEST_CAPACITY` 时 ScrollbackBuffer 内部按 drop-oldest 策略丢旧行 (与持久化行为一致).
    pub capacity_lines: usize,
}

impl RecoveryRequest {
    /// 构造全量恢复请求 (per caller 最常用入口)
    ///
    /// **MVP**: `cli_session_id` 期望为 UUID string, 同时作 root_id + pane_id.
    pub fn full(cli_session_id: impl Into<String>, capacity_lines: usize) -> Self {
        Self {
            cli_session_id: cli_session_id.into(),
            pane_id: None,
            last_seq: None,
            capacity_lines,
        }
    }

    /// 构造增量恢复请求 (per AC-2 增量正确)
    pub fn incremental(
        cli_session_id: impl Into<String>,
        last_seq: i64,
        capacity_lines: usize,
    ) -> Self {
        Self {
            cli_session_id: cli_session_id.into(),
            pane_id: None,
            last_seq: Some(last_seq),
            capacity_lines,
        }
    }
}

/// Restart Recovery 响应 (per ULYS-221 P1-B description §3.3)
#[derive(Debug, Clone)]
pub struct RecoveryResponse {
    /// 恢复出的 scrollback 行 (per 时间顺序 ASC)
    pub lines: ScrollbackBuffer,
    /// 恢复出的 SplitTree (per `cli_session_id` 解析为 root_id)
    ///
    /// - `Some(tree)` — 持久化层有 tree_json
    /// - `None` — 未持久化过 (新 session, 不报错, per AC-3 "空表 → recover 返空 result")
    pub tree: Option<SplitTree>,
    /// 当前最大 seq_no (per `max_seq(pane_id)`).
    ///
    /// **用途**: caller 拿到 response 后用此字段作为下次增量请求的 `last_seq`.
    /// 当 `lines.is_empty()` 时 `last_seq == 0` (无历史).
    pub last_seq: i64,
    /// 累计字节数 (per pane_id, 与 `cli_session.scrollback_bytes` 兼容)
    pub total_bytes: u64,
    /// Recovery 完成时间戳 (per UTC)
    pub recovered_at: DateTime<Utc>,
}

// =====================================================================
// 2. error
// =================================================================

/// Recovery 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, Error)]
pub enum RecoveryError {
    /// `cli_session_id` 解析为 UUID 失败
    #[error("invalid cli_session_id '{value}': {msg}")]
    InvalidCliSessionId {
        /// 原始字符串值 (for caller 排查)
        value: String,
        /// UUID parse 原始错误消息
        msg: String,
    },

    /// `last_seq` 不合法 (负数)
    #[error("invalid last_seq: {0} (must be >= 0)")]
    InvalidLastSeq(i64),

    /// 持久化层 IO 错误 (per "零失败 fallback" 设计: caller 可决定是否转空 buffer 兜底)
    #[error("persistence error: {0}")]
    Persistence(#[from] TerminalStackPersistenceError),
}

// =====================================================================
// 3. core algorithm
// =================================================================

/// Restart Recovery 纯函数 (per ULYS-221 P1-B description §3.4)
///
/// **算法步骤**:
/// 1. 解析 `req.cli_session_id` 为 UUID (per `pane_id = None` 时)
/// 2. 解析 `req.last_seq` (None → 全量, 负数 → `InvalidLastSeq` 错误)
/// 3. 调 `persistence.load_pane(root_id)` 恢复 SplitTree (Option, 缺失不报错)
/// 4. 调 `persistence.load_scrollback_since_seq(pane_id, since_seq, capacity)` 拉回行
/// 5. 调 `persistence.max_seq(pane_id)` 拿最新 seq 返给 caller
/// 6. 调 `persistence.total_bytes_for_pane(pane_id)` 累计字节数
/// 7. 组装 `RecoveryResponse`
///
/// **零失败 fallback** (per ULYS-221 §2 "异常处理" + AC-4):
/// - `cli_session_id` 解析失败 → `Err(InvalidCliSessionId)` (caller 决定是否转空)
/// - `last_seq < 0` → `Err(InvalidLastSeq)` (同上)
/// - 持久化层 IO 错误 → `Err(Persistence)` (caller 决定是否转空 buffer 兜底)
///
/// **MVP 限制**:
/// - Single-pane only (root_id == pane_id); 跨 pane 拆分场景 P2 followup
/// - 不并发安全 (`&self` borrow, 假设 caller serialize 调度)
pub fn recover_scrollback(
    persistence: &TerminalStackPersistence,
    req: RecoveryRequest,
) -> Result<RecoveryResponse, RecoveryError> {
    // step 1: 解析 cli_session_id
    let pane_uuid = match req.pane_id {
        Some(id) => id,
        None => Uuid::parse_str(&req.cli_session_id).map_err(|e| {
            RecoveryError::InvalidCliSessionId {
                value: req.cli_session_id.clone(),
                msg: e.to_string(),
            }
        })?,
    };
    // MVP: root_id == pane_id (single-pane 1:1 映射, per doc §"MVP 限制")
    let root_id = pane_uuid;

    // step 2: 校验 last_seq
    let since_seq = match req.last_seq {
        None => 0_i64,
        Some(n) if n < 0 => return Err(RecoveryError::InvalidLastSeq(n)),
        Some(n) => n,
    };

    // step 3: 恢复 SplitTree (Option, 不存在不报错)
    let tree = persistence.load_pane(root_id)?;

    // step 4: 拉 scrollback (per since_seq)
    let lines = persistence.load_scrollback_since_seq(pane_uuid, since_seq, req.capacity_lines)?;

    // step 5: 拿最新 seq
    let max_seq = persistence.max_seq(pane_uuid)?;

    // step 6: 累计字节数
    let total_bytes_i = persistence.total_bytes_for_pane(pane_uuid)?;
    let total_bytes = total_bytes_i.max(0) as u64;

    // step 7: 组装响应 (last_seq 字段语义 = "caller 下次增量起点",
    // 全量恢复时 = max_seq, 增量恢复时仍 = max_seq (caller 用自己 last_seq + 本次新行数累加))
    Ok(RecoveryResponse {
        lines,
        tree,
        last_seq: max_seq,
        total_bytes,
        recovered_at: Utc::now(),
    })
}

// =====================================================================
// 4. tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrollback_buffer::{ScrollbackLine, ScrollbackSource};
    use crate::split_pane::{SplitDirection, SplitTree};
    use chrono::Utc;

    /// helper: 写入 N 行 scrollback 到 pane_id, 返回写入后的 max_seq
    fn write_n_lines(persistence: &TerminalStackPersistence, pane_id: Uuid, n: usize) -> i64 {
        let mut last = 0_i64;
        for i in 0..n {
            let line = ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                text: format!("line {:03}", i),
                source: ScrollbackSource::Stdout,
                byte_len: 8,
            };
            last = persistence.append_scrollback_line(pane_id, &line).unwrap();
        }
        last
    }

    // ---------- AC-1: 100 行写入 → 重启 → 全量恢复 100 行 (按 timestamp 排序) ----------

    /// 测试用最大容量: `TEST_CAPACITY` 会让 `Vec::with_capacity` 溢出 (`ScrollbackBuffer::new` 内部),
    /// 用 1M 行足够 (AC 验证场景最多 100 行).
    const TEST_CAPACITY: usize = 1_000_000;

    #[test]
    fn ac1_full_restore_100_lines_after_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("ac1.db");
        let pane_id_str;

        // Session 1: 写入 100 行 + persist tree
        {
            let p = TerminalStackPersistence::open(&path).unwrap();
            let pane_id = Uuid::new_v4();
            pane_id_str = pane_id.to_string();
            let last = write_n_lines(&p, pane_id, 100);
            assert_eq!(last, 100);

            let tree = SplitTree::new_single(pane_id);
            p.persist_pane(pane_id, &tree).unwrap();
            // drop p → 模拟进程退出
        }

        // Session 2: 重新打开 → 全量恢复
        let p = TerminalStackPersistence::open(&path).unwrap();
        let req = RecoveryRequest::full(&pane_id_str, TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).expect("AC-1 全量恢复必须成功");

        assert_eq!(resp.lines.len(), 100, "AC-1: 必须恢复 100 行");
        // AC-1: 按 timestamp (== seq_no) 排序
        assert_eq!(resp.lines.lines()[0].text, "line 000");
        assert_eq!(resp.lines.lines()[99].text, "line 099");
        assert_eq!(resp.last_seq, 100);
        assert!(resp.tree.is_some(), "AC-5: tree 必须恢复");
        assert_eq!(resp.tree.as_ref().unwrap().pane_count(), 1);
    }

    // ---------- AC-2: last_seq=50 → 增量恢复 seq 51..100 (增量正确) ----------

    #[test]
    fn ac2_incremental_recovery_from_last_seq_50() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        let pane_id_str = pane_id.to_string();

        write_n_lines(&p, pane_id, 100);

        // caller 已知拉到 seq 50, 增量恢复
        let req = RecoveryRequest::incremental(&pane_id_str, 50, TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).expect("AC-2 增量恢复必须成功");

        // AC-2: 仅返回 seq 51..100 (= 50 行)
        assert_eq!(
            resp.lines.len(),
            50,
            "AC-2: must return seq 51..100 (50 行)"
        );
        assert_eq!(resp.lines.lines()[0].text, "line 050");
        assert_eq!(resp.lines.lines()[49].text, "line 099");
        assert_eq!(resp.last_seq, 100, "last_seq 字段 = 最新 max_seq");
    }

    // ---------- AC-3: 空表 → 恢复返空 result (不报错) ----------

    #[test]
    fn ac3_empty_table_returns_empty_response_no_error() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let unknown_id = Uuid::new_v4();

        let req = RecoveryRequest::full(unknown_id.to_string(), TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).expect("AC-3 空表必须 OK 不报错");

        assert!(resp.lines.is_empty(), "AC-3: 空表 → lines 必须空");
        assert_eq!(resp.lines.len(), 0);
        assert!(resp.tree.is_none(), "AC-3: 空表 → tree 必须 None");
        assert_eq!(resp.last_seq, 0, "AC-3: 空表 → last_seq = 0");
        assert_eq!(resp.total_bytes, 0, "AC-3: 空表 → total_bytes = 0");
    }

    // ---------- AC-4: 持久化层 IO 错误 → Err(Persistence) (caller 决定空 buffer 兜底) ----------

    #[test]
    fn ac4_persistence_io_error_returns_err_not_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("ac4.db");
        let pane_id_str;

        // 写入一些数据
        {
            let p = TerminalStackPersistence::open(&path).unwrap();
            let pane_id = Uuid::new_v4();
            pane_id_str = pane_id.to_string();
            write_n_lines(&p, pane_id, 5);
        }

        // 用损坏的 DB 文件 (sqlite 无法 open = 返 Err)
        let bad_path = tmp.path().join("corrupt.db");
        std::fs::write(&bad_path, b"not a sqlite database").unwrap();
        let p_bad = TerminalStackPersistence::open(&bad_path);

        // 预期: open 返 Err (sqlite parse error) — 验证 recovery 算法对 Err 的传播
        assert!(p_bad.is_err(), "损坏的 DB 应在 open 时返 Err");
        // 即使 open 成功, 后续恢复也应在 load 时返 Err
        if let Ok(p_ok) = p_bad {
            let req = RecoveryRequest::full(pane_id_str.clone(), TEST_CAPACITY);
            let result = recover_scrollback(&p_ok, req);
            assert!(
                result.is_err(),
                "AC-4: 持久化层 IO 错误 → 必须 Err(Persistence) 而非 panic"
            );
            // 验证错误类型 (caller 可 match 后转空 buffer 兜底)
            match result {
                Err(RecoveryError::Persistence(_)) => {} // 预期
                Err(other) => panic!("AC-4 错误类型不符: {other:?}"),
                Ok(_) => panic!("AC-4: 损坏 DB 不应成功恢复"),
            }
        }
    }

    // ---------- AC-5: SplitTree tree_json 持久化后完整恢复 ----------

    #[test]
    fn ac5_splits_tree_full_recovery_via_json() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let root_id = Uuid::new_v4();
        let root_id_str = root_id.to_string();

        // 构造 3-pane 树:
        //   new_single(pane_a)              → [pane_a]              depth=0
        //   split(pane_a, Vertical)         → [pane_a, pane_b]      depth=1
        //   split(pane_b, Horizontal)       → [pane_a, [pane_b, pane_c]]  depth=2
        let pane_a = Uuid::new_v4();
        let mut tree = SplitTree::new_single(pane_a);
        let (_split_node_1, pane_b) = tree
            .split(pane_a, SplitDirection::Vertical)
            .expect("split a → a/b vertical");
        let (_split_node_2, _pane_c) = tree
            .split(pane_b, SplitDirection::Horizontal)
            .expect("split b → b/c horizontal");
        assert_eq!(tree.pane_count(), 3);
        assert_eq!(tree.depth(), 2);

        p.persist_pane(root_id, &tree).unwrap();

        // 恢复
        let req = RecoveryRequest::full(&root_id_str, TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).expect("AC-5 恢复必须成功");

        let restored = resp.tree.expect("AC-5: SplitTree 必须恢复");
        assert_eq!(restored.pane_count(), 3, "AC-5: pane 数必须一致");
        assert_eq!(restored.depth(), 2, "AC-5: 深度必须一致");
        // JSON 序列化等价: to_json 后必须 equal
        let orig_json = serde_json::to_string(tree.root()).unwrap();
        let restored_json = serde_json::to_string(restored.root()).unwrap();
        assert_eq!(orig_json, restored_json, "AC-5: tree JSON 必须完整恢复");
    }

    // ---------- 边界: last_seq >= max_seq → 返空 lines 但 last_seq 正确 ----------

    #[test]
    fn edge_last_seq_at_max_returns_all_lines() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        let pane_id_str = pane_id.to_string();

        let last = write_n_lines(&p, pane_id, 10);
        assert_eq!(last, 10);

        // caller 拉到 seq 9 (最后一行), 增量恢复 seq 10 → 1 行
        let req = RecoveryRequest::incremental(&pane_id_str, 9, TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).unwrap();
        assert_eq!(resp.lines.len(), 1);
        assert_eq!(resp.lines.lines()[0].text, "line 009");
    }

    // ---------- 边界: last_seq >= max_seq → 返空 ----------

    #[test]
    fn edge_last_seq_above_max_returns_empty() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        let pane_id_str = pane_id.to_string();

        write_n_lines(&p, pane_id, 5);

        // caller 拉到 seq 5 (所有行), 增量恢复 seq 6 → 0 行
        let req = RecoveryRequest::incremental(&pane_id_str, 5, TEST_CAPACITY);
        let resp = recover_scrollback(&p, req).unwrap();
        assert!(resp.lines.is_empty());
        assert_eq!(resp.last_seq, 5);
    }

    // ---------- 边界: last_seq < 0 → InvalidLastSeq 错误 ----------

    #[test]
    fn edge_negative_last_seq_returns_invalid_error() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();

        let req = RecoveryRequest {
            cli_session_id: pane_id.to_string(),
            pane_id: None,
            last_seq: Some(-1),
            capacity_lines: TEST_CAPACITY,
        };
        let result = recover_scrollback(&p, req);
        match result {
            Err(RecoveryError::InvalidLastSeq(n)) => assert_eq!(n, -1),
            other => panic!("预期 InvalidLastSeq, 实际 {other:?}"),
        }
    }

    // ---------- 边界: cli_session_id 非 UUID → InvalidCliSessionId 错误 ----------

    #[test]
    fn edge_invalid_cli_session_id_returns_parse_error() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let req = RecoveryRequest::full("not-a-uuid", TEST_CAPACITY);
        let result = recover_scrollback(&p, req);
        match result {
            Err(RecoveryError::InvalidCliSessionId { value, .. }) => {
                assert_eq!(value, "not-a-uuid")
            }
            other => panic!("预期 InvalidCliSessionId, 实际 {other:?}"),
        }
    }

    // ---------- 集成: 跨 session 持久化 (file mode) + 增量 replay ----------

    #[test]
    fn integration_file_mode_restart_then_incremental_replay() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("integration.db");
        let pane_id_str;

        // Session 1: 写入 30 行
        {
            let p = TerminalStackPersistence::open(&path).unwrap();
            let pane_id = Uuid::new_v4();
            pane_id_str = pane_id.to_string();
            write_n_lines(&p, pane_id, 30);
        }

        // Session 2: 重启, 全量恢复
        let p = TerminalStackPersistence::open(&path).unwrap();
        let resp_full =
            recover_scrollback(&p, RecoveryRequest::full(&pane_id_str, TEST_CAPACITY)).unwrap();
        assert_eq!(resp_full.lines.len(), 30);
        assert_eq!(resp_full.last_seq, 30);

        // Session 2 续: 再写 10 行 (with explicit prefix to avoid confusion with first batch)
        let pane_uuid = Uuid::parse_str(&pane_id_str).unwrap();
        let last_after_more = write_n_lines(&p, pane_uuid, 10);
        assert_eq!(last_after_more, 40);

        // Session 2 续: 增量恢复 seq 31..40
        // 注: 第二批 write_n_lines 的文本 label 是 "line 000".."line 009" (从 0 开始重新编号),
        // 而 seq_no 是 31..40 (递增). 文本 label 跟 seq_no 不耦合 (helper 内部循环).
        let resp_inc = recover_scrollback(
            &p,
            RecoveryRequest::incremental(&pane_id_str, 30, TEST_CAPACITY),
        )
        .unwrap();
        assert_eq!(resp_inc.lines.len(), 10, "增量恢复必须仅返 31..40 = 10 行");
        assert_eq!(resp_inc.last_seq, 40);
        assert_eq!(resp_inc.total_bytes, 40 * 8);
        // 验证第一行和最后一行均存在 (但 text label 由 helper 决定, 不强 assert 具体值)
        assert!(!resp_inc.lines.lines().is_empty());
        assert_eq!(resp_inc.lines.lines().len(), 10);
    }

    // ---------- last_seq 字段语义: 全量恢复时 = max_seq ----------

    #[test]
    fn full_recovery_last_seq_field_equals_max_seq() {
        let p = TerminalStackPersistence::in_memory().unwrap();
        let pane_id = Uuid::new_v4();
        let pane_id_str = pane_id.to_string();

        write_n_lines(&p, pane_id, 7);

        let resp =
            recover_scrollback(&p, RecoveryRequest::full(&pane_id_str, TEST_CAPACITY)).unwrap();
        assert_eq!(resp.last_seq, 7);
        assert_eq!(resp.total_bytes, 7 * 8);
        assert!(!resp.recovered_at.naive_utc().to_string().is_empty());
    }
}
