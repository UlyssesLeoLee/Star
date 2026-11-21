# F49 移动端与 PWA

> 状态摘要: 移动视口外壳（顶栏 MobileHeader、底栏 MobileBottomNav、窄屏隐藏桌面侧栏）、PWA 清单、Service Worker 注册与导航回退、安装引导、更新提示与离线页已在代码中落地。Web Push 仅为浏览器本地 MVP，无服务端订阅与发送；离线只覆盖访问过的页面 HTML，stale-while-revalidate 与离线数据缓存未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

1. 在手机视口（<768px）提供可用的 Star Web 壳：顶栏与底栏替代桌面侧栏，窄屏通过底栏打开项目目录。依据: `frontend/src/app/layout.tsx:80-99`、`frontend/src/components/Sidebar.tsx:118`、`frontend/src/components/MobileHeader.tsx:48`、`frontend/src/components/MobileBottomNav.tsx:70-75`。
2. 支持以 PWA 安装到主屏：Chrome/Edge 使用原生安装提示，iOS Safari 使用手动三步引导。依据: `frontend/public/manifest.json:1-10`、`frontend/src/components/PwaInstallPrompt.tsx:52-66`。
3. 网络中断时提供离线页，并回退到已访问页面的 HTML 壳。依据: `frontend/public/sw.js:97-113`、`frontend/src/app/offline/page.tsx:8-48`。
4. 在 /remote 提供浏览器通知权限、订阅状态与测试推送的入口。依据: `frontend/src/app/remote/page.tsx:36-39`、`frontend/src/components/remote/PushSettings.tsx:16-141`。

### 1.2 范围

范围内:
- 移动外壳: `frontend/src/app/layout.tsx` 注入的 MobileHeader、MobileBottomNav 与 PWA 浮层。
- PWA 清单 `frontend/public/manifest.json` 与 `/sw.js` 的响应头（`frontend/next.config.js:75-81`）。
- Service Worker `frontend/public/sw.js`、注册 `frontend/src/lib/pwaRegister.ts`、挂载 `frontend/src/components/PwaBoot.tsx`。
- 安装引导 `frontend/src/components/PwaInstallPrompt.tsx`、更新提示 `frontend/src/components/PwaUpdateToast.tsx`、离线页 `frontend/src/app/offline/page.tsx`。
- 浏览器 Push 客户端 `frontend/src/lib/pushClient.ts` 与推送设置卡 `frontend/src/components/remote/PushSettings.tsx`（挂载于 /remote）。

范围外:
- 底栏抽屉内的项目目录内容（`ProjectRunTree`）由 F01 拥有（`docs/features/F01-nav-project-lane.md:32, 73`）。本功能拥有触发按钮、抽屉容器与收起规则。
- /remote 的远程桌面、远程终端与远程文件页面及其 WebSocket relay（`frontend/src/app/remote/desktop`、`terminal`、`files`）。pin 上 `docs/features` 中没有对应功能文档，见 open_items。
- 桌面端 Tauri 应用（`docs/WBS.md:161`，8.8.1）。
- 开发与 mock 用的 MSW worker（`frontend/public/mockServiceWorker.js`），与 PWA 的 `sw.js` 无关。
- 服务端 Web Push 订阅与发送（FR-49-17）。
- 原生 Flutter 应用（FR-49-26），设计来源见 §8。

## 2. 用户与场景

| 角色 | 场景 | 对应需求 |
|---|---|---|
| 手机浏览器用户 | 窄屏浏览页面，用底栏打开项目目录 | FR-49-01、FR-49-03 |
| 安装用户 | 添加到主屏，从快捷方式进入 Worktree、Agent、Feedback、通知或项目 | FR-49-05、FR-49-06、FR-49-15、FR-49-16 |
| 网络不稳定用户 | 断网时打开已访问页面，或看到离线页 | FR-49-09、FR-49-13 |
| 通知订阅用户 | 在 /remote 启用、测试或关闭推送 | FR-49-18、FR-49-19 |
| 所有用户 | 版本发布后收到新版本提示并刷新 | FR-49-14 |

## 3. 需求

