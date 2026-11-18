# F14 原型

> 状态摘要: 🟡 1D–5D 原型入口、GemArc 六步流水线的顺序调用、设备与数据来源切换以及 GemArc 健康检查已接入代码; 生成类型选择、引用上下文、右侧 1D–5D 预览、导出/分享、交互预览与版本记录仍是静态展示或无行为控件。
> 代码基线: HEAD cdd00a78; 核对方法: path/symbol grep plus file reads; 核对日期 2026-11-16.

## 1. 目标与边界

### 1.1 目标

在 Branch 工作区顶栏页签 `/prototype` 中, 用户输入原型描述、选择生成类型与设备, 由 GemArc MCP 依次执行 建模 → 校验 → 推演 → 构建 → 验证 → 导出 六个工具, 并在页面查看流水线输出与 1D–5D 预览。

### 1.2 范围

| 范围 | 内容 |
|---|---|
| 范围内 | `/prototype` 页面的生成入口、流水线调用与日志展示; `/api/gemarc` 代理 (GET 健康检查, POST JSON-RPC 白名单转发); 设备与数据来源切换; 顶栏与导航注册表挂接 |
| 范围外 | GemArc MCP 服务本体 (位于仓外 `E:\GemArc`, 本仓不含实现); Physis 3D 物理引擎 (仅为后续集成项); 任务卡与画布的业务逻辑 (本页只提供跳转) |

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 产品 / 设计者 | 用自然语言描述原型, 选择生成类型, 查看 1D–5D 预览与流水线输出 |
| 开发者 | 运行六步 GemArc 流水线, 检查每步输出与失败位置 |
| 运维 / 管理者 | 确认本地 GemArc 与远端 GemArc 通道是否在线, 决定使用示例数据还是当前 Branch 接口 |

主要用户故事:

- US-1 输入描述, 点击生成, 依次看到六步流水线日志。
- US-2 切换手机 / 桌面设备, 查看预览宽度变化。
- US-3 在示例数据与当前 Branch 接口之间切换 GemArc 通道。
- US-4 将原型附加到任务卡或放进画布白板。
- US-5 导出或分享原型 (未实现)。

## 3. 需求

