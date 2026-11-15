# REVIEW-2026-11-14-IMPLEMENTATION — 2026-11-14 实现逐条审查记录

> 审查日期: 2026-11-15 · 审查人: Ulysses (Lane A 文档基线) · 基线: dev @ cfbfe4c5 (worktree agent/deepseek/ulys-docs-baseline)
> 范围: 2026-11-14 当日 dev 分支 39 条提交 (`git log --since=2026-11-14 --until=2026-11-15`, 只读) 与 docs/architecture/2026-11-14-*/ 四份文档的声称
> 方法: 只读 git (log/show/grep) + 工作树文件存在性/内容核对。**未复跑构建与测试** (任务约束: 不跑 npm/cargo), 测试数字以 commit 声称 + 仓内测试函数计数 (`git grep -E '#\[(tokio::)?test\]' <sha> -- <crate>`) 交叉核对。

## §1 审查项 (按来源文档)

### 1.1 WBS-BACKEND-SUPPORT-001 (W1–W5)

| 项 | 声称内容 | 实际文件与 commit | 验证命令与结果 | 结论 |
|---|---|---|---|---|
| W1 12 组 API 门面 | 8085313b: 12 组 route handlers, 404→200, 与 MSW 同数据源 | frontend/src/app/api/{agents, annotations, api-keys, arg, cli-profiles, comments, design-artifacts, incidents, notifications, rbac, validation, workspaces} 共 22 个 route.ts; commit 8085313b 22 files +466/-7 | `git show --stat 8085313b`; `ls frontend/src/app/api` (14 目录在树) | ✅ 已闭环 (WBS 文档内 W1 状态仍 🟡 — 文档滞后, 见 §2) |
| W2 4 crate 门面 + 本地联通 | 4e1565d1: /collab/session+annotations、/git/conflicts、/argo/nodes、/sandtree/resources, 234 单元测试全绿; 0f69810d: 8081 proxy 401 语义正常 | crates/star-api-rest/src/lib.rs:92–100 与 247–255 注册全部 5 端点; routes/integrations.rs `git_conflicts` 调 `star_git_core::repo::conflict::conflict_set`; frontend/src/lib/auth.ts 含 NETWORK_UNREACHABLE 降级; next.config.ts:50 `STAR_API_BACKEND_ORIGIN ?? http://127.0.0.1:8081` | `grep -n 'collab/session' crates/star-api-rest/src/lib.rs` (2 处注册); 测试计数: 4e1565d1 树内 `#[test]/#[tokio::test]` = src 218 + tests 211 = **429 处** (含 15 处 `#[ignore]` PG 验收) — 与声称 234 对不上, 未复跑 | ✅ 已闭环 (测试数字待复核, 见 §2 部分项 2) |
| W3 画布 Miro 工具集 | 0f69810d: 橡皮擦/直线/注释/5 形状库/10 快捷键/双击编辑 | frontend/src/components/CanvasView.tsx (+289 行), canvas/[id]/page.tsx | `git show --stat 0f69810d` | ✅ 已闭环 |
| W4 主题 token 化 | 74262b8d: 131+ 处 cel-* 硬值 token 化, 17 文件, 6 主题跟随 | 17 files +196/-201 (AppMatrixDrawer/AgentCanvasView/AgentFilter/agents 页/RelationshipEditor 等) | `git show --stat 74262b8d` | ✅ 已闭环 |
| W5 时间线 Gantt | 0f69810d: kanban 时间线 tab 挂真实 GanttChart, 去 /projects 旧页跳转 | frontend/src/components/gantt/GanttView.tsx (+100) + gantt/index.ts + kanban/page.tsx | `git show --stat 0f69810d`; 文件在树 | ✅ 已闭环 |
| §6 验收总闸 | tsc 0 / vitest 全绿 / curl 14 端点 2xx / cargo fmt --all --check | 无独立证据文件, 仅 commit 消息声称 | 未复跑 (约束) | 🟡 部分 (依赖声称) |

### 1.2 INTEGRATION-MAP-GITHUB-REPOS-001

