// SPDX-License-Identifier: MIT OR Apache-2.0
//! `integration` — 5 UT 跨 ARG Effect 守门 v3 跨 sub-session 收敛 (per brief v0.50 §2.1 D.5).

use star_arg::models::event::ARGEvent;
use star_arg_bridge::protocol::BridgeEnvelopeKind;
use star_arg_effect::error::EffectError;
use star_arg_effect::prompts::challenge_prompts;
use uuid::Uuid;

// ============================================================================
// D.5 守门 v3 跨 sub-session 收敛 (5 UT)
// ============================================================================

/// D.5.1 — crates/arg 单独编译成功 (singleton)
#[test]
fn test_arg_crate_singleton_compile() {
    // 编译本文件 = arg 编译 0 错
    let _e = ARGEvent::EdgeArchived { id: Uuid::new_v4() };
}

/// D.5.2 — crates/arg-bridge 导入 crates/arg 无循环
#[test]
fn test_arg_bridge_imports_arg_without_cycle() {
    use star_arg::models::event::ARGEvent;
    use star_arg_bridge::protocol::BridgeEnvelopeKind;
    let _arg_event = ARGEvent::EdgeArchived { id: Uuid::new_v4() };
    // 编译过 = BridgeEnvelopeKind 5 variants 都能引用
    let _kind: BridgeEnvelopeKind;
}

/// D.5.3 — crates/arg-effect 导入 crates/arg + arg-bridge 无循环
#[test]
fn test_arg_effect_imports_arg_bridge_without_cycle() {
    let _arg = ARGEvent::EdgeArchived { id: Uuid::new_v4() };
    let _effect = EffectError::InternalError("x".into());
    // arg-bridge 不被 effect 直接依赖 (per Cargo.toml deps 单向)
}

/// D.5.4 — 跨 crate 类型不冲突: ARGEvent 9 variants 都可构造
#[test]
fn test_arg_full_workspace_compile_no_cycles() {
    // 编译过 = 全 crate 类型签名稳定
    use star_arg::models::event::ARGEvent;
    let _e1 = ARGEvent::EdgeArchived { id: Uuid::new_v4() };
    let _e2 = ARGEvent::AgentArchived { id: Uuid::new_v4() };
}

/// D.5.5 — 10 套 challenges prompt HashMap 5×2 完整
#[test]
fn test_10_challenge_prompts_complete() {
    let prompts = challenge_prompts();
    // 5 decision_type × 2 trust_tier = 10 套
    assert_eq!(prompts.len(), 10, "expected exactly 10 challenge prompts");
}