优先级为本文档的设计分级: P0 核心闭环, P1 闭环补充, P2 增强。编号与 SRS-PROTOTYPE-001 的 FR-PT 对应关系见 §8。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-14-01 | 生成类型选择 (UI / 2D / 3D / 4D / 5D / 1D / 动效 / 图片与视频), 选择结果驱动生成与预览 | P0 | 🟡 | 类型按钮与高亮存在 `frontend/src/app/(app)/prototype/page.tsx:82-91, 96, 151, 155, 158`; `genType` 未出现在流水线参数 (`page.tsx:127`) 中, 也不改变右侧预览 (`page.tsx:248`) |
| FR-14-02 | 引用上下文 (文件 / 截图 / 任务卡) 可添加并随生成提交 | P1 | 🟡 | 两个上下文项为硬编码文本 `page.tsx:170-177`; "+ 文件 / 截图 / 任务卡" 按钮无 onClick `page.tsx:179`; 上下文未进入 `callGemArc` 参数 `page.tsx:125-128` |
| FR-14-03 | 生成流水线按 建模/校验/推演/构建/验证/导出 映射到 GemArc 工具并顺序执行, 首个失败即停止 | P0 | ✅ | 步骤表 `page.tsx:57-64`; 顺序执行与失败停止 `page.tsx:120-138` (失败分支 `page.tsx:131-135`); 白名单代理 `frontend/src/app/api/gemarc/route.ts:25-38, 50-52` |
| FR-14-04 | 流水线阶段进度随当前执行步骤变化 | P1 | 🟡 | 阶段标签在静态条件 `i <= 3` 下高亮 `page.tsx:222-230`; `runningStep` 仅用于按钮禁用与旋转图标 `page.tsx:208-213` |
| FR-14-05 | 每步 GemArc 输出完整展示 | P1 | 🟡 | 每步输出截断为 300 字符 `page.tsx:130`; 日志区 `page.tsx:199-203` |
| FR-14-06 | 右侧 1D–5D 预览由生成结果驱动 | P0 | 🟡 | 预览内容为固定文本与示例数字 `page.tsx:247-325` (5D 固定值 `page.tsx:304-305`; 3D 仅文字 `page.tsx:283-284`); 仅设备切换改变 UI 卡片宽度 `page.tsx:249` |
| FR-14-07 | 设备切换 (手机 / 桌面) | P1 | ✅ | 状态 `page.tsx:97`; 切换按钮 `page.tsx:347-352`; 宽度分支 `page.tsx:249`; 设备写入流水线 note `page.tsx:127` |
| FR-14-08 | 数据来源切换: 示例数据走远端 GemArc, 当前 Branch 接口走本地 GemArc | P0 | 🟡 | 切换按钮 `page.tsx:358-372`; 通道映射 `page.tsx:128`; 代理按 `_meta.source` 选择上游 `route.ts:59-61`; "当前 Branch 的接口" 实际只路由到本地 GemArc, 未绑定 Branch 或 Project (项目固定为 `prototype-demo`, `page.tsx:127`) |
| FR-14-09 | GemArc 本地与远端健康检查并显示在线状态 | P0 | ✅ | 客户端探测 `page.tsx:105-117`; 状态文案 `page.tsx:373-375`; 服务端并行探测 `route.ts:82-95` |
| FR-14-10 | 生成门控: 通道不在线时禁止生成 | P0 | 🟡 | 按钮禁用条件 `page.tsx:209`; 门控依据的状态逻辑按任一通道在线即置为在线 `page.tsx:111-114`, 与所选数据来源的通道可能不一致 |
| FR-14-11 | 导出原型 (文件 / HTML / 视频) | P1 | ⬜ | 导出按钮无 onClick `page.tsx:236`; 页脚 "导出 HTML / 视频" 无 onClick `page.tsx:395-397`; 流水线中的 `gemarc_render` 只把文本输出写入日志 `page.tsx:63, 130` |
| FR-14-12 | 分享原型 | P2 | ⬜ | 分享按钮无 onClick `page.tsx:239` |
| FR-14-13 | 附加到任务卡 | P1 | 🟡 | 仅为跳转链接, 目标为固定 URL `page.tsx:389` (`/kanban?view=按文件开卡`); 原型内容不随跳转传递 |
| FR-14-14 | 放进画布白板 | P1 | 🟡 | 仅为跳转链接, 目标固定为 `/canvas/canvas-001` `page.tsx:392`; 原型内容不随跳转传递 |
| FR-14-15 | 交互预览 (可点击原型, 播放 4D 时间线, 切换 5D 反事实) | P1 | ⬜ | "交互预览" 按钮无 onClick `page.tsx:232`; 对应交互仅存在于设计参考 `docs/prototypes/2d · 原型-html/Prototype.dc.html:198, 215, 218, 220, 225` |
| FR-14-16 | 从模板新建原型 | P2 | ⬜ | 按钮无 onClick `page.tsx:216` |
| FR-14-17 | 版本记录 (生成版本与说明) | P2 | ⬜ | 版本文本为硬编码 `page.tsx:381-385`; 无存储模型 |
| FR-14-18 | 示例数据与真实结果必须区分标注 (不得把示例当作生成结果) | P0 | ⬜ | 页面固定声明 "已生成 3 个页签的可点击原型" `page.tsx:196`, 不论流水线是否执行; 5D 数值为固定值 `page.tsx:304-305`; 依据 AGENTS.md §0 (未接通数据须显示未接入状态) |
| FR-14-19 | 图片与视频生成 (未接入时显示未接入态) | P2 | ⬜ | 类型按钮存在 `page.tsx:90`; 无生成逻辑; 设计参考写明 "当前显示「未接入」" `Prototype.dc.html:203` |
| FR-14-20 | 动效预览 | P2 | ⬜ | 右侧动效卡为静态 `page.tsx:320-323`; 无过渡播放逻辑 |

