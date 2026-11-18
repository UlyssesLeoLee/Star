# PHASE-DELIVERY-SYNC-20261118 — 工作区核清 + 交付基线刷新

**结论：发布保持 NO-GO（v1.0.6）。** 本报告以远端 `dev@94505e75e314bab3ac5ce50cb43e141ee578c84c`（`git ls-remote origin dev` 实测，本地 dev 与远端一致）为基线，完成四件事：
1. 核清工作区状态标记与 diff 文件的差异及 CRLF 影响（起始 177 条 = 174 M + 3 ??；87 真 diff + 87 CRLF-only；**后续已完成 CRLF 归一化，当前 91 条 = 88 M + 3 ??，status 与 diff 一致**，见 §1.3）；
2. 登记新 Lane 架构与近期已提交实现；
3. 登记未跟踪的 Feature 审计文档（14 份 F 系列）；
4. 登记未关闭门禁（稳定发行 W0–W9 + F 系列缺口 + 本工作区新增未验证项）。

本阶段起始**未做任何整体暂存、清理、提交、push、tag 或部署**；后续残留待办处理阶段按用户指令完成了 CRLF 归一化、空文件删除、Guardian 断链修复与按领域拆分提交（见 §1.3/§4.2/§4.3），全程逐文件操作，未用 `git add -A`/`stash`/`clean`。

---

## §1 工作区核清（174 状态标记 vs 87 diff 文件）

实测命令：`git status --porcelain=v1`（177 条）、`git diff --name-only`（87 个）、逐文件 `git diff -U0` 全量落盘解析（87/87 全覆盖，无遗漏）。

### 1.1 总账

| 类别 | 数量 | 说明 |
|---|---|---|
| 状态标记总数 | 177 | 174 个 ` M` + 3 个 `??` |
| 有内容 diff 的文件 | 87 | `git diff --name-only` 全集 |
| **CRLF-only 假修改** | **87** | status 标 M 但 `git diff` 为空；`.gitattributes` 为 `text eol=lf` 而工作区文件为 CRLF，`core.autocrlf=true` 下的行尾噪声，**无内容变化** |
| 未跟踪条目 | 3 | 展开后 16 个文件（见 §1.4） |

核对恒等式：174 = 87（真 diff）+ 87（CRLF-only）✅；87 diff = 70（纯 Cypher stamp）+ 11（实质代码）+ 6（实质文档）✅。

> 计数时点说明：177/174/3 为核清开始时的快照。**后续处理**：删除 0 字节 `Collaboration`、87 个 CRLF-only 文件字节级 `\r\n`→`\n` 归一化、3 个 lib Cypher 头重写（断链修复）后，当前实测 **91 条 = 88 M + 3 ??，`git diff --name-only` = 88，CRLF-only 假修改 = 0**，status 与 diff 完全一致。

### 1.2 87 个 diff 文件的三层分解

**A. 纯 Cypher stamp（70 个）** — diff 仅 `source_sha256` 头行刷新（含把占位符 `2026-11-11-*-v1`、全零 digest 换成真实 sha256），无任何代码/文档内容变化。Guardian 校验（`cgg_guard.py --json`，修复后复跑）：**70/70 全绿，0 错误**。原报 17 处结构断链已查明根因并修复：① `cgg_guard.py` 的 `norm_rel()` 用 `lstrip("./")` 把 `.github/...` 误剥成 `github/...`（脚本 bug，非仓库问题）——已改为 `_strip_dot_prefix()` 只剥 `./` 前缀；② workflowLib/auth/documents 三个 lib 的旧模块式 Symbol id 目标不存在——已按实际导出重写为路径式 id（`仓库相对路径::导出名`）并 `--stamp` PASS。

**B. 实质代码改动（11 个）** — 按领域分桶：

