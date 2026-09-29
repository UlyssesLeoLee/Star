# `crates/star-desktop/` — Tauri 2.0 PoC P0

> **status**: PoC P0 (per [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md))
> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: user 2026-09-30 "B: 跳过 PR-239, 直接开始 Tauri PoC"

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

## IPC commands (本期实现)

| Command | 入参 | 出参 | 来源 |
|---|---|---|---|
| `list_work_items` | 无 | `Vec<WorkItem>` (mock 4 items) | `crates/star-desktop/src-tauri/src/lib.rs` |

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

## 下一步 (P1)

| Task | 内容 | 估 |
|---|---|---|
| **T-001** | 添加 icons (32x32.png / 128x128.png / icon.icns / icon.ico) | 1 天 |
| **T-002** | `npm install` 在 `frontend/` + `cargo build -p star-desktop` | 1 天 |
| **T-003** | `cargo tauri dev` 实际跑起来 (验证 WebView 启动 < 200ms) | 1 天 |
| **T-004** | 真实 IPC: list_worktree_groups + list_work_items_from_db (接 crates/domain-board) | 1 周 |
| **T-005** | 真实前端: 复用 `frontend/src/components/board/KanbanBoard.tsx` (W/T/M swimlane) | 1 周 |

## 验证 (本 PR)

- ✅ `crates/star-desktop/` 完整骨架 (Cargo.toml + src-tauri/ + frontend/)
- ✅ Tauri 2.0 配置 + capabilities 完整 (`tauri.conf.json` + `capabilities/default.json`)
- ✅ Frontend TS strict + Vite config + index.html 1 page
- ⚠ **`cargo check -p star-desktop` 当前 Windows machine build 失败** (Tauri 2.0 引入 100+ Windows crates, 触发 `STATUS_STACK_BUFFER_OVERRUN` / Windows resource exhaustion). 验证需要 Linux build host 或更大内存 Windows machine.

## Refs

- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P0](../../../docs/architecture/2026-09-29-upgrade/rust-app-end-research.md)
- [PR #216 (Rust→WASM research)](https://github.com/UlyssesLeoLee/Star/pull/216)
- [PR #229 (WASM frontend hook)](https://github.com/UlyssesLeoLee/Star/pull/229)
- [PR #238 (query-engine-wasm dsl_to_cypher)](https://github.com/UlyssesLeoLee/Star/pull/238)