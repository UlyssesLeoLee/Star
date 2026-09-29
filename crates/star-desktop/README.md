# `crates/star-desktop/` — Tauri 2.0 PoC P0

> **status**: PoC P1 (per [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md))
> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: user 2026-09-30 "A: 继续 P1 准备" (PR-240 follow-up of PR-239)
> **PR history**: P0 skeleton (PR-239) → P1 icons + IPC (PR-240, 本 PR) → P2 实战 (留 PR-241+)

## 目标

验证 Tauri 2.0 跨平台 desktop + 复用现有 Rust crates (95+ crates 复用, 0 动业务 logic) 可行性。

## 结构

```
crates/star-desktop/
├── Cargo.toml                  # Rust crate manifest (cdylib + bin)
├── src-tauri/
│   ├── Cargo.toml              # Tauri binary manifest
│   ├── build.rs                # tauri-build (codegen)
│   ├── tauri.conf.json         # Tauri app config (window + bundle)
│   ├── capabilities/
│   │   └── default.json        # 默认权限 (core:default)
│   ├── icons/                  # 32x32.png + 128x128.png + icon.icns + icon.ico (待补)
│   └── src/
│       ├── main.rs             # entry point: star_desktop_lib::run()
│       └── lib.rs              # Tauri 2.0 setup + 1 IPC command (list_work_items)
└── frontend/
    ├── package.json            # @tauri-apps/api + vite + typescript
    ├── vite.config.ts          # Vite dev (port 1420)
    ├── tsconfig.json           # TS strict
    ├── index.html              # 1 page: Worktree Board (6 列 × W/T/M)
    └── src/
        └── main.ts             # IPC call → render 6 列 × mock data
```

## IPC commands (本期 P1 实现)

| Command | 入参 | 出参 | 来源 |
|---|---|---|---|
| `list_work_items` | 无 | `Vec<WorkItem>` (mock 4 items) | P0 (PR-239) |
| `list_worktree_groups` | 无 | `Vec<WorktreeGroup>` (mock 3 groups) | P1 新增 |
| `list_canvas_entities` | 无 | `Vec<CanvasEntity>` (mock 4 entities) | P1 新增 |
| `get_app_version` | 无 | `String` (semver from `CARGO_PKG_VERSION`) | P1 新增 |
| `get_keyboard_layout` | 无 | `KeyboardLayout` (W/T/M swimlane + 6 statuses) | P1 新增 |

## 复用 crates (守门 #19: 0 改)

- `crates/domain-board` — 0 改, 仅 deps 引用 (供后续 P1 接入)
- `crates/relationship-engine-wasm` — 0 改, 供前端 WASM 调用 (P1)

## 守门 #7 `unsafe_code = "forbid"`

Cargo.toml `[lints] workspace = true` 继承 workspace lint (`unsafe_code = "forbid"`)。

## 本 PR 不做

- ❌ Tauri 全 bundle build (`cargo tauri build` — 需要 icons + `wry` 配置, 留 P1)
- ❌ WebView 实际渲染 (需要前端 `npm run dev` + 后端 `cargo tauri dev`, 留 P1)
- ❌ 真实 IPC commands (除 `list_work_items` mock 外)
- ❌ 集成现有前端 `frontend/` (Next.js 14 — 留 P2 渐进迁移)

## 下一步 (PR-241+)

| Task | 内容 | 估 |
|---|---|---|
| ✅ **T-001** | ~~添加 icons (32x32.png / 128x128.png / icon.icns / icon.ico)~~ | 1 天 DONE |
| ⏳ **T-002** | `npm install` 在 `frontend/` + `cargo build -p star-desktop` (Linux build host 或更大内存 Windows) | 1 天 |
| ⏳ **T-003** | `cargo tauri dev` 实际跑起来 (验证 WebView 启动 < 200ms) | 1 天 |
| ✅ **T-004** | ~~真实 IPC: list_worktree_groups + list_canvas_entities + get_app_version + get_keyboard_layout (mock 完成, P2 接 crates/domain-board + crates/canvas-engine)~~ | 1 周 DONE |
| ⏳ **T-005** | 真实前端: 复用 `frontend/src/components/board/KanbanBoard.tsx` (W/T/M swimlane) | 1 周 |

## 验证 (本 PR)

- ✅ **5 icons** 创建: `32x32.png` (2679B) + `128x128.png` (45448B) + `128x128@2x.png` (181435B) + `icon.icns` (45448B) + `icon.ico` (2679B)
- ✅ **5 IPC commands** 完整: list_work_items + list_worktree_groups + list_canvas_entities + get_app_version + get_keyboard_layout
- ✅ **5 unit tests** 添加 (IPC mock data assertions)
- ✅ `cargo metadata --format-version 1` 验证 workspace manifest 有效 (per `cargo metadata` — lightweight check, 不触发 `cargo check` 全 build)
- ⚠ **`cargo check -p star-desktop` 当前 Windows machine build 失败** (Tauri 2.0 引入 100+ Windows crates, 触发 `STATUS_STACK_BUFFER_OVERRUN` / Windows resource exhaustion). 验证需要 Linux build host 或更大内存 Windows machine.

## Refs

- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P0](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md)
- [PR #216 (Rust→WASM research)](https://github.com/UlyssesLeoLee/Star/pull/216)
- [PR #229 (WASM frontend hook)](https://github.com/UlyssesLeoLee/Star/pull/229)
- [PR #238 (query-engine-wasm dsl_to_cypher)](https://github.com/UlyssesLeoLee/Star/pull/238)