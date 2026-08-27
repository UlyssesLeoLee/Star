# Spec-01: 数据缓存契约

> **状态**：Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per [ADR-0035 §8.2 Phase G 方向](../../adr/0035-phase-f-architecture.md) / 2026-08-27 21:59 JST 用户授权

## §1 目的

定义 Star 数据层缓存契约，统一 22 domain crate（per [spec/agents/02 §2.1 5 域映射](../agents/02-data-sources-spec.md)） + `star-mcp` Resources（per [spec/mcp/02 §1 4 资源类](../mcp/02-resources-prompts-spec.md)） + `star-sa` Provider（per [spec/vcs/05 §2 4 Provider 接入](../vcs/05-real-providers-spec.md)）的缓存策略。Phase G 落地 `crates/star-cache` + Redis 后端（per ADR-0035 §8.2 L262-264 "缓存层：Redis stream 支撑 star-sse 多 node + star-webhook 持久化"）。

本 spec 解决三个核心问题：
1. **接口统一**：22 domain crate + `star-mcp` Resources + `star-sa` Provider 共享同一 Cache 抽象 trait
2. **失效一致**：TTL + 写穿透 + 主动 invalidate + opt-out 4 策略覆盖 22 crate 全部数据访问路径
3. **可观测**：hit rate / miss rate / latency 指标（per §6 已知缺口 #5，待 Phase G 监控落地）

本 spec 适用范围：Phase G 落地的 `crates/star-cache` crate 暴露的 `CacheBackend` trait，**不**覆盖 Phase F 的 in-memory LRU 临时实现（per `crates/star-mcp/src/resources.rs` 当前 LRU 占位实现，Phase F stub → Phase G 真实实现）。

## §2 Cache 抽象 trait

```rust
// crates/star-cache/src/backend.rs（草案，Phase G 落地）
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("backend connection failed: {0}")]
    ConnectionFailed(String),
    #[error("backend timeout after {0}ms")]
    Timeout(u64),
    #[error("key not found")]
    NotFound,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("backend error: {0}")]
    Backend(String),
}

#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// 获取 key 对应的 value（bytes 形式，调用方负责反序列化）
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// 设置 key-value + TTL（秒），TTL=0 表示永不过期
    async fn set(&self, key: &str, value: &[u8], ttl_sec: u32) -> Result<(), CacheError>;

    /// 删除 key（不存在不报错）
    async fn del(&self, key: &str) -> Result<(), CacheError>;

    /// 判断 key 是否存在
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// 自增/自减（原子操作，delta 可负）
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, CacheError>;

    /// 重设 TTL（key 不存在返回 NotFound）
    async fn expire(&self, key: &str, ttl_sec: u32) -> Result<(), CacheError>;
}
```

### §2.1 错误模型 6 字段

`CacheError` 内部枚举映射到 MCP 6 字段错误模型（per [spec/mcp/03 §1 6 字段定义](../mcp/03-error-model-spec.md)）：

| # | 字段 | 类型 | 用途 |
|---|------|------|------|
| 1 | `code` | SCREAMING_SNAKE_CASE | `CACHE_CONNECTION_FAILED` / `CACHE_TIMEOUT` / `CACHE_KEY_NOT_FOUND` / `CACHE_SERIALIZATION_FAILED` / `CACHE_BACKEND_ERROR` |
| 2 | `message` | string | 人类可读消息（per `#[error("...")]` derive） |
| 3 | `source_module` | enum string | 固定 `mcp`（cache 由 MCP layer 暴露，per spec/mcp/01 §1.1） |
| 4 | `source_kind` | enum string | `internal`（连接/超时）/ `external`（后端错误）/ `validation`（key/value 校验） |
| 5 | `retriable` | boolean | `ConnectionFailed` / `Timeout` = true；`Serialization` = false |
| 6 | `hint` | string? | "检查 Redis 连接 / 重试 / 联系 SRE" |

### §2.2 关键方法约束

- `get` 返回 `None` 表示 key 不存在（**不**返回 `CacheError::NotFound`，因为 NotFound 是正常路径）
- `del` 对不存在的 key 是 no-op（**不**报错）
- `incr` 对不存在的 key 自动初始化为 0，再加 delta
- `expire` 对不存在的 key 返回 `CacheError::NotFound`
- 所有方法必须 thread-safe（trait `Send + Sync`）
- 所有方法必须支持 `tokio` async runtime（per ADR-0035 §3 L155 `crates/star-mcp` 全栈 tokio）

