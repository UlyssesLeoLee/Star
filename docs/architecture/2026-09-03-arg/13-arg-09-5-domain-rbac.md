# ARG-ARCH-001 ARG.9 5 域 RBAC 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md) · [frontend E2E §0 入口](./09-arg-05-frontend-e2e.md) · [PG persistence §0 入口](./10-arg-06-pg-persistence.md) · [Saga §0 入口](./11-arg-07-arg-saga.md) · [Memgraph §0 入口](./12-arg-08-memgraph-bridge.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7`) + RACI §1.2 跨域 consults 派生

---

## 0. 目的 (Objective)

承接基本設計書 §1.3 5 Module 跟 5-tier 架构映射 + RACI §1.2 5 域 Lead 跨域 RACI 约束, 本詳細定義 **ARG.9 5 域 RBAC** 跨 **5 域 (player / economy / match / social / admin) + 5 module + 9 SA + 13 RLS 类** 的实施路径, 包含:

1. **5 域 RBAC 矩阵** (per RACI §1.1 横向 + 5 module 命中)
2. **13 RLS 类 policies** (per doc 08 §3 + doc 10 §4)
3. **5 域 Lead RACI 跨域约束** (per 守门 #3 v2 跨域 consults 派生)
4. **9 SA 跟 5 域 Lead RACI 映射** (per RACI §2.1)
5. **Mavis 临时代签 policy** (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级)
6. **OAuth2 + JWT 鉴权集成** (per v0.47 §14.12 IV)
7. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.9 docs 阶段 ~1.5M (per v0.45 brief §1 估 22M / 9 docs 均分); RBAC 实装跨 session 续 (估 2-3M / 1-2 session).

---

## 1. 5 域 RBAC 矩阵 (per RACI §1.1 + 5 module)

### 1.1 5 域 Lead × 5 module RACI 总表 (per RACI §1.1)

| 5 域 Lead | ArgCrate | SubAgentOrchestrator | LLMService | AgentLease | AgentRuntime |
|---|---|---|---|---|---|
| **player Lead** (per 守门 #3 v2) | R+A | C (player 业务) | C (player prompt) | I | R+A |
| **economy Lead** (per 守门 #3 v2) | R+A | C (economy 业务) | C (economy prompt) | I | R+A |
| **match Lead** (per 守门 #3 v2) | R+A | R+A (Dispatch 路由主战场) | C (match prompt) | I | R+A |
| **social Lead** (per 守门 #3 v2) | R+A | R+A (Context 注入主战场) | C (social prompt) | I | R+A |
| **admin Lead** (per 守门 #3 v2) | R+A | R+A (Trust + Output 评估主战场) | R+A (审计摘要) | R+A (audit trail) | R+A (admin UI) |
| **跨域协调 (Mavis 接手)** | I | I (5 域 Lead 决策) | I | I | I |

**关键派生** (per RACI §1.1):
- **player + economy Lead**: ArgCrate + AgentRuntime 主 R+A (业务 CRUD 主战场)
- **match Lead**: Dispatch 路由是 match 主战场 (effect #1)
- **social Lead**: Context 注入是 social 主战场 (effect #2)
- **admin Lead**: Trust + Output 评估 + 审计是 admin 主战场 (effect #3-#4)
- **跨域协调**: Mavis 接手 (per 守门 #3 v2), 真人到位后追溯签字

### 1.2 5 域 Lead 跨域 RACI 约束 (per 守门 #3 v2 跨域 consults 派生)

| 5 域 Lead → 5 域 Lead | 跨域关系类型 | RACI | 跨域约束 |
|---|---|---|---|
| player → economy | consults (e.g. 玩家代币改动) | C + C | 跨域咨询, 不允许 delegates_to |
| player → match | consults (e.g. 玩家匹配) | C + C | 跨域咨询 |
| player → social | consults (e.g. 玩家好友) | C + C | 跨域咨询 |
| player → admin | consults (e.g. 玩家审计) | C + I | admin 是 I (审计通知) |
| economy → match | consults (e.g. 比赛奖励) | C + C | 跨域咨询 |
| economy → social | consults (e.g. 社交交易) | C + C | 跨域咨询 |
| economy → admin | consults (e.g. 经济审计) | C + I | admin 是 I |
| match → social | consults (e.g. 赛季好友) | C + C | 跨域咨询 |
| match → admin | consults (e.g. 比赛审计) | C + I | admin 是 I |
| social → admin | consults (e.g. 社交合规) | C + I | admin 是 I |

**派生约束 (per 守门 #3 v2)**: 5 域之间**禁止** `delegates_to` / `reports_to` 边, **强制** `consults` / `collaborates` 边. ArgCrate.create_edge() 会检查此约束.

---

## 2. 13 RLS 类 policies (per doc 08 §3 + doc 10 §4)

### 2.1 13 RLS 类总表 (per doc 08 §3.1)

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

### 2.2 13 RLS 类跟 5 PG 表映射 (per doc 10 §4)

每张表 13 RLS 类 policies = 5 表 × 13 = **65 policies**:
- agents: 13 policies (player_view/edit + economy_view/edit + match_view/edit + social_view/edit + admin_view/edit + sa_specific_view/edit + cross_domain_audit)
- edges: 13 policies
- audit: 13 policies (T 表 WORM, RLS 强制)
- template_instances: 13 policies (W 表 TTL 30d, retention worker bypass)
- unlocks: 13 policies (T 表 WORM, RLS 强制)

### 2.3 13 RLS 类跟 Memgraph 集成 (per doc 12 §3 + ARG.1 C-12)

Memgraph 端 13 RLS 类 policies 派生 (per Memgraph 2.14 mgmt API + Auth module):
```cypher
-- Memgraph Auth module (per Memgraph 2.14 文档)
CREATE USER player_view IF NOT EXISTS IDENTIFIED BY 'rls_player_view';
CREATE USER player_edit IF NOT EXISTS IDENTIFIED BY 'rls_player_edit';
-- 13 user 类似
GRANT READ ON Agent, Edge, ARGEvent, TrustScore, AchievementUnlock TO player_view;
GRANT READ, WRITE ON Agent, Edge, ARGEvent, TrustScore, AchievementUnlock TO player_edit;
-- 12 user 类似
```

**跨 Memgraph + PG 双层 RLS 派生** (per 守门 #13 d):
- **Memgraph 层**: 13 user × 24 组件 R/W 权限
- **PG 层**: 13 RLS policies × 5 表 policies
- **双层一致**: Memgraph user name 跟 PG RLS policy name 100% 对应

---

## 3. 5 域 Lead RACI 跨域约束 (per 守门 #3 v2)

### 3.1 跨域 consults 边约束 (per ArgCrate.create_edge 派生)

```rust
// crates/arg/src/ops/edge.rs (per ARG.1 commit `43c1f0c` 6 子模块 ops)
impl ArgCrate {
    pub async fn create_edge(&self, edge: NewEdge) -> Result<Edge, ARGError> {
        // 守门 #3 v2 跨域 consults 校验
        if !self.validate_consults_edge(&edge).await? {
            return Err(ARGError::InvalidCrossDomain);
        }
        // 走 Memgraph + PG 双重写入
        // ...
        Ok(edge)
    }

    async fn validate_consults_edge(&self, edge: &NewEdge) -> Result<bool, ARGError> {
        // 1. 读取 from_agent + to_agent
        let from = self.get_agent(edge.from_agent_id).await?;
        let to = self.get_agent(edge.to_agent_id).await?;

        // 2. 跨域 check (5 域之间禁止 delegates_to / reports_to)
        if from.domain != to.domain {
            if matches!(edge.relationship_type, "DELEGATES_TO" | "REPORTS_TO") {
                return Err(ARGError::InvalidCrossDomain);
            }
        }

        // 3. 跨域 RACI 校验 (per §1.2 派生)
        match (from.domain.as_str(), to.domain.as_str()) {
            ("player", "admin") | ("economy", "admin") | ("match", "admin") | ("social", "admin") => {
                // 跨 admin 必须是 consults 边 (per §1.2)
                if !matches!(edge.relationship_type, "AUDITS" | "CHALLENGES") {
                    return Err(ARGError::InvalidCrossDomain);
                }
            }
            _ => {}
        }
        Ok(true)
    }
}
```

### 3.2 5 域 Lead RACI 跟 9 SA 映射 (per RACI §2.1)

| 9 SA | player Lead | economy Lead | match Lead | social Lead | admin Lead |
|---|---|---|---|---|---|
| **SA-01 code-review** | — | — | — | — | R+A (admin 主) |
| **SA-02 test-gen** | — | — | — | — | R+A (admin 主) |
| **SA-03 5-域-lead-audit** | C | C | C | C | R+A (admin 主) |
| **SA-04 git-ops** | — | — | R+A (match 主) | — | R+A (admin 主) |
| **SA-05 doc-sync** | I | I | I | I | I (跨域协调 Mavis) |
| **SA-06 refactor** | — | — | — | — | R+A (admin 主) |
| **SA-07 db-migration** | — | — | — | — | R+A (admin 主) |
| **SA-08 domain-dev** | R+A | R+A | R+A | R+A | R+A (5 域 R+A) |
| **SA-09 achievement-eval** | — | — | — | R+A (social 主) | — |

---

## 4. Mavis 临时代签 policy (per 守门 #14 v3)

### 4.1 5 域 Lead 真人到位前 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级)

| 5 域 Lead | 真人到位状态 | 临时代签人 | 追溯签字 |
|---|---|---|---|
| **player Lead** | 待定 (per 9/5 10:43 JST 拍板 D) | Mavis 接手 agent per DEC-008 | (真人到位后追溯, per 守门 #1 禁回溯叙事) |
| **economy Lead** | 待定 | Mavis 接手 agent per DEC-008 | (同上) |
| **match Lead** | 待定 | Mavis 接手 agent per DEC-008 | (同上) |
| **social Lead** | 待定 | Mavis 接手 agent per DEC-008 | (同上) |
| **admin Lead** | 待定 | Mavis 接手 agent per DEC-008 | (同上) |
| **跨域协调** | Mavis 接手 (默认) | Mavis 接手 agent per DEC-008 | (永久代签) |

### 4.2 5 域 Lead 真人到位流程 (per docs/recruitment/5-business-domain-lead-referral.md v0.1)

- (a) Ulysses 内推 brief 模板 (per 9/5 10:43 JST 拍板, `docs/recruitment/5-business-domain-lead-referral.md` v0.1)
- (b) 6 周满员 timeline T0-T5 (per v0.1 模板)
- (c) 5 域 Lead 真人到位后 Mavis 临时代签维持 (per 9/3 19:35 JST 拍板 D)
- (d) 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事)

### 4.3 5 域 Lead 真人到位前 commit 形式 (per 守门 #10 + 8/27 19:39 JST 授权)

```bash
# 5 域 Lead 决策 commit (per 守门 #10 author=Ulysses)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'

# 报告"审批者"列 (per 8/27 19:39 JST 授权)
# 形式: 架构师 (Mavis 接手 agent per DEC-008)

# 报告"修订人"列
# 形式: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
```

---

## 5. OAuth2 + JWT 鉴权集成 (per v0.47 §14.12 IV)

### 5.1 OAuth2 5 endpoints (per v0.47 §14.12 IV 拍板"both")

| # | 端点 | Method | 用途 |
|---|---|---|---|
| 1 | `/oauth2/authorize` | GET | Authorization Code + PKCE 启动 |
| 2 | `/oauth2/token` | POST | 换 access_token + refresh_token |
| 3 | `/oauth2/introspect` | POST | token 验证 |
| 4 | `/oauth2/revoke` | POST | token 撤销 |
| 5 | `/.well-known/jwks.json` | GET | 公钥分发 |

### 5.2 JWT claims 派生 (per §2.1 13 RLS 类)

```json
{
  "sub": "agent-uuid",
  "domain": "player",  // 5 域之一
  "role": "edit",       // view / edit / audit
  "sa_type": null,      // 9 SA 之一 (optional)
  "exp": 1234567890,
  "iat": 1234567800
}
```

**13 RLS 类派生** (per doc 08 §3.2 + §2.1):
- `domain` + `role` 派生 5 域 × 2 (view/edit) = 10 类
- `sa_type` 派生 sa_specific_view + sa_specific_edit = 2 类
- `role=audit` 派生 cross_domain_audit = 1 类
- **总计 13 RLS 类**

### 5.3 middleware extractor (per v0.47 §14.12 IV)

```rust
// crates/api/src/arg/middleware.rs (per doc 08 §3.2 + v0.47)
pub async fn oauth2_extractor(req: Request, next: Next) -> Result<Response, ApiError> {
    let jwt = req.extensions().get::<JwtClaims>().ok_or(ApiError::Unauthorized)?;
    let rls_class = derive_rls_class(&jwt);
    if !rls_class.allows(req.method(), req.uri().path()) {
        return Err(ApiError::RLSViolation);
    }
    req.extensions_mut().insert(rls_class);
    Ok(next.run(req).await)
}
```

---

## 6. 验证摘要

### 6.1 docs 阶段

- 5 域 RBAC 矩阵 (per §1)
- 13 RLS 类 policies (per §2)
- 5 域 Lead RACI 跨域约束 (per §3)
- 9 SA 跟 5 域 Lead RACI 映射 (per §3.2)
- Mavis 临时代签 policy (per §4)
- OAuth2 + JWT 鉴权集成 (per §5)

### 6.2 RBAC 实装 (跨 session 续)

- 13 RLS 类 policies 100% (per §2.2)
- 5 域 Lead RACI 跨域约束 100% (per §3.1)
- Mavis 临时代签 policy 维持 (per §4.1)
- OAuth2 + JWT 集成 100% (per §5.3)

### 6.3 跨 stage 集成 (per 守门 #1 v3)

- 跟 ARG.1 + ARG.4 + ARG.6 + ARG.7 + ARG.8 命名 100% 一致
- 跨 5 域 + 5 module + 9 SA 集成 (per §3.2)

---

## 7. 已知缺口 (per 守门 #11)

1. **RBAC 实装跨 session 续** (per 估 ~2-3M / 1-2 session, 落档后 v0.51+)
2. **13 RLS 类 policies 65 policies 实证** (per §2.2, 跨 session 续, 实证需要 5 表 × 13 类 RLS 测试)
3. **Memgraph Auth module 集成** (per §2.3, 跨 session 续, 实证需要 Memgraph 2.14 启动后跑 Auth module)
4. **5 域 Lead 跨域 consults 校验实证** (per §3.1, 跨 session 续, 实证需要 24 组件 + 10 类关系边测试)
5. **5 域 Lead 真人到位流程** (per §4.2, 跨 session 续, 等 Ulysses 主动发令激活)
6. **ARG.9 跟 ARG.10 frontend 集成** (per v0.51+ 续做项)

---

## 8. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.9 5 域 RBAC 详细 (5 域 × 5 module RACI + 13 RLS 类 65 policies + 5 域 Lead 跨域 consults 约束 + 9 SA 映射 + Mavis 临时代签 policy + OAuth2 + JWT 鉴权) | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 + 守门 #3 v2 跨域 consults 派生 |
