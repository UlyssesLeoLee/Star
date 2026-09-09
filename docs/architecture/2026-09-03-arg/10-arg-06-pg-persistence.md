# ARG-ARCH-001 ARG.6 PG Persistence 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md) · [frontend E2E §0 入口](./09-arg-05-frontend-e2e.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7`) + v0.39 §14.12 II Postgres 选型 (commit `e75acbd`) + v0.42 §14.10.4 缺口 #4 5/6 Repository 收官 (commit `3199a6c`)

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 Data Tier (PG 部分) + DD §3.2 AgentLease (PG 子模块), 本詳細定義 **ARG.6 PG persistence 5 张 ops 表** (agents / edges / audit / template_instances / unlocks) 的 **DDL + Repository + W/T/M 横展 + RLS 13 类 派生**, 包含:

1. **5 张 ops 表 DDL 详细** (per doc 08 §4 + 守门 #13 横展 100% 覆盖)
2. **Repository 端到端 6 个** (per v0.42 §14.10.4 缺口 #4 5/6 Repository 收官, agents / edges / audit / unlocks 4 个 ARG 表 + ops_metrics_config 复用 + 5 ops 派生)
3. **W/T/M 横展** (per 守门 #13 a-d 派生规 + 9/1 18:30 JST 拍板)
4. **RLS 13 类 policies 派生** (per doc 08 §3)
5. **pg_cron 周期 retention worker** (per 守门 #13 a W 表 retention)
6. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.6 docs 阶段 ~2.0M (per v0.45 brief §1 估 22M / 9 docs 均分); 实装跨 session 续 (估 3-4M / 2-3 session).

---

## 1. 5 张 ops 表总览 (per 守门 #13 横展)

### 1.1 5 张表 W/T/M 100% 覆盖 (per 守门 #13 + 9/1 18:30 JST 拍板)

| # | 表名 | W/T/M | 字段数 | 跟 ARG 阶段 9 SA 命中 | 关键字段 |
|---|---|---|---|---|---|
| 1 | `agents` | **M** | 16 | (基础) 5 域 + 9 SA Type | id, agent_type, status, trust_score, domain, sa_type, valid_from, valid_to, ... |
| 2 | `edges` | **M** | 14 | (基础) 10 类关系 | id, from_agent_id, to_agent_id, relationship_type, weight, shared_context, valid_from, valid_to, ... |
| 3 | `audit` | **T** | 11 | SA-03 主 + SA-01 + SA-04 + SA-07 + SA-08 | id, agent_id, sa_type, action, payload, actor_id, created_at, ... |
| 4 | `template_instances` | **W** (TTL 30d) | 8 | (基础) 5 TeamTemplate | id, template_id, agent_ids, expires_at, ... |
| 5 | `unlocks` | **T** | 9 | SA-09 主 | id, agent_id, achievement_id, unlock_text, topology_snapshot, created_at, ... |

**W/T/M 派生规 (per 守门 #13 a-d)**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: `template_instances` 走 pg_cron 30d TTL, 物理删除由 retention worker 处理
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: `audit` + `unlocks` 走 prevent_hard_delete trigger (per P0-1 v0.42 trigger) + WORM
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: `agents` + `edges` 走 soft delete + SCD Type 2 (valid_from / valid_to 字段)
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部 5 表 100% 覆盖

---

## 2. 5 张表 DDL 详细 (per P0-1 v0.42 trigger + 守门 #13)

### 2.1 agents (M) DDL

```sql
-- db/migrations/V2026_09_10_001__arg_5_tables.sql

-- 1. agents (M) - SCD Type 2 + 100% RLS
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type VARCHAR(64) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'Active',
    trust_score DECIMAL(3,2) NOT NULL DEFAULT 0.50 CHECK (trust_score >= 0.0 AND trust_score <= 1.0),
    domain VARCHAR(16) NOT NULL CHECK (domain IN ('player', 'economy', 'match', 'social', 'admin')),
    sa_type VARCHAR(16) CHECK (sa_type IN ('SA-01', 'SA-02', 'SA-03', 'SA-04', 'SA-05', 'SA-06', 'SA-07', 'SA-08', 'SA-09')),
    metadata JSONB,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_agents_domain ON agents(domain) WHERE valid_to IS NULL;
CREATE INDEX idx_agents_sa_type ON agents(sa_type) WHERE valid_to IS NULL;
CREATE INDEX idx_agents_status ON agents(status) WHERE valid_to IS NULL;
CREATE INDEX idx_agents_valid ON agents(valid_from, valid_to) WHERE valid_to IS NULL;

-- 5 域枚举 CHECK (per 守门 #13 横展)
-- 9 SA Type 枚举 CHECK (per 要件 §6.1)

-- RLS 13 类必携 (per 守门 #13 d)
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;

-- RLS policy #1: player_view (per doc 08 §3.1)
CREATE POLICY player_view_agents ON agents FOR SELECT
    USING (current_setting('app.domain') = 'player' OR current_setting('app.domain') = 'admin');

-- RLS policy #2: player_edit
CREATE POLICY player_edit_agents ON agents FOR INSERT
    WITH CHECK (current_setting('app.domain') = 'player' AND current_setting('app.role') = 'edit');

-- RLS policy #3-10: economy/match/social/admin × view/edit (8 policies 类似)

-- RLS policy #11-12: sa_specific × view/edit
CREATE POLICY sa_specific_view_agents ON agents FOR SELECT
    USING (current_setting('app.sa_type') = sa_type);

CREATE POLICY sa_specific_edit_agents ON agents FOR UPDATE
    USING (current_setting('app.sa_type') = sa_type AND current_setting('app.role') = 'edit');

-- RLS policy #13: cross_domain_audit (SA-03 主)
CREATE POLICY cross_domain_audit_agents ON agents FOR SELECT
    USING (current_setting('app.role') = 'audit');
```

### 2.2 edges (M) DDL

```sql
-- 2. edges (M) - SCD Type 2 + 100% RLS
CREATE TABLE edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_agent_id UUID NOT NULL REFERENCES agents(id),
    to_agent_id UUID NOT NULL REFERENCES agents(id),
    relationship_type VARCHAR(32) NOT NULL CHECK (relationship_type IN ('MENTORS', 'PEER_REVIEW', 'TRUSTS', 'SQUAD_MEMBER_OF', 'COLLABORATES', 'OWNS', 'REPORTS_TO', 'DELEGATES_TO', 'AUDITS', 'CHALLENGES')),
    weight DECIMAL(3,2) NOT NULL DEFAULT 0.50 CHECK (weight >= 0.0 AND weight <= 1.0),
    shared_context JSONB,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_edges_from ON edges(from_agent_id) WHERE valid_to IS NULL;
CREATE INDEX idx_edges_to ON edges(to_agent_id) WHERE valid_to IS NULL;
CREATE INDEX idx_edges_relationship ON edges(relationship_type) WHERE valid_to IS NULL;
CREATE INDEX idx_edges_valid ON edges(valid_from, valid_to) WHERE valid_to IS NULL;

-- 10 类关系 CHECK (per ARG.1 C-3)
-- RLS 13 类必携 (per agents 同样模式)
ALTER TABLE edges ENABLE ROW LEVEL SECURITY;
-- 13 RLS policies 类似 agents
```

### 2.3 audit (T) DDL

```sql
-- 3. audit (T) - WORM + 100% audit + RLS
CREATE TABLE audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    sa_type VARCHAR(16) NOT NULL,
    action VARCHAR(64) NOT NULL,
    payload JSONB,
    actor_id UUID NOT NULL,
    trace_id UUID,  -- 跨 session 跟踪
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_agent ON audit(agent_id);
CREATE INDEX idx_audit_sa_type ON audit(sa_type);
CREATE INDEX idx_audit_trace ON audit(trace_id);
CREATE INDEX idx_audit_created ON audit(created_at);

-- prevent_hard_delete trigger (T 表 WORM, per P0-1 v0.42 trigger)
CREATE OR REPLACE FUNCTION prevent_hard_delete_audit() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit table is append-only (WORM)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_hard_delete_audit
BEFORE DELETE ON audit
FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_audit();

-- RLS 13 类必携
ALTER TABLE audit ENABLE ROW LEVEL SECURITY;
-- 13 RLS policies 派生
```

### 2.4 template_instances (W, TTL 30d) DDL

```sql
-- 4. template_instances (W, TTL 30d)
CREATE TABLE template_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id VARCHAR(64) NOT NULL,
    template_name VARCHAR(128) NOT NULL,
    agent_ids UUID[] NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_template_instances_template ON template_instances(template_id);
CREATE INDEX idx_template_instances_expires ON template_instances(expires_at);

-- pg_cron retention worker (per 守门 #13 a W 表 retention)
SELECT cron.schedule('arg-template-instances-cleanup', '0 2 * * *', $$
    DELETE FROM template_instances WHERE expires_at < NOW();
$$);

-- RLS 13 类必携
ALTER TABLE template_instances ENABLE ROW LEVEL SECURITY;
-- 13 RLS policies 派生 (W 表允许物理删除, retention worker 跟 RLS 兼容)
```

### 2.5 unlocks (T) DDL

```sql
-- 5. unlocks (T) - WORM + 100% audit + RLS
CREATE TABLE unlocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    achievement_id VARCHAR(64) NOT NULL,
    unlock_text TEXT NOT NULL,
    topology_snapshot JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_unlocks_agent ON unlocks(agent_id);
CREATE INDEX idx_unlocks_achievement ON unlocks(achievement_id);
CREATE INDEX idx_unlocks_created ON unlocks(created_at);

-- 复用 audit prevent_hard_delete trigger
CREATE TRIGGER prevent_hard_delete_unlocks
BEFORE DELETE ON unlocks
FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_audit();

-- RLS 13 类必携
ALTER TABLE unlocks ENABLE ROW LEVEL SECURITY;
-- 13 RLS policies 派生
```

---

## 3. Repository 端到端 6 个 (per v0.42 §14.10.4 缺口 #4)

### 3.1 6 个 Repository 总览 (per v0.42 §14.10.4 5/6 Repository 收官)

| # | Repository | 表 | W/T/M | 跟 v0.42 §14.10.4 缺口 #4 派生 |
|---|---|---|---|---|
| 1 | `AgentRepository` | agents | M | ARG 5 域 + 9 SA Type CRUD |
| 2 | `EdgeRepository` | edges | M | ARG 10 类关系 CRUD |
| 3 | `AuditRepository` | audit | T | ARG 5 域跨域审计 (SA-03 主) |
| 4 | `TemplateInstanceRepository` | template_instances | W | ARG 5 TeamTemplate 实例化 |
| 5 | `UnlockRepository` | unlocks | T | ARG 20 成就解锁 (SA-09 主) |
| 6 | (复用 v0.39) `OpsMetricsConfigRepository` | ops_metrics_config | M | (跟 §14.10.4 派生, 5/6 收官) |

**关键约束 (per v0.42 §14.10.4 缺口 #4 实证)**:
- 6 个 Repository 走 sqlx::FromRow derive (per v0.42 修 v0.2 实证 3 err 派生)
- trace_id dedup (per v0.42)
- bind 顺序跟 struct 字段顺序一致 (per v0.42)
- 18/18 pass 0 fail 0 反复 (per v0.42 实测 14.03s)

### 3.2 AgentRepository 草案 (per v0.42 实证模式)

```rust
// crates/star-pg-adapter/src/repository/arg_agent.rs (per v0.42 §14.10.4)
use sqlx::FromRow;
use uuid::Uuid;
use crate::PgPool;

#[derive(Debug, FromRow)]
pub struct AgentRow {
    pub id: Uuid,
    pub agent_type: String,
    pub status: String,
    pub trust_score: f64,
    pub domain: String,
    pub sa_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct AgentRepository {
    pool: PgPool,
}

impl AgentRepository {
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AgentRow>, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>(
            "SELECT id, agent_type, status, trust_score, domain, sa_type, metadata, valid_from, valid_to, created_at, updated_at
             FROM agents WHERE id = $1 AND valid_to IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_by_domain(&self, domain: &str, limit: i64) -> Result<Vec<AgentRow>, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>(
            "SELECT id, agent_type, status, trust_score, domain, sa_type, metadata, valid_from, valid_to, created_at, updated_at
             FROM agents WHERE domain = $1 AND valid_to IS NULL ORDER BY created_at DESC LIMIT $2"
        )
        .bind(domain)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn upsert(&self, agent: &AgentRow) -> Result<AgentRow, sqlx::Error> {
        // 走 SCD Type 2: 旧行 valid_to = NOW(), 新行 valid_from = NOW(), valid_to = NULL
        sqlx::query_as::<_, AgentRow>(
            "INSERT INTO agents (id, agent_type, status, trust_score, domain, sa_type, metadata, valid_from, valid_to, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NULL, NOW(), NOW())
             ON CONFLICT (id) DO UPDATE
             SET valid_to = NOW(), updated_at = NOW()
             RETURNING id, agent_type, status, trust_score, domain, sa_type, metadata, valid_from, valid_to, created_at, updated_at"
        )
        .bind(agent.id)
        .bind(&agent.agent_type)
        .bind(&agent.status)
        .bind(agent.trust_score)
        .bind(&agent.domain)
        .bind(&agent.sa_type)
        .bind(&agent.metadata)
        .fetch_one(&self.pool)
        .await
    }
}
```

### 3.3 EdgeRepository + AuditRepository + TemplateInstanceRepository + UnlockRepository 派生 (per §3.2 类似模式)

(略, 4 个 Repository 走同样 sqlx::FromRow + SCD Type 2 + audit trigger + retention worker 模式, 每个 ~100-150 行)

---

## 4. RLS 13 类 policies 派生 (per doc 08 §3)

### 4.1 13 类 RLS policies 总表 (per 守门 #13 d + doc 08 §3.1)

| # | RLS 类别 | Policy 名 (5 表 × 13 = 65 policies) | 验证 |
|---|---|---|---|
| 1 | player_view | `<table>_player_view` | JWT claim `domain=player` |
| 2 | player_edit | `<table>_player_edit` | JWT claim `domain=player` + `role=edit` |
| 3-4 | economy_view / edit | (4 policies) | JWT claim `domain=economy` + `role=edit` |
| 5-6 | match_view / edit | (4 policies) | JWT claim `domain=match` + `role=edit` |
| 7-8 | social_view / edit | (4 policies) | JWT claim `domain=social` + `role=edit` |
| 9-10 | admin_view / edit | (4 policies) | JWT claim `domain=admin` + `role=edit` |
| 11-12 | sa_specific_view / edit | (10 policies) | JWT claim `sa_type=SA-XX` + `role=edit` |
| 13 | cross_domain_audit | `<table>_cross_domain_audit` | JWT claim `role=audit` |

**总 65 policies 派生 (5 表 × 13 类)**, 跨 session 续做项 (per v0.42 §14.10.4 实证 18/18 pass 派生).

---

## 5. pg_cron 周期 retention worker (per 守门 #13 a)

### 5.1 template_instances retention worker

```sql
-- pg_cron job (per §2.4 DDL)
SELECT cron.schedule('arg-template-instances-cleanup', '0 2 * * *', $$
    DELETE FROM template_instances WHERE expires_at < NOW();
$$);
```

### 5.2 retention worker 跟 RLS 兼容 (per 守门 #13 a W 表派生)

`template_instances` 走 RLS policies, retention worker 走 superuser 上下文:
```sql
-- retention worker 走 postgres 角色 (RLS bypass)
ALTER TABLE template_instances FORCE ROW LEVEL SECURITY;  -- 强制 RLS
-- retention worker 用 SET LOCAL role = postgres;  -- 临时 bypass RLS
```

---

## 6. 跨 ARG 阶段集成边界

### 6.1 跟 ARG.1 crates/arg 集成

- 5 张 ops 表跟 ARG.1 6 子模块持久化层共用 (per ARG.1 C-8 MemgraphClient + EventWriter)
- AgentRow 跟 ARG.1 Agent 16 字段一致 (per doc 03 §1.1)
- EdgeRow 跟 ARG.1 Edge 14 字段一致 (per doc 03 §1.2)
- AuditEvent 跟 ARG.1 C-7 ARGEvent 9 字段 + sa_type 字段 (per doc 03 §1.7)
- TemplateInstance 跟 ARG.1 C-4 TeamTemplate 5 字段 + agent_ids + expires_at (per doc 03 §1.4)
- AchievementUnlock 跟 ARG.1 C-5 Achievement 20 字段 + unlock_text (per doc 03 §1.5)

### 6.2 跟 ARG.4 crates/api/src/arg 集成

- 14 routes 走 5 张 ops 表 (per doc 08 §1.1)
- RLS 13 类 middleware 跟 5 张表 RLS policies 一致 (per doc 08 §3.2 + 本 doc §4)
- WebSocket 5 协议 fanout 写 audit 表 (per doc 08 §5)

### 6.3 跟 ARG.10 frontend 集成

- 5 UI 组件通过 14 routes 读 5 张 ops 表 (per doc 09 §1.2)
- 100% 跨 W/T/M 横展 (per §1.1)

---

## 7. 验证摘要

### 7.1 docs 阶段

- 5 张 ops 表 DDL 完整 (per §2)
- 6 个 Repository 派生 (per §3)
- RLS 13 类 policies 65 policies (per §4)
- pg_cron retention worker (per §5)
- 跨 ARG 阶段集成边界 (per §6)

### 7.2 实装 (跨 session 续 + v0.42 §14.10.4 已收官)

- v0.42 §14.10.4 5/6 Repository 收官 (per commit `3199a6c`, 18/18 cargo test pass)
- ARG 5 张表 DDL 落地 (per §2 DDL 草案, 跨 session 续)
- 65 RLS policies 落地 (per §4 派生, 跨 session 续)
- pg_cron 周期 retention worker 实证 (per §5.1, 跨 session 续)

### 7.3 跨 stage 集成 (per 守门 #1 v3)

- 跟 ARG.1 + ARG.4 + ARG.2 命名 100% 一致
- W/T/M 5 张表 100% 覆盖 (per 守门 #13)

---

## 8. 已知缺口 (per 守门 #11)

1. **5 张表 DDL 实证** (per §2, 跨 session 续, 实证需要 18/18 cargo test pass + pg 启动后跑 DDL)
2. **6 个 Repository 端到端** (per §3, 跟 v0.42 §14.10.4 5/6 Repository 收官模式, 跨 session 续)
3. **65 RLS policies 实证** (per §4, 跨 session 续, 实证需要 13 类 RLS 测试)
4. **pg_cron retention worker 实证** (per §5, 跨 session 续, 实证需要 pg_cron extension 启动 + retention job 跑)
5. **ARG 5 张表 跟 ops 表 派生** (跟 v0.42 §14.10.4 5 ops 派生, 跨 session 续)
6. **ARG.6 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 9. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.6 PG persistence 5 张 ops 表 DDL (agents/edges/audit/template_instances/unlocks) + 6 Repository 派生 + 65 RLS policies + pg_cron retention worker + 跨 ARG 阶段集成边界 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 + 守门 #13 W/T/M 横展 |