优先级列沿用源文档的标注（P1、V2 候选）；源文档未标注时记为 —。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-49-01 | <768px 显示顶栏 MobileHeader；≥768px 不显示 | — | ✅ | `frontend/src/components/MobileHeader.tsx:48`（md:hidden）；`layout.tsx:92` |
| FR-49-02 | <768px 隐藏桌面侧栏；≥768px 显示 | — | ✅ | `Sidebar.tsx:111, 118`（hidden md:flex）；`frontend/src/app/globals.css:63-68` |
| FR-49-03 | 窄屏底栏提供项目目录入口；Esc 与路由切换收起抽屉 | — | ✅ | `MobileBottomNav.tsx:76-93, 61-66, 45-47` |
| FR-49-04 | 底栏 5 域快捷入口（首页、Worktree、Agent、Feedback、更多）与“更多”抽屉 | — | 🗑 | 提交 `4dcf567d` 将 `mobile-nav-${item.id}` 替换为 `mobile-nav-project`（`MobileBottomNav.tsx:79`）；`frontend/src/components/__tests__/MobileBottomNav.test.tsx:90-97` 断言旧入口不存在 |
| FR-49-05 | PWA 清单包含 name、short_name、start_url、standalone、主题色与图标 | — | ✅ | `frontend/public/manifest.json:2-10, 12-36`；`layout.tsx:46-61` |
| FR-49-06 | 清单快捷方式直达 Worktree、Agent、Feedback、通知、项目 | — | 🟡 | `manifest.json:37-83`；/worktree、/agent、/projects 直接可达；/feedback 与 /notification 经 307 重定向（`frontend/src/lib/redirects.ts:105, 113-116`；`frontend/next.config.js:39-43`） |
| FR-49-07 | production 注册 /sw.js（scope /）；非 production 不注册 | — | ✅ | `frontend/src/lib/pwaRegister.ts:18, 23`；`PwaBoot.tsx:6-8` |
| FR-49-08 | SW 安装时预缓存壳资源，激活时清理旧版本缓存 | — | ✅ | `frontend/public/sw.js:13-16, 29-41, 44-56` |
| FR-49-09 | 导航请求网络优先；失败时回退到已缓存的 HTML，再回退到 /offline | — | ✅ | `sw.js:76-79, 97-111` |
| FR-49-10 | 离线读关键页面（Worktree、WorkItem），stale-while-revalidate 缓存 | V2 候选（`docs/frontend-design.md:785, 808`） | 🟡 | `CORE_ROUTES` 已声明（`sw.js:19-26`），fetch 处理未引用；API 不缓存（`sw.js:71-73`）；离线仅命中访问过的 HTML（`sw.js:101-102, 106-108`） |
| FR-49-11 | 静态资源（/_next/static、图标、字体）缓存优先 | — | ✅ | `sw.js:84-94, 115-126` |
| FR-49-12 | 业务 API 与 /v1 请求不经 SW 缓存 | — | ✅ | `sw.js:70-73` |
| FR-49-13 | 离线页提示离线，并提供重新加载与返回首页 | — | ✅ | `frontend/src/app/offline/page.tsx:8-41` |
| FR-49-14 | 新 SW 就绪时提示新版本，可立即刷新或稍后 | — | 🟡 | `pwaRegister.ts:27-35`；`PwaUpdateToast.tsx:24-42`；install 期间调用 `skipWaiting()`（`sw.js:38`），SKIP_WAITING 路径的实际触发未核实，见 §6 G6 |
| FR-49-15 | Chrome/Edge 原生安装提示；安装或“暂不”后 24h 内不再提示 | — | ✅ | `PwaInstallPrompt.tsx:52-57, 41-47, 117-141` |
| FR-49-16 | iOS Safari 手动三步添加到主屏引导 | — | ✅ | `PwaInstallPrompt.tsx:50, 59-66, 156-215` |
| FR-49-17 | 服务端 Web Push：VAPID 公钥、订阅与注销端点、推送发送 | — | ⬜ | `pushClient.ts:18, 63-65` 仅有注释与本地存储；`git grep` 在 crates、bff、frontend/src、docs/api 中搜索 `push/subscribe`、`v1/push`、`VAPID`，无实现 |
| FR-49-18 | 浏览器推送订阅：请求通知权限，并在本地记录订阅 | — | 🟡 | `pushClient.ts:41-74`（本地记录见 `:65, 72`）；无服务端登记（FR-49-17） |
| FR-49-19 | 测试推送与关闭推送 | — | 🟡 | `PushSettings.tsx:38-52`；`pushClient.ts:81, 109` 无超时地等待 `serviceWorker.ready`，见 §6 G8 |
| FR-49-20 | SW 接收 push 事件并显示通知；点击通知聚焦已打开窗口或打开目标 URL | — | ✅ | `sw.js:152-177, 180-195` |
| FR-49-21 | 顶栏通知角标显示真实未读数 | — | 🟡 | `MobileHeader.tsx:41` 为固定示例值 3；铃铛链接指向 /notification（`:81-82`），经 307 到 /inbox（`redirects.ts:105`）。AGENTS.md:22 要求未接通数据的功能显示未接入状态 |
| FR-49-22 | 顶栏搜索与菜单按钮打开应用矩阵 | — | 🟡 | `MobileHeader.tsx:40, 74, 96` 调用 `navStore.openMatrix`；`AppMatrixDrawer` 仅挂载于 `frontend/src/components/AppHeader.tsx:188`，无 AppShell 的路由下无效（§4.7.3） |
| FR-49-23 | 视口与安全区适配：100dvh、safe-area 内边距、可缩放至 5 倍 | — | ✅ | `layout.tsx:56-61, 82-88`；`MobileBottomNav.tsx:74` |
| FR-49-24 | iOS 真机上的后台挂起与通知权限行为 | — | ❓ | 源报告记为未跑（`docs/reports/PHASE-MOBILE-PWA-V0.4-REPORT.md:106`）；pin 上无真机用例。确认方式：iOS 真机依次执行安装、切后台、通知权限三项 |
| FR-49-25 | 移动端与 PWA 文案支持 en、zh-CN、ja | — | 🟡 | `MobileBottomNav.tsx:72, 80`、`MobileHeader.tsx:54, 76` 走 i18n；PWA 组件、离线页与推送卡含硬编码中文（§6 G10） |
| FR-49-26 | 原生移动 App（Flutter MVP，Android 为主） | — | ⬜ | 设计文档 `docs/mobile-flutter-mvp/01-requirements.md:1-3` 等；pin 上无 `pubspec.yaml` 与 `.dart` 文件 |
| FR-49-27 | 画布在移动 Web 触屏可用（NFR-COMPAT-3） | P1 | ❓ | `docs/requirements/SRS-STAR-CANVAS-001.md:1937`；pin 上未核实画布触屏交互，需 canvas 功能 owner 确认 |

## 4. 设计

### 4.1 概念模型

- **视口分流**: 断点为 768px（Tailwind `md`），与 `globals.css:63` 的 `max-width: 767px` 规则一致。<768px 显示 MobileHeader 与 MobileBottomNav；≥768px 显示 Sidebar，宽度 256px 或折叠态 64px（`globals.css:56, 59-61`）。
- **移动外壳（Mobile Shell）**: 由 RootLayout 注入，覆盖全部路由，不是页面级组件。包含顶栏、底栏与目录抽屉。
- **PWA 壳**: 清单（manifest）、Service Worker、两类缓存（静态资源与导航 HTML）与离线页。
- **安装状态**: `PwaInstallPrompt` 的三态 `hidden`、`native`、`ios-instructions`（`PwaInstallPrompt.tsx:15-18`）。
- **更新事件**: SW 进入 `installed` 且存在 controller 时派发 `star:pwa-updated`（`pwaRegister.ts:31-33`）。
- **推送**: 浏览器 `Notification.permission` 加本地订阅记录。服务端订阅与发送未建模（FR-49-17）。

### 4.2 数据模型

本功能没有数据库表。客户端持久化如下:

| 存储 | 键或名称 | 内容 | 证据 |
|---|---|---|---|
| localStorage | `star:pwa-install-dismissed` | 关闭时的毫秒时间戳，24h 内不再提示 | `PwaInstallPrompt.tsx:25, 41-47, 73-78` |
| localStorage | `star:push-enabled` | 值为 `"1"` 表示本地已启用 | `pushClient.ts:17, 31, 72, 91` |
| localStorage | `star:push-subscription` | `PushSubscription.toJSON()` 的结果 | `pushClient.ts:65, 92` |
| Cache Storage | `star-pwa-v1-static`、`star-pwa-v1-runtime` | 壳资源；已访问页面的 HTML | `sw.js:13-15, 101-102, 116-121` |
| 内存（navStore） | `isMatrixOpen` | 应用矩阵抽屉是否打开 | `frontend/src/lib/nav/navStore.ts:161, 169` |

