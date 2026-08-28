# Mock MSW Handler 完整化 + fixtures 目录 v0.1

> **状态**: 🟡 草案 v0.1
> **日期**: 2026-08-28
> **基点 commit**: `05f90de` (mock infra zod→TS type guards hotfix)
> **触发**: Phase E.2+ 用户选 m1-msw-fixtures (per 8/28 22:13 JST questionnaire), d4b3193 mocks 基础已落地, 3 P2/P3 缺口 (MSW/fixtures/i18n) 待补
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审 (per 8/27 19:39/21:59 JST 三次强化代签授权)

---

## 0. 目的

承接 d4b3193 + 05f90de mock infra 基础 (data/ + schemas/ + __tests__/), Phase E.2+ 任务: 闭环 mock 独立项目 — page 不直接 import MOCK_*, 改 `useEffect + fetch`, MSW 拦截返回 mock; 加 fixtures/ 目录 (read-only JSON 备份) 便于 review.

**目标**:
1. **MSW handler 完整化** — 6 endpoint 拦截 + 真实 schema 校验
2. **fixtures/ 目录** — read-only JSON 备份 (人工 review)
3. **page 改 useEffect + fetch** — 4 panel (agents/analytics/inbox/issues) 走 fetch
4. **MSW handler 自身 unit test** — 不 mount React, 直接 import handler 模拟 fetch
5. **回归测试** — 改 mock 1 处 → handler + fixture + page 全部自动同步

**非目标** (Phase F+):
- backend 真实 API 接入 (后端就绪时, 只删 MSW handler 启用 fetch)
- lib/store.ts (W5) 改造 (W5 scope)
- mock data i18n (zh-CN / en-US)

---

## 1. 现状分析

### 1.1 d4b3193 + 05f90de mock infra 现状

| 文件 | 状态 |
|---|---|
| `frontend/src/mocks/data/{agents,analytics,inbox,kanban}.ts` | ✅ 已 commit d4b3193 |
| `frontend/src/mocks/schemas/{agent,inbox,analytics}.ts` | ✅ TS type guards (zod→type guard per 05f90de hotfix) |
| `frontend/src/mocks/seed.ts` | ✅ mulberry32(seed=1) |
| `frontend/src/mocks/__tests__/{agents,inbox,analytics,kanban,snapshot}.test.ts` | ✅ 22 tests pass |
| `frontend/src/mocks/__tests__/__snapshots__/snapshot.test.ts.snap` | ✅ 5 snapshot 入库 |
| `frontend/src/mocks/handlers/` | ❌ **空** (per 设计书 §2.1, Phase E.2+ 补) |
| `frontend/src/mocks/fixtures/` | ❌ **空** (per 设计书 §2.1, Phase E.2+ 补) |

### 1.2 4 panel 当前状态 (直接 import MOCK_*)

| 文件 | 改前 | 改后 |
|---|---|---|
| `frontend/src/app/(app)/agents/page.tsx` | `import { MOCK_AGENTS }` 直接用 | `useEffect + fetch("/api/agents")` + MSW |
| `frontend/src/app/(app)/analytics/page.tsx` | `import { MOCK_KPI, COST_SERIES }` | `useEffect + fetch("/api/analytics/cost")` |
| `frontend/src/app/(app)/inbox/page.tsx` | `import { MOCK_NOTIFS }` | `useEffect + fetch("/api/notifications")` |
| `frontend/src/app/(app)/issues/page.tsx` | (inline 4 view mock) | 不改 (issue size 30K, 风险大于收益) |

### 1.3 缺失 (per 用户选 m1-msw-fixtures)

- ❌ `frontend/src/mocks/handlers/{agents,analytics,inbox}.ts` — MSW handler
- ❌ `frontend/src/mocks/handlers/index.ts` — handler re-export
- ❌ `frontend/src/mocks/fixtures/{agents,inbox,analytics}.json` — read-only 备份
- ❌ `frontend/src/mocks/server.ts` — MSW node server (vitest setup 用)
- ❌ `frontend/src/mocks/client.ts` — MSW browser worker (dev 用, Phase E.3+)
- ❌ `frontend/src/mocks/__tests__/handlers.test.ts` — MSW handler 自身测试
- ❌ `frontend/vitest.setup.ts` — 加 `setupServer` listen
- ❌ `frontend/package.json` — 加 `msw@^2.0` devDep
- ❌ 4 panel page — 改 useEffect+fetch (其中 issues 不改)

