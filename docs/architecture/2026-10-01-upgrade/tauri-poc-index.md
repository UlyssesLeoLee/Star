# Star Desktop — Tauri PoC 文档索引 + 按角色阅读指南

> **date**: 2026-10-01 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR-264 (Tauri PoC P0→P10 docs 总览, 新 contributor onboarding 入口)
> **status**: v1.0 (索引 + 4 角色阅读指南 + 21 PR 时间线)

## 1. 文档清单

| 文档 | 阶段 | PR | 摘要 |
|---|---|---|---|
| [rust-app-end-research.md](../../architecture/2026-09-29-upgrade/rust-app-end-research.md) | (前置) | (前 session) | 4 路径对比 (Tauri / WASM / Rust full-stack / egui)，推荐 A+B 双轨 |
| [star-desktop-p0p3-impl-summary.md](../../architecture/2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md) | P0-P3 | #243 | 4 PR 累计 (#239-#242)：skeleton + icons + adapters + docs |
| [star-desktop-p5-frontend-impl-plan.md](../../architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md) | P5 plan | #244 | 5 阶段 9 天实施计划 |
| [star-desktop-p6-distribution-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p6-distribution-guide.md) | P6 | #253 | 3 平台 (.deb/.msi/.dmg) build + 系统依赖 + CI/CD |
| [star-desktop-p7-e2e-performance-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md) | P7 | #254 | 5 services verify + 性能 baseline (8-16x start, 5-8x memory) |
| [star-desktop-p8-signing-updater-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p8-signing-updater-guide.md) | P8 | #256 | Code signing + 公证 + auto-updater (Windows + macOS + Linux) |
| [tauri-p9-ci-secrets-setup-guide.md](../../architecture/2026-09-30-upgrade/tauri-p9-ci-secrets-setup-guide.md) | P9 | #257 | 9 GitHub Actions secrets + Tauri keypair 生成 + workflow 集成 |
| [tauri-p10-sentry-release-guide.md](../../architecture/2026-09-30-upgrade/tauri-p10-sentry-release-guide.md) | P10 | #258 | Sentry SDK 集成 + 第一次 release v0.1.0 实战 |
| [star-desktop-p0p10-final-impl-summary.md](../../architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md) | Final | #259 | P0→P10 终极总结 (timeline + tests + docs + scripts + CI + secrets + 性能) |
| [../../../crates/star-desktop/README.md](../../../crates/star-desktop/README.md) | — | #239 | Tauri 2.0 架构 (本地) |

## 2. 按角色阅读路径

### 🏛️ 架构组 (5 min — 看 design decision)

1. [rust-app-end-research.md](../../architecture/2026-09-29-upgrade/rust-app-end-research.md) — 4 路径对比
2. [star-desktop-p0p10-final-impl-summary.md](../../architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md) §1 + §11 — 21 PR 时间线 + 守门
3. [star-desktop-p5-frontend-impl-plan.md](../../architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md) — React 19 + WASM 集成设计

### 💻 开发组 (15 min — 看 implementation)

1. [star-desktop-p0p3-impl-summary.md](../../architecture/2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md) §1-2 — skeleton + IPC commands
2. [star-desktop-p5-frontend-impl-plan.md](../../architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md) — 5 阶段实施细节
3. PR-#247 types/ids.ts + #252 BoardView.tsx — 看实际代码 (git log / GitHub)
4. [crates/star-desktop/README.md](../../../crates/star-desktop/README.md) — 本地架构

### 🚀 DevOps 组 (10 min — 看 CI/CD)