清单字段见 `frontend/public/manifest.json`。顶栏角标计数是常量（`MobileHeader.tsx:41`），没有对应数据模型。服务端不存在推送订阅表：在 `crates`、`bff`、`frontend/src`、`docs/api` 中未发现相关实现（FR-49-17）。

### 4.3 接口

**HTTP 静态端点与页面路由**

| 路径 | 类型 | 说明 | 证据 |
|---|---|---|---|
| `/sw.js` | 静态文件 | 响应头 `Service-Worker-Allowed: /`、`Cache-Control: no-cache` | `frontend/next.config.js:75-81` |
| `/manifest.json` | 静态文件 | 由 layout metadata 引用 | `layout.tsx:46` |
| `/offline` | 页面路由 | 离线回退页 | `frontend/src/app/offline/page.tsx:8` |
| `/icon-192.png`、`/icon-512.png`、`/apple-touch-icon.png`、`/favicon.ico` | 静态文件 | 清单与 metadata 图标 | `manifest.json:14-35`；`layout.tsx:39-45` |
| `/mockServiceWorker.js` | 静态文件 | MSW，不属于 PWA | `next.config.js:68-74` |

**浏览器事件与 SW 消息**

| 通道 | 名称 | 位置 |
|---|---|---|
| window 事件 | `star:pwa-updated` | 派发 `pwaRegister.ts:33`；监听 `PwaUpdateToast.tsx:29` |
| SW message | `SKIP_WAITING` | 发送 `PwaUpdateToast.tsx:37`；处理 `sw.js:131-134` |
| SW message | `SIMULATE_PUSH` | 发送 `pushClient.ts:111-120`；处理 `sw.js:135-145` |
| SW event | `push`、`notificationclick` | `sw.js:152`、`sw.js:180` |

**浏览器 API**: `Notification`、`navigator.serviceWorker.register` 与 `ready`、`pushManager.getSubscription`、`subscribe`、`unsubscribe`、`beforeinstallprompt`。

**服务端 API**: 无。推送订阅与发送端点未实现（FR-49-17）。`/api/` 与 `/v1/` 路径在 SW 中直通网络（`sw.js:71-73`）。

### 4.4 状态机与关键流程

**A. Service Worker 生命周期（仅 production）**

1. `PwaBoot` 挂载后，`usePwaRegister` 在文档已 load 时立即注册，否则等待 load 事件（`pwaRegister.ts:43-47`），注册 `/sw.js`（`:23`）。
2. 注册成功后执行 `reg.update()`（`:26`）。
3. install 阶段预缓存壳资源，并调用 `skipWaiting()`（`sw.js:29-41`）。
4. activate 阶段删除不以 `star-pwa-v1` 开头的缓存，并调用 `clients.claim()`（`sw.js:44-56`）。
5. 页面侧: `updatefound` 后，新 worker 进入 `installed` 且存在 controller 时派发 `star:pwa-updated`（`pwaRegister.ts:27-35`）。
6. `PwaUpdateToast` 显示；“立即刷新”发送 `SKIP_WAITING`，200ms 后 reload（`PwaUpdateToast.tsx:35-42`）。

第 3 步已调用 `skipWaiting()`，waiting 状态可能很短，第 6 步的消息是否仍有实际作用未核实（§6 G6）。

**B. 导航请求回退（`sw.js:97-113`）**

网络成功 → 写入 runtime 缓存并返回。
网络失败 → runtime 缓存命中则返回 → 否则返回 static 缓存中的 `/offline` → 否则返回 503 `Offline`（`sw.js:111`）。

**C. 安装提示状态（`PwaInstallPrompt.tsx:27-78`）**

- `hidden` → 已是 standalone，或 24h 内曾关闭：保持隐藏（`:33-47`）。
- `hidden` → `native`：收到 `beforeinstallprompt`，阻止默认提示（`:52-55`）。
  - 点击“安装”：调用 `prompt()`；accepted → 持久关闭；dismissed → 仅关闭当前提示（`:117-133`）。
  - 点击“暂不”：持久关闭（`:134-141`）。
  - 点击 ×：仅关闭当前提示（`:144-151`）。
- `hidden` → `ios-instructions`：iOS UA 且无 MSStream，延迟 3s 显示（`:50, 59-66`）。点击 × 或“我知道了”持久关闭（`:177-184, 206-213`）。

**D. 推送启用、关闭与测试（`PushSettings.tsx:21-52`；`pushClient.ts:41-121`）**

- 启用: 若 `Notification.permission` 为 denied，直接返回失败（`pushClient.ts:45`）；否则请求权限（`:47-49`）。granted 时，若存在 `NEXT_PUBLIC_VAPID_PUBLIC_KEY`，则等待 `serviceWorker.ready` 并 `pushManager.subscribe`（`:53-61`）。随后写入 `star:push-enabled`（`:72`）。
- 关闭: 等待 `serviceWorker.ready`，取消订阅，清除两个本地键（`:77-94`）。
- 测试: 等待 `serviceWorker.ready`，向 SW 发送 `SIMULATE_PUSH`（`:109-120`）。

**E. 底栏目录（`MobileBottomNav.tsx:37-66`）**

- 收起 → 点击底栏按钮 → 展开。展开时才渲染 `ProjectRunTree`（`:83, 122`）。
- 展开 → Esc → 收起，焦点回到触发按钮（`:61-66, 55-58`）。
- 展开 → pathname 变化 → 收起（`:45-47`）。
- 展开 → 关闭按钮 → 收起（`:116`）。

### 4.5 权限与安全

- SW 只处理同源 GET 请求（`sw.js:63, 67-68`）。API 与 `/v1` 直通网络，不进入缓存（`:71-73`）。
- runtime 缓存保存已访问导航请求的 HTML（`:101-102`）。HTML 是否包含业务数据未核实；在共享设备上，已访问页面可能留在 Cache Storage 中（§6 G14）。
- 推送权限由浏览器决定；denied 时启用按钮禁用（`pushClient.ts:23-27`；`PushSettings.tsx:125`）。
- 推送订阅（含端点与密钥）以 JSON 明文保存在 localStorage（`pushClient.ts:65`），这是 MVP 形态。
- SW 的 `notificationclick` 直接用 payload 中的 `data.url` 调用 `openWindow`，未校验同源（`sw.js:182, 192`）。SIMULATE_PUSH 路径的 url 同样未校验（`pushClient.ts:117`）。
- 安装提示与更新提示没有权限门。页面内容的授权由各页面与 API 决定，不属于本功能。