---

## 2. 设计

### 2.1 目录结构补全

```
frontend/src/mocks/
├── index.ts                    # ✅ 已有
├── seed.ts                     # ✅ 已有
├── data/                       # ✅ 已有
│   ├── agents.ts
│   ├── analytics.ts
│   ├── inbox.ts
│   ├── kanban.ts
│   └── index.ts
├── schemas/                    # ✅ 已有 (zod→type guard)
│   ├── agent.ts
│   ├── inbox.ts
│   └── analytics.ts
├── handlers/                   # ❌ 新建
│   ├── agents.ts               # MSW: GET /api/agents
│   ├── analytics.ts            # MSW: GET /api/analytics/cost
│   ├── inbox.ts                # MSW: GET /api/notifications
│   └── index.ts                # handlers re-export
├── server.ts                   # ❌ 新建 (MSW node server, vitest setup)
├── fixtures/                   # ❌ 新建 (read-only JSON 备份)
│   ├── agents.json
│   ├── inbox.json
│   ├── analytics-kpi.json
│   ├── analytics-cost-series.json
│   └── README.md               # 标注: "fixtures/ is read-only, code imports from data/"
└── __tests__/                  # ✅ 已有
    ├── agents.test.ts
    ├── inbox.test.ts
    ├── analytics.test.ts
    ├── kanban.test.ts
    ├── snapshot.test.ts
    ├── handlers.test.ts        # ❌ 新建 (MSW handler 自身测试, 不 mount React)
    └── __snapshots__/
        └── snapshot.test.ts.snap
```

### 2.2 MSW handler 示例

```typescript
// frontend/src/mocks/handlers/agents.ts
import { http, HttpResponse } from "msw";
import { MOCK_AGENTS } from "@/mocks/data";
import { isAgentRow } from "@/mocks/schemas/agent";

export const agentsHandlers = [
  http.get("/api/agents", () => {
    return HttpResponse.json(MOCK_AGENTS);
  }),
  // 真实接入时: 后端返回 schema 不一致 → response handler 用 isAgentRow 校验, fail 时 500
  http.post("/api/agents", async ({ request }) => {
    const body = await request.json();
    if (!isAgentRow(body)) {
      return HttpResponse.json({ error: "Invalid agent row" }, { status: 400 });
    }
    // P3 缺口: 真实持久化
    return HttpResponse.json(body, { status: 201 });
  }),
];
```

### 2.3 MSW server (vitest setup)

```typescript
// frontend/src/mocks/server.ts
import { setupServer } from "msw/node";
import { handlers } from "./handlers";

export const server = setupServer(...handlers);
```

```typescript
// frontend/vitest.setup.ts (新增)
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// MSW node server for vitest (per mock-msw-handlers §2.3)
import { server } from "./mocks/server";
beforeAll(() => server.listen({ onUnhandledRequest: "warn" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// next/navigation mock (per U3 baseline fix)
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn(), back: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => ({ get: vi.fn() }),
  redirect: vi.fn(),
}));
```

### 2.4 page 改 useEffect + fetch

**改前** (agents/page.tsx 直接 import):
```typescript
const MOCK_AGENTS: ReadonlyArray<AgentRow> = [...];  // 删
export default function AgentsPage() {
  return <div>{MOCK_AGENTS.map(a => <Row ... />)}</div>;
}
```

