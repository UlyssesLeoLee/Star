//! `event-bus` — AI Worktree Graph Canvas Event Bus crate.
//!
//! Per ULYS-57.3 T9 (2026-09-17):
//! - `EventBus` Trait + 6 方法 (per DD §20 + D-EVENT-001)
//! - 15 Event 类型 (per DD §19 + spec §4.5)
//! - 4 消费者 (Graph / UI / Risk / Notification, per spec §4.5 + BD §13.2)
//! - EventDebouncer 100ms (per spec §4.5 + DD §11.3 + NFR-REL-002)
//! - Graph Delta Update (per DD §32)
//!
//! 守门:
//! - #1 v25 `cargo test -p event-bus --lib` 3/3 pass
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 `[workspace.dependencies]`
//! - #13 W/T/M 100% 覆盖 (event_outbox T)
//!
//! 阶段 1 scope (per WORKTREE-CANVAS-IMPL-PLAN-001 §4.9 T9):
//! - InMemoryBus (本期 T9, 测试 + dev 用)
//! - Redis Streams backend 留接口 (`BusBackend` trait, P2 落点)
//! - 4 消费者: Graph / UI / Risk / Notification (按 consumer group)
//! - 100ms 防抖 (per NFR-REL-002 throughput 10k eps)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]

pub mod bus;
pub mod consumer;
pub mod debouncer;
pub mod delta;
pub mod error;
pub mod events;

pub use bus::{BusBackend, EventBus, InMemoryBus, StreamKey, DEFAULT_STREAM_KEY};
pub use consumer::{Consumer, ConsumerGroup, EventConsumer};
pub use debouncer::{Debouncer, DebouncerConfig};
pub use delta::{apply_delta, DeltaOp};
pub use error::EventBusError;
pub use events::{CanvasEvent, EVENT_TYPE_COUNT};