### 4.6 依赖关系

| 依赖 | 用途 | 证据 |
|---|---|---|
| Next.js App Router | 根布局、metadata 与 viewport 导出、headers 与 redirects | `layout.tsx:34-61`；`next.config.js:39-43, 66-83` |
| 重定向表 | /notification、/feedback、/work-item、/issues 等旧路由 | `frontend/src/lib/redirects.ts:91-116`；`next.config.js:39-43` |
| Tailwind 断点与全局 CSS | 768px 分流、侧栏宽度变量 | `frontend/src/app/globals.css:54-68` |
| i18n | 文案 | `MobileHeader.tsx:38`；`MobileBottomNav.tsx:39`；`PwaUpdateToast.tsx:20`；`PwaInstallPrompt.tsx:28`；`layout.tsx:74` |
| navStore | 应用矩阵开关 | `MobileHeader.tsx:20, 40`；`navStore.ts:155-169` |
| AppShell 与 AppHeader | 挂载 AppMatrixDrawer | `frontend/src/components/AppShell.tsx:48-50`；`AppHeader.tsx:188` |
| 项目目录 ProjectRunTree（F01） | 底栏抽屉内容 | `MobileBottomNav.tsx:31, 122`；`docs/features/F01-nav-project-lane.md:73` |
| CommandBar | ⌘K 全局命令面板，与顶栏搜索按钮无连接 | `layout.tsx:128`；`MobileHeader.tsx` 无 CommandBar 引用 |
| 远程功能页面 | PushSettings 的宿主 | `frontend/src/app/remote/page.tsx:12, 38` |
| 通知收件箱（F15 范围） | 顶栏铃铛与推送通知的目标 | `MobileHeader.tsx:81-82`；`redirects.ts:105` |
| Web API | Notification、Push、Cache Storage | `pushClient.ts`；`sw.js` |

### 4.7 页面设计

#### 4.7.1 /offline

**文件**: `frontend/src/app/offline/page.tsx`

**布局**: `app/offline` 下没有 layout 文件，也不包含 AppShell。区域由根布局提供: 侧栏 `layout.tsx:77-79`（≥768 显示）、顶栏 MobileHeader `layout.tsx:92`（<768 显示）、主区 `layout.tsx:93`、底栏 MobileBottomNav `layout.tsx:99`。页面本身 `min-h-[60vh]` 居中（`offline/page.tsx:12`）。没有桌面 AppHeader。

**组件树**: RootLayout（`layout.tsx:63`）→ OfflinePage（`offline/page.tsx:8`）→ 图标容器与 CloudOff（`:14-15`）→ h1（`:17-19`）→ 说明段落（`:20-22`）→ 重新加载按钮（`:24-35`）→ 返回首页 Link（`:36-41`）→ 页脚（`:43-45`）。

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | SW 导航回退到预缓存的 /offline（`sw.js:109-110`），或直接访问 | 离线说明与两个按钮 | ✅ |
| 加载 | — | 静态页面，无加载态 | 不适用 |
| 空 | — | 不适用 | 不适用 |
| 错误 | 安装时离线，/offline 未进入 static 缓存 | SW 返回纯文本 503 `Offline`（`sw.js:111`），不显示本页 | 🟡 预缓存依赖安装时网络可达（`sw.js:35`） |
| 未授权 | — | 不适用，无鉴权 | 不适用 |
| 未接入 | — | 不适用 | 不适用 |
| 禁用 | — | 不适用 | 不适用 |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 重新加载 | 无 | `window.location.reload()`（`offline/page.tsx:26-30`） | 浏览器导航 | ✅ 仍离线时由 SW 再次回退 |
| 返回首页 | 无 | Link 到 "/"（`:36-41`） | 客户端路由；`app/page.tsx` 重定向到 /inbox（`redirects.ts:30-31` 注释） | 🟡 离线时 "/" 能否由缓存提供未核实（`sw.js:35` 预缓存了 "/"） |

**数据**: 无数据来源，全部为静态文案（`offline/page.tsx:18-44`）。

**与文档设计的差距**
- 页面文案承诺“继续使用上次看过的页面”（`offline/page.tsx:21`）。实际只有 URL 与 HTML 缓存命中的页面可用，业务数据不缓存（`sw.js:71-73`）。对应 FR-49-09、FR-49-10。
- 文案为硬编码中文，未走 i18n（`offline/page.tsx:18, 21, 34, 40, 44`）。对应 FR-49-25。

#### 4.7.2 /remote（推送设置区块）

**文件**: `frontend/src/app/remote/page.tsx`。页面归远程功能所有；本功能拥有其中的 PushSettings 挂载段（`remote/page.tsx:12, 36-39`）与卡片本身。

**布局**: `app/remote` 下没有 layout 文件，也不包含 AppShell。区域同 §4.7.1。主区内容宽度 `max-w-5xl`（`remote/page.tsx:23`），依次为 PageHeader、PushSettings 卡、运行时网格（`:41`）。

**组件树**: RemoteHomePage（`remote/page.tsx:15`）→ PageHeader（`:24-30`）→ PushSettings（`:37-39`）→ 运行时卡片网格（`:41` 起）。PushSettings: 容器 `data-testid="push-settings"`（`frontend/src/components/remote/PushSettings.tsx:70`）→ 图标（`:73-86`）→ 标题与状态文本（`:88-93`）→ 按钮组（`:98-137`）。

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 不支持 | 浏览器无 Notification API | 状态“浏览器不支持”（`PushSettings.tsx:56`），启用按钮禁用（`:125`） | ✅ |
| 未授权（权限被拒） | `Notification.permission === "denied"` | 状态“权限被拒, 请在浏览器设置中开启”（`:57`），启用按钮禁用 | ✅ |
| 未启用 | 默认 | 状态“未启用”，显示“启用”（`:58-60, 135`） | ✅ |
| 请求中 | 点击启用或关闭 | “请求中...”（`:135`），按钮 disabled（`:113, 125`） | ✅ |
| 已订阅 | `star:push-enabled` 为 `"1"` | “已订阅”，显示“测试”和“关闭”（`:99-120`） | 🟡 仅为本地记录，无服务端登记（FR-49-17） |
| 加载 | 挂载前 | 初始为 `default` 与未订阅（`:17-18`），挂载后读取（`:21-24`） | 🟡 首帧固定为默认值 |
| 错误 | 订阅失败 | 静默降级（`pushClient.ts:66-69`），UI 仍显示已订阅 | 🟡 |
| 未接入 | 服务端订阅 | 卡片文案“路上收 Worktree / Agent / Feedback 实时通知”（`:95`）未标明仅本地 | ⬜ FR-49-17 |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 启用 | 权限非 denied 或 unsupported | 请求权限；granted 后写入本地标记（`pushClient.ts:41-74`） | Notification、Push API、localStorage | 🟡 未配置 VAPID 公钥时只写本地标记（`pushClient.ts:53, 72`） |
| 关闭 | 已订阅 | 取消浏览器订阅并清除本地键（`pushClient.ts:77-94`） | Push API、localStorage | 🟡 无服务端注销；等待 `ready` 无超时（`:81`） |
| 测试 | 已订阅 | 通过 SW 显示本地通知（`PushSettings.tsx:45-52`；`pushClient.ts:100-121`） | SW postMessage | 🟡 SW 未注册时挂起（`pushClient.ts:109`） |

