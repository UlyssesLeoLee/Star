# Frontend Mock 数据独立项目设计 v0.1

> **状态**: 🟡 草案 v0.1
> **日期**: 2026-08-28
> **触发**: Phase E UI/UX redesign 后, mock 数据散落 5+ panel page (per 8/28 21:30 JST 用户反馈"mock 应该是一个独立的项目, 便于回归测试")
> **基点 commit**: `7f3df1c` (Phase E 报告入库)
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审 (per 2026-08-28 19:39/21:59 JST 三次强化代签授权)

---

## 0. 目的

把 Phase E 5 worker (U1-U5) 写在 panel page 文件里的 mock 数据抽出成**独立可复用项目**, 满足:

1. **回归测试** — mock 自身 unit test 可独立跑 (snapshot + 一致性 + boundary)
2. **统一改点** — 改 mock 只改 1 个文件, 8+ panel 自动同步
3. **dev/test 分离** — MSW (Mock Service Worker) 拦截 fetch, page 代码不感知 mock 存在
4. **deterministic** — fixed seed 随机, CI 跑结果稳定
5. **类型安全** — schema 单一来源, 后端真实接入时只改 handler, mock shape 一致

---

## 1. 现状分析 (per 8/28 JST grep)

### 1.1 mock 散落位置 (5 worker inline mock)

| 文件 | mock 变量 | 行数 | 优先级 |
|---|---|---|---|
| `frontend/src/app/(app)/agents/page.tsx` | `MOCK_AGENTS` (5 row) | 95 | P0 抽 |
| `frontend/src/app/(app)/analytics/page.tsx` | `MOCK_KPI` + 7-day 折线 | 115 | P0 抽 |
| `frontend/src/app/(app)/inbox/page.tsx` | `MOCK_NOTIFS` (10 row) | 114 | P0 抽 |
| `frontend/src/app/(app)/issues/page.tsx` | inline work-item list (sprint 4 view 各自 mock) | 784 | P0 抽 |
| `frontend/src/app/projects/page.tsx` | 5 tab mock (overview/board/timeline/calendar/members) | 797 | P0 抽 |
| `frontend/src/app/(app)/settings/page.tsx` | 5 tab form initial value | 167 | P1 抽 |
| `frontend/src/components/board/KanbanBoard.tsx` | hard-coded columns | (已存在) | P2 |
| `frontend/src/lib/store.ts` (W5 zustand) | initial state mock | (W5) | P2 |

### 1.2 当前问题 (per 用户反馈)

- **8+ 文件散落 mock** — 改 mock 要 8 处改
- **不可复用** — 新 panel 写 mock 又重复抄
- **回归测试困难** — mock fixture 不能 import 复用, 测试只能 snapshot 整 page
- **不可独立跑** — mock 紧耦合 page 组件, 单元测试必须 mount React 树
- **dev/test 不分离** — 没 MSW, fetch 调用直达 backend (没 backend 时只能 mock 整个 fetch)
- **不可 deterministic** — 随机 mock 改 seed 困难

---

## 2. 设计 — 独立 mock 项目 `frontend/src/mocks/`

### 2.1 目录结构

