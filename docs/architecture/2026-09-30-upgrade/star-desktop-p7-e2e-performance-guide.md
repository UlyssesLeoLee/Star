# Star Desktop — Tauri P7 Performance Baseline + E2E 验证指南

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #254 follow-up of PR #253 (P6 distribution)
> **status**: P7 完整 E2E 验证骨架 (per PR #244 plan + docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P7)

## 1. 目标

P7 = 完整 E2E 验证 5 services + Tauri desktop app:
1. **5 services verify**: canvas-game + canvas-engine + domain-canvas + canvas-realtime + star-api-rest → HTTP 200
2. **Tauri desktop app 集成**: Tauri webview 加载 BoardView → verify IPC list_work_items 走 MockDb
3. **Performance baseline**: 启动时间 / 内存 / CPU / WASM bundle size
4. **Screenshot capture**: BoardView 渲染 6 列 × W/T/M swimlane screenshot

## 2. 5 Services Verify

### 2.1 5 services 配置 (per session memory)

| Service | Local Port | Remote Port | Health Endpoint | Namespace |
|---|---|---|---|---|
| canvas-game | 12080 | 8084 | `/healthz` | star-system |
| canvas-engine | 13080 | 8080 | `/healthz` | star-system |
| domain-canvas | 14080 | 8081 | `/healthz` | star-system |
| canvas-realtime | 15080 | 8082 | `/healthz` | star-system |
| star-api-rest | 8081 | 8081 | `/api/v1/health` | star-system |

### 2.2 验证脚本 (per PR #254 NEW)

```bash
# scripts/verify-5-services.sh (NEW per PR #254)
# Usage:
bash scripts/verify-5-services.sh
# Output:
# === Step 1: Cleanup existing port-forwards ===
# === Step 2: Start port-forwards for 5 services ===
# === Step 3: Verify 5 services HTTP 200 ===
#   ✅ canvas-game: HTTP 200
#   ✅ canvas-engine: HTTP 200
#   ✅ domain-canvas: HTTP 200
#   ✅ canvas-realtime: HTTP 200
#   ✅ star-api-rest: HTTP 200
# === Summary ===
#   PASS: 5/5
#   FAIL: 0/5
```

### 2.3 WSL2 + k3s 死结恢复 (per session memory 反复实证)

session 内 retry 5+ 次都 hit WSL2 deadlock (per memory: cni0 interface DOWN + linkdown → pod ClusterIP 10.42.0.x unreachable). 唯一可靠 fix = host 端:

```powershell
wsl --shutdown
taskkill /F /IM wsl.exe /IM wslhost.exe 2>NUL
timeout /t 15
wsl -d Ubuntu
```

之后 session 内立即 verify 5 services HTTP 200.

## 3. Tauri Desktop App 集成验证

### 3.1 启动流程

```bash
# 1. 启动 5 services (verify-5-services.sh)
bash scripts/verify-5-services.sh --no-cleanup
# → 5 services HTTP 200 OK

# 2. 启动 Tauri desktop app
cd crates/star-desktop
cargo tauri dev
# → Tauri 2.0 webview 打开 + React 19 加载 + React Router → /worktree route

# 3. BoardView 集成验证
# - useTauriWorkItems hook → invoke('list_work_items') → 4 mock items
# - KanbanBoard 渲染 6 列 × W/T/M swimlane
# - useLayoutEngine pickAlgorithm → 取决于 viewMode + nodeCount
# - useQueryEngine parseKeywords → 6 keywords
```

### 3.2 预期 UI 行为

| 操作 | 预期响应 |
|---|---|
| 启动 app | Toolbar (select + input + button + meta) + KanbanBoard (6 列 × W/T/M) |
| 切换 viewMode | algo meta 改变 (dagre / d3_force / elk) |
| 输入 DSL filter "bug" | board-meta 显示 "1 items" (只显示 kind=bug) |
| 拖卡片从 todo → review | 工作项状态迁移 + 计数更新 |
| 点击 Refresh button | invoke('list_work_items') + 4 items 重新显示 |

## 4. Performance Baseline (per docs §3.1 数字)

### 4.1 启动时间 (cold start)

| 平台 | 启动时间 (估) |
|---|---|
| **Tauri desktop (macOS)** | ~50-80 ms (WebView + React 19) |
| **Tauri desktop (Linux)** | ~80-120 ms (libwebkit2gtk) |
| **Tauri desktop (Windows)** | ~100-150 ms (WebView2 cold) |
| **对比: Next.js browser** | ~800 ms (full page reload) |
| **加速倍数** | **8-16x faster** (per docs §3.1) |

### 4.2 稳态内存 (resident)

| 平台 | 内存 (估) |
|---|---|
| **Tauri desktop** | ~10-20 MB (WebView + React) |
| **对比: Next.js browser** | ~100-150 MB (Chrome tab) |
| **加速倍数** | **5-8x lower** (per docs §3.1) |

### 4.3 CPU usage (BFS algorithm, 100+ nodes)

| 操作 | CPU time |
|---|---|
| **Tauri desktop (WASM dagre)** | ~5-10 ms |
| **对比: JS BFS** | ~50-100 ms |
| **加速倍数** | **5-10x faster** (per docs §3.1) |

### 4.4 Bundle size

| 组件 | 大小 |
|---|---|
| **Tauri binary** | ~8 MB (Rust + WebView) |
| **Frontend bundle (Vite)** | ~200 KB (React 19 + Vite + components) |
| **WASM bundle** (layout-engine-wasm + query-engine-wasm + relationship-engine-wasm) | ~6 MB |
| **合计** | ~14 MB |
| **对比: Electron app** | ~150 MB |

## 6. Screenshot Capture (留 PR #254+)

### 6.1 期望 screenshots

| Screenshot | 内容 |
|---|---|
| **01-launch.png** | Tauri 启动后看到 Toolbar + KanbanBoard (mock 4 items) |
| **02-viewmode-tree.png** | viewMode=tree, algo=dagre |
| **03-viewmode-agent.png** | viewMode=agent, algo=d3_force (4 items <= 50) |
| **04-dsl-filter.png** | DSL "bug" → 1 items 显示 |
| **05-drop-zone.png** | 拖卡片从 todo → review, drop zone 高亮 |
| **06-wip-limit.png** | 6 items in_progress → "Over WIP limit" 警告 |

### 6.2 捕获工具

```bash
# macOS
screencapture -W -o docs/screenshots/01-launch.png

# Linux
gnome-screenshot -f docs/screenshots/01-launch.png

# Windows (PowerShell)
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Screen]::PrimaryScreen.Bounds
# 然后用 PrintScreen + paste 到 Paint
```

## 7. 守门

| 守门 | 状态 |
|---|---|
| **#7 unsafe_code = "forbid"** | ✅ Tauri binary 0 unsafe (workspace lint) |
| **#19 0 动 V0.1 业务 logic** | ✅ 脚本仅验证, 不改源码 |
| **#11 缺标比错标** | ✅ 5 services 列与 session memory 严格同步 |

## 8. 不在 PR-254 范围 (留后续)

- ❌ 实际跑 verify-5-services.sh (需 WSL2 + k3s 恢复 per session memory)
- ❌ 实际跑 cargo tauri dev (需 Linux build host per PR #242 + verify 5 services)
- ❌ Screenshot 捕获 (需 manual run)
- ❌ Performance baseline 实测数字 (需 Linux build + Tauri 实际启动)

## 9. 后续 PR 计划

| PR | 内容 | 估 |
|---|---|---|
| **PR-255** | Linux CI matrix 升级: multi-OS build verification (ubuntu + windows + macos) | 1 周 |
| **PR-256** | Tauri e2e 截图 + 性能 baseline 实测 | 1 周 (需 Linux build + Tauri dev) |
| **PR-257** | Code signing + 公证 + auto-updater | 2 周 (需 EV cert + Apple Developer ID) |

## 10. Refs

- [scripts/verify-5-services.sh](../scripts/verify-5-services.sh) (NEW PR #254)
- [crates/star-desktop/README.md](../crates/star-desktop/README.md) (PR #239 P0)
- [crates/star-desktop/src/components/BoardView.tsx](../crates/star-desktop/src/components/BoardView.tsx) (PR #252 P5-5)
- [.github/workflows/star-desktop-build.yml](../.github/workflows/star-desktop-build.yml) (PR #242 Linux CI)
- [scripts/build-desktop.sh](../scripts/build-desktop.sh) (PR #253 P6)
- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.1 + §3.2 P7](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- [docs/architecture/2026-09-30-upgrade/star-desktop-p6-distribution-guide.md](../2026-09-30-upgrade/star-desktop-p6-distribution-guide.md) (PR #253)
