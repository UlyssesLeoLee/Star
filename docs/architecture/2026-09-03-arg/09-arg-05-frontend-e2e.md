# ARG-ARCH-001 ARG.5 Frontend E2E 詳細 (per brief v0.50 §14.11)

> **Status**: 🟡 Draft v0.1 (per 2026-09-10 01:36 JST 派发 brief v0.50 §14.11 ARG.2-11 docs 阶段)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [要件 §0 入口](./01-requirements.md) · [基本 §0 入口](./02-basic-design.md) · [詳細 §0 入口](./03-detailed-design.md) · [RACI §0 入口](./04-raci.md) · [Skill Diagrams §0 入口](./05-skill-diagrams.md) · [arg-bridge §0 入口](./06-arg-02-arg-bridge.md) · [SA 实施 §0 入口](./07-arg-03-5-sa-impl.md) · [13 REST + 1 WS §0 入口](./08-arg-04-13rest-1ws.md)
> **承接**: 守门 #14 v2 拍板 D + 9/3 11:35 JST 拍板 B 反转 + 9/5 10:43 JST 拍板 D 维持 + 9/9 12:02 JST 守门 #14 v3 升级 + v0.45 ARG.1 docs (`c678db7` 5 doc) + ARG.4 实装 (commit `6e2cda6` 14 routes)

---

## 0. 目的 (Objective)

承接基本設計書 §1.1 5-tier 架构 UI Tier + DD §3.6 C-24 ArgUIComponents, 本詳細定義 **ARG.5 `frontend/agent-relationships` 5 UI 组件** (RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery) 的 **E2E 端到端路径**, 包含:

1. **5 UI 组件 物理布局** (per 基本 §1.1 UI Tier + DD §3.6 C-24)
2. **zustand `useARGStore` 5 channel** (agents / edges / templates / achievements / events WebSocket)
3. **Playwright E2E 测试** (per WBS v0.18 §14.11 ARG.7 8 E2E)
4. **OAuth2 + JWT 鉴权** (per v0.47 §14.12 IV OAuth2 5 endpoints + middleware extractor)
5. **跨 session 续做边界** (per WBS v0.18 line 663 估 ~22M tokens / 5-7 session, 落档后 v0.51+)

**Token 估算**: ARG.5 docs 阶段 ~1.2M (per v0.45 brief §1 估 22M / 9 docs 均分); frontend E2E 实装跨 session 续 (估 4-6M / 3-5 session).

---

## 1. 5 UI 组件 物理布局

### 1.1 frontend/agent-relationships 目录结构

```
frontend/agent-relationships/
├── package.json                 # Next.js 14 + zustand 4 + Playwright 1.40
├── src/
│   ├── app/
│   │   ├── layout.tsx           # 全局 layout (per 守门 #12 OAuth2 AuthProvider)
│   │   ├── page.tsx             # 主入口: RelationshipView
│   │   ├── editor/page.tsx      # RelationshipEditor
│   │   ├── achievements/page.tsx  # AchievementWall
│   │   ├── templates/page.tsx   # TemplateGallery
│   │   └── api/auth/[...nextauth]/route.ts  # OAuth2 callback (per v0.47 §14.12 IV)
│   ├── components/
│   │   ├── RelationshipEditor.tsx    # 关系编辑 (5 域 + 9 SA Type + 10 关系类型)
│   │   ├── RelationshipView.tsx      # 关系可视化 (D3 7.8 + Memgraph Bolt)
│   │   ├── AchievementWall.tsx       # 成就墙 (SA-09 推送)
│   │   ├── EdgeTypeSelector.tsx      # 边类型选择器 (10 类)
│   │   └── TemplateGallery.tsx       # 模板画廊 (5 TeamTemplate)
│   ├── store/
│   │   └── useARGStore.ts       # zustand 5 channel store
│   ├── api/
│   │   └── arg-client.ts        # 14 routes HTTP client + WS subscriber
│   ├── auth/
│   │   ├── AuthProvider.tsx     # OAuth2 + JWT 鉴权 (per v0.47)
│   │   └── useAuth.ts           # JWT 派生 hook
│   └── e2e/                     # Playwright 8 E2E
│       ├── relationship-editor.spec.ts  # 3 E2E
│       ├── achievement-wall.spec.ts     # 2 E2E
│       └── template-gallery.spec.ts     # 3 E2E
```

### 1.2 5 UI 组件 跟 5 module 集成 (per 基本 §1.1 + §1.3)