**数据**: `Notification.permission`；localStorage 的 `star:push-enabled` 与 `star:push-subscription`。无 REST 数据。

**与文档设计的差距**
- 卡片文案承诺实时通知（`PushSettings.tsx:95`），但服务端推送源未实现（FR-49-17）。
- VAPID 公钥未配置时，启用不生成真实订阅（`pushClient.ts:18, 53`）。对应 FR-49-18。
- 标题、状态与说明为硬编码中文（`PushSettings.tsx:47-48, 56-58, 89, 95`）。对应 FR-49-25。

#### 4.7.3 全站移动外壳（RootLayout 注入，非独立路由）

**文件**: `frontend/src/app/layout.tsx`；`frontend/src/components/MobileHeader.tsx`；`MobileBottomNav.tsx`；`PwaBoot.tsx`；`PwaUpdateToast.tsx`；`PwaInstallPrompt.tsx`。

**布局**
- 顶栏: MobileHeader，sticky，高 56px，<768 显示（`MobileHeader.tsx:45-48`；`layout.tsx:92`）。
- 侧栏: Sidebar，<768 隐藏（`Sidebar.tsx:118`），≥768 显示（`layout.tsx:77-79`）。
- 主区: `height: 100dvh`、`overflowY: auto`，底部内边距 `calc(4rem + safe-area)`（`layout.tsx:80-88`）。
- 底栏: MobileBottomNav，fixed bottom，z-index 9999，<768 显示（`MobileBottomNav.tsx:70-75`；`layout.tsx:99`）。
- 浮层: PwaUpdateToast（移动端 `bottom-20`，z-index 9997，`PwaUpdateToast.tsx:54-56`）；PwaInstallPrompt（z-index 9998，`PwaInstallPrompt.tsx:88-91`）；Toaster（`layout.tsx:113-126`）；CommandBar（`layout.tsx:128`）。
- 桌面 AppHeader 由 AppShell 挂载（`frontend/src/components/AppShell.tsx:48-50`），不在根布局中。

**组件树**: RootLayout（`layout.tsx:63`）→ I18nProvider（`:74`）→ Providers（`:75`）→ Sidebar（`:78`）、主区 → MobileHeader（`:92`）与 children；MobileBottomNav（`:99`）；PwaBoot（`:101`）；PwaUpdateToast（`:103`）；PwaInstallPrompt（`:105`）；OnboardingGuard（`:107`，不属于本功能）；Toaster（`:113`）；CommandBar（`:128`）。

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常（<768） | 窄屏视口 | 顶栏与底栏可见 | ✅ |
| 正常（≥768） | 宽屏视口 | 侧栏可见，顶栏与底栏隐藏 | ✅ |
| 目录展开 | 点击底栏按钮 | 抽屉展开并渲染 ProjectRunTree（`MobileBottomNav.tsx:96-123`） | ✅ |
| 目录收起 | Esc、路由变化或关闭按钮 | 抽屉隐藏（`:61-66, 45-47, 116`） | ✅ |
| 更新待刷新 | 收到 `star:pwa-updated` | 底部 toast | 🟡 见 §6 G6 |
| 安装提示 | 收到 beforeinstallprompt，或 iOS UA | 底部卡片 | ✅ |
| 离线 | 导航失败 | 回退到 /offline（§4.7.1） | 🟡 见 FR-49-10 |
| 未接入 | 通知角标 | 固定显示 3（`MobileHeader.tsx:41`） | 🟡 见 FR-49-21 |
| 错误 | 顶栏按钮打开矩阵，但当前路由未挂载 AppMatrixDrawer | 无反馈 | 🟡 见 FR-49-22 |
| 加载、禁用 | — | 不适用 | 不适用 |
| 未授权 | 目录内容 | 授权由 F01 的 ProjectRunTree 决定 | 不适用（归 F01） |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 点击底栏项目目录 | <768 | 切换 `aria-expanded`，渲染抽屉（`MobileBottomNav.tsx:83, 122`） | 本地 state | ✅ |
| Esc | 抽屉展开 | 收起并把焦点回到触发按钮（`:61-66, 55-58`） | — | ✅ |
| 路由变化 | 任意 | 收起抽屉（`:45-47`） | usePathname（`:38`） | ✅ |
| 顶栏搜索 | <768 | `navStore.openMatrix()`（`MobileHeader.tsx:74`） | navStore | 🟡 仅在挂载 AppHeader 的路由生效（FR-49-22） |
| 顶栏铃铛 | 任意 | 链接到 /notification（`:81-82`），经 307 到 /inbox | 路由重定向 | 🟡 角标固定（FR-49-21） |
| 顶栏菜单 | <768 | `navStore.openMatrix()`（`:96`） | navStore | 🟡 同搜索按钮 |
| 立即刷新 | 更新 toast 显示 | `postMessage(SKIP_WAITING)`，200ms 后 reload（`PwaUpdateToast.tsx:35-42`） | SW message | 🟡 见 §6 G6 |
| 稍后或 × | 更新 toast 显示 | 仅隐藏，不持久化（`:44-47`） | — | ✅ 下次 updatefound 时重新出现 |
| 安装 | native 态 | 调用 prompt()；accepted 则持久关闭（`PwaInstallPrompt.tsx:117-133`） | localStorage | ✅ |
| 暂不 | native 态 | 持久关闭 24h（`:134-141`） | localStorage | ✅ |
| × | native 或 iOS 态 | native 仅关闭当前提示（`:144-151`）；iOS 持久关闭（`:177-184`） | localStorage（iOS） | ✅ |
| 我知道了 | iOS 态 | 持久关闭（`:206-213`） | localStorage | ✅ |

