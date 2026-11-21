# F11 协同空间（文件即任务，多人多 Agent）

> 状态摘要: `/nema` 页面可登记共享文件、生成 `.nema` 侧车、增删编辑者并查看 strict 编辑痕迹，编辑者状态按诚实降级规则显示；实时在线、文件租约写入、冲突协议、语义合并、代码生成与 Agent 加入在产品路径上均未接入，页面中的代码栏、规约预览与协作组仍为示例内容。
> 库级代码（`domain-collaboration`、`ide-collab-core`、`nema-codec` 的租约仲裁器）存在，但没有挂到产品路由。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

让人与多个 Agent 以"文件即任务"的方式共同编辑同一共享文件。文件被两人以上共同编辑时生成 `.nema` 侧车，侧车记录编辑者与编辑痕迹，并可打开为任务卡。写入须持有带递增 fencing token 的文件租约；重叠编辑不静默覆盖，双方草稿保留并生成可恢复的冲突；在线状态与推送只来自真实的实时连接。

### 1.2 范围内 / 范围外

范围内:

- `/nema` 页面，即 2b 协同原型的产品落点（`frontend/src/app/(app)/nema/page.tsx:45`）。
- `.nema` 侧车的 store、编辑者状态、磁盘监听与解析（`frontend/src/lib/nema/`）。
- `domain-collaboration` 的 Presence、订阅与冲突协议（规范 `docs/specs/domain-collaboration-spec.md` v1.6）。
- 文件租约与 owner CAS 的库级实现及其接入需求（`crates/ide-collab-core`、`crates/nema-codec`）。
- 2b 原型的五步流水线、协作组、语义合并与验证面板的设计意图。

范围外:

- `/collaboration` 画布列表，归 F06；该路由已被重定向（`frontend/src/lib/redirects.ts:71`）。
- A12 画布多人编辑与 BFF WebSocket，归 F08（`bff/src/collaboration/`）。
- Worktree 画布事件流 `canvas-events`，归 F06 与 F07。F11 复用其唤醒模式，见 §8。
- FileSession（文件级 Agent 会话），归 F12；当前未实现（`docs/features/F12-file-session.md:3`）。
- 看板"按文件开卡"与看板轮询，归 F04。
- 顶栏实时状态点，归 F01。
- 沙盒页签（2c），归 F13。

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 编辑者（人） | 登记共享文件；增删共同编辑者；查看编辑痕迹；把 `.nema` 打开为任务卡 |
| Agent（AI 编辑者） | 以编辑者名称参与；在文件租约下写入自己负责的段；与人并行编辑 |
| 协同负责人 | 查看各文件的编辑者在线状态与冲突；选择合并采用哪一方；推进验证 |
| 平台管理员 | 管理 Worktree 级的协作策略与租约超时 |

主要用户故事:

1. 把一个被多人编辑的文件登记为共享文件，系统生成 `.nema` 侧车并记录编辑者。
2. 查看每个编辑者的状态：有痕迹即显示编辑记录，无痕迹即显示"未开始"，无实时源则显示"未接入"。
3. 把 `.nema` 打开为任务卡，标题与描述随侧车变化回写。
4. 邀请 Agent 作为编辑者加入，并在租约下写入其负责的段。
5. 两人同时修改同一段时，双方草稿保留，形成同一冲突 ID 的冲突记录，经显式选择解决。
6. 在 `.nema` 上完成语义合并，生成代码，并在验证面板中确认结果。

## 3. 需求

判定规则（本文件的分配规则，规范与原型未给出优先级）:

- 状态按产品路径判定：可从 `/nema` 或生产二进制的路由到达的行为才计为 ✅ 或 🟡；仅存在于库代码的需求计为 ⬜，库代码证据列在 §5 与 §6。
- 优先级：P0 为 INV/AC 门禁，或直接关系到数据正确性与授权边界的需求；P1 为协同核心交互与实时状态；P2 为原型的展示与便捷功能。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-11-01 | 协作数据全部带 tenant_id，跨 tenant 访问拒绝（INV-CB-01、INV-CB-04） | P0 | 🟡 | 领域层 `assert_tenant` 与 `require_session` 已实现（`crates/domain-collaboration/src/lib.rs:257-262`、`:870-884`）；`/nema` 产品路径无租户校验，开卡时写死租户（`frontend/src/lib/nema/nemaStore.ts:328-329`）；画布事件流按 worktree 重新授权（`crates/star-api-rest/src/group_api/canvas_event_stream.rs:171`、`:191`） |
| FR-11-02 | Presence 在线状态，心跳过期后判为离线（INV-CB-03，设计值 60 秒） | P1 | ⬜ | 产品无 presence 数据源（`frontend/src/lib/nema/nemaCollaboration.ts:61`）；领域层 `Presence::recompute_active` 使用 30 秒阈值（`crates/domain-collaboration/src/lib.rs:307`、`:330-333`），与设计值不一致（§8 R2） |
| FR-11-03 | RealtimeSubscription：单连接多订阅（每连接不超过 100），filter 只能缩小可见范围（INV-CB-02，规范 §2） | P1 | ⬜ | 产品代码中无 `star.v1` 子协议与 `realtime/subscribe` 端点（`crates/`、`bff/src/`、`frontend/src/`、`db/` 检索 0 处）；领域层无订阅实体 |
| FR-11-04 | 推送只覆盖当前授权的资源范围；撤权后关闭订阅并拒绝恢复读取（INV-CB-04，规范 §7） | P0 | 🟡 | 仅 Worktree 画布事件流具备按资源重新授权（`canvas_event_stream.rs:154-171`、`:191-227`）；`.nema` 文件与任务没有推送路径 |
| FR-11-05 | 已连接协作不依赖固定周期轮询，推送由 owner 提交触发（INV-CB-07、AC-COLLAB-01） | P0 | 🟡 | 画布组投影由唤醒订阅驱动（`frontend/src/lib/group/groupProjection.tsx:218`；`frontend/src/lib/group/worktreeGroupApi.ts:1576`）；`.nema` 磁盘扫描每 20 秒轮询（`frontend/src/lib/nema/nemaWatcher.ts:137`、`:164`）；看板 2 秒轮询（`frontend/src/hooks/useBoardSync.ts:4`，归 F04） |
| FR-11-06 | 慢消费者使用有界队列；溢出时标记 resync 并从持久游标恢复，不静默丢弃（INV-CB-09、AC-COLLAB-05/06） | P1 | 🟡 | 画布事件流队列容量 64、全进程流数 128（`canvas_event_stream.rs:73-74`）；收到 ResyncRequired 信号时发送 `resync_required`（`:222-224`），监听异常时发布该信号（`:307-331`）；A12 hub 的 `lagged` 分支会丢弃事件（`bff/src/collaboration/wss_hub.rs:249-262`），但 bff 无可达入口（§5） |
| FR-11-07 | 持久业务事件与临时实时信号分开（REQ-RT-003，规范 §5） | P1 | 🟡 | 画布唤醒信号 `WakeSignal`（`canvas_event_stream.rs:73-88`）与持久 outbox 分离；`.nema` 与任务没有 owner 事件 outbox |
| FR-11-08 | 草稿按 base_revision 做 owner CAS 保存，不以客户端时钟判定冲突（AC-COLLAB-02） | P0 | ⬜ | 产品无 SaveDraft 接口（`crates/`、`frontend/src/` 检索 0 处）；库层 `commit` 检查 fencing 与 revision（`crates/ide-collab-core/src/coordinator.rs:246-276`），但无路由，且 coordinator 无测试 |
| FR-11-09 | 重叠编辑保留双方草稿，生成同一 conflict_id 的可恢复冲突，经显式 CAS 解决，不做 last-write-wins（AC-COLLAB-02/03/08，规范 §13.3） | P0 | ⬜ | 产品代码中无 conflict_id、ResolveConflict、ReadConflict（检索 0 处）；`.nema` 编辑直接改写条目（`nemaStore.ts:412-468`） |
| FR-11-10 | 冲突对在线双方显示同一持久未读提醒，p99 不超过 1 秒（AC-COLLAB-04） | P1 | ⬜ | 无冲突实体与推送，同 FR-11-09 |
| FR-11-11 | OT 仅作为显式选择后的只读预览，写入须通过当前版本 CAS（AC-COLLAB-09） | P2 | ⬜ | 无 OT 实现；`ide-collab-core` 的 `PatchEngine` 做补丁重定位（`crates/ide-collab-core/src/lib.rs:30`），未接路由 |
| FR-11-12 | 编辑者状态诚实降级：无实时源时只显示"未开始"或"未接入"，不显示"正在编辑" | P1 | ✅ | `nemaCollaboration.ts:70-87`；`frontend/src/app/(app)/nema/page.tsx:595`、`:615-627`；`nemaCollaboration.test.ts:215-230`；`nemaPage.collaboration.test.tsx:50-68` |
| FR-11-13 | 实时在线状态："正在编辑""离线"只能来自实时连接 | P1 | ⬜ | `nemaCollaboration.ts:54-55` 定义 `NemaLivePresence` 类型但无实现；`NEMA_REALTIME_COLLABORATION_AVAILABLE = false`（`:61`）；页面调用时未传入 live 参数（`page.tsx:595`） |
| FR-11-14 | 当前编辑者由既有身份机制解析，不新建用户体系 | P1 | ✅ | `nemaCollaboration.ts:98-109`；`page.tsx:188-189` |
| FR-11-15 | 编辑者增删，每次变更写入编辑痕迹，并通过 strict profile 往返校验 | P1 | ✅ | `nemaStore.ts:412-468`；`nema.ts:871-882`；`page.tsx:255-283`、`:664-683`；`nemaCollaboration.test.ts:65-147` |
| FR-11-16 | 编辑者变更前复验资源授权，撤权即拒绝 | P0 | ⬜ | `updateNemaEditors` 不做授权判断（`nemaStore.ts:412-468`）；页面不做会话闸门，只解析显示名（`page.tsx:189`） |
| FR-11-17 | 共同编辑者不少于 2 人的共享文件自动生成 `.nema` 侧车，同一路径幂等 | P1 | ✅ | `nemaStore.ts:255-295`；少于 2 人返回 null（`:256-259`） |
| FR-11-18 | 单人登记不生成侧车，并给出可见反馈 | P2 | 🟡 | 单人分支返回 null（`nemaStore.ts:256-259`）；页面无论结果如何都关闭表单（`page.tsx:221-225`），用户看不到反馈 |
| FR-11-19 | 登记时固定解释器 profile，已登记条目不漂移；legacy 条目不静默升级 | P0 | ✅ | `nemaStore.ts:255-263`、`:348-363`、`:416-421`；`nemaCollaboration.test.ts:184` |
| FR-11-20 | `.nema` 可打开为 Star 任务，标题与描述回写到绑定的 WorkItem | P1 | 🟡 | 开卡（`nemaStore.ts:297-342`）、同步（`:370-410`）、编辑后回写（`:454-466`）已实现；租户、项目、报告人写死（`:328-334`） |
| FR-11-21 | 磁盘上的 `.nema` 变化同步到记录（仅本地模式） | P2 | 🟡 | 监听仅在本地模式启用（`nemaWatcher.ts:159`）；经 `/api/v1/workspace-files` 读取（`frontend/src/lib/localFiles.ts:71`、`:78`）；该路由在生产二进制挂载（`crates/star-api-rest/src/main.rs:159-161`）；网页版降级（`nemaWatcher.ts:23`） |
| FR-11-22 | 工作集按"任务卡"与"其他"切换；文件列表显示登记人数 | P1 | 🟡 | 切换可用（`page.tsx:296-311`）；人数为登记编辑者数，不是在线人数（`page.tsx:361`、`:210`）；标签与计数写死（`page.tsx:292`、`:343`、`:350`、`:383`） |
| FR-11-23 | 五步流水线显示当前阶段并可推进（①路由拆分 ②并行编辑 ③语义合并 ④生成代码 ⑤验证） | P1 | 🟡 | 阶段以常量渲染，当前阶段固定为 ②（`page.tsx:89`、`:456-469`、`:462`）；"推进到 ③ 语义合并"无处理函数（`:690-692`） |
| FR-11-24 | 路由 Agent 按文件拆分：同文件按段分区为协作组，不同文件为独立组 | P2 | ⬜ | 协作组 A/B 为静态示意，标注"示意, 实时协作未接入"（`page.tsx:522`、`:540`）；无路由逻辑（REUSE-MAP 的 `nema_pipeline` 不在仓内，§8 R5） |
| FR-11-25 | 邀请 Agent 作为编辑者加入 | P1 | ⬜ | 按钮无处理函数（`page.tsx:693-695`） |
| FR-11-26 | 在 CLI 中打开当前任务或文件 | P2 | ⬜ | 按钮无处理函数（`page.tsx:696-698`）；AGENTS.md:46 要求 Agent/Task CLI 闭环验收后才能标为可用 |
| FR-11-27 | 写入须持有文件租约；租约带递增 fencing token，过期自动释放，人可随时接管 | P0 | ⬜ | 页面只有静态说明（`page.tsx:514-517`）；编辑写入不检查租约（`nemaStore.ts:412-468`，仅有进程内互斥 `:231-248`）；库层 claim、heartbeat、commit、reap 存在但未接入（`crates/ide-collab-core/src/coordinator.rs:208-276`、`:357-364`）；coordinator 无测试 |
| FR-11-28 | 语义三方合并：不重叠段自动合并，同一字段冲突需选择 | P1 | ⬜ | 产品与库代码无合并实现；"语义合并"仅出现在流水线文案（`page.tsx:89`、`:691`） |
| FR-11-29 | `.nema` 生成代码并与 SourceMap 对齐，显示漂移 | P1 | ⬜ | 代码栏为写死样例（`page.tsx:103-116`、`:494-511`），漂移徽标为固定文字（`:499`）；无生成实现 |
| FR-11-30 | 验证（构建、测试、沙盒、漂移、许可证）结果写回任务卡 | P2 | ⬜ | 页面无第 5 步视图，无写回实现 |
| FR-11-31 | 从本地文件选择共享文件；从 Branch 选择文档或代码；导入表格与文本 | P2 | 🟡 | 仅本地文件选择（`page.tsx:428`；`frontend/src/components/nema/LocalFolderBrowser.tsx:12`）；Branch 选择与导入无实现 |
| FR-11-32 | 文档与表格视图，段落级租约标记，表格视图 | P2 | ⬜ | `DOC_FILES` 常量未使用（`page.tsx:98-101`）；文档只显示文件名（`:388-396`） |
| FR-11-33 | 协作数据存为 owner 表，带 RLS，并完成 W/T/M 分类（AC-COLLAB-07） | P1 | ⬜ | 无 `collaboration.presence`、subscription、草稿与冲突表的 DDL；全部 `db/migrations` 中的 presence 表属于画布（`db/migrations/2026-09-10-p3d6-15-more-tables.sql:346`）；WLD-7 数据设计为目标 v0.8，未生成迁移（`docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md:2-3`） |
| FR-11-34 | 同一 `.nema` 的编辑状态对所有协作者可见（共享持久） | P1 | ⬜ | 持久化只在浏览器 localStorage（`nemaStore.ts:470`，key `star-nema-files`）；无服务端端点 |
| FR-11-35 | 跨租户推送尝试与编辑者变更写入审计（规范 §5、附录 A） | P2 | ⬜ | `domain-collaboration` 依赖仅 `star-context` 等（`crates/domain-collaboration/Cargo.toml:16-23`），无审计端口；`.nema` 痕迹只存在于源文件内（`nemaStore.ts:441-447`） |

## 4. 设计

### 4.1 概念模型

