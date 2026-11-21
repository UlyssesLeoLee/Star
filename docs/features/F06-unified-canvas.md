# F06 统一画布

> 状态摘要: Worktree 群组内的无限画布已由服务端权威实现 (按 `worktree_id` 作用域的画布、元素、Frame 与连线文档, 带版本号、幂等键与变更事件, 并以 SSE 唤醒加游标追赶同步); 群组挂载把 `readOnly` 置为真, 新建元素工具、撤销与图层控制在群组 live 模式下不可用; PNG 导出、分享、实时光标与 CRDT 协作未实现。`/canvas` 与 `/canvas/[id]` 使用浏览器本地 zustand 画布, 不经服务端。

> 代码基线: 637ed19d; 核对方法: 路径与符号 git grep 与 `git show 637ed19d:<path>` 逐行读取, 每条实现结论回读对应行; 核对日期 2026-11-19。

---

## 1. 目标与边界

### 1.1 目标

统一画布是 Worktree 群组 (Branch 工作区) 内的一块无限画布, 把同一 Worktree 下的任务卡 (work-item 卡)、Worktree 节点、便签、文本、形状、图片位、连线与 Frame 放在同一空间中。画布元素对任务事实只持有类型化引用 `EntityRef(ref_type, ref_id, worktree_id)`, `ref_type` 取 `work_item` 或 `worktree`; 布局与画布元数据归画布所有, 任务状态归 WorkItem 域, 画布不维护第二份任务状态机 (SRS-CANVAS-001.md:170-172; BD-CANVAS-001.md:184-185; DD-CANVAS-001.md:565-589)。

画布与任务看板、Task Card、插件同级挂在群组应用导航中, 共用群组底部聊天栏; 画布只把当前 Worktree 与选中元素作为上下文传递 (BD-CANVAS-001.md:188; DD-CANVAS-001.md:914-917 之后的 §14.2 交互协议)。

### 1.2 范围内 / 范围外

范围内:
- 群组「无限画布」应用的挂载、工具栏、投影与交互: `frontend/src/app/worktree/[id]/group/page.tsx:3126-3256`。
- 共享渲染组件 `CanvasView`: `frontend/src/components/CanvasView.tsx`。
- 群组画布的客户端投影、API 客户端与唤醒流: `frontend/src/lib/group/groupProjection.tsx`, `frontend/src/lib/group/worktreeGroupApi.ts`。
- 服务端 Worktree 作用域画布 API 与事件流: `crates/star-api-rest/src/group_api/canvas.rs`, `crates/star-api-rest/src/group_api/canvas_event_stream.rs`。
- 画布持久化迁移: `db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql`, `…-phase-3c-canvas-document.sql`, `…-phase-3e-canvas-outbox-index.sql`, `2026-10-05-worktree-group-canvas-stream-offset.sql`, `2026-10-28-canvas-outbox-wakeup.sql`。
- 上下文入口与本地画布页: `frontend/src/app/canvas/page.tsx`, `frontend/src/app/canvas/[id]/page.tsx`, `frontend/src/lib/canvas-entry.ts`。
- 画布类型: `frontend/src/types/ids.ts` 中的 Canvas、CanvasElement、CanvasConnector、CanvasFrame。

范围外:
- `/canvas/worktree-graph` 即 Project 级 Worktree Overview Graph, 另一对象 (SRS-CANVAS-001.md:176-177; DD-CANVAS-001.md:917-931 中的 Project Worktree Overview Graph 行)。该页的 `Link` 与 `canvas-001` 引用仅作为旁路入口出现。
- `/canvas/[id]` 中 Agent 画布、关系图、流程、Git 画布四个子 tab 的内部能力, 各自归属其 feature; F06 只记录挂载点 (`frontend/src/app/canvas/[id]/page.tsx:158-245`)。
- 三核心子需求 `SRS-CANVAS-AGENT-001`、`SRS-CANVAS-GAMIFY-001`、`SRS-CANVAS-WORKFLOW-001` (SRS-CANVAS-001.md:103-108)。
- 外部 SaaS 连接器与具名第三方集成 (SRS-CANVAS-001.md:140-141)。

---

## 2. 用户与场景

| 角色 | 主要场景 | 证据 |
|---|---|---|
| 群组成员 (编辑) | 在当前 Worktree 画布上拖动、删除元素, 保存视口、Frame 与视觉连线 | group/page.tsx:1703-1775 (位置保存), 2349-2420 (文档保存); canvas.rs:1208, 1600, 840 |
| 群组成员 (建卡) | 在画布上新建任务卡, 同一事务内关联 Worktree 与画布 | group/page.tsx:1378-1440; canvas.rs:571-733 |
| 群组成员 (关联) | 把当前 Worktree 中未关联的任务卡放到画布 | group/page.tsx:1512-1569 |
| 观察者 | 查看任务卡与 Worktree 节点的状态色码; 通过 `?highlight=` 深链定位元素 | CanvasView.tsx:284-291, 784-824 |
| 单机草图用户 | 在 `/canvas/[id]` 用本地画布起草便签与形状, 不经服务端 | canvas/[id]/page.tsx:158-204; canvas-entry.ts:79-103 |

---

## 3. 需求