| UI 组件 | 5 module 命中 | 5-tier | 14 routes 命中 | WebSocket 命中 |
|---|---|---|---|---|
| **RelationshipEditor** | AgentRuntime (UI) | UI | POST `/arg/agents` + POST `/arg/edges` + DELETE `/arg/agents/{id}` | arg_edge_changed + arg_dispatch_route |
| **RelationshipView** | AgentRuntime (UI) | UI | GET `/arg/agents` + GET `/arg/edges` | (订阅全部 5 协议) |
| **AchievementWall** | AgentRuntime (UI) | UI | GET `/arg/achievements` | arg_achievement_unlocked |
| **EdgeTypeSelector** | AgentRuntime (UI) | UI | (无 API 调用, 客户端枚举) | — |
| **TemplateGallery** | AgentRuntime (UI) | UI | GET `/arg/templates` + POST `/arg/templates/instances` | — |

---

## 2. zustand useARGStore 5 channel

### 2.1 5 channel 总表 (per 基本 §1.1)

| # | Channel | 类型 | 来源 | WebSocket 同步 |
|---|---|---|---|---|
| 1 | `agents` | `Map<AgentId, Agent>` | GET `/arg/agents` | arg_edge_changed / arg_trust_score_update / arg_achievement_unlocked |
| 2 | `edges` | `Map<EdgeId, Edge>` | GET `/arg/edges` | arg_edge_changed / arg_dispatch_route |
| 3 | `templates` | `Map<TemplateId, TeamTemplate>` | GET `/arg/templates` | — |
| 4 | `achievements` | `Map<AgentId, AchievementUnlock[]>` | GET `/arg/achievements` | arg_achievement_unlocked |
| 5 | `events` | `EventLog[]` (WebSocket) | WS `/arg/events` | (5 协议 fanout) |

### 2.2 useARGStore 草案 (per zustand 4)

```typescript
// frontend/agent-relationships/src/store/useARGStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export interface ARGStore {
  agents: Map<string, Agent>;
  edges: Map<string, Edge>;
  templates: Map<string, TeamTemplate>;
  achievements: Map<string, AchievementUnlock[]>;
  events: EventLog[];

  // Actions
  loadAgents: () => Promise<void>;
  loadEdges: () => Promise<void>;
  loadTemplates: () => Promise<void>;
  loadAchievements: () => Promise<void>;
  handleWebSocketEvent: (event: ArgBridgeEvent) => void;
  createAgent: (newAgent: NewAgent) => Promise<Agent>;
  createEdge: (newEdge: NewEdge) => Promise<Edge>;
  deleteAgent: (id: string) => Promise<void>;
  deleteEdge: (id: string) => Promise<void>;
}

export const useARGStore = create<ARGStore>()(
  subscribeWithSelector((set, get) => ({
    agents: new Map(),
    edges: new Map(),
    templates: new Map(),
    achievements: new Map(),
    events: [],

    loadAgents: async () => {
      const agents = await argClient.get('/arg/agents');
      set({ agents: new Map(agents.map(a => [a.id, a])) });
    },

    loadEdges: async () => {
      const edges = await argClient.get('/arg/edges');
      set({ edges: new Map(edges.map(e => [e.id, e])) });
    },

    loadTemplates: async () => {
      const templates = await argClient.get('/arg/templates');
      set({ templates: new Map(templates.map(t => [t.id, t])) });
    },

    loadAchievements: async () => {
      const unlocks = await argClient.get('/arg/achievements');
      const grouped = new Map<string, AchievementUnlock[]>();
      for (const u of unlocks) {
        if (!grouped.has(u.agent_id)) grouped.set(u.agent_id, []);
        grouped.get(u.agent_id)!.push(u);
      }
      set({ achievements: grouped });
    },

    handleWebSocketEvent: (event) => {
      const events = [...get().events, { event, received_at: new Date().toISOString() }];
      set({ events: events.slice(-100) });  // 保留最近 100 条

      switch (event.type) {
        case 'arg_edge_changed':
          get().loadEdges();
          break;
        case 'arg_dispatch_route':
          get().loadEdges();
          break;
        case 'arg_trust_score_update':
          get().loadAgents();
          break;
        case 'arg_achievement_unlocked':
          get().loadAchievements();
          break;
        case 'arg_context_inject':
          // 仅 log, 不刷
          break;
      }
    },

    createAgent: async (newAgent) => {
      const agent = await argClient.post('/arg/agents', newAgent);
      const agents = new Map(get().agents);
      agents.set(agent.id, agent);
      set({ agents });
      return agent;
    },

    createEdge: async (newEdge) => {
      const edge = await argClient.post('/arg/edges', newEdge);
      const edges = new Map(get().edges);
      edges.set(edge.id, edge);
      set({ edges });
      return edge;
    },

    deleteAgent: async (id) => {
      await argClient.delete(`/arg/agents/${id}`);
      const agents = new Map(get().agents);
      agents.delete(id);
      set({ agents });
    },

    deleteEdge: async (id) => {
      await argClient.delete(`/arg/edges/${id}`);
      const edges = new Map(get().edges);
      edges.delete(id);
      set({ edges });
    },
  }))
);
```

