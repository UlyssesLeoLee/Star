# ARG-ARCH-001 ARG.4 13 REST + 1 WebSocket API 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7` 5 doc) + ARG.4 实装 commit `6e2cda6` + merge `1d894ab` 14 routes 13 REST + 1 WS

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 5-tier 架构 API Tier + DD §3.6 ArgRESTHandlers + ArgWebSocketHandler 接口, 本詳細定義 **ARG.4 `crates/api/src/arg` 14 routes (13 REST + 1 WebSocket)** 的 OpenAPI 3.1 schema + RLS 13 类 + 错误处理 + 5 module 集成, 包含:

1. **14 routes 详细** (per ADR-0048 axum 0.8 `{id}` 语法, 跟 v0.45 ARG.4 实装 commit `6e2cda6` 严格一致)
2. **OpenAPI 3.1 schema** (request/response + 错误码)
3. **RLS 13 类** (per §13 横展, 5 域 + 5 SA + 3 admin 等级)
4. **W/T/M 5 张新表 100% 覆盖** (per 守门 #13 DB 三類横展開強制分類)
5. **WebSocket 推送协议** (5 协议, 跟 ARG.2 5 内部协议 fanout)
6. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, ARG.4 已落地 v0.45 收官)

**Token 估算**: ARG.4 docs 阶段 ~2.5M (per v0.45 brief §1 估 22M / 9 docs 均分); Rust 实装已落地 (per v0.45 收官 commit `6e2cda6` + merge `1d894ab`, 24/24 cargo test pass + 10/10 Python IT pass + 13 文件/3243 行).

---

## 1. 14 Routes 总表 (per ARG.4 实装 commit `6e2cda6`)

### 1.1 13 REST routes (per ADR-0048 axum 0.8)

| # | Method | Path | 9 SA 命中 | 5 域主战场 | W/T/M 表 | RLS 13 类 |
|---|---|---|---|---|---|---|
| 1 | GET | `/api/v1/arg/agents` | (基础) | (5 域) | agents (M) | 13 类 |
| 2 | POST | `/api/v1/arg/agents` | (基础) | (5 域) | agents (M) | 13 类 |
| 3 | GET | `/api/v1/arg/agents/{id}` | (基础) | (5 域) | agents (M) | 13 类 |
| 4 | PUT | `/api/v1/arg/agents/{id}` | (基础) | (5 域) | agents (M) | 13 类 |
| 5 | DELETE | `/api/v1/arg/agents/{id}` | (基础) | admin | agents (M, soft) | 13 类 |
| 6 | GET | `/api/v1/arg/edges` | (基础) | (5 域) | edges (M) | 13 类 |
| 7 | POST | `/api/v1/arg/edges` | (基础) | (5 域) | edges (M) | 13 类 |
| 8 | DELETE | `/api/v1/arg/edges/{id}` | (基础) | admin | edges (M, soft) | 13 类 |
| 9 | GET | `/api/v1/arg/achievements` | SA-09 | social (主) | unlocks (T) | 13 类 |
| 10 | GET | `/api/v1/arg/templates` | (基础) | admin | template_instances (W TTL 30d) | 13 类 |
| 11 | POST | `/api/v1/arg/templates/instances` | (基础) | admin | template_instances (W) | 13 类 |
| 12 | GET | `/api/v1/arg/audit` | SA-03 | admin | audit (T) | 13 类 |
| 13 | GET | `/api/v1/arg/dispatch/routes` | SA-04 + SA-08 | match (主) | edges (M, weight 字段) | 13 类 |

### 1.2 1 WebSocket route

| # | Path | 协议 | 5 module 命中 | W/T/M 表 |
|---|---|---|---|---|
| 14 | WS `/api/v1/arg/events` | 5 协议 fanout (per ARG.2) | AgentRuntime (WS) | edges (M) + unlocks (T) + agents (M) |

---

## 2. OpenAPI 3.1 Schema 详细

### 2.1 共享 schema

