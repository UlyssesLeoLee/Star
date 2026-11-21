# F29 主题与视觉系统

> 状态摘要: 前端已具备 7 套内置主题（light / dark / christmas / charisma / shanghai / cel / physis）的切换、CSS 令牌与顶栏切换器，主题状态由 next-themes 维护并写入 `html` class；三层作用域（Personal > Tenant > Global）的后端 crate `domain-theme` 已有实体与服务代码，但未接入任何适配器、REST 路由或前端调用方，个人/租户主题的持久化与同步尚未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 为 Star 前端提供可切换的多套视觉主题，主题切换不刷新页面，切换状态跨刷新保留。
- 以 CSS 自定义属性（设计令牌）作为颜色、阴影、圆角、间距的唯一来源，Tailwind 工具类与 Three.js 效果共用同一套令牌。
- 为后续的三层主题作用域（Personal / Tenant / Global）提供后端领域模型与接口契约（`domain-theme`）。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | 主题枚举与调色板定义；next-themes 包装；顶栏主题切换器；用户菜单主题块；全局 CSS 令牌与主题覆盖样式；Tailwind 令牌映射；Three.js 取色工具；`domain-theme` 领域模型与端口 |
| 范围外 | 用户身份与租户隔离（读取自 `domain-identity` / `domain-tenant`）；字体文件的商用授权与分发；页面级布局（见各页面功能文档）；图表配色（见数据可视化相关功能） |

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 登录用户 | 在顶栏图标下拉中选择主题；按 `Cmd/Ctrl+Shift+T` 循环切换主题 |
| 登录用户（用户菜单） | 在用户菜单的“主题引擎”块中切换暗色 / 亮色两档 |
| 产品/设计 | 维护 `THEMES` 调色板与 CSS 令牌；按主题校验对比度与风格特征 |
| 后端/平台 | 在 `domain-theme` 中维护主题定义、注册与三层解析（尚未对外暴露） |

## 3. 需求

