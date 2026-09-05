# STAR I18N 全量接管 brief

> **Task ID**: STAR-I18N-TAKEOVER
> **拍板**: 2026-09-05 23:09 JST 用户拍板 C 完整 i18n 接管 + nav 全 i18n + 缺标 warn 兜底
> **落档**: docs/briefs/STAR-I18N-TAKEOVER.md
> **范围**: `D:\Star\frontend` (Next.js 14 App Router) i18n 全量接管
> **代签**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手

## §1 目标

STAR frontend 现在 i18n 3 语言 (zh-CN/en/ja) 但**选中文模式还显示英文**。根因 4 类：

1. **字典 key 没接组件**: 29 个 page.tsx + 多个核心组件直接硬编码 `title="Settings"` / `aria-label="Notifications"` 等
2. **字典里直接写英文**: zh-CN.ts `modules.settings.label = "Settings"` (跟 en 完全一致, 没翻译)
3. **nav/registry.ts 硬编码**: 33 个 module `label` 全部英文
4. **i18n shape 缺域**: 现有 dictionary 没有 `pageTitles` / `navModules` / `ariaLabels` namespace

## §2 范围 (in-scope)

- `frontend/src/lib/i18n/{config,dictionary,zh-CN,en,ja,useTranslation,useModuleTranslation}.ts(x)`
- `frontend/src/lib/nav/registry.ts` (33 module label 改 i18n key)
- `frontend/src/app/**/page.tsx` (~30 个)
- `frontend/src/components/{AppHeader,Sidebar,MobileHeader,MobileBottomNav,CommandBar,PageHeader,UserMenu,AppShell,PanelPlaceholder,SubNav,Tabs}.tsx`
- 13 个 chart 组件 (Chart01..Chart15)
- refactor 7 组件
- board/calendar/gantt 组件
- `scripts/automation/i18n_audit.py` (新, 守门 #19)

## §3 范围外 (out-of-scope)

- docs/*.md 翻译 (项目文档不在 i18n 范围)
- mocks/ 目录 mock 数据 (mock 数据本身是英文)
- __tests__ 测试 fixture (测试期望)
- Rust 后端 (前端活)

## §4 字典扩展设计 (5 域横展, 跟守门 #13 W/T/M 同 shape)

dictionary.ts 新增 5 域 (取代零散硬编码):

```ts
interface Dictionary {
  // 已有 ...
  navModules: Record<string, {
    label: string;       // 模块主名 (Inbox → 收件箱)
    categoryLabel: string; // 域标签
    description: string;   // 模块描述
  }>;
  pageTitles: Record<string, {
    title: string;
    subtitle: string;
  }>;
  ariaLabels: Record<string, string>;   // aria-label 兜底
  placeholders: Record<string, string>; // input placeholder 兜底
  // 5 域类别
  categoryNames: Record<string, string>; // core/work/agent/integration/system
}
```

## §5 缺标兜底 (per 拍板 missing_opt1)

`useTranslation` 加 fallback helper:

```ts
// 开发环境: console.warn + 渲染 key 路径
// 生产环境: 静默 fallback en
function txOrWarn(key: string, value: string | undefined): string {
  if (value) return value;
  if (process.env.NODE_ENV !== 'production') {
    console.warn(`[i18n] missing key: ${key}`);
  }
  return `[${key}]`;  // 显式标记, 不静默
}
```

## §6 守门 (5 维 per STAR-OLU-001 §6)

1. `pnpm typecheck` 0 err
2. `pnpm test` 全过 (vitest, 守住既有用例)
3. `pnpm build` 0 err
4. `python scripts/automation/i18n_audit.py` 输出硬编码 = 0
5. git 实证: commit message 含 brief 路径

## §7 交付物

- 5 份字典补全 (zh-CN/en/ja 完整对齐)
- 33+ 组件改造
- `scripts/automation/i18n_audit.py` 落档
- `docs/reports/STAR-I18N-TAKEOVER-REPORT.md` 7 段结构报告
- git 实证 commits

## §8 预估消耗

~2.0-3.0M tokens (per 用户拍板 C 选项)

## §9 排期

| Step | 内容 | 预估 |
|---|---|---|
| 1 | 扫硬编码 + 落审计 baseline | 0.2M |
| 2 | 字典扩展 + 5 域横展 | 0.4M |
| 3 | nav registry 全 i18n | 0.3M |
| 4 | page.tsx 30 个批量改 | 0.6M |
| 5 | 核心组件硬编码清理 | 0.4M |
| 6 | chart/refactor/board/calendar 收尾 | 0.4M |
| 7 | 守门 5 维 + 报告 + commit | 0.3M |
| **Total** | | **~2.6M** |