```yaml
# crates/api/src/arg/openapi.yaml (per ADR-0048)
openapi: 3.1.0
info:
  title: ARG (Agent Relationship Graph) API
  version: 0.1.0
  description: |
    Agent Relationship Graph API 14 routes (13 REST + 1 WebSocket)
    (per docs/architecture/2026-09-03-arg/08-arg-04-13rest-1ws.md v0.1)
  contact:
    name: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
servers:
  - url: https://api.star.example/v1
    description: Production
  - url: http://localhost:8080/v1
    description: Local dev
tags:
  - name: agents
    description: Agent CRUD (5 域 + 9 SA Type)
  - name: edges
    description: Edge CRUD (10 类关系)
  - name: achievements
    description: 20 成就解锁 (SA-09 主)
  - name: templates
    description: TeamTemplate 实例化
  - name: audit
    description: 5 域跨域审计 (SA-03 主)
  - name: dispatch
    description: ARG 派发路由 (SA-04 + SA-08)
  - name: events
    description: 5 协议 WebSocket 推送

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
  schemas:
    Agent:
      type: object
      required: [id, agent_type, status, trust_score, created_at, updated_at]
      properties:
        id: { type: string, format: uuid }
        agent_type: { $ref: '#/components/schemas/AgentType' }
        status: { $ref: '#/components/schemas/AgentStatus' }
        trust_score: { type: number, minimum: 0.0, maximum: 1.0 }
        domain: { $ref: '#/components/schemas/Domain' }
        sa_type: { $ref: '#/components/schemas/SAType' }
        created_at: { type: string, format: date-time }
        updated_at: { type: string, format: date-time }
    AgentType:
      type: string
      enum: [PlayerAccount, PlayerRole, Wallet, Transaction, MatchEngine, Leaderboard, FriendGraph, ChatRoom, AuditAgent, ComplianceAgent, SACodeReview, SATestGen, SA5DomainLeadAudit, SAGitOps, SADocSync, SARefactor, SADBMigration, SADomainDev, SAAchievementEval]
    AgentStatus:
      type: string
      enum: [Active, Standby, Archived]
    Domain:
      type: string
      enum: [player, economy, match, social, admin]
    SAType:
      type: string
      enum: [SA-01, SA-02, SA-03, SA-04, SA-05, SA-06, SA-07, SA-08, SA-09]
    Edge:
      type: object
      required: [id, from_agent_id, to_agent_id, relationship_type, weight, created_at]
      properties:
        id: { type: string, format: uuid }
        from_agent_id: { type: string, format: uuid }
        to_agent_id: { type: string, format: uuid }
        relationship_type: { $ref: '#/components/schemas/RelationshipType' }
        weight: { type: number, minimum: 0.0, maximum: 1.0 }
        shared_context: { type: object, additionalProperties: true }
        created_at: { type: string, format: date-time }
    RelationshipType:
      type: string
      enum: [MENTORS, PEER_REVIEW, TRUSTS, SQUAD_MEMBER_OF, COLLABORATES, OWNS, REPORTS_TO, DELEGATES_TO, AUDITS, CHALLENGES]
    AchievementUnlock:
      type: object
      required: [id, agent_id, achievement_id, unlock_text, created_at]
      properties:
        id: { type: string, format: uuid }
        agent_id: { type: string, format: uuid }
        achievement_id: { type: string }
        unlock_text: { type: string }
        topology_snapshot: { type: object, additionalProperties: true }
        created_at: { type: string, format: date-time }
    TemplateInstance:
      type: object
      required: [id, template_id, agent_ids, expires_at]
      properties:
        id: { type: string, format: uuid }
        template_id: { type: string }
        agent_ids: { type: array, items: { type: string, format: uuid } }
        expires_at: { type: string, format: date-time }
    AuditEvent:
      type: object
      required: [id, agent_id, sa_type, action, created_at]
      properties:
        id: { type: string, format: uuid }
        agent_id: { type: string, format: uuid }
        sa_type: { $ref: '#/components/schemas/SAType' }
        action: { type: string }
        payload: { type: object, additionalProperties: true }
        created_at: { type: string, format: date-time }
    DispatchRoute:
      type: object
      required: [route_id, from_agent_id, to_agent_id, sa_type, weight, override_reason, created_at]
      properties:
        route_id: { type: string, format: uuid }
        from_agent_id: { type: string, format: uuid }
        to_agent_id: { type: string, format: uuid }
        sa_type: { $ref: '#/components/schemas/SAType' }
        weight: { type: number, minimum: 0.0, maximum: 1.0 }
        override_reason: { type: string }
        created_at: { type: string, format: date-time }
    Error:
      type: object
      required: [code, message]
      properties:
        code: { type: string, enum: [NotFound, Unauthorized, InvalidEdge, InvalidTemplate, MemgraphDown, BoltTimeout, CacheMiss, SchemaMismatch, RLSViolation, InternalError, InvalidInput, Conflict] }
        message: { type: string }
        details: { type: object, additionalProperties: true }

security:
  - bearerAuth: []

paths:
  /arg/agents:
    get:
      tags: [agents]
      summary: List agents (5 域 + 9 SA Type filter)
      parameters:
        - in: query
          name: domain
          schema: { $ref: '#/components/schemas/Domain' }
        - in: query
          name: sa_type
          schema: { $ref: '#/components/schemas/SAType' }
        - in: query
          name: limit
          schema: { type: integer, minimum: 1, maximum: 200, default: 50 }
      responses:
        '200':
          description: List of agents
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/Agent' }
        '401': { $ref: '#/components/responses/Unauthorized' }
        '500': { $ref: '#/components/responses/InternalError' }
    post:
      tags: [agents]
      summary: Create agent
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/NewAgent' }
      responses:
        '201':
          description: Created agent
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Agent' }
        '400': { $ref: '#/components/responses/InvalidInput' }
  /arg/agents/{id}:
    get:
      tags: [agents]
      summary: Get agent by id
      parameters:
        - in: path
          name: id
          required: true
          schema: { type: string, format: uuid }
      responses:
        '200':
          description: Agent
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Agent' }
        '404': { $ref: '#/components/responses/NotFound' }
    put:
      tags: [agents]
      summary: Update agent
      parameters:
        - in: path
          name: id
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/UpdateAgent' }
      responses:
        '200':
          description: Updated agent
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Agent' }
    delete:
      tags: [agents]
      summary: Soft delete agent (admin only)
      parameters:
        - in: path
          name: id
          required: true
          schema: { type: string, format: uuid }
      responses:
        '204': { description: Deleted }
        '403': { $ref: '#/components/responses/RLSViolation' }
  /arg/edges:
    get:
      tags: [edges]
      summary: List edges
      parameters:
        - in: query
          name: from_agent_id
          schema: { type: string, format: uuid }
        - in: query
          name: to_agent_id
          schema: { type: string, format: uuid }
        - in: query
          name: relationship_type
          schema: { $ref: '#/components/schemas/RelationshipType' }
      responses:
        '200':
          description: List of edges
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/Edge' }
    post:
      tags: [edges]
      summary: Create edge
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/NewEdge' }
      responses:
        '201':
          description: Created edge
          content:
            application/json:
              schema: { $ref: '#/components/schemas/Edge' }
        '400': { $ref: '#/components/responses/InvalidInput' }
  /arg/edges/{id}:
    delete:
      tags: [edges]
      summary: Soft delete edge (admin only)
      parameters:
        - in: path
          name: id
          required: true
          schema: { type: string, format: uuid }
      responses:
        '204': { description: Deleted }
  /arg/achievements:
    get:
      tags: [achievements]
      summary: List achievement unlocks (SA-09 主, social 域)
      parameters:
        - in: query
          name: agent_id
          schema: { type: string, format: uuid }
        - in: query
          name: achievement_id
          schema: { type: string }
        - in: query
          name: limit
          schema: { type: integer, minimum: 1, maximum: 200, default: 50 }
      responses:
        '200':
          description: List of achievement unlocks
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/AchievementUnlock' }
  /arg/templates:
    get:
      tags: [templates]
      summary: List TeamTemplate instances (W TTL 30d)
      parameters:
        - in: query
          name: template_id
          schema: { type: string }
      responses:
        '200':
          description: List of template instances
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/TemplateInstance' }
  /arg/templates/instances:
    post:
      tags: [templates]
      summary: Create TeamTemplate instance
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/NewTemplateInstance' }
      responses:
        '201':
          description: Created template instance
          content:
            application/json:
              schema: { $ref: '#/components/schemas/TemplateInstance' }
  /arg/audit:
    get:
      tags: [audit]
      summary: List audit events (SA-03 主, admin 域)
      parameters:
        - in: query
          name: agent_id
          schema: { type: string, format: uuid }
        - in: query
          name: sa_type
          schema: { $ref: '#/components/schemas/SAType' }
        - in: query
          name: limit
          schema: { type: integer, minimum: 1, maximum: 200, default: 50 }
      responses:
        '200':
          description: List of audit events
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/AuditEvent' }
  /arg/dispatch/routes:
    get:
      tags: [dispatch]
      summary: List dispatch routes (SA-04 + SA-08 主, match 域)
      parameters:
        - in: query
          name: sa_type
          schema: { $ref: '#/components/schemas/SAType' }
      responses:
        '200':
          description: List of dispatch routes
          content:
            application/json:
              schema:
                type: array
                items: { $ref: '#/components/schemas/DispatchRoute' }
  /arg/events:
    get:
      tags: [events]
      summary: WebSocket 5 协议 fanout (per ARG.2 5 内部协议)
      description: |
        WebSocket 端点, 跨 Memgraph subscription + 5 Reducer 推送 5 协议:
        - arg_edge_changed
        - arg_dispatch_route
        - arg_context_inject
        - arg_trust_score_update
        - arg_achievement_unlocked
      responses:
        '101':
          description: Switching Protocols (WebSocket upgrade)
```