优先级说明: P0 = 当前前端已依赖的核心能力；P1 = 设计要求且应在近期闭合；P2 = 设计意图或后续扩展。状态标记按 F29 约定使用。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-29-01 | 内置 7 套主题（light、dark、christmas、charisma、shanghai、cel、physis）各有完整颜色、间距、圆角令牌 | P0 | ✅ | `frontend/src/lib/theme/types.ts:216-280`（`THEMES` 数组）；调色板 `types.ts:48-173` |
| FR-29-02 | 主题在 `<html>` 上以 class 生效，不跟随系统配色 | P0 | ✅ | `frontend/src/components/theme/ThemeProvider.tsx:33-39`（`attribute: "class"`, `enableSystem: false`） |
| FR-29-03 | 主题持久化到 `localStorage`，键名 `star-theme` | P0 | ✅ | `ThemeProvider.tsx:37`（`storageKey: "star-theme"`） |
| FR-29-04 | 切换主题不刷新页面 | P0 | ✅ | `ThemeProvider.tsx:33-39`（next-themes class 切换）；`frontend/src/components/UserMenu.tsx:78-89`（直接改 class，无 reload） |
| FR-29-05 | 顶栏主题切换器：图标按钮 + 下拉列出全部主题，当前项高亮 | P0 | ✅ | `frontend/src/components/theme/ThemeSwitcher.tsx:63-105`；挂载于 `frontend/src/components/AppHeader.tsx:142` |
| FR-29-06 | 键盘 `Cmd/Ctrl+Shift+T` 按 `THEMES` 顺序循环切换 | P1 | ✅ | `ThemeSwitcher.tsx:27-39` |
| FR-29-07 | 首屏避免主题闪烁：HTML 抑制水合告警，切换器未挂载前显示占位 | P0 | ✅ | `frontend/src/app/layout.tsx:69-72`（`suppressHydrationWarning`）；`ThemeSwitcher.tsx:41-49`（skeleton） |
| FR-29-08 | 用户菜单提供暗色 / 亮色双档切换 | P1 | 🟡 | `frontend/src/components/UserMenu.tsx:160-199`；仅两档，直接写 class 与 `localStorage`，不经过 next-themes 状态（见 §6 差距 G-02） |
| FR-29-09 | 全部主题的基础颜色在 `:root` / `.dark` / 各主题类上覆盖 | P0 | ✅ | `frontend/src/styles/theme.css:22`（`:root` 亮色）；`theme.css:146`（`.dark`）；`theme.css:245-354`（christmas）；`theme.css:363`（charisma）；`theme.css:491`（shanghai）；`theme.css:614`（cel）；physis 块见 `frontend/src/app/globals.css:638-656` |
| FR-29-10 | “cel”令牌（`--cel-*`）按主题覆盖，驱动卡片、按钮、阴影 | P0 | 🟡 | `frontend/src/app/globals.css:508-656`（按主题类定义）；与 `theme.css` 中同名令牌的默认值存在叠加冲突（见 §6 G-03） |
| FR-29-11 | 颜色令牌在 Tailwind 中映射为工具类（`bg-bg-*`、`text-ink-*`、`border-*`、`accent`、`ok/warn/err/info`） | P0 | ✅ | `frontend/tailwind.config.ts:34-64`；`frontend/src/app/globals.css:4`（`@config`） |
| FR-29-12 | 字号阶梯 micro 11px 至 display-lg 48px | P1 | ✅ | `frontend/tailwind.config.ts:66-75` |
| FR-29-13 | 字体栈：sans 以 Inter 为首，mono 以 JetBrains Mono 为首 | P1 | ✅ | `frontend/tailwind.config.ts:76-96` |
| FR-29-14 | 圆角按主题切换（亮色/暗色共用 3 档；pixel 主题 0px；shanghai、cel 各自一档） | P1 | 🟡 | `frontend/src/lib/theme/types.ts:188-213`（各主题圆角数组）；`frontend/src/styles/theme.css:115-118`（`--radius-*` 4/8/13/21）；`frontend/tailwind.config.ts:103-109`（`borderRadius` 3/5/6/8）。三处数值不一致，运行时以 CSS 变量为准，`types.ts` 的圆角数组无调用方（见 §6 G-04） |
| FR-29-15 | 间距令牌 4px 基础栅格 | P2 | 🟡 | `theme.css:103-112`（斐波那契 4/8/13/21/…）与 `types.ts:176-185`（4/8/12/16/…）数值不同；`themeToCss` 无调用方（见 §6 G-04） |
| FR-29-16 | Three.js / 动画效果从主题令牌取色，改主题无需改 3D 代码 | P1 | 🟡 | 取色工具 `frontend/src/lib/theme/themeTokens.ts:37-46`（`readThemeRgb`）；已知调用方 `frontend/src/app/automation-debug/components/AnimeCore3D.tsx:35,68-70`；`themeTokens.ts:5-7` 声称的其他 3D 组件调用方 ❓ 未逐一核实 |
| FR-29-17 | cel 调色四组（base / shadow / highlight / rim）由令牌派生，shadow 为 base × 0.16 | P2 | ✅ | `themeTokens.ts:78-92`（`resolveCelTokenPalette`，`mixRgb(base, [0,0,0], 0.84)`） |
| FR-29-18 | 弹层、Toast 使用主题色 | P1 | 🟡 | `theme.css` 中 `--popover-bg` 按主题定义（如 `theme.css:27`、`:150`）；Toast 颜色硬编码 `frontend/src/app/providers.tsx:67-74`，不随主题变化（见 §6 G-06） |
| FR-29-19 | 个人主题偏好（Personal）：按用户保存，跨设备同步 | P1 | ⬜ | 无后端接口与持久化；仅 `localStorage`（`UserMenu.tsx:62`）。`domain-theme` 的 Personal 作用域未接入（见 §4.3） |
| FR-29-20 | 租户主题（Tenant）与全局主题（Global）的三层解析 | P1 | 🟡 | 优先级常量 `frontend/src/lib/theme/types.ts:303-307`（`SCOPE_PRIORITY`，无调用方）；后端解析逻辑 `crates/domain-theme/src/lib.rs:16`（`ThemeService` resolve/list_available/set 文档注释）；未接入（见 §6 G-05） |
| FR-29-21 | 后端主题领域模型：`Theme` 聚合、`ThemeDefinition`、`ThemeId`、`ThemeScope`、`ThemeError` 8 变体 | P1 | 🟡 | `crates/domain-theme/src/entity.rs:9-21`；`crates/domain-theme/src/lib.rs:12,29-36`；错误映射 `crates/application/src/lib.rs:757-780`、`crates/star-api-rest/src/error.rs:445-470` |
| FR-29-22 | 主题不变量 INV-THEME-01（Personal > Tenant > Global 严格生效） | P1 | 🟡 | `crates/domain-theme/src/lib.rs:13-14`（`invariant` 模块声明 INV-THEME-01~04）；`crates/domain-theme/src/invariant.rs` 存在，未在本记录中逐条核实规则体 ❓ |
| FR-29-23 | 主题仓储端口 `ThemeRepository`、事件总线端口 `ThemeEventBus` | P1 | 🟡 | `crates/domain-theme/src/lib.rs:33`（re-export）；`crates/domain-theme/src/port.rs`；仅存在内存实现（`lib.rs:50-133`），crate 外无 `impl ThemeRepository` 命中 |
| FR-29-24 | 主题相关 REST 接口（列表、设置、解析） | P1 | ⬜ | `crates/star-api-rest/src` 无主题路由命中；`crates/star-api-rest/Cargo.toml:80` 仅依赖 `domain-theme` 做错误映射 |
| FR-29-25 | 主题切换的前端 API 客户端（设计称 `api/theme.ts` hook） | P1 | ⬜ | `ThemeProvider.tsx:26` 注释引用的 `api/theme.ts` 在 637ed19d 中不存在（`git ls-tree -r --name-only 637ed19d -- frontend/src` 无该路径） |
| FR-29-26 | 对比度与色盲检测（WCAG 2.1 AA，RISK-THEME-01） | P1 | ⬜ | 仅 CSS 注释中记录对比度数值，如 `theme.css:40-42`、`theme.css:382`；无自动化检测（`frontend/src/**` 无对应测试） |
| FR-29-27 | 高对比、Solarized 主题 | P2 | ⬜ | `types.ts:10` 的 `ThemeId` 联合类型包含 `"high-contrast"`、`"solarized"`，`THEMES`（`types.ts:216-280`）无对应条目 |
| FR-29-28 | 亮色默认、暗色优先的基线选择（设计稿 §1 “dark theme 优先”） | P2 | 🟡 | 代码默认 `defaultTheme="light"`（`frontend/src/app/providers.tsx:49`）；设计稿 `docs/frontend/design/ui-redesign-multica-style.md` §1 写 dark 优先。以代码为准，差异列入 §8 |
| FR-29-29 | 设计稿 §7 颜色 token（`#0a0d12` 背景、`#2f81f7` accent 等）作为默认暗色基线 | P2 | ⬜ | 设计稿 §7 颜色（`ui-redesign-multica-style.md` §7 `colors`）未出现在 `tailwind.config.ts` 或 `theme.css`；`dark` 调色板实际为 `types.ts:63-75` 的 `#00F0FF` 系列 |