```
frontend/src/mocks/
├── index.ts                    # setupMocks() 入口 (vitest 启动调用)
├── seed.ts                     # deterministic seed (fixed PRNG, 1 = 100)
├── data/
│   ├── agents.ts               # MOCK_AGENTS (5 row) + AgentRow type
│   ├── analytics.ts            # MOCK_KPI + 7-day cost series
│   ├── inbox.ts                # MOCK_NOTIFS (10 row) + MockNotif type
│   ├── issues.ts               # issues 4 view mock (Kanban/List/Tree/Sprint)
│   ├── projects.ts             # projects 5 tab mock (overview/board/timeline/calendar/members)
│   ├── settings.ts             # 5 tab form initial value
│   └── index.ts                # 统一 re-export
├── handlers/
│   ├── agents.ts               # MSW: GET /api/agents
│   ├── analytics.ts            # MSW: GET /api/analytics/cost
│   ├── inbox.ts                # MSW: GET /api/notifications
│   ├── issues.ts               # MSW: GET /api/work-items + PATCH /api/work-items/:id
│   ├── projects.ts             # MSW: GET /api/projects + PATCH
│   ├── settings.ts             # MSW: GET /api/settings + PUT
│   └── index.ts                # handlers re-export
├── schemas/
│   ├── agent.ts                # zod schema (与 backend 真实 schema 一致)
│   ├── inbox.ts
│   ├── issues.ts
│   ├── projects.ts
│   └── settings.ts
├── fixtures/
│   ├── agents.json             # 备份: 5 row (可读, import 不了)
│   ├── inbox.json
│   ├── ...
│   └── README.md               # "fixtures/ is for human eyeball only, code imports from data/"
└── __tests__/
    ├── agents.test.ts          # mock 自身 unit test (不 mount React)
    ├── analytics.test.ts       # 7-day series monotonic + sum 校验
    ├── inbox.test.ts           # 10 row + read/unread state transition
    ├── issues.test.ts          # 4 view mock 一致性 (Kanban columns 包含所有 work-item)
    ├── projects.test.ts        # 5 tab mock 一致性
    ├── settings.test.ts        # 5 tab form initial value
    └── snapshot.test.ts        # 整体 snapshot (改 mock 时强制 review)
```

### 2.2 入口 (vitest setup)

```typescript
// frontend/src/mocks/index.ts
import { setupServer } from "msw/node";
import { handlers } from "./handlers";

export const server = setupServer(...handlers);

export function setupMocks() {
  beforeAll(() => server.listen({ onUnhandledRequest: "warn" }));
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());
}

// frontend/vitest.setup.ts
import "vitest-canvas-mock";
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";
import { setupMocks } from "./mocks";

setupMocks();

// mock next/navigation (per U3 baseline fix)
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn(), back: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => ({ get: vi.fn() }),
  redirect: vi.fn(),
}));
```

### 2.3 使用方式 (panel page 改造)

**改前** (U4 inbox inline mock):
```typescript
const MOCK_NOTIFS: ReadonlyArray<MockNotif> = [
  { id: "1", type: "comment", title: "...", read: false, ... },
  // ... 9 more rows
];

export default function InboxPage() {
  // 直接读 MOCK_NOTIFS
}
```

**改后** (抽到 mocks/):
```typescript
// frontend/src/app/(app)/inbox/page.tsx
import { MOCK_NOTIFS } from "@/mocks/data";  // 单一来源

export default function InboxPage() {
  // 同 MOCK_NOTIFS, 不变
}
```

**完整 MSW 化** (后置, Phase E+):
```typescript
// frontend/src/app/(app)/inbox/page.tsx
export default function InboxPage() {
  const [notifs, setNotifs] = useState<Notification[]>([]);
  useEffect(() => {
    fetch("/api/notifications").then(r => r.json()).then(setNotifs);
  }, []);
  // MSW 拦截 /api/notifications, 返回 MOCK_NOTIFS
}
```

### 2.4 关键不变量

1. **deterministic** — 所有 mock 用 seeded PRNG (mulberry32, seed=1), CI 跑出相同结果
2. **schema 一致** — zod schema 与 backend 真实类型一致 (后端接入时只删 handler 文件, 真实 API 替代)
3. **snapshot stable** — `__tests__/snapshot.test.ts` 锁住 mock 输出, 改 mock 强制 review
4. **零外部依赖** — 不引 `msw` (在 devDep), 不引 `zod` (在 dep, 已存在)
5. **类型导出** — `import type { AgentRow } from "@/mocks/data/agents"` 与 mock 数据并列

### 2.5 依赖增量

| 包 | 用途 | scope |
|---|---|---|
| `msw@^2.0` | Mock Service Worker (handler 拦截 fetch) | devDep |
| `zod@^3.22` (已有) | schema 校验 | dep (已有) |
| `vitest-canvas-mock` (已有) | canvas 测试 | devDep (已有) |