优先级: P0 为群组画布的核心闭环, P1、P2、P3 依次降低。编号 FR-06-NN。状态标记按本文开头的约定使用。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-06-01 | `/canvas` 按当前 Worktree 或 Lane 上下文解析画布; 无匹配时显示未接入状态并可新建 | P1 | ✅ | canvas/page.tsx:37-47 (解析), 73-86 (新建), 114-127 (未接入); canvas-entry.ts:37-64 |
| FR-06-02 | `/canvas/[id]` 按路由 id 取画布; 找不到时显示未接入状态 | P1 | ✅ | canvas/[id]/page.tsx:73 (取用), 208-225 (未接入分支) |
| FR-06-03 | 群组导航提供「无限画布」应用, 仅该应用激活时加载画布投影 | P0 | ✅ | group/page.tsx:634 (应用定义), 995 (`includeCanvas = currentApp === "canvas"`); groupProjection.tsx:250-265 |
| FR-06-04 | live 模式显示授权投影; preview 模式不以 seed 数据冒充当前 Worktree 画布 | P0 | ✅ | groupProjection.tsx:161-164 (模式类型), 175-177 (无客户端时为 preview); group/page.tsx:3251-3252 (无画布时的提示文案) |
| FR-06-05 | live 模式下当前 Worktree 无画布时, 经幂等命令创建 | P0 | ✅ | group/page.tsx:1483-1511 (`createWorktreeCanvas`), 3252 (按钮); worktreeGroupApi.ts:1186-1192; canvas.rs:773-838 |
| FR-06-06 | 平移、滚轮与按钮缩放, 缩放范围 0.1 至 4 | P0 | ✅ | CanvasView.tsx:403 (滚轮夹取), 1198-1201 (按钮); canvas.rs:955-956 (服务端 0.1 至 4 校验) |
| FR-06-07 | `!` (Shift+1) 适应全部, `@` (Shift+2) 适应选中 | P1 | ✅ | CanvasView.tsx:491-510 |
| FR-06-08 | 视口、Frame 与连线以 canvas document 保存; 版本不符时拒绝 | P0 | ✅ | group/page.tsx:1570-1578 (暂存视口), 2349-2420 (保存); canvas.rs:840-958, 882-886 (版本比较与 `canvas_document_version_conflict`) |
| FR-06-09 | `?highlight=` 深链自动定位并高亮元素 | P1 | ✅ | CanvasView.tsx:284-291 (居中), 726-729 (高亮描边); group/page.tsx:3233 |
| FR-06-10 | 小地图显示视口范围与元素分布 | P1 | ✅ | CanvasView.tsx:1454-1475 (`canvas-minimap`) |
| FR-06-11 | 视口裁剪渲染, 1K 元素保持 60fps | P1 | ⬜ | CanvasView.tsx 中 `displayElements.map` 对全部元素渲染 (CanvasView.tsx:1085-1089), 无裁剪判断; 无性能基准 |
| FR-06-12 | 便签与文本元素创建 | P0 | 🟡 | 写模式工具栏与创建逻辑 CanvasView.tsx:1083-1111, 612-640; 群组 live 下 `readOnly` 为真使工具栏隐藏 (CanvasView.tsx:1083, group/page.tsx:3234); 服务端接受 `sticky_note` 与 `text` (canvas.rs:2178-2187; migration:53-56) |
| FR-06-13 | 形状元素 (矩形、圆角、圆、菱形、三角) 创建与渲染 | P1 | 🟡 | 渲染 CanvasView.tsx:942-965; 形状选择 CanvasView.tsx:1111-1124 (SHAPE_KINDS); 群组 live 创建门控同 FR-06-12; 服务端对 `shape_kind` 不做校验 (canvas.rs:2168-2211 中无该字段检查) |
| FR-06-14 | 图片元素渲染 | P2 | 🟡 | 渲染分支 CanvasView.tsx:966-968 (`case "image"`); 创建入口未在工具栏中找到; 服务端接受 `image` (canvas.rs:2178-2187) |
| FR-06-15 | 嵌入 (embed) 元素渲染与创建 | P2 | ⬜ | 服务端接受 `embed` (canvas.rs:2178-2187; migration:53-56); CanvasView.tsx 的渲染分支仅 `image` 与 `embed` 共用 (CanvasView.tsx:966-967), 无嵌入内容实现 |
| FR-06-16 | 元素拖动, 位置经 `update_mode=position` 保存; 锁定元素不可拖动 | P0 | ✅ | CanvasView.tsx:329-334 (拖动预览), 365-368 (释放提交), 434 (锁定判断); group/page.tsx:1703-1775; canvas.rs:1245-1250 (Position 分支), 1991-1992 (锁定拒绝 `canvas_element_locked`) |
| FR-06-17 | 元素宽高与旋转编辑 (geometry 模式) | P1 | ✅ | 群组几何编辑器 group/page.tsx:3179-3184 (宽、高、旋转); 保存 group/page.tsx:1904 (`saveCanvasElementGeometry`); 服务端 Geometry 分支 canvas.rs:1245-1250 |
| FR-06-18 | 元素内容编辑 (content 模式, 便签文字) | P1 | 🟡 | 群组便签编辑器 group/page.tsx:3185-3188 (仅 `sticky_note`/`text` 且未锁定, CanvasView.tsx:547); 保存 group/page.tsx:1837; 服务端 Content 分支 canvas.rs:1245-1250; 文字编辑在 CanvasView 的 `editingId` 路径为本地 store 写入 (CanvasView.tsx:1291 `replaceCanvasElementContent?.`) |
| FR-06-19 | 删除选中元素 (Delete、Backspace、工具栏) | P0 | ✅ | CanvasView.tsx:521-523 (快捷键), 443-455 (`deleteSelection`), 1256 (按钮); group/page.tsx:1775-1837; canvas.rs:1600-1756 (同事务清理 Frame 成员与连线) |
| FR-06-20 | 元素层级上移与下移 (z_index) | P2 | 🟡 | 图层面板 CanvasView.tsx:1399-1446 (上移、下移按钮); `moveLayer` 在 `onCreateElement` 已传入时直接返回 (CanvasView.tsx:131-133); 群组 live 与 `/canvas/[id]` 均传入该回调, 故按钮无效果 |
| FR-06-21 | 元素显示与隐藏切换 | P2 | 🟡 | 切换按钮 CanvasView.tsx:1447-1450; `toggleLayerVisible` 在 `onCreateElement` 已传入时直接返回 (CanvasView.tsx:148-150); 服务端 `hidden` 字段可写于创建 (canvas.rs:1154-1171), 更新路径不切换 (canvas.rs:1343-1345 沿用当前行) |
| FR-06-22 | 任务卡: 显示状态色码 (StatusPill) 并可打开对应任务 | P0 | ✅ | 渲染 CanvasView.tsx:784-802 (状态取自授权投影 `groupWorkItems`), 双击打开 CanvasView.tsx:545-557; 群组 `openCanvasTask` 校验当前 Worktree 绑定 group/page.tsx:1314-1327 |
| FR-06-23 | Worktree 节点: 显示 status 色码 | P0 | ✅ | CanvasView.tsx:808-828 |
| FR-06-24 | 在画布内新建任务卡, 并在同一事务中关联 Worktree 与画布 | P0 | ✅ | 服务端单事务: canvas.rs:571-733 (任务创建、元素行、EntityRef 行、事件、幂等写入同一 `tx`); 前端入口 group/page.tsx:3133 ("从画布…" 按钮) 与 1378-1440 (`createCanvasTask`) |
| FR-06-25 | 把已有任务卡或 Worktree 关联为画布元素 (entity_ref 设置与清除) | P0 | 🟡 | 服务端 `set_entity_ref` canvas.rs:1371-1510 与 `clear_entity_ref` canvas.rs:1511-1599 已实现; 前端 `WorktreeGroupApiClient` 无对应方法 (worktreeGroupApi.ts:1182-1360 之间无 `entity-ref` 调用); 群组的「关联已有任务卡」按钮 group/page.tsx:3134 与 1512-1569 走 `createCanvasElement` 创建新卡, 不调用 entity-ref 接口 |
| FR-06-26 | `EntityRef` 必须属于同一 Worktree, 跨 Worktree 引用被拒绝 | P0 | ✅ | canvas.rs:1853-1892 (`entity_ref_worktree_mismatch` 于 1858-1860; worktree 类型的他处引用返回 `not_found` 于 1868-1870); work_item 类型经 `multica.work_item_worktree` 关联校验 canvas.rs:1871-1891 |
| FR-06-27 | 连线三种路由 (straight、orthogonal、curved) 与输入输出端口 | P0 | ✅ | 路由校验 canvas.rs:2023-2028 (服务端 `validate_document` 的 `routing` 白名单, 见 canvas.rs:988-1003); 渲染 CanvasView.tsx:1007-1048; 端口渲染与拖线测试 CanvasView.test.tsx |
| FR-06-28 | 连线拖拽创建与持久化 | P0 | 🟡 | 端口拖线 `startPortDrag`/`finishPortDrag` 只调用本地 store 的 `addCanvasConnector` (CanvasView.tsx:220-253), 在 `readOnly` 下不启动 (CanvasView.tsx:221); 群组持久化通过「连接选中元素」按钮 (group/page.tsx:1650-1671, 3150-3151) 把两个选中元素以 `kind: "free"` 写入文档草稿后经 `PUT document` 保存 (group/page.tsx:2349-2420); 拖线本身未接入服务端 |
| FR-06-29 | Frame 分区渲染 | P1 | ✅ | CanvasView.tsx:977-995 (`renderFrame`); 服务端 frame 校验 canvas.rs:988-1005; 文档 frame 元素引用须属于当前画布 canvas.rs:1019-1059 |
| FR-06-30 | Frame 创建、把选中元素加入与移出 Frame | P1 | ✅ | 群组工具栏 group/page.tsx:3135-3148 (添加 Frame), 3149-3156 (加入、移出、删除); 草稿与保存 group/page.tsx:1589-1650; 画布内 `frame-select` 工具仍为占位 (CanvasView.tsx:1154, 标注"下一阶段") |
| FR-06-31 | 画笔自由绘制 (path 元素) | P2 | 🟡 | 本地创建 CanvasView.tsx:298-303 与渲染 CanvasView.tsx:735-759 (`case "path"`); 服务端 CHECK 与校验不接受 `path` (migration:53-56; canvas.rs:2178-2187); 群组 live 下画笔工具不可见 (CanvasView.tsx:1083, 1160 之后) |
| FR-06-32 | 评论钉渲染与评论工具 | P1 | 🟡 | 评论工具 CanvasView.tsx:310-318 (仅在 `onCreateElement` 存在且非只读时); `comment_pin` 渲染 CanvasView.tsx:931-941; 服务端接受 `comment_pin` 种类 (canvas.rs:2178-2187); 评论线程未持久化 (见 FR-06-47) |
| FR-06-33 | 撤销与重做 | P0 | ⬜ | 撤销栈仅在 `onCreateElement === undefined` 时工作 (CanvasView.tsx:134, 473, 1220); 群组 live 与 `/canvas/[id]` 均传入该回调 (group/page.tsx:3242; canvas/[id]/page.tsx:170-190), 当前挂载下撤销不可达 |
| FR-06-34 | 键盘工具切换 (V、H、P、S、T、N、L、E、C)、Delete、Escape | P1 | ✅ | CanvasView.tsx:512-525; 注意 readOnly 下工具切换仍生效而工具栏隐藏 |
| FR-06-35 | 多选与框选 | P0 | ❓ | 选区数组与多选删除存在 (CanvasView.tsx:262-263, 451); Shift 拖动为平移 (CanvasView.tsx:295), 与 Shift 多选约定冲突; 框选逻辑未在 CanvasView.tsx 中找到。确认方法: `git grep -n "marquee\|selectionBox\|rubber" 637ed19d -- frontend/src/components/CanvasView.tsx` |
| FR-06-36 | PNG 导出 | P1 | ❓ | `html2canvas` 依赖声明 (frontend/package.json:35 处, 需回读确认); `frontend/src` 中未见 import; 导出控件未找到。确认方法: `git grep -n "html2canvas\|Export PNG" 637ed19d -- frontend/src frontend/package.json` |
| FR-06-37 | 分享链接与复制画布地址 | P1 | ❓ | 分享控件未在画布组件中找到。确认方法: `git grep -n "share-link\|Share" 637ed19d -- frontend/src/components/CanvasView.tsx frontend/src/app/canvas` |
| FR-06-38 | JSON 或 Miro 导入导出 | P2 | ⬜ | 画布范围内未找到导入导出实现 |
| FR-06-39 | 实时同步: 服务端事件唤醒与游标追赶 | P0 | 🟡 | SSE 唤醒路由 canvas_event_stream.rs:148-152; 唤醒多路复用 worktreeGroupApi.ts:1291-1330, 1487-1575; 游标追赶 worktreeGroupApi.ts:1397-1486; 装配 groupProjection.tsx:216-237。不是 CRDT, 不做字段级合并; 唤醒仅携带通知, 事件体经游标 GET 读取 |
| FR-06-40 | 实时光标与 presence | P1 | ⬜ | 群组页与 CanvasView 无 presence 实现; 画布迁移中无 presence 表 (迁移表清单见 §4.2) |
| FR-06-41 | CRDT (Yjs) 协作与离线合并 | P1 | ⬜ | `frontend/package.json` 中无 yjs 或 y-websocket 依赖 (需回读确认); 并发冲突以 `expected_version` 拒绝处理 (canvas.rs:1218-1220, 882-886) |
| FR-06-42 | 乐观并发: 文档与元素版本号不符时拒绝 | P0 | ✅ | 文档 canvas.rs:882-886 (`canvas_document_version_conflict`); 元素 canvas.rs:1218-1220 (`version_conflict`); 删除 canvas.rs:1620-1626 |
| FR-06-43 | 幂等写: `Idempotency-Key` 与请求哈希 | P0 | ✅ | 客户端 worktreeGroupApi.ts:1186-1192 (`createCanvas` 带头), 1194-1260 (元素与文档写入); 服务端 `idempotency_key` 与 `request_hash` 在 canvas.rs:786, 863, 1120, 1224, 1392, 1527, 1616, 2230-2247 |
| FR-06-44 | 画布变更事件历史查询 | P1 | ✅ | 单画布事件 canvas.rs:439-484; 全 Worktree 事件 canvas.rs:485-543; 客户端 worktreeGroupApi.ts:1267-1290 |
| FR-06-45 | 作用域与行级安全: `canvas:read`、`canvas:write` 与 Worktree 授权 | P0 | ✅ | 读 canvas.rs:446, 492, 740, 1067; 写 canvas.rs:581, 781, 848, 1103, 1216, 1379, 1519, 1608; 授权 canvas.rs:1756-1766 (`request_worktree`), 1767-1823 (`authorize_group_scope`); RLS 启用迁移 `2026-09-29-worktree-group-phase-3-canvas.sql:167-269` |
| FR-06-46 | 画布写入审计 | P1 | 🟡 | 审计表 `canvas.canvas_group_audit` (迁移 `…-phase-3-canvas.sql:114-128`); 写入经 `append_event` canvas.rs:1923-1955; 审计读取接口与界面未找到 |
| FR-06-47 | 画布评论线程、@ 提醒与多人评论 | P1 | ⬜ | 画布迁移中无评论表; 仅 `comment_pin` 渲染 (CanvasView.tsx:931-941) |
| FR-06-48 | `/collaboration` 画布列表与新建 | P2 | 🟡 | `frontend/src/app/collaboration/page.tsx:16-24` (本地 store 的 `canvases`), 列表链接 `/canvas/${c.id}` (collaboration/page.tsx:69); 数据源为本地 store, 不经服务端 |
| FR-06-49 | `/canvas/[id]` 子 tab 挂载 (画布、Agent 画布、关系图、流程、Git 画布) | P2 | 🟡 | `frontend/src/app/canvas/[id]/page.tsx:54` (`CANVAS_SUB_TABS`), 158-245 (`renderCanvasContent`); 文件头注释仍写三标签 (canvas/[id]/page.tsx:9-13) |
| FR-06-50 | 演示模式 (Frame 作为幻灯片) | P3 | ⬜ | Frame 的 `is_slide` 字段可编辑 (group/page.tsx:3162-3169), 但无演示模式的播放实现; 画布目录与 CanvasView 检索无 presentation 播放组件 |
| FR-06-51 | 性能基准: 1K 元素与 50 协作者 | P1 | ❓ | 未见性能基准或压测。确认方法: 前端 benchmark 与 PostgreSQL 负载测试 |