## 4. 设计

### 4.1 概念模型

| 概念 | 说明 | 实现位置 |
|---|---|---|
| 生成类型 (GenKind) | 8 个取值: ui, 2d, 3d, 4d, 5d, 1d, motion, media | `page.tsx:82-91` |
| 流水线步骤 (GemArcStep) | 6 个步骤, 每步绑定一个 GemArc MCP 工具 | `page.tsx:51-64` |
| 数据通道 (DataSource) | `sample` 对应远端 GemArc, `branch` 对应本地 GemArc | `page.tsx:98, 128`; `route.ts:20-21, 59-61` |
| 设备 (Device) | `mobile` 或 `desktop`, 影响预览宽度与流水线 note | `page.tsx:97, 127, 249` |
| 预览 (Preview) | 右侧 7 块固定展示卡 (UI、2D、3D、4D、5D、1D、动效) | `page.tsx:247-325` |

仓库内不存在原型领域模型或持久化实体。SRS 声明的数据源为 "store (seed 预览模式)" (`docs/requirements/SRS-PROTOTYPE-001.md:13`), 代码中未找到对应 store 模块 (原型的流水线与代理逻辑仅位于 `page.tsx` 与 `route.ts`)。

### 4.2 数据模型

全部状态保存在页面内存中, 刷新即丢失。

| 实体 / 结构 | 字段 | 来源 | 状态 |
|---|---|---|---|
| GemArcStep | id, label, tool | `page.tsx:51-55` | ✅ |
| 页面状态 | genType, device, dataSource, prompt, gemarcStatus, runningStep, pipelineLog | `page.tsx:96-102` | ✅ (内存态) |
| JSON-RPC 请求体 | jsonrpc, id, method, params.name, params.arguments, params._meta.source | `page.tsx:66-70`; `route.ts:49-61` | ✅ |
| 健康检查响应 | local / remote 各自的 status 与 url | `route.ts:91-94`; 读取 `page.tsx:106-110` | 🟡 (GemArc `/healthz` 返回结构未在仓内验证, 见 §7) |
| 原型实体 (持久化) | 原型 id、版本、上下文引用 | 无 | ⬜ |
| 引用上下文 | 文件 / 截图 / 任务卡引用 | 仅硬编码展示 `page.tsx:170-177` | ⬜ |
| 版本记录 | when, text | 硬编码 `page.tsx:381-385` | ⬜ |

### 4.3 接口

| 接口 | 文件与行 | 说明 | 状态 |
|---|---|---|---|
| `GET /api/gemarc` | `route.ts:82-95` | 并行请求本地 `/healthz` (超时 4s) 与远端 `/healthz` (超时 8s); 失败时返回 `{status: "down"}` | ✅ |
| `POST /api/gemarc` | `route.ts:40-80` | JSON-RPC 代理; 仅放行 `tools/call` (工具在白名单内) 与 `initialize` / `tools/list`; 其他方法返回 403 | ✅ |
| 工具白名单 | `route.ts:25-38` | 12 个工具; 页面实际调用其中 6 个 (`gemarc_brief`, `gemarc_validate`, `gemarc_simulate`, `gemarc_build`, `gemarc_verify`, `gemarc_render`) | ✅ |
| 本地上游 | `route.ts:20` | 默认 `http://127.0.0.1:8760/mcp`, 可由 `GEMARC_MCP_URL` 覆盖 | 🟡 (配置存在; 服务是否运行在仓外, 见 §7) |
| 远端上游 | `route.ts:21-22, 68` | 默认 `https://gem-arc.vercel.app/mcp`, 可由 `GEMARC_REMOTE_URL` 覆盖; 若设置 `GEMARC_REMOTE_TOKEN` 则以 Bearer 方式发送 | 🟡 (是否需要 token 未在仓内确认) |
| 客户端调用 | `page.tsx:66-80` | `callGemArc(tool, args, source)`; 响应取 `result.content[].text` 拼接 | ✅ |
| 页签跳转 | `page.tsx:389, 392` | 两个 `<a>` 链接, 无数据传递 | 🟡 |