**改后** (fetch + MSW):
```typescript
"use client";
import { useEffect, useState } from "react";
import { MOCK_AGENTS_FALLBACK } from "@/mocks/data";  // SSR 兜底 (per Next.js 14 convention)
import type { AgentRow } from "@/mocks/schemas/agent";

export default function AgentsPage() {
  const [agents, setAgents] = useState<ReadonlyArray<AgentRow>>(MOCK_AGENTS_FALLBACK);
  useEffect(() => {
    fetch("/api/agents")
      .then((r) => r.json())
      .then((data) => setAgents(data))
      .catch(() => {/* keep fallback */});
  }, []);
  return <div>{agents.map(a => <Row ... />)}</div>;
}
```

注: `MOCK_AGENTS_FALLBACK` 改名以区分 (原本 MOCK_AGENTS), 是 SSR 时的兜底, MSW 在 client fetch 阶段接管。真实接入时删 fallback。

### 2.5 fixtures/ 目录

```json
// frontend/src/mocks/fixtures/agents.json
[
  { "id": "ag-001", "name": "Ulysses-CLI", "status": "active", "role": "root / architect", "last_active": "2 min ago" },
  ...
]
```

- fixtures/ 是 read-only JSON 备份
- `fixtures/README.md` 标注: "fixtures/ is for human eyeball only, code imports from data/"
- `data/*.ts` 改了, 重新 export JSON 同步 fixtures (Phase E.3+ 加 npm script)
- MSW handler 用 `MOCK_AGENTS` (从 data/), 不读 fixtures/

### 2.6 依赖增量

| 包 | 用途 | scope |
|---|---|---|
| `msw@^2.0` | MSW 拦截 fetch (server + browser) | **devDep** |
| (无 prod dep 增量) | — | — |

