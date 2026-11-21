# F14 原型

> 状态摘要: 🟡 `/prototype` 页面已接入 GemArc 六步工具流水线 (顺序调用、首个失败即停)、GemArc 本地与远端通道健康检查、设备与数据来源切换; 生成类型选择只切换按钮高亮, 不进入流水线与预览; 引用上下文、右侧 1D–5D 预览、导出与分享、交互预览、从模板与版本记录仍为静态展示或无行为控件。
> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

在 Branch 功能页签 `/prototype` 中, 用户输入原型描述、选择生成类型与设备, 由 GemArc MCP 依次执行 建模 → 校验 → 推演 → 构建 → 验证 → 导出 六个工具, 并在页面查看流水线输出与 1D–5D 交互预览。生成结果驱动预览, 未接通的数据来源显示未接入状态, 不以示例数据冒充生成结果。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | `/prototype` 页面 (`frontend/src/app/(app)/prototype/page.tsx`); GemArc 代理 `/api/gemarc` (`frontend/src/app/api/gemarc/route.ts`); 顶栏页签与导航注册 (`frontend/src/lib/nav/registry.ts`, `navStore.ts`); 页签标题与副标题的三语文案 |
| 范围外 | GemArc MCP 服务本体 (仓外 `E:\GemArc`, 本仓无实现; 集成映射见 `docs/architecture/2026-11-14-github-integration/INTEGRATION-MAP-GITHUB-REPOS-001.md:17`); Physis 3D 物理引擎 (同文件第 29 行列为后续集成项); 任务卡与画布的业务逻辑 (本页只提供跳转); 原型的持久化与版本服务 |

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 产品 / 设计者 | 用自然语言描述原型, 选择生成类型, 查看 1D–5D 预览与流水线输出 |
| 开发者 | 运行六步 GemArc 流水线, 检查每步输出与失败位置 |
| 运维 / 管理者 | 确认本地 GemArc 与远端 GemArc 通道是否在线, 选择示例数据或当前 Branch 接口 |

主要用户故事:

- US-1 输入描述, 点击生成, 依次看到六步流水线日志。
- US-2 切换手机 / 桌面设备, 预览卡宽度随之变化。
- US-3 在示例数据与当前 Branch 接口之间切换 GemArc 通道, 并看到通道在线状态。
- US-4 将原型附加到任务卡或放进画布白板 (当前仅跳转)。
- US-5 导出或分享原型 (未实现)。

## 3. 需求

优先级口径: P0 核心闭环, P1 闭环补充, P2 增强。编号 FR-14-NN 与来源 SRS-PROTOTYPE-001 的 FR-PT 对应关系见 §8。

