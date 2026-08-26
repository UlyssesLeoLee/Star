# Star 平台《Frontend Canvas Design — Miro 无限画布集成》

> **文档版本**: v0.1 (2026-08-26)
> **修订历史**:
>
> | 版本 | 日期 | 变更 | 审批者 |
> |---|---|---|---|
> | v0.1 | 2026-08-26 | 初始版本(吸收 Miro 无限画布模式,作为 collaboration 域主入口) | — |
>
> **上游 frontend-design**: `D:\Star\docs\frontend-design.md` v0.1
> **上游 frontend-internal-01-04**: 4 份架构/组件/数据/交互详细设计
> **参考对象**: Miro(无限画布 / Frame / Sticky Note / Connector / Presence)
> **文档定位**: 在 25 module 框架下,**改造 collaboration 域,加入无限画布主入口**;不破坏 25 module 1:1,只调整 frontend 路由 + 增补实体类型

---

## 0. 文档说明

### 0.1 目标

在 Star 平台已有 25 module + 6 状态机 + 4 份 frontend-internal 详细设计的基础上,**吸纳 Miro 无限画布模式**,让用户能从"工作清单视角"无缝切换到"画布视角",并与现有 25 module 深度联动。

### 0.2 核心约束

- **不破坏 25 module 1:1**:不新增 25 module 之外的独立 module;canvas 是 collaboration 域的强化
- **复用 StateMachineDiagram 算法**:CanvasView 与 SmView 共用 5×4 grid + bezier 边 + 颜色体系
- **复用 StatusPill 60+ 色码**:canvas 上的 status node 用 StatusPill 同一色码
- **25 module 1:1 路由不变**:canvas 是 collaboration 域的 sub-route(`/canvas/[id]` 是 `/collaboration` 的详情页)
- **其他界面与画布界面有效联动**:
  1. **正向**:WorkItem 列表 → 拖到画布变 element
  2. **反向**:画布 element → 跳 WorkItem 详情
  3. **状态同步**:Worktree status 变化 → 画布对应 node 颜色实时变
  4. **关系可视化**:Relation 域的 5 种关系 → 画布 connector

### 0.3 引用关系

| 引用本文 | 位置 |
|---|---|
| frontend-design §1.3 25 module 映射 | §1.1(协作域改造) |
| frontend-internal-02 §3 StateMachineDiagram 算法 | §3.1 CanvasView 复用 |
| frontend-internal-02 §2.1 StatusPill | §3.2 Status node 色码 |
| frontend-internal-03 §1 collaboration 字段 | §2 实体类型扩展 |
| frontend-internal-04 §1.1 9 快捷键 | §4.5 新增快捷键 |

---

## 1. 25 module 框架下的 canvas 定位

### 1.1 模块归属:collaboration 域强化

**前置状态**(`domain-collaboration`):
- 实体:PresenceCursor + Whiteboard
- 现状:`/collaboration` 页面是 PresenceCanvas(实时 cursor)+ WhiteboardGrid(2 张共享白板缩略图)
- **缺点**:Whiteboard 实体是独立页(快照式),没有"无限画布"概念

**改造后**:
- 实体:PresenceCursor + Whiteboard → **Whiteboard 升级为 Canvas + CanvasElement + CanvasConnector**
- `/collaboration` 入口:Canvas 列表(2 N 缩略图,可新建/搜索)
- `/canvas/[id]` 详情:无限画布主战场
- 保持 PresenceCursor 概念,在画布详情页内实时显示

### 1.2 不破坏 25 module 的 3 种方案对比

| 方案 | 后端 | 前端 | 优劣 |
|---|---|---|---|
| A 新增第 26 个 module `domain-canvas` | 增 1 个 crate | 增 1 个 route | ❌ 破坏 1:1 |
| **B 强化 collaboration 域(本设计采用)** | 0 crate 改动,改 collaboration 内部 | 改 `/collaboration` + 新 `/canvas/[id]` | ✅ 不破坏 + 复用 PresenceCursor |
| C 把 canvas 当成前端纯前端能力(无 backend) | 0 改动 | 0 后端支撑 | ❌ 与 25 module 数据无联动 |

**采用 B 方案**。backend `domain-collaboration` 0 crate 改动(只改内部 entity 设计),frontend 新增 1 个 route 详情页 + 改入口页。

### 1.3 25 module 联动矩阵