| 概念 | 定义 | 当前代码落点 |
|---|---|---|
| 共享文件 | 被多人共同编辑的源文件或文档，按路径标识 | `NemaTaskEntry.filePath`（`frontend/src/lib/nema/nemaStore.ts:151-173`） |
| `.nema` 侧车 | 与共享文件同名的规约文件，记录编辑者、解释器版本与编辑痕迹 | `NemaTaskEntry.nemaPath`、`source`（`nemaStore.ts:151-173`）；语法在 `frontend/src/lib/nema/nema.ts` 与 `crates/nema-codec` |
| 编辑者 | 参与编辑的人或 Agent，当前以显示名字符串登记 | `NemaTaskEntry.editors: string[]`（`nemaStore.ts:158`） |
| 编辑痕迹 | 每次编辑者变更追加的记录（editor、edited_at、change） | `NemaEditTraceEntry`（`nema.ts`，读取于 `:871-882`） |
| 任务卡 | 由 `.nema` 打开的 WorkItem，标题与描述随侧车回写 | `openNemaAsStarTask`（`nemaStore.ts:297-342`） |
| Presence | 某用户在某资源上的在线状态（领域层为 `Presence`，规范为 presence_id 与状态枚举） | 领域层 `crates/domain-collaboration/src/lib.rs:288-303`；产品层无 |
| 订阅 | 连接对资源范围的授权监听 | 规范 §2（RealtimeSubscription）；代码中无 |
| 文件租约 | 写入权的门禁，带递增 fencing token 与过期时间 | 库层 `crates/ide-collab-core/src/coordinator.rs:65-78`、`:208-276`；产品层无 |
| 草稿与冲突 | 重叠编辑的双方草稿与可恢复冲突记录 | 规范 §13（设计）；代码中无 |
| 流水线阶段 | 五步：路由拆分、并行编辑、语义合并、生成代码、验证 | 页面常量 `page.tsx:89`，无状态机 |
| 协作组 | 同一文件内按段分区的编辑者组，与不同文件的独立组 | 页面静态示意（`page.tsx:519-558`） |

### 4.2 数据模型

| 实体 | 关键字段（证据） | 持久化 | 现状 |
|---|---|---|---|
| NemaTaskEntry | id、filePath、nemaPath、title、editors、source、interpreterProfile、legacyFormat、workItemId、created_at、updated_at（`nemaStore.ts:151-173`） | 浏览器 localStorage，zustand persist（`:470`） | ✅ 本地可用；不跨用户共享 |
| .nema 文档中的编辑者与痕迹 | who、editors、editor_count、updated_at，以及 Field/Value 表中的痕迹行（`nemaStore.ts:208-212`，`nema.ts:757`） | 写在 `source` 全文内 | ✅ 本地可用 |
| 领域 Presence | id、session_id、user_id、cursor_position、selection、last_heartbeat、is_active（`crates/domain-collaboration/src/lib.rs:288-302`） | 内存（`InMemoryCollabRepository`，`:1225-1366`） | 🟡 库层；无产品路径 |
| 领域 CollaborationSession | id、tenant_id、project_id、parent_type（WorkItem、Document、Whiteboard）、parent_id、started_by、started_at、ended_at、status（Active、Paused、Ended）、version（`lib.rs:209-230`） | 内存 | 🟡 库层；与规范实体不同（§8 R3） |
| 领域 Cursor、Whiteboard、WhiteboardShape | `lib.rs:345-`、`:428-`、`:471-` | 内存 | 🟡 库层 |
| 库层 TaskState | task_id、job_id、workspace_id、file_id、symbol_path、base_revision、status、worker_id、fencing_token、lease_expires_at、priority（`crates/ide-collab-core/src/coordinator.rs:65-78`） | 无持久化适配器引用 | 🟡 库层；按符号级任务，不是 `.nema` 文件 |
| 库层 FileState | file_id、workspace_id、path、current_revision、content、content_hash（`coordinator.rs:44-52`） | 同上 | 🟡 库层 |
| 库层 RemoteCursor | agent_id、display_name、role、color、cursor、selection、active_symbol、last_active_epoch_ms（`crates/ide-collab-core/src/collaboration.rs:92`） | 内存 | 🟡 库层；由演示端点构造（`crates/star-api-rest/src/routes/integrations.rs:49-68`） |
| 设备撤权 inbox 表 | collaboration_device_inbox、collaboration_device_projection、collaboration_inbox_checkpoint、collaboration_device_event_incident（`db/migrations/2026-11-14-w17-device-outbox-inbox.sql:53-59`） | PostgreSQL | 🟡 DDL 存在；无 presence、subscription、草稿或冲突表 |
| collaboration.presence（设计） | TTL 5 分钟，Valkey 缓存为主（`docs/data-design/ipa-detail/tables/collaboration_presence.md:22`） | 无 DDL | ⬜ |

### 4.3 接口（routes、APIs、ports）

| 接口 | 位置 | 现状 |
|---|---|---|
| `GET /api/v1/workspace-files`、`/read` | `crates/star-api-rest/src/workspace_files.rs:128`、`:132`；生产挂载 `main.rs:159-161` | ✅ 受控只读，白名单 `STAR_FILE_ROOTS`（`main.rs:158` 注释） |
| `GET /api/v1/collab/session` | 路由 `crates/star-api-rest/src/lib.rs:215`，经 `build_supplemental_routers` 挂载（`lib.rs:182`）；处理函数 `routes/integrations.rs:40-76` | 🟡 生产挂载但无鉴权（`lib.rs:180-181`），返回写死的演示光标（`integrations.rs:48`、`:74`）；前端无调用方 |
| `GET /api/v1/collab/annotations` | `lib.rs:216-219`；`routes/integrations.rs:79-86` | 🟡 生产挂载但无鉴权；返回锚点计数与说明文字 |
| `GET /api/v1/worktrees/{id}/canvas-events` 与 `/stream` | `crates/star-api-rest/src/group_api/canvas.rs:430`；`canvas_event_stream.rs:149`；合并入 `group_api.rs:824-825`（F06 所有） | ✅ 生产挂载；F11 复用候选 |
| MCP 资源 `collaboration://{id}` | `crates/star-mcp/src/handlers/collaboration.rs:28-36` | 🟡 Phase H mock，返回固定参与者（`:29`、`:31-36`） |
| WebSocket `star.v1` 与订阅端点 | 规范 §10、§4 | ⬜ 产品代码中无 |
| `CollabCommandPort`、`CollabQueryPort`、`CollabRepository` | `crates/domain-collaboration/src/lib.rs:709-760`、`:762-783`、`:787-830` | 🟡 库层端口；无调用方 |
| `RealtimeEventRouter`、`CollaborationCommandPort`（规范端口） | 规范 §4（`docs/specs/domain-collaboration-spec.md:73-124`） | ⬜ 代码中无 |
| SaveDraft、ReadConflict、ResolveConflict、CatchUp | 规范 §13.2（`:327-334`） | ⬜ 代码中无 |
| 前端 nema store 动作 | `registerSharedFile`（`nemaStore.ts:255`）、`openNemaAsStarTask`（`:297`）、`importNemaSource`（`:348`）、`syncNemaSource`（`:370`）、`updateNemaEditors`（`:412`）、`removeNemaTask`（`:344`） | ✅ 本地 |
| 前端 `useNemaWatcher` | `frontend/src/lib/nema/nemaWatcher.ts:143` | 🟡 本地模式轮询 |
| 前端 presence 与身份 | `nemaEditorPresence`（`nemaCollaboration.ts:70`）、`resolveNemaActor`（`:98`）、类型 `NemaLivePresence`（`:55`） | ✅ 诚实降级；live 源未接入 |
| 库层租约 | `crates/ide-collab-core/src/coordinator.rs`：`claim_task`（`:208`）、`heartbeat`（`:222`）、commit（`:246`）、`reap_expired_leases`（`:357`） | 🟡 库层；无路由 |
| NEMA 所有权仲裁 | `crates/nema-codec/src/lib.rs:3417-3534`（`NemaLeaseArbitrator`） | 🟡 库层；未导出到 `nema-codec-wasm` |

### 4.4 状态机与关键流程

编辑者状态推导（`frontend/src/lib/nema/nemaCollaboration.ts:70-87`，判定顺序即优先级）:

| 条件 | 结果 | 现状 |
|---|---|---|
| live 源返回 `editing` | "正在编辑"（实时协作连接） | 不可达：页面不传 live 参数（`page.tsx:595`） |
| live 源返回 `offline` | "离线" | 不可达，同上 |
| 编辑者在 editors 中且痕迹中无记录 | "未开始"（.nema 编辑痕迹） | ✅ |
| 其余情况 | "未接入 (实时协作未接入)" | ✅ |

登记共享文件（`nemaStore.ts:255-295`）:

1. 编辑者少于 2 人，返回 null，不生成侧车。
2. 路径已存在，保留已固定的 profile，更新标题、编辑者与 source。
3. 新路径，使用调用方固定的 profile；未固定时使用 `NEMA_DEFAULT_NEW_DOCUMENT_REVISION`（`frontend/src/lib/nema/nema.ts:108`，值 `matrix-2026-11-15`）。
4. 同一路径的操作经进程内互斥串行化（`nemaStore.ts:231-248`，互斥表是模块级变量，仅在当前浏览器标签页内有效）。

编辑者变更（`nemaStore.ts:412-468`）:

1. 进程内互斥（同上）。
2. legacy 条目直接拒绝（`:416-421`）。
3. 编辑者集合未变，返回 unchanged（`:426`）。
4. 按固定 profile 解析，改写编辑者与痕迹，格式化，并以同一 profile 重新解析验证（`:429-447`）。
5. 已绑定任务时回写标题与描述（`:454-466`）。
6. 整个过程不检查授权与租约。

开卡（`nemaStore.ts:297-342`）: 已绑定则直接返回 workItemId；否则解析源并在主 store 创建 WorkItem（租户与项目为写死的 `00000000-0000-0000-0000-000000000001`，报告人为 `usr-001`），再写回绑定。

磁盘同步（`nemaWatcher.ts:159-168`；`nemaStore.ts:370-410`）: 本地模式下挂载时扫描一次，之后每 20 秒扫描；新文件导入，已有文件在源变化时更新并回写已绑定任务。

文件租约（库层，未接入）: `claim_task` 把状态置为 Claimed，fencing_token 加 1，租约 60 秒（`coordinator.rs:212-215`）；`heartbeat` 校验 worker 与 token 并续期（`:222-242`）；commit 校验 token 与租约到期（`:260-276`，到期返回 LeaseExpired）；`reap_expired_leases` 把过期 Claimed 任务重新入队（`:357-364`）。该状态机针对符号级任务，不针对 `.nema` 文件。

草稿与冲突（规范 §13.3，设计）: 草稿写入可合并到可变草稿头；两份完整且摘要校验通过的重叠草稿在同一资源 fence 内封存并固定，生成同一 conflict_id；语义重叠即生成冲突，不因 OT 可转换而自动合并；解决需显式选择并通过当前 revision CAS。代码中均未实现（FR-11-08 至 FR-11-10）。

五步流水线（原型，UI 状态）: 当前阶段是页面常量索引 1（`page.tsx:462`），"推进"按钮无处理函数（`:690-692`）。

### 4.5 权限与安全

- 租户隔离：`.nema` 数据与 `NemaTaskEntry` 没有 tenant_id 字段（`nemaStore.ts:151-173`）；开卡时租户与项目写死（`:328-329`）。领域层的 tenant 检查不经过此路径（FR-11-01）。
- 身份：编辑者与操作者都是显示名字符串（`nemaCollaboration.ts:98-109`；`page.tsx:189`）。无会话时以 `usr-001` 回退（`nemaCollaboration.ts:90`、`:106-107`）。身份悬停提示显示邮箱（`page.tsx:610`）。
- 授权：`updateNemaEditors` 与 `registerSharedFile` 不做任何授权核验（`nemaStore.ts:255-295`、`:412-468`）。页面不做会话闸门（FR-11-16）。
- 数据暴露：`.nema` 全文与痕迹保存在浏览器 localStorage（`nemaStore.ts:470`），不经服务端审计。
- 未鉴权演示端点：`/api/v1/collab/session` 与 `/api/v1/collab/annotations` 在生产二进制中挂载且无鉴权（`crates/star-api-rest/src/lib.rs:180-182`、`:215-219`）。
- 本地文件：`/api/v1/workspace-files` 受控只读，白名单 `STAR_FILE_ROOTS`（`crates/star-api-rest/src/main.rs:158`）。
- 画布事件流：每次订阅与每个信号都重新授权（`canvas_event_stream.rs:171`、`:191`、`:208`、`:227`），有队列与流数上限（`:73-74`）。这是 F06 的实现，F11 复用时须保持这些约束。

### 4.6 依赖关系

上游（产品路径）:

- `nema-codec` 经 WASM 绑定（`frontend/src/lib/nema/nema.ts:892` 的加载器，渐进增强，失败回退 TypeScript 实现）。
- 主 store 的 `identities`（`page.tsx:188`）与 `lib/auth` 的 `getSession`（`nemaCollaboration.ts:99`）。
- `StatusNotice`（`page.tsx:86`）与 `LocalFolderBrowser`（`page.tsx:87`）。
- `/api/v1/workspace-files`（`main.rs:159-161`）。
- 导航：`navStore.ts:94` 的顶栏默认页签包含 `nema`；`registry.ts:574-580` 注册为 "NEMA Tasks"。

复用与相邻（F11 不拥有）:

- 画布唤醒与追赶：`frontend/src/lib/group/worktreeGroupApi.ts:1397`（`CanvasOutboxCatchUp`）、`:1576`（`CanvasOutboxWakeSubscriber`），由 `groupProjection.tsx:218` 使用（F06）。
- 看板"按文件开卡"读取 nemaStore（F04:51）。
- F12 把 nemaStore 作为 `.nema` 文件即任务的证据（`docs/features/F12-file-session.md:228`）。

库层依赖:

- `domain-collaboration` 仅依赖 `star-context`、serde、tokio 等（`crates/domain-collaboration/Cargo.toml:16-25`），无数据库、无审计、无消费者。
- `ide-collab-core` 被 `star-api-rest` 依赖（`crates/star-api-rest/Cargo.toml:25`），实际只在演示端点中使用 `CollaborationSession::new`（`integrations.rs:43`）。

约束来源（硬约束，不作为状态证据）: `AGENTS.md:22`（未接通授权数据的功能须显示未接入状态，不得以示例数据冒充）；`AGENTS.md:42`（实时协作以 owner 事务事实与 Outbox 推送，冲突以 owner revision/CAS 判定，不以客户端时钟，不做 last-write-wins）；`AGENTS.md:46`（Agent/Task CLI 功能须闭环验收后才能标为可用）。

### 4.7 页面设计

本功能的页面路由只有 `/nema`。

#### 4.7.1 /nema

- 文件: `frontend/src/app/(app)/nema/page.tsx`（703 行）。
- 布局:
  - 顶栏: `AppHeader`，由 `frontend/src/components/AppShell.tsx:49` 渲染；页签 `nema` 来自 `navStore.ts:94`。顶栏实时状态点恒为绿色（`frontend/src/components/AppHeader.tsx:173-180`，`:179`）。
  - 侧栏: `AppShell` 不渲染全局侧栏（`AppShell.tsx:19-23` 仅导入 AppHeader、RightChatRail 与效果组件）。页面内有左侧"工作集"区（`page.tsx:288-451`）。
  - 主区: `AppShell.tsx:61-73` 的 `main`；`/nema` 在 `frontend/src/app/(app)/layout.tsx:30` 的全高页面列表中，内容自管 padding。页面内主区为 `page.tsx:454`。
  - 右侧栏: 全局 `RightChatRail`，与 AppHeader 同级（`AppShell.tsx:77`）。
  - 底栏: 无独立底栏。操作按钮位于主区末尾（`page.tsx:689-699`）。