**数据**: navStore 的 `isMatrixOpen`（`navStore.ts:161, 169`）；`usePathname`；i18n 字典键（如 `t.sidebar.*`、`t.ariaLabels.*`）；ROUTE_TITLES 常量（`MobileHeader.tsx:23-35`）；铃铛计数常量（`:41`）。

**与文档设计的差距**
- `docs/frontend/design/ui-3pane-arch.md:79` 规定 <768 为“移动端布局（走 PWA 离线模式）”。代码的离线只覆盖 HTML 回退（FR-49-09、FR-49-10）。
- 顶栏搜索按钮的 aria-label 为 `searchCmdK`（`MobileHeader.tsx:76`），但打开的是应用矩阵，不是 CommandBar（`layout.tsx:128`）。对应 FR-49-22。
- 底栏为单一项目目录入口，不含 5 域快捷入口（FR-49-04，🗑）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 视口分流与根布局 | `frontend/src/app/layout.tsx` | ✅ | `:56-61, 80-99` |
| 桌面侧栏隐藏 | `frontend/src/components/Sidebar.tsx` | ✅ | `:111, 118` |
| 窄屏全局样式 | `frontend/src/app/globals.css` | ✅ | `:54-68` |
| 移动顶栏 | `frontend/src/components/MobileHeader.tsx` | 🟡 | 角标固定（`:41`）；搜索与菜单走 navStore（`:40, 74, 96`） |
| 移动底栏与目录抽屉 | `frontend/src/components/MobileBottomNav.tsx` | ✅ | `:70-123`，单一项目目录入口 |
| 5 域底栏与更多抽屉 | 已删除（提交 `4dcf567d`） | 🗑 | `MobileBottomNav.test.tsx:90-97` |
| 抽屉内项目目录 | `frontend/src/components/run/ProjectRunTree.tsx`（导入于 `MobileBottomNav.tsx:31`） | 归 F01 | `MobileBottomNav.tsx:122` |
| PWA 清单 | `frontend/public/manifest.json` | ✅ | `:1-84` |
| SW 响应头 | `frontend/next.config.js` | ✅ | `:66-83` |
| Service Worker | `frontend/public/sw.js` | 🟡 | 导航与静态缓存已实现（`:76-126`）；`CORE_ROUTES` 未使用（`:19-26`）；SWR 未实现（`:5-6` 注释） |
| SW 注册 | `frontend/src/lib/pwaRegister.ts`、`frontend/src/components/PwaBoot.tsx` | ✅ | `pwaRegister.ts:18-47`；`PwaBoot.tsx:4-9` |
| 更新提示 | `frontend/src/components/PwaUpdateToast.tsx` | 🟡 | 组件完整（`:19-100`）；SKIP_WAITING 路径未核实（§6 G6） |
| 安装提示 | `frontend/src/components/PwaInstallPrompt.tsx` | ✅ | native 与 iOS 两路（`:52-66, 156-215`） |
| 离线页 | `frontend/src/app/offline/page.tsx` | ✅ | `:8-48` |
| Web Push 客户端 | `frontend/src/lib/pushClient.ts` | 🟡 | 本地 MVP（`:41-121`），无服务端登记 |
| 推送设置卡 | `frontend/src/components/remote/PushSettings.tsx` | 🟡 | 挂载于 `remote/page.tsx:38`，见 §4.7.2 |
| 服务端 Web Push | 无 | ⬜ | 全仓无实现（FR-49-17） |
| 原生 Flutter App | 无 | ⬜ | `docs/mobile-flutter-mvp/` 仅有设计（FR-49-26） |
| 移动 e2e | `frontend/e2e/remote-mobile.spec.ts` | 🟡 | 文件存在；断言已删除测试 ID 的用例失效（§7.1） |
| 底栏单元测试 | `frontend/src/components/__tests__/MobileBottomNav.test.tsx` | ✅ | 7 个用例（§7.1） |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| G1 顶栏角标固定为 3（`MobileHeader.tsx:41`） | 显示虚假未读数，违反未接通数据须显示未接入状态的规则（`AGENTS.md:22`） | 接入通知未读数（F15 范围）；未接入时显示未接入状态或隐藏角标 |
| G2 顶栏搜索与菜单调用 `navStore.openMatrix`（`MobileHeader.tsx:40, 74, 96`），AppMatrixDrawer 只挂载于 AppHeader（`AppHeader.tsx:188`） | /offline、/remote、/agent 无 AppShell，按钮无效 | 在根布局挂载矩阵抽屉，或让搜索按钮打开 CommandBar（`layout.tsx:128`） |
| G3 AppHeader 根节点无 `md:hidden`（`AppHeader.tsx:63-66`），AppShell 无条件挂载（`AppShell.tsx:48-50`），与 MobileHeader（`MobileHeader.tsx:48`）可能同时出现 | 窄屏下 AppShell 页面可能出现双顶栏。❓ | 在 390px 视口检查 AppShell 页面；确认后给 AppHeader 增加 `md:flex`，或隐藏 MobileHeader |
| G4 根布局内联 `paddingBottom`（`layout.tsx:86`）覆盖 `md:pb-0`（`:81`） | 桌面端也保留 4rem 底部空白 | 把底部内边距改为仅 <768 生效的类 |
| G5 SWR 未实现，`CORE_ROUTES` 未使用（`sw.js:5-6, 19-26`），API 不缓存（`:71-73`） | 离线只能查看访问过的 HTML，数据不可读 | 实现 `CORE_ROUTES` 的 SWR 或删除死代码并修正头部注释；为离线数据设计 IndexedDB 缓存（`docs/frontend-design.md:785`） |
| G6 install 期间调用 `skipWaiting`（`sw.js:38`），而更新提示发送 SKIP_WAITING（`PwaUpdateToast.tsx:37`） | 更新提示与实际激活顺序可能不一致；提示可能不出现，或 reload 时没有新版本。❓ | 在 production 构建中实测新版本发布流程；决定保留握手或去掉 install 期的 skipWaiting |
| G7 推送无服务端端点与 VAPID（`pushClient.ts:18, 63-65`） | 卡片与安装提示承诺的实时通知不存在（`PushSettings.tsx:95`；`PwaInstallPrompt.tsx:114`） | 增加 BFF 的 subscribe、unsubscribe、send 端点与 VAPID 环境变量；客户端改为登记服务端 |
| G8 pushClient 无超时等待 `serviceWorker.ready`（`pushClient.ts:81, 109`），而 SW 只在 production 注册（`pwaRegister.ts:18`） | 非 production 下关闭与测试推送按钮可能挂起 | 为 ready 等待加超时；无注册时显示不可用状态 |
| G9 清单快捷方式 /feedback、/notification 经 307（`manifest.json:60, 69`；`redirects.ts:105, 113-116`），`app/feedback`、`app/notification` 页面被路由重定向遮蔽 | 快捷方式多一跳 | 清单直接指向 /inbox?type=feedback 与 /inbox；或保留并在文档中说明 |
| G10 硬编码中文：`PwaUpdateToast.tsx:65, 67, 77, 85`；`PwaInstallPrompt.tsx:112-115, 132, 140, 171, 174, 190, 197, 203`；`PushSettings.tsx:47-48, 56-58, 89, 95`；`offline/page.tsx:18-44`；`MobileHeader.tsx:24-34`；`sw.js:171-172`；`manifest.json:2-3, 39-82` | 三语要求未满足（FR-49-25） | 迁移到 i18n 字典；SW 通知动作文案改由页面经消息传入 |
| G11 e2e 断言已删除的测试 ID（`remote-mobile.spec.ts:34-38, 45-47`）；底栏重定向用例（`:82-88`）点击不存在的 `mobile-nav-worktree`，并期望 /issues 旧链 | 移动 e2e 套件无法通过（静态判断） | 改写为 `mobile-nav-project` 与 `mobile-project-directory`；去掉 /issues 相关期望 |
| G12 离线页“返回首页”依赖网络（`offline/page.tsx:36-41`） | 离线时该链接可能无效。❓ | 在浏览器离线模式下验证；必要时改为只提供刷新 |
| G13 Flutter 原生 App 只有设计（`docs/mobile-flutter-mvp/`），pin 上无代码 | 原生移动端无交付物 | 决定该轨道是否继续；若继续，按其实现规格建立工程骨架 |
| G14 HTML 运行时缓存（`sw.js:101-102`）在共享设备上的留存 | 可能保留已访问页面的内容。❓ | 确认 HTML 是否含业务数据；登出时清除 Cache Storage |
| G15 iOS 真机未验证（FR-49-24） | iOS 后台与通知行为未知 | 按真机清单执行 |
| G16 清单声明 maskable 图标（`manifest.json:23, 29`），安全区未核实 | 部分 Android 启动器可能裁切图标 | 制作符合 maskable 安全区的图标并验证 |
| G17 安装与更新组件、推送客户端和离线页无单元测试（pin 上无对应测试文件） | 关键流程无自动化保护 | 为 dismiss 持久化、事件派发、SKIP_WAITING 分支补 vitest |
| G18 画布触屏未核实（FR-49-27） | NFR-COMPAT-3 无证据 | 由 canvas 功能 owner 在移动 Web 上核实 |
| G19 WBS 行 8.1.2 标记 100% 已完成（`docs/WBS.md:122`），与 G1–G11 不一致 | 进度表述高于实际状态 | 将 8.1.2 改为部分完成，并引用本文 §3 |