---

## 4. 设计

### 4.1 概念模型

- **群组画布 (Group Canvas)**: `canvas.group_canvas_registry` 行。业务键为 `tenant_id + worktree_id + canvas_id`, `version` 为聚合版本, 同一 `canvas_id` 同一时刻仅一行 `is_current` (`db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql:20-45`, 含唯一索引 `uq_group_canvas_current` 与 `uq_group_canvas_single_owner_current`)。一个 Canvas 只属于一个 Worktree (`uq_group_canvas_single_owner_current`)。
- **画布文档 (Canvas document)**: `viewport`、`frames`、`connectors` 三列随注册行共用版本 (`db/migrations/2026-09-29-worktree-group-phase-3c-canvas-document.sql:4-12`)。连线是布局投影, 不是 WorkItem 关系 (同文件注释与列注释)。
- **画布元素 (Element)**: `canvas.canvas_elements_backend` 中的 SCD Type 2 行, 含 `kind`、几何、`z_index`、`content`、`locked`、`hidden`, 每次变更关闭旧版本并插入新版本 (canvas.rs:1997-2072 `close_element_version` 与 `bump_element_version`)。
- **实体引用 (EntityRef)**: `canvas.canvas_entity_ref`, 以 `ref_type` 与 `ref_id` 关联元素与任务卡或 Worktree, 同样是版本化行 (canvas.rs:1894-1921 `insert_entity_ref`)。
- **事件 (Outbox)**: `canvas.canvas_group_outbox` 记录画布变更; 客户端经游标读取与 SSE 唤醒同步。
- **本地画布 (Local canvas)**: zustand store 中的 `canvases`、`canvasElements`、`canvasConnectors` (`frontend/src/lib/store.ts:170-172`), 与服务端无关 (canvas-entry.ts:79-103)。

### 4.2 数据模型

