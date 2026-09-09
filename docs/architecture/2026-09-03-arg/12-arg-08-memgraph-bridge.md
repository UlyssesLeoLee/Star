# ARG-ARCH-001 ARG.8 Memgraph Integration 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md) · [frontend E2E §0 入口](./09-arg-05-frontend-e2e.md) · [PG persistence §0 入口](./10-arg-06-pg-persistence.md) · [Saga §0 入口](./11-arg-07-arg-saga.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7`) + ARG.1 commit `43c1f0c` MemgraphClient + scripts/automation/memgraph_setup.py

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 Data Tier (Memgraph 部分) + DD §3.1 ArgCrate.client (MemgraphClient + CypherCache), 本詳細定義 **ARG.8 Memgraph Integration** 跨 **Memgraph 2.14 启动 + Bolt 7687 + HTTP 7444 + Cypher schema V1→V2 + 5 内部协议 + 24 组件持久化** 的实施路径, 包含:

1. **Memgraph 2.14 docker compose 启动** (per `scripts/automation/memgraph_setup.py` v0.1)
2. **Bolt 7687 客户端 + Connection Pool** (per ARG.1 C-8 MemgraphClient)
3. **HTTP 7444 mgmt API** (per Memgraph 2.14 mgmt API + health check)
4. **Cypher schema V1→V2 迁移** (per ARG.1 C-8 Migration)
5. **Cypher 5min TTL + 256 MB LRU cache** (per ARG.1 C-9 CypherCache)
6. **5 内部协议写入 Memgraph** (per doc 06 §2 + ARG.2 5 协议)
7. **24 组件持久化路径** (per 基本 §2.1)
8. **离线降级 sled 嵌入** (per ARG.2 C-16 OfflineQueue, 跨进程 pub/sub)
9. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.8 docs 阶段 ~2.0M (per v0.45 brief §1 估 22M / 9 docs 均分); Memgraph 实装跨 session 续 (估 4-6M / 3-5 session).

---

## 1. Memgraph 2.14 docker compose 启动 (per memgraph_setup.py v0.1)

### 1.1 启动命令 (per `scripts/automation/memgraph_setup.py` v0.1)

```bash
# 启动 Memgraph 2.14 (per memgraph_setup.py v0.1 line 60-80)
python scripts/automation/memgraph_setup.py

# 假设已运行 (skip docker)
python scripts/automation/memgraph_setup.py --skip-docker

# 自定义 env file
python scripts/automation/memgraph_setup.py --env-file .env.memgraph
```

**Memgraph 容器**:
- Image: `memgraph/memgraph:2.14`
- Container name: `arg-memgraph`
- Ports: `7687:7687` (Bolt) + `7444:7444` (HTTP / mgmt)
- Volume: `memgraph_data:/var/lib/memgraph`
- Healthcheck: `curl --fail http://localhost:7444/api/v1/health || exit 1` (per 30s interval)

### 1.2 关键 env 变量 (per 守门 #5 env 安全, 2026-08-27 11:06 JST hard ban)

```bash
# .env.memgraph (per memgraph_setup.py, 不打印明文)
MEMGRAPH_USER=memgraph
MEMGRAPH_PASSWORD=memgraph  # 守门 #5: 走 $env:MEMGRAPH_PASSWORD pipe, 不打印
MEMGRAPH_BOLT_URL=bolt://localhost:7687
MEMGRAPH_HTTP_URL=http://localhost:7444
```

**守门 #5 派生**: env 变量**不打印**到任何 log / 终端, 仅 invoke 时引用 (per 8/27 11:06 JST 拍板).

### 1.3 Memgraph 2.14 mgmt API health check (per `memgraph_setup.py` line 70-90)