---

## 3. Playwright E2E 测试 (per WBS v0.18 §14.11 ARG.7 8 E2E)

### 3.1 8 E2E 分布 (per 3 spec 文件)

| Spec 文件 | E2E 数量 | 5 UI 组件 命中 | 14 routes 命中 |
|---|---|---|---|
| `relationship-editor.spec.ts` | 3 | RelationshipEditor | POST agents/edges + DELETE |
| `achievement-wall.spec.ts` | 2 | AchievementWall | GET achievements + WS |
| `template-gallery.spec.ts` | 3 | TemplateGallery | GET templates + POST instances |

### 3.2 E2E 示例 (relationship-editor.spec.ts #1)

```typescript
// frontend/agent-relationships/src/e2e/relationship-editor.spec.ts
import { test, expect } from '@playwright/test';
import { loginAsPlayer } from '../helpers/auth';

test.describe('RelationshipEditor E2E', () => {
  test('player can create mentor edge between two PlayerRoles', async ({ page }) => {
    // 1. OAuth2 login (per v0.47 §14.12 IV)
    await loginAsPlayer(page);

    // 2. 导航到 RelationshipEditor
    await page.goto('/editor');

    // 3. 选择 source agent (PlayerRole)
    await page.locator('[data-testid="from-agent-select"]').click();
    await page.locator('[data-testid="agent-option-player-role-alice"]').click();

    // 4. 选择 target agent (PlayerRole)
    await page.locator('[data-testid="to-agent-select"]').click();
    await page.locator('[data-testid="agent-option-player-role-bob"]').click();

    // 5. 选择 relationship_type
    await page.locator('[data-testid="edge-type-selector"]').click();
    await page.locator('[data-testid="edge-type-MENTORS"]').click();

    // 6. 设置 weight
    await page.locator('[data-testid="edge-weight-input"]').fill('0.8');

    // 7. 提交
    await page.locator('[data-testid="create-edge-submit"]').click();

    // 8. 验证 POST 成功 + UI 更新
    await expect(page.locator('[data-testid="edge-list"]')).toContainText('MENTORS');

    // 9. 验证 WebSocket fanout
    await expect(page.locator('[data-testid="event-log"]')).toContainText('arg_edge_changed');
  });

  test('RLS violation: player cannot create edge in admin domain', async ({ page }) => {
    await loginAsPlayer(page);
    await page.goto('/editor');

    await page.locator('[data-testid="from-agent-select"]').click();
    await page.locator('[data-testid="agent-option-admin-audit"]').click();

    await page.locator('[data-testid="to-agent-select"]').click();
    await page.locator('[data-testid="agent-option-admin-compliance"]').click();

    await page.locator('[data-testid="create-edge-submit"]').click();

    await expect(page.locator('[data-testid="error-message"]')).toContainText('RLS Violation');
  });

  test('WebSocket reconnection after disconnect', async ({ page, context }) => {
    await loginAsPlayer(page);
    await page.goto('/');

    // 等待 WS 连接
    await expect(page.locator('[data-testid="ws-status"]')).toHaveText('connected');

    // 强制断开
    await context.setOffline(true);
    await expect(page.locator('[data-testid="ws-status"]')).toHaveText('disconnected');

    // 重新连接
    await context.setOffline(false);
    await expect(page.locator('[data-testid="ws-status"]')).toHaveText('connected');
  });
});
```

### 3.3 E2E 跨 session 续做

- 8 E2E 跨 3 spec 文件 (per §3.1)
- 跨 OAuth2 + JWT 鉴权 (per v0.47 §14.12 IV)
- 跨 WebSocket 5 协议 fanout (per doc 08 §5)
- 跨 RLS 13 类 (per doc 08 §3)