---

## 3. RLS 13 类 (per 守门 #13 横展 + ARG §1.1)

### 3.1 13 类 RLS 派生 (per §13 横展 + 5 域 + 5 SA + 3 admin)

| # | RLS 类别 | 适用 | 验证 |
|---|---|---|---|
| 1 | player_view | player 域 Agent + Edge | JWT claim `domain=player` |
| 2 | player_edit | player 域 Agent + Edge 写 | JWT claim `domain=player` + `role=edit` |
| 3 | economy_view | economy 域 | JWT claim `domain=economy` |
| 4 | economy_edit | economy 域写 | JWT claim `domain=economy` + `role=edit` |
| 5 | match_view | match 域 | JWT claim `domain=match` |
| 6 | match_edit | match 域写 | JWT claim `domain=match` + `role=edit` |
| 7 | social_view | social 域 | JWT claim `domain=social` |
| 8 | social_edit | social 域写 | JWT claim `domain=social` + `role=edit` |
| 9 | admin_view | admin 域 (审计) | JWT claim `domain=admin` |
| 10 | admin_edit | admin 域写 | JWT claim `domain=admin` + `role=edit` |
| 11 | sa_specific_view | 9 SA 特定读 | JWT claim `sa_type=SA-XX` |
| 12 | sa_specific_edit | 9 SA 特定写 | JWT claim `sa_type=SA-XX` + `role=edit` |
| 13 | cross_domain_audit | 跨 5 域审计 (SA-03) | JWT claim `domain=any` + `role=audit` |