## 4. 设计

### 4.1 概念模型

```
ThemeId（7 个内置，类型上 9 个）
  └─ ThemeDefinition { id, displayName, isDark, colors[], spacings[], radii[], version }   frontend/src/lib/theme/types.ts:37-45
        ├─ ColorToken { name, hex, alpha? }
        ├─ SpacingToken { name, px }
        └─ RadiusToken { name, px }

CSS 令牌层（运行时真正生效的一层）
  ├─ 基础色/语义色：--color-*、--ok-DEFAULT、--warn-DEFAULT、--err-DEFAULT、--info-DEFAULT   theme.css
  ├─ 背景/文字/边框 Tailwind 映射：--bg-*、--ink-*、--line-DEFAULT、--border-*              theme.css:58-82
  ├─ 阴影/发光：--shadow-*、--glow-*                                                        theme.css:96-100
  └─ 赛璐璐/像素引擎令牌：--cel-*（按主题类覆盖）                                            globals.css:508-656

主题作用域（领域层，尚未接入前端）
  ThemeScope = personal | tenant | global                                                  crates/domain-theme/src/value_object.rs（类型）
  优先级：Personal(3) > Tenant(2) > Global(1)                                              frontend/src/lib/theme/types.ts:303-307
```

- 主题与“主题色系”的关系: 每套主题在 `types.ts` 中共享 `STAR_SPACING` 与 `STAR_RADII`/专属圆角数组；颜色数组按主题独立。
- physis 主题的基色复用 cel 调色（`types.ts:275`，注释“基色复用 cel (CSS 层覆盖 3D 深紫蓝)”），3D 深紫蓝由 `globals.css:638-656` 覆盖。

