// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Provider 抽象 + cache 层落点 (per spec/acceptance/08 R-007 v0.2 fix 2026-08-27)
//
// 目的: 缓解 GitHub/GitLab API 限速风险, 为 Version Control Provider
//       (per [spec/vcs/01-version-control-provider.md] +
//        [arch/05 §5 REST API 14 endpoints]) 提供:
//       1. 通用 cache trait (Cache<K, V>)
//       2. TTL 失效策略 (per metadata `ttlMs` + `cacheScope`, per
//          [spec/mcp/01-mcp-spec.md §1.1 ④可缓存 list 结果])
//       3. Provider 集成点 (GitHub / GitLab / Gitea)
//
// 状态: 空壳 + TODO 占位 (per GAP-R-007 已知缺口)
// 触发: Phase D 实施
// 代签: per 2026-08-27 07:16 JST 代签规则反转, author = Mavis 接手 agent
//       (per DEC-008), committer = Ulysses Leo Lee
//
// --- Phase D 待填实清单 (TODOs, 不完整) ---
//
// TODO(cache-trait): 定义 `pub trait Cache<K, V>` with
//   - `async fn get(&self, key: &K) -> Result<Option<V>, CacheError>`
//   - `async fn put(&self, key: &K, value: &V, ttl: Duration) -> Result<(), CacheError>`
//   - `async fn invalidate(&self, key: &K) -> Result<(), CacheError>`
//   - `fn scope(&self) -> CacheScope` (workspace / session / none)
//
// TODO(impl-inmem): 实现 `InMemoryCache` (per-crate 单例) 用 `tokio::sync::RwLock<HashMap>`,
//   满足 MVP 阶段单进程场景; 后续 Phase 2+ 可换 Redis / sled.
//
// TODO(provider-integration): 为 VersionControlProvider trait (per spec/vcs/01) 加
//   - `get_issue_cached` (wrap `get_issue`)
//   - `get_worktree_cached` (wrap `get_worktree`)
//   - `search_code_cached` (wrap `search_code`)
//   - 用 `ttlMs=30000` + `cacheScope=workspace` 默认 (per mcp/01 §1.1 ④)
//
// TODO(tests): 加 integration test
//   - 验证 30s TTL 失效
//   - 验证 workspace 切换时清空
//   - 验证 Provider 429 响应时降级到 stale cache
//
// --- 暂不实现 ---
// - Redis 后端 (Phase 2+)
// - 跨 crate 共享 cache (Phase 2+)
// - HMAC 链 audit (per arch/06 §1.2 T-10, 属 R-010 不属 R-007)

#![allow(dead_code)]
#![allow(unused_imports)]

// 占位: 空 struct 等 Phase D 填实.
///
/// Phase D 填实目标 (`Cache<K, V>` trait):
/// - `async fn get(&self, key: &K) -> Result<Option<V>, CacheError>`
/// - `async fn put(&self, key: &K, value: &V, ttl: Duration) -> Result<(), CacheError>`
/// - `async fn invalidate(&self, key: &K) -> Result<(), CacheError>`
/// - `fn scope(&self) -> CacheScope` (workspace / session / none)
///
/// 当前仅占位, Phase D 替换为 `InMemoryCache` impl (tokio RwLock<HashMap>).
pub struct VcsCache;

// 占位: CacheError 等 Phase D 填实.
///
/// Phase D 填实目标 (thiserror enum, 4 变体):
/// - `Backend(String)` — 后端错误 (Redis / sled / in-memory 故障)
/// - `Serialization(String)` — serde 序列化/反序列化失败
/// - `KeyNotFound` — invalidate 调用时 key 不存在
/// - `TtlExpired` — get 时 key 已过 TTL (用于触发 stale cache 降级判断)
#[derive(Debug)]
pub struct CacheError;

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(cache-tests): Phase D 填实
    #[test]
    fn placeholder() {
        // 仅占位, Phase D 替换
    }
}
