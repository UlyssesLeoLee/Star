# PHASE-MOBILE-PWA-V0.4-REPORT

> **文档版本**: v0.4 (2026-09-01 13:52 JST)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批者**: 架构师 (Mavis 接手 agent per DEC-008)
> **触发**: 2026-09-01 13:38 JST Ulysses 拍板 (A) 推 v0.4 (Web Push + Playwright e2e)
> **基线**: v0.3 commit `a8e231f` (PHASE-MOBILE-PWA-V0.3-REPORT.md)

---

## §0 目的

v0.3 已落地 PWA 完整 UX (SW 更新 / install prompt / noVNC 缩放)。**v0.4 推最后两项依赖拍板项**: Web Push 集成 + Playwright mobile viewport e2e。期间修复关键 mobile layout bug (z-index stacking + inbox 列表撑出 Bottom Nav 区域)。

---

## §1 改动矩阵

| # | 项 | 文件 | 行数 | 状态 |
|---|---|---|---|---|
| 1.1 | Web Push 客户端契约 (Notification API + pushManager.subscribe + VAPID placeholder) | `frontend/src/lib/pushClient.ts` | 125 | 🟢 完成 |
| 1.2 | SW push event handler (显示通知 + 点击聚焦) + SIMULATE_PUSH 消息支持 | `frontend/public/sw.js` | +60 | 🟢 完成 |
| 1.3 | PushSettings 组件 (启用/关闭/测试按钮 + 权限状态) | `frontend/src/components/remote/PushSettings.tsx` | 122 | 🟢 完成 |
| 1.4 | /remote home 整合 PushSettings (顶部) | `frontend/src/app/remote/page.tsx` | +5 | 🟢 完成 |
| 1.5 | Playwright mobile e2e (iPhone 13 viewport, 8 spec) | `frontend/e2e/remote-mobile.spec.ts` | 142 | 🟢 8/8 pass |
| 1.6 | vitest.config.ts 排除 Playwright-only spec | `frontend/vitest.config.ts` | +2 | 🟢 完成 |
| 1.7 | **Bug 修复**: inbox-item 撑出 Bottom Nav 区域 + z-index stacking 战争 | `frontend/src/app/layout.tsx` + `MobileBottomNav.tsx` + `inbox/page.tsx` | +20 | 🟢 完成 |
| 1.8 | **Bug 修复**: 动态路由 params 类型 Next.js 14 vs 15 不兼容 | `frontend/src/app/remote/{desktop,terminal,files}/[id]/page.tsx` | -3 | 🟢 完成 |
| 1.9 | **Bug 修复**: isRemoteMockMode 默认 false 导致 WebSocket fail + file list 不渲染 | `frontend/src/lib/remote/wsClient.ts` | +8 | 🟢 完成 |
| 1.10 | PwaInstallPrompt / PwaUpdateToast z-index 提到 9998/9997, 避免被 inbox 等覆盖 | `PwaInstallPrompt.tsx` + `PwaUpdateToast.tsx` | +6 | 🟢 完成 |

**合计**: 5 new + 6 modified = 11 files, +约 500 行

---

## §2 验证摘要

### §2.1 守门 #1 v1 (tsc --noEmit)

```bash
$ cd frontend && npx tsc --noEmit
# exit 0, 0 err
```

✅ **TS-OK** (0 type error, 含 ESM-only ambient module 声明)

### §2.2 守门 #1 v2 (vitest)

```bash
$ cd frontend && npx vitest run
# Test Files: 38 passed (38)
# Tests:      309 passed (309)
# Duration:   6.39s
```

✅ **309/309 tests pass** (Playwright spec 排除在 vitest, 38 个 vitest file 干净)

### §2.3 守门 #1 v3 (next build)

```bash
$ cd frontend && npm run build
# ✓ Compiled successfully
# ✓ Generating static pages (41/41)
# First Load JS shared by all: 87.2 kB
```