零新增 prod dep。

---

## 3. 改造范围 (Phase E.2+ 任务)

### 3.1 必须改造 (P0, 5 panel + 1 project page)

| 文件 | 当前 | 改后 |
|---|---|---|
| `frontend/src/app/(app)/agents/page.tsx` | 95 行 (含 MOCK_AGENTS 5 row inline) | 删 inline mock, `import { MOCK_AGENTS } from "@/mocks/data"` |
| `frontend/src/app/(app)/analytics/page.tsx` | 115 行 (含 MOCK_KPI + 7-day series) | 同上 |
| `frontend/src/app/(app)/inbox/page.tsx` | 114 行 (含 MOCK_NOTIFS 10 row) | 同上 |
| `frontend/src/app/(app)/issues/page.tsx` | 784 行 (含 4 view mock) | 同上 |
| `frontend/src/app/(app)/settings/page.tsx` | 167 行 (含 5 tab form initial value) | 同上 |
| `frontend/src/app/projects/page.tsx` | 797 行 (含 5 tab mock) | 同上 |

### 3.2 应做 (P1, 1 component)

| 文件 | 当前 | 改后 |
|---|---|---|
| `frontend/src/components/board/KanbanBoard.tsx` | hard-coded "todo/in_progress/review/done" | `import { KANBAN_COLUMNS } from "@/mocks/data/issues"` |

### 3.3 不动 (P2, 后置)

| 文件 | 原因 |
|---|---|
| `frontend/src/lib/store.ts` (W5 zustand) | W5 scope, 不在 Phase E.2+ 范围 |

### 3.4 测试

- **mock 自身 6 个 unit test** (`__tests__/{agents,analytics,inbox,issues,projects,settings}.test.ts`)
  - **不** mount React 树, 直接 import mock 数据
  - 校验: row count, schema valid, deterministic (固定 seed 输出), key invariant (Kanban columns 包含所有 work-item, 7-day series sum 一致)
- **1 个 snapshot test** (`__tests__/snapshot.test.ts`)
  - 整体 snapshot 锁住 mock 输出
  - 改 mock 时 review snapshot diff
- **现有 panel page test 5 个** (U1 16 + U2 14 + U3 8 + U4 5 + U5 13 = 56 新 + 34 baseline = 90 tests) 继续 pass, **import path 改 @/mocks/data, 行为不变**

---

## 4. 实施计划 (1-2 worker)

### 4.1 Worker M1: mock data 抽取 + 6 panel page import 改造

**wt**: `D:\Star-wt-m1` (branch `ui/m1-mock-data-isolation`)

**任务** (per 8/28 JST 21:30 用户授权 + 8/21 JST 5 域独立 Lead 不兼任):

1. 创建 `frontend/src/mocks/` 目录结构 (见 §2.1)
2. 抽 6 panel mock 数据到 `mocks/data/{agents,analytics,inbox,issues,projects,settings}.ts`
3. 抽 KANBAN_COLUMNS 到 `mocks/data/issues.ts`
4. 加 zod schema (6 文件) 到 `mocks/schemas/`
5. 加 MSW handlers (6 文件) 到 `mocks/handlers/`
6. 改 7 page 文件 import (`MOCK_*` 来源改 `@/mocks/data`)
7. 加 mock 自身 unit test 6 个 (不 mount React)
8. 加 snapshot test 1 个
9. `vitest.setup.ts` 加 `setupMocks()` 调用
10. 加 `msw@^2.0` 到 devDep, `npm install` 验证
11. `npm run typecheck` (0 error)
12. `npx vitest run` (≥ 97 tests pass: 90 现有 + 6 mock 自身 + 1 snapshot)
13. `npm run build` (35 routes 不变)
14. 1 commit, author = Ulysses per 8/27 19:39 JST