流水线的工具名以页面为准 (`page.tsx:57-64`)。`docs/architecture/2026-11-14-github-integration/INTEGRATION-MAP-GITHUB-REPOS-001.md` §2 使用 "init → validate → simulate → build" 的描述, 与代码不同, 以代码为准 (见 §8)。

### 4.4 状态机与关键流程

GemArc 通道状态: `checking` (初始, `page.tsx:100`) → `up` 或 `down` (`page.tsx:105-117`)。门控规则当前为: 所选通道在线 **或** 另一通道在线即为 `up` (`page.tsx:111-114`)。

生成流程 (`runPipeline`, `page.tsx:120-138`):

1. 清空日志, 按 `GEMARC_PIPELINE` 顺序循环。
2. 每步设置 `runningStep`, 调用 `callGemArc`, 把输出前 300 字符追加到日志。
3. 任一步抛错则追加失败信息, 清空 `runningStep` 并返回 (后续步骤不执行)。
4. 全部成功后清空 `runningStep`。

设计参考中的交互状态 (类型切换驱动中央画面、4D 播放与停止、5D 屏蔽开关、3D 机柜选择, `Prototype.dc.html:197-228`) 在代码中均不存在, 归为 ⬜ (见 FR-14-15)。

### 4.5 权限与安全

| 项 | 现状 | 证据 | 状态 |
|---|---|---|---|
| 工具白名单 | 只允许白名单工具与 3 个方法 | `route.ts:25-38, 50-56` | ✅ |
| 远端 token | 只在服务端读取, 只附加到远端请求 | `route.ts:22, 68` | ✅ |
| 身份认证 / 授权 | `/api/gemarc` 无认证校验; `proxy.ts` 的 matcher 排除 `api` 路径 | `frontend/src/app/api/gemarc/route.ts` (全文); `frontend/src/proxy.ts:39` | ⬜ |
| 默认外发数据 | 默认数据来源为示例, 会把用户描述 (`query`) 与设备 note 发送到第三方远端 `gem-arc.vercel.app` | `page.tsx:98, 127-128`; `route.ts:21` | 🟡 (行为存在; 是否允许外发需产品确认) |
| 上游超时 | POST 上游请求无超时 | `route.ts:63-71` | ⬜ |
| 输出截断 | 日志每步截断 300 字符, 可能掩盖错误细节 | `page.tsx:130` | 🟡 |

### 4.6 依赖关系