| 领域 | 文件 | 规模 | 复用关系与验证证据 |
|---|---|---|---|
| API 路由挂载（F15 G-01 修复） | `crates/star-api-rest/src/lib.rs`（+66 实质行）、`src/main.rs`（+5） | 新增 `build_supplemental_routers()`：8 模块（memory/agents/runtime/terminal/ide/im/notifications/comments）单一来源，legacy `build_router` 与生产 `main.rs` 共用 `.merge()`，杜绝两处漂移 | `cargo check -p star-api-rest` ✅；`cargo fmt --all --check` ✅；新增 `tests/production_router.rs`（oneshot 8 端点非 404）首跑绿 ✅；`notifications --no-run` 编译超时（环境慢，非代码缺陷）未验证 ⚠️ |
| 3D 特效纯 3D 化（8 组件 + 1 测试） | `frontend/src/components/effects/{Cel3DUI,TacticalCore3D,AnimeCelShaderCanvas,PhysisBackdrop,GasParticlesField,CelCard3D,PhysisHud}.tsx` + `app/automation-debug/components/AnimeCore3D.tsx` + `effects/__tests__/TacticalCore3D.test.tsx` | 合计 4,992 行（Cel3DUI 1582 / AnimeCelShaderCanvas 1051 / TacticalCore3D 791 / PhysisBackdrop 587 / AnimeCore3D 315 / GasParticlesField 294 / CelCard3D 215 / PhysisHud 157） | `npx tsc --noEmit` 0 错误 ✅；vitest 3 文件 12/12 绿 ✅；颜色全部经 `resolveCelTokenPalette`/`readThemeRgb` 读主题 token，7 主题色调零改动（静态保证）；**多主题浏览器截图对比未做** ⚠️ |

**C. 实质文档改动（6 个）** — 均为既有审计文档的行号/基线/事实修正，非新设计：
- `docs/architecture/2026-09-30-upgrade/star-desktop-p5-frontend-impl-plan.md`：核对基线改 `1470c33c`、KanbanBoard 行数 525→580、/board 307 重定向事实、seed 兜底描述修正；
- `star-desktop-p7-e2e-performance-guide.md`：WASM bundle 从"~6MB"改为"未打包未实测"（诚实标注）；
- `file-session-management/01-requirements.md`：+1 行 L4b 右侧聊天栏层级（与 RightChatRail.tsx:96 一致）；
- `PHASE-WLD7-COLLAB-SPEC-CITATION-LINK-FIX-20261107.md`：Owner migrations 缺口描述细化；
- `SRS-AUTOMATION-SCHEDULE-API-001.md`：transport/acceptance 措辞修正；
- `WBS-W14-DESKTOP-PERFORMANCE-AUDIT-20261025.md`：引用行号随代码漂移更新。

### 1.3 CRLF 影响判定

87 个 CRLF-only 文件分布：crates 其他（运行时/领域）76、star-api-rest 9、deploy 1（`deploy/k3s-local/build-and-deploy.sh`）、scripts 1（`scripts/build-desktop.sh`）。判定依据：`git diff -- <file>` 为空 + `git check-attr` 显示 `text: set, eol: lf`。**处置（已完成）**：逐文件验证（diff 为空 + 工作区含 `\r\n`）后做字节级 `\r\n`→`\n` 归一化写回，87/87 全部消除；归一化后 `git status` 与 `git diff` 计数一致（88 M = 88 diff），不再有任何行尾噪声。这些文件不携带内容变化，归一化结果随各自领域批次提交。

### 1.4 未跟踪文件（3 条目 → 16 文件）

| 条目 | 内容 | 处置建议 |
|---|---|---|
| `Collaboration` | **0 字节空文件**（2026-11-16 13:25 创建，疑似误触/占位） | **已删除**（残留待办阶段，`wc -c` = 0 确认后 `rm`） |
| `crates/star-api-rest/tests/production_router.rs` | F15 G-01 的 oneshot 集成测试（已跑绿） | 应随 F15 G-01 修复一起提交 |
| `docs/features/F*.md` × 14 | Feature 审计文档（见 §3） | 应作为文档批次提交 |

---

## §2 新 Lane 架构与近期实现登记（基线 dev@94505e75）

### 2.1 Lane 架构（已提交部分）

权威计划：[WBS-LANE-WORKSPACE-001](../architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md)（v1.5，8 阶段）。