**scope 限制** (per AGENTS.md §1.2 不可代签底线第 4 项 + 缺标比错标):
- **不**改 panel 业务逻辑 (只是 import 路径换)
- **不**改 MSW handler 真实行为 (只是搬位置)
- **不**改任何 wt (W1-W5 旧 wt + U1-U5 wt)
- **不**改 `lib/store.ts` (W5 scope, P2 不动)
- **不**改 backend 代码
- **不** push origin (per R-05)

**完成定义**:
- 1 commit 在 `ui/m1-mock-data-isolation` branch
- typecheck 0 error
- vitest ≥ 97 pass (90 现有 + 7 新)
- build 35 routes 不变
- 6 panel page 文件 modify, import 路径正确
- `frontend/src/mocks/` 目录结构按 §2.1
- 6 mock unit test + 1 snapshot test
- 已知缺口 (per 缺标比错标) 显式列

### 4.2 Worker M2 (可选, 不一定需要): MSW handler 完整化

如果 M1 完成顺利, M2 可加 6 MSW handler 的真实 fetch 模拟 (panel 改 useEffect + fetch, 而非直接 import mock), 进一步分离 dev/test。但 **这一步 Phase E+ 即可, M1 已经满足用户核心要求 "mock 独立项目 + 回归测试"**。

---

## 5. 已知缺口 (per 缺标比错标)

| # | 缺口 | 优先级 | 触发 |
|---|---|---|---|
| 1 | MSW handler 完整化 (panel 改 fetch 而非直接 import) | P2 | Phase E.3+ |
| 2 | MSW handler 与真实 backend schema 100% 一致 (zod schema 现行) | P2 | Phase F+ (后端就绪) |
| 3 | deterministic seed 暴露给 panel page (让 page 可控 mock 数据) | P3 | 后置 |
| 4 | `lib/store.ts` (W5) 改造 | P3 | 后置 |
| 5 | mock data i18n (zh-CN / en-US) | P3 | 后置 |
| 6 | fixtures/ 目录人工对照 (read-only) | — | 决策不变 |

---

## 6. 守门 (per AGENTS.md §4 12 项)

- ✅ **R-05 不 push** (8/27 11:09 JST)
- ✅ **bc23d6c 保留** (8/27 11:09 JST)
- ✅ **5 域独立 Lead 不兼任** (8/21 JST): M1 = mock infra 域, 不与原 U1-U5 重叠
- ✅ **AI 协作 token-OLU** (8/21 JST): M1 ≤ 200K tokens (6 mock 文件 + 6 schema + 6 handler + 7 panel modify + 7 test + commit + typecheck + vitest + build)
- ✅ **环境变量安全** (8/27 11:06 JST hard ban): M1 无 env var 操作
- ✅ **PowerShell only** (持续)
- ✅ **0 unsafe** (TS 严模式)
- ✅ **不沿用 bc23d6c 叙事** (8/27 11:09 JST)
- ✅ **不 commit 散落子代理产出** (8/27 11:09 JST): Mavis 终审后统一入库
- ✅ **代签规则应用** (8/27 19:39/21:59 JST): M1 commit author = Ulysses
- ✅ **缺标比错标安全** (8/26 JST): 6 项缺口显式列 (§5)
- ✅ **AI 协作文档治理** (8/26 JST): M1 brief 写明 "无证据叙事=禁止" + "panel 业务逻辑不变, 只改 import 路径"

---

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008） | 2026-08-28 | 🟢 Active; mock 数据抽独立项目 + 6 panel import 改造 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签 (per 2026-08-28 19:39/21:59 JST 三次强化); mock 自身 7 unit test 独立跑, 回归测试 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; MSW 拦截 + zod schema + dev/test 分离 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; M1 brief + 设计书 v0.1 自审 pass |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签; token-OLU ≤ 200K, 6 项 P2/P3 缺口显式 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: mock 独立项目设计 (mocks/ 目录 + MSW + zod + 7 unit test + snapshot) | 2026-08-28 21:30 JST 用户反馈"mock 应该是一个独立的项目, 便于回归测试" |
