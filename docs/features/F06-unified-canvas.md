# F06 统一画布

> 状态摘要: 群组内的 live 画布已由服务端权威实现 (Worktree 作用域的画布、元素、文档, 含版本号与幂等控制、事件流), 支持元素拖动与删除、work-item 卡与 Worktree 节点渲染、连线与 Frame 随文档保存; 但群组 live 挂载把 `readOnly` 置为真, 新增工具被门控关闭, 撤销重做、PNG 导出、分享、实时光标与 CRDT 协作未实现。/canvas 与 /canvas/[id] 使用本地 zustand 画布, 与服务端画布并行存在, 不经服务端。

> 代码基线: HEAD 1643559f
> 核对方法: 路径与符号 grep 加工作树文件读取, 每条实现结论回读对应行; 本文引用的文件与 HEAD 1643559f 无差异 (路径级 git status 为空); 核对日期 2026-11-17。

---

## 1. 目标与边界

### 1.1 目标

统一画布是 Worktree 群组 (Branch 工作区) 内的一块无限画布, 把同一 Worktree 下的 work-item 卡、Worktree 节点、便签、文本、形状、连线与 Frame 放在同一空间中。画布元素对任务事实只持有类型化引用 `EntityRef` (类型为 `work_item` 或 `worktree`); 布局与画布元数据归画布所有, 任务状态归 WorkItem 域, 画布不维护第二份任务状态机 (SRS-CANVAS-001.md:170-172; BD-CANVAS-001.md:184-185; DD-CANVAS-001.md:565-589)。"统一"一词取自设计稿标题 (docs/prototypes/2 · Branch 工作区 · 统一画布-html/Canvas.dc.html 的 `<title>统一画布</title>`)。

画布与任务看板、Task Card、插件同级挂在群组应用导航中, 共用群组底部聊天栏, 画布只把当前 Worktree 与选中元素作为上下文传递 (BD-CANVAS-001.md:188)。

### 1.2 范围

范围内:
- 群组「无限画布」应用及其投影、挂载与交互 (frontend/src/app/worktree/[id]/group/page.tsx:634, 3126-3252)。
- 共享渲染组件 CanvasView (frontend/src/components/CanvasView.tsx)。
- /canvas 上下文入口 (frontend/src/app/canvas/page.tsx) 与 /canvas/[id] 页的「画布」tab (frontend/src/app/canvas/[id]/page.tsx:158-169)。
- 服务端 Worktree 作用域画布 API 与事件流 (crates/star-api-rest/src/group_api/canvas.rs, crates/star-api-rest/src/group_api/canvas_event_stream.rs)。
- 画布持久化迁移 (db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql, db/migrations/2026-09-29-worktree-group-phase-3c-canvas-document.sql)。

范围外:
- /canvas/worktree-graph, 即 Project 级 Worktree Overview Graph, 另一对象 (SRS-CANVAS-001.md:176; docs/basic-design.md:4260)。
- 画布游戏叠加层 (frontend/src/components/canvas-game/), 其需求文档已退役 (docs/requirements/SRS-STAR-CANVAS-GAME-001.md:3)。
- /canvas/[id] 页中 Agent 画布、关系图、流程、Git 画布四个子 tab 的内部能力, 各自归属其 feature; F06 只记录挂载 (frontend/src/app/canvas/[id]/page.tsx:214-245)。
- SRS-CANVAS-AGENT-001、SRS-CANVAS-GAMIFY-001、SRS-CANVAS-WORKFLOW-001 三核心子需求 (SRS-CANVAS-001.md:103-108)。
- 外部 SaaS 连接器 (SRS-CANVAS-001.md:140)。

---

## 2. 用户与场景

| 角色 | 主要场景 | 证据 |
|---|---|---|
| 群组成员 (编辑) | 在当前 Worktree 画布上拖动、删除元素; 把已有 work-item 或 Worktree 关联为卡片 | group/page.tsx:1880, 1775, 1550; canvas.rs:1208, 1600, 1371 |
| 群组成员 (建卡) | 在画布上新建 work-item, 同时关联到 Worktree 与画布 | group/page.tsx:1414; canvas.rs:571 |
| 群组成员 (入口) | 从 /canvas 按当前 Worktree 或 Lane 进入画布; 无匹配时新建 | canvas/page.tsx:47-78, 105-131 |
| 观察者 | 查看 work-item 卡与 Worktree 节点的状态色码; 通过 ?highlight 深链定位元素 | CanvasView.tsx:784-824, 284-291 |
| 单机草图 | 在 /canvas/[id] 用本地画布起草便签与形状, 不经服务端 | canvas/[id]/page.tsx:163-204 |

---

## 3. 需求