✅ **Build 成功, 41 静态页, 87.2 kB shared JS, 0 err**

### §2.4 守门 #1 v4 (Playwright e2e — v0.4 新增)

```bash
$ cd frontend && npx playwright test e2e/remote-mobile.spec.ts --project=chromium
# 8 passed (18.2s)
```

✅ **8/8 Playwright spec pass**:
- 移动端布局 (Sidebar 隐藏 + MobileHeader + BottomNav 5 项)
- "更多" 抽屉含 Remote 入口
- /remote home (5 runtime + Push 设置)
- /remote/desktop/[id] (noVNC 容器)
- /remote/terminal/[id] (xterm 容器)
- /remote/files/[id] (SFTP + 5 层 mock 文件树)
- Bottom Nav 切 Worktree (per legacy redirect → /issues?view=tree)
- iOS install prompt 3 步说明

### §2.5 关键 Bug 修复路径

| # | Bug | 根因 | 修复 |
|---|---|---|---|
| B1 | Playwright click Bottom Nav 被 inbox-item-n-008 拦截 | inbox 列表 10 li × 50px = 500px, vh=844 撑到 y=804-863, 与 Bottom Nav 区域 (787-844) 重叠 | (a) main 容器加 height: 100dvh + overflowY: auto, 内容自滚 (b) BottomNav z-9999 inline style (c) layout padding-bottom: 4rem 让出空间 |
| B2 | `use(params)` Next.js 15 语法, 在 14.2.5 报 "params is not a Promise" | Next.js 14 params 是普通对象 | 改回 `const { id } = params;` |
| B3 | WebSocket 试图连后端 /v1/remote/* 不存在, 5s timeout 后 setStatus("error") | isRemoteMockMode 默认 false, 但 MVP 阶段后端没就位 | 加 NEXT_PUBLIC_REMOTE_LIVE=1 opt-in 显式切换, 默认 mock |
| B4 | vitest 误把 e2e/*.spec.ts 当 vitest 跑 | vitest.config include 'e2e/**' | 加 exclude 列表 |
| B5 | PwaInstallPrompt 用了 z-40 className 又 style zIndex, TS duplicate key | 编辑时合并 | 合并到单一 style 块 |

---

## §3 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| G1 | **WebSocket 后端 relay** 未实装 | real-mode 下 noVNC/xterm/SFTP 无法连远端, 只能看 mock (per `isRemoteMockMode()` 默认 true) | 后端 BFF 加 `/v1/remote/{kind}/{id}` WS endpoint + VNC server + shell exec + SFTP subsystem |
| G2 | **VAPID key 未生成** | push subscription 走 fallback (Notification API only) | 后端用 `web-push` 库生成 VAPID key pair, 公开 public key → `NEXT_PUBLIC_VAPID_PUBLIC_KEY` |
| G3 | **xterm 高级功能** (history 翻页 / Tab 补全) 简化 | 简单 6 命令 + 软键盘 | v1.0 接 Star agent runtime exec |
| G4 | **SFTP upload/download** 入口占位 disabled | 需后端 WS 协议扩展 (write_req) | v0.5+ |
| G5 | **iOS Safari 真机测** 未跑 | iOS PWA 后台 < 30s 被挂起 + 通知权限弹 | 真机测, v0.5+ |
| G6 | **PWA install icon maskable** 未优化 | 部分 Android launcher 圆角裁切 | 改 SVG 矢量 |
| G7 | **SW 缓存策略优化** (现在 static cache 太泛) | 升级后老 cache 仍占空间 | cache versioning + LRU 限制 |
| G8 | **Playwright CI 集成** | spec 已就位, 但 webServer.command `npm run dev` 慢 + 真 CI 需 build + serve | 后续: 用 `next start` + 已 build 的 .next, 或换 @playwright/test-expect |

---

## §4 子代理失败接手清单

**v0.4 全程 root 直实装, 0 子代理调用** (per 守门 #9)

---

## §5 守门规则 (15-17 项)

| # | 规则 | 实证 | 状态 |
|---|---|---|---|
| 1 | R-05 不 push (反转已落) | 守门 #1 反转 2026-08-30 07:09 JST | 🟡 待 push |
| 2 | bc23d6c 保留 | 未触碰 | 🟢 |
| 3 | 5 域独立 Lead | 未触碰 | 🟢 |
| 4 | AI token-OLU | 估算 +0.6M 实装 (vs 0.5M 预算, 含 Playwright e2e 调试) | 🟢 略超预算 (+0.1M) |
| 5 | 环境变量安全 | NEXT_PUBLIC_VAPID_PUBLIC_KEY / NEXT_PUBLIC_REMOTE_LIVE 引用未展开 | 🟢 |
| 6 | PowerShell only | 全 PowerShell 调用 | 🟢 |
| 7 | 0 unsafe | 无 unsafe 代码 | 🟢 |
| 8 | 不沿用 bc23d6c 叙事 | 新增 5 new file + 6 modified | 🟢 |
| 9 | 不 commit 散落子代理产出 | 0 子代理调用, root 直实装 | 🟢 |
| 10 | 代签规则应用 | commit author = Ulysses / 修订人 Mavis 接手 | 🟢 |
| 11 | 缺标比错标安全 | §3 显式列 8 项已知缺口 | 🟢 |
| 12 | AI 协作文档治理 | 本报告 7 段 + 引用 commit hash 待生成 | 🟡 |
| 1v | tsc --noEmit 0 err | exit 0, 0 err | 🟢 |
| 2v | vitest 309/309 pass | 38 file 309 test 全过 | 🟢 |
| 3v | next build 0 err | 41 静态页全生成, 87.2 kB shared JS, 0 err | 🟢 |
| 4v | workspace 多 crate | 不适用 (前端单仓) | 🟢 |
| 5v | Playwright e2e 8/8 pass (v0.4 新增) | iPhone 13 viewport, 8 spec 全过 | 🟢 |
| 6v | 守门 #12 文档治理 (7 段) | 本报告覆盖 7 段 (0-6) | 🟢 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 1 | 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 13:52 JST | Mavis 接手代签 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 13:52 JST | 5 域独立真实身份待 DDD Review 阶段补 |
| 3 | 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 13:52 JST | 同上 |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 13:52 JST | 同上 |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 13:52 JST | 同上 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.4 | 2026-09-01 13:52 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Web Push 集成 + Playwright e2e (8/8 pass) + 5 项关键 bug 修复 (z-index stacking, params 类型, mock 默认值, vitest exclude, TS duplicate key) | 2026-09-01 13:38 JST Ulysses 拍板 (A) |
| (v0.3) | 2026-09-01 13:04 JST | 同上 | SW 更新 toast + PWA install prompt + noVNC 缩放, commit `a8e231f` | 13:02 JST 拍板 "推进" |
| (v0.2) | 2026-09-01 12:32 JST | 同上 | 远程三件套, commit `5730caa` | 12:20 JST 拍板 all-three |
| (v0.1) | 2026-09-01 12:21 JST | 同上 | PWA 基础, commit `ab275be` | 12:05 JST 拍板 PWA 起步 |

---

## §8 后续路线

### v0.5+ (续做, 不依赖 Ulysses 拍板)

1. **SFTP upload/download** 协议实装
2. **iOS Safari 真机测** + 通知权限
3. **PWA install icon maskable** SVG
4. **SW 缓存 LRU 限制** + cache versioning
5. **Playwright CI 集成** (webServer 改 next start)

### v1.0 Flutter 阶段 (PWA 验证后)

1. Flutter 3.24+ 跨端 App
2. Codemagic CI (云端 Mac, 无 Mac 也可出 App Store)
3. Physis 物理引擎 FFI 通道
4. 后端实装真实 WS relay (noVNC RFB + xterm shell exec + SFTP JSON 协议)
5. VAPID key 接入 + 真推送