| 依赖 | 说明 | 位置 |
|---|---|---|
| GemArc MCP 服务 (本地) | Streamable HTTP, 默认端口 8760 | `route.ts:20`; 实现在仓外 `E:\GemArc` |
| GemArc MCP 服务 (远端) | Vercel 部署 | `route.ts:21` |
| 顶栏与导航 | 页签注册, 默认顶栏页签, 执行组成员 | `frontend/src/lib/nav/registry.ts:604-611, 774`; `frontend/src/lib/nav/navStore.ts:94` |
| 布局 | 全高页面前缀 | `frontend/src/app/(app)/layout.tsx:33` |
| 视觉与图标 | lucide-react, clsx, Tailwind token 类 | `page.tsx:16-42` |
| 设计参考 | 交互与布局参考 | `docs/prototypes/2d · 原型-html/Prototype.dc.html` |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 原型页面 | `frontend/src/app/(app)/prototype/page.tsx` | 🟡 | 401 行; 流水线与通道逻辑真实, 预览与导出为静态 (`page.tsx:247-325, 381-397`) |
| GemArc 代理 | `frontend/src/app/api/gemarc/route.ts` | ✅ | 95 行; GET 健康检查 (`route.ts:82-95`), POST 白名单代理 (`route.ts:40-80`); 引入于提交 23d60e75 |
| 导航注册 | `frontend/src/lib/nav/registry.ts:604-611` | ✅ | 页签 id `prototype`, href `/prototype`; 描述字段仍为 "设计原型与实验区画廊 (wireframe / 游戏 / IPA 看板原型)" (`registry.ts:610`), 与本功能不符 |
| 顶栏默认页签 | `frontend/src/lib/nav/navStore.ts:94` | ✅ | `DEFAULT_HEADER_TABS` 含 `prototype` |
| 执行组 | `frontend/src/lib/nav/registry.ts:774` | ✅ | 组成员含 `prototype` |
| 全高布局 | `frontend/src/app/(app)/layout.tsx:33` | ✅ | `/prototype` 在 `WORKSPACE_FULL_HEIGHT_PREFIXES` 中 |
| 国际化文案 | `frontend/src/lib/i18n/zh-CN.ts:679-682`; `frontend/src/lib/i18n/en.ts:664-667` | 🟡 | 副标题仍为 "设计原型与实验区画廊" |
| 请求代理鉴权 | 无 | ⬜ | `proxy.ts:39` 排除 `api` 路径 |
| GemArc 服务实现 | 仓外 `E:\GemArc` | ❓ | 不在本仓内; 运行状态与响应结构无法从仓内核对 |
| 自动化测试 | 无 | ⬜ | 见 §7 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| 固定文案声称已生成结果 (`page.tsx:196`) | 用户会把未执行的结果当作真实输出, 违背 AGENTS.md §0 对示例数据的要求 | 删除固定文案; 仅在流水线成功后按真实输出渲染结果摘要 |
| 右侧预览为固定内容, 5D 数值为示例 (`page.tsx:304-305`) | 示例数字可被误读为推演结果 | 预览改为由生成结果驱动; 在示例内容上加可见的 "示例" 标识, 直到接入真实输出 |
| 生成类型未进入流水线与预览 (`page.tsx:127`, `page.tsx:248`) | FR-14-01 与 FR-14-06 无法成立 | 把 `genType` 加入流水线参数; 预览按类型分支渲染 |
| 流水线阶段标签静态 (`page.tsx:225`) | 进度不可见, 失败步骤无法定位 | 绑定 `runningStep` 与每步结果状态 |
| 每步输出截断 300 字符 (`page.tsx:130`) | 日志不完整, 错误信息可能缺失 | 保存完整输出并提供展开查看 |
| 导出、分享、交互预览、从模板无行为 (`page.tsx:216, 232, 236, 239, 395-397`) | FR-14-11、FR-14-12、FR-14-15、FR-14-16 未实现; SRS 将导出/分享标为已实现, 与代码不符 | 先实现导出 (下载 `gemarc_render` 结果), 再接分享与交互预览; 完成前按钮应禁用并标注 |
| 上下文添加无行为, 附加与放置仅为跳转 (`page.tsx:170-179, 389, 392`) | 引用不随原型流转 | 定义引用数据模型并接入任务卡与画布的写入接口 |
| 门控使用任一通道在线 (`page.tsx:111-114`) | 所选通道离线时仍可生成, 运行必然失败 | 门控改为只检查当前数据来源对应的通道 |
| `/api/gemarc` 无认证 (`route.ts` 全文; `proxy.ts:39`) | 任何可访问应用的调用方都可调用 12 个工具 | 在共享部署前加入会话校验; 本地开发仅绑定回环地址 |
| 示例模式默认外发提示词 (`page.tsx:98, 128`; `route.ts:21`) | 用户描述被发送到第三方远端 | 默认改为本地通道; 远端通道需显式确认 |
| 上游 POST 无超时 (`route.ts:63-71`) | 上游挂起会阻塞请求 | 增加 `AbortSignal.timeout` 与错误响应 |
| "当前 Branch 的接口" 只路由到本地 GemArc, 项目固定为 `prototype-demo` (`page.tsx:127`, `page.tsx:371`) | 标签与行为不符; 原型未绑定 Branch | 按当前 Branch 与 Project 传入上下文; 未绑定时显示未接入态 |
| SRS 数据源为 store seed (`SRS-PROTOTYPE-001.md:13`) | 与代码不符 | 以 GemArc 通道为准修订 SRS, 或新增 store 层 |
| 描述与副标题仍为 "画廊" (`registry.ts:610`; `zh-CN.ts:681`; `en.ts:666`) | 导航语义误导 | 改为与本功能一致的描述 |
| 图片与视频、动效、从模板未实现 (FR-14-16, FR-14-19, FR-14-20) | 功能缺口 | 按优先级排期; 媒体生成在未接入时显示未接入态 |
| 无任何自动化测试 (§7) | 回归无保障 | 为路由白名单、通道门控、流水线顺序与失败停止补单元测试 |

