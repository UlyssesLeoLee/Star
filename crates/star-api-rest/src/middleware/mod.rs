// SPDX-License-Identifier: MIT OR Apache-2.0
//! 中间件 stub (per spec §1.3-§1.4)
//!
//! 三个中间件都是 stub — 当前 no-op 仅让 Router 注册成功.
//! P2 阶段 worker 子代理实装真实逻辑 (派前必先 `automation/dispatcher.py brief(...)`).

pub mod audit;
pub mod auth;
pub mod rate_limit;
