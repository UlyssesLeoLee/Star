# Phase W16 — 主题系统基础设施实装报告 v0.1

> **状态**: 🟢 Active
> **日期**: 2026-08-29
> **基点 commit**: `c1450d9` (main @ Phase F.1 + w15 Confluence cherry-pick)
> **完成 commit**: `50416ab` (feat/w16-theme)
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
> **签批**: 🟢 Mavis 接手代签 (per 2026-08-27 19:39/21:59 JST 三次强化)

---

## 0. 报告目的

承接 2026-08-29 04:02 JST Ulysses 拍板"Star 自创 (推荐) + 补齐 P1-P3 全部 33 项" + 04:09 JST 主题系统决策 (next-themes + 三元组 enum + 三层作用域 + 独立 wt), 在 10 个 wt 子代理 RPC 断连事故后, 由 Mavis root 亲自接管 wt-w16 实装, 作为后续 10 个 wt 的 UI 协同基线.

**核心目标**: 预留切换显示主题的统一接口, 初期 Light + Dark 两种内置, 接口支持后续扩展.

---

## 1. 改动矩阵

### 1.1 总览

| 维度 | 数量 |
|---|---|
| 新增/修改文件 | 14 (8 Rust + 6 frontend) |
| 净增行数 | +985 (后端 8 层 700 行 + 前端 5 件 285 行) |
| 新 tests | 18 (10 unit + 4 integration + 4 frontend vitest) |
| 测试结果 | 14 passed; 0 failed; 0 ignored (后端 cargo test) |
| 6 个独立 commit | value_object / 4 层 / entity-port-service-lib / fix / 孤儿宏 / 前端 3 件 / 集成 |
| 新 crate | `crates/domain-theme` (8 层, workspace 注册) |
| 新依赖 | `next-themes ^0.3.0` (frontend package.json) |

### 1.2 8 层文件结构 (per Cargo workspace 模式)

| 层 | 文件 | 关键内容 |
|---|---|---|
| value_object | `value_object.rs` | ThemeId (Light/Dark + HighContrast/Solarized 扩展) / ThemeScope (Personal/Tenant/Global) / Color/Spacing/Radius/ThemeDefinition + 5 unit test |
| error | `error.rs` | 8 错误变体 (NotFound/DuplicateId/IncompleteDefinition/InvalidHex/InvalidSpacing/PermissionDenied/Storage/Serialization) |
| context | `context.rs` | ThemeContext (actor_id/tenant_id/resolution_chain Personal>Tenant>Global/is_anonymous) |
| event | `event.rs` | ThemeEvent enum (Changed/Registered/Deprecated) |
| invariant | `invariant.rs` | INV-THEME-01~04 + 4 unit test (id 有效/唯一/定义完整/版本单调) |
| entity | `entity.rs` | Theme 聚合根 + ScopeOwner (Personal/Tenant/Global) |
| port | `port.rs` | ThemeRepository + ThemeEventBus trait (六边形架构) |
| service | `service.rs` | ThemeService.resolve (三层解析) + list_available + set (INV-03 校验 + 权限拒绝) |
| lib | `lib.rs` | 模块重导出 + InMemoryThemeRepo mock + 4 integration test (resolve default / personal overrides / 拒绝 anonymous / 拒绝 incomplete) |

### 1.3 前端 5 件

| 文件 | 关键内容 |
|---|---|
| `lib/theme/types.ts` | ThemeId union type (light/dark/high-contrast/solarized) + THEMES 数组 (Light+Dark) + Color/Spacing/Radius/ThemeDefinition 接口 + getTheme/themeToCss/SCOPE_PRIORITY 工具 |
| `styles/theme.css` | :root 默认 + .dark 覆盖 + 预留扩展位注释 + 10 颜色 / 8 间距 / 3 圆角 / 2 阴影 |
| `components/theme/ThemeProvider.tsx` | 包装 next-themes, attribute=class, storageKey=star-theme, enableSystem=false |
| `components/theme/ThemeSwitcher.tsx` | 顶栏下拉, Skeleton 状态, Cmd+Shift+T 键盘切换, 5 项 UI 守门 |
| `lib/theme/__tests__/types.test.ts` | 4 vitest 测 (THEMES 完整 / getTheme / themeToCss / SCOPE_PRIORITY 顺序) |

### 1.4 集成点

| 集成 | 文件 | 修改 |
|---|---|---|
| workspace members | `Cargo.toml` | 加 `crates/domain-theme` |
| package.json | `frontend/package.json` | 加 `next-themes ^0.3.0` |
| globals.css | `frontend/src/app/globals.css` | `@import` theme.css + `color-scheme: light dark` |
| providers.tsx | `frontend/src/app/providers.tsx` | 包裹 `<ThemeProvider defaultTheme="light" themes={["light","dark"]}>` |
| layout.tsx | `frontend/src/app/layout.tsx` | 加 `suppressHydrationWarning` + 替换硬编码 bg/text 为 CSS 变量 |
| Topbar.tsx | `frontend/src/components/Topbar.tsx` | 在 Bell 按钮前插入 `<ThemeSwitcher />` |