| 源 module | 画布表现 | 反向链接 |
|---|---|---|
| **work-item** | 拖 WorkItem → 画布变 element(sticky / card) | 点 element → 跳 `/work-item?selected=wi-001` |
| **worktree** | 自动在画布生成 node,显示 status 颜色 | 点 node → 跳 `/worktree?selected=wt-001` |
| **agent** | 画布上画 cursor(已有 PresenceCursor 升级) | 点 cursor → 跳 `/agent?selected=ag-001` |
| **relation** | 5 种关系渲染 connector | 点 connector → 跳 `/relation` 详情 |
| **comment** | element 上挂载 comment | 点 comment → 跳 comment 详情 |
| **automation** | 画布事件触发 rule | rule trigger_kind 加 `canvas_event` |
| **audit** | 画布操作写 audit(action: canvas.element.move) | — |
| **search** | 搜 work-item 跳 canvas 对应 element | — |
| **notification** | 画布"被 @ 提及时"通知 | — |

---

## 2. 实体类型扩展(在 collaboration 内,backend 0 改动)

### 2.1 Canvas(替代原 Whiteboard 实体)

```ts
// 替代原 Whiteboard
export interface Canvas {
  id: Uuid;
  tenant_id: Uuid;             // 13 类必带
  workspace_id: Uuid;
  title: string;
  /** 关联的 work-item / worktree / project(可空) */
  ref_kind?: "work_item" | "worktree" | "project" | "free";
  ref_id?: Uuid;
  /** 视口状态(用户进入画布时的初始 pan/zoom) */
  viewport: { x: number; y: number; zoom: number };
  /** Frame 列表(画布分区,可作 slide) */
  frames: CanvasFrame[];
  /** 创建者 / 协作者 */
  creator_id: Uuid;
  collaborator_ids: Uuid[];
  created_at: Iso8601;
  updated_at: Iso8601;
  snapshot_url?: string;       // PNG 导出(Miro 同款)
}

export interface CanvasFrame {
  id: Uuid;
  canvas_id: Uuid;
  title: string;
  /** 画布上的矩形区域 */
  x: number; y: number;
  width: number; height: number;
  /** Frame 内 element 子集(空 = 全部) */
  element_ids: Uuid[];
  /** 是否作为 presentation slide */
  is_slide: boolean;
  /** Frame 顺序(presentation 用) */
  order: number;
}
```

### 2.2 CanvasElement(画布上所有可视对象)

```ts
export type CanvasElementKind =
  | "sticky_note"     // 便利贴
  | "text"            // 自由文本
  | "shape"           // 矩形/圆/三角
  | "image"           // 图片
  | "embed"           // iframe 嵌入(YouTube/Figma)
  | "work_item_card"  // 关联 WorkItem 的卡片(联动主战场)
  | "worktree_node"   // 关联 Worktree 的节点(状态色码)
  | "agent_cursor"    // 关联 Agent 的 cursor(只读镜像)
  | "automation_node" // 关联 Automation rule 的节点
  | "comment_pin";    // 关联 comment 的图钉

export interface CanvasElement {
  id: Uuid;
  canvas_id: Uuid;
  kind: CanvasElementKind;
  /** 几何位置(无限画布坐标系) */
  x: number; y: number;
  width: number; height: number;
  rotation: number;            // 旋转(Miro 支持)
  z_index: number;            // 图层
  /** 元素内容(根据 kind 不同) */
  content: {
    text?: string;             // sticky_note / text
    color?: string;            // 颜色 hex(默认 6 种 palette)
    image_url?: string;        // image
    embed_url?: string;        // embed
    work_item_id?: Uuid;       // work_item_card
    worktree_id?: Uuid;        // worktree_node
    agent_session_id?: Uuid;   // agent_cursor
    automation_id?: Uuid;      // automation_node
    comment_id?: Uuid;         // comment_pin
  };
  /** 锁定/隐藏(presentation 模式用) */
  locked: boolean;
  hidden: boolean;
  created_by: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
}
```

### 2.3 CanvasConnector(画布上元素之间的连线)

```ts
export type CanvasConnectorKind =
  | "work_item_relation"     // 来自 Relation 域(blocks/duplicates/relates_to/...)
  | "agent_handoff"          // Agent 之间的 handoff
  | "free"                   // 用户手画
  | "dependency";            // 强依赖(箭头)

export interface CanvasConnector {
  id: Uuid;
  canvas_id: Uuid;
  kind: CanvasConnectorKind;
  from_element_id: Uuid;
  to_element_id: Uuid;
  /** 路由算法:straight / curved / orthogonal(Miro 3 选 1) */
  routing: "straight" | "curved" | "orthogonal";
  /** 起点箭头 / 终点箭头 */
  arrow_start: boolean;
  arrow_end: boolean;
  /** 颜色 + 粗细(可读性) */
  color: string;
  width: number;             // px
  label?: string;            // "blocks" / "handoff" / "h1"
  /** 关联的 Relation ID(如果来自 relation 域) */
  relation_id?: Uuid;
}
```