### 4.2 数据模型

前端不持久化主题数据，仅使用 `localStorage` 键 `star-theme`（`ThemeProvider.tsx:37`、`UserMenu.tsx:62,83,87`）。

后端 `domain-theme`（代码存在，未接入数据库）:

| 对象 | 字段（代码可见） | 位置 |
|---|---|---|
| `Theme`（聚合根） | `theme_id: ThemeId`、`definition: ThemeDefinition`、`scope: ThemeScope`、`id: Uuid`（`theme.id` 在 `lib.rs:124` 使用） | `crates/domain-theme/src/entity.rs:11-21`；`lib.rs:124` |
| `ThemeDefinition` | `id`、`display_name`、`is_dark`（`lib.rs:158-160` 构造样例） | `crates/domain-theme/src/value_object.rs`（类型定义，本记录未逐字段核对） |
| `ScopeOwner` | `Personal { actor_id }`、`Tenant { tenant_id }`、`Global` | `crates/domain-theme/src/lib.rs:81-87` 的匹配分支 |
| `ThemeEvent` | `Changed`、`Registered`、`Deprecated`（设计注释） | `crates/domain-theme/src/lib.rs:13`；`event.rs`（❓ 变体体未逐一核实） |
| `ThemeError` | 8 变体: `NotFound`、`DuplicateId`、`IncompleteDefinition`、`InvalidHex`、`InvalidSpacing`、`PermissionDenied`、`Storage`、`Serialization` | `crates/application/src/lib.rs:759-776` 的映射分支 |

数据库表与 W/T/M 分类: 本功能在 637ed19d 中无数据库表定义。主题定义若落库，应按横展开规则判定为 Master（参考/设定，慢变，SCD Type 2）。该判定为设计意图，❓ 未在仓库 DDL 中核实。

### 4.3 接口（routes, APIs, ports）

前端导出（无 HTTP 接口）:

| 导出 | 签名或用途 | 位置 |
|---|---|---|
| `ThemeProvider` | props: `children`、`defaultTheme?`（默认 `"light"`）、`themes?`（默认 7 个 id） | `frontend/src/components/theme/ThemeProvider.tsx:13-41` |
| `ThemeSwitcher` | 无 props，内部用 `useTheme()` | `frontend/src/components/theme/ThemeSwitcher.tsx:19-20` |
| `THEMES` | `ThemeDefinition[]`（7 项） | `frontend/src/lib/theme/types.ts:216-280` |
| `getTheme(id)` | 按 id 查找，未知返回 `undefined` | `types.ts:283-285`（仅测试调用，见 §7） |
| `themeToCss(theme)` | 生成 CSS 变量文本 | `types.ts:288-300`（frontend/src 中无调用方） |
| `SCOPE_PRIORITY` | `Record<ThemeScope, number>` | `types.ts:303-307`（无调用方） |
| `readThemeRgb(token, fallback)` | 读取 `documentElement` 上的 hex 令牌 | `frontend/src/lib/theme/themeTokens.ts:37-46` |
| `parseHexColor` / `hexToRgb` / `mixRgb` | 颜色解析与线性混合 | `themeTokens.ts:18-31`、`49-54`、`57-64` |
| `resolveCelTokenPalette(opts)` | base / shadow / highlight / rim 四组 | `themeTokens.ts:78-92` |