状态口径: 🟡 指需求的展示或某项操作已由代码实现并有证据、其余子项缺失; ⬜ 指需求的操作不存在, 页面仅有静态文本或无处理函数的控件。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-14-01 | 生成类型选择 (UI、2D、3D、4D、5D、1D、动效、图片与视频), 选择结果驱动生成与预览 | P0 | 🟡 | 类型按钮与高亮 `page.tsx:82-91, 146-165`; `genType` 未进入流水线参数 `page.tsx:125-129` 也不影响预览 `page.tsx:247-326` |
| FR-14-02 | 引用上下文 (文件、截图、任务卡) 可添加并随生成提交 | P1 | ⬜ | 两条引用为硬编码文本 `page.tsx:170-177`; "+ 文件 / 截图 / 任务卡" 按钮无 onClick `page.tsx:179`; 引用未进入调用参数 `page.tsx:125-129` |
| FR-14-03 | 生成流水线按 建模/校验/推演/构建/验证/导出 映射到 GemArc 工具, 顺序执行, 首个失败即停止 | P0 | ✅ | 步骤表 `page.tsx:57-64`; 顺序循环与失败停止 `page.tsx:120-138` (失败分支 `page.tsx:131-135`); 白名单代理 `route.ts:25-38, 49-56` |
| FR-14-04 | 流水线阶段高亮随当前执行步骤与结果变化 | P1 | 🟡 | 阶段高亮为静态条件 `i <= 3` `page.tsx:225`; `runningStep` 仅用于禁用按钮与旋转图标 `page.tsx:208-213` |
| FR-14-05 | 每步 GemArc 输出完整展示 | P1 | 🟡 | 每步输出截断为 300 字符 `page.tsx:130`; 日志区 `page.tsx:199-203` |
| FR-14-06 | 右侧 1D–5D 预览卡展示 (UI、2D、3D、4D、5D、1D、动效) | P0 | 🟡 | 七张预览卡存在 `page.tsx:247-325`; 内容为固定文本与示例数字 (3D 仅文字 `page.tsx:283-284`; 5D 固定值 `page.tsx:304-305`); UI 卡宽度随设备切换 (`page.tsx:249`) |
| FR-14-07 | 预览内容由生成结果驱动 | P0 | ⬜ | 预览区域未引用 `pipelineLog`、`runningStep` 或任何流水线输出 `page.tsx:247-326` |
| FR-14-08 | 示例数据与未接通数据显式标注, 不得把示例当作生成结果 | P0 | 🟡 | 数据来源以文字标注 "数据来自示例" 或 "数据来自当前 Branch 的接口" (`page.tsx:196`); 缺口: 同段固定声明 "已生成 3 个页签的可点击原型" 不论流水线是否执行 (`page.tsx:196`), 预览卡示例数值无标注 (`page.tsx:247-325`), 未接通态无文案; 依据 `AGENTS.md:22` (未接通授权数据须显示未接入状态, 不得以示例数据冒充) |
| FR-14-09 | 设备切换 (手机 390 × 844 / 桌面 1440 × 900) | P1 | 🟡 | 状态 `page.tsx:97`; 切换按钮 `page.tsx:347-352`; 预览卡宽度分支 `page.tsx:249` (手机 `w-[200px]`、桌面 `w-full`, 未使用 390 × 844 / 1440 × 900 尺寸); 设备写入流水线 note `page.tsx:127` |
| FR-14-10 | 数据来源切换: 示例数据走远端 GemArc, 当前 Branch 接口走本地 GemArc, 并绑定当前 Branch | P0 | 🟡 | 切换按钮 `page.tsx:359-372`; 通道映射 `page.tsx:128`; 代理按 `_meta.source` 选择上游 `route.ts:59-61`; 项目固定为 `prototype-demo` `page.tsx:127`, 未绑定 Branch |
| FR-14-11 | GemArc 本地与远端健康检查, 在线状态显示 | P0 | 🟡 | 客户端探测 `page.tsx:105-117`; 状态文案 `page.tsx:373-375`, 两通道合并为单一在线指示 (`page.tsx:111-114`); 服务端并行探测 `route.ts:82-95` |
| FR-14-12 | 生成门控: 所选数据来源对应通道不在线时禁止生成 | P0 | 🟡 | 按钮禁用条件 `page.tsx:209`; 门控状态在任一通道在线时即为 up `page.tsx:111-114`, 与所选来源可能不一致 |
| FR-14-13 | 导出原型 (文件、HTML、视频) | P1 | ⬜ | 导出按钮无 onClick `page.tsx:236`; 页脚 "导出 HTML / 视频" 无 onClick `page.tsx:395-397`; 流水线中 `gemarc_render` 仅把文本写入日志 `page.tsx:63, 130` |
| FR-14-14 | 分享原型 | P2 | ⬜ | 分享按钮无 onClick `page.tsx:239` |
| FR-14-15 | 附加到任务卡, 原型内容随跳转传递 | P1 | 🟡 | 链接指向固定地址 `/kanban?view=%E6%8C%89%E6%96%87%E4%BB%B6%E5%BC%80%E5%8D%A1` `page.tsx:389`; 未传递原型数据 |
| FR-14-16 | 放进画布白板, 原型内容随跳转传递 | P1 | 🟡 | 链接指向固定地址 `/canvas/canvas-001` `page.tsx:392`; 未传递原型数据 |
| FR-14-17 | 交互预览: 可点击原型, 4D 时间线播放与停止, 5D 屏蔽开关, 3D 机柜选择 | P1 | ⬜ | "交互预览" 按钮无 onClick `page.tsx:232`; 对应交互仅存在于设计参考 `docs/prototypes/2d · 原型-html/Prototype.dc.html:136, 197-198, 205, 215, 218-222, 225-226` |
| FR-14-18 | 从模板新建原型 | P2 | ⬜ | "从模板" 按钮无 onClick `page.tsx:216` |
| FR-14-19 | 版本记录 (生成版本与说明) | P2 | ⬜ | 版本文本为硬编码 `page.tsx:380-385`; 无存储模型 |
| FR-14-20 | 图片与视频生成, 未接入时显示未接入态 | P2 | ⬜ | 类型按钮存在 `page.tsx:90`; 无生成逻辑; 无未接入态文案; 设计参考标注 "当前显示「未接入」" `Prototype.dc.html:203` |
| FR-14-21 | 动效预览 | P2 | ⬜ | 动效卡为静态文本 `page.tsx:319-324`; 无过渡播放逻辑 |
| FR-14-22 | 画布视图控制: 缩放、适配、图层、评论、主色 | P2 | ⬜ | 页脚的 80%、适配、图层、评论、主色均为静态文本或图标 `page.tsx:331-341`; 无处理函数 |
| FR-14-23 | 代理仅放行白名单工具与指定方法 (tools/call、initialize、tools/list) | P0 | ✅ | 白名单 12 项 `route.ts:25-38`; 方法与工具校验并返回 403 `route.ts:49-56` |
| FR-14-24 | 代理鉴权: 仅已登录用户可调用 `/api/gemarc` | P1 | ⬜ | `route.ts` 无鉴权逻辑; 中间件 matcher 排除 `api` 路径 `proxy.ts:39` |
| FR-14-25 | 上游调用超时与错误结构化返回 | P1 | ⬜ | 上游 POST 无超时与 try/catch `route.ts:63-71`; 客户端以非 2xx (`page.tsx:72`) 或 JSON-RPC `error` 字段 (`page.tsx:74`) 判定失败 |
| FR-14-26 | 远端 token 仅服务端持有, 仅附加到远端请求 | P0 | ✅ | 读取环境变量 `route.ts:22`; 仅在 `useRemote` 时附加 `Authorization` `route.ts:68` |
| FR-14-27 | 页面文案随界面语言切换 | P2 | ⬜ | 页面文案为硬编码中文 (如 `page.tsx:145`); 无 i18n 调用 `page.tsx:16-42`; 页签标题与副标题走 i18n `zh-CN.ts:680-683` |

## 4. 设计

### 4.1 概念模型

| 概念 | 说明 | 实现位置 | 状态 |
|---|---|---|---|
| 生成类型 (GenKind) | 8 个取值: `ui`、`2d`、`3d`、`4d`、`5d`、`1d`、`motion`、`media` | `page.tsx:82-91` | ✅ |
| 流水线步骤 (GemArcStep) | 6 个步骤, 每步绑定一个 GemArc 工具 | `page.tsx:51-64` | ✅ |
| 数据通道 (DataSource) | `sample` 对应远端 GemArc, `branch` 对应本地 GemArc | `page.tsx:98, 128`; `route.ts:59-61` | 🟡 |
| 设备 (Device) | `mobile` 或 `desktop`, 影响预览卡宽度与流水线 note | `page.tsx:97, 127, 249` | ✅ |
| 预览卡 | 右侧七张静态卡 (UI、2D、3D、4D、5D、1D、动效) | `page.tsx:247-325` | 🟡 |
| 原型实体、引用上下文、版本 | 无领域模型与持久化实体 | 原型业务逻辑仅位于 `page.tsx` 与 `route.ts`; `crates/` 中无 gemarc 或 `/prototype` 相关实现 (637ed19d 的 `git grep` 无结果) | ⬜ |

### 4.2 数据模型

全部页面状态保存在组件内存中, 刷新即丢失。

