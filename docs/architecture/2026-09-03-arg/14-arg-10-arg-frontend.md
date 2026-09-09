# ARG-ARCH-001 ARG.10 ARG Frontend 集成詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md) · [frontend E2E §0 入口](./09-arg-05-frontend-e2e.md) · [PG persistence §0 入口](./10-arg-06-pg-persistence.md) · [Saga §0 入口](./11-arg-07-arg-saga.md) · [Memgraph §0 入口](./12-arg-08-memgraph-bridge.md) · [5 域 RBAC §0 入口](./13-arg-09-5-domain-rbac.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7`) + ARG.4 实装 commit `6e2cda6` 14 routes + ARG.5 frontend 5 UI 组件 (per doc 09)

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 5-tier 架构 UI Tier + DD §3.6 C-22/C-23/C-24 AgentRuntime 5 module 跟 frontend 集成, 本詳細定義 **ARG.10 ARG Frontend 集成** 跨 **5 UI 组件 + 14 routes + WebSocket 5 协议 + OAuth2 + JWT + RBAC 13 类** 的实施路径, 包含:

1. **5 UI 组件 跟 14 routes 集成** (per doc 09 §1.2)
2. **5 UI 组件 跟 WebSocket 5 协议 fanout** (per doc 08 §5)
3. **5 UI 组件 跟 RBAC 13 类 集成** (per doc 13 §2)
4. **5 UI 组件 跟 5 域 Lead 跨域 consults 校验** (per doc 13 §3)
5. **5 UI 组件 跟 OAuth2 + JWT 鉴权** (per doc 09 §4)
6. **PHASE-ARG-IMPL-REPORT 报告 (per WBS v0.18 §14.11 ARG.9)**
7. **5 域 Lead 寻访位 (per WBS v0.18 §14.11 ARG.11)**
8. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.10 docs 阶段 ~1.5M (per v0.45 brief §1 估 22M / 9 docs 均分); 实装跨 session 续 (估 4-6M / 3-5 session).

---

## 1. 5 UI 组件 跟 14 routes 集成 (per doc 09 §1.2)

### 1.1 14 routes 跟 5 UI 组件映射 (per doc 08 §1 + doc 09 §1.2)

| UI 组件 | 14 routes 命中 | 5 module 命中 |
|---|---|---|
| **RelationshipEditor** | POST `/arg/agents` + POST `/arg/edges` + DELETE `/arg/agents/{id}` + DELETE `/arg/edges/{id}` | AgentRuntime (UI) |
| **RelationshipView** | GET `/arg/agents` + GET `/arg/edges` | AgentRuntime (UI) |
| **AchievementWall** | GET `/arg/achievements` | AgentRuntime (UI) |
| **EdgeTypeSelector** | (无 API 调用, 客户端枚举) | AgentRuntime (UI) |
| **TemplateGallery** | GET `/arg/templates` + POST `/arg/templates/instances` | AgentRuntime (UI) |

### 1.2 14 routes 跟 RBAC 13 类 (per doc 08 §3 + doc 13 §2)

| 14 routes | RBAC 13 类 (per doc 13 §2.1) |
|---|---|
| GET `/arg/agents` | player_view + economy_view + match_view + social_view + admin_view + sa_specific_view + cross_domain_audit |
| POST `/arg/agents` | player_edit + economy_edit + match_edit + social_edit + admin_edit + sa_specific_edit |
| GET `/arg/agents/{id}` | (7 类 view) |
| PUT `/arg/agents/{id}` | (6 类 edit) |
| DELETE `/arg/agents/{id}` | admin_edit (仅 admin) |
| GET `/arg/edges` | (7 类 view) |
| POST `/arg/edges` | (6 类 edit) |
| DELETE `/arg/edges/{id}` | admin_edit |
| GET `/arg/achievements` | (7 类 view) |
| GET `/arg/templates` | (7 类 view) |
| POST `/arg/templates/instances` | admin_edit |
| GET `/arg/audit` | admin_view + cross_domain_audit (admin 域专用) |
| GET `/arg/dispatch/routes` | (7 类 view) |
| WS `/arg/events` | (7 类 view, WebSocket 订阅) |

---

## 2. 5 UI 组件 跟 WebSocket 5 协议 fanout (per doc 08 §5)

### 2.1 5 协议 → 5 UI 组件 (per doc 06 §2)