```python
# scripts/automation/memgraph_setup.py v0.1
import urllib.request

def health_check(http_url: str, max_retries: int = 30) -> bool:
    for i in range(max_retries):
        try:
            with urllib.request.urlopen(f"{http_url}/api/v1/health", timeout=2) as resp:
                if resp.status == 200:
                    return True
        except Exception:
            time.sleep(1)
    return False
```

---

## 2. Bolt 7687 客户端 + Connection Pool (per ARG.1 C-8)

### 2.1 MemgraphClient 草案 (per ARG.1 6 子模块 client)

```rust
// crates/arg/src/client/memgraph.rs (per ARG.1 commit `43c1f0c`)
use bb8::Pool;
use bb8_bolt::BoltConnectionManager;
use std::sync::Arc;

pub struct MemgraphClient {
    pool: Pool<BoltConnectionManager>,
    bolt_url: String,
    user: String,
    password: String,
}

impl MemgraphClient {
    pub async fn new(bolt_url: &str, user: &str, password: &str) -> Result<Self, ARGError> {
        let manager = BoltConnectionManager::new(bolt_url, user, password);
        let pool = Pool::builder()
            .max_size(15)  // 15 连接 (per 24 组件并发派生)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .await?;
        Ok(Self { pool, bolt_url: bolt_url.to_string(), user: user.to_string(), password: password.to_string() })
    }

    pub async fn query(&self, cypher: &str, params: &[(&str, &dyn ToBolt)]) -> Result<Vec<Row>, ARGError> {
        let conn = self.pool.get().await?;
        let rows = conn.query(cypher, params).await?;
        Ok(rows)
    }

    pub async fn execute(&self, cypher: &str, params: &[(&str, &dyn ToBolt)]) -> Result<(), ARGError> {
        let conn = self.pool.get().await?;
        conn.execute(cypher, params).await?;
        Ok(())
    }

    pub async fn subscribe(&self, cypher: &str) -> Result<SubscriptionStream, ARGError> {
        // 走 Bolt 4.0+ SubscriptionRun (per doc 06 §3.1)
        let conn = self.pool.get().await?;
        let stream = conn.subscribe(cypher).await?;
        Ok(stream)
    }
}
```

### 2.2 Connection Pool 调优 (per 24 组件并发派生)

- **max_size = 15** (per 24 组件并发, 5 module × 5 域 = 25 业务高峰, 留 10 缓冲)
- **connection_timeout = 5s** (per Bolt 客户端默认)
- **idle_timeout = 60s** (释放空闲连接)
- **max_lifetime = 3600s** (1h 强制 reconnect, 避免 stale)

---

## 3. HTTP 7444 mgmt API (per Memgraph 2.14)

### 3.1 mgmt API 端点总表 (per Memgraph 2.14 文档)

| 端点 | Method | 用途 |
|---|---|---|
| `/api/v1/health` | GET | 健康检查 (per §1.3) |
| `/api/v1/log` | GET | 实时日志 (调试用) |
| `/api/v1/metrics` | GET | Prometheus 指标 |
| `/api/v1/cluster` | GET | 集群状态 (HA 模式) |
| `/api/v1/database` | GET/POST/DELETE | 多数据库管理 |
| `/api/v1/triggers` | GET/POST/DELETE | Trigger 管理 |
| `/api/v1/indices` | GET/POST/DELETE | 索引管理 |
| `/api/v1/constraints` | GET/POST/DELETE | 约束管理 |
| `/api/v1/streams` | GET/POST/DELETE | Stream 管理 (Kafka/Pulsar) |
| `/api/v1/snapshots` | GET/POST | 快照备份/恢复 |

### 3.2 mgmt API 跟 Memgraph 2.14 集成 (per 守门 #5 env 安全)

