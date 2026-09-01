# PHASE-MOBILE-PWA-IMPL-REPORT

> **文档版本**: v0.1 (2026-09-01 12:21 JST)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批者**: 架构师 (Mavis 接手 agent per DEC-008)
> **触发**: 2026-09-01 12:05 JST Ulysses 拍板 "按照你推荐推进吧" (B 起步 → 验证场景 → A 接上) + 12:14 JST 补充 "远程连接桌面等核心功能"
> **关联**: `docs/frontend/design/ui-3pane-arch.md` §1.1 (<768px 走 PWA) + `docs/frontend-design.md` §776-777 (PWA 离线)
> **下游**: PWA MVP 验证后接 Flutter 阶段 (Codemagic CI) — 见 §6 后续路线

---

## §0 目的

按 Ulysses 12:05 JST 拍板 "PWA 起步 → 验证 → Flutter 接上" 路线,先在 web 栈落地 5 域快查 + 离线 read + 移动端布局,为后续 Flutter 阶段验证场景。**v0.1 范围限于 PWA 基础实装,远程三件套(noVNC 远程桌面 / xterm.js 远程终端 / 远程文件)待 §3 缺口项拍板后 v0.2 续做**。

---

## §1 改动矩阵

| # | 项 | 文件 | 行数 | 状态 |
|---|---|---|---|---|
| 1.1 | manifest.json 升级 (start_url 修复 + 5 shortcuts + icon-192 + scope) | `frontend/public/manifest.json` | 64 | 🟢 完成 |
| 1.2 | icon-192.png 生成 (PIL LANCZOS 缩放 512x512) | `frontend/public/icon-192.png` | (binary) | 🟢 完成 |
| 1.3 | Service Worker (precache + network-first nav + SWR 5 域) | `frontend/public/sw.js` | 124 | 🟢 完成 |
| 1.4 | SW 注册 hook (production only) | `frontend/src/lib/pwaRegister.ts` | 41 | 🟢 完成 |
| 1.5 | PWA 引导组件 (注册 SW + 监听更新) | `frontend/src/components/PwaBoot.tsx` | 11 | 🟢 完成 |
| 1.6 | 移动端顶栏 (汉堡 + 标题 + 搜索 + 通知) | `frontend/src/components/MobileHeader.tsx` | 88 | 🟢 完成 |
| 1.7 | 移动端底部 5 域快捷导航 (含"更多"抽屉) | `frontend/src/components/MobileBottomNav.tsx` | 144 | 🟢 完成 |
| 1.8 | 离线 fallback 页面 (SW 触发) | `frontend/src/app/offline/page.tsx` | 50 | 🟢 完成 |
| 1.9 | next.config.js sw.js Service-Worker-Allowed header | `frontend/next.config.js` | +11 | 🟢 完成 |
| 1.10 | RootLayout 整合 (MobileHeader + MobileBottomNav + PwaBoot + viewport export) | `frontend/src/app/layout.tsx` | +30 | 🟢 完成 |
| 1.11 | Sidebar <768px 隐藏 (md:flex) | `frontend/src/components/Sidebar.tsx` | +2 | 🟢 完成 |

**合计**: 7 new file + 4 modified = 11 files,+约 500 行

---

## §2 验证摘要

### §2.1 守门 #1 验证 (v1: cargo check 类 → tsc --noEmit)

```bash
$ cd frontend && npx tsc --noEmit
# exit 0, 0 err
```

✅ **TS-OK** (0 type error)

### §2.2 守门 #1 验证 (cargo test 类 → vitest)

```bash
$ cd frontend && npx vitest run
# Test Files: 38 passed (38)
# Tests:      309 passed (309)
# Duration:   6.53s
```

✅ **309/309 tests pass** (新增 0 test 现有 309 全保留)

### §2.3 守门 #1 验证 (cargo build 类 → next build)

```bash
$ cd frontend && npm run build
# ✓ Compiled successfully
# ✓ Generating static pages (40/40)
# Route (app)              Size     First Load JS
# + First Load JS shared    87.1 kB
```

✅ **Build 成功,40 静态页全生成,0 错误**

### §2.4 设计 token 一致性

- Star 调色板 (`--color-bg`, `--color-line`, `--color-ink`, `--color-accent`, `--color-err`) 100% 复用 globals.css
- 5 字号 / 3 圆角 / 8 间距 全部走 Tailwind utility,无发明
- 12 条认知负荷规则未触碰

### §2.5 响应式规则

| 视口 | 显示 | 验证 |
|---|---|---|
| ≥768px | Sidebar (w-64) + AppHeader + 无 MobileHeader / MobileBottomNav | ✅ md:flex / md:hidden 类控制 |
| <768px | MobileHeader (sticky top) + 单栏 main + MobileBottomNav (fixed bottom) + Sidebar 隐藏 | ✅ md:hidden 控制 |

---

