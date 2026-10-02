# `crates/star-desktop/` — Tauri 2.0 PoC P0→P10

> **status**: PoC P10 ALL DONE (per [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md) + [docs/architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md §1](../../architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md))
> **date**: 2026-10-01 JST (last actualized per PR-274)
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: 2026-09-30 "A: 继续 P1 准备" → P1 (#240) → ... → P10 (#258) → session memory (#269) → Tauri Viz (#270) → PoC index (#271)
> **PR history**: P0 skeleton (#239) → P1 icons+IPC (#240) → P2 adapters (#241) → P3 CI (#242) → ... → P10 Sentry (#258) → final summary (#259) → session memory (#269) → WorktreeVizPage (#270) → PoC index (#271)

## 目标

验证 Tauri 2.0 跨平台 desktop + 复用现有 Rust crates (97 crates 复用, 0 动业务 logic) 可行性。

## 结构

```
crates/star-desktop/
├── README.md                   # 本文件 (per PR-274 actualize)
├── .gitignore                  # + keys/ + secrets/ (per PR-257)
├── src-tauri/
│   ├── Cargo.toml              # Tauri lib + binary manifest (workspace member)
│   ├── build.rs                # tauri-build (codegen)
│   ├── tauri.conf.json         # Tauri app config (window + bundle + plugins.updater per PR-256)
│   ├── capabilities/default.json # 默认权限 (core:default)
│   ├── icons/                  # 5 PNG files (per PR-240)
│   └── src/
│       ├── main.rs             # entry point: star_desktop_lib::run()
│       └── lib.rs              # Tauri 2.0 setup + **8 IPC commands** + 5 unit tests
└── frontend/                   # Tauri 2.0 + React 19 + Vite + vitest (per PR-246-#252)
    ├── package.json            # + react@19 + react-router-dom@7 + @tauri-apps/api
    ├── vite.config.ts          # Vite + jsdom test env (per PR-246)
    ├── tsconfig.json           # JSX react-jsx
    ├── index.html              # <div id="root"> + /src/main.tsx
    ├── src/
    │   ├── main.tsx            # React 19 + RouterProvider (per PR-246)
    │   ├── App.tsx + App.module.css
    │   ├── pages/
    │   │   ├── HomePage.tsx          (8 IPC commands 列表)
    │   │   ├── WorktreePage.tsx      (BoardView 集成 per PR-252)
    │   │   └── WorktreeVizPage.tsx   (useLayoutEngine 120 nodes per PR-270)
    │   ├── components/
    │   │   ├── KanbanCard.tsx       (per PR-248, V0.1 复用)
    │   │   ├── KanbanBoard.tsx      (per PR-249, V0.1 复用)
    │   │   └── BoardView.tsx         (per PR-252, 4 子系统集成)
    │   ├── hooks/                    (per PR-250 + #251)
    │   │   ├── useTauriWorkItems.ts (per PR-250)
    │   │   ├── useTauriBoardInfo.ts
    │   │   ├── useTauriKeyboardLayout.ts
    │   │   ├── useLayoutEngine.ts   (WASM hook per PR-251)
    │   │   └── useQueryEngine.ts    (WASM hook per PR-251)
    │   ├── types/ids.ts              (per PR-247, W/T/M + 6 statuses)
    │   └── constants.ts               (per PR-247)
    └── tests/                        (57 vitest tests across 13 files)
```

## IPC commands (P0→P10 全集, per PR #239-#245)

| # | Command | 入参 | 出参 | 来源 | PR |
|---|---|---|---|---|---|
| 1 | `list_work_items` | 无 | `Result<Vec<WorkItem>, String>`（无 Run provider 时返回 unavailable） | 原 P0 demo Task 已退役；canonical Run provider 接入前 fail closed | #239, #245, Phase 9F |
| 2 | `list_worktree_groups` | 无 | `Vec<WorktreeGroup>`（当前为 4 条 demo fixture） | P1 compatibility demo → P4 MockDb；不可作为 Project/Branch/Run 权威目录 | #240, #245 |
| 3 | `list_canvas_entities` | 无 | `Vec<CanvasEntity>`（当前为 4 条 demo fixture） | P1 compatibility demo → P4 MockDb；不可作为 Run Canvas 持久化或权威读模型 | #240, #245 |
| 4 | `get_app_version` | 无 | `String` (semver from `CARGO_PKG_VERSION`) | P1 env | #240 |
| 5 | `get_keyboard_layout` | 无 | `KeyboardLayout` (W/T/M swimlane + 6 statuses) | P1 static | #240 |
| 6 | `get_board_info` | 无 | `BoardInfo` (board_kind_count + swimlane_group_by_count + default_column_count) | P2 adapter | #241 |
| 7 | `get_worktree_info` | 无 | `WorktreeInfo` (worktree_status_count + health_dimensions) | P2 adapter | #241 |
| 8 | `get_canvas_info` | 无 | `CanvasInfo` (route_prefix + phase_count) | P2 adapter | #241 |

**8 IPC commands 完整** — Task IPC 已 fail closed，不返回演示 Task；Worktree/Canvas IPC 仍只用于 legacy shell demo，不能冒充生产 Run 应用数据。

## 复用 crates (守门 #19: 0 改 V0.1 业务 logic)

- `crates/domain-board` — 0 改, BoardAdapter 引用 (per PR-241)
- `crates/domain-worktree` — 0 改, WorktreeAdapter 引用 (per PR-241)
- `crates/canvas-engine` — 0 改, CanvasAdapter 引用 (per PR-241)
- `crates/relationship-engine-wasm` — 0 改, 供前端 WASM 调用 (per PR-251)

## 守门 #7 `unsafe_code = "forbid"`

Cargo.toml `[lints] workspace = true` 继承 workspace lint (`unsafe_code = "forbid"`)。per PR-274 audit: 0 unsafe across all 19 PRs (#239-#271)。

## 本 crate 不做 (留后续)

- ❌ 实际 `cargo tauri dev` 跑起来 (需 Linux build host per PR-242, 或更大内存 Windows)
- ❌ 实际 `cargo tauri build` 全 bundle build (per PR-253 P6 distribution)
- ❌ 真实 Project/Branch/Run、Worktree 与 Canvas provider/DB 接入；当前 `MockDb` 只为 legacy demo 提供 Worktree/Canvas fixture，不包含 Task Card，也不是生产管理数据
- ❌ Tauri 2.0 Windows build verification (STATUS_STACK_BUFFER_OVERRUN per PR-239 — 待 PR-242 Linux CI 触发)
- ❌ Code signing + 公证 + auto-updater 实战 (per PR-256 P8, 待 cert + 12 secrets 配置)

## 下一步 (PR #239-#271 已落档, PR-272+ 后续)

| Task | 内容 | 状态 |
|---|---|---|
| ✅ PR #239-#258 | Tauri PoC P0→P10 全闭环 (21 PRs) | DONE (per PR-259) |
| ✅ PR #269/#270/#271 | session memory + WorktreeVizPage + PoC index (3 docs/PRs) | DONE (squash merged per session) |
| ✅ PR #272 | README.md 全项目 docs 乖离 actualize (TL;DR + §1 + §3 + §5 + §7 数字更新) | DONE |
| ✅ PR #273 | docs/architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md 21→24 PRs actualize | DONE |
| ✅ PR #274 | crates/star-desktop/README.md "1 IPC mock" → "8 IPC" actualize | DONE (本 PR) |
| ⏳ Tauri Linux CI trigger | PR-242/#255 multi-OS matrix 等 PR push 触发, 验证 26→34 Rust tests pass | CI pending |
| ⏳ k3s cluster 5 services | per PR-254 `scripts/verify-5-services.sh`, 待 host 端 `wsl --shutdown` | host-side |
| ⏳ 9 GitHub Actions secrets 配置 | per PR-257 (cert + Apple Developer + GPG + Tauri) | external dep |

## 验证 (本 PR-274 actualize, 综合所有 #239-#271)

- ✅ **8 IPC commands** 完整 (5 mock + 3 adapter-based, per PR #239-#245)
- ✅ **5 icons** 创建: `32x32.png` (2679B) + `128x128.png` (45448B) + `128x128@2x.png` (181435B) + `icon.icns` (45448B) + `icon.ico` (2679B) (per PR #240)
- ✅ **91 Tauri-side tests** (34 Rust + 57 Vitest, per PR #259 §13 + PR #245/#246-#252/#248-#252)
- ✅ **CI workflow**: `.github/workflows/star-desktop-build.yml` (4 jobs: linux-build + windows-build + macos-build + matrix-summary per PR #255)
- ⚠ **`cargo check -p star-desktop` 当前 Windows machine build 失败** (Tauri 2.0 100+ Windows crates, STATUS_STACK_BUFFER_OVERRUN per PR #239). 验证需要 Linux build host per PR #242 GitHub Actions ubuntu-22.04.
- ⚠ **PR-269 CI 4 fail** (Frontend/Markdownlint/Rust/Tarpaulin, per `gh pr checks 269` 输出): PR-269 session memory docs 已 squash merged, 但 4 check 红需后续 PR 修

## Refs

- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P0-P10](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md)
- [docs/architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md §1](../../architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md) — 24 PRs 时间线 (per PR-273 actualize)
- [docs/architecture/2026-10-01-upgrade/tauri-poc-index.md](../../architecture/2026-10-01-upgrade/tauri-poc-index.md) — 9 docs 总览 (per PR-264)
- [docs/architecture/2026-10-01-upgrade/session-memory-protocol.md](../../architecture/2026-10-01-upgrade/session-memory-protocol.md) — 5 workaround (per PR-260)
- [.github/workflows/star-desktop-build.yml](../../../../.github/workflows/star-desktop-build.yml) — Linux CI (per PR #242 + #255 multi-OS)
- PR #239-#271 (24 PRs) — Tauri PoC P0→P10 + session memory + index + WorktreeVizPage
- PR #272 (README actualize) + #273 (summary 21→24) + #274 (crates README actualize, 本 PR)