```rust
// crates/arg/src/client/mgmt.rs (派生, 跟 MemgraphClient 集成)
pub struct MemgraphMgmtClient {
    http_url: String,
    user: String,
    password: String,
}

impl MemgraphMgmtClient {
    pub async fn health(&self) -> Result<HealthStatus, ARGError> {
        let url = format!("{}/api/v1/health", self.http_url);
        let resp = reqwest::Client::new()
            .get(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send().await?;
        let status: HealthStatus = resp.json().await?;
        Ok(status)
    }

    pub async fn list_triggers(&self) -> Result<Vec<Trigger>, ARGError> {
        let url = format!("{}/api/v1/triggers", self.http_url);
        let resp = reqwest::Client::new()
            .get(&url)
            .basic_auth(&self.user, Some(&self.password))
            .send().await?;
        let triggers: Vec<Trigger> = resp.json().await?;
        Ok(triggers)
    }
}
```

---

## 4. Cypher schema V1→V2 迁移 (per ARG.1 C-8 Migration)

### 4.1 V1 schema (per ARG.1 commit `43c1f0c`)

```cypher
-- V1: ARG 6 子模块基础 schema (per ARG.1 §2.1 24 组件)
// 节点类型
CREATE NODE TABLE IF NOT EXISTS Agent (
    id STRING,
    agent_type STRING,
    status STRING,
    trust_score FLOAT,
    domain STRING,
    sa_type STRING,
    created_at DATETIME,
    updated_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS Edge (
    id STRING,
    from_agent_id STRING,
    to_agent_id STRING,
    relationship_type STRING,
    weight FLOAT,
    shared_context STRING,  -- JSON 序列化
    created_at DATETIME,
    updated_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS TeamTemplate (
    id STRING,
    template_name STRING,
    template_data STRING,  -- JSON 序列化
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS Achievement (
    id STRING,
    achievement_name STRING,
    achievement_type STRING,
    unlock_condition STRING,  -- JSON 序列化
    created_at DATETIME,
    PRIMARY KEY (id)
);

// 关系类型 (10 类 per ARG.1 C-3)
CREATE EDGE TABLE IF NOT EXISTS MENTORS (
    FROM Agent TO Agent,
    weight FLOAT,
    created_at DATETIME
);

CREATE EDGE TABLE IF NOT EXISTS PEER_REVIEW (
    FROM Agent TO Agent,
    weight FLOAT,
    created_at DATETIME
);

-- 8 关系类型类似 (TRUSTS / SQUAD_MEMBER_OF / COLLABORATES / OWNS / REPORTS_TO / DELEGATES_TO / AUDITS / CHALLENGES)

// 索引
CREATE INDEX ON Agent(domain);
CREATE INDEX ON Agent(sa_type);
CREATE INDEX ON Edge(from_agent_id);
CREATE INDEX ON Edge(to_agent_id);
```

### 4.2 V2 schema (per 24 组件扩展)

```cypher
-- V2: 24 组件扩展 (per ARG.1 §2.1)
-- 5 module 扩展节点类型
CREATE NODE TABLE IF NOT EXISTS ARGEvent (
    id STRING,
    event_type STRING,  -- 5 协议之一
    payload STRING,     -- JSON 序列化
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS TrustScore (
    id STRING,
    agent_id STRING,
    old_score FLOAT,
    new_score FLOAT,
    delta_reason STRING,
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS TemplateInstance (
    id STRING,
    template_id STRING,
    agent_ids STRING,   -- JSON 序列化
    expires_at DATETIME,
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS AchievementUnlock (
    id STRING,
    agent_id STRING,
    achievement_id STRING,
    unlock_text STRING,
    topology_snapshot STRING,  -- JSON 序列化
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS Decision (
    id STRING,
    saga_id STRING,
    from_domain STRING,
    to_domain STRING,
    sa_types STRING,    -- JSON 序列化
    decision STRING,
    created_at DATETIME,
    PRIMARY KEY (id)
);

CREATE NODE TABLE IF NOT EXISTS PeerReview (
    id STRING,
    reviewer_id STRING,
    reviewee_id STRING,
    sa_type STRING,
    review_text STRING,
    score FLOAT,
    created_at DATETIME,
    PRIMARY KEY (id)
);

// 新增索引
CREATE INDEX ON ARGEvent(event_type);
CREATE INDEX ON ARGEvent(created_at);
CREATE INDEX ON TrustScore(agent_id);
CREATE INDEX ON TemplateInstance(expires_at);
CREATE INDEX ON AchievementUnlock(agent_id);
CREATE INDEX ON AchievementUnlock(achievement_id);
```