### 2.4 CanvasViewport(实时共享)

```ts
export interface CanvasViewport {
  canvas_id: Uuid;
  user_id: Uuid;
  /** 当前 pan / zoom */
  x: number; y: number; zoom: number;
  /** 选中的 element 列表(可多选) */
  selected_element_ids: Uuid[];
  updated_at: Iso8601;
}
```

---

## 3. CanvasView 组件规范(Organism)

### 3.1 Props interface

```ts
interface CanvasViewProps {
  canvas: Canvas;
  elements: CanvasElement[];
  connectors: CanvasConnector[];
  /** 当前用户的 viewport(可受控) */
  viewport: { x: number; y: number; zoom: number };
  onViewportChange: (v: { x: number; y: number; zoom: number }) => void;
  /** 选中 element 回调(可多选) */
  onElementSelect: (ids: Uuid[]) => void;
  /** element 拖动 / 缩放 回调 */
  onElementMove: (id: Uuid, x: number, y: number) => void;
  onElementResize: (id: Uuid, w: number, h: number) => void;
  /** 只读模式(presentation) */
  readOnly?: boolean;
  /** 联动高亮(从其他 page 跳过来时) */
  highlightElementId?: Uuid;
}
```

### 3.2 视口控制(无限画布核心)

```
世界坐标 (World)   ← 画布无限延伸,所有 element 真实位置
  ↕ zoom + pan
屏幕坐标 (Screen)  ← 浏览器视口
```

**公式**:
```
screen.x = (world.x - viewport.x) * viewport.zoom
screen.y = (world.y - viewport.y) * viewport.zoom
world.x = screen.x / zoom + viewport.x
world.y = screen.y / zoom + viewport.y
```

**最小画布尺寸**:100,000 × 100,000 px(超过则 panic 给 warning)
**缩放范围**:0.1x ~ 4x

### 3.3 平移与缩放

| 输入 | 行为 |
|---|---|
| 鼠标中键拖 / 双指拖 | pan(viewport.x/y 跟随) |
| Ctrl + 滚轮 / 双指捏 | zoom(以光标为中心) |
| Space + 拖 | 临时 pan(Miro 同款) |
| Fit to content 按钮 | 自动计算所有 element bounding box + 居中 |
| Zoom 100% 按钮 | zoom = 1,viewport = 0,0 |
| Minimap(右下角) | 显示当前 viewport 在世界中的位置 |

### 3.4 元素渲染(7 种 kind)

| kind | 渲染 | 颜色 palette |
|---|---|---|
| `sticky_note` | 矩形(120×120 默认),内文 14px | 黄/粉/蓝/绿/紫 5 色 |
| `text` | 自由文本框 | ink-dim |
| `shape` | 矩形 / 圆 / 三角(可切换) | line |
| `image` | `<img>` | — |
| `embed` | iframe(16:9) | — |
| **`work_item_card`** | WorkItem 缩略卡(显示 key/title/status/priority) | **StatusPill 色码同步 status** |
| **`worktree_node`** | 圆角矩形,显示 branch + status 颜色 | **StatusPill 色码同步 status** |
| **`agent_cursor`** | 圆点 + 名字 | accent |
| **`automation_node`** | 六边形,显示 rule 名称 | warn |
| **`comment_pin`** | 图钉,带 comment 数量 badge | info |

**关键约束**:`work_item_card` / `worktree_node` 的状态色码**必须走 StatusPill 60+ 颜色**(frontend-internal-02 §2.1,ADR-FE-013)— 这是联动一致性的基础。

### 3.5 连接线渲染

复用 StateMachineDiagram 的 bezier 算法(frontend-internal-02 §3.4):

```
from (fx, fy) → to (tx, ty)
C1 = (fx + dx*0.25, fy + dy*0.1)
C2 = (tx - dx*0.25, ty - dy*0.1)
```

3 种 routing:
- **straight**:直线(用于 free / dependency)
- **curved**:bezier(用于 work_item_relation,美观)
- **orthogonal**:直角(用于 system architecture,工程感)