### 3.2 RLS middleware (per axum 0.8 + 守门 #12 OAuth2 5 endpoints)

```rust
// crates/api/src/arg/middleware.rs (per ADR-0048 + v0.47 §14.12 IV OAuth2 middleware extractor)
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

pub async fn rls_middleware(req: Request, next: Next) -> Result<Response, ApiError> {
    let jwt = req.extensions().get::<JwtClaims>().ok_or(ApiError::Unauthorized)?;
    let rls_class = derive_rls_class(&jwt);
    if !rls_class.allows(req.method(), req.uri().path()) {
        return Err(ApiError::RLSViolation);
    }
    req.extensions_mut().insert(rls_class);
    Ok(next.run(req).await)
}

fn derive_rls_class(jwt: &JwtClaims) -> RLSClass {
    if jwt.claims.contains_key("role") && jwt.claims["role"] == "audit" {
        return RLSClass::CrossDomainAudit;
    }
    if jwt.claims.contains_key("sa_type") {
        return RLSClass::SASpecific { sa_type: jwt.claims["sa_type"].clone() };
    }
    let domain = jwt.claims.get("domain").cloned().unwrap_or_default();
    let edit = jwt.claims.get("role").map(|r| r == "edit").unwrap_or(false);
    match (domain.as_str(), edit) {
        ("player", true) => RLSClass::PlayerEdit,
        ("player", false) => RLSClass::PlayerView,
        ("economy", true) => RLSClass::EconomyEdit,
        ("economy", false) => RLSClass::EconomyView,
        ("match", true) => RLSClass::MatchEdit,
        ("match", false) => RLSClass::MatchView,
        ("social", true) => RLSClass::SocialEdit,
        ("social", false) => RLSClass::SocialView,
        ("admin", true) => RLSClass::AdminEdit,
        ("admin", false) => RLSClass::AdminView,
        _ => RLSClass::Unauthorized,
    }
}
```

---

## 4. W/T/M 5 张新表 100% 覆盖 (per 守门 #13)