## 7. 验收与测试

现有测试:

| 测试 | 位置 | 验证内容 |
|---|---|---|
| 顶栏页签存在 | `frontend/src/components/__tests__/AppHeader.test.tsx:64-69` | `tab-prototype` 元素渲染 |
| 侧栏页签配置 | `frontend/src/components/__tests__/Sidebar.test.tsx:43` | `headerTabIds` 包含 `prototype` |

已验证: 导航页签的呈现与配置。

未验证:

- `/api/gemarc` 的白名单、方法拦截与上游转发 (无测试文件)。
- 页面的流水线顺序、失败停止、通道门控与设备切换 (无测试文件; `frontend/e2e` 中无原型相关规格)。
- GemArc 服务的在线状态与 `/healthz` 响应结构 (依赖仓外服务)。
- 导出、分享、交互预览 (未实现)。
- SRS-PROTOTYPE-001 §修订履历中 "浏览器实测 2026-11-15" 的记录: 仓内未找到对应的测试产物或截图, 无法核对, 状态 ❓。

## 8. 来源与裁决

FR 编号映射: FR-PT-001 → FR-14-01; FR-PT-002 → FR-14-02; FR-PT-003 → FR-14-03、FR-14-04、FR-14-05; FR-PT-004 → FR-14-06、FR-14-15; FR-PT-005 → FR-14-11、FR-14-12。FR-14-07、FR-14-08、FR-14-09、FR-14-10、FR-14-13、FR-14-14、FR-14-16 至 FR-14-20 不对应 SRS 的 FR 条目, 来源为参考原型 `Prototype.dc.html` 与代码, 在对应行的证据列中注明。

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-PROTOTYPE-001.md` | FR-PT-001 至 005 的需求条目, 流水线与 GemArc 工具映射, 文件映射 (§3) | SRS 将全部 FR 标为 ✅; 代码中生成类型、上下文、预览、导出与分享多为静态或无行为。数据源写为 store (line 13), 代码无 store, 实际通道为 GemArc | 状态以代码为准; 通道以代码为准 |
| `docs/prototypes/2d · 原型-html/README.md` | 参考原型的使用说明; 明确其为参考而非生产代码 | 无 | 仅作参考 |
| `docs/prototypes/2d · 原型-html/Prototype.dc.html` | 三栏布局 (左生成、中流水线与画面、右属性); 交互: 类型切换驱动中央画面 (`:197-228`), 4D 播放 (`:198, 215`), 5D 屏蔽开关 (`:136, 225`), 3D 机柜选择 (`:220`), UI 标签切换 (`:218`), 版本列表 (`:229`), 图片与视频标注未接入 (`:203`) | 代码为三区布局, 中央多画面画布与右侧属性面板均不存在; 交互未实现 | 布局以代码为准; 交互保留为 ⬜ 设计意图 |
| `docs/prototypes/INDEX.md` | 第 4 行映射: 2d · 原型 → `/prototype` → SRS-PROTOTYPE-001 → `page.tsx` | 与代码一致 | 采用 |
| `docs/architecture/2026-11-14-github-integration/INTEGRATION-MAP-GITHUB-REPOS-001.md` §1, §2 | GemArc 接入 `/api/gemarc` 代理; 流水线描述为 init → validate → simulate → build; Physis 3D 列为 GemArc 3D 物理原型 | 代码使用 brief / validate / simulate / build / verify / render 六步; Physis 未接入本页 | 工具名以代码为准; Physis 归为范围外 |
| `docs/architecture/2026-11-14-backend-support/WBS-BACKEND-SUPPORT-001.md:13` | `/api/gemarc` 路由处理器已通 | 与代码一致; 不证明 GemArc 服务在线 | 路由 ✅; 服务状态 ❓ |
| `docs/architecture/2026-11-15-lane-workspace/REVIEW-2026-11-14-IMPLEMENTATION.md:24` | GemArc 集成 "已闭环", 注明本地服务在仓外 | 与代码一致; 端到端未在仓内验证 | 路由 ✅; 端到端 ❓ |
| `docs/frontend/design/ui-3pane-arch.md:17` | 顶栏固定页签包含原型 `/prototype` | 与代码一致 | 采用 |
| `docs/INDEX.md:37` | 将 `prototypes/` 归类为 archive (PoC/调研) | 与 `prototypes/INDEX.md` 的 "参考设计" 定位不一致 | 以 `prototypes/INDEX.md` 为准; 该 INDEX 行视为过时 |
| `AGENTS.md` (仓库根, §0 一句话硬约束段) | 未接通数据的功能须显示未接入状态, 不得以示例数据冒充; mock / stub 不构成生产证明 | 当前页面违反该要求 (见 FR-14-18) | 作为 FR-14-18 的依据 |
| `frontend/src/app/(app)/prototype/page.tsx` | 当前页面实现 | 无 | 实现权威 |
| `frontend/src/app/api/gemarc/route.ts` | 当前代理实现 | 无 | 实现权威 |
| `frontend/src/proxy.ts` | 请求中间件, matcher 排除 `api` | 无 | 用于 §4.5 鉴权结论 |
| `frontend/src/lib/nav/registry.ts`, `navStore.ts`, `layout.tsx` | 导航、默认页签、全高布局 | 注册表描述 (`registry.ts:610`) 与本功能不符 | 描述列为 §6 差距 |
| `frontend/src/lib/i18n/zh-CN.ts:679-682`, `en.ts:664-667` | 页签标题与副标题 | 副标题仍为 "画廊" | 列为 §6 差距 |
| `frontend/src/components/__tests__/AppHeader.test.tsx`, `Sidebar.test.tsx` | 页签存在性测试 | 不覆盖页面行为 | 仅作为 §7 现有测试 |
| 提交 `23d60e75` | 引入 `/api/gemarc` 与真实流水线 | 无 | 实现来源证据 |
| 提交 `2719b66d`, `7471e77d` | 重做 `/prototype` 页面结构 | 无 | 页面历史证据 |
| `docs/requirements.md` PLG-004 及相关条目 | "原型预览" 指插件入口预览 | 与本功能的 "原型" 含义不同 | 不采用 |
| `docs/basic-design.md:4303` | 项目选择器的本地预览 | 与本功能无关 | 不采用 |
| `docs/frontend/design/ui-detailed-design.md` §6 | 其他页面的 PDF 原型图 | 与本功能无关 | 不采用 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` §17; `docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md` | W14 UI-1 流程原型 | "原型" 指桌面 UI 流程原型, 与本功能无关 | 不采用 |
| `docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md` | 协同与沙盒设计原型引用 | 与本功能无关 | 不采用 |
| `docs/reviews/W16-KMS-SOURCE-AUDIT-20261028.md` | 提及其他仓库的 gemarc-5d-atlas 分支 | 该目录不在本仓 (`docs/gemarc-5d-atlas` 不存在) | 不采用 |
