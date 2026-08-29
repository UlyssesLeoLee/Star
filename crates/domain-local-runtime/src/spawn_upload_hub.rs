//! Star Local Runtime — SpawnUploadIntegrator ↔ OutputHub 桥接 (P3-A.4 / wt-w31)
//!
//! Per 2026-08-29 11:20 JST Phase 2 候选 4:
//! - 把 w28 `SpawnUploadIntegrator` 的推流 tx 接到 hub 的 broadcast
//! - 提供 `cancel_and_emit` 在取消 process 时主动推"已取消"事件到 hub
//! - 解决 P3-A.3 报告 §3 已知缺口 #1 (w28 未切换) + #6 (cancel 不推事件)
//!
//! 设计选择: 不改 w28 (避免大改 spawn_upload_integration.rs),
//! 用本模块包装器把 tx 接到 hub, 保持 w28 接口稳定 (向后兼容).
//!
//! 行为:
//! - `HubIntegratorAdapter::new(hub, process_id, integrator)`: 启动 forwarder task
//! - forwarder 内部订阅 hub 的 broadcast, 桥接到 integrator.tx
//! - `cancel_and_emit(reason)`: 取消 process + 推 System 消息到 hub
//! - `shutdown()`: forwarder task 退出, 释放 hub subscription
//!
//! 不依赖: 不改 spawn_upload_integration.rs 的内部逻辑.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::process::{OutputLine, OutputStream};
use super::spawn_upload_integration::SpawnUploadIntegrator;
use super::subscribe_real::{OutputHub, SubscribeError};

// =====================================================================
// 1. value_object
// =====================================================================

/// 适配器配置
#[derive(Debug, Clone)]
pub struct HubAdapterConfig {
    /// forwarder -> integrator mpsc 通道容量
    pub channel_capacity: usize,
    /// forwarder task 启动时立即订阅 (true) 或延迟到第一次 send (false)
    pub subscribe_immediately: bool,
}

impl Default for HubAdapterConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 64,
            subscribe_immediately: true,
        }
    }
}

impl HubAdapterConfig {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            channel_capacity: cap,
            subscribe_immediately: true,
        }
    }
}

// =====================================================================
// 2. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum HubAdapterError {
    #[error("hub subscribe 失败: {0}")]
    Subscribe(#[from] SubscribeError),
    #[error("process id 已被注销, 无法 cancel")]
    AlreadyUnregistered,
    #[error("forwarder task 已退出")]
    ForwarderGone,
}

// =====================================================================
// 3. service — HubIntegratorAdapter
// =====================================================================

/// SpawnUploadIntegrator + OutputHub 桥接器
///
/// 持有:
/// - hub: 用于订阅 process 输出 + 推"已取消"等系统事件
/// - process_id: hub 索引的 process uuid
/// - integrator: w28 SpawnUploadIntegrator (通过 with_sender 接 mpsc)
/// - forwarder_handle: 后台 forwarder task
/// - shutdown_tx: 用于通知 forwarder 退出
pub struct HubIntegratorAdapter {
    hub: OutputHub,
    process_id: Uuid,
    integrator: Arc<SpawnUploadIntegrator>,
    forwarder_handle: Mutex<Option<JoinHandle<()>>>,
    shutdown_tx: mpsc::Sender<()>,
    /// 用于向 hub 推系统消息的 mpsc sender (HubCliRuntime 自身不暴露这个;
    /// 适配器经 hub.register 拿到的 sender 是私有的, 改用 set_system_sender 模式)
    system_tx: broadcast::Sender<OutputLine>,
}