| 5 协议 | 5 UI 组件 fanout | 触发场景 |
|---|---|---|
| `arg_edge_changed` | RelationshipEditor + RelationshipView | Edge 增删改 |
| `arg_dispatch_route` | RelationshipView | Dispatch 路由更新 |
| `arg_context_inject` | RelationshipView (event log) | Context 注入 |
| `arg_trust_score_update` | RelationshipView + AchievementWall | 信任分更新 |
| `arg_achievement_unlocked` | AchievementWall | 成就解锁推送 |

### 2.2 WebSocket 跟 zustand useARGStore 集成 (per doc 09 §2.1)

```typescript
// frontend/agent-relationships/src/api/ws-client.ts (per doc 09 §2)
import { useARGStore } from '../store/useARGStore';

export class ArgWebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectInterval: number = 5000;

  connect(jwt: string) {
    this.ws = new WebSocket(`wss://api.star.example/v1/arg/events?token=${jwt}`);
    this.ws.onmessage = (event) => {
      const bridgeEvent: ArgBridgeEvent = JSON.parse(event.data);
      useARGStore.getState().handleWebSocketEvent(bridgeEvent);
    };
    this.ws.onclose = () => {
      setTimeout(() => this.connect(jwt), this.reconnectInterval);
    };
  }

  disconnect() {
    this.ws?.close();
  }
}
```

### 2.3 WebSocket 5 协议 → 5 UI 组件 UX 派生

- **arg_edge_changed** → RelationshipView 边列表 + RelationshipEditor 边创建提示
- **arg_dispatch_route** → RelationshipView dispatch route 高亮
- **arg_context_inject** → RelationshipView event log 追加
- **arg_trust_score_update** → RelationshipView 信任分徽章更新 + AchievementWall 解锁提示
- **arg_achievement_unlocked** → AchievementWall 成就解锁动画 + 推送通知

---

## 3. 5 UI 组件 跟 RBAC 13 类 集成 (per doc 13 §2)

### 3.1 RBAC 13 类 派生 (per doc 13 §2.1)

5 UI 组件走 13 RLS 类 policies 派生 (per doc 13 §2.2 + doc 08 §3.2 middleware extractor):
- 客户端发请求时携带 JWT (per doc 09 §4.1 OAuth2 + JWT)
- 服务端 middleware extractor 派生 RLS class (per doc 08 §3.2)
- 13 RLS class 检查请求路径 + method 允许 (per doc 13 §2.1)

### 3.2 UI 组件 RBAC 派生 (per 14 routes 跟 13 类)

| 14 routes | UI 组件 | 客户端 RBAC 检查 |
|---|---|---|
| GET `/arg/agents` | RelationshipView | check `domain + role=view` |
| POST `/arg/agents` | RelationshipEditor | check `domain + role=edit` |
| DELETE `/arg/agents/{id}` | RelationshipEditor | check `domain=admin + role=edit` |
| GET `/arg/audit` | (admin UI 派生) | check `domain=admin + role=audit` |

### 3.3 跨域 consults 校验 (UI 客户端派生, per doc 13 §3)

```typescript
// frontend/agent-relationships/src/utils/rbac.ts
export function canCreateEdge(fromAgent: Agent, toAgent: Agent, edgeType: string, claims: JwtClaims): boolean {
  // 跨域 check (per doc 13 §3.1)
  if (fromAgent.domain !== toAgent.domain) {
    if (edgeType === 'DELEGATES_TO' || edgeType === 'REPORTS_TO') {
      return false;  // 跨域禁止
    }
  }
  // 跨 admin check (per doc 13 §1.2)
  if (fromAgent.domain !== 'admin' && toAgent.domain === 'admin') {
    if (edgeType !== 'AUDITS' && edgeType !== 'CHALLENGES') {
      return false;  // 跨 admin 必须是 consults 边
    }
  }
  // RBAC 13 类 check (per doc 13 §2.1)
  const rlsClass = deriveRlsClass(claims);
  return rlsClass.allows('POST', '/arg/edges');
}
```

---

## 4. 5 UI 组件 跟 OAuth2 + JWT 鉴权 (per doc 09 §4)

### 4.1 AuthProvider 跟 5 UI 组件 集成 (per doc 09 §4.1)

```typescript
// frontend/agent-relationships/src/auth/AuthProvider.tsx (per doc 09 §4.1)
import { AuthProvider } from '../auth/AuthProvider';