### 3.6 Frame 渲染

- 矩形边框 + 标题(顶部)
- 半透明背景(`bg-soft/30`)
- 拖动 Frame = 拖动内部所有 element(同时移动)
- 缩放 Frame = 触发内部 element 缩放(本设计 V1 候选,先不支持)

---

## 4. 联动设计(与 25 module 双向同步)

### 4.1 联动机制:3 种模式

#### 模式 A:实时订阅(Realtime 通道)

- `star.collaboration.canvas.element.*`(新增 NATS Subject)
- BFF 推送 element 增删改
- 所有浏览同一 canvas 的用户实时看到

#### 模式 B:Polling 兜底(无 WS 时)

- 每 30s GET `/v1/collaboration/canvases/[id]/elements`
- 简单 fallback,30s 延迟可接受

#### 模式 C:从其他 page 跳到 canvas

- URL `/canvas/canvas-001?highlight=element-001` → CanvasView 自动 pan/zoom 到该 element

### 4.2 联动 1:WorkItem → Canvas Element(正向)

**触发**:WorkItem 详情页"添加到画布"按钮
**流程**:
1. WorkItem page 调 `useStore.addCanvasElement({ kind: "work_item_card", work_item_id: "wi-001", x: 0, y: 0 })`
2. store 增 1 个 element
3. 跳 `/canvas/canvas-001?highlight=element-001`
4. CanvasView 收到 `highlightElementId` prop,自动 pan/zoom + 高亮(蓝边框 2s)

### 4.3 联动 2:Canvas Element → WorkItem 详情(反向)

**触发**:Canvas 双击 `work_item_card` element
**流程**:
1. CanvasView `onDblClick` → 调 `router.push('/work-item?selected=' + element.work_item_id)`
2. WorkItem page 读 `?selected=...` URL param,自动选中 + 滚动到该 row

### 4.4 联动 3:Worktree status → Canvas 节点色码同步(实时)

**机制**:`worktree_node` element 的 status 字段不存 CanvasElement,而是**每次渲染时从 useStore 读 worktree.status**
- 渲染时: `const wt = useStore.getState().worktrees.find(w => w.id === element.worktree_id)`
- 颜色:`<StatusPill value={wt.status}>` 同一色码
- 状态变化:`useStore` 自动触发 re-render(已是 zustand 标准行为)

### 4.5 联动 4:Relation → Canvas Connector(批量导入)

**触发**:`/relation` 页"导入到画布"按钮
**流程**:
1. Relation 列表 50 条 relation
2. 创建 1 个新 canvas
3. 为每条 relation 创建 2 个 element(`work_item_card` × 2)+ 1 个 connector
4. 元素自动布局:grid 排列,connector 走 curved

### 4.6 联动 5:Agent cursor(已有 PresenceCursor 升级)

- `/collaboration` 已有 PresenceCursor(x/y/selection)
- 升级:Cursor 锚定在 canvas 上的特定 element
- 显示 agent 名字 + 当前正在操作什么

### 4.7 联动 6:Automation Rule 触发

- Automation 域 trigger_kind 加 `canvas_event`(V1 候选)
- canvas 上的"拖到指定 zone" / "创建 element" 触发 rule

### 4.8 联动 7:Search 跳 Canvas

- `/search` 搜 "wi-001"
- 结果列表点击 → `/canvas/canvas-001?highlight=element-001`
- CanvasView 自动定位到该 element

### 4.9 联动 8:URL param 透传(ADR-FE-010)

```
/canvas/canvas-001?highlight=element-001&zoom=1.5
```

- `highlight`: 跳转后高亮该 element
- `zoom`: 跳转后 zoom 级别(0.1 ~ 4)
- 不存 store,只 URL 读

---

## 5. 交互规范(继承 frontend-internal-04 §1.1 + 新增)

### 5.1 新增快捷键(在 INT-04 §1.1 9 基础上 +5)

| 快捷键 | 行为 | MVP |
|---|---|---|
| `v` | 切换到 select tool | V0.1 |
| `h` | 切换到 pan tool | V0.1 |
| `t` | 切换到 text tool(新增,WorkItem t 冲突?已用,改 `T`) | V0.1 |
| `T`(大写) | 切换到 text tool | V0.1 |
| `f` | frame 模式 | V0.1 |
| `n` | 新建 sticky note | V0.1 |
| `Delete` | 删除选中 element | V0.1 |
| `Cmd+D` | 复制选中 element | V0.1 |
| `Cmd+Z` | undo | V1 候选 |
| `Cmd+Shift+Z` | redo | V1 候选 |
| `0` | 缩放到 100% | V0.1 |
| `1` | fit to content | V0.1 |

