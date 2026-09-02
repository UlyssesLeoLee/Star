# domain-vcs 实施 spec

> **状态**: Draft v0.1 (2026-09-03) — **未纳入 `basic-design.md` §2.1 25-Module 表** (该表 domain 计数本身存在矛盾, 见 `docs/refactor/AUDIT-001-requirements-basicdesign-specs.md` F9; 本 spec 不在计数被人工核实/拍板前抢占任何行号或域序号)
> **上游依赖**:
> - `docs/architecture/2026-08-26-upgrade/spec/acceptance/08-acceptance-criteria.md` v0.2 R-007 修复 (2026-08-27) — cache 层落点 `crates/star-vcs/src/cache.rs`
> - `docs/architecture/2026-08-26-upgrade/spec/vcs/01-version-control-provider.md` — Version Control Provider trait 抽象 (4 Git Provider: GitHub / GitLab / Gitea / Bitbucket)
> - `docs/architecture/2026-08-26-upgrade/arch/05-rest-api-spec.md` §5 REST API 14 endpoints — Provider 抽象
> - `docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md` §1.1 ④ — 可缓存 list 结果 (TTL `ttlMs` + `cacheScope`)
> **下游交付**: Implementation team — Rust crate 路径 `crates/star-vcs/` (本 commit 落档 Cargo.toml + lib.rs, src/cache.rs 占位 8/27 已落)
> **最后审稿**: 待 RFC 化时
> **触发**: per 2026-09-03 07:04 JST Ulysses 拍板 "A. 注册" (`docs/plans/PHASE-EXEC-PLAN-2026-09-03.md` §6.2 #1) + RF-001 WBS §1 T1.3 (T1 = 机械级, 零行为风险)

---

## 0. 现状与动机 (为何需要这个 crate)

**8/27 拍板 (per R-007 fix 2026-08-27)**: GitHub / GitLab API 限速风险需 cache 层缓解, 落点 = `crates/star-vcs/src/cache.rs`. 当时 commit `48610ff2` (Mavis 接手 DEC-008) 已落档 `src/cache.rs` 空壳 + 4 项 TODO 占位 (cache-trait / impl-inmem / provider-integration / tests), 但 **没有 Cargo.toml** (未注册 workspace.members), **没有 lib.rs** (cache.rs 无法被外部 `mod` 引用).

**9/3 RF-001 T1.3 拍板 A. 注册**: 跟 8/27 R-007 决策对齐, 补齐缺失的 Cargo.toml + lib.rs + workspace.members 注册. 占位代码保持不变, Phase D 实施时再填实.

**不属于本 crate 的** (显式排除, 避免与相邻机制混淆):
- **Phase D 实施工作** (cache-trait 定义 / InMemoryCache impl / Provider integration / 集成测试) — 留给 phase-d 任务, 不在 T1.3 范围
- **Redis / sled 后端** — Phase 2+ 跨 crate 共享 cache (per cache.rs §"暂不实现"清单)
- **HMAC 链 audit** — R-010 范围, 不属 R-007

## 1. 职责与边界

`star-vcs` 承载 **Version Control Provider 抽象的 cache 层** — 缓解 4 Git Provider (GitHub / GitLab / Gitea / Bitbucket) API 限速, 通过 TTL 失效 + workspace 切换清空, 透明降级到 stale cache.

**属于本 crate 的**:
- `cache::VcsCache` 占位 (Phase D 填实 `Cache<K, V>` trait)
- `cache::CacheError` 占位 (Phase D 填实 thiserror enum)
- Phase D 填实路径: `Cache` trait (get/put/invalidate/scope) + `InMemoryCache` impl (tokio RwLock<HashMap>) + Provider 集成点 (get_issue_cached / get_worktree_cached / search_code_cached, ttl=30s scope=workspace)

**不属于本 crate 的**:
- Version Control Provider trait 本身 (在 `spec/vcs/01-version-control-provider.md` 定义, 不在本 crate)
- MCP / REST 协议层 (在 `star-mcp` / `star-api-rest` / `star-sse` crate, 本 crate 只暴露 `Cache` 给它们用)
- DB 持久化 (Phase 2+ Redis / sled 后端)

## 2. 关键实体 (Phase D 填实)

**VcsCache** (占位, Phase D 填实)
- 标识: provider-specific namespace (github.com / gitlab.com / gitea.io)
- 缓存键: provider API endpoint + query params (URL hash)
- 缓存值: 序列化 response (per `serde_json::Value` 或 typed DTO)
- TTL 策略: per metadata `ttlMs` (默认 30s) + `cacheScope` (workspace / session / none)
- 失效: workspace 切换时清空 (per spec/mcp/01 §1.1 ④)

**CacheError** (占位, Phase D 填实)
- 变体: `Backend(String)` / `Serialization(String)` / `KeyNotFound` / `TtlExpired`

## 3. 关键不变量

| ID | 不变量 | 上游依据 |
|---|---|---|
| INV-VCS-01 | TTL 默认 30s, scope=workspace (per spec/mcp/01 §1.1 ④) | 限速缓解 + workspace 隔离 |
| INV-VCS-02 | Phase 1 单进程 (InMemoryCache), 不跨 crate 共享 (per cache.rs §"暂不实现") | R-007 阶段, 不预判 Phase 2+ 跨进程需求 |
| INV-VCS-03 | 429 响应时降级到 stale cache (per cache.rs TODO(tests) #3) | API 限速降级, 不阻塞 UI |
| INV-VCS-04 | workspace 切换时清空 cache (per cache.rs TODO(tests) #2) | 数据隔离, 避免跨租户数据泄漏 |

## 4. 接口签名 (Phase D 填实)

```rust
// crates/star-vcs/src/cache.rs (Phase D 填实目标)

pub trait Cache<K, V> {
    async fn get(&self, key: &K) -> Result<Option<V>, CacheError>;
    async fn put(&self, key: &K, value: &V, ttl: Duration) -> Result<(), CacheError>;
    async fn invalidate(&self, key: &K) -> Result<(), CacheError>;
    fn scope(&self) -> CacheScope;  // workspace / session / none
}

pub struct InMemoryCache { /* tokio::sync::RwLock<HashMap> */ }
```

## 5. 当前占位状态 (8/27 → 9/3)

| 元素 | 状态 | 触发 |
|---|---|---|
| `crates/star-vcs/src/cache.rs` | 🟢 已落档 (2403 字节, 8/27 commit `48610ff2` Mavis 接手) | R-007 落点 |
| `crates/star-vcs/Cargo.toml` | 🟢 本 commit 落档 (9/3, 622 字节) | T1.3 拍板 A |
| `crates/star-vcs/src/lib.rs` | 🟢 本 commit 落档 (9/3, 913 字节) | T1.3 拍板 A |
| root `Cargo.toml` workspace.members 注册 | 🟢 本 commit 落档 (9/3) | T1.3 拍板 A |
| `cache::VcsCache` trait 定义 | 🟡 Phase D 待填实 (per cache.rs TODO(cache-trait)) | phase-d 任务 |
| `InMemoryCache` impl | 🟡 Phase D 待填实 (per cache.rs TODO(impl-inmem)) | phase-d 任务 |
| Provider 集成点 | 🟡 Phase D 待填实 (per cache.rs TODO(provider-integration)) | phase-d 任务 |
| 集成测试 | 🟡 Phase D 待填实 (per cache.rs TODO(tests)) | phase-d 任务 |

## 6. 已知缺口 (per 缺标比错标安全守门)

1. **未跑 cargo check 实证**: 本 commit 仅落档 Cargo.toml + lib.rs, 实际 cargo check --workspace 验证待 9/3 commit 后跑 (per Phase 1 后续任务)
2. **Phase D 实施工作未排期**: cache-trait / impl-inmem / provider-integration / tests 4 项 TODO 需 phase-d 任务承接, 当前 WBS §7 待办无具体排期
3. **Redis / sled 后端** (per cache.rs §"暂不实现"): Phase 2+ 跨 crate 共享, 排期未定
4. **HMAC 链 audit** (R-010 范围, 不属 R-007): 排期见 R-010 任务
5. **Provider 集成点 4 Git Provider 范围**: 8/27 R-007 写"GitHub / GitLab / Gitea", 9/3 §1 扩到含 Bitbucket — 4 Provider 完整覆盖待 Phase D 实施时确认

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: R-007 落点 spec, 9/3 T1.3 拍板 A 注册 (Cargo.toml + lib.rs + workspace.members), 5 项已知缺口 | 2026-09-03 07:04 JST Ulysses 拍板 A (`docs/plans/PHASE-EXEC-PLAN-2026-09-03.md` §6.2 #1) + RF-001 WBS §1 T1.3 |