| 实体 / 结构 | 字段 | 来源 | 状态 |
|---|---|---|---|
| GemArcStep | id, label, tool | `page.tsx:51-55` | ✅ |
| 页面状态 | genType, device, dataSource, prompt, gemarcStatus, runningStep, pipelineLog | `page.tsx:96-102` (`useState`, 内存态) | ✅ |
| JSON-RPC 请求体 | jsonrpc, id, method, params.name, params.arguments, params._meta.source | `page.tsx:70`; `route.ts:49-61` | ✅ |
| 健康检查响应 | local / remote 对象; 在线时含 status 与 url, 离线时为 `{status: "down"}` | `route.ts:91-94`; 页面只读取 `status` `page.tsx:109-110` | 🟡 (`/healthz` 的其余字段结构由仓外服务决定, 见 §7) |
| 引用上下文 | 文件、截图、任务卡引用 | 硬编码展示 `page.tsx:172, 176` | ⬜ |
| 版本记录 | when, text | 硬编码 `page.tsx:382-384` | ⬜ |
| 原型实体 | 原型 id、版本、上下文引用 | 无 | ⬜ |

### 4.3 接口

| 接口 | 文件与行 | 说明 | 状态 |
|---|---|---|---|
| `GET /api/gemarc` | `route.ts:82-95` | 并行请求本地 `/healthz` (超时 4000 ms, `route.ts:84`) 与远端 `/healthz` (超时 8000 ms, `route.ts:87`); 失败时对应通道返回 `{status: "down"}` (`route.ts:92-93`) | ✅ |
| `POST /api/gemarc` | `route.ts:40-80` | JSON-RPC 代理; 放行 `tools/call` (白名单工具), `initialize`、`tools/list`; 其他方法返回 403 (`route.ts:54-56`) | ✅ |
| 工具白名单 | `route.ts:25-38` | 12 个工具: brief、contracts、assets、new_model、validate、simulate、diff、render、build、verify、inspect、media_doc_guide | ✅ |
| 页面调用的工具 | `page.tsx:57-64` | `gemarc_brief`、`gemarc_validate`、`gemarc_simulate`、`gemarc_build`、`gemarc_verify`、`gemarc_render`, 均在白名单内 | ✅ |
| 本地上游 | `route.ts:20` | 默认 `http://127.0.0.1:8760/mcp`; 可由 `GEMARC_MCP_URL` 覆盖 | 🟡 (配置存在; 服务是否运行见 §7) |
| 远端上游 | `route.ts:21` | 默认 `https://gem-arc.vercel.app/mcp`; 可由 `GEMARC_REMOTE_URL` 覆盖 | 🟡 (可用性未在仓内验证) |
| 远端凭证 | `route.ts:22, 68` | 设置 `GEMARC_REMOTE_TOKEN` 时以 Bearer 方式发送至远端 | ✅ |
| 客户端调用 | `page.tsx:66-80` | `callGemArc(tool, args, source)`: 向 `/api/gemarc` 发送 JSON-RPC; 拼接 `result.content[].text`; `error.message` 转为异常 | ✅ |
| 页签跳转 | `page.tsx:389, 392` | 两个 `<a>` 链接, 不携带原型数据 | 🟡 |

流水线工具名以页面代码为准 (`page.tsx:57-64`)。`INTEGRATION-MAP-GITHUB-REPOS-001.md` 第 17 行使用 "init→validate→simulate→build" 的说法, 第 36 行使用 "brief/inspect/validate/build/verify" 的说法, 两者与页面代码均不完全一致。

### 4.4 状态机与关键流程

GemArc 通道状态 (`gemarcStatus`): `checking` (初始, `page.tsx:100`) → `up` 或 `down` (`page.tsx:105-117`)。挂载时与每次切换数据来源时重新请求; 切换时不先回到 `checking`, 旧状态保留至响应返回。

门控表达式 (`page.tsx:109-114`) 实际等价于 `localUp || remoteUp`: 只要任一通道在线即为 `up`, 与所选数据来源无关。按钮禁用条件为 `runningStep !== null || gemarcStatus !== "up"` (`page.tsx:209`)。

生成流程 (`runPipeline`, `page.tsx:120-138`):

1. 清空日志 (`page.tsx:121`), 按 `GEMARC_PIPELINE` 顺序循环 (`page.tsx:122`)。
2. 每步设置 `runningStep` (`page.tsx:123`), 以 `{project: "prototype-demo", query: prompt, note: "device=..."}` 调用工具 (`page.tsx:125-129`), 通道按数据来源选择 (`page.tsx:128`)。
3. 成功时把输出前 300 字符追加到日志 (`page.tsx:130`)。
4. 任一步抛错时追加 "失败" 行, 清空 `runningStep` 并返回, 后续步骤不执行 (`page.tsx:131-135`)。
5. 全部成功后清空 `runningStep` (`page.tsx:137`)。

步骤状态当前只以日志行表达, 没有每步的结构化状态 (待定、运行中、成功、失败) 对应的界面高亮。

### 4.5 权限与安全

| 项 | 现状 | 证据 | 状态 |
|---|---|---|---|
| 工具白名单 | 只允许白名单工具与三个方法 | `route.ts:25-38, 49-56` | ✅ |
| 远端凭证 | 只在服务端读取, 只附加到远端请求 | `route.ts:22, 68` | ✅ |
| 代理鉴权 | 无; 中间件 matcher 排除 `api` 路径; 登录客户端 `getSession` 未被本路由引用 | `route.ts` 全文; `proxy.ts:39`; `frontend/src/lib/auth.ts` | ⬜ |
| 默认外发 | 默认数据来源为 sample, 用户描述 (`query`) 与设备 note 发往远端 `gem-arc.vercel.app`; 是否允许外发需产品确认 | `page.tsx:98, 127-128`; `route.ts:21` | 🟡 |
| 上游超时 | POST 上游请求无超时 | `route.ts:63-71` | ⬜ |
| 上游异常 | 上游 POST 无 try/catch; 异常由框架返回, 客户端按非 2xx 或 JSON-RPC `error` 记为失败 | `route.ts:63-71`; `page.tsx:72, 74` | ⬜ |
| 输出截断 | 每步日志截断 300 字符, 可能掩盖错误细节 | `page.tsx:130` | 🟡 |
| 本地上游地址 | 默认上游为回环地址 `127.0.0.1:8760` | `route.ts:20` | ✅ |