### 5.2 鼠标交互

| 输入 | 行为 |
|---|---|
| 单击 element | 选中(高亮边框) |
| Shift+ 单击 | 多选 |
| 双击 element | 打开关联详情(联动 2) |
| 拖动 element | 移动(x, y 更新到 store) |
| 拖动 element 角 | 缩放(width, height) |
| 拖动空白 | pan viewport |
| 双指捏 | zoom |
| 滚轮 | 上下 pan |

### 5.3 多选操作

- 选中 N 个 element(Shift+ 单击 或 框选)
- 右键菜单:删除 / 复制 / 编组(V1 候选)
- 拖动多选 = 整体平移

### 5.4 Frame 操作

- F 键:进入 frame 模式,拖出矩形创建 frame
- 拖入 element 到 frame = 加入该 frame
- 拖出 frame = 离开该 frame
- Frame 不可重叠(自动避让,V1 候选)

---

## 6. 工具栏规范(继承 frontend-internal-01 §2.1 4 级组件 + 新增)

### 6.1 工具栏(顶部固定)

```tsx
<CanvasToolbar>
  <ToolButton icon="MousePointer" tool="select" />
  <ToolButton icon="Hand" tool="pan" />
  <ToolButton icon="StickyNote" tool="sticky_note" />
  <ToolButton icon="Type" tool="text" />
  <ToolButton icon="Square" tool="shape" />
  <ToolButton icon="Frame" tool="frame" />
  <Divider />
  <WorkItemSearchButton />  // 拖入 work_item_card
  <WorktreeSearchButton />  // 拖入 worktree_node
  <AutomationSearchButton /> // 拖入 automation_node
  <Divider />
  <ZoomControls />
  <UndoRedo />  // V1 候选
  <PresentationMode />  // V1 候选
</CanvasToolbar>
```

### 6.2 侧边栏(右侧可折叠)

- **Layers**(图层):所有 element / frame 列表
- **Comments**:所有 comment_pin 列表
- **History**:canvas 操作历史(V1 候选)

### 6.3 状态栏(底部固定)

- 当前 zoom %
- 当前 cursor 位置(x, y)
- 在线协作人数(Presence 实时)
- 最后保存时间

---

## 7. 实施分解

### 7.1 阶段 1(MVP,V0.1)

- Canvas / CanvasElement / CanvasConnector TS type(types/ids.ts)
- CanvasView Organism(基础 pan/zoom/select)
- 7 种 element 渲染(简化版)
- 5 种联动(WorkItem / Worktree / Relation / Comment / Search URL)
- 入口页改 `/collaboration`(Canvas 列表)
- 详情页 `/canvas/[id]`

### 7.2 阶段 2(V1 候选)

- Realtime WS 通道(联动 1-2)
- Frame 操作完整支持
- Auto-arrange(智能布局)
- 多选 + 编组 + 撤销/重做
- 嵌入(YouTube / Figma / PDF)

### 7.3 阶段 3(V2 候选)

- Template 库(2.5k+ — 不会做全,做 5-10 个)
- 投票 / 计时器 / 演示模式
- 高级 diagram 类型(BPMN / UML)
- Mind map auto-arrange

---

## 8. 路由调整(frontend 路由层)

### 8.1 新增路由

```
src/app/
├── collaboration/
│   └── page.tsx          # 改:Canvas 列表(原 WhiteboardGrid 改 CanvasCard 网格)
└── canvas/
    └── [id]/
        └── page.tsx      # 新增:无限画布详情页
```

### 8.2 路由跳转(联动 5)

- `/work-item?selected=wi-001` → 点"添加到画布"按钮 → 跳 `/canvas/canvas-001?highlight=element-001`
- `/worktree?selected=wt-001` → 同上
- `/search?q=wi-001` → 点结果 → 同上

### 8.3 Sidebar 调整

- `collaboration` 路由保留(Canvas 列表入口)
- 新增顶级菜单项?否 — canvas 是 collaboration 详情,不放顶级

---

## 9. 已知缺口(V1/V2 候选)