- 组件树:
  - `NemaPage`（`page.tsx:118`）
    - 工作集 `aside`（`:288`）
      - 任务卡范围切换（`:296-311`）、任务列表（`:316-345`）、代码文件列表（`:349-380`）、文档与表格列表（`:382-399`）
      - 关联共享文件按钮与表单（`:401-436`），表单包含 `LocalFolderBrowser`（`:428`）
      - 监听状态与错误（`:438-445`）
    - 主区 `main`（`:454`）
      - 五步流水线（`:456-470`）
      - 规约预览 `nema-spec`（`:476-491`）与生成代码 `nema-generated`（`:494-511`）
      - 租约说明（`:514-517`）
      - 协作组 A、B（`:521-558`，静态示意）
      - 共同编辑区块 `nema-collaboration`（`:561-686`）：`StatusNotice`（`:565`）、编辑者列表（`:593-640`）、添加编辑者（`:641-658`）、错误行（`:659-662`）、痕迹（`:664-683`）
      - 操作按钮（`:689-699`）

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常（共同编辑区块） | 已有 nemaTasks 且选中任务 | 任务选择器、编辑者列表、痕迹 | ✅ 数据来自 store（`page.tsx:568-685`） |
| 正常（页面主体） | profile 与 store 已就绪 | 流水线、规约、代码栏、协作组 | 🟡 规约预览来自写死文档（`page.tsx:161-172`、`:190-194`、`:483-490`）；代码栏与协作组为样例（`:103-116`、`:519-558`） |
| 加载 | profile 尚未加载 | "○ 正在加载 Rust NEMA interpreter profile" | ✅ `page.tsx:441` |
| 加载 | 痕迹读取中 | "解析中…" | ✅ `page.tsx:670-671` |
| 空 | 无 nemaTasks | "暂无已开卡 / 未开卡的 .nema 文件" | ✅ `page.tsx:317-318` |
| 空 | 无代码文件 | "暂无代码文件 — 关联共享文件后自动生成 .nema 侧车" | ✅ `page.tsx:352-353` |
| 空 | 无文档或表格 | "暂无文档/表格" | ✅ `page.tsx:384-385` |
| 空 | 无选中任务 | "暂无共同编辑文件 — 先关联共享文件 (编辑者 ≥ 2) 生成 .nema 侧车" | ✅ `page.tsx:569-571` |
| 空 | legacy 条目 | "legacy 格式条目 — 无 strict 痕迹表（不静默升级）" | ✅ `page.tsx:668-669` |
| 错误 | profile 加载或注册失败 | 红色 status 文案 | ✅ `page.tsx:426`、`:441` |
| 错误 | 监听扫描错误 | 最多三条红色文案 | ✅ `page.tsx:443-445` |
| 错误 | 编辑者变更被拒（legacy 或未找到） | 红色 status 文案 | ✅ `page.tsx:659-662`；`nemaStore.ts:413-421` |
| 未授权 | 无会话或权限不足 | 应显示未授权 | ⬜ 页面不做会话或权限判断；无会话时以 `usr-001` 显示（`nemaCollaboration.ts:106-107`） |
| 未接入 | 实时协作不可用 | 共同编辑区块顶部"实时协作未接入" | ✅ `page.tsx:564-566`（`StatusNotice` disconnected） |
| 未接入 | 无实时源 | 编辑者状态"未接入（实时协作未接入）"或"未开始" | ✅ `page.tsx:615-627`；`nemaCollaboration.ts:83-86` |
| 未接入 | 协作组 A、B | 标注"(示意, 实时协作未接入)" | 🟡 标注存在（`page.tsx:522`、`:540`），内容仍为静态示意 |
| 禁用 | 未选择 profile | 注册按钮禁用 | ✅ `page.tsx:432` |
| 禁用 | 编辑者名为空 | 添加按钮禁用 | ✅ `page.tsx:653` |
| 禁用 | 推进、邀请、CLI 按钮 | 应禁用并说明未接入 | ⬜ 三个按钮既不禁用也无处理函数（`page.tsx:690-698`） |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换"按任务卡 / 其他" | 无 | 切换任务列表 | 页面本地状态（`page.tsx:120`、`:296-311`） | ✅ |
| 选择共同编辑任务 | nemaTasks 非空 | 切换编辑者列表并重新读取痕迹 | `readNemaEditTrail`（`page.tsx:243`，`nema.ts:871`） | ✅ |
| 打开（已开卡项） | workItemId 已存在 | 跳转 `/work-item/{id}` | 路由 `frontend/src/app/(app)/work-item/[id]/page.tsx`（存在，未运行验证） | ❓ 需在浏览器中点击确认 |
| 开卡 | 无 workItemId | 在主 store 创建 WorkItem 并跳转 | `openNemaAsStarTask`（`nemaStore.ts:297-342`），租户与项目写死 | 🟡 |
| 关联共享文件（展开表单） | 无 | 展开或收起表单 | 页面本地状态（`page.tsx:401-403`） | ✅ |
| 选择 NEMA profile | profiles 已加载 | 更换注册所用语法版本 | WASM 列表（`page.tsx:140`、`:413-425`） | ✅ |
| 选择本地文件 | 本地模式 | 把路径填入表单 | `/api/v1/workspace-files`（`localFiles.ts:71`） | 🟡 仅本地模式 |
| 注册并生成 .nema | 已选 profile | 注册；少于 2 人时不生成，表单仍关闭 | `registerSharedFile`（`page.tsx:221`；`nemaStore.ts:255-295`） | 🟡 单人时静默（FR-11-18） |
| 添加编辑者 | 已选任务，名称非空 | 追加编辑者与痕迹 | `updateNemaEditors`（`page.tsx:255-269`；`nemaStore.ts:412-468`） | 🟡 无授权核验（FR-11-16） |
| 移除编辑者 | 已选任务 | 移除编辑者与痕迹 | `updateNemaEditors`（`page.tsx:271-283`） | 🟡 无授权核验（FR-11-16） |
| 推进到 ③ 语义合并 | — | 应推进流水线 | 无（`page.tsx:690-692`） | ⬜ |
| 邀请 Agent 加入 | — | 应添加 Agent 为编辑者 | 无（`page.tsx:693-695`） | ⬜ |
| 在 CLI 中打开 | — | 应在 CLI 中打开 | 无（`page.tsx:696-698`） | ⬜ |
| 租约写入 | — | 静态说明，无操作 | 无（`page.tsx:514-517`） | ⬜ |

数据:

| 来源 | 提供的字段 | 现状 |
|---|---|---|
| zustand persist `star-nema-files`（`nemaStore.ts:470`） | nemaTasks 全部字段 | ✅ 仅当前浏览器 |
| WASM 解释器（`nema.ts`） | profiles、grammar_revision、plugin_id、plugin_version | ✅ |
| 主 store（`lib/store`） | identities（id、display_name、email、type） | ✅ |
| `readNemaEditTrail`（`nema.ts:871-882`） | 痕迹条目（editor、edited_at、change） | ✅ 无 profile 时返回 null |
| `nemaEditorPresence`（`nemaCollaboration.ts:70`） | 状态种类与标签 | ✅ 不接入 live 源 |
| `useNemaWatcher`（`nemaWatcher.ts:143`） | lastResult、lastScanAt、watching | 🟡 20 秒轮询，仅本地模式 |
| 写死的预览文档（`page.tsx:161-172`） | 规约预览的全部行 | 🟡 示例，非所选任务的 source |
| 写死的代码行与标签（`page.tsx:91-116`、`:292`、`:343`、`:350`、`:383`、`:473`、`:499`） | 代码栏、工作集标题、计数、漂移文案 | 🟡 示例数据 |

与文档设计的差距:

- 规约预览显示写死的示例文档，不是所选任务的 `.nema` 源（`page.tsx:161-172` → `:190-194` → `:483-490`）。违反 AGENTS.md:22（FR-11-22、FR-11-29）。
- 代码栏为写死的 `bfs.rs` 样例，漂移徽标为固定文字（`page.tsx:103-116`、`:494-511`、`:499`）（FR-11-29）。
- 协作组 A、B 为静态示意（`page.tsx:519-558`）（FR-11-24）。
- 流水线固定在第 2 步，推进按钮无处理（`page.tsx:462`、`:690-692`）（FR-11-23）。
- 邀请 Agent 与在 CLI 中打开两个按钮无处理（`page.tsx:693-698`）（FR-11-25、FR-11-26）。
- 租约写入的说明文字存在，但写入路径不检查租约（`page.tsx:514-517`；`nemaStore.ts:412-468`）（FR-11-27）。
- 单人登记时表单静默关闭（`page.tsx:221-225`；`nemaStore.ts:256-259`）（FR-11-18）。
- 文件列表"人数"为登记编辑者数，不是在线人数（`page.tsx:361`）（FR-11-22）。
- 代码列表包含所有任务（含文档与表格），文档区又重复显示（`page.tsx:208-215`）；`CODE_FILES`、`DOC_FILES` 未被使用（`page.tsx:91-101`）（FR-11-22、FR-11-32）。
- 顶栏实时点恒为绿色（`AppHeader.tsx:179`），与"实时协作未接入"矛盾（FR-11-13）。
- 本页不做会话与授权闸门，也不做租户标识（FR-11-01、FR-11-16）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 2b 协同页 `/nema` | `frontend/src/app/(app)/nema/page.tsx` | 🟡 | 渲染并接线真实 store；示例内容与真实数据混排（§4.7.1） |
| .nema 侧车 store | `frontend/src/lib/nema/nemaStore.ts` | ✅ | 注册、开卡、同步、编辑者变更、痕迹；浏览器持久化（`:470`） |
| 编辑者状态与身份 | `frontend/src/lib/nema/nemaCollaboration.ts` | 🟡 | 诚实降级已实现（`:70-87`）；live 源未接入（`:61`） |
| 磁盘监听 | `frontend/src/lib/nema/nemaWatcher.ts` | 🟡 | 本地模式 20 秒轮询（`:137`、`:164`） |
| .nema 解析与痕迹 | `frontend/src/lib/nema/nema.ts` | ✅ | `readNemaEditTrail`（`:871-882`）；WASM 渐进加载（`:892`） |
| 本地文件浏览 | `frontend/src/components/nema/LocalFolderBrowser.tsx`，`frontend/src/lib/localFiles.ts` | 🟡 | 调用 workspace-files（`localFiles.ts:71`、`:78`），网页版降级 |
| 本地文件 API | `crates/star-api-rest/src/workspace_files.rs`；`main.rs:159-161` | ✅ | 生产挂载，受控只读 |
| .nema 编解码（Rust） | `crates/nema-codec/src/lib.rs`；`crates/nema-codec-wasm/src/lib.rs` | 🟡 | 编解码经 WASM 使用；租约仲裁器未导出 |
| 领域协作 crate | `crates/domain-collaboration/src/lib.rs` | 🟡 | 内存服务与 17 个测试；无持久适配器、无路由、无消费者 |
| 文件协调与租约内核 | `crates/ide-collab-core/src/coordinator.rs` | 🟡 | 库存在（claim、heartbeat、commit、reap）；无路由；无测试 |
| 协作会话内核 | `crates/ide-collab-core/src/collaboration.rs` | 🟡 | 库；3 个测试；仅被演示端点使用 |
| 协同演示端点 | `crates/star-api-rest/src/routes/integrations.rs:39-86`；`lib.rs:182`、`:215-219` | 🟡 | 生产挂载，无鉴权，返回演示数据；前端无调用方 |
| NEMA 所有权仲裁器 | `crates/nema-codec/src/lib.rs:3417-3534` | 🟡 | 库；1 个测试（`:3762`）；未导出到 WASM |
| 画布事件流（F06 所有） | `crates/star-api-rest/src/group_api/canvas_event_stream.rs`；`group_api.rs:824-825`；`main.rs:106-108` | ✅ | 生产挂载；F11 复用候选 |
| A12 BFF WebSocket hub（F08 所有） | `bff/src/collaboration/wss_hub.rs`；`controller.rs:61-76` | 🟡 | 库；`bff` 是独立 workspace（`bff/Cargo.toml:21`），无可执行入口，无依赖方；生产者仅在测试中调用（`wss_hub.rs:368`） |
| star-sse 事件路由 | `crates/star-sse/src/lib.rs:1-8` | 🟡 | 内存存储，注释标注 Phase F；无消费者 |
| canvas-realtime 服务 | `crates/canvas-realtime/src/lib.rs:73-90` | 🟡 | 骨架：`Phase::Skeleton`（`:80`），协作操作返回 `NotImplemented`（`:87-88`） |
| 设备撤权 inbox | `crates/star-api-rest/src/device_outbox/collaboration_inbox.rs`；`device_outbox/mod.rs:50-52` | 🟡 | 库与 PostgreSQL 实现；生产二进制未构造；relay 无生产调用方 |
| 协作相关 DDL | `db/migrations/2026-11-14-w17-device-outbox-inbox.sql:53-59` | 🟡 | 设备 inbox 表存在；无 presence、subscription、草稿、冲突表 |
| presence 设计 | `docs/data-design/ipa-detail/tables/collaboration_presence.md` | ⬜ | 设计文档，TTL 5 分钟，无 DDL |
| 共享协作 WebSocket | — | ⬜ | 产品代码中无 `star.v1` |
| MCP 协作资源 | `crates/star-mcp/src/handlers/collaboration.rs:28-36` | 🟡 | Phase H mock，返回固定参与者 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 示例内容与真实数据混排：规约预览、代码栏、协作组、漂移徽标、计数、工作集标题均为写死值（FR-11-22、FR-11-24、FR-11-29） | 用户把示例当成真实状态，违反 AGENTS.md:22 | 以 nemaTasks 驱动所有区块；无数据时显示"未接入"占位；示例移入测试夹具 |
| 流水线固定在第 2 步，推进按钮无处理（FR-11-23） | 流水线不反映状态，按钮看似可用 | 由状态驱动阶段；推进按钮禁用并说明未接入 |
| 邀请、CLI、推进三个按钮无处理（FR-11-25、FR-11-26、FR-11-23） | 操作无反馈 | 禁用并标注未接入原因；接入后再启用 |
| 写入路径无租约门禁（FR-11-27） | 两人同时修改同一侧车会互相覆盖 | 把 `ide-collab-core` 的 claim 与 commit 接到 `.nema` 写路径，并为 coordinator 补测试（当前 0 个） |
| 编辑者变更与注册无授权核验，操作者仅为显示名（FR-11-16） | 任何人可改协作者名单 | 写入路径取服务端会话并复验 owner 授权 |
| 持久化仅在浏览器 localStorage（FR-11-34） | 协作者之间互不可见 | 建立 owner 表与事务写入，经服务端端点读写 |
| 无 Presence 与订阅实现；领域层 presence 为 30 秒，设计为 60 秒（FR-11-02、FR-11-03、FR-11-13） | 在线状态无真实来源，页面只能显示未接入 | 把 `domain-collaboration` 接成带适配器的服务，统一 60 秒阈值，经 WebSocket `star.v1` 推送 |
| `domain-collaboration` 无消费者、无持久适配器、无审计依赖（FR-11-01、FR-11-35） | 领域规则不生效 | 定义 owner 端口与审计端口，接入路由 |
| 冲突与草稿协议未实现（FR-11-08 至 FR-11-10） | 重叠编辑会静默覆盖（`nemaStore.ts:412-468`） | 按规范 §13.3 在 owner 事务中实现草稿封存与冲突记录 |
| `.nema` 磁盘扫描每 20 秒轮询（FR-11-05、FR-11-21） | 违反 INV-CB-07 与 AC-COLLAB-01 的非轮询要求 | 改为 owner 事件唤醒，复用画布唤醒模式 |
| 演示端点在生产挂载且无鉴权（FR-11-01） | 对外暴露演示光标与标识 | 移除，或加鉴权并标注为演示 |
| 顶栏实时点恒为绿色（FR-11-13） | 与"实时协作未接入"矛盾 | 绑定真实连接状态（与 F01 共同闭合） |
| 单人登记静默关闭表单（FR-11-18） | 用户误以为已登记 | 检查注册返回值并给出提示 |
| 开卡写死租户、项目、报告人（FR-11-01、FR-11-20） | 数据归属错误，跨租户风险 | 从会话与 Project 上下文取值 |
| NEMA 所有权仲裁器未导出到 WASM（FR-11-27） | 前端无法使用 Observer、Lease、Epoch 状态机 | 导出绑定，并接入写路径 |
| 语义合并、代码生成、验证无实现（FR-11-28 至 FR-11-30） | 流水线第 3 至第 5 步无后端 | 按 REUSE-MAP-COLLAB-SANDBOX-001 §2 移植，移植前完成 §4 许可审查 |
| 常量 `CODE_FILES`、`DOC_FILES` 未使用，代码列表包含文档（FR-11-22、FR-11-32） | 同一文件重复显示，死代码 | 删除未使用常量；代码列表按类型过滤 |
| 画布事件流不覆盖 `.nema` 与任务（FR-11-04、FR-11-07） | 文件协作没有推送 | 评估复用画布事件流的唤醒模式，或为 `.nema` 建立 owner 事件 outbox |
| 设备撤权 inbox 未在运行时构造（FR-11-04） | 撤权不会关闭协作订阅 | 在 owner 入口接线，并验证 gap 恢复 |