后端端口（`domain-theme`，无外部适配器）:

| 端口 | 方法（代码可见） | 位置 |
|---|---|---|
| `ThemeRepository` | `find_by_id`、按 `theme_id + scope + owner` 查找、按 `scope` 列表、`list_builtin`、`save`、`delete` | `crates/domain-theme/src/lib.rs:63-133`（内存实现） |
| `ThemeEventBus` | `publish(event)` | `crates/domain-theme/src/lib.rs:135-137` |
| `ThemeService` | `new(repo, bus)`；设计注释列出 `resolve` / `list_available` / `set` | `crates/domain-theme/src/service.rs`；`lib.rs:16,34` |

REST 路由: 无。`crates/star-api-rest/src` 中未找到主题相关路由（见 FR-29-24）。

### 4.4 状态机与关键流程

主题状态由 next-themes 持有（`useTheme().theme`）。关键流程:

1. 首次挂载: `ThemeProvider` 以 `defaultTheme="light"` 初始化（`providers.tsx:49`）；`ThemeSwitcher` 在 `mounted` 前渲染 skeleton（`ThemeSwitcher.tsx:41-49`）。
2. 顶栏切换: 点击按钮展开列表（`ThemeSwitcher.tsx:65-74`）→ 点击选项调用 `setTheme(t.id)` 并关闭列表（`ThemeSwitcher.tsx:86-89`）→ next-themes 在 `<html>` 上替换主题 class → `globals.css` 与 `theme.css` 中对应选择器生效。
3. 快捷键循环: `Cmd/Ctrl+Shift+T` 取 `THEMES` 下一项（`ThemeSwitcher.tsx:30-35`）。
4. 用户菜单挂载（`UserMenu.tsx:59-73`）: 读取 `localStorage["star-theme"]`；值为 `"light"` 时移除 `dark` 并加 `light`，其他情况（包括无值）移除 `light` 并加 `dark`。该逻辑不移除其他主题 class（见 §6 G-02）。
5. 用户菜单切换（`UserMenu.tsx:78-89`）: 直接修改 `document.documentElement.classList` 与 `localStorage`，不调用 `setTheme`。

状态矩阵（主题切换器）:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 未挂载 | 首屏 | 占位方块 `aria-label="loading theme"`（`ThemeSwitcher.tsx:43-47`） | ✅ |
| 正常 | 挂载完成 | 当前主题图标按钮（`ThemeSwitcher.tsx:65-74`） | ✅ |
| 展开 | 点击按钮 | 列表，当前项 `aria-selected` 与 ✓（`ThemeSwitcher.tsx:81-101`） | ✅ |
| 未知主题 | `theme` 不在 `THEMES` | 回退 `THEMES[0]`（`ThemeSwitcher.tsx:61`） | ✅ |
| 错误 | 主题 id 不被 next-themes 识别 | 依据 `providers.tsx:49` 缺少 `physis`，见 §6 G-01 | 🟡 |

### 4.5 权限与安全

- 前端主题切换无鉴权门槛；`localStorage` 中仅存放主题字符串（`ThemeProvider.tsx:37`、`UserMenu.tsx:83,87`）。
- 后端 `ThemeError::PermissionDenied`（`crates/application/src/lib.rs:771-773`）已定义映射，但无调用它的服务路径（见 §4.3）。
- 三层作用域的写权限（Personal / Tenant / Global 的 set）未在前端或 REST 层执行（❓ 需 `ThemeService::set` 实现核对）。

### 4.6 依赖关系