## 7. 验收与测试

### 7.1 现有测试

| 测试 | 位置 | 覆盖 | 状态 |
|---|---|---|---|
| 底栏单元测试 | `frontend/src/components/__tests__/MobileBottomNav.test.tsx:54-123` | 三语标签（`:64-76`）；打开并渲染目录树（`:78-88`）；旧 5 域入口不存在（`:90-97`）；Esc 恢复焦点（`:99-108`）；路由变化收起（`:110-123`） | ✅ 用例与当前组件对应；本核对未执行，运行结果 ❓ |
| 移动 e2e（iPhone 13 视口） | `frontend/e2e/remote-mobile.spec.ts:24-105` | 8 个用例：布局（`:27-41`）、更多抽屉（`:43-48`）、/remote 与三件套（`:50-80`）、底栏切换到 Worktree（`:82-88`）、iOS 安装引导（`:96-104`） | 🟡 `:27-48` 与 `:82-88` 断言的测试 ID 在 pin 上已不存在；`:50-80` 属远程功能；`:96-104` 的文本断言与 `PwaInstallPrompt.tsx:190-197` 文案一致；运行结果 ❓ |
| Playwright 配置 | `frontend/playwright.config.ts:4, 11, 22-23` | testDir `./e2e`，baseURL `:3001`，webServer 为 `npm run dev` | ✅ 配置存在 |

### 7.2 已核实（静态读取）
- 视口分流类名与根布局挂载点（§5）。
- SW 的缓存分支与消息处理（§4.4 A、B）。
- 安装提示三态与持久化键（§4.4 C）。
- 推送本地 MVP 的读写路径（§4.4 D）。
- 底栏的展开、收起与焦点规则（§4.4 E）。