## 7. 验收与测试

已有测试（静态读取，未运行）:

- `frontend/src/lib/nema/nemaCollaboration.test.ts`，8 个用例:
  - 追加编辑者后 `.nema` 源更新、写入痕迹，strict v2 往返合法（`:65`）。
  - 移除编辑者后源更新并写入移除痕迹（`:125`）。
  - 编辑者不变返回 unchanged，不追加痕迹（`:147`）。
  - 变更后已绑定 WorkItem 的标题与描述回写（`:163`）。
  - legacy 条目不被静默升级（`:184`）。
  - 无实时连接时不显示"正在编辑"（`:215`）。
  - 无痕迹的编辑者显示"未开始"（`:224`）。
  - 只有真实 presence 源能给出 editing 或 offline（`:230`）。
- `frontend/src/app/(app)/nema/__tests__/nemaPage.collaboration.test.tsx`，2 个用例:
  - 共同编辑区块渲染，并显示"实时协作未接入"提示（`:42-48`）。
  - 登记后的编辑者显示"未开始"，不显示"正在编辑"；添加、移除与痕迹区块存在（`:50-69`）。
- `crates/domain-collaboration/src/lib.rs`，17 个 `tokio::test`（`:1418-2109`）:
  - start_session、end_session、已结束会话拒绝写入、跨 tenant 拒绝（`test_cross_tenant_session_denied`，`:1521`）。
  - presence 心跳刷新、已结束会话的 presence 失活、列出活跃 presence。
  - cursor 更新与颜色唯一（`:1657`、`:1714`）。
  - 白板形状的增、改、删、列出与非法尺寸拒绝（`:1767-2032`）。
  - 管理员可结束他人会话；非所有者不可结束（`:2072`、`:2109`）。
- `crates/ide-collab-core/src/collaboration.rs`，3 个 `#[test]`:
  - `test_collaborator_color_deterministic_hashing`（`:416`）：相同 ID 得到相同颜色。
  - `test_keystroke_broadcast_and_attribution`（`:426`）：按键广播与归属。
  - `test_anchor_offset_table_shifting`（`:478`）：锚点偏移随分区变化。
- `crates/nema-codec/src/lib.rs`，1 个租约相关测试：`test_lease_arbitrator_epoch_and_idempotency`（`:3762`），断言初始角色为 Observer，并检查 epoch 与幂等。
- `crates/ide-collab-core/src/coordinator.rs`：无任何 `#[test]`。
- `crates/star-mcp/src/handlers/collaboration.rs`：`read_test`，断言 mock 返回的 collab_id。

已验证（静态）:

- 诚实降级规则与编辑者状态推导的分支（`nemaCollaboration.ts:70-87`）。
- 侧车注册、编辑者变更、痕迹与开卡的数据流（`nemaStore.ts`）。

未验证:

- 运行中的 `/nema` 页面（未在浏览器中打开，未点击任何按钮）。
- 示例内容与真实数据的分离（无测试覆盖）。
- 租约写入与 fencing 的任何路径（无产品测试，coordinator 无测试）。
- `/api/v1/collab/session` 与 `/api/v1/collab/annotations` 的响应（无测试，无前端调用方）。
- WebSocket、订阅、冲突、草稿与 CatchUp 的任何路径（代码不存在）。
- 顶栏实时点与页签标签的运行时显示。
- `nema-codec-wasm` 与 TypeScript 实现的解析一致性（nema.test.ts 未在本文件范围内逐条核对）。

## 8. 来源与裁决