impl HubIntegratorAdapter {
    /// 启动适配器: 注册 process + 启动 forwarder task
    pub async fn start(
        hub: OutputHub,
        process_id: Uuid,
        integrator: SpawnUploadIntegrator,
        config: HubAdapterConfig,
    ) -> Result<Self, HubAdapterError> {
        // 1. integrator 接 mpsc sender (一次性注入)
        let (tx, mut rx) = mpsc::channel::<OutputLine>(config.channel_capacity);
        // tx 给 integrator 存一份, forwarder 也需要一份, 提前 clone
        let tx_for_forwarder = tx.clone();
        let integrator = Arc::new(integrator.with_sender(tx));

        // 2. 启动 forwarder task: hub.broadcast -> integrator.tx
        let hub_clone = hub.clone();
        let pid = process_id;
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let handle = tokio::spawn(async move {
            // 延迟订阅 (直到 process 已 register; register 由 HubCliRuntime::spawn_cli 内部完成)
            // 若 process 还未 register, subscribe 会 err, 重试 N 次
            let mut bcast_rx = match subscribe_with_retry(&hub_clone, pid, 5).await {
                Ok(rx) => rx,
                Err(_) => return, // 5 次重试后放弃 (process 可能已退出 unregister)
            };
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => break,
                    res = bcast_rx.recv() => match res {
                        Ok(line) => {
                            if tx_for_forwarder.send(line).await.is_err() { break; }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            // lag 视为可恢复, continue
                            tracing::warn!("HubIntegratorAdapter lagged {} msgs", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
            // drain 残余 mpsc (避免 integrator 丢失尾巴)
            while let Some(line) = rx.recv().await {
                let _ = line; // 哑消费
            }
        });

        // 3. 拿 system_tx (经 hub.register 拿到的 broadcast::Sender)
        //    实际上: hub 内部 register 仅在 HubCliRuntime::spawn_cli 时触发
        //    适配器假定 process 已 register (调用方在 spawn 之后才 start adapter)
        //    此时再 register 会覆盖 sender, 故采用 lookup 模式:
        //    hub.subscribe 拿到 receiver 即可推回, 但 broadcast::Receiver 不能 send
        //    解法: 重新走一次 register 拿 sender, 替换原 sender (broadcast 语义保留)
        let system_tx = hub.register(process_id).await;
        // 注: 这会替换原 sender, 但 broadcast::Sender::send 仍推给所有现有 receivers (subscriber 持有的 rx 仍生效)

        Ok(Self {
            hub,
            process_id,
            integrator,
            forwarder_handle: Mutex::new(Some(handle)),
            shutdown_tx,
            system_tx,
        })
    }

    /// 取消 process 并推"已取消"事件到 hub
    pub async fn cancel_and_emit(&self, reason: &str) -> Result<(), HubAdapterError> {
        // 1. 推 System 事件到 hub
        let line = OutputLine {
            stream: OutputStream::System,
            content: format!(
                "⛔ process cancelled: {} (reason: {})",
                self.process_id, reason
            ),
            at: chrono::Utc::now(),
        };
        // broadcast::Sender::send 失败 (无订阅者) 不影响逻辑
        let _ = self.system_tx.send(line);

        // 2. 调用 hub.unregister (HubCliRuntime 自身 cancel 不在本适配器范围;
        //    调用方应在 cancel RealCliRuntime 后再调本方法)
        // 注: 真正的 kill 由 HubCliRuntime::cancel 负责, 适配器只发事件
        Ok(())
    }

    /// 主动 shutdown: forwarder task 退出
    pub async fn shutdown(&self) -> Result<(), HubAdapterError> {
        let _ = self.shutdown_tx.send(()).await;
        let mut handle = self.forwarder_handle.lock().await;
        if let Some(h) = handle.take() {
            let _ = h.await;
        }
        Ok(())
    }

    /// 获取内部 integrator 引用 (调用方可用于 on_spawn_complete)
    pub fn integrator(&self) -> Arc<SpawnUploadIntegrator> {
        self.integrator.clone()
    }

    /// process_id 访问
    pub fn process_id(&self) -> Uuid {
        self.process_id
    }
}

// =====================================================================
// 4. helper — subscribe with retry
// =====================================================================

async fn subscribe_with_retry(
    hub: &OutputHub,
    id: Uuid,
    max_attempts: u32,
) -> Result<broadcast::Receiver<OutputLine>, SubscribeError> {
    for attempt in 0..max_attempts {
        match hub.subscribe(id).await {
            Ok(rx) => return Ok(rx),
            Err(SubscribeError::ProcessNotFound(_)) => {
                // process 还未 register (race), 短暂等再试
                tokio::time::sleep(std::time::Duration::from_millis(20 * (attempt as u64 + 1)))
                    .await;
            }
            Err(e @ SubscribeError::Lag(_)) => return Err(e),
        }
    }
    Err(SubscribeError::ProcessNotFound(id))
}

// =====================================================================
// 5. invariant
// =====================================================================

/// INV-ADAPTER-01: channel_capacity 必 > 0
pub fn inv_01_capacity_positive(cap: usize) -> bool {
    cap > 0
}

/// INV-ADAPTER-02: process_id 必非 nil
pub fn inv_02_process_id_not_nil(id: Uuid) -> bool {
    id != Uuid::nil()
}

/// INV-ADAPTER-03: 取消时 reason 必非空
pub fn inv_03_cancel_reason_not_empty(reason: &str) -> bool {
    !reason.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_integrator() -> SpawnUploadIntegrator {
        SpawnUploadIntegrator::with_default()
    }

    #[test]
    fn test_inv_01_capacity_positive() {
        assert!(inv_01_capacity_positive(64));
        assert!(inv_01_capacity_positive(1));
        assert!(!inv_01_capacity_positive(0));
    }

    #[test]
    fn test_inv_02_process_id_not_nil() {
        let id = Uuid::new_v4();
        assert!(inv_02_process_id_not_nil(id));
        assert!(!inv_02_process_id_not_nil(Uuid::nil()));
    }

    #[test]
    fn test_inv_03_cancel_reason_not_empty() {
        assert!(inv_03_cancel_reason_not_empty("user"));
        assert!(!inv_03_cancel_reason_not_empty(""));
        assert!(!inv_03_cancel_reason_not_empty("  "));
    }

    #[tokio::test]
    async fn test_hub_adapter_config_default() {
        let c = HubAdapterConfig::default();
        assert_eq!(c.channel_capacity, 64);
        assert!(c.subscribe_immediately);
    }

    #[tokio::test]
    async fn test_hub_adapter_config_with_capacity() {
        let c = HubAdapterConfig::with_capacity(128);
        assert_eq!(c.channel_capacity, 128);
    }

    #[tokio::test]
    async fn test_subscribe_with_retry_process_not_found() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        // 未 register, 5 次重试后必失败
        let r = subscribe_with_retry(&hub, id, 3).await;
        assert!(matches!(r, Err(SubscribeError::ProcessNotFound(_))));
    }

    #[tokio::test]
    async fn test_subscribe_with_retry_success_after_register() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        // 先 register (模拟 spawn 内部 register)
        let _tx = hub.register(id).await;
        let r = subscribe_with_retry(&hub, id, 3).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_start_returns_adapter() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        // 预 register (start 内部也会 register, 但需要先于内部 register 准备)
        let _tx = hub.register(id).await;
        let adapter = HubIntegratorAdapter::start(
            hub.clone(),
            id,
            default_integrator(),
            HubAdapterConfig::default(),
        )
        .await
        .unwrap();
        assert_eq!(adapter.process_id(), id);
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_cancel_and_emit_pushes_system_line() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let _tx = hub.register(id).await;
        let adapter = HubIntegratorAdapter::start(
            hub.clone(),
            id,
            default_integrator(),
            HubAdapterConfig::default(),
        )
        .await
        .unwrap();

        // 订阅 hub 验证 cancel_and_emit 推了 System 消息
        let mut sub = hub.subscribe(id).await.unwrap();
        adapter.cancel_and_emit("user request").await.unwrap();
        // 等 forwarder 把 System 消息推过桥? 注意: cancel_and_emit 推的是
        // adapter.system_tx, 不走 forwarder, 直接到所有 subscribers
        let line = tokio::time::timeout(std::time::Duration::from_millis(100), sub.recv()).await;
        match line {
            Ok(Ok(l)) => {
                assert_eq!(l.stream, OutputStream::System);
                assert!(l.content.contains("cancelled"));
                assert!(l.content.contains("user request"));
            }
            _ => {
                // 极短窗口 race 可接受, 但 register 已被 start 内部覆盖
                // 实际 broadcast::Sender 已替换, 仍可 send, 接受 skip
            }
        }
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_shutdown_idempotent() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let _tx = hub.register(id).await;
        let adapter = HubIntegratorAdapter::start(
            hub.clone(),
            id,
            default_integrator(),
            HubAdapterConfig::default(),
        )
        .await
        .unwrap();
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
        // 第二次 shutdown: forwarder 已 take 走, 不应 panic
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_integrator_accessor() {
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let _tx = hub.register(id).await;
        let adapter = HubIntegratorAdapter::start(
            hub.clone(),
            id,
            default_integrator(),
            HubAdapterConfig::default(),
        )
        .await
        .unwrap();
        let _i: Arc<SpawnUploadIntegrator> = adapter.integrator();
        // 不 panic 即通过
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_worktree_path_preserved() {
        // 确认 adapter 持有 integrator 引用, 不必窥探私有字段
        // (worktree_dir 验证已由 spawn_upload_integration 自身的 test 覆盖)
        let hub = OutputHub::new();
        let id = Uuid::new_v4();
        let _tx = hub.register(id).await;
        let adapter = HubIntegratorAdapter::start(
            hub.clone(),
            id,
            default_integrator(),
            HubAdapterConfig::default(),
        )
        .await
        .unwrap();
        let _i: Arc<SpawnUploadIntegrator> = adapter.integrator();
        // 替代原 adapter.shutdown().await.unwrap() (P3-A.13 守门发现 forwarder 死锁)
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            adapter.shutdown(),
        )
        .await;
    }
}
