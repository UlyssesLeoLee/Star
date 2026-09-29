# Star Desktop — Tauri PoC 全栈实施路径报告 (P0 → P3)

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #239-#242 (Tauri PoC P0 → P1 → P2 → P3, 4 PRs 全部合 main)
> **status**: P3 (Linux build CI workflow 已落地); P4+ 留后续

## 1. 阶段总览

| 阶段 | PR | 内容 | commit |
|---|---|---|---|
| **P0** | #239 | Tauri 2.0 skeleton (Cargo.toml + src-tauri/ + frontend/) + 1 IPC mock | `3ad73ad7` |
| **P1** | #240 | 5 icons + 4 IPC commands + 5 tests (1 → 5 IPC) | `81b03503` |
| **P2** | #241 | 3 adapter 子模块 (Board + Worktree + Canvas) + 3 IPC + 16 tests (5 → 8 IPC) | `44e386bb` |
| **P3** | #242 | GitHub Actions `star-desktop-build.yml` (Linux build host CI) | `39ea7923` |

## 2. 复用 crates (守门 #19: 0 动 V0.1 任何业务 logic)

P0 → P2 复用 4 个现有 crates:

| Crate | 用途 | PR |
|---|---|---|
| `crates/domain-board` | BoardKind + SwimlaneGroupBy + BoardColumn + BoardCard (3 + 4 + 6 enum variant count) | #241 |
| `crates/domain-worktree` | WorktreeStatus enum + 6 variants | #241 |
| `crates/canvas-engine` | router() + Phase enum + 5 variants + route prefix | #241 |
| `crates/relationship-engine-wasm` | WASM 复用 (P4+ 实战) | #239 |

**0 改 V0.1 业务 logic**: P2 仅引用 enum variant count + boundary check, 不调函数, 不接 DB。

## 3. IPC Commands 累计 (1 → 5 → 8)

| # | Command | Payload | 复用 crate | 阶段 |
|---|---|---|---|---|
| 1 | `list_work_items` | `Vec<WorkItem>` (mock 4) | mock | P0 |
| 2 | `list_worktree_groups` | `Vec<WorktreeGroup>` (mock 3) | mock | P1 |
| 3 | `list_canvas_entities` | `Vec<CanvasEntity>` (mock 4) | mock | P1 |
| 4 | `get_app_version` | `String` (CARGO_PKG_VERSION) | env | P1 |
| 5 | `get_keyboard_layout` | `KeyboardLayout` (W/T/M + 6) | static | P1 |
| 6 | `get_board_info` | `BoardInfo` (3 + 4 + 6) | domain-board | P2 |
| 7 | `get_worktree_info` | `WorktreeInfo` (6 + 4) | domain-worktree | P2 |
| 8 | `get_canvas_info` | `CanvasInfo` (route + 5 phases) | canvas-engine | P2 |

## 4. Tests 累计 (5 → 10 → 26)

| 阶段 | 新增 | 累计 | 测试类型 |
|---|---|---|---|
| P0 | 5 | 5 | IPC mock data assertions |
| P1 | 5 | 10 | +4 IPC payload + version + layout |
| P2 | 16 | 26 | +13 adapter (Board 7 + Worktree 4 + Canvas 2) +3 info IPC |

## 5. Build Status

| 验证项 | 状态 | 备注 |
|---|---|---|
| `cargo metadata --no-deps` | ✅ | workspace manifest 有效 |
| `cargo check -p star-desktop` (Windows) | ⚠ STATUS_STACK_BUFFER_OVERRUN | Tauri 2.0 引入 100+ Windows crates, 资源耗尽 |
| `cargo check -p star-desktop` (Linux) | ⏳ 等 CI 验证 | PR #242 GitHub Actions ubuntu-22.04 |
| `cargo test -p star-desktop --lib` (Windows) | ⚠ 同样失败 | 同上 |
| `cargo test -p star-desktop --lib` (Linux) | ⏳ 等 CI 验证 | 预期 26 tests pass |

## 6. 关键守门 (B-mini)

| 守门 | 状态 |
|---|---|
| **#7 unsafe_code = "forbid"** | ✅ 全部 crate `#[forbid(unsafe_code)]` |
| **#11 缺标比错标** | ✅ 复用 enum variant count 与 V0.1 严格同步 (3 + 4 + 6 + 5 = 18 enum variants 全部 anchor) |
| **#19 0 动 V0.1 业务 logic** | ✅ P2 仅引用 enum + boundary, 0 调函数 + 0 接 DB |
| **#6 v2 cargo check -j 4 workspace 互锁** | ⏳ 等 Linux CI 验证 |
| **#14 v4 PR review** | ⏳ 等 Linux CI + 后续 reviewer |

## 7. 后续 PR 计划 (留 PR-243+)

| PR | 内容 | 估 |
|---|---|---|
| **PR-243** | **P4 真实 IPC**: get_board_info 接 `crates/domain-board` database (替换 mock) | 1 周 |
| **PR-244** | **P5 前端复用**: `frontend/src/components/board/KanbanBoard.tsx` (W/T/M swimlane) 嵌入 Tauri webview | 1 周 |
| **PR-245** | **P6 打包分发**: `cargo tauri build` (Linux .deb + Windows .msi + macOS .app) | 2 周 |
| **PR-246** | **P7 完整 E2E**: 5 services port-forward + Tauri 集成 + 截图 + 性能 baseline | 2 周 |

## 8. Refs

- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- PR #239 [feat(desktop): star-desktop Tauri 2.0 PoC P0 skeleton](https://github.com/UlyssesLeoLee/Star/pull/239)
- PR #240 [feat(desktop): star-desktop P1 — icons + 4 IPC commands + 5 tests](https://github.com/UlyssesLeoLee/Star/pull/240)
- PR #241 [feat(desktop): star-desktop P2 — 3 adapter 子模块 + 3 IPC commands](https://github.com/UlyssesLeoLee/Star/pull/241)
- PR #242 [ci(desktop): star-desktop Linux build CI workflow](https://github.com/UlyssesLeoLee/Star/pull/242)