### 7.3 未核实
- SW 在 production 构建中的注册、预缓存、激活与更新（需浏览器运行）。
- 离线导航回退的实际行为（需断网测试）。
- 更新提示的 SKIP_WAITING 路径（§6 G6）。
- Chrome 原生安装提示与 iOS 真机行为（FR-49-24）。
- 推送订阅的端到端投递：服务端不存在，无法验证（FR-49-17）。
- 窄屏双顶栏（§6 G3）与桌面底部空白（§6 G4）的视觉表现。
- 画布触屏（FR-49-27）。
- `frontend/vitest.config.ts:42-46` 的 exclude 只含 `node_modules` 与 `.next`；e2e spec 是否被 vitest 的 include 规则拾取，未核实。
- 清单图标是否满足 maskable 安全区（§6 G16）。
- PWA 审计（如 Lighthouse）：pin 上未发现相关配置或依赖（`frontend/package.json` 与 `.github` 中无匹配）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/reports/PHASE-MOBILE-PWA-IMPL-REPORT.md` | 基础 PWA 范围：清单、SW、移动布局、离线页、底栏 | 底栏描述为 5 域快捷导航与更多抽屉（`:28`）；代码为单一项目目录入口（`MobileBottomNav.tsx:76-93`） | 采用代码现状；5 域底栏不作为当前设计 |
| `docs/reports/PHASE-MOBILE-PWA-V0.2-REPORT.md` | /remote 三件套入口；底栏“更多”抽屉中的 Remote 入口（`:33`） | 抽屉与 Remote 入口在代码中不存在（`MobileBottomNav.test.tsx:90-97`）；/remote 页面仍存在（`remote/page.tsx`） | 底栏远程入口 🗑；远程页面归远程功能 |
| `docs/reports/PHASE-MOBILE-PWA-V0.3-REPORT.md` | SW 更新提示、安装提示（含 iOS 三步）及更新流程（`:69-70`） | 组件与描述一致；更新流程中 SKIP_WAITING 与 install 期 skipWaiting 的组合未核实（`sw.js:38`） | 采用；更新流程标 🟡 |
| `docs/reports/PHASE-MOBILE-PWA-V0.4-REPORT.md` | Web Push 客户端、SW push 处理、PushSettings、移动 e2e、缺口 G1–G8（`:100-109`） | 报告称 e2e 8/8 通过（`:76-83`），但 pin 上 spec 断言的测试 ID 不存在（`remote-mobile.spec.ts:34-38`）；报告 G1–G8 仍与代码一致 | 采用缺口清单；e2e 通过声明不采纳 |
| `docs/reports/WBS-UI1-PROJECT-FIRST-NAV-20261022.md:19` | 以 Project 层级目录替代旧模块快捷栏 | 与代码一致（`MobileBottomNav.tsx:76-123`） | 采用为当前底栏设计 |
| `docs/WBS.md:120-122` | 8.1 与 8.1.2“PWA 与移动端导航”标记为 100% 已完成 | 与 G1–G11 不一致 | 不采纳 100%；以本文 §3 为准；见 §6 G19 |
| `docs/frontend/design/ui-3pane-arch.md:75-79` | 自适应规则：<768 为移动布局（走 PWA 离线模式） | 离线仅 HTML 回退（`sw.js:97-113`） | 采用布局规则；离线按 FR-49-09、FR-49-10 标注 |
| `docs/frontend-design.md:785, 808` | V2 候选：关键页面离线读 | 代码未实现（§6 G5） | 作为 V2 候选；FR-49-10 标 🟡 |
| `docs/frontend/design/ui-redesign-multica-style.md:194-196` | 移动响应式与 PWA 为后续扩展，当前仅 web | 移动响应式与 PWA 壳已存在 | 作为历史范围说明，不作为当前限制 |
| `docs/frontend/design/dynamic-interaction-design.md:233, 235` | 离线编辑暂用 localStorage；真离线 PWA 待后续；移动端拖动待验证 | 代码未见 PWA 层的离线编辑缓存；移动拖动未核实 | 作为待办；离线编辑 ⬜，拖动 ❓ |
| `docs/internal-design.md:1454, 1632` | SW 用于静态资源缓存（V1）；是否上 PWA 为 V1 候选 | 代码实现静态与导航缓存（`sw.js:84-113`） | 采用；PWA V1 范围按已实现部分计 |
| `docs/requirements/SRS-STAR-CANVAS-001.md:259, 338, 428-432, 1936-1937, 2438` | 画布仅支持 Web 与 PWA，排除移动 native；NFR-COMPAT-3 为 P1 | 与 Flutter 轨道不冲突，范围限定为画布 | FR-49-27 取 P1；native 排除只适用于画布 |
| `docs/design/BD-STAR-CANVAS-001.md:93, 1526` | ADR-CANVAS-008 的设计侧记录 | 同上 | 同上 |
| `docs/requirements/SRS-CANVAS-AGENT-001.md:123, 132` | 本期不含移动专属体验与离线编辑 | 与 F49 无冲突 | 不作为 F49 的需求来源 |
| `docs/mobile-flutter-mvp/01-requirements.md`、`02-basic-design.md`、`03-detailed-design.md`、`04-implementation-spec.md` | 原生 Flutter MVP 的 IPA 三段式设计与实现规格 | pin 上无 Flutter 代码（无 `pubspec.yaml`、无 `.dart`） | 采用为设计来源；FR-49-26 标 ⬜；不并入 PWA 状态 |
| `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md:1-3` | Flutter 草稿 | 文首声明已被 IPA 三段式文件取代 | 历史草稿，不作为来源 |
| `docs/architecture/2026-08-26-upgrade/adr/0052-file-session-management.md:96`；`docs/architecture/file-session-management/01-requirements.md:124`；`docs/architecture/file-session-management/README.md:96` | z-index 分层 L7 包含 PWA toast 与底栏 | 代码 z-index：底栏 9999（`MobileBottomNav.tsx:74`）、安装提示 9998（`PwaInstallPrompt.tsx:89`）、更新 toast 9997（`PwaUpdateToast.tsx:56`），与 L7 一致 | 采用为层级约束 |
| `docs/features/F01-nav-project-lane.md:32, 73, 192` | 窄屏通过底栏抽屉打开项目目录（FR-01-34） | 与代码一致 | 目录内容归 F01；本功能拥有触发与容器 |
| `docs/briefs/STAR-I18N-TAKEOVER.md:23`；`docs/reports/STAR-I18N-AUDIT-BASELINE.json`；`docs/reports/STAR-I18N-EXTRACT.json` | 列出底栏与 PWA 组件中待 i18n 的文案 | 与硬编码现状一致 | 作为 §6 G10 的证据 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:889`；`docs/reports/PHASE-NAMING-WEB-REMEDIATION-20261106.md:35` | 底栏测试文案随 LANE 改名的记录 | 测试断言使用 LANE（`MobileBottomNav.test.tsx:65-67`）；测试 mock 文案仍含 Engineering Run（`:41`） | 背景记录；mock 文案列为低优先级整理项 |
| `AGENTS.md:22` | 未接通授权数据的功能须显示未接入状态，不得以示例数据冒充 | 角标固定为 3（`MobileHeader.tsx:41`） | 作为 FR-49-21 与 §6 G1 的依据 |
| `docs/architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md:28` | 提及 PwaBoot 与 OnboardingGuard 共同挂载 | OnboardingGuard 不属于本功能 | 仅引用挂载位置，不纳入 F49 |
| `frontend/next.config.js`；`frontend/src/lib/redirects.ts` | headers、redirects；`redirects.ts:10` 注明 /worktree 为活跃路由 | — | 采用为实现证据 |
| `frontend/src/instrumentation.ts`；`frontend/public/mockServiceWorker.js`；`frontend/next.config.js:58-74` | MSW 开发与 mock worker | 与 PWA 的 `sw.js` 为不同 worker | 设为范围外 |
| `frontend/src/app/remote/page.tsx` | 远程首页与 PushSettings 挂载 | 远程页面不归 F49 | 仅纳入 PushSettings 挂载段（§4.7.2） |
| `frontend/e2e/remote-mobile.spec.ts` | 移动 e2e 与远程三件套用例 | 部分用例失效（§7.1） | 作为 §7.1 的证据 |