### §2.3 实现要求

Phase G 至少落地 2 个 `CacheBackend` 实现：

| 实现 | 后端 | 用途 | 性能 |
|------|------|------|------|
| `InMemoryBackend` | `dashmap` + `tokio::time::sleep` | 开发 / 测试 / 单 node 部署 | 纳秒级 |
| `RedisBackend` | `redis-rs` 0.27+ | 生产 / 多 node 部署 | 毫秒级 |

## §3 Key 命名规范

Cache key 与 [spec/agents/02 §2.2 URI 模式约束](../agents/02-data-sources-spec.md) `L80-86` 的 MCP Resource URI **形式上不同**（`:` vs `://`），但**语义对齐**（都按 crate + id 维度命名）：

| 形态 | 模板 | 示例 |
|------|------|------|
| 单个 resource | `cache:v1:{crate}:{id}` | `cache:v1:agent:agent-uuid-xxx` |
| 列表 resource | `cache:v1:{crate}:list:{filter_hash}` | `cache:v1:worktree:list:abc123` |
| 派生字段 | `cache:v1:{crate}:{id}:{field}` | `cache:v1:repo:star:default_branch` |
| Provider 元数据 | `cache:v1:sa:{provider}:{key}` | `cache:v1:sa:github:repo:star-101` |

### §3.1 命名规则

- **v1 = schema version**（首个稳定版），后续可 v2/v3 演进
- **`{crate}` 必须匹配 spec/agents/02 §2 22 crate 名单**（agent / worktree / workspace / decision / ...）
- **`{id}` 必须与 crate 主键类型一致**（u64 / uuid / 字符串）
- **`{filter_hash}` 用 SHA-256 前 16 字符**（避免 key 过长，碰撞概率 2^-64）
- **不允许特殊字符**（`:`, `*`, `?`, `\`, 空格 全部禁止；如需分隔用 `_`）

### §3.2 Key 长度限制

- Redis 单 key 建议 ≤ 256 字符
- 推荐 ≤ 200 字符（包含前缀）
- 超过限制返回 `CACHE_KEY_TOO_LONG`（per §6 #1 已知缺口，待 Phase G 实现补）

### §3.3 Key 命名 vs Resource URI 对照

| Resource URI（per spec/agents/02 §2.2） | Cache Key（本 spec §3） | 关系 |
|------------------------------------------|------------------------|------|
| `agent://{agent_id}` | `cache:v1:agent:{agent_id}` | URI id 部分 = key id 部分 |
| `worktree://list?limit=50&offset=0` | `cache:v1:worktree:list:{sha256("limit=50&offset=0")[:16]}` | URI query = filter_hash |
| `decision://{dec_id}` | `cache:v1:decision:{dec_id}` | URI id = key id |
| `repo://{repo_id}/default_branch` | `cache:v1:repo:{repo_id}:default_branch` | URI 路径段 = key field |

`crates/star-mcp/src/resources.rs` `parse_resource_uri()` 解析 Resource URI 后，调用 `cache_key_for_uri(uri)` 函数转换为本 spec §3 模板的 cache key（per ADR-0035 §2 D8 L80-90 缓存层 L3 维度）。

## §4 TTL 策略

per 22 domain crate 数据特征 + 实时性需求（per ADR-0035 §4 L155 缓存策略）：

| 数据类型 | TTL (s) | 说明 | 引用 |
|----------|---------|------|------|
| workspace 详情 | 300 | 5 分钟,变化少 | spec/agents/02 §2 domain-workspace |
| worktree 列表 | 30 | 30 秒,实时性高（git status/wt 状态频繁变） | spec/agents/02 §2 domain-worktree |
| agent state | 5 | 5 秒,心跳频繁（30s heartbeat × 6 = 5s 内必有变化） | spec/agents/02 §2 domain-agent |
| pull request | 60 | 1 分钟,中间态多（Open → Review → Merged 转换频繁） | spec/vcs/05 §3 PR 模型 |
| commit | 86400 | 24 小时,immutable（commit hash = content hash，append-only） | spec/vcs/05 §3 commit 模型 |
| branch | 3600 | 1 小时,变化少（branch 创建/删除相对低频） | spec/vcs/05 §3 branch 模型 |
| decision (Active) | 60 | 1 分钟,高一致性（决策状态需快速同步到 SSE 订阅） | spec/agents/02 §2 domain-decision |
| decision (Superseded/Invalidated) | 86400 | 24 小时,immutable（终态后不可改） | spec/agents/02 §2 domain-decision |
| event (per spec/flows/08) | 300 | 5 分钟,中等实时（event 频繁但非心跳级） | spec/flows/08 §1 event schema |
| audit | 86400 | 24 小时,append-only（audit 不可改，TTL 仅用于 LRU 回收） | spec/agents/02 §2 domain-audit |