### 4.3 V1→V2 迁移 (per ARG.1 C-8 Migration)

```rust
// crates/arg/src/client/migration.rs (per ARG.1 6 子模块 client)
pub struct MemgraphMigration {
    client: MemgraphClient,
}

impl MemgraphMigration {
    pub async fn migrate_v1_to_v2(&self) -> Result<(), ARGError> {
        // 1. 备份 V1 schema (走 mgmt API snapshot)
        self.snapshot_v1().await?;

        // 2. 创建 V2 schema (增量)
        for cypher in V2_SCHEMA_STATEMENTS {
            self.client.execute(cypher, &[]).await?;
        }

        // 3. 迁移数据 (V1 节点 → V2 节点)
        self.client.execute(
            "MATCH (a:Agent) CREATE (a2:ARGEvent {id: 'migration-' + a.id, event_type: 'migration', payload: '{\"from\":\"v1\",\"to\":\"v2\"}', created_at: a.created_at})",
            &[]
        ).await?;

        // 4. 验证
        self.verify_v2().await?;

        Ok(())
    }
}
```

---

## 5. Cypher 5min TTL + 256 MB LRU cache (per ARG.1 C-9)

### 5.1 CypherCache 草案 (per ARG.1)

```rust
// crates/arg/src/client/cache.rs
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CypherCache {
    cache: Arc<Mutex<LruCache<String, CachedQuery>>>,
    ttl: Duration,  // 默认 5min
}

struct CachedQuery {
    result: serde_json::Value,
    expires_at: Instant,
}

impl CypherCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(max_size))),
            ttl,
        }
    }

    pub async fn query(&self, cypher: &str, params: &[(&str, &dyn ToBolt)]) -> Result<serde_json::Value, ARGError> {
        let key = self.cache_key(cypher, params);
        let mut cache = self.cache.lock().await;

        if let Some(cached) = cache.get(&key) {
            if cached.expires_at > Instant::now() {
                return Ok(cached.result.clone());
            }
        }

        // 缓存 miss, 走 Memgraph (per ARG.1 C-8)
        let result = self.client.query(cypher, params).await?;
        cache.put(key, CachedQuery {
            result: result.clone(),
            expires_at: Instant::now() + self.ttl,
        });
        Ok(result)
    }

    fn cache_key(&self, cypher: &str, params: &[(&str, &dyn ToBolt)]) -> String {
        let mut key = cypher.to_string();
        for (k, v) in params {
            key.push_str(&format!("|{}={}", k, v.to_bolt_string()));
        }
        key
    }
}
```

### 5.2 缓存策略 (per 24 组件 + 5 module 派生)

- **TTL = 5min** (per ARG.1 C-9, 24 组件 5min 内重复查询高概率)
- **max_size = 256 MB** (per ARG.1 C-9, 走 LRU 淘汰)
- **cache_key = cypher + params hash** (per §5.1 cache_key 派生)
- **invalidation**: 写操作 (POST/PUT/DELETE) 触发 cache invalidation (per ARG.1 C-9 派生)

---

## 6. 5 内部协议写入 Memgraph (per doc 06 §2)

### 6.1 5 协议 → Memgraph 写入路径 (per doc 06 §2.2 schema)