## §3 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| G1 | **远程控制三件套 (noVNC / xterm.js / SFTP)** 未实装 | 路上"接手 agent 跑任务"硬场景未覆盖 | v0.2 续,等 Ulysses 拍板范围 + 连接模式 |
| G2 | **Web Push 集成** 未实装 | iOS 16.4+ / Android Chrome 可用,需 VAPID key | v0.2 续,等 Star BFF 暴露 push 端点 |
| G3 | **SW 更新提示 UI** 未实装 | 新 SW 安装后用户无感知 | v0.2 续,加 toast + "刷新可用"按钮 |
| G4 | **iOS Safari 后台限制** 未测试 | iOS PWA 后台 < 30s 会被挂起,通知延迟 | v0.2 续,在真机测一遍 |
| G5 | **Service Worker 缓存命中率** 未监控 | 无 metrics,不知离线 read 效果 | 加 `navigator.serviceWorker.controller.postMessage` 采样,接 star-sse |
| G6 | **PWA 安装提示 (beforeinstallprompt)** 未触发 | Android Chrome 加 install banner 未实装 | v0.2 续 |
| G7 | **icon-192 矢量级** 当前是 PIL 缩 512 PNG | 4MB 解码延迟,大屏模糊 | 改 SVG,小尺寸自动渲染 |

---

## §4 子代理失败接手清单

**v0.1 全程 root 直实装,0 子代理调用** (per 守门 #9: 子代理 RPC 不可靠实证 P3-A.6/A.7)

---

## §5 守门规则 (15-17 项)

| # | 规则 | 实证 | 状态 |
|---|---|---|---|
| 1 | R-05 不 push (反转已落) | 守门 #1 反转 2026-08-30 07:09 JST | 🟡 待 push |
| 2 | bc23d6c 保留 | 未触碰 | 🟢 |
| 3 | 5 域独立 Lead | 未触碰 | 🟢 |
| 4 | AI token-OLU | 估算 0.4M 实装 (vs 0.4M 预算) | 🟢 在预算内 |
| 5 | 环境变量安全 | 无 env 引用 | 🟢 |
| 6 | PowerShell only | 全 PowerShell 调用 | 🟢 |
| 7 | 0 unsafe | 无 unsafe 代码 | 🟢 |
| 8 | 不沿用 bc23d6c 叙事 | 新增 7 new file,不引旧 | 🟢 |
| 9 | 不 commit 散落子代理产出 | 0 子代理调用,root 直实装 | 🟢 |
| 10 | 代签规则应用 | commit author = Ulysses / 修订人 Mavis 接手 | 🟢 |
| 11 | 缺标比错标安全 | §3 显式列 7 项已知缺口 | 🟢 |
| 12 | AI 协作文档治理 | 本报告 7 段 + 引用 commit hash 待生成 | 🟡 |
| 1v | cargo check 类 → tsc --noEmit 0 err | exit 0, 0 err | 🟢 |
| 2v | cargo test 类 → vitest 309/309 pass | 38 file 309 test 全过 | 🟢 |
| 3v | cargo build 类 → next build 0 err | 40 静态页全生成,87.1 kB shared JS | 🟢 |
| 4v | workspace 多 crate | 不适用 (前端单仓) | 🟢 |
| 5v | 守门 #12 文档治理 (7 段) | 本报告覆盖 7 段 (0-6) | 🟢 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 1 | 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 12:21 JST | Mavis 接手代签 (per 19:39/20:56/21:59 JST 三次强化) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 12:21 JST | 5 域独立真实身份待 DDD Review 阶段补 (per 8/21 JST 拒绝兼任) |
| 3 | 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 12:21 JST | 同上 |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 12:21 JST | 同上 |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 12:21 JST | 同上 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 12:21 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版:PWA 基础实装 (manifest + SW + 移动端布局 + offline 页面 + Bottom Nav) | 2026-09-01 12:05 JST Ulysses 拍板 PWA 起步 + 12:14 JST 补充远程控制是硬场景 + 守门 #1+#9+#12 三过 |

---

## §8 后续路线 (per 守门 #12 死循环饱和边界 + 用户拍板)

### v0.2 续做 (待 Ulysses 拍板)

1. **远程控制三件套** (noVNC + xterm.js + SFTP) — OLU +1.5-2M
2. **Web Push** 集成 (VAPID + star-sse 通道) — OLU +0.4M
3. **SW 更新提示** UI (toast + 刷新按钮)
4. **iOS Safari 真机测** 离线 / 后台 / 安装 banner
5. **PWA 安装 prompt** (beforeinstallprompt) Android Chrome

### v1.0 Flutter 阶段 (PWA 验证后)

1. Flutter 3.24+ 跨端 App (per 12:05 JST 推荐路线 A)
2. Codemagic CI (云端 Mac,无 Mac 也可出 App Store)
3. 复用 BFF API + 设计 token (Star Indigo / 5 字号 / 12 认知规则结构化迁移)
4. Physis 物理引擎 FFI 通道 (Flutter 相对 RN 隐性优势)