### §4.1 TTL 选择 rationale

- **5s / 30s（高频）**：agent state / worktree 列表 — 30s heartbeat + 5s cache TTL 配合，agent 状态变化 30s 内全网可见
- **60s / 300s（中频）**：decision / event / pull request — 中等实时性，避免 Redis 写穿
- **3600s / 86400s（低频）**：branch / commit / audit / 终态 decision — 变化少或 immutable，长 TTL 减少后端压力

### §4.2 TTL 边界

- TTL = 0 表示永不过期（仅适用于 immutable 数据，如 commit hash + content）
- TTL 必须 > 0 且 ≤ 86400（24h）以避免 Redis 内存爆炸（per §6 #1 已知缺口）
- TTL 单位 = 秒（`u32`，最大 4_294_967_295 ≈ 136 年）

## §5 失效策略

4 种失效策略覆盖 22 crate 全部数据访问路径：

| 策略 | 触发 | 实现 | 适用 |
|------|------|------|------|
| **TTL 自然过期** | key 超过 TTL | Redis EXPIRE 自动 | 默认（所有 cache key） |
| **写穿透失效** | Write 提交成功 | write path 同步 `cache.del(key)` | 资源被修改（per spec/agents/02 §4 Write 权限矩阵） |
| **主动 invalidate** | 订阅事件触发 | SSE event → in-memory LRU invalidate | Agent 状态变化、SSE 推送（per spec/services/02 §3） |
| **拒绝缓存** | per-resource opt-out | `cache:false` 请求头 | 实时性要求 100% 一致（如 audit write） |

### §5.1 写穿透失效伪代码

```rust
// crates/star-mcp/src/write_guard.rs（per spec/agents/02 §4.3 伪代码 + 本 spec §5.1 扩展）
async fn write_with_cache_invalidate(
    actor: ActorType,
    target: &ResourceUri,
    new_value: &[u8],
) -> Result<()> {
    // 1. 写穿透 (per §5 写穿透)
    let cache_key = cache_key_for_uri(target);
    cache.del(&cache_key).await?;  // 先失效,避免 stale read

    // 2. 资源存在性 + 权限校验 (per spec/agents/02 §4.3 L152-167)
    let current = target.fetch()?;
    check_write_matrix(actor, &current, WriteOp::Update)?;

    // 3. 状态机校验 (per spec/agents/02 §4.1 L132-138)
    if let Some(new_state) = extract_state(new_value) {
        check_transition(&current.state, &new_state, &actor)?;
    }

    // 4. 写后端
    target.write(new_value).await?;

    // 5. Lease 校验 (per spec/agents/01 §2.1 L108-117)
    if actor == ActorType::Agent {
        check_lease(actor.lease_id(), target)?;
    }

    // 6. 主动 invalidate 广播 (per §5 主动 invalidate)
    sse_bus.publish(SseEvent::CacheInvalidate {
        keys: vec![cache_key.clone()],
    }).await?;

    Ok(())
}
```

### §5.2 主动 invalidate 通道

- Agent 状态变化 → `crates/star-sse` SSE 事件 `CacheInvalidate` 广播
- 多 node 部署 → Redis Pub/Sub 跨 node 同步 invalidate（per §6 #1 已知缺口待 Phase G 落地）
- in-memory LRU → 本 node 立即清；其他 node 通过 Pub/Sub 异步清

### §5.3 拒绝缓存 opt-out

- MCP request `meta.cache: false`（per spec/mcp/01 §1.1 ⑤ 字段扩展）
- 适用场景：audit 写、decision 状态转换、agent 心跳更新
- 行为：跳过 cache.get 直接走后端；写后端后**不**写 cache
- 反例：普通 read 不传 `cache: false` → 走 cache.get，miss 时回源后 set

## §6 已知缺口