优先级: P0 为必须 (SRS-CANVAS-001.md §4.4 P0 清单同级), P1、P2、P3 依次降低。编号 FR-06-NN。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-06-01 | /canvas 按当前 Worktree 或 Lane 解析画布; 无匹配时显示未接入状态并可新建 | P1 | ✅ | canvas/page.tsx:47-54, 60-78, 105-131; canvas-entry.ts:37-64 |
| FR-06-02 | /canvas/[id] 按路由 id 取画布; 找不到时显示未接入状态 | P1 | ✅ | canvas/[id]/page.tsx:73, 80, 208 |
| FR-06-03 | 群组导航提供「无限画布」应用, 仅该应用激活时加载画布投影 | P0 | ✅ | group/page.tsx:634, 995; groupProjection.tsx:218, 265-277 |
| FR-06-04 | live 模式显示授权投影; preview 模式不以 seed 冒充当前 Worktree 画布 | P0 | ✅ | groupProjection.tsx:161-164; group/page.tsx:3251 |
| FR-06-05 | live 模式下当前 Worktree 无画布时, 经幂等命令创建 | P0 | ✅ | group/page.tsx:1483, 3252; worktreeGroupApi.ts:1186; canvas.rs:773, 786 |
| FR-06-06 | 平移、滚轮与按钮缩放, 缩放范围 0.1 至 4 | P0 | ✅ | CanvasView.tsx:295, 400-403, 1198-1201 |
| FR-06-07 | Shift+1 适应全部, Shift+2 适应选中 | P1 | ✅ | CanvasView.tsx:491-510 |
| FR-06-08 | 视口、Frame 与连线以 canvas document 保存; 版本不符时拒绝 | P0 | ✅ | group/page.tsx:1570, 1589, 1650, 2393; canvas.rs:840, 892-893 |
| FR-06-09 | ?highlight 深链自动定位并高亮元素 | P1 | ✅ | CanvasView.tsx:284-291; group/page.tsx:3233 |
| FR-06-10 | 小地图显示视口范围 | P1 | ✅ | CanvasView.tsx:1397, 1454 |
| FR-06-11 | 视口裁剪渲染, 1K 元素保持 60fps | P1 | ⬜ | CanvasView.tsx 无裁剪实现 (检索无命中); 无性能基准 |
| FR-06-12 | 便签与文本元素创建 | P0 | 🟡 | /canvas/[id] 写模式可创建 (canvas/[id]/page.tsx:167, 170, 188; CanvasView.tsx:1083); 群组 live 挂载门控关闭 (group/page.tsx:3234, 3242; CanvasView.tsx:612, 1083, 1324); 服务端接受 sticky_note 与 text (canvas.rs:2178-2187) |
| FR-06-13 | 形状元素 (矩形、圆角、圆、菱形、三角) 创建与渲染 | P1 | 🟡 | 渲染 CanvasView.tsx:942; 形状菜单 CanvasView.tsx:1104-1111; 类型 types/ids.ts:807-846 (shape_kind); 服务端对 shape_kind 的校验 ❓; 群组 live 创建同 FR-06-12 门控 |
| FR-06-14 | 图片元素渲染 | P2 | 🟡 | 渲染分支 CanvasView.tsx:966; 创建入口未找到; 服务端接受 image (canvas.rs:2178-2187) |
| FR-06-15 | 嵌入 (embed) 元素渲染与创建 | P2 | ⬜ | 服务端接受 embed (canvas.rs:2178-2187; migration:53-56); CanvasView.tsx 无 embed 分支 |
| FR-06-16 | 元素拖动, 位置经 update_mode position 保存; 锁定元素不可拖动 | P0 | ✅ | CanvasView.tsx:329, 434; group/page.tsx:1880, 1957; canvas.rs:1208, 1245; 锁定 canvas.rs:1970 |
| FR-06-17 | 元素宽高与旋转编辑 (geometry 模式) | P1 | 🟡 | 服务端 Geometry 分支与 rotation 字段 (canvas.rs:319, 1245); 画布内缩放与旋转手柄未找到 ❓ |
| FR-06-18 | 元素内容编辑 (content 模式) | P1 | 🟡 | 服务端 Content 分支 (canvas.rs:1245); 文本内联编辑仅在非只读时启用 (CanvasView.tsx:547, 1284-1288) |
| FR-06-19 | 删除选中元素 (Delete、Backspace、工具栏) | P0 | ✅ | CanvasView.tsx:521-523, 1256; group/page.tsx:1775, 1810; canvas.rs:1600 |
| FR-06-20 | 元素层级上移与下移 (z_index) | P2 | 🟡 | 排序交换逻辑 CanvasView.tsx:136-142; UI 入口未找到 ❓ |
| FR-06-21 | 元素显示与隐藏切换 | P2 | 🟡 | 切换逻辑 CanvasView.tsx:148; 服务端 hidden 投影 canvas.rs:2157; UI 入口未找到 ❓ |
| FR-06-22 | work-item 卡: 显示状态色码 (StatusPill) 并可打开对应任务 | P0 | ✅ | CanvasView.tsx:784-802, 555-556; group/page.tsx:1314, 3237 |
| FR-06-23 | Worktree 节点: 显示 status 色码 | P0 | ✅ | CanvasView.tsx:808-824 |
| FR-06-24 | 在画布内新建 work-item, 并在同一事务中关联 Worktree 与画布 | P0 | 🟡 | 服务端 canvas.rs:571, 581, 604; 前端调用 group/page.tsx:1414; 画布工具栏入口未找到 ❓ |
| FR-06-25 | 把已有 work-item 或 Worktree 关联为画布元素 (entity_ref 设置与清除) | P0 | ✅ | canvas.rs:1371, 1511; group/page.tsx:1550 |
| FR-06-26 | entity_ref 必须属于同一 Worktree, 跨 Worktree 引用被拒绝 | P0 | ✅ | canvas.rs:1853-1893 (1863 entity_ref_worktree_mismatch; 1868 not_found) |
| FR-06-27 | 连线三种路由 (straight、orthogonal、curved) 与输入输出端口 | P0 | ✅ | CanvasView.tsx:1007-1028; 端口测试 CanvasView.test.tsx:104-174; 服务端路由校验 canvas.rs:998-1013 |
| FR-06-28 | 连线拖拽创建与持久化 | P0 | 🟡 | 端口拖线交互测试 CanvasView.test.tsx:149-174; 群组侧连线增删 group/page.tsx:1650, 1672; 群组 live 下拖线是否可用 ❓ |
| FR-06-29 | Frame 分区渲染 | P1 | ✅ | CanvasView.tsx:977-978; 服务端 frame 校验 canvas.rs:989 |
| FR-06-30 | Frame 创建与把选中元素加入 Frame | P1 | 🟡 | 群组侧 group/page.tsx:1589, 1622; 画布 frame-select 工具为占位 (CanvasView.tsx:1154, 标注"下一阶段") |
| FR-06-31 | 画笔自由绘制 (path 元素) | P2 | 🟡 | 写模式创建 CanvasView.tsx:298, 663; 渲染 CanvasView.tsx:735; 服务端 CHECK 与校验不接受 path (migration:53-56; canvas.rs:2178-2187) |
| FR-06-32 | 评论钉与评论工具 | P1 | 🟡 | 评论工具 CanvasView.tsx:310, 601; comment_pin 渲染 CanvasView.tsx:931; 评论线程持久化见 FR-06-47 |
| FR-06-33 | 撤销与重做 | P0 | 🟡 | 撤销栈与快捷键仅在 onCreateElement 未传入时生效 (CanvasView.tsx:124-134, 473, 1220); /canvas/[id] 与群组 live 均传入 onCreateElement (canvas/[id]/page.tsx:170; group/page.tsx:3242), 当前挂载下不可达 ❓ |
| FR-06-34 | 键盘工具切换 (V、H、P、S、T、N、L、E、C)、Delete、Escape | P1 | ✅ | CanvasView.tsx:512-525 |
| FR-06-35 | 多选与框选 | P0 | ❓ | 选区数组与多选删除存在 (CanvasView.tsx:1256); Shift 拖动为平移 (CanvasView.tsx:295), 与 Shift 多选约定冲突; 框选未核实 |
| FR-06-36 | PNG 导出 | P1 | ❓ | html2canvas 依赖声明 (frontend/package.json:35), frontend/src 无 import; 导出控件未找到; canvas-share-export.spec.ts:47 仍断言。确认方法: `git log -S "Export PNG" -- frontend/src` 查找移除提交 |
| FR-06-37 | 分享链接与复制画布地址 | P1 | ❓ | 分享控件未找到; canvas-share-export.spec.ts:36 仍断言。确认方法同 FR-06-36 |
| FR-06-38 | JSON 或 Miro 导入导出 | P2 | ⬜ | 画布范围内未找到导入导出实现 |
| FR-06-39 | 实时同步: 服务端事件推送与游标追赶 | P0 | 🟡 | SSE 路由 canvas_event_stream.rs:148-149; 唤醒订阅 worktreeGroupApi.ts:1291, 1329, 1487, 1576; 游标追赶 worktreeGroupApi.ts:1397; 装配 groupProjection.tsx:218。不是 CRDT, 不做字段级合并 |
| FR-06-40 | 实时光标与 presence | P1 | ⬜ | 群组页与 CanvasView 无 presence 实现; 画布迁移中无 presence 表 (migration 表清单: 20, 47, 86, 114, 132) |
| FR-06-41 | CRDT (Yjs) 协作与离线合并 | P1 | ⬜ | frontend/package.json 无 yjs 或 y-websocket 依赖; 冲突以 expected_version 拒绝处理 (canvas.rs:892-893, 1242) |
| FR-06-42 | 乐观并发: 文档与元素版本号不符时拒绝 | P0 | ✅ | canvas.rs:892-893, 1219-1220, 1242, 1382 |
| FR-06-43 | 幂等写: Idempotency-Key 与请求哈希 | P0 | ✅ | worktreeGroupApi.ts:1186-1260; canvas.rs:786, 863, 1120, 1224, 1392, 1527, 1616, 2230 |
| FR-06-44 | 画布变更事件历史查询 | P1 | ✅ | canvas.rs:426-432, 439, 485; migration canvas_group_outbox (132) |
| FR-06-45 | 作用域与 RLS: canvas:read、canvas:write 与 Worktree 授权 | P0 | ✅ | canvas.rs:446, 581 (其余 scope 调用见 4.5); 授权 canvas.rs:1756, 1767; 迁移 RLS 启用 migration:167-269 |
| FR-06-46 | 画布写入审计 | P1 | 🟡 | 审计表 migration:114; 写入 append_event canvas.rs:1923; 审计读取接口与 UI 未找到 |
| FR-06-47 | 画布评论线程、@ 提醒与多人评论 | P1 | ⬜ | 画布迁移中无评论表; 仅 comment_pin 渲染 (CanvasView.tsx:931) |
| FR-06-48 | /collaboration 画布列表与新建 | P2 | 🟡 | collaboration/page.tsx:17, 32, 54, 62-64; 数据源为本地 store (store.ts:170) |
| FR-06-49 | /canvas/[id] 子 tab 挂载 (画布、Agent 画布、关系图、流程、Git 画布) | P2 | 🟡 | canvas/[id]/page.tsx:54, 158-245; 头部注释仍写三标签 (canvas/[id]/page.tsx:27-29) |
| FR-06-50 | 演示模式 (Frame 作为幻灯片) | P3 | ⬜ | 画布目录与 CanvasView 检索无 presentation 实现 |
| FR-06-51 | 性能基准: 1K 元素与 50 协作者 | P1 | ❓ | 未见性能基准或压测; 确认方法: 前端 benchmark 与 PostgreSQL 负载测试 |