| 实体 | 存储 | 关键字段与约束 | 分类 (守门 #13) |
|---|---|---|---|
| group_canvas_registry | `canvas.group_canvas_registry` | `version` CHECK > 0 (phase-3:25), `title` 长度 1 至 200 (:26), `is_current` 与 `valid_to` 互斥 CHECK (:37-38), 外键到 `worktree_canvas_worktree (tenant_id, id)` (:31-32); 文档列 `viewport`、`frames`、`connectors` 与形状 CHECK `ck_group_canvas_document_shape` (3c:4-30, zoom 0.1 至 4) | Master, SCD Type 2 (3c:6-7 声明) |
| canvas_elements_backend | `canvas.canvas_elements_backend` | `kind` CHECK 10 值 (phase-3:53-56), `width` 与 `height` 0 < 值 ≤ 10000 (:58-59), `version`, `valid_from` / `valid_to` | Master, SCD Type 2 |
| canvas_entity_ref | `canvas.canvas_entity_ref` | `ref_type` CHECK 为 `work_item` 或 `worktree` (phase-3:92), 版本与有效期 (:94-107), 目标索引 (:110-111) | Master, SCD Type 2 |
| canvas_group_audit | `canvas.canvas_group_audit` | 只追加: `reject_group_append_only_mutation` 触发器 (phase-3:152-165) 与无更新、无删除策略 (:263-266) | Transaction |
| canvas_group_outbox | `canvas.canvas_group_outbox` | `schema_version`, `aggregate_version`, 游标索引 (3e:12); `stream_offset` 迁移 (2026-10-05 stream-offset:13) | Transaction |
| canvas 唤醒触发器 | `canvas.notify_canvas_outbox_wake` | Outbox 事务提交后 `pg_notify('star_canvas_outbox_wake', worktree_id)` (2026-10-28 canvas-outbox-wakeup.sql:16-30) | 基础设施 |
| Canvas (前端类型) | `frontend/src/types/ids.ts` | 视口、`frames`、`ref_kind` (`work_item`、`worktree`、`project`、`free`) | 参考类型 |
| CanvasElement (前端类型) | `frontend/src/types/ids.ts` | `kind`、几何、`entity_ref`、`content`、`locked`、`hidden` | 参考类型 |
| CanvasConnector (前端类型) | `frontend/src/types/ids.ts` | `kind`、`from_element_id`、`to_element_id`、`routing`、颜色、宽度、标签 | 参考类型 |
| CanvasFrame (前端类型) | `frontend/src/types/ids.ts` | 矩形、`element_ids`、`is_slide`、顺序 | 参考类型 |

词表差异 (以代码为准):
- 前端 `CanvasElementKind` 含 `path`, 服务端 KINDS 与迁移 CHECK 均为 10 值, 不含 `path` (canvas.rs:2168-2189 `validate_element_body`; phase-3:53-56)。
- 前端 `CanvasEntityRef.ref_type` 为六值 (`work_item`、`worktree`、`agent_session`、`automation`、`pull_request`、`comment`), 服务端只接受 `work_item` 与 `worktree` (canvas.rs:2213-2228 `validate_required_ref`; canvas.rs:1853-1892)。

### 4.3 接口

服务端路由 (`crates/star-api-rest/src/group_api/canvas.rs:403-436`, 合并于 `crates/star-api-rest/src/group_api.rs:824`):

| 方法 | 路径 | 作用域 | 处理函数 | 状态 |
|---|---|---|---|---|
| GET、POST | `/api/v1/worktrees/{worktree_id}/canvases` | canvas:read / canvas:write | `list_canvases` (canvas.rs:734), `create_canvas` (canvas.rs:773) | ✅ |
| GET、POST | `…/canvases/{canvas_id}/elements` | read / write | `list_elements` (canvas.rs:1061), `create_element` (canvas.rs:1095) | ✅ |
| PUT、DELETE | `…/canvases/{canvas_id}/elements/{element_id}` | write | `update_element` (canvas.rs:1208), `delete_element` (canvas.rs:1600) | ✅ |
| PUT、DELETE | `…/elements/{element_id}/entity-ref` | write | `set_entity_ref` (canvas.rs:1371), `clear_entity_ref` (canvas.rs:1511) | 🟡 服务端 ✅, 前端无调用 |
| PUT | `…/canvases/{canvas_id}/document` | write | `update_document` (canvas.rs:840) | ✅ |
| GET | `…/canvases/{canvas_id}/events` | read | `list_canvas_events` (canvas.rs:439) | ✅ |
| GET | `/api/v1/worktrees/{worktree_id}/canvas-events` | read | `list_worktree_canvas_events` (canvas.rs:485) | ✅ |
| POST | `…/canvases/{canvas_id}/work-items` | work-item:write + canvas:write | `create_work_item_on_canvas` (canvas.rs:571) | ✅ |
| GET (SSE) | `/api/v1/worktrees/{worktree_id}/canvas-events/stream` | worktree:read + canvas:read | `stream_worktree_canvas_events` (canvas_event_stream.rs:147-152, 155-170) | ✅ |

请求体要点:
- `UpdateElementBody` 字段为 `expected_version`、`update_mode` (`position`、`geometry`、`content`, canvas.rs:304-310), 以及 x、y、宽、高、旋转、content; 未选中的字段由服务端从当前行带回 (canvas.rs:1244-1250)。
- `CreateElementBody` 含 `locked` 与 `hidden` (canvas.rs:281-299), 更新路径不切换这两个字段。
- 文档请求 `CanvasDocumentBody` 字段为 `expected_version`、`viewport`、`frames`、`connectors` (canvas.rs:191-200); 服务端校验: 视口 zoom 0.1 至 4, 最多 1000 个 Frame, 最多 5000 条连线, 连线 `kind` 取 `work_item_relation`、`agent_handoff`、`free`、`dependency`, `routing` 取 `straight`、`curved`、`orthogonal` (canvas.rs:960-1017)。

前端客户端 (`frontend/src/lib/group/worktreeGroupApi.ts`): `listCanvases` 1182、`createCanvas` 1186、`createCanvasElement` 1194、`updateCanvasElement` 1210、`deleteCanvasElement` 1227、`updateCanvasDocument` 1244、`listCanvasElements` 1260、`listCanvasEvents` 1267、`listWorktreeCanvasEvents` 1283、`openWorktreeCanvasWakeStream` 1291、`subscribeWorktreeCanvasWake` 1329、`createWorkItemOnCanvas` 1345。同文件另有 `CanvasOutboxCatchUp` (1397)、`WorktreeCanvasWakeMultiplexer` (1487)、`CanvasOutboxWakeSubscriber` (1576)。

不在群组 live 路径上的服务: `crates/canvas-realtime` 与 `crates/canvas-engine`、`crates/domain-canvas` 的健康与占位路由, 以及 `crates/canvas-collab` 的骨架, 不计入本功能的接口。

### 4.4 状态机与关键流程

元素版本 (服务端):

```
create (version 1)
  → update: expected_version 必须等于当前版本 (canvas.rs:1218-1220)
            旧版本 close_element_version (canvas.rs:1997) → bump_element_version (canvas.rs:2024)
  → locked 元素拒绝 update (canvas.rs:1991-1992, canvas_element_locked)
  → delete: 同事务清理 Frame 成员与连线, 写审计与事件 (canvas.rs:1600-1756)
```

画布入口 (`/canvas`): 挂载前 loading (canvas/page.tsx:80-90) → 按上下文解析 redirecting (canvas/page.tsx:92-103) 或 unbound 并提供新建 (canvas/page.tsx:105-131)。

群组 live 挂载的文档保存:
1. 视口变化经 `stageCanvasViewport` 写入待保存草稿 (group/page.tsx:1570-1578)。
2. 位置、几何、连线、Frame 的修改同样写入草稿 (group/page.tsx:1589-1703)。
3. `saveCanvasDocument` 携带 `expected_version` 调用 `PUT document` 并带 `Idempotency-Key` (group/page.tsx:2349-2420)。
4. 服务端版本不符返回 `canvas_document_version_conflict` (canvas.rs:882-886); 界面显示错误并提供「放弃草稿并加载最新版本」按钮 (group/page.tsx:3223-3224 附近), 按钮调用 `clearCanvasDocumentDraft` 并递增 `projectionRefreshKey`。

元素拖动 (群组 live): `CanvasView` 释放时经 `onElementPositionChange` (CanvasView.tsx:365-368) → `saveCanvasElementPosition` (group/page.tsx:1703) → `updateCanvasElement`, `update_mode=position` (canvas.rs:1245-1250) → 追加事件 (canvas.rs:1923-1955)。

建卡 (群组 live): `createCanvasTask` (group/page.tsx:1378-1440) → `createWorkItemOnCanvas` → 服务端单事务 (canvas.rs:571-733): 幂等锁、任务创建 (`run_task_card_pg_adapter.create_composed_with_audit_in_transaction`, canvas.rs:675-690)、`work_item_card` 元素行、`EntityRef` 行、事件、幂等结果写入后提交。

同步: 游标追赶 `CanvasOutboxCatchUp` 按 cursor 读取 `listWorktreeCanvasEvents` (worktreeGroupApi.ts:1283-1290, 1397-1486); 唤醒多路复用器经 SSE 订阅并在断线后重连 (worktreeGroupApi.ts:1487-1575, 1291-1330); 装配在 groupProjection.tsx:216-237。服务端 `PgListener` 监听 `star_canvas_outbox_wake`, 唤醒经 `broadcast` 扇出到各 SSE 流 (canvas_event_stream.rs:71, 289-305)。

### 4.5 权限与安全

- **作用域**: 读取 `canvas:read`, 写入 `canvas:write` (见 §3 FR-06-45 证据)。`work-item:write` 与 `work-item:read` 分别用于建卡与字段级可见性 (canvas.rs:580, 726, 1085)。
- **字段级**: 任务卡详情仅在持有 `work-item:read` 时随元素投影返回 (canvas.rs:726, 1085, 2161-2166 `has_scope`)。
- **Worktree 绑定**: `request_worktree` (canvas.rs:1756-1766) 与 `authorize_group_scope` (canvas.rs:1767-1823) 基于设备上下文校验。
- **引用校验**: `EntityRef` 必须属于同一 Worktree (canvas.rs:1853-1892)。
- **并发与幂等**: `expected_version` 与请求哈希 (canvas.rs:786, 863, 1120, 1224, 1392, 1527, 1616) 配合 `identity_key_present` 拒绝把身份键写入内容 (canvas.rs:2230-2247, 2212-2226)。
- **行级安全**: 五张画布表启用 RLS 并声明 SELECT、INSERT、UPDATE 与删除阻断策略 (phase-3:167-269)。注册表与元素表的删除策略为 `no_delete`。
- **唤醒流授权**: SSE 流每 60 秒重新校验授权租约 (canvas_event_stream.rs:75, 161-180), 进程内最多 128 条流 (canvas_event_stream.rs:74, 94-96)。
- **前端门控**: `readOnly` 与 `onCreateElement` 只决定 UI 是否显示工具, 不是安全边界; 服务端以 scope 为准。
- **未实现**: 分享链接、公开访问与 OAuth 授权均未实现 (见 FR-06-37)。

### 4.6 依赖关系

- 服务端: `star-api-rest` 的 `group_api` 模块 (`crates/star-api-rest/src/group_api.rs:206-207` 声明, `:824-825` 合并路由); PostgreSQL 与画布迁移; 任务卡创建依赖 `crates/infrastructure/src/run_task_card_pg.rs` 的 `PgRunTaskCardAdapter`; 任务关联校验读取 `multica.task_metadata` 与 `multica.work_item_worktree`。
- 启动: `crates/star-api-rest/src/main.rs:106-108` 启动唤醒监听, 失败则进程启动失败。
- 前端: 群组页 (`frontend/src/app/worktree/[id]/group/page.tsx`)、投影 (`frontend/src/lib/group/groupProjection.tsx`)、客户端 (`frontend/src/lib/group/worktreeGroupApi.ts`)、共享组件 `CanvasView`、本地 store (`frontend/src/lib/store.ts`)、状态图例 `StatusPill`。
- 任务域: WorkItem 与 Worktree 通过 `entity_ref` 与 `create_work_item_on_canvas` 关联 (canvas.rs:571, 1853-1892)。
- 未使用的依赖: `html2canvas` 已声明 (`frontend/package.json:35`), 需回读确认未被引用 (见 FR-06-36)。
- 独立服务: `canvas-engine`、`domain-canvas`、`canvas-realtime` 仅有健康与占位路由, 不在群组 live 路径上 (`crates/canvas-realtime/src/lib.rs:95-98`; `crates/domain-canvas/src/lib.rs:84-87`; `crates/canvas-engine/src/lib.rs:88-92`)。

### 4.7 页面设计

本功能拥有以下页面路由, 每个路由单独成节。

#### 4.7.1 /worktree/[id]/group?app=canvas

文件: `frontend/src/app/worktree/[id]/group/page.tsx`。应用由 `app=canvas` 激活 (`AppId` 定义于 group/page.tsx:526, 应用项于 :634)。

布局:
- 顶栏、侧栏、主区与底栏由群组壳提供, 画布应用渲染于主区; 群组壳的 `currentApp` 分支在 group/page.tsx:3126 开始。
- 右侧栏与底栏 (聊天栏) 属于群组壳, 本功能不实现 (`docs/design/BD-CANVAS-001.md` §1.3 所述共享规则)。
- 画布应用内部分区自上而下: 应用标题与工具条 (group/page.tsx:3127-3159)、Frame 编辑器 (3162-3169)、连线编辑器 (3170-3178)、几何编辑器 (3179-3184)、便签编辑器 (3185-3188)、建卡表单 (3189-3195)、关联表单 (3196-3222)、状态与错误提示 (3223-3249)、画布主体 (3228-3247)。

组件树:
- `WorkspaceGroupPage` (group/page.tsx) → `CanvasView` (`frontend/src/components/CanvasView.tsx:109`, 挂载于 group/page.tsx:3228) → 元素渲染 `renderElement` (CanvasView.tsx:724), 连线 `renderConnector`, Frame `renderFrame` (CanvasView.tsx:977), 工具栏 (CanvasView.tsx:1080-1260), 小地图 (CanvasView.tsx:1454), 图层面板 (CanvasView.tsx:1399)。
- 投影 `useWorktreeGroupProjection` (`frontend/src/lib/group/groupProjection.tsx:166`), 客户端 `WorktreeGroupApiClient` (worktreeGroupApi.ts:801)。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 (live) | 授权投影返回画布 | 画布主体与全部工具栏 | ✅ group/page.tsx:3126-3247; CanvasView 中仅平移、缩放、拖动、删除、任务卡双击打开、高亮 (readOnly 为真) |
| 加载 | `mode === "loading"` | 「正在读取已授权的 Worktree Group 数据…」 | ✅ group/page.tsx:2387 附近 (加载分支) |
| 空 (无画布) | 授权列表中无 `groupCanvas` | 「当前 Worktree 没有显式绑定的 Group Canvas。」与「创建此 Worktree 的 Canvas」按钮 | ✅ group/page.tsx:3248-3256 |
| 错误 | 投影 `mode === "error"` 或刷新失败 | 「Canvas 更新暂不可用」提示并保留最后成功投影 | ✅ group/page.tsx:3223; groupProjection.tsx:226-232 |
| 版本冲突 | `PUT document` 返回 `canvas_document_version_conflict` | 错误提示与「放弃草稿并加载最新版本」 | ✅ group/page.tsx:3223-3224 附近 |
| 元素冲突 | 元素更新返回 `version_conflict` | 位置、几何、便签各自的错误提示 | ✅ group/page.tsx:3202-3247 中的 `canvasElement*Error` |
| 未授权 | 授权失败或 scope 缺失 | 错误或 403 | 🟡 服务端返回 403 路径存在 (canvas.rs:1767-1823); 页面对 403 的专门呈现未核实 |
| 未接入 | 预览 (`mode === "preview"`) | 「本地预览 · seed 任务已隐藏」标签; 画布工具栏的写入按钮隐藏; 画布主体仍可显示 seed | 🟡 group/page.tsx:2612-2613 与 3251 文案区分 live 与 preview; preview 下 `CanvasView` 以 `readOnly=false` 运行 (group/page.tsx:3234), 不符合 FR-06-04 中的"不以 seed 冒充"要求 |
| 禁用 | `readOnly` 为真 | 工具栏创建类按钮隐藏; 拖动与删除保留 | ✅ CanvasView.tsx:1083, 1220, 1256 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换 Canvas (多于一个) | live 且 `groupCanvases.length > 1` | `canvas_id` 写入 URL | `selectCanvas` (group/page.tsx:1352-1358) | ✅ |
| 新建 Canvas | live | 创建并选中 | `createWorktreeCanvas` → `POST /canvases` (group/page.tsx:1483-1511) | ✅ |
| 从画布新建任务卡 | live 且有画布 | 任务卡与元素同事务创建 | `createCanvasTask` → `POST /canvases/{id}/work-items` (group/page.tsx:1378-1440) | ✅ |
| 关联已有任务卡 | live 且有未关联任务卡 | 创建 `work_item_card` 元素 | `linkExistingCanvasWorkItem` (group/page.tsx:1512-1569) | 🟡 通过 `createCanvasElement` 创建元素, 不调用 `entity-ref`; 与设计"关联"语义有偏差 |
| 添加 Frame | live 且名称非空 | 草稿追加 Frame | `stageCanvasFrame` (group/page.tsx:1589-1612) | ✅ |
| 删除 Frame | live 且选中 Frame | 草稿移除 | `removeCanvasFrame` (group/page.tsx:1613-1621) | ✅ |
| 选中元素加入 Frame | live 且选中 Frame | 草稿更新 `element_ids` | `addSelectedElementsToCanvasFrame` (group/page.tsx:1622-1649) | ✅ |
| 连接选中元素 | live 且恰好两个选中元素 | 草稿追加 `free` 连线 | `addCanvasVisualConnector` (group/page.tsx:1650-1671) | ✅ |
| 删除连线 | live 且选中连线 | 草稿移除 | `removeCanvasVisualConnector` (group/page.tsx:1672-1680) | ✅ |
| 修改 Frame 或连线属性 | live | 草稿更新 | `updateCanvasFrame` (group/page.tsx:1681-1691), `updateCanvasConnector` (1692-1702) | ✅ |
| 保存画布更改 | 草稿存在 | `PUT document` | `saveCanvasDocument` (group/page.tsx:2349-2420) | ✅ |
| 拖动元素 | 元素未锁定 | 位置保存 | `saveCanvasElementPosition` (group/page.tsx:1703-1774) → `PUT element` | ✅ |
| 修改宽高与旋转 | 选中未锁定元素 | 几何保存 | `saveCanvasElementGeometry` (group/page.tsx:1904) → `PUT element` | ✅ |
| 编辑便签文字 | 选中 text 或 sticky_note 且未锁定 | 内容保存 | `saveCanvasElementContent` (group/page.tsx:1837-1903) → `PUT element` | ✅ |
| 删除选中元素 | 选中且 live | 元素删除 | `deleteCanvasElements` (group/page.tsx:1775-1836) → `DELETE element` | ✅ |
| 双击任务卡 | 绑定当前 Worktree | 打开任务卡 | `openCanvasTask` (group/page.tsx:1314-1327) | ✅ |
| 新建便签、文字、形状、画笔、评论 (画布内) | live | 工具栏隐藏, 无法操作 | `readOnly=true` (group/page.tsx:3234); `onCreateElement` 存在但被门控 (CanvasView.tsx:612, 645) | ❌ 与 FR-06-12 目标不符 |
| 撤销 | live | 无效 | 撤销栈在 `onCreateElement` 存在时不工作 (CanvasView.tsx:134, 1220) | ❌ 与 FR-06-33 目标不符 |
| 图层上移下移、显隐、锁定 | live | 按钮可点但无效果 | `moveLayer`、`toggleLayerVisible`、`toggleLayerLock` 早返回 (CanvasView.tsx:131-150) | ❌ 与 FR-06-20、FR-06-21 目标不符 |
| 端口拖线创建连线 | live | 不启动 | CanvasView.tsx:221 早返回 | ❌ 与 FR-06-28 目标不符 |

数据: 来源为 `useWorktreeGroupProjection` 的授权投影 (`groupProjection.tsx:250-289`), 字段包括 `canvases` (id、版本、标题、视口、Frame、连线)、`canvas`、`elements` (id、`canvas_id`、kind、几何、`z_index`、`entity_ref`、`content`、`locked`、`hidden`、`version`)、`connectors`、`work_items` 与 `worktree`。连线与 Frame 来自 `document` 字段 (`groupProjection.tsx:397-400`)。

与文档设计的差距:
- FR-06-12、FR-06-13、FR-06-14、FR-06-15、FR-06-31、FR-06-32: 群组 live 无法创建新元素 (readOnly 门控)。
- FR-06-20、FR-06-21: 图层按钮在 live 中为空操作。
- FR-06-24: 已实现且有入口; FR-06-25: 服务端已实现, 前端未接入 entity-ref 设置与清除。
- FR-06-28: 端口拖线不写服务端。
- FR-06-33: 撤销不可用。
- DD-CANVAS-GROUP-03 (未授权 `EntityRef` 不可读写): 服务端 `validate_entity_ref` 满足 (canvas.rs:1853-1892); 前端 `openCanvasTask` 对 `refWorktreeId` 与绑定做校验 (group/page.tsx:1314-1327)。
- DD-CANVAS-GROUP-04 (Canvas 只持有 typed ref 与布局): `create_work_item_on_canvas` 在元素的 `content` 中写入 `title`、`item_type` 与 `status: "pending"` 副本 (canvas.rs:670-686), 与"不复制 lifecycle"冲突; 渲染时状态取自授权投影 (CanvasView.tsx:784-802), 副本未被读取。
- DD-CANVAS-GROUP-05 (插件 disable 不删除 Canvas): 不适用于本功能页面; 插件入口见 F17 与群组 App Registry。
- 引用失效的处理 (DD-CANVAS-001.md:§14.3 "保留 disabled/broken reference 并显示原因"): CanvasView 对找不到任务的 `work_item_card` 直接返回 `null` (CanvasView.tsx:786-787), 不显示断链原因。

### 4.7.2 /canvas

文件: `frontend/src/app/canvas/page.tsx`。布局由 `frontend/src/app/canvas/layout.tsx:9-17` 提供 (`AppShell fullBleed`)。

组件树: `CanvasPage` (canvas/page.tsx:37), 使用 `useNavStore` 与 `useStore`, 解析函数 `resolveCanvasId` (canvas-entry.ts:37-64), 创建函数 `createCanvasLocal` (canvas-entry.ts:79-103)。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 加载 | 首帧 (未水合) | 「正在按当前 Worktree / Lane 上下文解析画布…」 (`canvas-entry-loading`) | ✅ canvas/page.tsx:80-90 |
| 跳转中 | 解析到画布 | 「进入画布 …」 (`canvas-entry-redirecting`) | ✅ canvas/page.tsx:92-103 |
| 未接入 | 无上下文或无匹配 | 「画布未接入」与「新建画布」 (`canvas-entry-unbound`) | ✅ canvas/page.tsx:105-131 |
| 正常、错误、未授权 | 不适用 | 本地数据, 无授权与服务端错误 | ❌ 数据源为本地 store, 不经服务端 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新建画布 | 未接入 | 本地创建并跳转 `/canvas/{id}` | `createCanvasLocal` 写入 zustand (canvas-entry.ts:79-103) | 🟡 仅本地持久化, 不调用服务端 |

数据: `selectedWorktreeId` 与 `selectedLaneId` 来自 `navStore`; `worktrees`、`lanes`、`canvases` 来自 `store` (canvas/page.tsx:37-41)。

与文档设计的差距:
- FR-06-01: 解析正确, 但数据源为本地, 与服务端授权画布不一致 (见 §6 G-13)。
- 页面头部注释与 SRS 纠正注记 (SRS-CANVAS-001.md:4-6) 所述"直进 canvas-001 三标签"不一致; 以代码为准, 当前行为为按上下文解析。

### 4.7.3 /canvas/[id]

文件: `frontend/src/app/canvas/[id]/page.tsx`。布局由 `frontend/src/app/canvas/layout.tsx:9-17` 提供 (`AppShell fullBleed`)。

布局:
- 顶栏与侧栏由 `AppShell` 提供。
- 页内自上而下: 范围条 (`canvas-worktree-context`, canvas/[id]/page.tsx:265-316)、范围提示 (`canvas-entry-scope-notice`, :318-326)、子 tab 栏 (`canvas-merged-tabs`, :328-343)、内容区 (`main`, :345-348)。

组件树: `CanvasDetailPage` (canvas/[id]/page.tsx:52) → `CanvasScopeSelector` (`frontend/src/components/canvas/CanvasScopeSelector.tsx`) → 当前 tab 的内容: 「画布」为 `CanvasView` (canvas/[id]/page.tsx:170-203, `readOnly={false}`), 「Agent 画布」为 `AgentViewPage` (:214-218), 「关系图」为 `RelationshipView` (:219-245), 「流程」为 `WorkflowBuilder` (:246-251), 「Git 画布」为 `GitCanvas` (:252-258)。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 路由 id 命中本地画布 | 「画布」tab 显示 `CanvasView` | ✅ canvas/[id]/page.tsx:73, 166-203 |
| 未接入 | 路由 id 无匹配 | 「画布未接入 — 找不到 {id} 对应的画布」与「新建画布」 | ✅ canvas/[id]/page.tsx:205-225 |
| 加载、错误、未授权 | 不适用 | 无服务端请求 | ❌ 数据源为本地 store |
| 禁用 | 数据未按 Worktree 隔离 | 标签「数据未按 Worktree 隔离 (画布投影待接入)」 | ✅ canvas/[id]/page.tsx:293-303 显式披露 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换子 tab | 无 | 切换内容 | `setActiveTab` (canvas/[id]/page.tsx:328-343) | ✅ |
| `?tab=` 深链 | 参数在映射中 | 切换到对应 tab (board→画布, agent, relation, flow, git) | canvas/[id]/page.tsx:37-50 | ✅ |
| `?worktree=` 深链 | Worktree 存在于 store | 同步到 navStore | canvas/[id]/page.tsx:110-122 | ✅ |
| 画布内新增便签、形状、文字 | 写模式 | 写入本地 store | `onCreateElement` → `addCanvasElement` (canvas/[id]/page.tsx:170-190) | 🟡 仅本地, 不调用服务端 |
| BI、Plugins、Workflow 入口 | 任意 | 仅显示范围提示; Workflow 切换到「流程」 tab | `openScopedEntry` (canvas/[id]/page.tsx:150-170) | 🟡 TODO 标注后端调用未接入 |
| 新建画布 (未接入时) | 路由 id 无匹配 | 以该 id 写入本地 store | `createCanvasLocal` (canvas/[id]/page.tsx:215-222) | 🟡 仅本地 |

数据: 本地 store 的 `canvases`、`canvasElements`、`canvasConnectors`、`workItems`、`worktrees`、`projects` (canvas/[id]/page.tsx:60-72); 种子数据含 `canvas-001` (`frontend/src/lib/seed.ts:365` 起)。

与文档设计的差距:
- FR-06-02 满足; FR-06-49 子 tab 挂载存在, 文件头注释过期 (canvas/[id]/page.tsx:9-13 仍写三标签)。
- FR-06-12 与 FR-06-33: 本地写入的 `onCreateElement` 使撤销栈不工作 (CanvasView.tsx:134), 本地创建可达。
- DD-CANVAS-GROUP-02 (Worktree 画布与 Project 图使用不同对象): `/canvas/[id]` 的「关系图」tab 链接到 `/canvas/worktree-graph` (canvas/[id]/page.tsx:224-233), 两者以 `Link` 互通, 对象不同 (SRS-CANVAS-001.md:176-177)。

### 4.7.4 /collaboration

文件: `frontend/src/app/collaboration/page.tsx`。布局由 AppShell 提供; 导航条目见 `frontend/src/lib/nav/registry.ts:645-652`。

组件树: `CollaborationPage` (collaboration/page.tsx:14) 读取 `canvases`、`canvasElements`、`canvasConnectors`、`presenceCursors`、`whiteboards`、`worktrees`、`agentSessions`、`feedbacks` (collaboration/page.tsx:16-23)。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 本地 store 有画布 | 「Infinite Canvases」列表, 每项链接到 `/canvas/{id}` (collaboration/page.tsx:66-72) | ✅ |
| 空 | 本地 store 无画布 | 列表为空 | ❓ 未核实空态文案; 确认方法: 阅读 collaboration/page.tsx:66-104 的空分支 |
| 加载、错误、未授权 | 不适用 | 本地数据 | ❌ |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新建画布 (`New Canvas`) | 无 | 本地新增 | collaboration/page.tsx:61 (按钮) | 🟡 本地 store |
| 打开画布 | 列表项 | 跳转 `/canvas/{id}` | collaboration/page.tsx:69 | ✅ |

数据: 本地 store; 旧 Whiteboard 区块标注为 legacy (collaboration/page.tsx:54, 105-106)。

与文档设计的差距:
- FR-06-48: 列表与新建为本地数据, 不经群组授权投影, 与 DD-CANVAS-GROUP-03 不一致 (未授权 Worktree 的画布会因数据源为本地而不被过滤)。

---

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 群组画布应用 (挂载与交互) | frontend/src/app/worktree/[id]/group/page.tsx | 🟡 | 挂载 :3126-3256; `readOnly` 门控 :3234; `onCreateElement` :3242 |
| 群组投影与订阅装配 | frontend/src/lib/group/groupProjection.tsx | ✅ | 模式 :161-164; 订阅 :216-237; 数据加载 :250-289 |
| 群组 API 客户端 | frontend/src/lib/group/worktreeGroupApi.ts | ✅ | 画布方法 :1182-1360; 唤醒与游标 :1291-1576 |
| 共享画布渲染组件 | frontend/src/components/CanvasView.tsx | 🟡 | 1482 行; 属性 :55-107; 工具栏 :1080-1260; 只读门控 :221, 612, 645, 1083, 1220 |
| /canvas 上下文入口 | frontend/src/app/canvas/page.tsx | ✅ | 解析 :37-47; 新建 :51-71; 未接入 :114-127 |
| 上下文解析与本地建画布 | frontend/src/lib/canvas-entry.ts | 🟡 | 解析 :37-64; 本地建画布 :79-103 (本地通道) |
| /canvas/[id] 画布页 | frontend/src/app/canvas/[id]/page.tsx | 🟡 | 子 tab :54, 158-258; 本地写入 :170-190 |
| /collaboration 画布列表 | frontend/src/app/collaboration/page.tsx | 🟡 | 列表 :66-72 (本地数据) |
| 本地 store 画布 | frontend/src/lib/store.ts | 🟡 | 状态 :170-172 |
| 画布类型 | frontend/src/types/ids.ts | ✅ | 类型定义集中于该文件 |
| 服务端画布 API | crates/star-api-rest/src/group_api/canvas.rs | ✅ | 路由 :403-436; 处理函数 :439-2250; 测试 :2253 起 |
| 服务端事件流 | crates/star-api-rest/src/group_api/canvas_event_stream.rs | ✅ | 路由 :147-152; 常量 :71-81; 监听 :289-305 |
| 唤醒监听启动 | crates/star-api-rest/src/main.rs | ✅ | :106-108 |
| 画布迁移 (群组) | db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql | ✅ | 表与约束 :20-150; RLS :167-269 |
| 画布文档迁移 | db/migrations/2026-09-29-worktree-group-phase-3c-canvas-document.sql | ✅ | 文档列与形状约束 :4-49 |
| 出站迁移 | db/migrations/2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql; 2026-10-05-worktree-group-canvas-stream-offset.sql | ✅ | 游标索引 :12; stream_offset :13 |
| 唤醒触发器迁移 | db/migrations/2026-10-28-canvas-outbox-wakeup.sql | ✅ | 函数与触发器 :16-35 |
| 独立服务骨架 | crates/canvas-engine, crates/domain-canvas, crates/canvas-realtime | 🟡 | 仅健康与占位路由 |
| 协作领域骨架 | crates/canvas-collab | ⬜ | 仅骨架 |
| 设计参考稿 | docs/prototypes/2 · Branch 工作区 · 统一画布-html/ | 🟡 | 参考稿, 非生产代码 (README 1-21) |

---

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| G-01 群组 live 挂载 `readOnly={mode === "live"}` (group/page.tsx:3234) 同时 `onCreateElement` 存在 (group/page.tsx:3242), 创建类工具与撤销、图层不可用 | 群组画布无法新增便签、文字、形状、画笔、评论; 语义与"只读"相反 | 以权限决定只读 (服务端 scope 已决定), 以 `onCreateElement` 是否存在决定写模式; 为 live 挂载补组件测试 |
| G-02 撤销栈与快捷键绑定在 `onCreateElement` 未传入时 (CanvasView.tsx:134, 473, 1220) | 群组 live 与 `/canvas/[id]` 均不可撤销 | 解耦撤销栈与创建回调; 撤销基于服务端版本或 CRDT 二选一 (见 G-06) |
| G-03 `path` 元素未被服务端接受 (phase-3:53-56; canvas.rs:2168-2189), 前端仍会创建 (CanvasView.tsx:298-303) | 画笔数据在群组画布中无法保存 | 将 `path` 纳入迁移 CHECK 与校验, 或在 live 模式隐藏画笔 |
| G-04 `EntityRef` 类型不一致: 前端六值 (types/ids.ts), 服务端二值 (canvas.rs:2213-2228) | 前端可构造被后端拒绝的引用 | 以 `work_item` 与 `worktree` 收敛前端类型; 其余四值另行设计 |
| G-05 元素 kind 词表不一致: 前端含 `path`, 服务端与迁移 10 值 | 与 G-03 同源 | 与 G-03 同步修订 |
| G-06 设计写 WebSocket 推送与 Yjs CRDT (basic-design.md §16 与 DD-CANVAS-001.md 的 Realtime 段); 实现为 SSE 唤醒加游标追赶加版本拒绝 (canvas_event_stream.rs:147-152; worktreeGroupApi.ts:1397-1486) | 并发编辑依赖冲突后重试, 无自动合并 | 以现实现为当前基线; CRDT 选型作为独立决策单列 |
| G-07 实时光标与 presence 缺失 (FR-06-40) | 多人编辑无位置感知 | 在 SSE 通道增加 presence 事件与过期表, 或独立通道 |
| G-08 作用域: basic-design.md §16 描述 Engineering Run 作用域的 Canvas 与多画布投影 (`/api/v1/engineering-runs/...` 设计); 服务端路由仅 Worktree 作用域 (canvas.rs:403-436), 代码库中未找到 run 作用域画布路由 | 跨 Worktree 的画布无法表达; 设计与代码对象不一致 | 以 Worktree 作用域为当前基线; 若需 Run 作用域, 新增路由并迁移数据, 并回写设计 |
| G-09 PNG 导出与分享缺失; `html2canvas` 已声明 (frontend/package.json:35) 但未引用 | 功能缺口与依赖冗余 | 实现导出与分享, 或移除依赖与对应规格 |
| G-10 e2e 规格过期: `frontend/e2e/canvas-view.spec.ts`、`canvas-share-export.spec.ts`、`worktree-canvas.spec.ts` 中断言的入口与控件需回读核对 | 测试失去守护作用 | 逐条回读核对后改写到 `/canvas/[id]` 与群组页, 或删除过期规格 |
| G-11 `frame-select` 工具为占位 (CanvasView.tsx:1154); 群组 Frame 操作在页面侧 (group/page.tsx:1589-1691) | Frame 创建入口分裂 | 统一到 CanvasView 工具栏 |
| G-12 「关联已有任务卡」调用 `createCanvasElement` 而非 `entity-ref` 接口 (group/page.tsx:1512-1569; worktreeGroupApi.ts:1194-1209) | 关联语义与服务端 `set_entity_ref` 能力不一致; 同一任务卡可能出现多个元素 | 前端接入 `set_entity_ref`/`clear_entity_ref` 客户端方法, 关联改走引用接口 |
| G-13 两条画布数据通道并存: 本地 store (canvas-entry.ts:79-103; store.ts:170-172) 与服务端 (worktreeGroupApi.ts:1182-1260) | `/canvas` 与 `/canvas/[id]` 显示本地数据, 群组画布显示服务端数据 | 决定 `/canvas/[id]` 迁移到服务端或退役本地通道 |
| G-14 `basic-design.md` §16 对账段称现行实现为 `CanvasOutboxPoller` 轮询; 当前前端的 `CanvasOutboxCatchUp` 与唤醒多路复用器已替代 (worktreeGroupApi.ts:1397, 1487) | 设计对账失真 | 修订 basic-design §16 的对账段落, 指向唤醒与追赶实现 |
| G-15 `canvas-engine`、`domain-canvas`、`canvas-realtime` 仅有骨架路由 | 部署与功能无对应 | 决定保留或移出部署; 群组 live 不依赖这些服务 |
| G-16 设计要求 CanvasToolbar、CanvasSidebar、CanvasMinimap 独立文件 (BD-CANVAS-001.md §3.1), 实现内联于 CanvasView.tsx (1482 行) | 单文件过大, 难以测试 | 拆分为独立模块 |
| G-17 无视口裁剪与性能基准 (FR-06-11, FR-06-51) | 大画布可能掉帧 | 增加裁剪与基准测试 |
| G-18 元素内容保存了 `title`、`item_type` 与 `status: "pending"` 副本 (canvas.rs:670-686) | 与"Canvas 不复制任务生命周期"冲突; 副本可能与授权投影不一致 | 元素内容仅保存引用与布局; 任务属性一律从授权投影读取 |
| G-19 引用失效时 `work_item_card` 静默不渲染 (CanvasView.tsx:786-787) | 断链无原因提示, 与 DD-CANVAS-001.md §14.3 不符 | 渲染断链占位并显示原因 |
| G-20 群组 live 无 `CanvasEventWakeHub` 之外的 presence 或评论线程; 元素只读 (G-01) 与撤销 (G-02) 叠加, 群组画布实际为"看板式浏览 + 拖动 + 删除 + 文档保存" | 群组画布的可用功能弱于设计 | 随 G-01、G-02、G-07 一并规划 |

---

## 7. 验收与测试

| 类别 | 位置 | 覆盖 | 说明 |
|---|---|---|---|
| 组件测试 | frontend/src/components/CanvasView.test.tsx | 画布组件用例 (画笔按钮默认显示与 readOnly 隐藏; 输入输出端口渲染与拖线) | 存在 (用例数与名称需回读核对) |
| 入口测试 | frontend/src/lib/canvas-entry.test.ts | 上下文解析与本地建画布 | 存在 |
| 群组 API 测试 | frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts | 画布接口与唤醒流 | 存在 |
| 投影测试 | frontend/src/lib/group/__tests__/groupProjection.test.tsx | 投影与订阅装配 (含 canvases 数据形状) | 存在 |
| Store 测试 | frontend/src/lib/store.test.ts | 画布相关状态 | 存在 |
| 服务端单元测试 | crates/star-api-rest/src/group_api/canvas.rs (`#[cfg(test)]` 起于 :2253) | 游标与分页边界等 | 存在 |
| 服务端 PostgreSQL 集成测试 | crates/star-api-rest/tests/canvas_event_stream_pg.rs; crates/star-api-rest/tests/canvas_device_auth_pg.rs | 事件流与设备授权 | 依赖 PostgreSQL; 本文档未执行 |
| 迁移验证 | db/migrations/2026-10-05-worktree-group-canvas-stream-offset.sql 注释 (:4-12) | 在一次性 PostgreSQL 上可重复应用、RLS、追加只写拒绝、advisory lock 与超时 fail-closed | 声明为已验证, 本文档未复现 |
| E2E | frontend/e2e/canvas-view.spec.ts; frontend/e2e/canvas-share-export.spec.ts; frontend/e2e/worktree-canvas.spec.ts | 需回读确认覆盖的入口 | 存在; 与当前入口的一致性未核实 (G-10) |

未覆盖的验证点:
- 群组 live 挂载的创建门控 (G-01) 与撤销不可达 (G-02) 无组件测试。
- 元素版本冲突在界面上的恢复路径 (`version_conflict`) 无测试。
- 文档版本冲突的「放弃草稿并加载最新版本」路径无测试。
- 唤醒流断线重连后的游标追赶无端到端测试。
- 迁移在空库上的执行未在本文档内复现。
- 缺失 scope 的负向测试与 `entity-ref` 接口的测试。

本文档未运行上述测试套件; 测试存在性依据测试文件与用例名称, 未逐条阅读用例正文的部分已在表中标注。

---

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-CANVAS-001.md (v1.4) | 群组边界与 `EntityRef(type, id, worktree_id)` (§1.6, :168-176); 三核心划分 (§1.3.1, :103-109); 群组验收项 (§8.1.1, :659-670); 现状纠正注记 (:4-6) | 纠正注记称 `/canvas` 直进三标签; 代码 (canvas/page.tsx:37-127) 按上下文解析或未接入; §1.3.1 的 132 项为三核心索引, 与 F06 边界不同 | 采用为 F06 范围与 EntityRef 权威; `/canvas` 入口行为以代码为准; 三核心子需求不纳入 F06 |
| docs/design/BD-CANVAS-001.md (v0.2) | 群组上下文与 EntityRef (§1.3, :180-191); 同级应用与聊天栏 (:188); 组件拆分要求 (§3.1, :277-283); W/T/M 汇总 (§4.1) | §3.1 要求的组件拆分与代码不符 (G-16); 纠正注记与代码的 `/canvas` 行为不一致 | 采用 §1.3 作为群组边界; 组件拆分记为 G-16 |
| docs/design/DD-CANVAS-001.md (v0.2) | §14 群组画布详细契约 (:917-1009): 对象与上下文 (§14.1)、跨应用协议 (§14.2)、W/T/M (§14.3)、接口与失败 (§14.4)、验收 DD-CANVAS-GROUP-01 至 05 (§14.5); 群组 DDL 参考 (§9.1, :565-589) | §14.3 要求保留 broken reference 并显示原因 (代码未实现, G-19); §14.2 的 Canvas→Domain 命令路径部分实现 (创建与关联 via canvas.rs:571); 类清单与代码模块不对应 | 采用 §14 作为群组契约权威; 类清单不作为实现依据 |
| docs/requirements/SRS-STAR-CANVAS-001.md | 无 (文件头为 SUPERSEDED 2026-09-07, :3-27) | 整册被 SRS-STAR-CANVAS-GAME-001 替代 | 🗑 不作为设计依据 |
| docs/design/BD-STAR-CANVAS-001.md | 无 (文件头为 SUPERSEDED 2026-09-07, :3-15) | 整册被 BD-STAR-CANVAS-GAME-001 替代 | 🗑 不作为设计依据 |
| docs/design/DD-STAR-CANVAS-001.md | 无 (文件头为 SUPERSEDED 2026-09-07, :3-15) | 整册被 DD-STAR-CANVAS-GAME-001 替代; Yjs 方案未被采用 (frontend/package.json 无 yjs 依赖, FR-06-41) | 🗑 不作为设计依据 |
| docs/frontend-canvas-design.md (v0.1, 2026-08-26; 现状纠正注记 2026-11-15) | 元素与连线模型 (§2, :130-185 附近); 联动规则 (§4); 快捷键 (§5.1); 联动方案 B 的 `collaboration` 强化 (§1.2) | §1.2 称后端无改动, 而群组 API 已在 star-api-rest 中; §5.1 快捷键与代码 (CanvasView.tsx:512-525) 基本一致; 页头注释指向 SRS 现状纠正 | 以代码为准; 方案 B 被群组 API 取代 (服务端 EntityRef 校验与版本控制无法由客户端单独完成) |
| docs/prototypes/2 · Branch 工作区 · 统一画布-html/README.md 与 Canvas.dc.html | "统一画布" 命名; 视觉与布局参考 | README 明确为参考稿, 非生产代码 (README 1-21) | 采用为命名与视觉参考; 不作为实现证据 |
| docs/basic-design.md §16 (4229 起) | Worktree 群组基本设计; 画布 `expected_version` 与 `Idempotency-Key` (:4373-4374 附近); 原子建卡事务 (:4375 附近); 事件与 WebSocket 推送与对账 (:4399-4409 附近) | 作用域描述为 Engineering Run (G-08); 对账段描述为轮询 (G-14) | 以代码为准作为实现基线; 作用域与对账差异记为 G-08 与 G-14 |
| docs/features/F06-unified-canvas.md (基线 1643559f 的既有版本) | 结构与证据线索 | 基线较旧, 行号与部分结论已变 | 仅作核对线索; 所有结论在 637ed19d 重新读取 |
| docs/deployment/star-canvas-services-deploy.md | 三个 Canvas 后端服务与入口的部署清单 | 服务为骨架 (G-15) | 不作为 live 画布实现依据 |
| docs/requirements/SRS-STAR-CANVAS-GAME-001.md | 游戏方向 | 已退役; 游戏叠加层代码 (frontend/src/components/canvas-game/) 不属 F06 | 不纳入裁决 |
| docs/requirements/SRS-WORKTREE-CANVAS-001.md、docs/design/BD-WORKTREE-CANVAS-001.md | Project 级 Worktree Overview Graph | 另一对象 (SRS-CANVAS-001.md:176-177) | 不属 F06 范围; 仅识别路径 |
| SRS-CANVAS-AGENT-001、SRS-CANVAS-GAMIFY-001、SRS-CANVAS-WORKFLOW-001 及其 BD、DD | 三核心子需求 | 属其他 feature (F08 agent-graph-canvas、F10 gamified-canvas 等) | 不纳入裁决 |
| 代码源: CanvasView.tsx、canvas 三个页面、canvas-entry.ts、group/page.tsx、groupProjection.tsx、worktreeGroupApi.ts、store.ts、types/ids.ts、collaboration/page.tsx、canvas.rs、canvas_event_stream.rs、group_api.rs、main.rs、画布迁移与唤醒迁移 | 实现依据 | 无 | 代码为现状权威 |

---
