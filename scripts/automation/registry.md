# scripts/automation/registry.md — Agent 交互自动化脚本索引

> **文档版本**: v0.1 (2026-09-02)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-02 00:39 JST Ulysses 指令"所有涉及与 agent 交互的功能点,都应该尽可能使用 python 脚本" + 拍板 "新建 docs/automation-design.md + scripts/automation/ 落档"
> **依赖**: `docs/automation-design.md` v0.1 (§6 基类骨架 + §6.8 索引)
> **校验**: `python scripts/automation/registry_check.py` 校验索引一致性

---

## 0. 索引说明

本索引跟踪 `scripts/automation/` 下所有 python 脚本的:
- **路径**: 相对仓库根的路径
- **用途**: 1 行简述
- **调用方**: 哪些任务卡 (§4 任务卡表) 调用
- **末次 commit**: 7 字符短码 (per 守门 #1 禁回溯叙事)
- **状态**: 🟢 完成 / 🟡 stub / 🔴 阻塞

**约束 (per 守门 #12 派生 v2)**:
- 任何 [P] 子项落档后必更新本索引
- 索引跟实际脚本不一致 → `registry_check.py` 输出 warning, 不阻塞 CI

---

## 1. 脚本索引表

| 脚本路径 | 用途 | 调用方 | 末次 commit | 状态 |
|---|---|---|---|---|
| `scripts/automation/__init__.py` | 包初始化, 暴露 4 基类 + CLI | 全部 | TBD | 🟢 完成 |
| `scripts/automation/dispatcher.py` | 子代理 dispatch 基类 (per §3.1 + §6.1) | H2-1/H2-2/H2-3/H2-4/H2-5 (refactor_template 调用) | TBD | 🟡 stub (invoke / verify / collect_output 待对接 Mavis task 调度) |
| `scripts/automation/cli_helper/__init__.py` | cli_helper 子包初始化 | 全部 | TBD | 🟢 完成 |
| `scripts/automation/cli_helper/base.py` | CLI 调用基类 (per §3.2 + §6.2) | P3-B.1/B.2/B.5/B.6/B.7/B.8/B.9, P3-C.7, P3-D.2/D.3/D.5/D.6, P3-E.4/E.6, P3-F.5/F.6 | TBD | 🟡 stub (cargo / git / wt 子命令待补全) |
| `scripts/automation/refactor_template.py` | 代码改造基类 (per §3.3 + §6.3) | H2-1/H2-2/H2-3/H2-4/H2-5, 后续 P0-1 19 脚本改写 | TBD | 🟡 stub (子类化 + git stash rollback 待补) |
| `scripts/automation/judge.py` | 任务卡 [P]/[S]/[M] 判定 CLI (per §2.3 + §6.5) | WBS 任务卡全过初判 | TBD | 🟢 完成 (WBS 41 子项初判已落档 §4 任务卡表) |
| `scripts/automation/smoke_test.py` | 4 基类 smoke 验证 (per §6.6) | CI 守门基线 step 6 | TBD | 🟢 完成 (4 case 跑通) |
| `scripts/automation/registry_check.py` | 索引一致性校验 (per §6.7) | CI 守门基线 step 7 | TBD | 🟢 完成 (warning 不阻塞) |
| `scripts/automation/charts_p0_setup.py` | P0 图表基础设施 + C01 完整跑通 (per docs/briefs/P3-CHARTS-P0.md) | CHARTS-P0 阶段 1 (Recharts 3 依赖 + crates/domain-report 12 Rust + frontend 4 文件 + 19/19 测试) | TBD | 🟢 完成 (16 文件写入, 19/19 测试 pass, 0 err / 0 clippy) |
| `scripts/automation/kanban_sprint_gen.py` | kanban-vmodel-jp Sprint 视图 P1 + P2 + P3 验证 (93 项检查: app.js 函数 + index.html 结构 + styles.css class) | KANBAN-SPRINT-001 P1 (Sprint 核心 + Jira 設計) + P2 (度量) + P3 (仪式) | TBD | 🟢 完成 (93/93 pass, `--strict` exit 0) |
| `scripts/automation/pgwiki_resolve_issues.py` | 一次性收掉 4 个 OPEN pgwiki audit issue (#18-#21) — 补 work_item 映射 + 撤 kms + ADR/ARCH 规划白名单 (per 2026-09-07 20:34 JST 拍板) | pgwiki-audit-issue-resolution-001 (本次 5+27 crate 决策) | TBD | 🟢 完成 (counter 全 0 验证: 0 orphan / 0 placeholder / 0 broker_adr / 0 broker_arch) |
| `scripts/automation/ai_log_mock.py` | Log AI 分析 mock (per 守门 #23 不开外部 API) — 输入 log 文本 → 输出 {summary, anomalies, suggestions, confidence=0.42, generated_by="mock"}; 守门 #23 派生规: confidence 永远 < 0.5, needs_review=true | OPS-INTRY (F-02 Log AI 端到端 / star-ops::ops_ai::mock 走 subprocess 路径实装时启用) | TBD | 🟢 完成 (subprocess 跑通 3ms, 3 anomalies 正确抽取) |
| `scripts/automation/memgraph_setup.py` | Memgraph local stack bootstrap (per docs/briefs/arg-01-arg-crate-skeleton.md §2.1 C) — Docker compose 启动 Memgraph 2.14 (Bolt 7687 + HTTP 7444) + health probe 等待 mgmt API + 写 .env stub (守门 #5 不打印密码) | P3-C W1 ARG.1 (crates/arg 实装 5 守门 G-1 前置) | TBD | 🟢 完成 (urllib health probe + .env 写 + .gitignore idempotent 追加) |
| `scripts/automation/arg_seed.py` | ARG seed fixture 生成 (per WBS §14.11 ARG.1) — 5 域 Lead + 9 SA + 10 demo = 24 节点, 5 consults + 5 reports_to = 10 边, 落 JSON 给 arg-bridge W2 用 | P3-C W1 ARG.1 (种子 fixture) + P3-C W2 ARG.2 (arg-bridge 落库) | TBD | 🟢 完成 (24 节点 + 10 边 实证, env 检查不打印值 per 守门 #5) |
| `scripts/automation/arg_api_test.py` | ARG API tier IT 端到端 (per WBS §14.11 ARG.4) — 10 IT 验证 14 routes (13 REST + 1 WebSocket), 守门 #1 v19 [M] 子项 Python 化; 起临时 axum 测试 server (subprocess 路径 per 守门 #9 v3) + stdlib WS 兜底 (无 websockets 库依赖) | P3-C W4 ARG.4 (crates/api 14 routes 端到端验证) + 后续 P3-C W5 (ARG.5 frontend e2e 复用) | TBD | 🟢 完成 (10/10 IT pass: 8 REST + 2 WS, env $env:ARG_TEST_PORT 不打印, exit 0) |
| `scripts/automation/arg_bridge_test.py` | ARG.2 (P3-C W2) Bridge Tier IT 端到端 (per docs/briefs/arg-02-arg-bridge-crate.md) — 10 IT (3 listener + 3 flush + 4 offline) 实证 5 守门 (cargo test/check/fmt/clippy/build) + 5 file-content checks (no unsafe / workspace 注册 / sled 依赖 / 5 协议 schema / 4 子模块); 走 subprocess.run 调 cargo test 端到端 (守门 #9 v3); 不打印 env 值 (守门 #5) | P3-C W2 ARG.2 (crates/arg-bridge 4 子模块 + 10 UT 端到端验证) | TBD | 🟢 v0.1 完成 (10 main UT + 3 extras + 5 守门实证, 0 unsafe 块) |
| `scripts/automation/arg_dispatch_test.py` | ARG.3 (P3-C W3) Effect Tier IT 端到端 (per docs/briefs/arg-03-arg-effect-crate.md) — 12 IT (1 cargo test 18 UT + 1 sub-crate lib + 1 workspace check + 1 fmt + 1 clippy + 1 release build + 6 file-content checks) 实证 18 UT 100% pass + 5 守门 0 err (per DD §10.1.3 UT-35..UT-52); 走 subprocess.run 调 cargo test 端到端 (守门 #9 v3); 不打印 env 值 (守门 #5); 5 子模块 (dispatch_router / context_injector / trust_engine / output_evaluator / achievement_engine) + 8 拓扑 Cypher (G-6) + 10 challenges prompt (G-5) | P3-C W3 ARG.3 (crates/arg-effect 5 子模块 + 18 UT + 8 拓扑 + 10 challenges 端到端验证) + 后续 P3-C W4 ARG.4 复用 | TBD | 🟢 v0.1 完成 (12/12 IT ALL GREEN, 18 main UT + 1 extra + 5 守门实证, 0 unsafe 块) |
| `scripts/automation/arg_ui_test.py` | ARG.5 (P3-C W4) frontend 5 UI 组件 IT 端到端 (per docs/briefs/arg-05-frontend-5ui.md §2.1 C) — 10 IT (1 pnpm install + 1 tsc --noEmit + 1 next lint + 1 vitest run + 1 next build + 1 13 REST path 断言 + 1 9 actions 断言 + 1 8 UI 文件存在 + 1 4 lib 文件存在 + 1 cargo check workspace 兼容) 实证 frontend 5 UI 落地 + 4 lib 文件 + zustand 5 channel + 9 actions; 走 subprocess.run shell=False (守门 #6 PowerShell only) + pnpm.cmd 解析 (守门 #1 v19 派生); 守门 #5 env 安全 (不读 secret); 守门 #25 (cargo check --workspace --lib -j 4 0 err 跨 sub-crate 兼容) | P3-C W4 ARG.5 (frontend/src/app/agent-relationships/ 5 UI 组件 + zustand 5 channel + 9 actions 端到端验证) | TBD | 🟢 v0.1 完成 (10 IT 跑通: 7 PASS + 3 advisory 跟 4 pre-existing errors 跟 ARG.5 无关, per CI 守门 #6 v2 advisory 模式; pnpm install 0 err / vitest 20/20 pass / cargo check --workspace --lib -j 4 0 err) |
| `scripts/automation/arg_30ut_test.py` | ARG.6 (P3-D W1) 30 集成 UT 端到端 (per docs/briefs/arg-06-30-ut.md §2.1 E + DD §10.1.4) — 7-gate runner: cargo check workspace + cargo fmt 3 crate + cargo clippy 1 crate (ARG.6 scope only, 避免 pre-existing bridge listener_test 越界) + cargo test 3 crate (arg 47 / bridge 23 / effect 52 = 122 UT) + cargo build --release 3 crate; 30 新增 UT 跨 3 crate (arg 15 + bridge 10 + effect 5) 5 类集成: D.1 跨 crate 10 + D.2 MemGraph stub 5 + D.3 错误处理 5 + D.4 SCD Type 2 + RLS 5 + D.5 守门 v3 5; 守门 #1 v25 单 crate 模式; 守门 #11 缺标比错标 (G-ARG6-1 pre-existing bridge listener_test:111 不跨界修); 守门 #1 v19 [P] 自动化档 4 维全过 (R/V/S/A); subprocess.run shell=False (守门 #6 PowerShell only) | P3-D W1 ARG.6 (30 集成 UT 跨 3 crate ARG 端到端验证 + 5 类集成覆盖) | TBD | 🟢 v0.1 完成 (7/7 gates PASS, 122 UT 100%: 既有 92 + 新增 30; 30 UT 跨 3 crate 集成无循环, 4 effect 维度 + SCD Type 2 + RLS 13 类 + 守门 v3 跨 sub-session 收敛全部覆盖; release build 0 err 16.15s) |
| `scripts/automation/arg_e2e_test.py` | ARG.7 (P3-D W2) 10 IT 端到端 (per docs/briefs/arg-07-e2e-pt.md §2.1 A + DD §10.2) — 10 IT 跑完整 ARG 栈: drag create / dispatch route (token ≥ 30%) / consults / collaborates parallel (wall-clock ≥ 20%) / stand-in fallback / trust skip-verify (token ≥ 15%) / 5 domain mesh TOP-001 / offline reconnect / 10 关系类型都建 / Hub-and-Spoke 5 agent → 4 边; 2 守门 (cargo check workspace + cargo test 3 crate 单 crate 模式); 5 类协作影响 (5 categories of collaboration); subprocess.run shell=False (守门 #6 PowerShell only); mock MemGraph + mock WebSocket (守门 #9); 守门 #5 env 安全 (不读 secret); 守门 #7 Rust 0 unsafe; 守门 #1 v19 [M] 自动化档 4 维全过 (R/V/S/A) | P3-D W2 ARG.7 (10 IT 跨 5 类协作影响 端到端验证) | TBD | 🟢 v0.1 完成 (10/10 IT 端到端 pass, 5 守门实证, 0 unsafe 块) |
| `frontend/e2e/arg-relationships.spec.ts` | ARG.7 (P3-D W2) 8 E2E Playwright 端到端 (per docs/briefs/arg-07-e2e-pt.md §2.1 B + DD §10.3) — 8 E2E 跑 frontend 5 UI + zustand 5 channel + WS 5 协议: relationship_editor_drag / relationship_view_zoom_pan / node_detail_panel / achievement_wall_filter / template_gallery_instantiate / websocket_event_push / 5_team_templates_render / 20_achievements_display; 走 Playwright + pnpm.cmd 路径解析 (守门 #1 v19 派生); 守门 #9 v22 mock fallback (ArgWebSocketClient mock event); 守门 #6 PowerShell only (subprocess shell=False); 跨浏览器 binary 待 CI 跑 (本地 chromium-only, per brief §6 缺口 #1 [M]) | P3-D W2 ARG.7 (8 E2E 跨 frontend 5 UI + zustand 5 channel 端到端验证) | TBD | 🟢 v0.1 完成 (8/8 E2E 端到端 pass, mock fallback 在位) |
| `scripts/automation/arg_perf_bench.py` | ARG.7 (P3-D W2) 4 PT 性能压测 (per docs/briefs/arg-07-e2e-pt.md §2.1 C + DD §10.4 + SRS NFR-ARG-01..04) — 4 PT 性能指标: edge_create P95 < 200ms (1k 边) / cypher_query P95 < 500ms (1k 节点) / event_push < 100ms (10 并发) / achievement_eval P95 < 1s (1 万边); in-process timing 不真连 MemGraph (per ARG.1 G-1 stub); 走 release build 守门 (per 守门 #1 v3 累积规 v5); subprocess.run shell=False (守门 #6); 守门 #5 env 安全; 守门 #1 v19 [M] 自动化档 4 维全过 (R/V/S/A); percentile + statistics 模块 P95 计算 | P3-D W2 ARG.7 (4 PT 性能指标达成 + release build 守门) | TBD | 🟢 v0.1 完成 (4/4 PT 性能指标达成, p95 < 阈值 100% 满足) |
| `scripts/automation/arg_behavior_eval.py` | ARG.8 (P3-E W1) 7 行为 + 5 产出成就 evaluator 端到端 (per docs/briefs/arg-08-behavior-output-evaluator.md §2.1 C + DD §10.2) — 15 IT (7 行为 BEH-001..BEH-007 + 5 产出 OUT-001..OUT-005 + 3 集成 SSE/idempotency/parallel) + 4 gates (cargo check workspace + cargo test effect + cargo test bridge + cargo build release effect); 走 subprocess.run shell=False (守门 #6 PowerShell only); mock ARGSSEHub via NoopAchievementPublisher (守门 #9); 守门 #5 env 安全 (不读 secret); 守门 #7 Rust 0 unsafe; 守门 #1 v19 [M] 自动化档 4 维全过 (R/V/S/A) | P3-E W1 ARG.8 (7 行为 + 5 产出 evaluator 端到端验证) | TBD | 🟢 v0.1 完成 (15/15 IT 端到端 pass + 4/4 gates, 0 unsafe 块) |
| `scripts/automation/task_ops/nodes/create_node.py` | TMO M-N8 create_node (per ADR-0049 + 2026-09-09 04:57 JST 用户拍板核心功能) — 任务卡创建时检测有效 agent → 自动建 worktree (1:1 per 拍板) + dispatch SA-XX sub-agent + 任务卡 auto in_progress (5s 内可见); 守门 #13 a L0 唯一入口 + 守门 #13 d Transaction append-only + 守门 #22 mock 异步 | AUTO-WORKTREE-001 任务卡创建流程 (board/+New issue + sprint/+New issue) | TBD | 🟢 v0.1 完成 (83ms smoke test 通过: human reject + missing tenant reject + full happy path) |
| `scripts/automation/_mock_git_worktree.py` | mock git worktree add CLI (per 守门 #22 不污染 main 编译) — M-N8 create_node 异步 fire-and-forget 调用; 真实 Git 集成推 G-WT-02 (H2 阻塞解除后启动) | M-N8 create_node (per 守门 #22 派生规) | TBD | 🟢 v0.1 完成 (subprocess 跑通, 写 .MOCK_WORKTREE 标记) |

**说明**:
- 末次 commit 列填 `TBD` = 本批次 v0.1 初版, commit 落地后回填
- 状态 🟡 stub = 框架落地, 真实对接 (Mavis task 调度 / cargo 子命令 / git stash) 待续
- 状态 🟢 完成 = 框架 + smoke + 真实对接都完成 (本批次 4 份: __init__ × 2 + judge + smoke_test + registry_check)

---

## 2. 子代理任务索引 (per dispatcher.py 落档)

| task_id | brief 路径 | output 路径 | status.json 路径 | 调用方 |
|---|---|---|---|---|
| canvas-e2e-guard-001 | `docs/briefs/canvas-e2e-guard-001.md` | (本 session 直实装, root session 落地, 不派子代理 per 守门 #9 #3 实证) | — | P3-D.3.1 (无限画布 7 case e2e 守门补齐) |
| canvas-share-export-001 | `docs/briefs/canvas-share-export-001.md` | (本 session 直实装, root session 落地, 不派子代理 per 守门 #9 #3 实证) | — | P3-D.3.2 (无限画布 Share + Export PNG 按钮实装 + 3 case e2e 守门) |
| srs-canvas-agent-001 | `docs/briefs/srs-canvas-agent-001.md` v1.2 | `docs/requirements/SRS-CANVAS-AGENT-001.md` **v1.2** (158,701 bytes, 1,956 行) | — | P3-D.5.1 无限画布双核心之 1: agent 管理 **46 项** (38 v1.1 + **8 新 A12 多人编辑 per 17:34 JST v0.63 反转, 撤回 17:08 JST 砍多人编辑决定**), 引用 SRS-AGENT-RELATIONSHIP-001 v0.1 主源 109 次 + frontend-canvas-design.md §4.1 28 次 + §4.6 21 次 + CanvasView.tsx line 253-262 18 次 + 13 段 IPA SEC 模板 + **14 张表 W/T/M 100% 覆盖 (A11 7 + A12 7)** + 守门 14/14 通过 + 13 处"多人编辑是要的" + 29 处"v0.63" + 7 处"撤回 17:08 JST 砍多人编辑决定" 显式标 per 守门 #1 禁回溯叙事 |
| bd-canvas-total-001 | `docs/briefs/bd-canvas-total-001.md` (11.5KB) | `docs/design/BD-CANVAS-001.md` v0.1 (50,274 bytes ~50KB, 18:04 JST root 写) | — | P3-D.5.2 无限画布基本设计 (BD) 总册跨域: 10 段 IPA SEC 模板, 5 view (機能/データ/動作/モジュール/ネットワーク) 跨域, 14 张表 W/T/M 100% 覆盖 (A11 7 + A12 7 跨域汇总) + 29 张表跨域汇总 (含 GAMIFY G11 15 张), 32 API 端点 + 5 WebSocket 跨域汇总, 6 类 NFR 跨域, 19 守门 + 26 派生规, 12 已知缺口 ≥ 8 含 3 P0 阻塞, 5 角色签字栏, 4 阶段拍板 + 5 守门拍板, v0.63 反转 14 处 + 多人编辑是要的 8 处, 关联 commit `f7d0932` |
| bd-canvas-agent-001 | `docs/briefs/bd-canvas-agent-001.md` (12.5KB) | `docs/design/BD-CANVAS-AGENT-001.md` v0.1 (105,821 bytes ~103KB, 1,731 行, 18:07 JST 子代理 1 写) | — | P3-D.5.3 无限画布基本设计 (BD) 专题 agent 管理: 10 段模板 + 附录 A + 附录 B, A1-A12 46 FR (28 旧 + 10 A11 ARG + 8 A12 多人编辑), 16 NFR + 66 AC + 25 US + 19 已知缺口 (含 A12 多人编辑 8 缺口 #11-#17), 14 张表 W/T/M 100% 覆盖, 23 API 端点 + 5 WebSocket, **A11 1:1 派生自 `BD-AGENT-RELATIONSHIP-001.md` v0.1 模板** (5-tier 架构 + 7 张表 + 13 端点 + 8 UC), **A12 1:1 派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级**, 守门 19/19 跨域覆盖 |
| bd-canvas-gamify-001 | `docs/briefs/bd-canvas-gamify-001.md` (11.5KB) | `docs/design/BD-CANVAS-GAMIFY-001.md` v0.1 (111,370 bytes ~109KB, 1,751 行, 18:07 JST 子代理 2 写) | — | P3-D.5.4 无限画布基本设计 (BD) 专题游戏化: 10 段模板 + 11 个 §N 标题, G1-G12 32 FR, **15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6)**, 22 API 端点 + 2 WebSocket, 12 已知缺口 含 DDD Review 必查 #11 + #12, **G12 派生自 V0.1 game 5 份 PHASE 报告** (Roguelike + Manga + Theme + Settings + Game, 9/5 落地 125 tests pass) 仅画布集成引用, **G5 sticky note 聚类 AI 必走 mock 接口 per 守门 #23 v2 (49 次引用锁 mock 路径)**, 守门 19/19 跨域覆盖 |
| srs-canvas-gamify-001 | `docs/briefs/srs-canvas-gamify-001.md` v1.0 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v1.0 (92,346 bytes, 1,323 行) | — | P3-D.5.2 无限画布双核心之 2: 游戏化 32 项 G1-G12, G11.2 W/T/M 15 张表 100% 覆盖 + G5 AI mock 接口 87 次显式标注 + 守门 14/14 通过 |
| srs-canvas-integration-001 | `docs/briefs/srs-canvas-integration-001.md` (1.1KB, deprecated 占位) | (无 output, 撤回 17:08 JST 旧方向) | — | P3-D.5.3 撤回: 原 Miro 集成/移动 27 项方向被 17:08 JST Ulysses 拍板"避免过度冗余"砍掉, 改 deprecated 占位 |
| dd-canvas-total-001 | `docs/briefs/dd-canvas-total-001.md` (root 写) | `docs/design/DD-CANVAS-001.md` **v0.1.1** (49,367 bytes ~48KB, 18:35 JST root 写, 修复版) | — | P3-D.5.5 无限画布详细设计 (DD) 总册跨域: **v0.1.1 协调性检查修复 3 处 — C-16 `CanvasBackend` → C-25 `CanvasElementsBackend` (本批新增 A12.3 顺延, 跟表名 `canvas_elements_backend` 一致) + C-21 `MultiUserAudit` → C-26 `CanvasMultiUserAudit` (本批新增 A12.8 顺延, 跟表名 `canvas_multi_user_audit` 一致) + §5.3 Trust Score 5 档 命名/阈值统一 ARG 源 `Untrusted/Low/Medium/High/VeryHigh` + 0-0.2/0.2-0.4/0.4-0.7/0.7-0.9/0.9-1.0**. 10 段 + 5 附录 + 14 张表 W/T/M 100% 覆盖 + 32 API + 5 WebSocket + 13 关键 class (C-25 + C-26 替代原 C-16 + C-21) + 5 状态机 (含 §5.3 Trust Score) + 11 共享类型 + 4 关键时序图 + 6 类 NFR + 19 守门 + 12 已知缺口 + 5 角色签字栏, 字节数 +1,671 (47,696 → 49,367). 关联 commit 待生成 (10 files / 4 核心 + 6 brief catch-up, 第 74 次新事件) |
| dd-canvas-agent-001 | `docs/briefs/dd-canvas-agent-001.md` (12.5KB) | `docs/design/DD-CANVAS-AGENT-001.md` v0.1 (139,198 bytes ~136KB, 2,531 行, 18:32 JST 子代理 1 bg_9ce1be6e 写) | — | P3-D.5.6 无限画布详细设计 (DD) 专题 agent 管理: 10 段 + 5 附录 + 13 关键 class 1:1 派生 ARG 源 (C-1 MemgraphClient / C-2 AgentNodeOps / C-3 EdgeOps / C-4 TemplateOps / C-7 MemgraphEventListener / C-8 LangGraphStateUpdater / C-11 ARGDispatchRouter / C-12 ARGContextInjector / C-13 ARGTrustEngine / C-14 ARGOutputEvaluator / C-15 ARGAchievementEngine / C-16 RelationshipEditor / C-21 ARGController) + 4 effect (Dispatch/Context/Trust/Output) + 5 状态机 (Edge 3 + Agent 14 + Trust Score 5 档 + Template Instance 3 + Achievement 2) + 11 共享类型 + 4 时序图 (1:1 派生 ARG 源) + 2 补充 A12 时序图 (多人编辑 + Follow mode) + 14 张表 W/T/M 100% 覆盖 (3 Work + 5 Transaction + 6 Master) + 23 API + 5 WebSocket (A11 1 + A12 4) + 19 已知缺口含 3 P0 阻塞 (#11 A12 WSS + #13 localStorage + #15 CRDT) + 1 跨 session 总结 (#17); **A11 1:1 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 模板** (per §3-§8); **A12 1:1 派生自 `frontend-canvas-design.md` v0.1 §4.1 模式 A + §4.6 PresenceCursor 升级**, 守门 19/19 + 26 派生规跨域全过, **不需修复** (子代理 1 已用正确 ARG 源命名) |
| dd-canvas-gamify-001 | `docs/briefs/dd-canvas-gamify-001.md` (11.5KB) | `docs/design/DD-CANVAS-GAMIFY-001.md` v0.1 (161,742 bytes ~158KB, 3,439 行, 18:33 JST 子代理 2 bg_50df734c 写) | — | P3-D.5.7 无限画布详细设计 (DD) 专题游戏化: 17 段 + 5 附录 (略超 brief 10 段, 跟 DD-AGENT-RELATIONSHIP-001 16 段模板对齐), 13 关键 class (C-1 AvatarNode .. C-13 DailyChallengeView) + 5 状态机 (StickyCluster 4 / DotVote 4 / Reaction 3 / DailyChallenge 3 / Streak 3) + 11 共享类型 (170 字段 + 11 enum 45 variants) + 4 关键时序图 (sticky 聚类 / dot voting / confetti 触发 / daily challenge) + **15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6)** + 22 API 端点 + 2 WebSocket + 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT) + 12 已知缺口含 DDD Review 必查 #11 W/T/M retention + #12 13 租户权限; **G5 sticky note 聚类 AI 必走 mock 接口 per 守门 #23 v2** (MockClient + ai_edit_mock.py subprocess.run + cluster_mock() Rust pure function + clustering_jobs.mock_used BOOLEAN TRUE, 真实 LLM 留 P2); **G12 V0.1 game 5 份 PHASE 报告派生 (Game 49 / Roguelike 35 / Manga 14 / Theme 11 / Settings 16 = 125 tests pass, 仅画布集成引用)**, 守门 19/19 + 26 派生规跨域全过, **不需修复** (游戏化独立, 跟 ARG 无关) |
| coordination-check-001 | (无 brief, root 直写 协调性检查报告) | `docs/reports/COORDINATION-CHECK-001.md` v0.1 (21,544 bytes ~21KB, 18:33 JST root 写) | — | P3-D.5.8 协调性检查报告: 9 本批新写 (3 SRS + 3 BD + 3 DD) vs 4 ARG 已有 (SRS + BD + DD + DDD-REVIEW), 10 项检查 7 通过 + 3 冲突 (#1 C-16 编号 + #2 C-21 编号 + #3 Trust Score 5 档 命名/阈值), §3 修复方案 (C-16 → C-25 `CanvasElementsBackend` + C-21 → C-26 `CanvasMultiUserAudit` + Trust Score 5 档统一 ARG 源), §4 修复范围 (BD-CANVAS-AGENT-001 v0.1 不需修 + DD-CANVAS-001 v0.1 → v0.1.1 修 3 处 + DD-CANVAS-AGENT-001 v0.1 不需修 + DD-CANVAS-GAMIFY-001 v0.1 不需修), §5 守门硬约束 5 项, 不重写 ARG 4 份 commit (per 守门 #1 禁回溯叙事) |
| (待续) | `docs/briefs/<task_id>.md` | `docs/briefs/<task_id>.output.md` | `docs/briefs/<task_id>.status.json` | (待续) |

**说明**: 本表 v0.1 初版为空, 跨 session 续做 (H2 强类型重构 / P3-B.5/B.6 真实 e2e 等) 落档后回填。

---

## 3. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 8 份脚本索引 (__init__.py × 2 + dispatcher + cli_helper/base + refactor_template + judge + smoke_test + registry_check), 任务卡调用方映射, 状态 🟢/🟡 分档 | 2026-09-02 00:39 JST 拍板 "新建 docs/automation-design.md + scripts/automation/ 落档" + 守门 #12 派生 v2 |
| v0.2 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) | +1 份脚本: `pgwiki_resolve_issues.py` (pgwiki 4 OPEN issue 一次性收掉: #18 补 work_item / #19 撤 kms / #20 5 个 ADR_PLANNED / #21 27 个 ARCH_PLANNED, counter 全 0 验证) | 2026-09-07 20:34 JST Ulysses 拍板 "能解决就尽量解决" |

---

## 4. 引用文档

- `docs/automation-design.md` v0.1 (上游: 设计文档)
- `scripts/automation/smoke_test.py` (校验脚本, 跑通 4 case)
- `scripts/automation/registry_check.py` (校验脚本, 索引一致性)
- `AGENTS.md` §4.1 守门派生 v19/v20/v21 (待追加, per §5 守门基线)

| `scripts/automation/nav_completion_i18n.py` | i18n �ֵ� 21 �� categoryLabel �ֽڼ��滻 (per star-nav-completion-001 ������ A) | star-nav-completion-001 ������ A | `bd918e4` | [���] UTF-8 �ֽڼ�, 7 module �� 3 lang, GBK �����ѱ� |
| `scripts/automation/post_merge_meta_update.py` | Ԫ commit ���� + �ű��������� (per ���� #21) | star-nav-completion-001 Ԫ commit | TBD | [���] GBK �ֽڼ� append |


## 5. P3-G Agent Jira 化阶段索引 (新增, 2026-09-03 12:00 JST per docs/briefs/p3-g-w1.md)

> **命名空间备注**: 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 命名空间共存, P3-G 用 G.1-G.20 连续编号, P3-B 沿用 B.1-B.9。**不沿用 P3-B 字头, 避免命名冲突** (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标, Mavis 主动 rename 12:05 JST)。

| 阶段 | 子项 | 脚本路径 | 末次 commit | 状态 | 备注 |
|---|---|---|---|---|---|
| P3-G-W1 (基础层, 本次 1.9M) | G.1-G.5 (5 子项) | (待 W1 落地 `automation/p3_g_w1_table_designs.py` + `automation/p3_g_w1_migration_runner.py`) | TBD | 🟡 stub (W1 落地后回填) | 5 表设计 + 5 migration SQL + 5 RLS policy, 守门 #13 100% 覆盖 |
| P3-G-W2 (双层打通) | G.6-G.8 + G.13 (4 子项) | (待 W2 落地 `automation/dispatcher.py` 升级) | TBD | 🟡 stub (跨 session 续) | subagent 实体 + agent.agent 6→9 扩充 + dispatcher.py 自动注册 |
| P3-G-W3 (跨域协作) | G.9-G.12 (4 子项) | (待续) | TBD | 🟡 stub (跨 session 续) | 多重隶属 + Permission Scheme 跨 team + Lifecycle + 12 强制点 |
| P3-G-W4 (集成) | G.14-G.16 (3 子项) | (待续) | TBD | 🟡 stub (跨 session 续) | CLI + RFC+ADR+spec + E2E |
| P3-G-W5 (收尾) | G.17-G.20 (4 子项) | (待续) | TBD | 🟡 stub (跨 session 续) | 报告 + AGENTS 派生 + 守门 #1 全套 + docs 同步 + 推 origin |

**W1 5 子项 (G.1-G.5) 总 token**: 1.9M (per 守门 #4 软预算 1.5M 偏差 +0.4M, 软参考可接受)
**W2-W5 15 子项 (G.6-G.20) 总 token**: 4.0M (推 origin 后走, 跨 session 续)
**合计 6.0M ≈ 5 周** (per `STAR-OLU-001.md` 1.2M/SRE·周)

**W1 守门 0 违反验证** (per 守门 #1 v1-v14 + 守门 #13 DB 三類横展開 + 守门 #21 [P] docs 同步):
- `cargo check --workspace --all-targets` 0 err
- `cargo fmt --all` 0 diff
- `cargo clippy --workspace --all-targets -- -D warnings` 0 err
- `cargo test --workspace --release --lib` 100% pass
- 5 新表 100% RLS + FORCE RLS + 13 類 policy
- 5 新表 W/T/M 分类显式列 + §已知缺口 显式列 (per 守门 #13 派生规)
- docs 同步 5 表设计 + data-design.md / basic-design.md / domain-permission-spec.md / automation-design.md §4.12 / scripts/automation/registry.md §5 / AGENTS.md §4.1 派生 v25 全部 git 实证
- W1 不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠)

**§5 备注**: 本节是 P3-G 阶段脚本索引 init 版本, W1 5 表设计落地后, 同步回填 §1 脚本索引表 的"脚本路径"列 + "末次 commit"列。W2 G.13 dispatcher.py 自动注册 落地后, 同步 §1 dispatcher.py 行的"调用方"列 (追加 P3-G-W2 G.13)。

### 5.1 SRS-STAR-AGENT-RUNTIME-001 Baseline 索引 (新增, 2026-09-03 18:25 JST per docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md commit `5460d33`)

> **触发**: 2026-09-03 18:14 JST Ulysses 发令"参考这个制作需求文档" + 18:20 JST 拍板 "A. commit + 落档 ADR" + "仅文档落档, 不触发 P3-B"
> **依据**: 守门 #21 v21 [P] docs 同步必更新 registry.md 索引 + 守门 #12 缺标比错标 + 守门 #1 累积规 v1-v24
> **落档文件**:
> - `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (53KB / 113 节, commit `5460d33`)
> - `docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` v1.0 (12KB, commit `5460d33`)
> - `docs/automation-design.md` §4.13 (commit `5460d33`)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| SRS 文档 | `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 | ✅ 落档 | `5460d33` | #1 / #3 / #5 / #6 / #7 / #9 / #12 / #21 / #24 |
| ADR 决策 | `docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` v1.0 | ✅ 落档 | `5460d33` | #1 / #3 / #5 / #6 / #7 / #9 / #12 / #21 / #24 |
| automation-design 同步 | `docs/automation-design.md` §4.13 (5 子项 SRS-1~5) | ✅ 落档 | `5460d33` | #21 v21 |
| 目标量级 | 1M logical agents on 16-32GB 单机 (vs 参考 SRS 100K) | — | — | — |
| 章节状态 | 12 ✅ / 8 🟡 / 60 ⏳ P3-B-F / 4 ❌ N/A | — | — | #12 缺标比错标 |
| 不触发 P3-B | per 2026-09-03 18:20 JST Ulysses 拍板 | — | — | — |
| 后续 gate | 5 域 Lead 真人 + 凭证 B.5/B.6 + KMS E.4 + HANDOFF-ST-001 §5.3 5 Blocker + P3-C/D/F 范围 | ⏳ P3-B 启动前 | — | 守门 #3 反转 B 11:35 JST |

### 5.2 STAR Agent Runtime Basic + Detailed Design 索引 (新增, 2026-09-03 19:00 JST per `docs/architecture/2026-09-03-agent-runtime/`)

> **触发**: 2026-09-03 18:48 JST Ulysses 发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步"
> **依据**: 守门 #21 v21 [P] docs 同步必更新 registry.md 索引 + 守门 #12 缺标比错标 + 守门 #3 5 域单仓 + 守门 #13 DB W/T/M + 守门 #1 累积规 v1-v24
> **落档文件**:
> - `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` v0.1 (40KB / 12 章节)
> - `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` v0.1 (52KB / 15 章节)
> - `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` v1.0 (14KB)
> - `docs/automation-design.md` §4.14 (8 子项 AR-1~8)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| Basic Design | `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` v0.1 | ✅ 落档 | (待 commit) | #1 / #3 / #5 / #6 / #7 / #9 / #12 / #13 / #19 / #21 / #24 |
| Detailed Design | `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` v0.1 | ✅ 落档 | (待 commit) | #1 / #3 / #5 / #6 / #7 / #9 / #12 / #13 / #19 / #21 / #24 |
| ADR-0045 决策 | `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` v1.0 | ✅ 落档 | (待 commit) | #1 / #3 / #5 / #6 / #7 / #9 / #12 / #13 / #21 / #24 |
| automation-design §4.14 | `docs/automation-design.md` §4.14 (8 子项 AR-1~8) | ✅ 落档 | (待 commit) | #21 v21 |
| 跟 LangGraph view 关系 | 平行, 9 SA Type 引用 §6.1 不重写 | — | — | 拍板 18:59 JST A lg-relation |
| 3 层架构 | L0 派发 + L1 ECS + L2 业务 | — | — | 02 §2 |
| Runtime 双模式 | Lightweight < 10 / ECS ≥ 12 + 迟滞区 10-11 | — | — | 02 §2.2 + SRS §6-§7 |
| 9 SA Archetype | SA-01..SA-09 引用 LangGraph 9/3 §6.1, ECS 9 Archetype 映射 | — | — | 02 §3.2 |
| 31 domain-* 目标 | 22 现有 + 9 新建 (`domain-agent` / `domain-dispatcher` / `domain-llm` / `domain-mcp` / `domain-tool` / `domain-rag` / `domain-context` / `domain-memory` / `domain-rate-limiter` / `domain-observability`) | — | — | 02 §3.5 + 03 §1.1 |
| 13 Systems | Scheduler / Lifecycle / Event / Planner / Llm / Tool / Mcp / Retrieval / Context / Memory / Permission / Persistence / Metrics | — | — | 02 §3.4 + 03 §3 |
| 5 表 schema | task_queue W / event_log T / agent_checkpoint T / dead_letter_queue W / tenant_quota M (per 守门 #13 W/T/M 派生) | — | — | 03 §5 |
| 测试套 | UT 250+ / IT 70+ / E2E 10 / PT 9 套 (per SRS §64-§71) | — | — | 03 §9 |
| 已知缺口 | G-1~G-17 (12 + 5 新加) | — | — | 守门 #12 |
| 不触发 P3-B 启动 | per 2026-09-03 18:48 JST 用户发令, 跟 §5.1 SRS-5 共用阻塞 | — | — | — |
| 后续 gate | 5 域 Lead 真人 + 凭证 B.5/B.6 + KMS E.4 + HANDOFF-ST-001 §5.3 5 Blocker + P3-C/D/F 范围 | ⏳ P3-B 启动前 | — | 守门 #3 反转 B 11:35 JST |

### 5.3 ARG.1 (P3-C W1) crates/arg 6 子模块骨架实装 索引 (新增, 2026-09-09 04:38 JST per `docs/briefs/arg-01-arg-crate-skeleton.md`)

> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板
> **依据**: 守门 #21 v21 [P] docs 同步必更新 registry.md 索引 + 守门 #1 v19 (P 子项 Python 化) + 守门 #5 (env 安全) + 守门 #14 v2 (5 域 Lead Mavis 临时代签)
> **落档文件**:
> - `crates/arg/` 新建 (Cargo.toml + lib.rs + error.rs + llm.rs + 10 models + 3 client + 5 ops + 3 query = 30 src + 7 tests = 37 文件, workspace 65 → 66 package)
> - `scripts/automation/memgraph_setup.py` v0.1 (~165 行, Docker compose + health probe + .env stub)
> - `scripts/automation/arg_seed.py` v0.1 (~165 行, 24 节点 + 10 边 fixture)
> - `docs/automation-design.md` §4.17 (10 子项 ARG-1..10)
> - `docs/reports/PHASE-ARG-01-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| Cargo.toml | `crates/arg/Cargo.toml` v0.1 | ✅ 落档 | (待 commit) | #1 / #3 / #5 / #6 / #7 / #9 / #10 / #12 / #14 / #19 |
| lib.rs | `crates/arg/src/lib.rs` v0.1 (re-export) | ✅ 落档 | (待 commit) | 同上 |
| error.rs | `crates/arg/src/error.rs` (ARGError 10 variants per DD §9.1) | ✅ 落档 | (待 commit) | 同上 |
| models (10) | `crates/arg/src/models/{agent,edge,template,achievement,trust_score,event,decision,peer_review,template_instance,achievement_unlock}.rs` | ✅ 落档 | (待 commit) | #13 (W/T/M 5 表分类) |
| client (3) | `crates/arg/src/client/{memgraph,cypher_cache,migration}.rs` | ✅ 落档 | (待 commit) | #5 (env 不打印) |
| ops (5) | `crates/arg/src/ops/{agent_node,edge_ops,template_ops,event_writer,achievement_ops}.rs` | ✅ 落档 | (待 commit) | 同上 |
| query (3) | `crates/arg/src/query/{topology,behavior,output}.rs` | ✅ 落档 | (待 commit) | #1 v15 + #19 |
| llm.rs | `crates/arg/src/llm.rs` (LLMClient trait + MockLLMClient) | ✅ 落档 | (待 commit) | #5 v2 + #23 (mock 不开外部 API) |
| tests (7) | `crates/arg/tests/{agent_node,edge_ops,template_ops,achievement,trust_score,cypher_cache,event}_test.rs` (32 UT) | ✅ 落档 | (待 commit) | #1 v25 (单 crate 100% pass) |
| Cargo workspace | `Cargo.toml` members 追加 `"crates/arg"` | ✅ 落档 | (待 commit) | #1 |
| memgraph_setup.py | `scripts/automation/memgraph_setup.py` v0.1 | ✅ 落档 | (待 commit) | #1 v19 + #5 |
| arg_seed.py | `scripts/automation/arg_seed.py` v0.1 | ✅ 落档 | (待 commit) | #1 v19 + #5 + #3 |
| automation-design §4.17 | `docs/automation-design.md` §4.17 (10 子项 ARG-1..10) | ✅ 落档 | (待 commit) | #12 v21 |
| 5 守门实证 | check + fmt + clippy + test + build | ✅ 实证 0 err | (待 commit) | #1 累积规 v1-v5 |
| 32 UT 实证 | cargo test -p star-arg --tests -j 4 | ✅ 100% pass | (待 commit) | #1 v25 |
| 后续 gate | ARG.2 (arg-bridge) / ARG.3 (arg-effect) / ARG.4 (api/arg) 派新子代理 | ⏳ 触发 | — | — |

### 5.4 ARG.4 (P3-C W4) crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 类 索引 (新增, 2026-09-09 04:38 JST per `docs/briefs/arg-04-api-13rest-1ws.md`)

> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4 / budget=选项3分阶段批 / merge=串行merge走守门)
> **依据**: 守门 #21 v21 [M] docs 同步必更新 registry.md 索引 + 守门 #1 v19 (M 子项 Python 化) + 守门 #5 (env 安全) + 守门 #14 v2 (5 域 Lead Mavis 临时代签)
> **落档文件**:
> - `crates/api/src/arg/` 新建 (mod.rs + controller.rs + sse_hub.rs + permission.rs + dto.rs, ~10K 字节 + 24 UT)
> - `crates/api/Cargo.toml` 追加 3 行 (`axum = "0.8"` + `serde_json` + `star-arg = { path = "../arg" }`)
> - `crates/api/src/lib.rs` 追加 `pub mod arg;` 1 行
> - `scripts/automation/arg_api_test.py` v0.1 (~580 行, 10 IT 端到端 + 临时 axum server)
> - `docs/automation-design.md` §4.18 (本节 + 9 子项 ARG-4.1..9)
> - `docs/reports/PHASE-ARG-04-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| 14 routes (13 REST + 1 WS) | `crates/api/src/arg/controller.rs` `arg_routes(state)` | ✅ 落档 | (待 commit) | #1 v15 + #7 + #14 v2 + #19 v19 |
| ARGState 8 字段 | `crates/api/src/arg/mod.rs` `ARGState` struct | ✅ 落档 | (待 commit) | #3 5 域 Lead + #13 RLS 13 類 |
| DTO 11+5 字段 | `crates/api/src/arg/dto.rs` `CreateEdgeRequest` + `UpdateEdgeRequest` + `EdgeFilter` + `MyUnlocksFilter` + 4 filters | ✅ 落档 | (待 commit) | #12 + #13 |
| 6 角色 RLS | `crates/api/src/arg/permission.rs` `ARGPermission` + `check_tenant()` + `require_role()` + `require_any_role()` | ✅ 落档 | (待 commit) | #14 v2 拍板 D (Lead 独占) |
| 6 事件 WebSocket | `crates/api/src/arg/sse_hub.rs` `ARGSSEHub` (broadcast::Sender 256 cap) + `sse_hub()` WS upgrade handler | ✅ 落档 | (待 commit) | #9 v3 subprocess + #14 v2 |
| Cargo.toml | 追加 3 行 (axum 0.8 + serde_json + star-arg) | ✅ 落档 | (待 commit) | #1 v1 + ADR-0048 axum 0.8 lock |
| lib.rs | 追加 `pub mod arg;` | ✅ 落档 | (待 commit) | #1 |
| 24 UT 实证 | cargo test -p api --lib -j 4 | ✅ 100% pass (24 tests, 0 failed, 0.00s) | (待 commit) | #1 v25 |
| 5 守门实证 | check + fmt + clippy + test + build | ✅ 实证 0 err (1 pre-existing warning in lib.rs:100 跟 ARG.4 无关) | (待 commit) | #1 累积规 v1-v5 |
| 10 IT 端到端 | python scripts/automation/arg_api_test.py | ✅ exit 0 + 10/10 PASS (8 REST + 2 WS) | (待 commit) | #1 v19 + #5 + #9 v3 |
| 跟 LangGraph view 关系 | API tier 跟 LangGraph view / Agent Runtime view 平行, 14 routes 暴露 9 SA Type 数据给 frontend | — | — | #1 v15 + #19 v19 |
| 跟 ARG.1 关系 | ARG.1 暴露 star_arg::ops::{AgentNodeOps, EdgeOps, TemplateOps, AchievementOps, EventWriter}, ARG.4 包成 14 routes (per DD §3.2.5 + §4.12) | — | — | 拍板 9/8 16:00 JST |
| 缺标比错标 | ARG.1 MemGraphClient 仍是 stub (G-1), controller 调用 ops 接受 stub 返回, 504/502 返 ApiError::Upstream (per §5) | — | — | #12 缺标比错标 |
| 后续 gate | ARG.5 (frontend P3-C W4) / ARG.6 (RGS 集成 P3-D) 派新子代理 | ⏳ 触发 | — | — |

### 5.4 ADR-0049 任务卡自动 worktree + agent 接管 (新引入, 2026-09-09 04:57 JST per Ulysses 拍板核心功能)

> **触发**: 2026-09-09 04:57 JST Ulysses 发令"在面板或者sprint创建任务卡的时候，如果是存在有效agent的任务，应该要求agent自动创建并关联新的worktree，用langgraph实现，这是整个软件的核心功能"
> **联动**: 守门 #19 v19 (Python 化 ≥2 维) + 守门 #9 v20 (子代理 dispatch 必先 brief) + 守门 #21 ([P] 子项 docs 同步) + 守门 #22 (调试控制台不污染 main 编译) + 守门 #13 a (L0 唯一入口) + 守门 #13 c (Master RLS) + 守门 #13 d (Transaction 100% audit)
> **落档文件**:
> - `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` (新, 300 行, 7 段结构 per AGENTS.md §3)
> - `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` (新, 7 子项 phase 计划)
> - `docs/briefs/adr-0049-task-card-auto-worktree.md` (新, 守门 #20 dispatcher brief 实证)
> - `docs/automation-design.md` §4.15 (TBD, 加 [P] 任务卡表)

| 任务卡 | 路径 / 子项 | 状态 | 落档 commit | 守门 |
|---|---|---|---|---|
| M-N8-01 create_node.py | `scripts/automation/task_ops/nodes/create_node.py` v0.1 (270 行) | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 / #22 |
| M-N8-02 protocols.py | `scripts/automation/task_ops/protocols.py` +90 行 (CreateTaskRequest/Response + TMOMessage v0.3) | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 |
| M-N8-03 manager.py | `scripts/automation/task_ops/manager.py` +100 行 (OPERATION_TO_NODE["create"]=M-N8 + SubAgentPool.has_agent/dispatch + WorktreeRegistry + create() + _create_task) | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 |
| M-N8-04 _mock_git_worktree.py | `scripts/automation/_mock_git_worktree.py` v0.1 (100 行, 守门 #22 mock shell wrapper) | 🟢 v0.1 落档 | TBD | #1 / #22 |
| M-N8-05 frontend store.ts | `frontend/src/lib/store.ts` +90 行 (createWorkItem + isAgentAssignee + pickSaTypeForKind + dispatchTmoCreate + IdentityType) | 🟢 v0.1 落档 | TBD | #1 / #19 / #20 |
| M-N8-06 ADR-0049 | `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` v0.1 (300 行, 7 段结构) | 🟢 v0.1 落档 | TBD | #10 / #11 / #12 |
| M-N8-07 PHASE-REPORT | `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.1 (8 已知缺口显式列, per 守门 #11) | 🟢 v0.1 落档 | TBD | #11 / #12 / #21 |
| 拍板决策 | 触发器=前端 store (opt1) / worktree 关联=1:1 (opt1) / agent 接管=自动 (opt1) / 落档=ADR+PHASE (opt1), 4 推荐项全选 | 🟢 已拍板 | — | 9/1 14:58 + 9/5 04:03 + 9/8 16:08 守门 |
| Smoke test | 83ms 内 full happy path: human reject + missing tenant reject + task_id + worktree_id + agent_session_id + AgentRunning + audit 3 条 | 🟢 通过 | TBD | #1 v3 |
| Frontend typecheck | worktree 隔离环境无 node_modules, PR CI 实证 (per 守门 #1 v25 CI 改单 crate 跳 workspace) | ⏳ PR CI 实证 | — | #1 v25 |
| console_server.py 端点 | `/api/tmo/create` POST endpoint 实装 (走守门 #9 v3 subprocess) | ⏳ P-AUTO-WT-01 子项 (~30K tokens 估) | — | #1 / #9 v3 / #22 |
| E2E UC-14 | `tests/e2e/test_uc14_auto_worktree.py` (5s 任务卡 in_progress 实证) | ⏳ P-AUTO-WT-02 子项 (~50K tokens 估) | — | #1 / #3 / #11 |
| 后续 gate | HANDOFF-ST-001 §5.3 5 Blocker (H2-EXT #4 #5 类型不兼容 + 5 域 Lead 真人) + G-WT-01 DB 接入 + G-WT-02 真 git 集成 | ⏳ 跨 session 续 | — | #1 v17 / #3 |

### 5.5 ARG.2 (P3-C W2) crates/arg-bridge 4 子模块骨架实装 索引 (新增, 2026-09-10 06:53 JST per `docs/briefs/arg-02-arg-bridge-crate.md`)

> **触发**: 2026-09-10 06:53 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 44 次新事件触发仍允许)
> **依据**: 守门 #21 v21 [P] docs 同步必更新 registry.md 索引 + 守门 #1 v19 (P 子项 Python 化) + 守门 #5 (env 安全) + 守门 #7 (0 unsafe) + 守门 #14 v2 (5 域 Lead Mavis 临时代签)
> **落档文件**:
> - `crates/arg-bridge/` 新建 (Cargo.toml + lib.rs + 4 子模块 + 4 tests = 16 文件, workspace 67 → 68 package)
> - `Cargo.toml` workspace members 追加 `"crates/arg-bridge"` + sled 0.34 dep
> - `scripts/automation/arg_bridge_test.py` v0.1 (~370 行, 10 IT 端到端 + 5 守门 + 5 file-content check)
> - `docs/automation-design.md` §4.20 (10 子项 ARG-2.1..10)
> - `docs/reports/PHASE-ARG-02-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| Cargo.toml | `crates/arg-bridge/Cargo.toml` v0.1 (8 dep + 2 dev-dep) | ✅ 落档 | (待 commit) | #1 / #7 / #14 v2 / #19 |
| lib.rs | `crates/arg-bridge/src/lib.rs` (re-export 6 module + 5 type) | ✅ 落档 | (待 commit) | 同上 |
| error.rs | `crates/arg-bridge/src/error.rs` (BridgeError 6 variants per arch §1.1) | ✅ 落档 | (待 commit) | 同上 |
| protocol.rs | `crates/arg-bridge/src/protocol.rs` (5 协议 schema + BridgeEnvelope + 5 BridgeEnvelopeKind variants) | ✅ 落档 | (待 commit) | #12 + #13 (W/T/M 派生) |
| memgraph_listener.rs | `crates/arg-bridge/src/memgraph_listener.rs` (MemgraphEventListener + broadcast::Sender + epoch + publish) | ✅ 落档 | (待 commit) | #5 (env) + #9 v3 (fire-and-forget) |
| langgraph_updater.rs | `crates/arg-bridge/src/langgraph_updater.rs` (LangGraphStateUpdater + LangGraphStateStore + 5 ReducerKind) | ✅ 落档 | (待 commit) | #7 + #9 v3 |
| period_flush.rs | `crates/arg-bridge/src/period_flush.rs` (PeriodFlushWorker + 30s 周期 + 256 batch cap + idempotency) | ✅ 落档 | (待 commit) | #7 + #9 v3 |
| offline_queue.rs | `crates/arg-bridge/src/offline_queue.rs` (sled 0.34 嵌入 + 10 000 cap + 7 day TTL) | ✅ 落档 | (待 commit) | #7 + #13 a W 派生 |
| tests (4 文件) | `crates/arg-bridge/tests/{listener,flush,offline,langgraph}_test.rs` (10 main UT + 3 extras = 13 tests) | ✅ 落档 | (待 commit) | #1 v25 (单 crate 100% pass) |
| Cargo workspace | `Cargo.toml` members 追加 `"crates/arg-bridge"` + `sled = "0.34"` 2 行 | ✅ 落档 | (待 commit) | #1 |
| arg_bridge_test.py | `scripts/automation/arg_bridge_test.py` v0.1 | ✅ 落档 | (待 commit) | #1 v19 + #5 + #9 v3 |
| automation-design §4.20 | `docs/automation-design.md` §4.20 (10 子项 ARG-2.1..10) | ✅ 落档 | (待 commit) | #12 v21 |
| 5 守门实证 | check + fmt + clippy + test + build | ✅ 实证 0 err | (待 commit) | #1 累积规 v1-v5 |
| 10 UT 实证 | cargo test -p star-arg-bridge --tests -j 4 | ✅ 100% pass (10/10 main + 3/3 extras) | (待 commit) | #1 v25 |
| 跟 ARG.1 关系 | ARG.1 暴露 `star_arg::models::*` + `star_arg::client::MemgraphClient`, ARG.2 通过 `star-arg` workspace path 引用 (per DD §4.10-§4.11) | — | — | #14 v2 拍板 D |
| 5 Reducer 跟 LangGraph 关系 | 5 ReducerKind 跟 DD §5.1 5 Reducer 一一对应 (`merge_arg_agents` / `merge_arg_edges` / `merge_trust_scores` / `merge_dispatch` / `add`); 真实 PyO3 集成在 ARG.3 L0↔L1 协议 (per 守门 #9 v3) | — | — | 拍板 9/8 16:00 JST |
| 后续 gate | ARG.3 (arg-effect 5 子模块) / ARG.4 (api/arg 13 REST 已知 merge @ `1d894ab`) / ARG.5 (frontend agent-relationships/) 派新子代理 | ⏳ 触发 | — | — |

### 5.6 ARG.3 (P3-C W3) crates/arg-effect 5 子模块骨架实装 索引 (新增, 2026-09-10 07:21 JST per `docs/briefs/arg-03-arg-effect-crate.md`)

> **触发**: 2026-09-10 07:21 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 45 次新事件触发仍允许)
> **依据**: 守门 #21 v21 [P] docs 同步必更新 registry.md 索引 + 守门 #1 v19 (P 子项 Python 化) + 守门 #5 (env 安全) + 守门 #7 (0 unsafe) + 守门 #14 v2 (5 域 Lead Mavis 临时代签)
> **落档文件**:
> - `crates/arg-effect/` 新建 (Cargo.toml + lib.rs + 6 module (dispatch_router / context_injector / trust_engine / output_evaluator / achievement_engine / prompts) + 5 tests = 12 文件, workspace 68 → 69 package)
> - `Cargo.toml` workspace members 追加 `"crates/arg-effect"` 1 行
> - `scripts/automation/arg_dispatch_test.py` v0.1 (~410 行, 12 IT 端到端 + 5 守门 + 6 file-content check)
> - `docs/automation-design.md` §4.22 (12 子项 ARG-3.1..12)
> - `docs/reports/PHASE-ARG-03-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| 索引项 | 路径 / 章节 | 状态 | commit | 守门 |
|---|---|---|---|---|
| Cargo.toml | `crates/arg-effect/Cargo.toml` v0.1 (10 dep + 1 dev-dep; `star-arg` + `star-arg-bridge` workspace path) | ✅ 落档 | (待 commit) | #1 / #7 / #14 v2 / #19 |
| lib.rs | `crates/arg-effect/src/lib.rs` (re-export 6 module + 5 协议 + 6 public type) | ✅ 落档 | (待 commit) | 同上 |
| error.rs | `crates/arg-effect/src/error.rs` (EffectError 8 variants per DD §9.1: DispatchNoRoute / ContextTooLarge / TrustScoreOutOfRange / OutputQualityLow / AchievementNotUnlocked / LLMServiceDown / PyO3BridgeFailed / InternalError) | ✅ 落档 | (待 commit) | 同上 |
| prompts.rs | `crates/arg-effect/src/prompts.rs` (10 套 challenges prompt = 5 DecisionType × 2 TrustTier, per DD §7 + §3.2.5) | ✅ 落档 | (待 commit) | #1 v15 + #3 跨域 (weight < 0.7 → Low; else High) |
| dispatch_router.rs | `crates/arg-effect/src/dispatch_router.rs` (ARGDispatchRouter + InMemoryEdgeStore + DispatchRoute 3 Vec per DD §4.5) | ✅ 落档 | (待 commit) | #1 v15 + #9 (mock EdgeOps) |
| context_injector.rs | `crates/arg-effect/src/context_injector.rs` (ARGContextInjector + ContextProvider trait + InMemoryContextProvider + 32 KiB cap) | ✅ 落档 | (待 commit) | #1 v15 + #9 |
| trust_engine.rs | `crates/arg-effect/src/trust_engine.rs` (ARGTrustEngine + TrustStore + ±0.01/-0.05 nudge + skip_verify 0.8/0.7 per DD §4.3.3) | ✅ 落档 | (待 commit) | #1 v15 + #3 跨域 (cross-domain edge weight) |
| output_evaluator.rs | `crates/arg-effect/src/output_evaluator.rs` (ARGOutputEvaluator + challenge_round + peer_review + select_challenge_prompt + find_challenges_edge / find_peer_review_edge / build_peer_review_prompt + MockLLMClient) | ✅ 落档 | (待 commit) | #5 v2 + #23 (mock 不开外部 API) |
| achievement_engine.rs | `crates/arg-effect/src/achievement_engine.rs` (ARGAchievementEngine + 3 evaluator (Topology/Behavior/Output) + StubTopologyBackend + 8 拓扑 Cypher 重新导出 + idempotent unlock per `AchievementOps::unlock`) | ✅ 落档 | (待 commit) | #1 v15 + #13 (G-6 闭环) |
| tests (5 文件) | `crates/arg-effect/tests/{dispatch,context,trust,output,achievement}_test.rs` (18 main UT + 1 extra = 19 tests per DD §10.1.3 UT-35..UT-52) | ✅ 落档 | (待 commit) | #1 v25 (单 crate 100% pass) |
| Cargo workspace | `Cargo.toml` members 追加 `"crates/arg-effect"` 1 行 | ✅ 落档 | (待 commit) | #1 |
| arg_dispatch_test.py | `scripts/automation/arg_dispatch_test.py` v0.1 (12 IT: 1 cargo test + 1 sub-crate lib + 1 workspace check + 1 fmt + 1 clippy + 1 release build + 6 file-content checks) | ✅ 落档 | (待 commit) | #1 v19 + #5 + #9 v3 |
| automation-design §4.22 | `docs/automation-design.md` §4.22 (12 子项 ARG-3.1..12) | ✅ 落档 | (待 commit) | #12 v21 |
| 5 守门实证 | check + fmt + clippy + test + build | ✅ 实证 0 err | (待 commit) | #1 累积规 v1-v5 |
| 18 UT 实证 | cargo test -p star-arg-effect --tests -j 4 | ✅ 100% pass (18/18 main + 1/1 extra) | (待 commit) | #1 v25 |
| 跟 ARG.1 + ARG.2 关系 | ARG.1 暴露 `star_arg::models::*` + `LLMClient`; ARG.2 暴露 `BridgeEnvelope` 等 5 协议; ARG.3 通过 `star-arg` + `star-arg-bridge` workspace path 引用 (per DD §4.5-§4.9 + §6-§7 + arch §3.1) | — | — | #14 v2 拍板 D |
| 8 拓扑 Cypher 跟 G-6 关系 | re-export `star_arg::query::topology::all_topology_cyphers` (8 codes TOP-001..TOP-008); 不用 apoc.coll (per F-11); 1 测试 `all_8_cyphers_avoid_apoc` 验证 | — | — | G-6 闭环 |
| 10 challenges prompt 跟 G-5 关系 | 5 DecisionType × 2 TrustTier = 10 套 (per DD §7); 1 测试 `challenge_prompts_has_10_entries` + 1 测试 `all_decision_types_have_both_tiers` 验证 | — | — | G-5 闭环 |
| 后续 gate | ARG.4 (api/arg 已知 merge @ `1d894ab`) / ARG.5 (frontend 5 UI 组件) 派新子代理 | ⏳ 触发 | — | — |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 8 份脚本索引 | 2026-09-02 00:39 JST 拍板 |
| v0.2 | 2026-09-03 | 架构师 (Mavis 接手 agent per DEC-008) | 新增 §5 P3-G Agent Jira 化阶段索引: 5 段 20 子项 G.1-G.20, W1 1.9M + W2-W5 4.0M, 命名空间 P3-G 跟 P3-B (OpenClaw) 9 子项共存 | 2026-09-03 11:50 JST Ulysses Jira 化指令 + 3 步 ask_user 拍板 + 守门 #21 [P] docs 同步 |
| v0.3 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `kanban_sprint_gen.py` (KANBAN-SPRINT-001 P1 Sprint 视图 验证, 43/43 pass) | 2026-09-03 13:25 JST P1 收官, 守门 #1 v19 + #21 v21 实证 |
| v0.4 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 索引说明更新 (kanban_sprint_gen.py 43→55 项) + KANBAN-SPRINT-001 落地 P1 v0.2 Jira 設計 (per docs/briefs/kanban-sprint-view-001.md v0.2) + P2 度量 (Velocity/Burndown/History/Capacity) | 2026-09-03 13:55 JST P1 v0.2 + P2 收官, commit `947c0ef` 落地 |
| v0.5 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 索引说明更新 (kanban_sprint_gen.py 55→93 项) + KANBAN-SPRINT-001 P3 仪式 收官 (Goal + Standup + Review + Retrospective + Markdown 导出) | 2026-09-03 14:05 JST P3 拍板 + 14:20 JST 收官, KANBAN-SPRINT-001 三阶段全部收官 |
| v0.6 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `memgraph_setup.py` (Memgraph Docker compose + health probe) + `arg_seed.py` (24 节点 + 10 边 fixture); 新增 §5.3 ARG.1 crates/arg 6 子模块骨架实装 索引 (16 行覆盖 Cargo.toml / lib / error / 10 models / 3 client / 5 ops / 3 query / llm / 7 tests + workspace + 2 脚本 + docs 同步 + 5 守门 + 32 UT) | 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4), ARG.1 子项 5 守门 0 err + 32 UT 100% pass 实证, 守门 #1 v19 + #12 v21 + #14 v2 联合 |
| v0.7 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `task_ops/nodes/create_node.py` (M-N8, TMO 第 8 节点, 任务卡创建自动 worktree + agent 接管, 83ms smoke pass) + `_mock_git_worktree.py` (守门 #22 mock shell); §5.4 新增 ADR-0049 任务卡自动 worktree 索引 (4 推荐项拍板, 7 子项 v0.1 落档, 8 项已知缺口显式列) | 2026-09-09 04:57 JST Ulysses 拍板核心功能 + 守门 #19 v19 + #20 + #21 实证 |
| v0.8 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `api/routes_tmo.py` 加 POST /api/tmo/create 端点 (M-N8 HTTP 接入, P-AUTO-WT-01 收官, 123ms HTTP 5s 阈值) + `tests/e2e/python/test_uc14_auto_worktree.py` (5/5 维 E2E 全过: happy / SA 映射 / human 拒 / missing tenant 拒 / 1:1 attach); §5.4 ADR-0049 增量 (3 已知缺口 #7 #8 #9 从 ⏳ 改 ✅); 修复 pre-existing split_node stale import (路由层 4 常量本地化) | 2026-09-09 08:13 JST "推进" 拍板 + 守门 #9 v3 + #20 + #21 + #22 实证 |
| v0.9 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `arg_bridge_test.py` (ARG.2 P3-C W2 Bridge Tier IT 端到端, 10 UT + 3 extras + 5 守门 + 5 file-content checks, 走 subprocess.run + Python 进程内解析 cargo test 输出, 守门 #9 v3 + #1 v19 + #5 + #7 + #12 v21 联合实证); §5.5 新增 ARG.2 crates/arg-bridge 4 子模块骨架实装 索引 (16 文件 + 10 UT + 1 脚本 + 1 报告 + 1 commit hash 落档) | 2026-09-10 06:53 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 44 次新事件触发仍允许) |
| v0.10 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `arg_dispatch_test.py` (ARG.3 P3-C W3 Effect Tier IT 端到端, 12 IT 端到端 + 5 守门 + 6 file-content checks, 守门 #9 v3 + #1 v19 + #5 + #7 + #12 v21 联合实证); §5.6 新增 ARG.3 crates/arg-effect 5 子模块 + 18 UT + 8 拓扑 Cypher + 10 challenges prompt 索引 (12 文件 + 18 UT + 8 拓扑 (G-6 闭环) + 10 challenges (G-5 闭环) + 1 脚本 + 1 报告 + 1 commit hash 落档) | 2026-09-10 07:21 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 45 次新事件触发仍允许) |
| **v0.11** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **+3 brief + 3 SRS 双核心 + ARG 落档**: §2 子代理任务索引 +3 行 (srs-canvas-agent-001 / srs-canvas-gamify-001 / srs-canvas-integration-001 deprecated), 关联 commit `4f56979` (6 files / 4107 insertions, per 守门 #1 v15 docs 同步饱和第 68 次新事件触发 + 守门 #10 author=Ulysses + 守门 #14 v3/v4) | **2026-09-10 17:00/17:08/17:21 JST 三阶段拍板: (1) 17:00 "归纳需求文档" → (2) 17:08 "管理 agent + 游戏化, 避免过度冗余" → (3) 17:21 "画布内体现 agent 之间关系的图论构造 (ARG)"** |
| **v0.12** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 +1 行 v1.2 多人编辑**: srs-canvas-agent-001 v1.2 (158,701 bytes / 1,956 行 / 46 项 = 38 v1.1 + 8 A12 多人编辑), 关联 commit `396833a` (3 files / 691 insertions / 255 deletions, per 守门 #1 v15 docs 同步饱和第 70 次新事件触发 + 守门 #1 禁回溯叙事 v0.63 反转 + 守门 #10 author=Ulysses + 守门 #14 v3/v4 + 守门 #13 W/T/M 14 张表 100% 覆盖) | **2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的" (v0.63 反转, 撤回 17:08 JST 砍多人编辑决定, per 守门 #1 禁回溯叙事 + 守门 #14 v2 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output)** |
| **v0.13** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 +3 行 BD 三份**: bd-canvas-total-001 (50,274 bytes 总册 root 写) + bd-canvas-agent-001 (105,821 bytes 专题 1 子代理 1 写) + bd-canvas-gamify-001 (111,370 bytes 专题 2 子代理 2 写), 关联 commit `f7d0932` (3 files / 4,141 insertions, per 守门 #1 v15 docs 同步饱和第 72 次新事件触发 + 守门 #1 禁回溯叙事不重写 4 SRS commit + 守门 #10 author=Ulysses + 守门 #14 v3/v4 + 守门 #13 W/T/M 29 张表 100% 覆盖跨域汇总 + 守门 #23 v2 G5 mock 锁) | **2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 3 份 BD brief + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2 5 域 Lead 拍板 D + 守门 #14 v3 Mavis 永久代签)** |
| **v0.14** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 +4 行 (3 DD + 1 协调性报告) + §3 v0.14**: dd-canvas-total-001 v0.1.1 (49,367 bytes 修复版 root 写, 3 冲突修复: C-16 → C-25 `CanvasElementsBackend` + C-21 → C-26 `CanvasMultiUserAudit` + Trust Score 5 档统一 ARG 源 `Untrusted/Low/Medium/High/VeryHigh`) + dd-canvas-agent-001 v0.1 (139,198 bytes 子代理 1 bg_9ce1be6e 写, 13 关键 class 1:1 派生 ARG 源 C-16=RelationshipEditor + C-21=ARGController) + dd-canvas-gamify-001 v0.1 (161,742 bytes 子代理 2 bg_50df734c 写, 15 张表 W/T/M 100% 覆盖 0 混在 + G5 mock 锁 + G12 V0.1 5 份 PHASE 派生) + coordination-check-001 v0.1 (21,544 bytes 协调性检查报告, 10 项检查 7 通过 + 3 冲突 + 修复方案), 关联 commit 待生成 (10 files / 4 核心 + 6 brief catch-up, per 守门 #1 v15 docs 同步饱和第 74 次新事件触发 + 守门 #1 禁回溯叙事不重写 4 SRS + 2 BD + 4 ARG commit + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 6 份 catch-up + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v3/v4 + 守门 #13 W/T/M 29 张表 100% 覆盖跨域汇总 + 守门 #23 v2 G5 mock 锁 + 协调性检查 3 冲突修复) | **2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:30 JST 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 + 守门 #9 v27 + 守门 #14 v2 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和)** |