| 源文档或代码 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/specs/domain-collaboration-spec.md`（v1.6） | Presence、订阅、INV-CB-01 至 09、§13 草稿与冲突协议、AC-COLLAB-01 至 09 | §13 称 Group 投影仍用 `CanvasOutboxPoller.pollOnce`；代码中该符号不存在（R1）。AC §10 的离线阈值与 INV-CB-03 不一致（R2）。crate 头注释引用的"§10.3"在 v1.6 中不存在 | 设计意图采用 v1.6（最新已接受版本）；实现状态以代码为准 |
| `docs/prototypes/2b · 协同（文件即任务 · 多人多 Agent）-html/README.md`、`Docs.dc.html` | 布局、五步流水线、租约说明文案、协作组、语义合并、验证面板 | 原型为示意稿，内容为样例；与代码的按钮与状态不对应 | 作为 UX 参考；不作为实现状态证据 |
| `docs/requirements.md` §15（`:533-537`）与 v7.03 修订说明（`:15`） | REQ-RT-001 至 003 | `:15` 称 Group 投影仍实例化 `CanvasOutboxPoller` 并周期调用 `pollOnce`，与代码不符（R1） | 需求采用 REQ-RT；状态以代码为准 |
| `docs/ac-test-matrix.csv:25-27` | REQ-RT-001 至 003 的状态为 gap | 仅状态记录，无实现证据 | 作为需求对照，未采信为实现证据 |
| `docs/basic-design.md:381` | domain-collaboration 的职责（实时状态、Presence、订阅） | 无 | 采纳职责边界 |
| `docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md:2-3` | owner-local 目标结构，状态为目标设计 v0.8，未生成迁移 | 与规范 §13 对齐 | 作为目标设计；标记为 ⬜ |
| `docs/data-design/ipa-detail/tables/collaboration_presence.md:22` | presence 表设计，TTL 5 分钟，Valkey 缓存 | TTL 与规范 60 秒不一致（R2） | 不采用为阈值；记为设计草案 |
| `docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md` §2（`:23-40`）、§5 G-2 | 复用计划：nema-codec 直接复用，`nema_pipeline` 适配，coordinator 适配，file_lock 适配；协作广播需改为持久事实与 Outbox | 源仓 `E:\IDE1.0` 与 BD-002 设计文档不在本仓内 | 作为复用意图；不作为状态证据（R5） |
| `docs/features/F12-file-session.md:3`、`:228` | FileSession 未实现；`.nema` 文件即任务引用 nemaStore | F12 与 F11 对 nemaStore 的归属需划分 | `.nema` 与 nemaStore 归 F11；FileSession 归 F12（R6） |
| `docs/features/F08-agent-graph-canvas.md:68-69`、`:92` | A12 BFF 与 canvas-collab 归属 | 无 | A12 归 F08；F11 只引用（R6） |
| `docs/features/F06-unified-canvas.md:101`、`:211`、`:219` | `/collaboration` 画布列表与 canvas-collab 骨架归属 | 无 | 归 F06（R6） |
| `docs/features/F01-nav-project-lane.md:62`、`:113` | 顶栏页签与旧路由重定向 | 无 | 顶栏实时点与页签归 F01（R9） |
| `docs/features/F04-task-board.md:51` | "按文件开卡"读取 nemaStore；看板轮询 | 无 | 相邻引用，归 F04 |
| `AGENTS.md`（pin，`:22`、`:42`、`:46`） | 未接通授权数据须显示未接入；实时协作以 owner 事务与 Outbox 推送，不做 last-write-wins；CLI 闭环验收 | 硬约束，不是状态证据 | 作为约束，用于 §3 与 §4.5 |
| `frontend/src/app/(app)/nema/page.tsx` | 页面结构、按钮、状态文案 | 示例与真实数据混排 | 以代码为准 |
| `frontend/src/lib/nema/nemaCollaboration.ts`、`nemaStore.ts`、`nemaWatcher.ts` | 诚实降级、store 动作、监听 | 无授权、无租约、无租户 | 以代码为准 |
| `frontend/src/lib/nema/nema.ts`（`:108`、`:613`、`:871-892`） | 默认 profile、共享文件文档、痕迹读取、WASM 加载 | 仅读取所列行 | 以代码为准 |
| `frontend/src/lib/nema/nemaCollaboration.test.ts`、`frontend/src/app/(app)/nema/__tests__/nemaPage.collaboration.test.tsx` | 诚实降级与登记行为 | 未运行 | 见 §7 |
| `frontend/src/lib/nav/registry.ts:574-580`、`navStore.ts:93-94` | nema 模块定义与默认顶栏页签 | 原型标签为"协同"，registry 标签为"NEMA Tasks" | 以代码为准（R9） |
| `frontend/src/components/AppShell.tsx`、`AppHeader.tsx`、`frontend/src/app/(app)/layout.tsx` | 布局区域、实时点、全高页面列表 | 无全局侧栏 | 以代码为准 |
| `frontend/src/lib/redirects.ts:71`、`:93`；`redirects.shim.cjs:37` | `/collaboration` 与 `/work-item` 重定向 | `/work-item` 仅匹配精确路径 | 以代码为准；`/work-item/[id]` 目录存在 |
| `frontend/src/app/(app)/work-item/[id]/page.tsx` | 任务详情路由 | 未运行验证 | 存在；运行时结果列为 ❓ |
| `frontend/src/lib/localFiles.ts:71`、`:78`；`components/nema/LocalFolderBrowser.tsx:12` | 本地文件 API 调用 | 无 | 以代码为准 |
| `frontend/src/lib/group/worktreeGroupApi.ts`（`:1286`、`:1304`、`:1397`、`:1482-1485`、`:1559`、`:1576`、`:1591`、`:1637`）、`groupProjection.tsx:218` | 画布唤醒订阅与退避重连 | 与规范 §13 的"轮询"描述不符（R1） | 以代码为准；归 F06 |
| `frontend/src/hooks/useBoardSync.ts:4` | 看板 2 秒轮询 | 与 INV-CB-07 不符 | 归 F04 |
| `frontend/src/app/collaboration/page.tsx` | 旧画布列表页，读取 store 的 presenceCursors | 已被重定向覆盖，不可达 | 归 F06；不计入 F11 |
| `crates/domain-collaboration/src/lib.rs`、`Cargo.toml` | 会话、presence、光标、白板；内存服务与 17 个测试 | 实体与规范不同（R3） | 作为待适配骨架；标 🟡 |
| `crates/ide-collab-core/src/lib.rs`、`coordinator.rs`、`collaboration.rs` | 从 IDE1.0 `ide-agent-router` 移植的内核；租约、CAS、协作会话 | coordinator 无测试；无路由 | 作为库层实现；标 🟡 |
| `crates/nema-codec/src/lib.rs:3382-3534`、`:3762` | 所有权仲裁器（Observer、Lease、Epoch） | 未导出到 WASM | 标 🟡（R4） |
| `crates/nema-codec-wasm/src/lib.rs`（检索） | 编解码绑定 | 无租约符号 | 作为依赖说明 |
| `crates/canvas-realtime/src/lib.rs:64-90` | 服务骨架与 Phase 标记 | 无协作实现 | 标 🟡 |
| `crates/star-sse/src/lib.rs:1-8`（检索消费者） | 内存事件路由 | 无消费者 | 标 🟡 |
| `bff/Cargo.toml:21`、`bff/src/collaboration/wss_hub.rs`、`controller.rs` | A12 hub 与路由 | 独立 workspace，无可执行入口，无依赖方 | 归 F08；标 🟡 |
| `crates/star-api-rest/src/main.rs`、`lib.rs`、`group_api.rs:806-829`、`canvas_event_stream.rs`、`canvas.rs:430`、`routes/integrations.rs`、`device_outbox/*` | 生产路由组合、画布事件流、演示端点、撤权 inbox | 演示端点无鉴权；inbox 未构造 | 以代码为准 |
| `crates/star-mcp/src/handlers/collaboration.rs` | MCP 资源 mock | Phase H mock | 标 🟡 |
| `db/migrations/2026-11-14-w17-device-outbox-inbox.sql:53-59`；`2026-09-10-p3d6-15-more-tables.sql:346`（检索） | 设备 inbox 表；画布 presence 表 | 无协作 presence、订阅、草稿、冲突表 | 标 🟡 与 ⬜ |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | 规范引用的 E2E-COLLAB 验收计划 | 未读取，只确认存在 | 置于 §6 open_items |

裁决记录:

- R1（Group 投影的实现状态）：规范 §13 与 requirements v7.03 称 Group 投影仍周期调用 `CanvasOutboxPoller.pollOnce`。pin 中该符号不存在，投影由 `CanvasOutboxWakeSubscriber` 唤醒驱动（`groupProjection.tsx:218`）。裁决：状态以代码为准；规范描述不采纳为现状。
- R2（presence 阈值）：INV-CB-03 与规范 §2 定义为 60 秒；AC §10 写作"≥ 300 秒区间"；T53 设计为 5 分钟；领域代码为 30 秒（`lib.rs:307`）。coordinator 的 60 秒是租约时长，不是 presence。裁决：设计值采用 INV-CB-03 的 60 秒（直接约束，且 v1.6 为最新版本）；代码 30 秒列为差距；AC 与 INV 的不一致列入 open_items。
- R3（领域模型）：规范实体为 Presence、RealtimeSubscription、RealtimeEventPayload；crate 为 CollaborationSession、Presence、Cursor、Whiteboard，父实体为 WorkItem、Document、Whiteboard。裁决：规范实体标 ⬜；crate 作为待适配骨架，标 🟡。
- R4（并发控制）：原型以文件租约为写入门禁，人可随时接管（`page.tsx:514-517`；`Docs.dc.html:134`）。规范 §13 与 AGENTS.md:42 要求 owner CAS、不做 last-write-wins、双草稿保全。裁决：租约作为写入权门禁，叠加在 owner CAS 之上：租约决定谁可以提交，CAS 决定提交是否基于最新版本，重叠时仍生成冲突；租约不得覆盖未封存的草稿。规范与原型都未定义租约与冲突的交互，列入 open_items。
- R5（流水线归属）：REUSE-MAP §2 把 ②至⑤ 指向 IDE1.0 的 `nema_pipeline`，仓内不存在。裁决：列为 ⬜；REUSE-MAP 作为复用意图；移植前按 REUSE-MAP §4 完成许可审查。
- R6（功能归属）：`.nema` 与 nemaStore 归 F11；FileSession 归 F12（F12 自述未实现）；A12 BFF 与 canvas-collab 归 F08；`/collaboration` 与画布事件流归 F06（画布事件流可复用）。
- R7（优先级）：本文件的分配规则见 §3，原型与规范均未给出优先级。
- R8（状态规则）：§3 按产品路径判定；仅库代码时该需求为 ⬜，库代码证据见 §5 与 §6。
- R9（顶栏标签）：原型顶栏标签为"协同"；代码注册名为"NEMA Tasks"（`registry.ts:575`）。运行时显示以 `AppHeader` 的模块翻译结果为准，未在运行时核对（open_items）。
- R10（实时状态的显示语义）：原型显示"在线人数"，代码只显示"未开始 / 未接入"。裁决：不采用原型的在线人数语义，直到有真实 presence 源。