| 项 | 声称内容 | 实际文件与 commit | 验证命令与结果 | 结论 |
|---|---|---|---|---|
| GemArc → /prototype + MCP | 23d60e75: /api/gemarc 双通道代理 (本地 full + 远端 gem-arc.vercel.app) + /prototype 真实 1D-5D 流水线 | frontend/src/app/api/gemarc/route.ts (95 行); prototype/page.tsx 重写 | `ls` + `git show --stat 23d60e75` | ✅ 已闭环 (本地 E:\GemArc 服务状态在仓外, 无法从仓内验证) |
| MAGOS → LANE 树 + worktree 面板 | 23d60e75: /api/magos 只读代理 + MagosLanePanel (8 条真实登记); b4f19d1d: POST /api/magos plan+protect-show 审批流, 写类命令 403 | frontend/src/app/api/magos/route.ts (51+32 行); components/magos/MagosLanePanel.tsx; 头部注释: magos_guard.py 只读子命令 list/plan/doctor/version/protect-show | `ls` + 文件头核对 | ✅ 已闭环 |
| GitGit → star-git-core | 713b12d2: repo/conflict.rs (2498 行); b4f19d1d: repo/refs.rs (162 行) + remote_sync.rs (386 行, sync feature+tokio), 45 测试全绿 | crates/star-git-core/src/repo/{conflict,refs}.rs + remote_sync.rs; 源文件头授权注记 (D:\GitGit @ 69770e6, MIT OR Apache-2.0) | `git grep -E '#\[(tokio::)?test\]' b4f19d1d -- crates/star-git-core` = **45 ✓ 与声称一致** | ✅ 已闭环 |
| SandTree → 沙盒 | 文档落点写 crates/sandboxd; 实际 713b12d2 落 **crates/sandtree-core** (graph/observation/model 四件套, Apache-2.0 NOTICE 保留) | crates/sandtree-core/{NOTICE, src/graph.rs, src/observation.rs, src/model/*} | `ls`; NOTICE 存在 ✓ | ✅ 已闭环 (落点与文档记载不同, 记偏差) |
| Argo → 实体机组/agents | 713b12d2: crates/argo-core (model.rs 391 行); 4e1565d1: /argo/nodes 门面 + /agents 页 Argo CLI 徽章 (失败回退 mock) | crates/argo-core; 授权头 (UlyssesLeoLee/Argo @ 2941596, 权利人 MIT 拍板) | `head -8 crates/argo-core/src/lib.rs` ✓ | ✅ 已闭环 (模型层; 真实多机接入走本地 argo-runtime, 未做) |
| IDE1.0 nema-codec | 23d60e75: nema-codec 移植 (19 测试全过); 1787896e: nema-codec-wasm glue + wasm 产物入库, 前端 wasm-first 解析失败回退 JS | crates/nema-codec (lib.rs 3846 行 + interpreter.rs 733 行); crates/nema-codec-wasm; nemaStore 异步化 | `git grep -E '#\[(tokio::)?test\]' 23d60e75 -- crates/nema-codec` = **19 ✓** | ✅ 已闭环 |
| IDE1.0 ide-agent-router | 文档: collaboration/coordinator/nema_pipeline 下一批 | 713b12d2 ide-collab-core: identity/parser/patch/coordinator/collaboration 5 模块 ✓; nema_pipeline/reviewer/tenant/file_lock/history 未移植 | `ls crates/ide-collab-core/src` (5 模块) | 🟡 部分 |
| Physis (文档 §1 列为待设计) | 45eb081f: physis-wasm (146 测试, 51KB wasm) + 第 7 主题 'Physis 3D 物理'; 7e85d328/5ef9bae8 主题深化 | crates/physis-wasm | `git grep -E '#\[(tokio::)?test\]' 45eb081f -- crates/physis-wasm` = **146 ✓** | ✅ 已闭环 (文档未回写, 滞后) |
| 其余仓库 (IM1.0/CATs/XINGTIAN/Ada/Rigin/PANTHEON/ICHOR…) | 文档: 待设计/后续轮次 | 未检出/未开始 | — | ❌ 未做 (符合文档计划) |

### 1.3 REUSE-MAP-COLLAB-SANDBOX-001

| 项 | 声称内容 | 实际文件与 commit | 验证命令与结果 | 结论 |
|---|---|---|---|---|
| 文档 v0.2 状态「未开始移植」 | 同日 713b12d2 已移植 4 crates (125 测试全绿声称) | crates/{ide-collab-core, star-git-core, argo-core, sandtree-core} | `git grep -E '#\[(tokio::)?test\]' 713b12d2 -- crates/ide-collab-core crates/argo-core crates/sandtree-core crates/star-git-core` = **125 ✓ 与声称一致** | 文档滞后 (实现超出文档自述) |
| §4 许可处理 | 移植注明来源 + SandTree 保留 Apache-2.0 NOTICE | 4 crates 源文件头均有来源/授权注记; sandtree-core/NOTICE 存在 | `head -8` 逐个核对 ✓ | ✅ 已闭环 |
| 协同页 5 步流水线/合并/验证 (nema_pipeline) | REUSE-MAP §2 列为适配项 | 未移植 | — | ❌ 未做 |
| 沙盒页实体机/Docker/限路径同步/快照 | REUSE-MAP §3 全量能力 | /sandbox 页已接 sandtree/argo API (观测四级显示, 4e1565d1) ✓; 其余未 | `ls frontend/src/app/(app)/sandbox/page.tsx` ✓ | 🟡 部分 |

### 1.4 OPS-CONSOLE-INDEPENDENT-RELEASE-001

| 项 | 声称内容 | 实际文件与 commit | 验证命令与结果 | 结论 |
|---|---|---|---|---|
| 运维控制台 + 按 App 单独发布 | 纯设计基线 (🟡 Draft, 自述「待实装」), 无实现声称 | 引用文件核对全部存在: deploy/gitops/kargo/kargo-dev-staging.yaml ✓, deploy/gitops/argocd/argocd-applications.yaml ✓, .github/workflows/publish-star-canvas-services.yml ✓, docs/architecture/2026-08-26-upgrade/spec/ops/01-ops-requirements-spec.md ✓, ADR 0054-cd-delivery-stack-argocd-kargo-rollouts.md ✓ | `ls` 5 项全 OK | 设计基线 (不适用实现判定); §1.3 旧规格 superseded 注记「另行提交」未落地 → ❌ 未做 (1 项) |

### 1.5 窗口内其余提交抽查 (无独立 WBS 的)

| 域 | commit | 抽查结果 |
|---|---|---|
| 聊天 | e610c705 / ea84a9e1 / 7e85d328 | frontend/src/components/chat/RightChatRail.tsx 在树 ✓ |
| 任务卡 | e610c705 / 5526c673 / e5e17153 | /work-item/[id]/page.tsx ✓, components/board/WorkItemDetailDrawer.tsx ✓, components/sprint/SprintBoardView.tsx ✓ |
| 日历 | 54d29e16 | components/calendar/CalendarView.tsx ✓ |
| 移动重定向 | 4416ca32 | proxy.ts 含 STAR_FLUTTER_WEB_URL 检测 ✓ |
| 模块矩阵 | 713b12d2 | AppMatrixDrawer 6 模块卡 ✓ |
| 6 页 UI / 侧栏 / 主题修复 | 7471e77d → 4c6bc49c 系列 | 文件树核对 ✓ (kanban/sandbox/prototype/ops 页均在) |
| Lane Phase 1 + 布局整合 | 462260e8 + cfbfe4c5 | ✅ 已闭环, 状态更新见 WBS-LANE-WORKSPACE-001 修订 v1.2 |
| kanban 预览数据模式标识 (Phase 3.2) | cf48593b | ⚠️ 在 fork 点 (cfbfe4c5) 之后的 dev 新提交, **本 worktree 未含** — 见 §3 |

## §2 结论表

| 结论 | 数量 | 项 |
|---|---|---|
| ✅ 已闭环 | 13 | 1. W1 12 组 API 门面; 2. W2 4 crate 门面端点 + 本地联通; 3. W3 画布 Miro 工具集; 4. W4 主题全覆盖; 5. W5 时间线 Gantt; 6. GemArc 集成; 7. MAGOS 集成 + Plan 审批流; 8. GitGit → star-git-core (45 测试计数一致); 9. SandTree → sandtree-core (NOTICE+授权头); 10. Argo → argo-core; 11. nema-codec(+wasm) (19 测试计数一致); 12. Physis → physis-wasm (146 测试计数一致); 13. 窗口内其余 UI 提交抽查 (聊天/任务卡/日历/移动重定向/矩阵/6 页/Lane Phase 1+2.1/2.2) |
| 🟡 部分 | 7 | 1. WBS §6 验收总闸未复跑 (依赖 commit 声称); 2. star-api-rest 测试计数 234 vs 仓内实测 429 处测试函数 (15 ignored) 对不上, 待复跑确认; 3. ide-agent-router 其余模块 (nema_pipeline/reviewer/tenant/file_lock/history) 未移植; 4. 沙盒页全量能力 (实体机/Docker/限路径同步/快照) 未落地, 仅 sandtree/argo 数据面; 5. WBS-BACKEND-SUPPORT-001 文档滞后 (W1/W3/W4/W5 已实装未打钩); 6. REUSE-MAP 文档滞后 (自述未移植, 实际已移植 4 crates); 7. INTEGRATION-MAP 文档滞后 (Physis 已移植未回写) |
| ❌ 未做 | 2 | 1. OPS-CONSOLE 实装 + 旧规格 superseded 注记; 2. INTEGRATION-MAP 其余仓库集成 (IM1.0/CATs/XINGTIAN/Ada/Rigin/PANTHEON 等) |

## §3 需要 parent 处理的事项

1. **dev 已前移**: cf48593b (feat(kanban): 预览数据模式标识, Phase 3.2) 在 fork 点 cfbfe4c5 之后, 本 worktree 未含 — 合并时注意。
2. **测试数字复核**: star-api-rest 声称 234 单元测试 vs 仓内 429 处测试函数 (含 15 处 #[ignore] PG 验收) — 建议 parent 复跑 `cargo test -p star-api-rest` 确认。
3. **文档滞后三处** (WBS-BACKEND-SUPPORT 打钩 / REUSE-MAP v0.2 状态 / INTEGRATION-MAP Physis 回写) — 不在本任务三个文件范围内, 建议后续 lane 处理。
4. **cgg stamp 未跑** (任务约束): 新文档 REVIEW-2026-11-14-IMPLEMENTATION.md 未带 @cypher 头; WBS-LANE-WORKSPACE-001.md 的 @cypher 头 sha256 保持全零占位 — 如需 stamp 请 parent 决定。