---

## 4. OAuth2 + JWT 鉴权 (per v0.47 §14.12 IV)

### 4.1 AuthProvider (per v0.47 §14.12 IV OAuth2 5 endpoints)

```typescript
// frontend/agent-relationships/src/auth/AuthProvider.tsx
import { createContext, useContext, useEffect, useState, ReactNode } from 'react';

interface AuthContext {
  jwt: string | null;
  claims: JwtClaims | null;
  login: () => Promise<void>;
  logout: () => void;
}

const AuthCtx = createContext<AuthContext | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [jwt, setJwt] = useState<string | null>(null);
  const [claims, setClaims] = useState<JwtClaims | null>(null);

  useEffect(() => {
    const stored = localStorage.getItem('arg_jwt');
    if (stored) {
      setJwt(stored);
      setClaims(parseJwt(stored));
    }
  }, []);

  const login = async () => {
    // 走 OAuth2 Authorization Code + PKCE (per v0.47 §14.12 IV both)
    const code = await new Promise<string>((resolve) => {
      window.location.href = `/api/auth/login?redirect_uri=${encodeURIComponent(window.location.origin + '/callback')}`;
      window.addEventListener('message', (e) => resolve(e.data.code), { once: true });
    });
    const tokenResp = await fetch('/api/auth/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, code_verifier: sessionStorage.getItem('pkce_verifier') }),
    });
    const { jwt } = await tokenResp.json();
    localStorage.setItem('arg_jwt', jwt);
    setJwt(jwt);
    setClaims(parseJwt(jwt));
  };

  const logout = () => {
    localStorage.removeItem('arg_jwt');
    setJwt(null);
    setClaims(null);
  };

  return <AuthCtx.Provider value={{ jwt, claims, login, logout }}>{children}</AuthCtx.Provider>;
}

export const useAuth = () => {
  const ctx = useContext(AuthCtx);
  if (!ctx) throw new Error('useAuth must be inside AuthProvider');
  return ctx;
};
```

### 4.2 RLS 派生 (per doc 08 §3)

JWT claims 包含 `domain` + `role` + `sa_type`, 派生 RLS 13 类 (per doc 08 §3.1), 14 routes 走 RLS middleware (per doc 08 §3.2).

---

## 5. 验证摘要

### 5.1 docs 阶段

- 5 UI 组件物理布局明确 (per §1.1)
- zustand 5 channel 详细 (per §2)
- Playwright 8 E2E 分布 (per §3)
- OAuth2 + JWT 鉴权集成 (per §4)

### 5.2 frontend E2E 实装 (跨 session 续)

- 8 E2E 100% 跨 5 UI 组件 (per §3.1)
- 100% 跨 14 routes (per §1.2)
- 100% 跨 OAuth2 + RLS (per §4)
- 100% 跨 WebSocket 5 协议 (per §3.2 E2E #3)

### 5.3 跨 stage 集成 (per 守门 #1 v3 跨 stage 累积)

- 5 module 命名 / 9 SA 命名 / 5 域命名 / 5 UI 组件命名 100% 一致
- 跟 ARG.4 14 routes 集成 (per §1.2)
- 跟 ARG.2 5 协议 WebSocket 集成 (per §2.1 + §3.2)

---

## 6. 已知缺口 (per 守门 #11)

1. **frontend E2E 实装跨 session 续** (per 估 ~4-6M / 3-5 session, 落档后 v0.51+)
2. **5 UI 组件 SSR 路径** (per Next.js 14 App Router, 跨 session 续)
3. **WebSocket 5 协议断线重连** (per §3.2 E2E #3 实证, 跨 session 续)
4. **OAuth2 + JWT 跨域 SSO** (per v0.47 §14.12 IV 实证, 跨 session 续)
5. **ARG.5 跟 ARG.6 30 UT 端到端集成** (per WBS v0.18 §14.11 ARG.6, 跨 session 续)
6. **ARG.5 跟 ARG.10 frontend 集成** (per v0.51+ 续做项, 跨 doc 14 集成)

---

## 7. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.5 frontend/agent-relationships 5 UI 组件 + zustand 5 channel + Playwright 8 E2E + OAuth2 + JWT 鉴权 + 跨 session 续做边界 | 2026-09-10 01:36 JST 用户发令"继续" + brief v0.50 §14.11 ARG.2-11 docs 阶段派发 + 守门 #14 v3 Mavis 永久代签 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 43 次新事件触发仍允许 |