### 4.6 依赖关系

| 依赖 | 说明 | 位置 | 状态 |
|---|---|---|---|
| GemArc MCP 服务 (本地) | Streamable HTTP, 默认端口 8760 | `route.ts:20`; 实现在仓外 `E:\GemArc` | ❓ (见 §7) |
| GemArc MCP 服务 (远端) | Vercel 部署 | `route.ts:21` | ❓ |
| 导航注册表 | 页签 id `prototype`, href `/prototype`, 描述 | `registry.ts:604-611` | ✅ |
| 执行组 | 组成员包含 `prototype` | `registry.ts:771-775` (tabs 在 774 行) | ✅ |
| 默认顶栏页签 | `DEFAULT_HEADER_TABS` 包含 `prototype` | `navStore.ts:94` | ✅ |
| 全高布局前缀 | `/prototype` 在 `WORKSPACE_FULL_HEIGHT_PREFIXES` 中 | `(app)/layout.tsx:28-38` (`/prototype` 在 33 行) | ✅ |
| 外壳 (顶栏、主区、右侧聊天栏) | 由 AppShell 提供 | `AppShell.tsx:48-50, 57-73, 77` | ✅ |
| 侧栏 | 根布局挂载 Sidebar | `frontend/src/app/layout.tsx:76-79`; `Sidebar.tsx:47` | ✅ |
| 移动端重定向 | 配置 `STAR_FLUTTER_WEB_URL` 时, 移动 UA 访问 `/prototype` 会被 301 到 Flutter 版 | `proxy.ts:22-34` | 🟡 (未配置时放行) |
| 页签文案 (i18n) | 标题与副标题 | `zh-CN.ts:680-683`; `en.ts:665-668`; `ja.ts:666-669`; 类型 `dictionary.ts:582-585` | 🟡 (副标题描述为画廊, 见 §6) |
| 图标与样式 | lucide-react、clsx, Tailwind 语义类 | `page.tsx:17-42` | ✅ |
| 设计参考 | 布局与交互参考, 非生产代码 | `docs/prototypes/2d · 原型-html/Prototype.dc.html` | ✅ 文件存在, 仅作参考 |

### 4.7 页面设计

### 4.7.1 /prototype

文件: `frontend/src/app/(app)/prototype/page.tsx` (401 行, `"use client"` 于第 14 行)。该路由目录仅有此文件; 其父级 `(app)/` 与 `app/` 下无 `error.tsx`、`loading.tsx` 或 `global-error.tsx`。

布局 (区域与提供方):

| 区域 | 提供方 | 位置 |
|---|---|---|
| 侧栏 | 根布局 Sidebar | `frontend/src/app/layout.tsx:76-79`; `frontend/src/components/Sidebar.tsx:47` |
| 顶栏 | AppShell 中的 AppHeader (Suspense 包裹) | `frontend/src/components/AppShell.tsx:48-50`; `AppHeader.tsx:52-59` |
| 主区 | AppShell main, 全高模式 | `AppShell.tsx:57-73`; `frontend/src/app/(app)/layout.tsx:45-47` |
| 页内左栏 | PrototypePage `aside` | `page.tsx:144-183` |
| 页内中区 | PrototypePage `main` | `page.tsx:186-244` |
| 页内右侧预览 | PrototypePage `aside` | `page.tsx:247-326` |
| 页内底栏 | PrototypePage `footer` | `page.tsx:330-398` |
| 右侧全局聊天栏 | RightChatRail | `AppShell.tsx:77`; `frontend/src/components/chat/RightChatRail.tsx:29` |

组件树:

- `AppLayout` (`(app)/layout.tsx:43`)
  - `AppShell` (`AppShell.tsx:31`)
    - `AppHeader` (`AppHeader.tsx:45`)
    - `main` > `PrototypePage` (`page.tsx:95`)
      - 左栏 `aside` (`page.tsx:144`): 类型按钮组 (`:146-165`), 引用上下文 (`:167-182`)
      - 中区 `main` (`page.tsx:186`): 提示词 textarea (`:188-194`), 固定说明文本 (`:195-198`), 日志 `pre` (`:199-203`), 生成与从模板按钮 (`:204-219`), 流水线条与页脚操作 (`:222-242`)
      - 右侧 `aside` (`page.tsx:247`): UI、2D、3D、4D、5D、1D、动效预览卡 (`:248-325`)
      - 底栏 `footer` (`page.tsx:330`): 缩放与图层控件 (`:331-341`), 设备 (`:345-355`), 数据来源与通道状态 (`:357-376`), 版本 (`:380-385`), 两个跳转链接与导出按钮 (`:389-397`)
    - `RightChatRail` (`AppShell.tsx:77`; `frontend/src/components/chat/RightChatRail.tsx:29`)
  - 页面内辅助: `callGemArc` (`page.tsx:66-80`)、`GEN_TYPES` (`:82-91`)、`GEMARC_PIPELINE` (`:57-64`)

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 (流水线成功) | 点击生成且六步全部成功 | 日志逐行为 `[建模]` 至 `[导出]`, 每行输出截断为 300 字符 (`page.tsx:130`) | 🟡 日志可见; 右侧预览不随结果变化 (`page.tsx:247-326`) |
| 加载 (通道检测) | 挂载 (初始状态 `checking`, `page.tsx:100`) | 文案 "… GemArc 检测中" (`page.tsx:374`); 生成按钮禁用 (`:209`) | 🟡 仅挂载时可见; 切换数据来源时重新请求但不回到检测中, 旧状态保留至响应返回 (`page.tsx:105-117`) |
| 加载 (流水线运行) | `runningStep` 非空 | 按钮图标旋转 (`:213`), 按钮禁用 (`:209`); 阶段高亮不随步骤变化 (`:225`) | 🟡 |
| 空 (无日志) | 初始状态或清空后 | 日志区不渲染 (`page.tsx:199`) | ✅ |
| 空 (无提示词) | `prompt` 为空 | 生成按钮不校验提示词 (`page.tsx:209` 无 prompt 条件); 以空 `query` 调用工具 | ⬜ |
| 错误 (步骤失败) | 任一工具 HTTP 非 2xx 或返回 `error` | 日志追加 "[步骤] 失败: ..." 并停止后续步骤 (`page.tsx:131-135`) | 🟡 失败可见; 无步骤高亮与重试 |
| 错误 (通道离线) | 两个通道健康检查均非 ok | 文案 "GemArc MCP 离线", 前缀为圆点符号 (`page.tsx:374`); 生成按钮禁用 | 🟡 门控依据与所选来源不一致 (`:111-114`) |
| 错误 (页面级) | 组件渲染异常 | 无错误边界: `frontend/src/app` 与 `(app)/` 下无 `error.tsx` 或 `global-error.tsx` | ⬜ |
| 未授权 | 访问者无登录会话 | 无校验: 页面、根布局 (`frontend/src/app/layout.tsx`)、`frontend/src/app/providers.tsx` 与 `OnboardingGuard.tsx` 均无登录校验; 页面与代理对请求开放 (`route.ts` 全文; `proxy.ts:39`) | ⬜ |
| 未接入 | 选择 "当前 Branch 的接口" 或图片与视频类型 | 无未接入态文案; 数据来源按钮仅显示 "当前 Branch 的接口" (`page.tsx:371`); 项目固定 `prototype-demo` (`:127`) | ⬜ |
| 禁用 | 流水线运行中或通道非 up | 生成按钮 `disabled` 与 `disabled:opacity-50` (`page.tsx:209, 211`); 其余按钮无禁用态 (`:216, 232, 236, 239, 395`) | 🟡 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 选择生成类型 | 无 | 按钮高亮 (`page.tsx:151, 155`) | 本地状态 `genType` (`:96`) | 🟡 不进入流水线参数 (`:125-129`), 不改变预览 (`:247-326`) |
| 编辑提示词 | 无 | 更新 `prompt` (`page.tsx:192`) | 本地状态 | ✅ 作为 `query` 参与流水线 (`:127`) |
| 生成 | `gemarcStatus === "up"` 且未运行 (`:209`) | 顺序调用六个工具并写入日志 (`:120-138`) | `POST /api/gemarc` (`:67`) | ✅ |
| 从模板 | 无 | 无 (`:216`) | 无 | ⬜ |
| 添加上下文 | 无 | 无 (`:179`) | 无 | ⬜ |
| 切换设备 | 无 | 预览卡宽度 (`:249`) 与流水线 note (`:127`) 变化 | 本地状态 `device` (`:97`) | ✅ |
| 切换数据来源 | 无 | 通道切换并重新检测 (`:117`) | 本地状态 `dataSource` (`:98`); `GET /api/gemarc` | 🟡 门控不随通道 (`:111-114`) |
| 交互预览 | 无 | 无 (`:232`) | 无 | ⬜ |
| 导出 (流水线条与页脚两处) | 无 | 无 (`:236, 395-397`) | 无; 流水线中 `gemarc_render` 仅写入日志 (`:63, 130`) | ⬜ |
| 分享 | 无 | 无 (`:239`) | 无 | ⬜ |
| 附加到任务卡 | 无 | 跳转至 `/kanban?view=%E6%8C%89%E6%96%87%E4%BB%B6%E5%BC%80%E5%8D%A1` (`:389`) | 无数据传递 | 🟡 |
| 放进画布白板 | 无 | 跳转至 `/canvas/canvas-001` (`:392`) | 无数据传递 | 🟡 |
| 缩放、适配、图层、评论、主色 | 无 | 无 (`:331-341`) | 无 | ⬜ |

数据:

- 来源: 组件内 React 状态 (`page.tsx:96-102`); 无 store, 无持久化。
- REST: `GET /api/gemarc` 返回 `local` 与 `remote` 对象; 在线时含 `status` 与 `url`, 离线时仅 `{status: "down"}` (`route.ts:91-94`); 页面读取 `status` (`page.tsx:109-110`)。`POST /api/gemarc` 返回 JSON-RPC, 页面读取 `result.content[].text` 与 `error.message` (`page.tsx:73-78`)。
- 硬编码: 固定说明文本 (`page.tsx:196`)、引用上下文 (`:172, 176`)、预览卡内容 (`:247-325`)、版本列表 (`:382-384`); 页面其余文字亦为字面量。

与文档设计的差距:

- 布局: 设计参考中提示词与回复位于左栏 (`Prototype.dc.html:60-63`), 代码位于中区 (`page.tsx:188-198`)。设计参考的设备与数据来源选择位于右侧属性面板 (`Prototype.dc.html:177-178`), 代码位于底栏 (`page.tsx:345-376`)。FR-14-01、FR-14-09、FR-14-10 相关。
- 画布: 设计参考中央为多画面画布 (`Prototype.dc.html:81-166`), UI、2D、3D、4D、5D、1D、动效七个画面框并排布置, 以阴影高亮当前类型 (`:204`); 媒体类型无画面框; 代码无画布, 七张预览卡同时堆叠 (`page.tsx:247-325`)。FR-14-01、FR-14-06 未完全满足; FR-14-07 未满足。
- 4D 与 5D 交互: 设计参考提供时间播放与停止 (`Prototype.dc.html:197-198, 215, 129`) 与 5D 屏蔽开关 (`:136, 225-226`); 代码无对应控件。FR-14-17 未满足。
- UI 标签: 设计参考的出边、入边、环路径切换 (`Prototype.dc.html:218-219`) 在代码中为静态三组 (`page.tsx:255-266`)。FR-14-06 未完全满足, FR-14-17 未满足。
- 3D: 设计参考为机柜可选的空间图 (`Prototype.dc.html:110-119, 220-222`); 代码为标题加一行静态文字 (`page.tsx:283-284`), Physis 未接入。FR-14-06 未完全满足, FR-14-17 未满足。
- 版本: 设计参考有版本列表 (`Prototype.dc.html:179-184, 229`); 代码为硬编码 (`page.tsx:380-385`)。FR-14-19 未满足。
- 图片与视频: 设计参考标注 "当前显示「未接入」" (`Prototype.dc.html:203`); 代码无该态。FR-14-20 未满足; FR-14-08 未完全满足。
- 页脚控件: 设计参考的缩放按钮与 "适配" (`Prototype.dc.html:160-165`) 在代码中为静态文本 (`page.tsx:331-336`)。FR-14-22 未满足。
- 导出与分享: 设计参考的导出与分享位于流水线条 (`Prototype.dc.html:75-77`); 代码两处均无行为 (`page.tsx:236, 239, 395-397`)。FR-14-13、FR-14-14 未满足。
- 文案: 页面文案未走 i18n (`page.tsx:16-42`)。FR-14-27 未满足。
- 窄屏: 左栏与右栏为固定宽度 `w-64` 与 `w-80` (`page.tsx:144, 247`), 文件中无断点类; 根布局在 768px 以下改用移动端顶栏与底部导航 (`frontend/src/app/layout.tsx:89-99`)。关联 FR-14-09 (设备切换); 本页窄屏可用性未验证 (§7)。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 原型页面 | `frontend/src/app/(app)/prototype/page.tsx` | 🟡 | 401 行; 流水线与通道逻辑 `page.tsx:95-138`; 预览与导出为静态 `page.tsx:247-326, 330-398` |
| GemArc 代理 | `frontend/src/app/api/gemarc/route.ts` | ✅ | 95 行; GET 健康检查 `route.ts:82-95`; POST 白名单代理 `route.ts:40-80` |
| 导航注册 | `frontend/src/lib/nav/registry.ts:604-611` | ✅ | 页签 id `prototype`, href `/prototype`; 描述 `registry.ts:610` 仍为 "设计原型与实验区画廊 (wireframe / 游戏 / IPA 看板原型)" |
| 顶栏默认页签 | `frontend/src/lib/nav/navStore.ts:94` | ✅ | `DEFAULT_HEADER_TABS` 含 `prototype` |
| 页签渲染 | `frontend/src/components/AppHeader.tsx:52-59, 220` | ✅ | 按 `headerTabIds` 渲染 `tab-<id>` 链接 |
| 执行组 | `frontend/src/lib/nav/registry.ts:771-775` | ✅ | 组成员含 `prototype` (774 行) |
| 全高布局 | `frontend/src/app/(app)/layout.tsx:28-38` | ✅ | `/prototype` 在 33 行 |
| 页签标题与副标题 | `zh-CN.ts:680-683`; `en.ts:665-668`; `ja.ts:666-669` | 🟡 | 标题正确; 副标题仍为画廊语义: `zh-CN.ts:682` 为 "设计原型与实验区画廊", `en.ts:667` 为 "Design prototype and experiment gallery", `ja.ts:668` 为 "設計プロトタイプと実験ギャラリー" |
| 页面文案 i18n | `frontend/src/app/(app)/prototype/page.tsx` | ⬜ | 无 i18n 调用 (`page.tsx:16-42`) |
| 代理鉴权 | `frontend/src/app/api/gemarc/route.ts` | ⬜ | 无鉴权逻辑 (全文) |
| 原型持久化 (store) | 无 | ⬜ | 637ed19d 中无对应模块 |
| GemArc 服务实现 | 仓外 `E:\GemArc` | ❓ | 不在本仓内; 运行状态与响应结构无法从仓内核对 |
| 页签存在性测试 | `frontend/src/components/__tests__/AppHeader.test.tsx:62-71` | 🟡 | 只验证 `tab-prototype` 渲染 (`:69`) |
| 页面与代理测试 | 无 | ⬜ | `frontend/src/app/(app)/prototype/` 与 `frontend/src/app/api/gemarc/` 仅含实现文件 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 固定说明文本声称已生成 3 个页签的可点击原型 (`page.tsx:196`), 不论流水线是否执行 | 用户会把未执行的结果当作真实输出; 违背未接通数据须显式标注的要求 (`AGENTS.md:22`; FR-14-08) | 删除固定句; 仅在流水线成功后按真实输出渲染摘要 |
| 右侧预览为固定内容, 5D 数值为示例 (`page.tsx:247-325`, `:304-305`) | 示例数字可被误读为推演结果 (FR-14-06, FR-14-07) | 预览改为由生成结果驱动; 接入前在示例内容上加可见的 "示例" 标识 (FR-14-08) |
| 生成类型未进入流水线与预览 (`page.tsx:125-129`, `:247`) | FR-14-01 无法成立 | 将 `genType` 加入流水线参数; 预览按类型分支渲染 |
| 流水线阶段高亮静态 (`page.tsx:225`) | 进度与失败步骤不可见 (FR-14-04) | 绑定 `runningStep` 与每步结果状态 |
| 每步输出截断 300 字符 (`page.tsx:130`) | 错误细节缺失 (FR-14-05) | 保存完整输出, 提供展开查看 |
| 门控使用任一通道在线 (`page.tsx:111-114`) | 所选通道离线时仍可点击生成, 运行必然失败 (FR-14-12) | 门控仅检查当前数据来源对应的通道 |
| `/api/gemarc` 无鉴权 (`route.ts` 全文; `proxy.ts:39`) | 任何可访问应用的调用方都可调用 12 个工具 (FR-14-24) | 共享部署前加入会话校验 (可复用 `frontend/src/lib/auth.ts` 的 `getSession`); 本地开发保持回环绑定 |
| 示例模式默认外发 (`page.tsx:98, 128`; `route.ts:21`) | 用户描述被发送到第三方远端 (§4.5) | 默认改为本地通道; 远端通道需显式确认; 是否允许外发需产品确认 |
| 上游 POST 无超时与异常处理 (`route.ts:63-71`) | 上游挂起会阻塞请求; 异常无结构化返回 (FR-14-25) | 增加 `AbortSignal.timeout` 与结构化错误响应 |
| "当前 Branch 的接口" 实际路由到本地 GemArc, 项目固定 `prototype-demo` (`page.tsx:127, 371`) | 标签与行为不符; 原型未绑定 Branch (FR-14-10) | 按当前 Branch 与 Project 传入上下文; 未绑定时显示未接入态 (FR-14-08) |
| 导出、分享、交互预览、从模板、添加上下文、版本记录无行为 (`page.tsx:179, 216, 232, 236, 239, 380-385, 395-397`) | FR-14-02、FR-14-13、FR-14-14、FR-14-17、FR-14-18、FR-14-19 未实现; `SRS-PROTOTYPE-001.md` 将导出与分享 (FR-PT-005) 标为 ✅, 与代码不符 | 先实现导出 (下载 `gemarc_render` 输出), 再接分享与交互预览; 未完成前按钮禁用并标注 |
| 附加任务卡与放进画布仅为跳转 (`page.tsx:389, 392`) | 原型内容不随跳转传递 (FR-14-15, FR-14-16) | 定义引用数据模型, 接入任务卡与画布的写入接口 |
| 3D 预览为文字, Physis 未接入 (`page.tsx:283-284`; `INTEGRATION-MAP-GITHUB-REPOS-001.md:29`) | FR-14-06 的 3D 预览不可用 | 作为后续集成项, 接入前保持文字标注为示例 |
| 页面文案硬编码中文 (`page.tsx:145, 196, 248` 等) | 三语切换不生效 (FR-14-27) | 文案迁入 i18n 字典 |
| 导航描述与副标题仍为画廊语义 (`registry.ts:610`; `zh-CN.ts:682`; `en.ts:667`; `ja.ts:668`) | 导航语义与功能不符 | 改为与本功能一致的描述 |
| 三栏固定宽度, 无断点样式 (`page.tsx:144, 247`) | 窄屏可用性未验证 | 定义窄屏布局并补充断点样式 |
| 无自动化测试 (§7) | 回归无保障 | 为代理白名单、通道门控、流水线顺序与失败停止补单元测试; 补页面级 e2e |
| `SRS-PROTOTYPE-001.md:13` 数据源写为 store seed | 与代码 (GemArc 通道) 不符 | 以 GemArc 通道为准更新 SRS 数据源行, 或新增 store 层并同步代码 |