零 prod dep 增量 (per 守门 #2 "token-OLU 估算")。

### 2.7 测试策略

- **mock 自身 unit test 5 个** (已有) — 测 data/ 完整性
- **MSW handler test 1 个 (新)** — 测 handler 真实 fetch 行为, 不 mount React:
  ```typescript
  import { describe, it, expect } from "vitest";
  import { server } from "@/mocks/server";

  describe("MSW handlers", () => {
    it("GET /api/agents returns 5 rows", async () => {
      const res = await fetch("http://localhost/api/agents");
      const data = await res.json();
      expect(data).toHaveLength(5);
    });
    // ... 类似测 inbox / analytics
  });
  ```
- **4 panel page test 已更新** — 之前 5 panel test 5 tests 验证 mock 直接 import, 现在改测 fetch 异步:
  - 加 `await waitFor(() => screen.getByTestId("agent-row-ag-001"))`
  - 现有 5 panel test 必须 pass (per 守门)

---

## 3. 实施计划 (2 worker 并行)

### 3.1 Worker M2-A: MSW handler + server + vitest setup + 4 panel 改 fetch

**wt**: `D:\Star-wt-m2a` (branch `ui/m2a-msw-handlers`)

**任务** (per 8/27 19:39/21:59 JST 守门):
1. 加 `msw@^2.0` 到 devDep (用 `npm install --save-dev msw@^2.0`)
2. 创建 `frontend/src/mocks/handlers/{agents,analytics,inbox}.ts` — 6 endpoint
3. 创建 `frontend/src/mocks/handlers/index.ts` — re-export
4. 创建 `frontend/src/mocks/server.ts` — MSW node server
5. 改 `frontend/vitest.setup.ts` — 加 MSW setupServer
6. 改 4 panel page (agents/analytics/inbox/issues 不改) — useEffect + fetch
7. 加 1 个 MSW handler test
8. `npm run typecheck` (0 error)
9. `npx vitest run` (≥ 116 pass: 115 现有 + 1 新 MSW handler test)
10. `npm run build` (35 routes 不变)
11. 1 commit author = Ulysses per 8/27 19:39 JST

**scope 限制**:
- 不改 data/ (d4b3193 已落地, 业务逻辑不变)
- 不改 schemas/ (05f90de hotfix 已落地)
- 不动 W1-W5 旧 wt + U1-U5 wt + M1 wt
- 不 push origin (per R-05)

### 3.2 Worker M2-B: fixtures/ 目录 + sync script

**wt**: `D:\Star-wt-m2b` (branch `ui/m2b-mock-fixtures`)

**任务**:
1. 创建 `frontend/src/mocks/fixtures/{agents,inbox,analytics-kpi,analytics-cost-series}.json`
2. 创建 `frontend/src/mocks/fixtures/README.md` — 标注 read-only + sync 流程
3. 加 1 个 vitest test: 验证 `data/*.ts` 与 `fixtures/*.json` 一致 (改 data 但忘改 fixture 会 fail)
4. `npm run typecheck` (0 error)
5. `npx vitest run` (≥ 116 pass)
6. 1 commit author = Ulysses

**scope 限制**: 同 M2-A

### 3.3 串行合并 (Mavis 接手, 在 main 上 ff merge)

M2-A + M2-B 各自 wt 完成后:
1. merge `ui/m2a-msw-handlers` → main
2. merge `ui/m2b-mock-fixtures` → main
3. 写 PHASE-E.2-MOCK-MSW-FIXTURES-REPORT.md (7 段)
4. main 验证 typecheck / vitest / build 全绿

---

## 4. 已知缺口 (per 缺标比错标, 8/26 JST)

| # | 缺口 | 优先级 | 触发 |
|---|---|---|---|
| 1 | 4 panel page 用 `MOCK_*_FALLBACK` SSR 兜底, 真实 fetch 失败时 fallback 显示 (避免 UX 退化) | P3 | 当前决定保留 |
| 2 | MSW client worker (browser 端) — page 在 production build 时仍走 MSW (per msw v2 design) | P2 | Phase E.3+ |
| 3 | data → fixtures sync 自动化 (npm script) | P2 | Phase E.3+ |
| 4 | 4 panel page 改 fetch 后, SSR 时如何保留 mock (per Next.js 14 `use client` boundary) | P3 | 已用 `MOCK_*_FALLBACK` 解决 |
| 5 | MSW handler 与 backend 真实 schema 100% 一致 (zod schema 现行, Phase F+) | P2 | 后端就绪时 |
| 6 | handler test 只测 GET, 不测 POST/PATCH 失败 (response validation) | P3 | Phase E.3+ |
| 7 | issues/page.tsx (784 行) 不改 fetch (size + 风险) | — | 决策不变 |

---

## 5. 守门 (per AGENTS.md §4 12 项)

- ✅ **R-05 不 push** (8/27 11:09 JST): M2-A + M2-B commit author = Ulysses, 不 push origin
- ✅ **bc23d6c 保留** (8/27 11:09 JST)
- ✅ **5 域独立 Lead 不兼任** (8/21 JST): M2-A = handlers + fetch 域, M2-B = fixtures 域, 不重叠
- ✅ **AI 协作 token-OLU** (8/21 JST): M2-A ≤ 200K, M2-B ≤ 100K
- ✅ **环境变量安全** (8/27 11:06 JST hard ban)
- ✅ **PowerShell only**
- ✅ **0 unsafe** (TS 严模式)
- ✅ **不沿用 bc23d6c 叙事**
- ✅ **不 commit 散落子代理产出** (8/27 11:09 JST): Mavis 终审后统一入库
- ✅ **代签规则应用** (8/27 19:39/21:59 JST 三次强化)
- ✅ **缺标比错标安全** (8/26 JST): 7 项 P2/P3 缺口显式列 (§4)
- ✅ **AI 协作文档治理** (8/26 JST): M2-A + M2-B brief 写明 "无证据叙事=禁止" + "data 业务逻辑不变"

---

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008） | 2026-08-28 | 🟢 Active; MSW handler 完整化 + fixtures/ 目录 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; MSW setupServer + 1 handler test + 回归测试 闭环 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; msw@^2.0 devDep, 零 prod dep 增量 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; M2-A + M2-B brief 自审 pass, 5 worker 并行 (U1-U5) 不重叠 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; token-OLU M2-A ≤ 200K + M2-B ≤ 100K, 7 项 P2/P3 缺口显式 |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: MSW handler 完整化 + fixtures/ 目录 (per 8/28 22:13 JST questionnaire m1-msw-fixtures 选项) | d4b3193 mocks 基础 + 05f90de hotfix 后续 P2 缺口 |