```rust
// crates/arg-bridge/src/writer.rs (per ARG.2 5 协议写入)
impl ArgBridgeEventWriter {
    pub async fn write_arg_edge_changed(&self, e: &ArgEdgeChanged) -> Result<(), ARGError> {
        // 1. 写 Memgraph Edge 节点 + MENTORS/PEER_REVIEW/... 边
        self.client.execute(
            "MATCH (a:Agent {id: $from}), (b:Agent {id: $to})
             CREATE (a)-[r:RELATIONSHIP_TYPE {weight: $weight, created_at: $created_at}]->(b)
             SET r.id = $edge_id",
            &[
                ("from", &e.from_agent_id.to_string()),
                ("to", &e.to_agent_id.to_string()),
                ("weight", &e.weight),
                ("RELATIONSHIP_TYPE", &e.relationship_type),
                ("edge_id", &e.edge_id.to_string()),
                ("created_at", &e.created_at),
            ],
        ).await?;

        // 2. 写 ARGEvent 节点 (审计)
        self.client.execute(
            "CREATE (e:ARGEvent {id: $id, event_type: 'arg_edge_changed', payload: $payload, created_at: $created_at})",
            &[
                ("id", &Uuid::new_v4().to_string()),
                ("payload", &serde_json::to_string(e)?),
                ("created_at", &Utc::now()),
            ],
        ).await?;

        // 3. 触发 Bolt subscription fanout (per ARG.2 C-13)
        // 跨进程 fanout 由 Memgraph subscription 处理
        Ok(())
    }

    // 4 个协议写入路径类似 (arg_dispatch_route / arg_context_inject / arg_trust_score_update / arg_achievement_unlocked)
}
```

---

## 7. 24 组件持久化路径 (per 基本 §2.1)

### 7.1 24 组件跟 Memgraph 持久化映射