## 7. 验收与测试

现有测试:

| 测试 | 位置 | 验证内容 |
|---|---|---|
| 顶栏页签存在 | `frontend/src/components/__tests__/AppHeader.test.tsx:62-71` | 渲染 `tab-prototype` 元素 (`:69`) |
| 侧栏页签配置 fixture | `frontend/src/components/__tests__/Sidebar.test.tsx:39-49` | `headerTabIds` 初始化包含 `prototype` (`:43`); 仅为 store 初始化, 不断言页面行为 |

测试断言: 顶栏存在 `prototype` 页签 (`AppHeader.test.tsx:69`); 测试运行结果在仓内无记录 ❓。

未验证:

- `/api/gemarc` 的白名单、方法拦截与上游转发: 无测试文件。
- 页面的流水线顺序、失败停止、通道门控与设备切换: 无单元测试; `frontend/e2e` 中无原型相关规格 (637ed19d 中检索无结果)。
- GemArc 服务的在线状态与 `/healthz` 响应结构: 依赖仓外服务, ❓。
- 导出、分享、交互预览: 未实现。
- `SRS-PROTOTYPE-001.md:10` 所述 "已实现并实测": 637ed19d 中未找到对应截图或测试产物, ❓。`REVIEW-2026-11-14-IMPLEMENTATION.md:24` 记录本地 GemArc 服务状态在仓外, 无法从仓内验证。
- `GEMARC_REMOTE_TOKEN` 是否为远端通道的必需项: ❓ (仓内未确认)。
- 远端 `gem-arc.vercel.app` 是否允许外发用户描述: 需产品确认 (§4.5)。

## 8. 来源与裁决