---

## 4. 设计

### 4.1 概念模型

- **群组画布 (Group Canvas)**: 以 `group_canvas_registry` 行表示, 按 Worktree 与 canvas_id 唯一标识当前版本; 同一 canvas_id 同一时刻仅一行 `is_current` (db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql:39-42)。同一 Worktree 下可有多条当前画布 (非唯一索引, :43-46)。
- **画布文档 (Canvas document)**: 视口 (viewport)、Frame 列表 (frames) 与可视连线 (connectors) 由 canvas document 保存, 与画布注册行共用版本 (db/migrations/2026-09-29-worktree-group-phase-3c-canvas-document.sql:4-12)。连线不是 WorkItem 关系, 仅是布局投影 (同文件 :12)。
- **画布元素 (Element)**: `canvas_elements_backend` 中的 SCD Type 2 行, 字段含 kind、几何、锁定与隐藏, 每次变更生成新版本 (:47-79)。
- **实体引用 (EntityRef)**: `canvas_entity_ref` 以 ref_type 与 ref_id 关联元素与 work-item 或 Worktree, 同样是 SCD Type 2 (:86-107)。
- **事件 (Outbox)**: `canvas_group_outbox` 记录画布变更, 客户端经游标读取与 SSE 唤醒同步 (:132; canvas_event_stream.rs:148-149)。
- **本地画布 (Local canvas)**: zustand store 中的 `canvases`、`canvasElements`、`canvasConnectors`, 与服务端无关 (store.ts:170-172; canvas-entry.ts:75-77)。

