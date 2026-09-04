# Brief: canvas-e2e-guard-001

**Agent**: mavis (root 直实装, 0 子代理 per 守门 #9 #3 实证 5/5 RPC 不可靠)
**Phase**: P3-F.1 frontend e2e 守门补齐
**Created**: 2026-09-04 19:18 JST
**Worktree**: `wt-canvas-e2e-guard-001` @ d9f65b3 (从 main 干净状态拉)
**Token 预算**: ~0.3M (守门 #4 / #19 估算)

---

## 0. 触发

2026-09-04 19:14 JST Ulysses 拍板"无限画布后续做哪一块? e2e 守门补齐 (推荐)"。
per 9/1 PHASE-MOBILE-PWA v0.4 模式 + 守门 #1 v3 (check+fmt+clippy 不替代 e2e) 硬约束。

## 1. 范围 (in-scope)

### 1.1 CanvasView.tsx 加 6 个 testid (5 行小改, 不破坏现有功能)

| testid | 位置 | 用途 |
|---|---|---|
| `canvas-svg` | `<svg>` 主画布 (line 404) | 元素拖拽 + 滚轮 zoom 锚点 |
| `canvas-toolbar` | toolbar `<div>` (line 374) | 工具按钮定位 |
| `canvas-minimap` | minimap `<svg>` (line 436) | 缩略图 + viewport rect 验证 |
| `canvas-frame-{frame.id}` | `{canvas.frames.map(renderFrame)}` (line 425) | 4 frame 验证 |
| `canvas-element-{element.id}` | `{elements.map(renderElement)}` (line 431) | 25 element 验证 + 双击跳详情 |

### 1.2 frontend/e2e/canvas-view.spec.ts 新建 (覆盖 6 项)

| # | 守门项 | 触发 |
|---|---|---|
| 1 | /canvas/canvas-001 路由 200 + 25 element 渲染 | 入口可用性 |
| 2 | **pan**: shift+drag 主 svg, viewport.x 变化 | 视口控制 |
| 3 | **zoom**: 滚轮 + 工具栏 zoom-in 双路径, 0.1x~4x 边界 | 缩放控制 |
| 4 | **fit-to-content**: 工具栏 Maximize2 按钮, minX/minY 重置 | 视野自适应 |
| 5 | **双击跳详情**: work_item_card 双击 → /work-item?selected=wi-001 | 25 module 联动 |
| 6 | **选区删除**: 选中 1 element + 工具栏 trash → store 0 该 id | 状态变更 |

### 1.3 minimap viewport rect 验证 (附 1 项, 跟 6 项绑定)

minimap svg 中 `<rect>` (line 438-446) 的 x/y/width/height 等于 viewport 转换后的可见范围。playwright 读 `getAttribute` 验。

## 2. 范围外 (out-of-scope, 后续子项)

- 协作 presence 真实接入 (被 P3-C Realtime 选型拍板阻塞)
- Share / Export PNG 按钮 handler (库选型未拍板)
- 真实数据接入 (被 P3-B 16 tool 真实接入阻塞, 当前 3/16 完成)
- minimap 拖拽 (CanvasView 现状只渲染, 不可拖)
- 滚轮以光标为中心 zoom 验证 (复杂 viewport 数学, 留 P2)

## 3. 已知缺口 (per 守门 #11 缺标比错标安全)

- 当前 canvas 数据全 mock, e2e 验证 mock 路径; 真实数据接入后 e2e 复用同一 spec
- 双击跳详情测 `work_item_card` 一种, 不覆盖 `worktree_node` / `agent_cursor` / `automation_node` (4 类同算法, 测 1 类足够)
- pan/zoom 用 mousedown/mousemove/mouseup + wheel 模拟, 不测 touch 事件 (mobile 走 cross-domain-5b + remote-mobile 现有 e2e)
- 工具栏 zoom-in/+0.1x 边界 = 4x 后按钮无效 (zoom 已 Math.min(4, ...)), 不验证 "拒绝 zoom 超过 4x" 异常路径

## 4. 守门硬约束 (per 守门 #1)

- `cd frontend && pnpm typecheck` 0 错
- `cd frontend && pnpm test` (vitest) 0 失败
- `cd frontend && pnpm test:e2e -- canvas-view` 6/6 全过
- 0 子代理调用 (per 守门 #9 #3)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权)
- 1 commit 落档 (per 守门 #15 docs 同步饱和, 等 e2e 实证事件再触发下一笔)

## 5. 落地清单

| # | 文件 | 内容 |
|---|---|---|
| 1 | `frontend/src/components/CanvasView.tsx` | +5 行 testid (5 处属性, 0 逻辑改动) |
| 2 | `frontend/e2e/canvas-view.spec.ts` | +6 case + 1 minimap 验证, ~150 行 |
| 3 | `docs/automation-design.md` §4 任务卡表追加 1 行 (守门 #21) |
| 4 | `scripts/automation/registry.md` 索引追加 1 行 (守门 #21) |

预估 1 commit, 4 文件, ~155 行新增。