| 依赖 | 用途 | 位置 |
|---|---|---|
| `next-themes` | 主题 class 与状态 | `ThemeProvider.tsx:7`、`ThemeSwitcher.tsx:7`、`Sidebar.tsx:28` |
| Tailwind v4 + `@config` | 令牌到工具类映射 | `frontend/src/app/globals.css:4-5`、`tailwind.config.ts:23-30`（`darkMode: "class"`） |
| `react-hot-toast` | Toast 容器 | `frontend/src/app/providers.tsx:62-76` |
| `lucide-react` 图标 | 用户菜单主题图标 | `UserMenu.tsx:165-195` |
| `domain-theme` | 后端领域模型 | `Cargo.toml:31`（workspace member）；`crates/application/Cargo.toml:38`；`crates/star-api-rest/Cargo.toml:80` |

### 4.7 页面设计

4.7 页面设计: 本功能无独立页面，入口见顶栏主题切换器（`frontend/src/components/AppHeader.tsx:142`）与用户菜单主题块（`frontend/src/components/UserMenu.tsx:160`）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 主题 Provider（next-themes 包装） | `frontend/src/components/theme/ThemeProvider.tsx` | ✅ | `ThemeProvider.tsx:28-41`；挂载于 `frontend/src/app/providers.tsx:49` |
| 主题切换器 | `frontend/src/components/theme/ThemeSwitcher.tsx` | ✅ | `ThemeSwitcher.tsx:19-106`；挂载于 `AppHeader.tsx:142` |
| 主题类型与调色板 | `frontend/src/lib/theme/types.ts` | ✅ | `types.ts:10`、`48-280` |
| 3D/动效取色工具 | `frontend/src/lib/theme/themeTokens.ts` | 🟡 | `themeTokens.ts:37-92`；已知调用 `AnimeCore3D.tsx:35` |
| 基础主题样式表 | `frontend/src/styles/theme.css` | ✅ | `theme.css:22-138`（亮色）、`146-230`（暗色）、`245-354`（christmas）、`363-…`（charisma）、`491-…`（shanghai）、`614-…`（cel） |
| 全局样式与主题覆盖 | `frontend/src/app/globals.css` | ✅ | `globals.css:1`（引入 theme.css）、`91-169`（body 背景）、`508-656`（`--cel-*` 按主题） |
| Tailwind 映射 | `frontend/tailwind.config.ts` | ✅ | `tailwind.config.ts:23-113` |
| 用户菜单主题块 | `frontend/src/components/UserMenu.tsx` | 🟡 | `UserMenu.tsx:59-89`、`160-199`；见 §6 G-02 |
| 主题类 Toast 样式 | `frontend/src/app/providers.tsx` | 🟡 | `providers.tsx:62-76`（硬编码颜色） |
| 领域实体、服务、端口 | `crates/domain-theme/src/` | 🟡 | `lib.rs:1-36`、`entity.rs:9-86`、`service.rs`、`port.rs` |
| 错误映射 | `crates/application/src/lib.rs:757-780`；`crates/star-api-rest/src/error.rs:445-470` | ✅ | 映射到 `THEME_NOT_FOUND` 等错误码 |
| 主题 REST 接口 | — | ⬜ | `crates/star-api-rest/src` 无主题路由 |
| 主题持久化适配器 | — | ⬜ | 无 `impl ThemeRepository` 在 crate 外 |

## 6. 差距与后续