### 4.2 数据模型

| 实体 | 存储 | 关键字段与约束 | 分类 (守门 #13) |
|---|---|---|---|
| group_canvas_registry | canvas.group_canvas_registry | tenant_id、worktree_id、canvas_id、version (CHECK > 0, :25)、title (长度 1 至 200, :26)、valid_from/valid_to、is_current (:34-35) | Master, SCD Type 2 (3c:11 声明) |
| canvas_elements_backend | canvas.canvas_elements_backend | kind (CHECK 10 值, :53-56)、x、y、width 与 height (0 < 值 ≤ 10000, :58-59)、version (:65)、valid_from/valid_to (:73-74) | Master, SCD Type 2 ❓ (未在迁移注释中核实) |
| canvas_entity_ref | canvas.canvas_entity_ref | ref_type (CHECK work_item 或 worktree, :92)、version (:94)、有效期 (:101-102) | Master, SCD Type 2 ❓ |
| canvas_group_audit | canvas.canvas_group_audit | 审计行, 迁移位于 :114 | Transaction ❓ (分类未在迁移中核实) |
| canvas_group_outbox | canvas.canvas_group_outbox | schema_version、aggregate_version (:140-141) | Transaction ❓ |
| Canvas (前端类型) | types/ids.ts:769 | id、ref_kind (work_item、worktree、project、free)、ref_id、viewport、frames | 参考类型 |
| CanvasElement (前端类型) | types/ids.ts:807 | kind、x、y、width、height、rotation、z_index、entity_ref、content (含 path_data、shape_kind 等)、locked、hidden | 参考类型 |
| CanvasConnector (前端类型) | types/ids.ts:854 | kind、from_element_id、to_element_id、routing、端口、颜色、宽度、label | 参考类型 |
| CanvasFrame (前端类型) | types/ids.ts:755 | 矩形、element_ids、is_slide、order | 参考类型 |
| CanvasEntityRef (前端类型) | types/ids.ts:800-804 | ref_type 六值 (work_item、worktree、agent_session、automation、pull_request、comment) | 与服务端不一致, 见 6 |

词表差异: 前端 CanvasElementKind 共 11 值 (含 path, types/ids.ts:786-796); 迁移与服务端校验共 10 值, 不含 path (migration:53-56; canvas.rs:2178-2187)。服务端 entity_ref 仅接受 work_item 与 worktree (canvas.rs:1382, 2213-2228)。

### 4.3 接口

服务端路由 (crates/star-api-rest/src/group_api/canvas.rs:403-436, 合并于 group_api.rs:814):

| 方法 | 路径 | 作用域 | 处理函数 | 状态 |
|---|---|---|---|---|
| GET、POST | /api/v1/worktrees/{worktree_id}/canvases | canvas:read / canvas:write | list_canvases (734)、create_canvas (773) | ✅ |
| GET、POST | /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements | read / write | list_elements (1061)、create_element (1095) | ✅ |
| PUT、DELETE | .../canvases/{canvas_id}/elements/{element_id} | write | update_element (1208)、delete_element (1600) | ✅ |
| PUT、DELETE | .../elements/{element_id}/entity-ref | write | set_entity_ref (1371)、clear_entity_ref (1511) | ✅ |
| PUT | .../canvases/{canvas_id}/document | write | update_document (840) | ✅ |
| GET | .../canvases/{canvas_id}/events | read | list_canvas_events (439) | ✅ |
| GET | /api/v1/worktrees/{worktree_id}/canvas-events | read | list_worktree_canvas_events (485) | ✅ |
| POST | .../canvases/{canvas_id}/work-items | write | create_work_item_on_canvas (571) | ✅ |
| GET (SSE) | /api/v1/worktrees/{worktree_id}/canvas-events/stream | ❓ 作用域未在该路由核实 | canvas_event_stream.rs:148-149 | ✅ |

UpdateElementBody 字段: `expected_version`、`update_mode` (枚举 Position、Geometry、Content, canvas.rs:304-314), 按模式读取 x、y、宽高、rotation 或 content (canvas.rs:1245-1250)。

前端客户端 (frontend/src/lib/group/worktreeGroupApi.ts): `listCanvases` 1182、`createCanvas` 1186、`createCanvasElement` 1194、`updateCanvasElement` 1210、`deleteCanvasElement` 1227、`updateCanvasDocument` 1244、`listCanvasElements` 1260、`listCanvasEvents` 1267、`listWorktreeCanvasEvents` 1283、`openWorktreeCanvasWakeStream` 1291、`subscribeWorktreeCanvasWake` 1329、`createWorkItemOnCanvas` 1345。同文件另有 `CanvasOutboxCatchUp` (1397)、`WorktreeCanvasWakeMultiplexer` (1487)、`CanvasOutboxWakeSubscriber` (1576)。

其它服务端路由 (不在群组 live 路径上): canvas-realtime `/ws` 与健康路由 (crates/canvas-realtime/src/lib.rs:95-98); domain-canvas `/api/v1/canvas` 占位 (crates/domain-canvas/src/lib.rs:84-87); canvas-engine `/api/v1/services` (crates/canvas-engine/src/lib.rs:88-92)。

### 4.4 状态机与关键流程

元素版本 (服务端):

```
create (version 1) → update (expected_version 必须等于当前版本, canvas.rs:1242)
                     → 旧版本 close_element_version (canvas.rs:1997) → bump_element_version (canvas.rs:2024)
                   → delete (写审计与事件, canvas.rs:1600, 1923)
```

画布入口状态 (canvas/page.tsx): 挂载前 loading (canvas/page.tsx:80-90) → 按上下文解析 redirecting (:92-103) 或 unbound 并提供新建 (:105-131)。

文档保存 (群组 live): 视口变化经 stageCanvasViewport (group/page.tsx:1570) 写入待保存草稿 → updateCanvasDocument (group/page.tsx:2393) 携带 expected_version 调用 PUT → 服务端版本不符返回 `canvas_document_version_conflict` (canvas.rs:892-893)。客户端对冲突的恢复路径未核实 ❓。

元素拖动 (群组 live): CanvasView 释放时经 onElementPositionChange (CanvasView.tsx:329) → saveCanvasElementPosition 调用 updateCanvasElement (group/page.tsx:1880, 1957) → 服务端 Position 分支 (canvas.rs:1245) → 追加事件 (canvas.rs:1923)。

建卡 (群组 live): createWorkItemOnCanvas (group/page.tsx:1414) → canvas.rs:571 (写作用域 581, 幂等键 604) → 原子创建 work-item 与画布卡 (basic-design.md:4375 声明; 代码路径 canvas.rs:571-640 之内的事务实现未逐行核实 ❓)。

同步: 游标追赶 CanvasOutboxCatchUp 按 cursor 读取 listWorktreeCanvasEvents (worktreeGroupApi.ts:1283, 1397); 唤醒多路复用器经 SSE 订阅 (worktreeGroupApi.ts:1487, 1291) 断线重连; 装配在 groupProjection.tsx:218。服务端 LISTEN/NOTIFY 通道 `star_canvas_outbox_wake` 经 PgListener 转发 (canvas_event_stream.rs:61, 71, 287-294)。

### 4.5 权限与安全

- **作用域**: 读取 canvas:read (canvas.rs:446, 492, 740, 1067; 1832 require_scope); 写入 canvas:write (canvas.rs:581, 781, 848, 1103, 1216, 1379, 1519, 1608)。
- **字段级**: work-item 详情仅在持有 work-item:read 时返回 (has_scope 调用 canvas.rs:726, 1085; 投影 canvas.rs:2161)。
- **Worktree 绑定**: request_worktree (canvas.rs:1756) 与 authorize_group_scope (canvas.rs:1767) 基于设备上下文 (AuthenticatedDeviceContext) 校验。
- **引用校验**: entity_ref 必须属于同一 Worktree; 不匹配返回 entity_ref_worktree_mismatch (canvas.rs:1863), 指向其他 Worktree 的 worktree 返回 not_found (canvas.rs:1868)。
- **并发与幂等**: expected_version (canvas.rs:892-893, 1219-1220, 1242, 1382) 与请求哈希 (canvas.rs:786, 863, 1120, 1224, 1392, 1527, 1616) 配合 identity_key_present (canvas.rs:2230)。
- **行级安全**: 五张画布表启用 RLS 并声明 INSERT 与 UPDATE 策略 (migration:167, 194, 221, 248, 269)。
- **前端门控**: readOnly 与 onCreateElement 只决定 UI 是否显示工具, 不是安全边界; 服务端以 scope 为准。
- **未实现**: 分享链接、公开访问与 OAuth 授权均未实现 (见 FR-06-37)。

### 4.6 依赖关系

- 服务端: star-api-rest 的 group_api 模块 (group_api.rs:206-207 声明, :814-815 合并路由); PostgreSQL 与画布迁移。
- 前端: 群组页 (group/page.tsx)、投影 (groupProjection.tsx)、客户端 (worktreeGroupApi.ts)、共享组件 CanvasView、本地 store (store.ts)、导航状态 (canvas/page.tsx:30)、StatusPill (CanvasView.tsx:51)。
- 任务域: WorkItem 与 Worktree 通过 entity_ref 与 create_work_item_on_canvas 关联 (canvas.rs:571, 1853-1893)。
- 共享基线: docs/basic-design.md §16 (群组基本设计)。
- 未使用的依赖: html2canvas 已声明但未引用 (frontend/package.json:35)。
- 独立服务: canvas-engine、domain-canvas、canvas-realtime 不在群组 live 路径上, 其部署清单见 docs/deployment/star-canvas-services-deploy.md:3-11。

---

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 群组画布应用 (挂载与交互) | frontend/src/app/worktree/[id]/group/page.tsx | 🟡 | 挂载 :3126-3252; 门控 readOnly 与 onCreateElement :3234, 3242 (见 6 G-01) |
| 群组投影与订阅装配 | frontend/src/lib/group/groupProjection.tsx | ✅ | 模式 :161-164; 订阅装配 :218; 数据加载 :265-277 |
| 群组 API 客户端 | frontend/src/lib/group/worktreeGroupApi.ts | ✅ | 画布方法 :1182-1345; 唤醒与游标 :1291-1576 |
| 共享画布渲染组件 | frontend/src/components/CanvasView.tsx | 🟡 | 1482 行; 工具与渲染 :96, 735-966; 写门控 :1083; 撤销门控 :134, 1220 |
| /canvas 上下文入口 | frontend/src/app/canvas/page.tsx | ✅ | 解析 :47-54; 新建 :60-78; 未接入 :105-131 |
| 上下文解析与本地建画布 | frontend/src/lib/canvas-entry.ts | 🟡 | 解析 :37-64; 本地建画布 :79-103 (本地通道, 见 6 G-13) |
| /canvas/[id] 画布页 | frontend/src/app/canvas/[id]/page.tsx | 🟡 | 五个子 tab :54, 158-245; 本地写入 :170, 188 |
| /collaboration 画布列表 | frontend/src/app/collaboration/page.tsx | 🟡 | 列表与新建 :17, 32, 54, 62-64 (本地数据) |
| 本地 store 画布 | frontend/src/lib/store.ts | 🟡 | 状态 :170-172; 动作 :367-379, 1047-1083; 持久化排除 :1677-1679 |
| 画布类型 | frontend/src/types/ids.ts | ✅ | :748-877 |
| 服务端画布 API | crates/star-api-rest/src/group_api/canvas.rs | 🟡 | 路由 :403-436; 处理函数 :439-2161; 测试 :2263, 2300, 2330 |
| 服务端事件流 | crates/star-api-rest/src/group_api/canvas_event_stream.rs | ✅ | 路由 :148-149; 监听 :61, 287-294 |
| 画布迁移 (群组) | db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql | ✅ | 表与约束 :20-132; RLS :167-269 |
| 画布文档迁移 | db/migrations/2026-09-29-worktree-group-phase-3c-canvas-document.sql | ✅ | 文档持久化 :2-12 |
| 独立服务骨架 | crates/canvas-engine, crates/domain-canvas, crates/canvas-realtime | 🟡 | 仅健康与占位路由 (canvas-realtime/src/lib.rs:95-98; domain-canvas/src/lib.rs:84-87; canvas-engine/src/lib.rs:88-92) |
| 协作领域骨架 | crates/canvas-collab | ⬜ | 仅骨架, 0 个业务方法 (crates/canvas-collab/src/lib.rs:3) |
| 设计参考稿 | docs/prototypes/2 · Branch 工作区 · 统一画布-html/ | 🟡 | 参考稿, 非生产代码 (README 3-8) |
| 运维状态页 | frontend/src/app/ops/page.tsx | 🟡 | 服务版本为静态数据 (:41-43), 不代表运行状态 |

---

## 6. 差距与后续

| 编号 | 差距 | 影响 | 下一步 |
|---|---|---|---|
| G-01 | 群组 live 挂载 `readOnly={mode === "live"}` 与 `onCreateElement` 同时成立, 创建工具被门控关闭 (group/page.tsx:3234, 3242; CanvasView.tsx:1083, 612, 1324), 而移动与删除仍开放 (CanvasView.tsx:329, 1256) | 群组画布无法新增便签、文本、形状、画笔与评论; 语义与"只读"相反 | 确认 readOnly 的产品语义; 以权限决定只读, 以 onCreateElement 是否存在决定写模式; 为 live 挂载补组件测试 |
| G-02 | 撤销栈与快捷键绑定在 onCreateElement 未传入时 (CanvasView.tsx:134, 473, 1220) | /canvas/[id] 与群组 live 均不可撤销 | 解耦撤销栈与创建回调; 撤销基于服务端版本或 CRDT 二选一 (见 G-06) |
| G-03 | path 元素未被服务端接受 (migration:53-56; canvas.rs:2178-2187), 前端仍会创建 (CanvasView.tsx:663) | 群组画布中的画笔数据无法保存 | 将 path 与 path_data 纳入迁移 CHECK 与校验, 或在 live 模式隐藏画笔 |
| G-04 | EntityRef 类型不一致: 前端六值 (types/ids.ts:800-804), 服务端二值 (migration:92; canvas.rs:1382, 2213) | 前端可构造被后端拒绝的引用 | 以 work_item 与 worktree 收敛前端类型; agent_session、automation、pull_request、comment 另行设计 |
| G-05 | 元素 kind 词表三处不一致: 前端 11 值 (含 path), 迁移与服务端 10 值 | 与 G-03 同源 | 与 G-03 同步修订 |
| G-06 | 设计写 WebSocket 推送与 Yjs CRDT (docs/basic-design.md:4401; SRS-CANVAS-001.md 的 A12.6 项); 实现为 SSE 唤醒加游标追赶加版本拒绝 (canvas_event_stream.rs:148-149; worktreeGroupApi.ts:1397) | 并发编辑依赖冲突后重试, 无自动合并 | 以现实现为当前基线; CRDT 选型作为独立决策单列 |
| G-07 | 实时光标与 presence 缺失 (FR-06-40) | 多人编辑无位置感知 | 在 SSE 通道增加 presence 事件与过期表, 或独立通道 |
| G-08 | 作用域: docs/basic-design.md:4297, 4311 描述 Run 作用域画布; 服务端路由仅 Worktree 作用域 (canvas.rs:403-436) | 跨 Worktree 的画布无法表达 | 按 8 中的裁决执行; 若需跨 Worktree, 新增 Run 作用域路由并迁移数据 |
| G-09 | PNG 导出与分享缺失; html2canvas 已声明未使用 (frontend/package.json:35) | 功能缺口与依赖冗余 | 实现导出与分享, 或移除依赖与对应规格 |
| G-10 | e2e 规格过期: canvas-view.spec.ts:44 使用 /canvas/canvas-001; canvas-share-export.spec.ts:36-68 断言 Share 与 Export PNG | 测试失去守护作用 | 改写到 /canvas/[id] 与群组页, 或删除规格 |
| G-11 | frame-select 工具为占位 (CanvasView.tsx:1154); 群组 Frame 操作在页面侧 (group/page.tsx:1589-1672) | Frame 创建入口分裂 | 统一到 CanvasView 工具栏 |
| G-12 | work-item 建卡的画布内入口未核实 (group/page.tsx:1414) | 需求 FR-06-24 无法被用户触达 | 在工具栏或右键菜单增加入口 |
| G-13 | 两条画布数据通道并存: 本地 store (canvas-entry.ts:79-103; store.ts:170-172) 与服务端 (worktreeGroupApi.ts:1182-1260) | /canvas 与群组画布显示不同数据 | 决定 /canvas/[id] 迁移到服务端或退役本地通道 |
| G-14 | docs/basic-design.md:4403 声明现行实现为 CanvasOutboxPoller 轮询; 前端检索不到该符号 | 设计对账失真 | 修订 basic-design §16 的对账段落, 指向唤醒与追赶实现 |
| G-15 | canvas-engine、domain-canvas、canvas-realtime 仅有骨架路由, 却列入部署清单 (docs/deployment/star-canvas-services-deploy.md:11) | 部署的服务与功能无对应 | 决定保留或移出部署; live 画布不依赖这些服务 |
| G-16 | 设计要求 CanvasToolbar、CanvasSidebar、CanvasMinimap 独立文件 (docs/design/BD-CANVAS-001.md:277-283), 实现内联于 CanvasView.tsx | 单文件 1482 行, 难以测试 | 拆分为独立模块 |
| G-17 | 无视口裁剪与性能基准 (FR-06-11, FR-06-51) | 大画布可能掉帧 | 增加裁剪与基准测试 |
| G-18 | 运维页服务版本为静态数据 (frontend/src/app/ops/page.tsx:41-43) | 可能被误读为运行状态 | 接入真实服务状态, 或标注为示例 |

---

## 7. 验收与测试

| 类别 | 位置 | 覆盖 | 说明 |
|---|---|---|---|
| 组件测试 | frontend/src/components/CanvasView.test.tsx:29-174 | 8 个用例 (A 至 H): 画笔按钮默认显示与 readOnly 隐藏 (D); 输入输出端口渲染与拖线 (E 至 H) | 存在 |
| 入口测试 | frontend/src/lib/canvas-entry.test.ts | 10 个用例, 上下文解析与本地建画布 | 存在 |
| 群组 API 测试 | frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts | 34 个用例, 含画布接口与唤醒流 | 存在 |
| 投影测试 | frontend/src/lib/group/__tests__/groupProjection.test.tsx | 6 个用例 | 存在 |
| Store 测试 | frontend/src/lib/store.test.ts | 20 个用例, 画布相关 2 处 | 存在 |
| 服务端单元测试 | crates/star-api-rest/src/group_api/canvas.rs:2263, 2300, 2330 | 游标非负与分页边界, 游标时间戳与 ID 配对 | 存在 |
| 服务端 PG 集成测试 | crates/star-api-rest/tests/canvas_event_stream_pg.rs (3 个); crates/star-api-rest/tests/canvas_device_auth_pg.rs (4 个) | 事件流与设备授权 | 依赖 PostgreSQL; 本文档未执行 |
| E2E | frontend/e2e/canvas-view.spec.ts (9 个); frontend/e2e/canvas-share-export.spec.ts (3 个) | 规格过期 (G-10), 不能作为当前行为的证据 | 过期 |

未覆盖的验证点: 元素版本冲突在界面上的恢复路径; 文档版本冲突的恢复路径; live 挂载的创建门控 (G-01); 唤醒流断线重连后的游标追赶; 迁移在空库上的执行; 缺失 scope 的负向测试。

本文档未运行上述测试套件, 测试存在性依据测试文件与用例名称。

---

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-CANVAS-001.md (v1.4, 2026-09-28) | 群组边界与 EntityRef (§1.6, 168-176; §1.4 消歧 151); 三核心划分 (§1.3.1, 103-109); P0 清单 (§4.4); 2026-11-15 现状纠正 (4-6) | 纠正注记称 /canvas 直进三标签; 代码于 2026-11-16 改为按上下文解析 (canvas/page.tsx:17-21); §1.3.1 的 132 项为三核心索引, 与 F06 边界不同 | 采用为 F06 范围与 EntityRef 权威; /canvas 入口行为以代码为准; 三核心子需求不纳入 F06 |
| docs/design/BD-CANVAS-001.md (v0.2) | 群组上下文与 EntityRef (§1.3, 180-191); 同级应用与聊天栏 (188); CanvasView 复用 (§3.1, 277-283); 14 张表 W/T/M 汇总 (§1.1.2, 82-110) | §1.1.2 所列 A11、A12 表与 ARG 模块不在迁移中; §3.1 要求的组件拆分与代码不符 (G-16); 纠正注记 (4-6) 与代码冲突 | 采用 §1.3 作为群组边界; A11、A12 表记为 ⬜, 不属 F06 核心; 组件拆分记为 G-16 |
| docs/design/DD-CANVAS-001.md (v0.2, 2026-09-28; 已读至约 589 行, 含 §9.1 群组扩展 DDL 起始段) | group_canvas_registry 与 canvas_entity_ref 的 DDL (565-589), 与迁移表结构一致 (migration:20-107); C-25、C-26 类 (281-282) | 类清单与代码模块不对应; 589 行之后未读 | 采用群组 DDL 作为数据模型权威; 类清单不作为实现依据 |
| docs/frontend-canvas-design.md (v0.1, 2026-08-26) | 元素与 bezier 连线 (§3.5); StatusPill 色码 (§3.4); 快捷键表 (§5.1); /collaboration 画布列表 (§8.1); URL 高亮 (§4.9); 方案 B (§1.2) | 方案 B 称后端无改动 (§1.2, 65-73), 而群组 API 已在 star-api-rest 中; §5.1 快捷键与代码 (CanvasView.tsx:512-520) 不同; §9 的撤销与小地图状态与代码不同 | 以代码为准; 方案 B 被群组 API 取代, 因为服务端 EntityRef 校验与版本控制无法由客户端单独完成; 快捷键取代码 |
| docs/requirements/SRS-STAR-CANVAS-001.md (v0.1, 标注 SUPERSEDED 2026-09-07; 已读前 890 行) | 历史 Miro 对标的 8 大功能域 | 整册被替代 (3-27) | 🗑 不作为设计依据; 仅保留三 crate 骨架名称, 见 G-15 |
| docs/design/BD-STAR-CANVAS-001.md (v0.1, 标注 SUPERSEDED; 已读前 806 行) | 三 crate 骨架概念 | 其余内容作废 (3-15) | 🗑 |
| docs/design/DD-STAR-CANVAS-001.md (v0.1, 标注 SUPERSEDED; 已读前 1282 行) | Yjs 协议与 SCD Type 2 模式 | 作废 (3-15); Yjs 未被采用 (frontend/package.json) | 🗑 Yjs 部分; 当前以版本号 CAS 为准 |
| docs/requirements/SRS-STAR-CANVAS-GAME-001.md (标注已退役 2026-10-30; 仅读头部) | 游戏方向 | 退役 (3) | 🗑; canvas-game 代码不属 F06 |
| docs/basic-design.md §16 (4229-4409, 相关段) | Run 作用域画布 (4297, 4311); 可写字段的 Element PUT 模式 (4313); expected_version 与 Idempotency-Key (4373); 原子建卡 (4375); 事件与 WebSocket 推送 (4399-4401); 实现对账 (4403-4409) | 作用域与代码不符 (G-08); 轮询描述与代码不符 (G-14) | 作用域采用 Worktree 作用域, 理由: 代码路由与迁移以 worktree_id 为键 (canvas.rs:403-436; migration:20-46), 与 DD v0.2 及 BD §1.3 一致; 事件通道以代码的 SSE 唤醒加追赶为准 |
| docs/deployment/star-canvas-services-deploy.md | 三个 Canvas 后端服务与 Envoy 入口的部署清单 (3, 7, 11) | 服务为骨架 (G-15) | 不作为 live 画布实现依据 |
| docs/prototypes/2 · Branch 工作区 · 统一画布-html/README.md 与 Canvas.dc.html (读 README 与 `<title>`) | "统一画布" 命名; 设计参考 | README 明确为参考稿, 非生产代码 (3-8) | 采用为命名与视觉参考; 不作为实现证据 |
| 未通读, 仅按路径识别: docs/requirements/SRS-WORKTREE-CANVAS-001.md、docs/design/BD-WORKTREE-CANVAS-001.md | Worktree Overview Graph (另一对象, SRS-CANVAS-001.md:176) | 不属 F06 范围 | 不纳入裁决 |
| 未通读, 仅按路径识别: SRS-CANVAS-AGENT-001、SRS-CANVAS-GAMIFY-001、SRS-CANVAS-WORKFLOW-001 及其 BD、DD | 三核心子需求 (SRS-CANVAS-001.md:103-108) | 属其他 feature | 不纳入裁决 |
| 代码源: CanvasView.tsx、canvas 页面、group/page.tsx、groupProjection.tsx、worktreeGroupApi.ts、store.ts、types/ids.ts、collaboration/page.tsx、ops/page.tsx、group_api 的画布与事件流模块、画布迁移 | 实现依据 | ops 为静态数据 (ops/page.tsx:41-43) | 代码为现状权威 |

---

## 附: 术语

- EntityRef: `(type, id, worktree_id)` 形式的类型化引用; 服务端仅接受 work_item 与 worktree (canvas.rs:2213-2228)。
- 群组 live 模式: groupProjection 的授权投影 (groupProjection.tsx:164); preview 模式为非授权展示 (groupProjection.tsx:161)。