- **Phase 1 ✅**：Lane 命名统一（Engineering Run → Lane 显示层）、store `lanes` 集合 + `Lane` 类型、`Worktree.lane_id?`、navStore 四层选择（selectedProjectId/LaneId/WorktreeId/Branch）+ 深链、ProjectRunTree 四层渲染、授权上下文统一。
- **Phase 2 大部分 ✅**：Lane 级中央应用页签（Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins）、Group Workspace 双页签、任务卡统一可关闭子窗口（3cc5b6b7）。
- **Phase 3/4 部分 ✅**：kanban 预览数据标识 + 未接入禁用写入、Canvas ?worktree= 上下文条 + 四态 + 真实 401。
- **Phase 5 大部分 ✅**：chatScope + RightChatRail 范围切换、GLOBAL 授权目标下拉 + 失败保草稿、未接入徽标。
- **关键决策**：内部 Run 命名（engineering_run 路由/ID）保留不动，前端 Lane 显示层 ↔ 内部契约兼容映射；seed/live 双模式保留但 UI 必须标识数据来源。
- **最新提交**：`94505e75` feat(frontend): LANE 专属设置页面 `/lane/[laneId]/settings`。

### 2.2 近期已提交实现（dev 历史，94505e75 往前）

| commit | 内容 |
|---|---|
| `94505e75` | LANE 专属设置页面 |
| `ec1de33b` | 主题风格统一 + Three.js 内容增加（2026-11-17） |
| `e3f3601f` | chrono/chrono-tz unmaintained 忽略项登记，修 License Gate 红 |
| `81460884` | 全 workspace rustdoc unclosed HTML tag 警告清零（31 处/17 文件） |
| `1643559f` | IDE1.0 网页版内核引入 + nema 多 agent 共同编辑 + jira 模块去重（3 子代理并行） |
| `b04c15a0` | ide-kernel 移植 + /api/v1/ide/* + 前端真实内核工作区 |
| `bb6b860d` | tasks 去重 jira 性质模块 + parent_id 层级/看板筛选/Sprint 项目范围 |
| `23274bc0` | nema 文件即任务式多 agent 共同编辑（.nema 侧车 strict v2 + 任务回写） |
| `63848d95` | 通知/评论/Agent 中心接真实 owner API |

### 2.3 按独立领域的脏改动审查（用户第 3 步）

全量 `git diff --numstat` 按领域分桶（87 个 diff 文件全覆盖）：

| 领域桶 | 文件数 | +/- | 审查结论 |
|---|---|---|---|
| A 3D-effects（本 session） | 9 | +2674/−645 | 实质改动：8 组件纯 3D 化 + 1 测试 title 修正；tsc 0 / vitest 12/12；复用 `themeTokens` 统一取色，无新增依赖。当前行数：Cel3DUI 1582、TacticalCore3D 791、AnimeCelShaderCanvas 1051、PhysisBackdrop 587、GasParticlesField 294、CelCard3D 215、PhysisHud 157、AnimeCore3D 315 |
| F API 路由/运行时 | 15 | +73/−28 | 其中 lib.rs/main.rs 为 F15 G-01 实质修复（复用既有 8 个 `router()`，单一来源 `build_supplemental_routers()`，legacy 与生产共用）；其余 13 个为纯 stamp |
| J 文档 | 6 | +21/−20 | 全部为行号/基线/事实修正（§1.2-C），无新设计 |
| E API 认证/OAuth | 8 | +8/−8 | **全部纯 stamp**（含 device_enrollment.rs），无代码逻辑变化 |
| G 运行时/调度/持久化 | 14 | +14/−14 | **全部纯 stamp**（含 run-owner-runtime/schedule_admission.rs） |
| D 任务与 UI | 15 | +15/−15 | **全部纯 stamp**（含 workflowLib.ts） |
| H 其他 crate | 7 | +7/−7 | 全部纯 stamp（domain-agent/scm/git-core/agent-bridge） |
| C canvas-game | 5 | +5/−5 | 全部纯 stamp |
| B i18n | 3 | +3/−3 | 全部纯 stamp |
| I 部署/CI 脚本 | 5 | +5/−5 | 全部纯 stamp（scripts/ci 5：apply_rulesets.sh、dispatch_release_workflows.sh、gate.py、install_pinned_binary.sh、release_preflight.py）；其中前两者的 Cypher 引用 `github/...` 缺 `.github` 前缀，属既有断链（§4.2-3） |

**NEMA**：工作区脏文件中**无任何 nema 相关改动**——nema 实现（23274bc0/1643559f）已全部提交进 dev，不在本次审查范围。
**部署脚本**：`deploy/k3s-local/build-and-deploy.sh` 与 `scripts/build-desktop.sh` 仅 CRLF-only（无内容变化）；scripts/ci 5 个脚本仅 stamp 刷新。
**复用关系结论**：唯一的实质代码复用点是 F15 G-01（生产与 legacy 共用同一组合函数）与 3D 组件共用 cel shader/主题 token；其余 70 个 stamp 文件不涉及逻辑复用变化。

### 2.4 工作区未提交实现（本 session，见 §1.2-B）

F15 G-01 生产路由挂载修复 + 8 组件纯 3D 人格化。**均未提交**，验证证据与残留缺口见 §1.2-B 表。

---

## §3 未跟踪 Feature 审计登记（docs/features/，14 份）

基线：14 份文档全部自带核对基线 `1643559f`（2026-11-16/17 核对，grep 实测 14/14）。F05/F09/F11 编号不存在（历史裁剪）。

| 文档 | 状态摘要（一句话） | 缺口规模 |
|---|---|---|
| F01 导航·Lane | 🟡 骨架已落地；Lane 创建/授权、宿主目录点选写入、面包屑绑定未实现 | §6 表若干 |
| F02 分支工作区 | 外壳已有；应用页签三套并存、Lane 无写入路径、聊天未接通 | §6 |
| F03 Worktree 领域 | 内存聚合根+17 状态已实现；REST/MCP 只暴露创建/读取 | **G1–G26（26 行缺口表）**：状态名 6 项互异、abandon 绕过迁移表、Completion Gate/Isolation/Reconciliation/权限全缺 |
| F04 任务看板 | /kanban 本地 store 不写服务端；RunTaskCard PG+REST 已实现但前端不在 /kanban 调用；两套状态模型不统一 | §6 |
| F06 统一画布 | 群组 live 服务端权威已实现；readOnly 门控矛盾、撤销/导出/CRDT 缺失、双通道并存 | **G-01–G-18（18 行）** |
| F07 Worktree 关系图 | 领域类型完整但运行时无数据通道；视图未挂载；BFF 桩 | §6 |
| F08 Agent 关系图 | 前端渲染代码在但数据源默认空；ARG REST 未挂载；后端 stub | §6 |
| F10 游戏化画布 | 弹幕逻辑+17 测试绿但未挂载任何页面（入口 23d60e75 移除）；G1–G12、服务端游戏模型、3渲2 渲染均未实现；依赖的 V0.1 组件与 star-game crate 已删除 | §6（6 大项） |
| F12 文件会话 | **零实现**；归属未裁决 | §6 |
| F13 沙盒 | sandboxd gRPC/池/审计已实现；网络/文件系统两维未接入；/sandbox 页常量演示 | §6 |
| F14 原型 | 🟡 GemArc 流水线已接；预览/导出/版本记录静态 | §6 |
| F15 收件箱·通知·IM | 前端+REST 契约在但**生产不挂载路由**（G-01，本工作区已修未提交）；无持久化/重试/渠道 | **G-01–G-17（17 行）** |
| F16 分析 | 页面接线全但数据来自 store 派生；报告 REST/持久化未实现 | §6 |
| F17 设置 | 双面板并行；MCP/Skills/Plugins 仅 localStorage；仅 Hook 策略走服务端 | §6 |

**与任务卡关联需求的衔接**（用户最新需求：任务卡可关联虚拟机/沙盒/测试 Build）：F04（任务卡两套模型）+ F13（沙盒）+ multipass-adapter/domain-local-runtime（VM）是天然锚点；该需求尚未开工，登记为后续独立工作项，不在本阶段实施。

---

## §4 未关闭门禁登记（NO-GO 依据）

### 4.1 稳定发行 W0–W9（权威：[WBS-STAR-STABLE-RELEASE-20261031](../implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md)，最后同步 2026-11-08）

| 门禁 | 状态 | 核心缺口（当前仍无证据） |
|---|---|---|
| W0 发行矩阵 | PARTIAL | 无真实 v1.0.6 tag build、hosted CI、制品、Vercel production、image digest、rollback |
| W1 法律身份 | **BLOCKED** | 根 `Cargo.toml:214` 仍 `license = "Apache-2.0"` vs 用户 MIT 决策（`LICENSE` 正文为 MIT）；无 NOTICE；第三方闭包未审 |
| W2 锁与依赖闭包 | OPEN/BLOCKED | toolchain/Cargo.toml 基线已为 1.99.1，AGENTS.md:38 亦已写 1.99.1（2026-11-08 登记的 1.99.0 漂移已消除）；但 locked offline metadata、目标依赖闭包、MSRV 验收、唯一 JS lockfile 仍未跑 |
| W3 BOM/签名/provenance | OPEN/BLOCKED | 真实制品的 SBOM/attestation/consumer 验证未运行 |
| W4 强制 CI | OPEN/BLOCKED | cargo-audit 实际执行、hosted runs、required-check 保护证据缺 |
| W5 桌面闭环 | OPEN | Node 24 hosted、原生安装包、签名/公证、安装/升级/回滚、CSP 缺 |
| W6 Web/Vercel | ADAPT/OPEN | 无 Vercel production deployment、Neon/Owner runtime、协作 consumer、rollback；npm/pnpm 双 lockfile 并存 |
| W7 Canvas 镜像 | OPEN/BLOCKED | 静态过滤已审；真实 BuildKit archive/镜像层/digest 未验 |
| W8 Multipass/K3s 测试基础设施（稳定 WBS） | OPEN | 本机 Multipass 1.16.4 < 脚本要求 1.17/HCS；双实例容量、K3s ready/升级/恢复未实测 |
| W9 RC 验证与审批门 | BLOCKED | 依赖 W0–W8 全部完成 |

**事件传输 W8（另一 WBS：[WORKTREE-GROUP-NEXT-WBS-2026-10-18](../implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md) §23.23）**：状态 **OPEN / PARTIAL**——2026-11-09 disposable PostgreSQL 18.6 上 8 个 W8 套件/断言首次实跑全绿（[PHASE-W8-EVENT-TRANSPORT-ACCEPTANCE-20261109](PHASE-W8-EVENT-TRANSPORT-ACCEPTANCE-20261109.md)，基线 dev@97ae06bc），§23.22 登记的 P1 `pull_one` 直接 runtime grant 已撤销并经双向变异验证，quarantine exact replay 二义性与生产编译已修。**mTLS、真实网络 commit-before-ACK、取消/撤权恢复、durable quarantine 留存、目标库部署与性能证据仍未跑**，不构成发行门禁关闭。

**用户点名的五类门禁现状**：目标数据库/身份（W2/W6/事件传输：Neon/Owner profile、RLS、mTLS 部署身份均未验证）、运行时隔离（F03 G5 Isolation 端口未实现 + F13 沙盒网络/文件系统未接入 + F17 Mod 隔离未实现）、持久化（F15 G-04 通知/IM 进程内存、F04 kanban 不写服务端、F16 报告无持久化）、部署（W5/W6/W7/W8 全缺真实记录）、回滚（W0/W3/W5/W6/W9 均无 rollback 证据）。**全部保持 OPEN，NO-GO 不变。**

### 4.2 本工作区新增未验证项（残留待办处理后的状态）

1. F15 G-01 修复：`production_router` oneshot 绿 ✅；`notifications` 测试编译验证进行中（环境慢，多次超时后改后台长跑）。
2. 3D 纯 3D 化：tsc 0 + vitest 12/12 ✅；**7 主题浏览器截图对比已完成**——dev 模式 Turbopack 运行时在 headless 下不 hydrate（48 轮诊断排除代码/资源/错误因素，生产构建 `next build` EXIT=0 后 `next start` hydration 正常：fiber 515 / canvas 8），改用生产模式对 /agents、/projects、/automation-debug × 7 主题截图（见 §5）。**结论：dev-only 环境问题，非 3D 代码缺陷**。
3. ~~Guardian 基线断链 17 处~~ **已修复**：根因是 `cgg_guard.py` 的 `lstrip("./")` bug + 3 个 lib 旧式 Symbol id；修脚本 + 重写头后 70 纯 stamp 与全量 changed 复跑均 **0 错误**（§1.2-A）。
4. ~~`Collaboration` 空文件~~ **已删除**（0 字节确认）。
5. ~~87 个 CRLF-only 噪声~~ **已归一化**（§1.3），status/diff 计数一致。
6. **新发现（不阻塞本次提交）**：dev 模式 headless Turbopack 不 hydrate 的根因未完全定位（chunk 全部注册、0 错误、0 rejection，但 runtime 模块从不 instantiate）；生产模式正常，登记为后续 dev 工具链调查项。

### 4.3 建议后续顺序（与用户指令一致）

1. ✅ 工作区核清（本报告 §1）——已完成，未动任何文件。
2. ✅ 交付文档刷新（本报告 §2–§4）——已完成。
3. ✅ 按领域拆分提交（残留待办阶段执行，逐文件 `git add`，无整体暂存）：批次 = ① F15 G-01（lib.rs/main.rs/production_router.rs）② 3D 组件（9 文件）③ 纯 stamp + Guardian 修复（70+3 文件）④ 文档修正（6 文件 + INDEX）⑤ F 审计（14 文件）⑥ 本报告；CRLF 归一化已并入各领域文件，无独立噪声批次。
4. ⏭ 门禁证据补齐前保持 NO-GO；任务卡关联 VM/沙盒/Build 需求另立工作项（锚点 F04/F13/local-runtime）。
5. ⏭ 部署更新：本地生产构建 + `next start` 验证已完成（可复核证据见 §5）；正式发行部署仍受 W0–W9 门禁约束，**不解除 NO-GO**。

---

## §5 验证摘要（本阶段实际运行）

- `git status --porcelain=v1` / `git diff --name-only` / `git ls-files --others`：177/87/16 计数实测。
- 全量 `git diff -U0` 落盘解析：87/87 文件全覆盖，70 纯 stamp + 11 代码 + 6 文档。
- `cgg_guard.py --json <70 纯 stamp 文件>`：修复后 **0 错误**（原 17 断链为脚本 `lstrip` bug + 3 lib 旧式 Symbol id，均已修）；全量 changed 81 文件复跑亦 0 错误。
- `git ls-remote origin dev`：94505e75e314bab3ac5ce50cb43e141ee578c84c，与本地 HEAD 一致。
- 3D 行数 `wc -l`：8 组件合计 4,992 行。
- 起始阶段未运行 cargo/vitest/tsc（引用的是 §1.2-B 登记的既有回执）；未改任何代码、未暂存、未清理、未 push。

### 5.1 残留待办阶段新增验证（2026-11-18 后续）

- `npx tsc --noEmit`（frontend）：exit 0 ✅（CRLF 归一化 + Cypher 头重写后复跑）。
- `cgg_guard.py --stamp`（3 lib + 2 CI 脚本）：`PASS (5 source file(s) checked)` ✅；`--json` 70 纯 stamp / 81 changed 全量：**ok=true, errors=0** ✅。
- CRLF 归一化：87/87 逐文件验证后 `\r\n`→`\n` 写回；复测 status 91 = 88 M + 3 ??，diff 88，CRLF-only 0 ✅。
- `NODE_ENV=production npx next build`：**EXIT=0** ✅（全量生产构建通过，含 3D 组件）。
- `npx next start -p 3200` + headless Chromium（swiftshader WebGL）：/agents hydration **fiber=515, canvas=8, INITIALIZING 消失** ✅；dev 模式同页面 fiber=0（Turbopack dev 运行时问题，48 轮诊断记录在案，非代码缺陷）。
- 7 主题 × 3 页面截图（生产模式）：21 张 PNG 落盘 `E:\Temp\pw-shots\`，主题注入经 `localStorage["star-theme"]` + classList + data-theme（next-themes 实际机制）；每页仅 2 条 API 500（后端未起，预期内，不影响 UI/主题渲染）。
- `cargo test -p star-api-rest notifications --no-run`：**EXIT=0** ✅（此前前台超时为环境慢，后台长跑完成）；`--test notifications_endpoints` 实跑 **12 passed / 0 failed** ✅；`--test production_router` 复跑 **1 passed** ✅；`cargo fmt --all --check` **EXIT=0** ✅。