1. [star-desktop-p6-distribution-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p6-distribution-guide.md) — 3 平台 build + 系统依赖
2. [star-desktop-p7-e2e-performance-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p7-e2e-performance-guide.md) §3-5 — verify-5-services + baseline
3. `.github/workflows/star-desktop-build.yml` — Linux CI 主入口 (per PR #242 + #255)
4. [tauri-p9-ci-secrets-setup-guide.md](../../architecture/2026-09-30-upgrade/tauri-p9-ci-secrets-setup-guide.md) — 9 secrets + keygen

### 🔐 安全组 (10 min — 看 signing)

1. [star-desktop-p8-signing-updater-guide.md](../../architecture/2026-09-30-upgrade/star-desktop-p8-signing-updater-guide.md) — 3 平台 signing + 公证 + auto-updater
2. [tauri-p9-ci-secrets-setup-guide.md](../../architecture/2026-09-30-upgrade/tauri-p9-ci-secrets-setup-guide.md) — 9 secrets 详细配置
3. [tauri-p10-sentry-release-guide.md](../../architecture/2026-09-30-upgrade/tauri-p10-sentry-release-guide.md) — Sentry 集成 + release 实战

## 3. PR 时间线 (21 PRs)

| PR | 阶段 | commit SHA | 标题 |
|---|---|---|---|
| #239 | P0 | 3ad73ad7 | feat(desktop): star-desktop Tauri 2.0 PoC P0 skeleton |
| #240 | P1 | 81b03503 | feat(desktop): star-desktop P1 — icons + 4 IPC commands + 5 tests |
| #241 | P2 | 44e386bb | feat(desktop): star-desktop P2 — 3 adapter 子模块 + 3 IPC commands |
| #242 | P3 | 39ea7923 | ci(desktop): star-desktop Linux build CI workflow |
| #243 | P3 docs | 01bb5e00 | docs(desktop): star-desktop Tauri PoC P0→P3 实施总结报告 |
| #244 | P5 plan | e994e362 | docs(desktop): Tauri P5 前端复用 KanbanBoard 实施计划 |
| #245 | P4 | e8156b43 | feat(desktop): star-desktop P4 DB Adapter trait + MockDb |
| #246 | P5-1 | df099df1 | feat(desktop): Tauri P5 React 19 + Vite skeleton |
| #247 | P5-2a | b8f0a18a | feat(desktop): Tauri P5 types + constants port |
| #248 | P5-2b | 508e7fee | feat(desktop): Tauri P5 KanbanCard port |
| #249 | P5-2c | 630ec19c | feat(desktop): Tauri P5 KanbanBoard port |
| #250 | P5-3 | afbed082 | feat(desktop): Tauri P5 Tauri IPC 3 hooks |
| #251 | P5-4 | 0a596d8e | feat(desktop): Tauri P5 WASM 2 hooks (useLayoutEngine + useQueryEngine) |
| #252 | P5-5 | a1616b5a | feat(desktop): Tauri P5 BoardView 集成 + WorktreePage 接入 |
| #253 | P6 | fb8a8005 | feat(desktop): Tauri P6 distribution script + docs |
| #254 | P7 | e44d02f4 | feat(desktop): Tauri P7 完整 E2E 验证脚本 + performance baseline docs |
| #255 | CI | 2ae1e7fe | ci(desktop): star-desktop multi-OS matrix CI |
| #256 | P8 | 950fbfd0 | feat(desktop): Tauri P8 code signing + 公证 + auto-updater |
| #257 | P9 | 6c82af3b | feat(desktop): Tauri P9 CI secrets 配置 + keygen script |
| #258 | P10 | 2d6ac918 | feat(desktop): Tauri P10 Sentry crash reporting + release 实战 |
| #259 | Final | b3ebded2 | docs(desktop): Tauri PoC P0→P10 final implementation summary |

(所有 21 SHAs 已用 `git cat-file -t <sha>` 验证)

## 4. 关键数字

| 维度 | 数量 |
|---|---|
| **Tauri IPC commands** | 8 (5 mock + 3 adapter-based) |
| **React hooks** | 6 (3 Tauri IPC + 2 WASM + 1 JS-fallback) |
| **React components** | 3 (KanbanCard + KanbanBoard + BoardView) |
| **Vitest tests** | 57 (Tauri 端) |
| **Rust unit tests** | 34 (Tauri 端) |
| **Total Tauri-side tests** | 91 |
| **GitHub Actions secrets** | 12 (9 signing + 3 Sentry) |
| **CI jobs** | 4 (linux + windows + macos + matrix-summary) |
| **Icons** | 5 (32x32 / 128x128 / 128x128@2x / icns / ico) |
| **Docs LOC** | ~1146 (8 docs + 本索引) |
| **Scripts** | 6 (vite + build.rs + build-wasm + build-desktop + verify-5-services + generate-tauri-keys) |
| **性能加速** (vs Next.js) | 8-16x 启动, 5-8x 内存, 5-10x CPU |
| **Bundle size** | 14 MB (vs Electron 150 MB) |

## 5. Refs

- 21 PRs (per §3 table)
- [rust-to-wasm-frontend-memory-research.md](../../architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md) — WASM 集成层 design (前 session)
- `.github/workflows/star-desktop-build.yml` — Linux CI 主入口
- `crates/star-desktop/README.md` — Tauri 架构 (本地)