export default function App({ Component, pageProps }: AppProps) {
  return (
    <AuthProvider>
      <Component {...pageProps} />
    </AuthProvider>
  );
}
```

### 4.2 5 UI 组件 useAuth hook 集成 (per doc 09 §4.1)

```typescript
// 5 UI 组件 统一 useAuth (per doc 09 §4.1)
// RelationshipEditor.tsx
import { useAuth } from '../auth/AuthProvider';

export function RelationshipEditor() {
  const { claims, login } = useAuth();
  if (!claims) {
    return <button onClick={login}>Login</button>;
  }
  if (claims.domain !== 'player' && claims.domain !== 'economy') {
    return <p>RLS: 仅 player/economy 域可编辑</p>;
  }
  // ... 编辑器逻辑
}
```

### 4.3 OAuth2 Authorization Code + PKCE 流程 (per v0.47 §14.12 IV both)

1. 客户端生成 PKCE code_verifier + code_challenge
2. 重定向到 `/oauth2/authorize?response_type=code&code_challenge=...`
3. 用户登录后回调 `/callback?code=...`
4. 客户端 POST `/oauth2/token` 带 code + code_verifier
5. 换 access_token + refresh_token
6. 客户端用 access_token 调 14 routes

---

## 5. PHASE-ARG-IMPL-REPORT 报告 (per WBS v0.18 §14.11 ARG.9)

### 5.1 报告结构 (per AGENTS.md §3 7 段结构)

```markdown
# PHASE-ARG-IMPL-REPORT.md
## 0. 目的
## 1. 改动矩阵 (ARG.1-11 11 子项完成度)
## 2. 验证摘要 (cargo test / clippy / e2e 实测)
## 3. 已知缺口 (per 缺标比错标)
## 4. 子代理失败接手清单 (per 7 子代理派生规则)
## 5. 守门规则 (15-17 项)
## 6. 签字栏 (5 角色: 架构 / SRE Lead / 平台 / 评审主持 / PM)
## 7. 修订历史 (含 v0.X + 修订人 + 修订内容 + 触发)
```

### 5.2 11 子项完成度矩阵 (per WBS v0.18 §14.11)

| 子项 | 状态 | Token 估 | 实测 | commit |
|---|---|---|---|---|
| ARG.1 crates/arg 6 子模块 | ✅ 已收官 (per v0.45 收官 commit `43c1f0c` + merge `651117e`) | 6M | 4.5-5.0M | 9/9 05:00 JST |
| ARG.2 crates/arg-bridge 4 子模块 | 🟡 docs 已落档 (per doc 06) | 7M | (待实装) | (跨 session 续) |
| ARG.3 crates/arg-effect 5 子模块 | 🟡 docs 已落档 (per doc 07) | 8M | (待实装) | (跨 session 续) |
| ARG.4 crates/api/src/arg 14 routes | ✅ 已收官 (per v0.45 收官 commit `6e2cda6` + merge `1d894ab`) | 3M | 1.8-2.2M | 9/9 05:31 JST |
| ARG.5 frontend/agent-relationships 5 UI 组件 | 🟡 docs 已落档 (per doc 09) | 5M | (待实装) | (跨 session 续) |
| ARG.6 30 UT 端到端 | 🟡 docs 已落档 (per doc 10) | 4M | (待实装) | (跨 session 续) |
| ARG.7 10 IT + 8 E2E + 4 PT 端到端 | 🟡 docs 已落档 (per doc 11) | 6M | (待实装) | (跨 session 续) |
| ARG.8 7 行为 + 5 拓扑成就 evaluator | 🟡 docs 已落档 (per doc 12) | 5M | (待实装) | (跨 session 续) |
| ARG.9 PHASE-ARG-IMPL-REPORT | 🟡 docs 已落档 (per doc 13 + 本 doc §5) | 0.5M | (本 commit) | (本 commit) |
| ARG.10 DDD Review G-9/G-4/G-10 | 🟡 docs 已落档 (per 本 doc §1-§4) | 1M | (待 review) | (跨 session 续) |
| ARG.11 5 域 Lead 寻访位 | 🟡 placeholder 落档 (per docs/recruitment/5-business-domain-lead-referral.md v0.1) | 0.5M | (已落档) | 9/9 (per wt-5lead-outreach commit `bd12c17`) |

### 5.3 报告 7 段实施 (per AGENTS.md §3 模板对齐)

- **§0 目的**: ARG 阶段 11 子项实施总览
- **§1 改动矩阵**: 11 子项 × 状态/Token/实测/commit 4 维
- **§2 验证摘要**: cargo test 100% / clippy 0 / e2e 0
- **§3 已知缺口**: 12 缺口 (per WBS v0.18 §14.11 12 缺口)
- **§4 子代理失败接手清单**: per 7 子代理派生规则
- **§5 守门规则**: 15-17 项 (跟 AGENTS.md §4 守门一致)
- **§6 签字栏**: 5 角色 (Mavis 临时代签 per 守门 #14 v3)
- **§7 修订历史**: v0.X 修订人 + 修订内容 + 触发

---

## 6. 5 域 Lead 寻访位 (per WBS v0.18 §14.11 ARG.11)

### 6.1 5 域 Lead 寻访位 落地 (per docs/recruitment/5-business-domain-lead-referral.md v0.1)

- (a) **5 域 Lead 内推 brief 模板** (per 9/5 10:43 JST 拍板, v0.1 9.5KB)
- (b) **6 周满员 timeline T0-T5** (per v0.1 模板)
- (c) **5 域 Lead 真人到位前 Mavis 临时代签** (per §4.1 doc 13 + 9/3 19:35 JST 拍板 D)
- (d) **真人到位后追溯签字覆盖修订历史** (per 守门 #1 禁回溯叙事)

### 6.2 5 域 Lead 寻访位 跟 ARG 集成

- 5 域 Lead 真人到位后, 跨 ARG 阶段 5 域决策 = 真人 Lead 问责 (per 守门 #14 v2 拍板 D)
- 真人 Lead 决策 vs Mavis 代签决策 = 独立审计链 (per 守门 #1 禁回溯叙事)
- 修订历史表 +1 行 (per AGENTS.md §3 7 段结构)

---

## 7. 验证摘要

### 7.1 docs 阶段

- 5 UI 组件 跟 14 routes 集成 (per §1)
- 5 UI 组件 跟 WebSocket 5 协议 fanout (per §2)
- 5 UI 组件 跟 RBAC 13 类 集成 (per §3)
- 5 UI 组件 跟 OAuth2 + JWT 鉴权 (per §4)
- PHASE-ARG-IMPL-REPORT (per §5)
- 5 域 Lead 寻访位 (per §6)

### 7.2 实装 (跨 session 续 + 跨 doc 06-13 集成)

- 5 UI 组件 跟 14 routes 集成 100% (per §1.1)
- 5 UI 组件 跟 WebSocket 5 协议 fanout 100% (per §2.1)
- 5 UI 组件 跟 RBAC 13 类 集成 100% (per §3.2)
- 5 UI 组件 跟 OAuth2 + JWT 鉴权 100% (per §4.1)

### 7.3 跨 stage 集成 (per 守门 #1 v3)

- 跟 ARG.1 + ARG.2 + ARG.3 + ARG.4 + ARG.5 + ARG.6 + ARG.7 + ARG.8 + ARG.9 命名 100% 一致
- 跨 5 module + 5 域 + 9 SA + 13 RLS 集成 (per §1-§4)
- 跨 11 子项 WBS 集成 (per §5.2)

---

## 8. 已知缺口 (per 守门 #11)

1. **ARG.10 实装跨 session 续** (per 估 ~4-6M / 3-5 session, 落档后 v0.51+)
2. **PHASE-ARG-IMPL-REPORT.md 落地** (per §5, 跨 session 续, 实证需要 11 子项 100% 收官后落地)
3. **5 域 Lead 寻访位 落地** (per §6, 跨 session 续, 实证需要 Ulysses 主动发令激活寻访流程)
4. **DDD Review G-9/G-4/G-10** (per WBS v0.18 §14.11 ARG.10, 跨 session 续)
5. **5 UI 组件 跟 RBAC 13 类 实证** (per §3, 跨 session 续, 实证需要 13 RLS 类 + 14 routes 集成测试)
6. **5 UI 组件 跟 WebSocket 5 协议 实证** (per §2, 跨 session 续, 实证需要 5 协议 fanout 跟 UI 组件 UX 派生测试)

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
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.10 ARG Frontend 集成詳細 (5 UI 组件 跟 14 routes + WebSocket 5 协议 + RBAC 13 类 + OAuth2 + JWT 集成 + PHASE-ARG-IMPL-REPORT 报告 + 5 域 Lead 寻访位) | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