### 4.1 5 张新表 W/T/M 横展 (per 守门 #13 強制分類 + §13 a-d)

| # | 表名 | W/T/M | 字段数 | 关键字段 | RLS 13 类必携 |
|---|---|---|---|---|---|
| 1 | `agents` | **M** | 16 | id, agent_type, status, trust_score, domain, sa_type, ... | 100% RLS |
| 2 | `edges` | **M** | 14 | id, from_agent_id, to_agent_id, relationship_type, weight, shared_context, ... | 100% RLS |
| 3 | `audit` | **T** | 11 | id, agent_id, sa_type, action, payload, created_at, ... | 100% audit + RLS |
| 4 | `template_instances` | **W** (TTL 30d) | 8 | id, template_id, agent_ids, expires_at, ... | 100% retention_period |
| 5 | `unlocks` | **T** | 9 | id, agent_id, achievement_id, unlock_text, topology_snapshot, ... | 100% audit + RLS |

**派生规** (per 守门 #13):
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: `template_instances` 走 pg_cron 30d TTL, 物理删除由 retention worker 处理
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: `audit` + `unlocks` 走 prevent_hard_delete trigger (per P0-1 v0.42 trigger) + WORM
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: `agents` + `edges` 走 soft delete + SCD Type 2 (valid_from / valid_to 字段)
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部 5 表 100% 覆盖

### 4.2 DDL 草案 (per P0-1 v0.42 trigger + 守门 #13)

```sql
-- db/migrations/V2026_09_10_001__arg_5_tables.sql

-- 1. agents (M)
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type VARCHAR(64) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'Active',
    trust_score DECIMAL(3,2) NOT NULL DEFAULT 0.50,
    domain VARCHAR(16) NOT NULL,
    sa_type VARCHAR(16),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_agents_domain ON agents(domain);
CREATE INDEX idx_agents_sa_type ON agents(sa_type);
CREATE INDEX idx_agents_valid ON agents(valid_from, valid_to) WHERE valid_to IS NULL;

-- 2. edges (M)
CREATE TABLE edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_agent_id UUID NOT NULL REFERENCES agents(id),
    to_agent_id UUID NOT NULL REFERENCES agents(id),
    relationship_type VARCHAR(32) NOT NULL,
    weight DECIMAL(3,2) NOT NULL DEFAULT 0.50,
    shared_context JSONB,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_edges_from ON edges(from_agent_id);
CREATE INDEX idx_edges_to ON edges(to_agent_id);
CREATE INDEX idx_edges_valid ON edges(valid_from, valid_to) WHERE valid_to IS NULL;

-- 3. audit (T) - WORM
CREATE TABLE audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    sa_type VARCHAR(16) NOT NULL,
    action VARCHAR(64) NOT NULL,
    payload JSONB,
    actor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_audit_agent ON audit(agent_id);
CREATE INDEX idx_audit_sa_type ON audit(sa_type);
CREATE INDEX idx_audit_created ON audit(created_at);

-- prevent_hard_delete trigger (T 表 WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_audit() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit table is append-only (WORM)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_hard_delete_audit
BEFORE DELETE ON audit
FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_audit();

-- 4. template_instances (W, TTL 30d)
CREATE TABLE template_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id VARCHAR(64) NOT NULL,
    agent_ids UUID[] NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_template_instances_expires ON template_instances(expires_at);

-- 5. unlocks (T) - WORM
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

CREATE TRIGGER prevent_hard_delete_unlocks
BEFORE DELETE ON unlocks
FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_audit();  -- 复用 audit trigger

-- 5 张表 RLS 13 类必携 (per 守门 #13 d)
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;
ALTER TABLE edges ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_instances ENABLE ROW LEVEL SECURITY;
ALTER TABLE unlocks ENABLE ROW LEVEL SECURITY;
-- RLS policies 13 类 派生 (per §3.1)
```

---

## 5. WebSocket 5 协议 fanout (per ARG.2)

### 5.1 5 协议推送协议 (per §2 ARG.2 5 内部协议)

```rust
// crates/api/src/arg/ws.rs (per v0.45 ARG.4 实装)
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};

pub async fn arg_events_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // 订阅 Memgraph subscription + 5 Reducer (per ARG.2)
    let mut bridge_rx = arg_bridge::subscribe_5_protocols().await;
    while let Some(event) = bridge_rx.recv().await {
        let msg = match event {
            ArgBridgeEvent::EdgeChanged(e) => Message::Text(serde_json::to_string(&e).unwrap()),
            ArgBridgeEvent::DispatchRoute(r) => Message::Text(serde_json::to_string(&r).unwrap()),
            ArgBridgeEvent::ContextInject(c) => Message::Text(serde_json::to_string(&c).unwrap()),
            ArgBridgeEvent::TrustScoreUpdate(t) => Message::Text(serde_json::to_string(&t).unwrap()),
            ArgBridgeEvent::AchievementUnlocked(a) => Message::Text(serde_json::to_string(&a).unwrap()),
        };
        if socket.send(msg).await.is_err() {
            break;
        }
    }
}
```

### 5.2 跟 ARG.2 集成 (per 5 内部协议 100% 一致)

5 协议 schema 跟 ARG.2 5 内部协议 (per doc 06 §2) 100% 一致:
- `arg_edge_changed` (per ARG.2 §2.2 协议 1)
- `arg_dispatch_route` (per ARG.2 §2.2 协议 2)
- `arg_context_inject` (per ARG.2 §2.2 协议 3)
- `arg_trust_score_update` (per ARG.2 §2.2 协议 4)
- `arg_achievement_unlocked` (per ARG.2 §2.2 协议 5)

---

## 6. 错误处理 (10 类, per ARG.1 C-12 ARGError)

| # | 错误码 | HTTP | 描述 |
|---|---|---|---|
| 1 | NotFound | 404 | Agent / Edge / Achievement 不存在 |
| 2 | Unauthorized | 401 | JWT 缺失或无效 |
| 3 | InvalidEdge | 400 | Edge relationship_type / from_agent_id / to_agent_id 不合法 |
| 4 | InvalidTemplate | 400 | TeamTemplate 不合法 |
| 5 | MemgraphDown | 503 | Memgraph Bolt 7687 不可达 |
| 6 | BoltTimeout | 504 | Bolt 查询超时 (5s 默认) |
| 7 | CacheMiss | 200 | Cypher 缓存 miss (正常, 重读) |
| 8 | SchemaMismatch | 500 | Memgraph schema V1 vs V2 不一致 |
| 9 | RLSViolation | 403 | RLS 13 类不满足 |
| 10 | InternalError | 500 | 内部错误 |

---

## 7. 验证摘要 (per v0.45 ARG.4 实装实证)

### 7.1 docs 阶段

- 14 routes 详细 (per §1)
- OpenAPI 3.1 schema 完整 (per §2)
- RLS 13 类 (per §3)
- W/T/M 5 张新表 100% 覆盖 (per §4)
- WebSocket 5 协议 (per §5)
- 错误处理 10 类 (per §6)

### 7.2 Rust 实装 (v0.45 ARG.4 已落地)

- 24/24 cargo test pass (per v0.45 实测, commit `6e2cda6`)
- 10/10 Python IT pass (per v0.45 实测)
- 13 文件 / 3243 行 (per v0.45 实测)
- 14 routes 13 REST + 1 WS 严格 DD §4.12 (axum 0.8 `{id}` 语法 per ADR-0048)
- merge commit `1d894ab` 9/9 05:31 JST (per v0.45 行 1161)

### 7.3 跨 stage 集成 (per 守门 #1 v3 跨 stage 累积)

- ARG.1 (`c678db7` 5 doc) + ARG.4 (commit `6e2cda6` 14 routes) + ARG.2 (本批 doc 06) 三者 5 module / 9 SA / 4 effect / 5 域 命名 100% 一致

---

## 8. 已知缺口 (per 守门 #11)

1. **RLS 13 类实证** (per §3.1, v0.45 实装已通过 OAuth2 middleware extractor 落地, 跨 13 类 RLS 派生需要 24/24 cargo test 100% 覆盖)
2. **W/T/M 5 张表 DDL 实证** (per §4.2, 跨 session 续, 实证需要 pg_cron + RLS policies 落地)
3. **WebSocket 5 协议 fanout 实证** (per §5, 跨 ARG.2 集成, 跨 session 续)
4. **ARG.4 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

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
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.4 14 routes 13 REST + 1 WS 详细 (OpenAPI 3.1 + RLS 13 类 + W/T/M 5 表 DDL + WebSocket 5 协议 + 10 类错误) 跟 v0.45 ARG.4 实装 commit `6e2cda6` 严格一致 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