| 编号 | 描述 | 优先级 |
|---|---|---|
| CANVAS-OI-01 | (V1) Realtime WS 通道(star.collaboration.canvas.*) | P1 |
| CANVAS-OI-02 | (V1) 撤销/重做(Cmd+Z / Cmd+Shift+Z) | P2 |
| CANVAS-OI-03 | (V1) Auto-arrange 智能布局 | P2 |
| CANVAS-OI-04 | (V1) 嵌入 widget(YouTube / Figma / PDF) | P2 |
| CANVAS-OI-05 | (V1) 演示模式 + Frame as slide | P2 |
| CANVAS-OI-06 | (V1) Mind map auto-arrange | P2 |
| CANVAS-OI-07 | (V1) 多选 + 编组 + 复制/粘贴 | P1 |
| CANVAS-OI-08 | (V1) 模板库(自建 5-10 个) | P3 |
| CANVAS-OI-09 | (V1) Performance > 1000 element | P2 |
| CANVAS-OI-10 | (V1) 投票 / 计时器 / 便利贴聚类 | P3 |

---

## 10. ADR-CANVAS-001~005(本设计新增)

### ADR-CANVAS-001:Canvas 是 collaboration 域强化,非新 module

- **背景**:25 module 1:1 原则下不能新增第 26 module
- **决策**:Canvas 实体 + CanvasElement + CanvasConnector 都归 collaboration 域
- **后果**:
  - 25 module 不变 ✓
  - 复用 PresenceCursor 实时协作基础设施
  - 复用 domain-collaboration 现有 Port / Repository

### ADR-CANVAS-002:CanvasView 复用 StateMachineDiagram 算法

- **决策**:5×4 grid layout + bezier 边 + 颜色 token 与 SmView 共用
- **后果**:
  - 维护成本低(1 套算法)
  - 但 canvas 是"无限"坐标,不是"5 列"网格(grid 算法只用于 minimap)

### ADR-CANVAS-003:Worktree 节点状态色码必须走 StatusPill

- **决策**:Canvas 节点颜色 = StatusPill(value=wt.status) 同一色码
- **后果**:
  - 视觉一致(列表页 + 画布页同步)
  - 联动 3(状态实时同步)成为 trivial 实现(用 store 派生)

### ADR-CANVAS-004:URL param 透传 highlight(继承 ADR-FE-010)

- **决策**:`/canvas/[id]?highlight=element-001` → CanvasView 自动 pan/zoom
- **后果**:
  - 跨 page 跳转可分享
  - 不存 store,只 URL 读

### ADR-CANVAS-005:Realtime 通道走 BFF fan-out(继承 ADR-FE-020)

- **决策**:浏览器不直连 NATS,经 BFF
- **后果**:
  - NATS 安全
  - 单 BFF 降采样 cursor 10Hz → 2Hz

---

## 11. 验证清单(本设计自检)

| # | 验证项 | 验证方法 | 状态 |
|---|---|---|---|
| 1 | 25 module 1:1 不破坏 | `ls frontend/src/app -Directory | wc -l` → 25 | (实施后验) |
| 2 | 7 种 element 渲染 | CanvasView render test | (实施后验) |
| 3 | 5 种联动双向通 | 手动 + URL param 验证 | (实施后验) |
| 4 | StatusPill 色码 100% 复用 | 6 种 status node 颜色与列表页一致 | (实施后验) |
| 5 | Bezier 算法复用 | CanvasView 边 = SmView 边 | (实施后验) |
| 6 | 修订历史"审批者"= "—" | head -10 | ✓ |
| 7 | 文档长度 20-30 KB | wc -c | ✓ (约 22 KB) |

---

> **下游交接**:
> 1. frontend/src/types/ids.ts 加 Canvas / CanvasElement / CanvasConnector 3 个 interface
> 2. frontend/src/lib/seed.ts 加 2-3 个 mock canvas + 30+ element
> 3. frontend/src/components/CanvasView.tsx 新建(Organism)
> 4. frontend/src/app/collaboration/page.tsx 改为 Canvas 列表
> 5. frontend/src/app/canvas/[id]/page.tsx 新建
> 6. frontend/src/lib/store.ts 加 3 个 mutator(addCanvasElement / moveCanvasElement / deleteCanvasElement)
> 7. frontend-internal-01 §2.4 25 module × 4 模式矩阵的 collaboration 行更新(改为 Canvas 入口)
> 8. frontend-internal-02 §2.1 4 级组件树加 CanvasView(V1 候选 → V0.1 实际)
> 9. frontend-internal-04 §1.1 11 行快捷键表 +5(共 16)