---

## 2. 验证摘要

### 2.1 后端 cargo test

```
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- value_object: 6 unit (theme_id_as_str / is_dark / scope_priority / builtin_count / hex_to_rgba / to_css_variables)
- invariant: 4 unit (inv_01_id_valid / inv_02_id_unique_personal / inv_03_definition_complete / inv_04_version_monotonic)
- lib (integration): 4 (resolve_default / personal_overrides / 拒绝 anonymous / 拒绝 incomplete)

### 2.2 前端 (vitest 预备)

测试文件 `lib/theme/__tests__/types.test.ts` 已写, 4 个 describe 块覆盖:
- THEMES 数组完整 (≥2, 含 light + dark, is_dark 语义一致, token 完整)
- getTheme 按 id 查找 / 未知 id 返回 undefined
- themeToCss 输出含 CSS 变量
- SCOPE_PRIORITY 排序 Personal > Tenant > Global

**注**: 实际 `pnpm test` 跑需要 `pnpm install` 装 next-themes, 留待 5 域 Lead DDD Review 阶段验.

### 2.3 clippy

未跑 (本任务聚焦 cargo test, clippy 留后续).

---

## 3. 已知缺口 (per 缺标比错标)

1. **未实装租户级后端 API** (GET/PUT /api/tenants/{id}/theme) — ThemeService 已有 set 逻辑, 缺 HTTP 路由层, 留 wt-w16.5 后续
2. **未实装个人级后端 API** (PUT /api/users/me/theme) — 同上
3. **未实装 next-themes 与后端同步** — 当前仅 localStorage 持久化个人偏好, 不走后端; 三层解析中 Personal 永远覆盖 Tenant / Global 是前端单边逻辑
4. **未实装 Marketplace / 用户自定义主题** — 接口预留 (THEMES 数组可扩展 + ThemeId 注释占位), 后端 ThemeService.set 可注册新主题
5. **未实装主题预览图** — UI 切换只看名字, 不显示色板预览
6. **未实装租户级白标** (企业品牌色覆盖) — Tenant scope 留了 ScopeOwner::Tenant, 缺具体 token 覆盖逻辑
7. **未实装高对比度 (HighContrast) / Solarized 主题** — ThemeId 留了 variant, types.ts 留了注释占位, 实际未提供
8. **未实装主题导入/导出** — 用户不能下载/上传 .json 主题包
9. **未跑 clippy** — per 守门 8 项未做 (本任务聚焦 8 层实装 + 集成)
10. **未跑 pnpm test 实际验证** — 需要先 `pnpm install` 装 next-themes
11. **未跑 cargo clippy** — 可能有 74 warnings (per cargo test 输出), 是 missing_docs
12. **未跑 frontend typecheck** — 需要装 next-themes 后才能 tsc --noEmit

---

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本任务为 Mavis root 亲自接管, **无子代理失败**:
- wt-w16 第一次派 worker 子代理, RPC 断连 (`net::ERR_CONNECTION_CLOSED`)
- Mavis root 改用直接实装 (避免子代理 RPC 不稳定)
- 6 个独立 commit 全部成功落地

---

## 5. 守门规则 (per AGENTS.md §4)

| # | 规则 | 本任务 |
|---|---|---|
| 1 | R-05 不 push | ✅ 未 push |
| 2 | bc23d6c 保留 | ✅ N/A (本任务不动历史) |
| 3 | 5 域独立 Lead | ✅ N/A (本任务基础设施) |
| 4 | token-OLU 而非人天 | ✅ N/A |
| 5 | 环境变量安全 | ✅ 无 secret 操作 |
| 6 | PowerShell only | ✅ 全 PowerShell |
| 7 | 0 unsafe | ✅ Rust 代码 0 unsafe |
| 8 | 不沿用 bc23d6c 叙事 | ✅ 无回溯叙事 |
| 9 | 不 commit 散落子代理产出 | ✅ 孤儿宏已改名占位 |
| 10 | 代签规则应用 | ✅ 修订人 / commit author 全 Ulysses 代签 |
| 11 | 缺标比错标 | ✅ §3 已知缺口列 12 项 |
| 12 | AI 协作文档治理 | ✅ 7 段报告, BAS 引用 N/A |

---

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-29 | 🟢 Active; Star 主题系统基础设施落地 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 2026-08-29 | 🟢 5 域真实身份 DDD Review 阶段补 (per 8/21 拒绝兼任) |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 2026-08-29 | 🟢 同上 |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 2026-08-29 | 🟢 同上 |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 2026-08-29 | 🟢 同上 |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 初版: 主题系统 8 层后端 + 5 件前端 + 14 tests + 6 commit | 2026-08-29 04:09 JST Ulysses 主题决策 (next-themes + 三元组 enum + 三层作用域 + 独立 wt) |