| # | 缺口 | 影响 | Phase G 落地计划 |
|---|------|------|------------------|
| 1 | Redis Cluster 模式（vs Standalone）未涉及 | 多 node 部署 + 数据分片场景未设计 | Phase G+ 评估 Redis Cluster 客户端 + 一致性 hash |
| 2 | Cache warming 策略（启动时预热）未设计 | 冷启动 cache miss 率高，后端压力大 | Phase G+ 评估 `cache.warm_keys()` 启动钩子 |
| 3 | 跨 region cache 一致性未涉及 | 多 region 部署 cache 同步延迟 | Phase G+ 评估 Redis Cross-Region Replication + CRDT |
| 4 | Cache 大小限制 + LRU 策略（in-memory backend） | `InMemoryBackend` 内存无限增长风险 | Phase G 落地 LRU 容量上限（默认 10_000 entries） |
| 5 | Cache 监控指标（hit rate / miss rate / latency） | 无法量化 cache 效果 | Phase G 集成 `metrics` crate + Prometheus exporter |
| 6 | 22 domain 接入优先级排期 | spec/agents/02 §6 #1 已知缺口（3 非核心 crate 暂未排期） | Phase G 排期：先 5 核心（agent/worktree/decision/event/audit），再 17 扩展 |

### §6.1 缺口处理原则

per 2026-08-26 11:06 JST Ulysses 拍板"缺标比错标安全"原则：所有缺口**显式列出**而不**默默假设**已解决。本 spec §6 6 项缺口均为 Phase G+ 待办，**不**在 v0.1 范围承诺实现。

## §7 引用文档

- [adr/0023-version-control-provider.md](../../adr/0023-version-control-provider.md) — VCS Core 归 GitGit（per AGENTS.md §6 ADR 索引）
- [adr/0035-phase-f-architecture.md](../../adr/0035-phase-f-architecture.md) — Phase F 整体架构，§8.2 Phase G 方向
- [spec/agents/02-data-sources-spec.md](../agents/02-data-sources-spec.md) — 22 domain 数据源契约，§2.2 URI 模式
- [spec/mcp/02-resources-prompts-spec.md](../mcp/02-resources-prompts-spec.md) — Resources + Prompts，§1 4 资源类
- [spec/mcp/03-error-model-spec.md](../mcp/03-error-model-spec.md) — 6 字段错误模型（§1 定义）
- [spec/vcs/05-real-providers-spec.md](../vcs/05-real-providers-spec.md) — 4 Git Provider 接入（github / gitlab / bitbucket / gitea）
- [spec/flows/08-event-model.md](../flows/08-event-model.md) — event schema（per §4 TTL 策略 event 行）
- [spec/services/02-sse-streaming-spec.md](../services/02-sse-streaming-spec.md) — SSE 推送通道（per §5.2 主动 invalidate）

### §7.1 引用原则

- §2 Cache trait 错误模型对齐 [spec/mcp/03 §1 6 字段定义](../mcp/03-error-model-spec.md) + [agent-api/01-schema.md §3.14](../agent-api/01-schema.md) 唯一权威
- §3 Key 命名与 [spec/agents/02 §2.2 URI 模式](../agents/02-data-sources-spec.md) 语义对齐（`{crate}:{id}` ↔ `{crate}://{id}`）
- §4 TTL 策略与 [spec/agents/02 §2 22 crate 数据特征](../agents/02-data-sources-spec.md) 实时性需求对齐
- §5 失效策略与 [spec/agents/02 §4 Write 权限矩阵](../agents/02-data-sources-spec.md) + [spec/services/02 §3 SSE event schema](../services/02-sse-streaming-spec.md) 协同

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转 + 19:39 JST 代签授权升级 + 21:59 JST 第三次强化） | 初版：Cache trait（§2 含 6 方法 + 6 字段错误映射 + 2 后端实现）+ Key 命名规范（§3 含 4 形态 + 3 命名规则 + 与 Resource URI 对照表）+ 10 类 TTL 策略（§4 含 rationale + 边界）+ 4 失效策略（§5 含写穿透伪代码 + 主动 invalidate 通道 + opt-out）+ 6 已知缺口（§6 含处理原则）+ 8 引用文档（§7 含引用原则） | [ADR-0035 §8.2 Phase G 方向](../../adr/0035-phase-f-architecture.md) "缓存层：Redis stream 支撑 star-sse 多 node + star-webhook 持久化" + 2026-08-27 21:59 JST 用户授权第三次强化代签 |

---

> **审批者**：架构师 (Mavis 接手 agent per DEC-008) — 2026-08-27
> **per AGENTS.md §1 代签规则反转 + 2026-08-27 19:39 JST 代签授权升级 + 21:59 JST 第三次强化**：Mavis 接手默认代签 Ulysses 无需再问