| 组件 | Memgraph 节点/边类型 | W/T/M (per 守门 #13) | 5 module 命中 |
|---|---|---|---|
| C-1 AgentNode | Agent | M | ArgCrate |
| C-2 Edge | Edge | M | ArgCrate |
| C-3 RelationshipType | (枚举, 无持久化) | — | ArgCrate |
| C-4 TeamTemplate | TeamTemplate | M | ArgCrate |
| C-5 Achievement | Achievement | M | ArgCrate |
| C-6 TrustScoreTier | (枚举, 无持久化) | — | ArgCrate |
| C-7 ARGEvent | ARGEvent | T | ArgCrate |
| C-8 MemgraphClient | (client, 无持久化) | — | ArgCrate |
| C-9 CypherCache | (cache, 内存) | W (TTL 5min) | ArgCrate |
| C-10 EventWriter | (跟 ARGEvent 复用) | T | ArgCrate + AgentLease |
| C-11 LLMService | (LLM API 客户端, 无持久化) | — | LLMService |
| C-12 ARGError | (error 枚举, 无持久化) | — | ArgCrate |
| C-13 MemgraphEventListener | (subscription 监听, 无持久化) | — | AgentLease |
| C-14 LangGraphStateUpdater | (跨 LangGraph 状态, 无 Memgraph 持久化) | — | AgentLease |
| C-15 PeriodFlushWorker | (周期 worker, 无持久化) | — | AgentLease |
| C-16 OfflineQueue | (sled 嵌入, 跨进程 pub/sub) | W (TTL 5min) | AgentLease |
| C-17 ARGDispatchRouter | (effect router, 无持久化) | — | SubAgentOrchestrator |
| C-18 ARGContextInjector | (effect injector, 无持久化) | — | SubAgentOrchestrator |
| C-19 ARGTrustEngine | TrustScore | T | SubAgentOrchestrator |
| C-20 ARGOutputEvaluator | (effect evaluator, 写 ARGEvent) | T | SubAgentOrchestrator |
| C-21 ARGAchievementEngine | AchievementUnlock | T | SubAgentOrchestrator |
| C-22 ArgRESTHandlers | (API 处理器, 无持久化) | — | AgentRuntime |
| C-23 ArgWebSocketHandler | (WS 处理器, 无持久化) | — | AgentRuntime |
| C-24 ArgUIComponents | (frontend 组件, 无持久化) | — | AgentRuntime |

### 7.2 24 组件跟 PG 5 张表映射 (per doc 10 §1.1)

| 组件 | PG 表 (per doc 10) |
|---|---|
| C-1 AgentNode | agents |
| C-2 Edge | edges |
| C-7 ARGEvent | audit |
| C-10 EventWriter | audit (复用) |
| C-15 PeriodFlushWorker | audit (复用) |
| C-16 OfflineQueue | (sled 嵌入, 不入 PG) |
| C-20 ARGOutputEvaluator | audit (复用) |
| C-21 ARGAchievementEngine | unlocks |

---

## 8. 离线降级 sled 嵌入 (per ARG.2 C-16)

### 8.1 OfflineQueue 跟 Memgraph 集成 (per doc 06 §6)

- **sled 1.0 embedded DB** 嵌入 arg-bridge 进程 (per doc 06 §6.1)
- **跨进程 pub/sub**: 走文件系统共享 `/var/lib/star-arg-bridge/offline.sled` (per doc 06 §6.2)
- **5min 重试**: 走 PeriodFlushWorker (per doc 06 §5.1)
- **fallback 协议**: Memgraph 不可达 → OfflineQueue 暂存 → 5min 重连 → 重放

### 8.2 跟 Memgraph 2.14 HA 集成 (per §3.1 /api/v1/cluster)

- 未来 HA 模式: Memgraph 集群 + 主从复制, OfflineQueue 仅在主从切换时触发
- 当前单节点模式: OfflineQueue 主用 fallback

---

## 9. 验证摘要

### 9.1 docs 阶段

- Memgraph 2.14 启动 (per §1)
- Bolt 7687 客户端 (per §2)
- HTTP 7444 mgmt API (per §3)
- Cypher schema V1→V2 (per §4)
- Cypher cache 5min TTL + 256 MB LRU (per §5)
- 5 内部协议写入 (per §6)
- 24 组件持久化 (per §7)
- 离线降级 sled (per §8)

### 9.2 Memgraph 实装 (跨 session 续)

- Memgraph 2.14 启动 (per `memgraph_setup.py` v0.1 已落地, 跨 session 续)
- 24 组件持久化路径 100% (per §7.1)
- V1→V2 迁移脚本 100% (per §4.3)
- 5 协议写入 100% (per §6.1)

### 9.3 跨 stage 集成 (per 守门 #1 v3)

- 跟 ARG.1 + ARG.2 + ARG.4 + ARG.6 命名 100% 一致
- 跨 5 module 持久化 (per §7.1)
- 跨 W/T/M 横展 (per §7.1 + doc 10 §1.1)

---

## 10. 已知缺口 (per 守门 #11)

1. **Memgraph 2.14 实装跨 session 续** (per 估 ~4-6M / 3-5 session, 落档后 v0.51+)
2. **Cypher schema V2 实证** (per §4, 跨 session 续, 实证需要 Memgraph 启动后跑 V1→V2 迁移)
3. **Cypher cache 跨 session 续** (per §5, 跨 session 续, 实证需要 24 组件并发测试)
4. **5 协议写入跨进程 fanout 实证** (per §6, 跨 session 续, 实证需要 Memgraph subscription 跑通)
5. **24 组件持久化端到端 100% 测试** (per §7, 跨 session 续)
6. **OfflineQueue 跨进程 pub/sub 实证** (per §8, 跨 session 续)
7. **ARG.8 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 11. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.8 Memgraph Integration 详细 (Memgraph 2.14 启动 + Bolt 7687 + HTTP 7444 + Cypher V1→V2 + 5 内部协议 + 24 组件持久化 + sled 离线降级) | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 + scripts/automation/memgraph_setup.py v0.1 已落档实证 |