| 编号 | 差距 | 影响 | 闭合它的下一步 |
|---|---|---|---|
| G-01 | `providers.tsx:49` 的 `themes` 列表缺少 `"physis"`，而 `ThemeSwitcher` 通过 `THEMES`（`types.ts:272`）列出 physis | 选择 physis 后 next-themes 可能不识别该主题，class 不生效；需浏览器验证（❓） | 将 `physis` 加入 `providers.tsx:49` 列表，或改为复用 `ThemeProvider` 默认列表（`ThemeProvider.tsx:31`） |
| G-02 | 用户菜单（`UserMenu.tsx:59-89`）与 next-themes 使用同一键 `star-theme` 但不同状态源；挂载时若存储值非 `"light"` 即加 `dark` 且只移除 `light` | 选择 christmas 等主题后刷新，`dark` 与主题 class 可能并存，令牌覆盖顺序不确定（按代码推断，需浏览器验证） | 用户菜单改为调用 `setTheme`；挂载逻辑删去，统一由 next-themes 读取 |
| G-03 | `--cel-*` 默认值两处定义：`theme.css:121-137`（`:root`）与 `globals.css:508-529`（`:root, html.dark`），后者在层叠上覆盖前者 | 无主题类时 `--cel-*` 呈暗色默认，而 `--color-*` 呈亮色默认（`theme.css:22-31`），同一页面可能色系错配 | 合并为单一默认定义，并在 `theme.css` 与 `globals.css` 之间只保留一处 |
| G-04 | 圆角与间距三处定义不一致（`types.ts:176-213`、`theme.css:103-118`、`tailwind.config.ts:103-109`） | 同一组件在不同入口得到不同圆角；`types.ts` 的间距与圆角数组无运行时消费 | 以 `theme.css` 为单一来源，`types.ts` 数组由构建生成或删除 |
| G-05 | `domain-theme` 无适配器、无 REST 路由、无前端 `api/theme.ts`；三层解析只有常量 `SCOPE_PRIORITY` | 个人/租户主题不可保存，FR-29-19、FR-29-20 不可用 | 实现 `ThemeRepository` 持久化适配器；暴露 `resolve/list/set` REST；前端新增 `api/theme.ts` 并替换 `ThemeProvider.tsx:26` 所指的 hook |
| G-06 | Toast 颜色硬编码（`providers.tsx:67-74`）；`--popover-bg` 按主题定义但 Toast 未引用 | 主题切换后 Toast 仍为深色 | Toast 样式改读 `--color-*` 令牌 |
| G-07 | WCAG AA 检测与色盲/高对比主题缺失（FR-29-26、FR-29-27） | RISK-THEME-01 未闭合 | 增加对比度单测（对 `THEMES` 文字/背景组合计算对比度）；补 `high-contrast` 条目 |
| G-08 | 设计稿 §7 颜色与 dark-first 基线未实现（FR-29-28、FR-29-29） | 默认外观与设计稿不一致 | 由设计决定基线；若采用设计稿颜色，需同步 `theme.css` 与 `types.ts` |
| G-09 | `ThemeSwitcher` 与 `UserMenu` 无组件测试（§7） | 切换回归无法被 CI 捕获 | 为二者补 React 测试，覆盖切换、持久化与未挂载态 |
| G-10 | `ThemeEvent` 变体与 `ThemeService` 方法体未逐条核实 | 领域行为描述只能到注释层 | 补 `service.rs` 与 `event.rs` 的单元测试证据 |

## 7. 验收与测试

已存在的测试:

| 文件 | 覆盖内容 |
|---|---|
| `frontend/src/lib/theme/__tests__/themeTokens.test.ts` | `parseHexColor`（hex 解析、短写、非 hex 返回 null、去空白）；`readThemeRgb`（读取、缺失与非 hex 回退）；`hexToRgb`；`mixRgb`（线性插值与 clamp）；`resolveCelTokenPalette`（shadow 派生与回退调色板） |
| `frontend/src/lib/theme/__tests__/types.test.ts` | `THEMES` 至少 5 个内置主题；每套有完整 color/spacing/radius；`isDark` 与 id 语义一致；christmas 圆角 > 0；charisma 0px 与紫金配色；shanghai 配色；`getTheme` 查找与未知 id；`themeToCss` 输出；`SCOPE_PRIORITY` 顺序 |
| `crates/domain-theme/src/lib.rs`（测试段，`lib.rs:141-233`） | `Theme` 构造与 `ThemeService` 使用内存仓储的单元测试（测试函数数量与断言未逐条核实，❓） |
| `crates/application/tests/p03_app_error_mapping.rs` | `domain_theme::ThemeError` 到 `ApplicationError` 的映射（8 变体，`:116-123`、`:259`） |
| `crates/star-api-rest/tests/p02_error_mapping_stage2.rs` | `ThemeError` 到 `RestError` 映射（`:190-253`、`:399-400`） |

已验证:

- 调色板数据完整性、`getTheme`、`themeToCss` 输出格式、3D 取色纯函数。
- 领域错误映射的变体覆盖。