SRS-PROTOTYPE-001 的 FR 与本文件需求的对应: FR-PT-001 → FR-14-01; FR-PT-002 → FR-14-02; FR-PT-003 → FR-14-03、FR-14-04、FR-14-05; FR-PT-004 → FR-14-06、FR-14-07、FR-14-17; FR-PT-005 → FR-14-13、FR-14-14。其余需求的来源为设计参考与代码, 在证据列注明。

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-PROTOTYPE-001.md` (需求文档) | 设计要点 §1; FR-PT-001 至 005 (`:27-31`); 文件映射 §3; 数据源 (`:13`) | FR 全部标为 ✅; 代码中生成类型、上下文、预览、导出与分享多为静态或无行为; 数据源写为 store, 代码中无 store 实现, 实际为 GemArc 通道 | 状态与数据源以代码为准; FR 映射见上 |
| `docs/prototypes/2d · 原型-html/README.md` (参考说明) | 参考原型的使用说明; 明确其为参考而非生产代码 | 无 | 仅作参考 |
| `docs/prototypes/2d · 原型-html/Prototype.dc.html` | 布局: 左生成 (`:44-66`)、中画布 (`:68-167`)、右属性 (`:169-190`); 类型切换高亮对应画面框 (`:204, 210-211, 216-217`); UI 标签 (`:218`); 3D 机柜 (`:205, 220-222`); 4D 播放 (`:197-198, 215, 129`); 5D 屏蔽 (`:136, 225-226`); 版本 (`:181-183, 229`); 未接入标注 (`:202-203`); 缩放 (`:160-165`) | 代码为三栏 (`page.tsx:144, 186, 247`); 无画布, 无与设计参考对应的属性页签 (调整、图层、评论); 交互未实现 | 布局以代码为准; 交互保留为 ⬜ 设计意图 |
| `docs/prototypes/INDEX.md:19` | 映射: 2d 原型 → `/prototype` → SRS-PROTOTYPE-001 → `page.tsx` | 与代码路径一致 | 采用 |
| `docs/INDEX.md:38` | 将 `prototypes/` 归类为 archive | 与 `prototypes/INDEX.md` 的参考设计定位不一致 | 以 `prototypes/INDEX.md` 为准 |
| `docs/frontend/design/ui-3pane-arch.md:17` | 顶栏固定页签包含原型 `/prototype` | 与 `navStore.ts:94` 一致 | 采用 |
| `docs/architecture/2026-11-14-github-integration/INTEGRATION-MAP-GITHUB-REPOS-001.md:17, 29, 35-36` | GemArc 经 `/api/gemarc` 接入 `/prototype`; Physis 为 3D 物理原型 | 流水线工具名在 `:17` 为 init→validate→simulate→build, 在 `:36` 为 brief/inspect/validate/build/verify; 代码为 brief/validate/simulate/build/verify/render (`page.tsx:57-64`); Physis 未接入 | 工具名以代码为准; Physis 归为范围外 |
| `docs/architecture/2026-11-14-backend-support/WBS-BACKEND-SUPPORT-001.md:13` | `/api/gemarc` 路由处理器已通 | 与代码一致; 不证明 GemArc 服务在线 | 路由 ✅; 服务状态 ❓ |
| `docs/architecture/2026-11-15-lane-workspace/REVIEW-2026-11-14-IMPLEMENTATION.md:24` | 记录 GemArc 集成 (提交 23d60e75 引入 `/api/gemarc` 与页面) | 本地服务状态在仓外, 端到端未在仓内验证 | 路由与页面 ✅; 端到端 ❓ |
| `docs/reports/PHASE-DELIVERY-SYNC-20261118.md:133` | F14 状态摘要: GemArc 流水线已接; 预览、导出与版本记录静态 | 与代码一致 | 采用 |
| `AGENTS.md:22` | 未接通授权数据的功能须显示未接入状态, 不得以示例数据冒充 | 页面固定文案与示例数字与之不符 (`page.tsx:196, 304-305`) | 作为 FR-14-08、FR-14-10 的依据 |
| `docs/features/F02-branch-workspace.md:199` | 外部链接目标包含 `/prototype`, 路由目录存在 | 与代码一致 | 仅作路由存在性交叉引用 |
| `docs/features/F14-prototype.md` (同路径, 637ed19d 版本) | 需求编号与证据条目的格式; 差距清单的条目方向 | 代码基线为 cdd00a78, 不是 637ed19d | 不作为状态依据; 状态与行号以 637ed19d 代码为准 |
| `frontend/src/app/(app)/prototype/page.tsx`; `frontend/src/app/api/gemarc/route.ts` | 页面与代理实现 | 无 | 实现权威 |
| `frontend/src/proxy.ts` | 移动 UA 重定向 (`:22-34`); matcher 排除 `api` (`:39`) | 无 | 用于 §4.5 与 §4.6 结论 |
| `frontend/src/lib/nav/registry.ts`; `navStore.ts`; `frontend/src/app/(app)/layout.tsx`; `frontend/src/app/layout.tsx`; `frontend/src/components/AppShell.tsx`; `AppHeader.tsx`; `Sidebar.tsx`; `frontend/src/components/chat/RightChatRail.tsx` | 导航、默认页签、全高布局、外壳与侧栏 | 注册表描述 (`registry.ts:610`) 与功能不符 | 描述列为 §6 差距 |
| `frontend/src/lib/i18n/zh-CN.ts`; `en.ts`; `ja.ts`; `dictionary.ts` | 页签标题与副标题; 类型定义 | 副标题描述为画廊 | 列为 §6 差距 |
| `frontend/src/components/__tests__/AppHeader.test.tsx`; `Sidebar.test.tsx` | 页签存在性测试; 侧栏 store fixture | fixture 不断言页面行为 | 仅作 §7 现有测试 |
| 提交 `23d60e75` (GemArc 代理与 `/prototype` 流水线, 新增 `route.ts`, 改动 `page.tsx`); `c0f149f2` (新增 `docs/prototypes/2d · 原型-html/` 设计参考文件); `2719b66d` (改动 `page.tsx`, 说明中列出交互预览); `7471e77d` (新建 `/prototype` 页, 改动 `page.tsx`) | 四个提交均为 637ed19d 的祖先提交 | `2719b66d` 说明中的交互预览在代码中无处理函数 (`page.tsx:232`), 以代码为准 | 作为来源引用, 不作为状态依据 |
| `docs/briefs/WBS-W14-UI1-FLOW-PROTOTYPE-20261026.md`; `docs/reports/PHASE-WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027.md` | 桌面导航流程原型 (brief 指定的产物 `docs/prototypes/w14-ui1-flow-20261026/index.html` 在 637ed19d 中不存在) | "原型" 指导航流程验证, 与 `/prototype` 生成器无关 | 不采用 |
| `docs/requirements.md:1547, 2470` (PLG-004) | 插件导航入口的服务端 projection | 与原型页无关 | 不采用 |
| `docs/basic-design.md:4303` | 项目选择器与 ProjectWorktreeIndex 的授权 projection | 与本功能无关 | 不采用 |
| `docs/frontend/design/ui-detailed-design.md:150-159` | 其他页面的 PDF 原型图索引 | 与本功能无关 | 不采用 |
| `frontend/src/app/(app)/settings/page.tsx:256`; `frontend/src/app/canvas/components/GitCanvas.tsx:11` | 引用其他 mockup (设置、统一画布) | 与本功能无关 | 不采用 |
| `docs/reports/WBS-DISPATCH-DEV-STATUS-20261023.jsonl`; `docs/reviews/W16-KMS-SOURCE-AUDIT-20261028.md`; `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md`; `docs/briefs/DDD-AUDIT-04.md` | 命中 gemarc 字样 | 命中词为未跟踪目录名 `docs/gemarc-5d-atlas/` 与其他仓库的分支名 (如 `wt-gemarc-5d-atlas`), 与本功能无关 | 不采用 |