未验证:

- `ThemeSwitcher` 的交互（下拉、快捷键循环、skeleton）无测试。
- 用户菜单主题块与 next-themes 的状态一致性（G-02）无测试。
- `providers.tsx` 的主题列表是否包含 physis（G-01）无测试。
- CSS 覆盖顺序（G-03）与对比度（G-07）无测试。
- `domain-theme` 的三层解析与 INV-THEME-01~04 无端到端证据。
- `frontend/e2e` 中 `git grep` 对主题的命中仅为 Monaco 编辑器主题（`e2e/editor-sakura-parity.spec.ts:122-127`），无主题切换的 e2e 用例（❓ 以目录全量扫描为准）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/specs/domain-theme-spec.md` | 职责（next-themes、三元组、三层优先级）；实体 `ThemePreference` / `ThemeRegistry`；不变量 INV-THEME-01~03；接口 `ThemePreferencePort` / `ThemeRegistryPort`；风险 RISK-THEME-01/02 | 代码命名为 `Theme` / `ThemeDefinition` / `ThemeRepository`（`entity.rs:9-11` 明确区分两者）；spec 称前端有 `api/theme.ts` 调用，代码不存在；spec 称 WCAG 检测，代码无实现 | 采用代码命名与结构作为现状；spec 的不变量与风险保留为设计意图，对应需求标 🟡 或 ⬜ |
| `docs/frontend/design/ui-redesign-multica-style.md` | §7 字号阶梯与字体栈（与 `tailwind.config.ts:66-96` 一致，记为 ✅）；§1 dark 优先；§7 颜色 token 与 `#0a0d12` 基线 | 代码默认 light（`providers.tsx:49`）；§7 颜色 hex 未出现在 `tailwind.config.ts` 或 `theme.css` | 以代码为准记录现状；设计颜色与 dark-first 列为 FR-29-28、FR-29-29 的 P2 设计意图 |
| `docs/frontend/design/dynamic-interaction-design.md` | §2.4 “软动效用 CSS + Tailwind transition”；§10.3 第 6 项 accessibility 缺口 | 全文以看板、甘特、日历、工作流交互为主，与主题无直接对应条款 | 仅取 §2.4 作为动效原则的引用；其余交互内容归属对应页面功能，本文不展开 |
| `frontend/src/components/theme/ThemeProvider.tsx`（代码） | next-themes 配置、`storageKey`、`themes` 默认列表 | 与 `providers.tsx:49` 列表不一致（缺 physis） | 代码为现状；差异列 G-01 |
| `frontend/src/components/theme/ThemeSwitcher.tsx`（代码） | 切换器结构、快捷键、skeleton | 无 | 代码为现状 |
| `frontend/src/lib/theme/types.ts`、`themeTokens.ts`（代码） | 调色板、圆角与间距数组、取色工具 | 圆角与间距与 CSS 不一致；`themeToCss`、`SCOPE_PRIORITY` 无调用方 | 代码为现状；差异列 G-04 |
| `frontend/src/styles/theme.css`、`frontend/src/app/globals.css`（代码） | CSS 令牌、主题覆盖、physis 块 | 两处 `--cel-*` 默认冲突 | 代码为现状；冲突列 G-03 |
| `crates/domain-theme/`（代码） | 实体、端口、服务注释、错误枚举 | 无适配器、无路由 | 代码为现状；未接入列 G-05 |
| `docs/reports/PHASE-W16-THEME-IMPL-REPORT.md`、`docs/reports/PHASE-AGENT-THEME-IMPL-REPORT.md` | 未读取 | 为历史报告，现状以代码为准 | 搁置；如需追溯历史决策，由后续核对补充（见 open_items） |
| `docs/basic-design.md:404`（仅 grep 命中） | domain-theme 列为第 3 号 supporting 行，指向 spec | 无 | 仅作索引引用 |
| `docs/arch-live/01-crate-map.html:133`（仅 grep 命中） | domain-theme 11 个 Rust 文件的图示 | 本记录未逐文件核对 | 搁置，不作为证据 |
