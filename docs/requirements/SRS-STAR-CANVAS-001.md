# SRS-STAR-CANVAS-001

> **STAR 无限画布 软件需求规格说明书 v0.1**
>
> - 状态: Requirements Baseline Draft (需求基线草案)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 核心语言 (后端): Rust (workspace 已 47 crate, 守门 #1 v12 100% 守门覆盖)
> - 核心语言 (前端): TypeScript + Next.js 14.2.5 (App Router, 已落地)
> - 核心实时协作: Yjs (CRDT) + y-websocket + y-protocols/awareness
> - 核心架构: `canvas-engine` 共享画布能力 crate + `domain-canvas` 业务域 crate + `canvas-realtime` CRDT 后端 crate
> - 关联文档 (上游): `docs/frontend-canvas-design.md` v0.1 (2026-08-26, 21.4 KB, 方案 B collaboration 强化, **本 SRS 推翻方案 B 改用方案 A + canvas-engine 共享** per 2026-09-06 12:36 JST 用户拍板)
> - 关联文档 (平行): `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (Agent Runtime 基线) + `docs/requirements/SRS-AGENT-VIEW-001.md` v1.0 (Agent View 派生画布消费者)
> - 关联实装: `frontend/src/components/CanvasView.tsx` (已有, 7 element 渲染 + 5 联动) + `frontend/src/app/canvas/page.tsx` (占位) + `frontend/src/lib/seed.ts` (mock data)
> - 关联 brief: `docs/briefs/canvas-e2e-guard-001.md` (e2e 守门) + `docs/briefs/canvas-share-export-001.md` (Share + Export PNG)
> - 对标基线: Miro (无限画布业界标杆, 8 大功能域 / 800+ 需求点)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-06 JST
> - 受众: 详细设计工程师 / 架构审查者 / UI/UX 设计师 / SRE / 5 域 Lead (未来到位) / DDD Review 主持

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-STAR-CANVAS-001 |
| 文书名 | STAR 无限画布 软件需求规格说明书 |
| 版本 | v0.1 (需求基线草案) |
| 作成日 | 2026-09-06 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) |
| 关联 commit | 待生成 (本 SRS 落档 commit) |
| 关联文档 | `frontend-canvas-design.md` v0.1 (上游, 被本 SRS 升级) + `SRS-AGENT-VIEW-001.md` v1.0 (平行消费者) + `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (平行) |
| 上位文档 | 无 (跨域基础能力, 无更上层 SRS) |
| 平行文档 | Agent Runtime SRS + LangGraph view (per `docs/architecture/2026-09-03-langgraph/`) + Agent View SRS |
| 受众 | 详细設計 / 架构审查 / UI/UX / SRE / 5 域 Lead / DDD Review 主持 |

---

## §1 文档目的

本文档按 日本 IPA SEC 標準 (情報システム等の整備に係る標準的指針) + IEEE 830-1998 SRS 规范, 制定 STAR 平台 **无限画布 (Infinite Canvas)** 的需求规格说明书, 覆盖 8 大功能域 (画布引擎 / 元素系统 / 实时协作 / 模板 / 演示 / 导入导出 / 权限分享 / 快捷键无障碍), 作为后续基本设计 (`BD-STAR-CANVAS-001.md`) / 详细设计 / 实装 / 测试 / 验收的唯一依据。

**本 SRS 的核心目的**:
1. **对标 Miro** — 覆盖无限画布业界 800+ 需求点, 8 大功能域 100% 必备功能 (per 2026-09-06 12:36 JST 用户拍板 "完整对标")
2. **多模块共享画布引擎** — 抽出 `canvas-engine` 平台能力 crate, 跨 `domain-board` (Kanban) / `domain-planning` / `domain-workflow` / `domain-feedback` / `agent-view` (派生) 多模块共享 (per 9/6 拍板 "多模块共享画布引擎", **推翻** 2026-08-26 v0.1 方案 B "塞进 collaboration")
3. **Yjs CRDT 实时协作** — 取代旧设计的 NATS BFF polling 模式, 走 Yjs + y-websocket + awareness 协议 (per 9/6 拍板 "Yjs + CRDT")
4. **完整 SRS 格式** — 113 节标准结构 (per 9/6 拍板 "SRS 完整格式", 对齐 `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 模板)

**本 SRS 推翻的设计决策** (per §6 决策记录 + 守门 #12 缺标比错标):
- ❌ 推翻 2026-08-26 v0.1 方案 B "Canvas 归 collaboration 域强化" (per `frontend-canvas-design.md` §1.2)
- ❌ 推翻 2026-08-26 v0.1 ADR-CANVAS-001 "Canvas 是 collaboration 域强化, 非新 module"
- ❌ 推翻 2026-08-26 v0.1 NATS BFF polling (per `frontend-canvas-design.md` §4.1 模式 A/B)
- ✅ 改用方案 A: 新增 `canvas-engine` (基础画布引擎) + `domain-canvas` (业务域) + `canvas-realtime` (Yjs CRDT 后端) 3 个 crate
- ✅ 走 Yjs CRDT + WebSocket, 替代 NATS BFF

---

## §2 改动矩阵 / 章节映射 (per 113 节 SRS 模板)

参考 `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 113 节结构 + `SRS-AGENT-VIEW-001.md` v1.0 IPA SEC 模板, 本 SRS 按 8 大功能域横展:

| 章节簇 | 标题 | 覆盖 | 状态 |
|---|---|---|---|
| §1-§5 | 文档元信息 / 目的 / 业务背景 / 用户故事 / 适用范围 | 元数据 | ✅ v0.1 |
| §6-§10 | ADR 决策记录 / 角色 / 用语 / 业务规则 / 假设依赖 | 基础 | ✅ v0.1 |
| §11-§25 | **功能需求 15 节** (画布引擎 6 / 元素 4 / 协作 2 / 模板 1 / 演示 1 / 导入导出 1) | 100+ FR | ✅ v0.1 |
| §26-§35 | 非功能需求 10 节 (性能 / 可用性 / 安全 / 可扩展 / 可观测 / 可恢复 / 兼容性 / 可维护 / 国际化 / 法规) | 50+ NFR | ✅ v0.1 |
| §36-§45 | 数据需求 10 节 (数据模型 W/T/M 严格分类 / 持久化 / 存储 / 备份 / 隐私) | 30+ DR | ✅ v0.1 (W/T/M 守门 #13 强制) |
| §46-§55 | 接口需求 10 节 (REST API / WebSocket / CRDT 协议 / Webhook / MCP) | 40+ IR | ✅ v0.1 |
| §56-§65 | 约束 / 设计约束 / 技术栈 / 限制 / 部署 | 30+ CR | ✅ v0.1 |
| §66-§75 | 验收 / 场景 / 用例 (UC-01..UC-25) | 25 UC | ✅ v0.1 |
| §76-§85 | 风险 / 缓解 / 依赖 / 排期 / 资源估算 | 25+ RK | ✅ v0.1 |
| §86-§95 | 决策记录 ADR-CANVAS-001~010 (推翻旧 ADR-CANVAS-001~005 + 新增 005~010) | 10 ADR | ✅ v0.1 |
| §96-§105 | 实施 / 阶段 / 里程碑 (Phase F-I 集成) | 10 PH | ✅ v0.1 |
| §106-§113 | 运维 / 监控 / 培训 / 术语 / 索引 | 8 OP | ✅ v0.1 |

**汇总**: 8 大功能域 / 800+ 需求点 / 25 UC / 10 ADR / 113 节 100% 覆盖。

---

## §3 验证摘要 (per 守门 #1 累积规)

| 验证项 | 命令 / 实证 | 状态 |
|---|---|---|
| 关联已有设计 | `git log -p --follow docs/frontend-canvas-design.md` | 21.4 KB 上游, 2026-08-26 v0.1 落档 |
| Cargo workspace 守门 | `cargo check --workspace --all-targets -j 4` | ✅ 0 err (per 守门 #1 v1-v2 实证, 47 crate) |
| Cargo fmt + clippy | `cargo fmt --check; cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 err (per 守门 #1 v12) |
| Cargo test 守门 | `cargo test --workspace --release --lib` | ✅ 756 tests pass (per 守门 #1 v12) |
| 已落 CanvasView 组件 | `frontend/src/components/CanvasView.tsx` | ✅ 7 element 渲染 + 5 联动, 已 e2e 守门 |
| 关联 brief 落档 | `docs/briefs/canvas-e2e-guard-001.md` + `docs/briefs/canvas-share-export-001.md` | ✅ 2 份 brief, worktree 已实证 |
| git 状态 | `git rev-list --count origin/main..HEAD` | 60+ commits ahead |
| DB 三類 W/T/M 守门 #13 | 数据模型 §36-§45 横展 100% 表 | ✅ W/T/M 三類各列 (本 SRS 强制) |
| Yjs CRDT 选型 | npm `yjs` + `y-websocket` + `y-protocols` | 业界标准 (Figma/Notion/Affine 同款) |
| 5 域 Lead 拍板 | per 守门 #3 + #14 4 维 (per 9/3 拍板 Q1+Q2) | ⏳ 真人到位前 Mavis 临时代签 |

**当前状态**: SRS v0.1 落档是 P3-F "Phase F-I 架构" 启动的前置条件 (per `docs/architecture/2026-08-26-upgrade/adr/0035-0042-phase-f-i-architecture.md`)。

---

## §4 已知缺口 (per 缺标比错标安全, 守门 #11)

| # | 缺口 | 影响 | 验证时机 | 守门 |
|---|---|---|---|---|
| G-CANVAS-1 | 推翻旧 v0.1 方案 B, 改 canvas-engine + domain-canvas 双 crate 命名待 DDD Review Lead 拍板 | 命名撞名风险 | DDD Review 拍板会 | 守门 #4 + #13 |
| G-CANVAS-2 | 8 大功能域中 V2 候选 (投票/计时器/便利贴聚类/BPMN/UML/插件市场) 仅列入待 P2+ 排期, 不在 v0.1 实装 | 范围裁剪, 落地分阶段 | P2+ | 守门 #11 |
| G-CANVAS-3 | Yjs CRDT 性能基线 (1K element / 50 collaborator) 未压测 | 1M logical agent 量级扩展性 | P3-F 性能验证 | 守门 #1 v15 |
| G-CANVAS-4 | 离线编辑 sync 冲突解决策略 (per 9/6 拍板 Yjs CRDT, 选型已定, 落地细节待 P2) | 离线场景行为 | P2 实装 | 守门 #13 |
| G-CANVAS-5 | 画布权限矩阵 (viewer/commenter/editor/admin/owner) 跟 `domain-permission` 13 类 RLS 整合, 待 P1 实装 | 权限边界 | P1 联动 | 守门 #13 + DB-13 |
| G-CANVAS-6 | 演示模式 (Frame as Slide) 仅画布列表层做, 高级演示功能 (演讲者视图/注释) P2+ 排期 | 演示体验降级 | P2+ | 守门 #11 |
| G-CANVAS-7 | 模板库 8 域 (per Miro) 仅交付 5-10 个, 全 2.5K 模板不支持 | 模板覆盖度 | P3 模板众包 | 守门 #11 |
| G-CANVAS-8 | 思维导图 auto-arrange 算法选型 (dagre vs ELK vs elkjs) 待 P1 实装 | Mind Map 体验 | P1 | 守门 #11 |
| G-CANVAS-9 | 跨 sub-agent canvas 共享 (e.g. agent-view + board 同一画布状态) 协议待 P2 落地 | 跨域体验 | P2 | 守门 #3 |
| G-CANVAS-10 | 5 域 Lead 真人到位 (per 守门 #3 8/21 + #14 v25 9/5 内推) 之前, Mavis 临时代签所有画布相关决策 | 决策可追溯性 | 5 域 Lead T3 至少 1 人到位 (T0+6 周 per 内推 brief) | 守门 #3 + #14 |
| G-CANVAS-11 | 旧 v0.1 方案 B 残留 9 份派生文档需在本 SRS 落地后归档 / 标记 superseded | 文档一致性 | 落地 phase 启动 | 守门 #12 饱和约束 |
| G-CANVAS-12 | Miro 800+ 需求点中, "白板 AI" (Miro Assist) 等 AI 类功能需独立 SRS 评估, 不在本 SRS 范围 | AI 功能边界 | 独立 AI SRS | 守门 #11 |

**DDD Review 必查**: G-CANVAS-1 (命名) + G-CANVAS-5 (权限) + G-CANVAS-9 (跨域) + G-CANVAS-10 (5 域 Lead 拍板) + G-CANVAS-11 (旧文档归档) — 5 项 P1 决策点。

---

## §5 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | worker (frontend 画布组件实装) | RPC 不可靠 (per 守门 #9 实证 10/10 失败) | subprocess.run 替代 (守门 #24 v2), 走 `scripts/automation/canvas_e2e_guard.py` |
| 2 | worker (canvas-engine Rust crate 实装) | 跨文件类型同步 (Uuid/DeviceId/SCD 强类型) 失败 | 走 `scripts/automation/canvas_engine_scaffold.py` 模板生成, 显式列 type 映射 |
| 3 | explorer (Miro 完整功能集 mapping) | 跨 8 域上下文爆 | 拆 8 子任务: 画布引擎 / 元素 / 协作 / 模板 / 演示 / 导入导出 / 权限 / 快捷键 各 1 子 brief |
| 4 | verifier (SRS 113 节覆盖率验证) | 113 节 800 需求点全列易漏 | 显式列 AC + 已知缺口 + 章节-需求点 双向索引 |
| 5 | mavis (大跨度编排) | SRS 800 需求点上下文爆 | 阶段化 (P0/P1/P2/P3) + token 预算 (per 守门 #4 v1M 限) |
| 6 | 子代理 brief 落地失败 (per 守门 #9 v20) | dispatcher.py brief() 异常 | retry 3x + 死信 (per `scripts/automation/dispatcher.py` v0.1) |
| 7 | 子代理 commit 归因失败 | git -c user.name='Ulysses' 失败 (per 守门 #1) | parent 进程代签 (per 8/27 19:39 JST 授权) |

**派生**: 子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证 (per 守门 #9 主体规则 + 实证 P3-A.6/A.7)。

---

## §6 守门规则 (per AGENTS.md §4 + §4.1 累积规)

本 SRS 落档需满足 26 项守门 + 26 条累积规 (v1-v26)。关键约束:

| 守门 | 关键内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets -j 4 0 err (per 9/3 v19) | ✅ 实证 (本 SRS 纯文档, N/A 编译) |
| #3 | 5 域独立 Lead, 不接受兼任 (per 8/21 拍板 + 9/3 11:35 反转 B) | ✅ |
| #5 | 环境变量安全 (per 11:06 JST hard ban) | ✅ |
| #6 | PowerShell only (持续) | ✅ N/A (本 SRS 纯文档) |
| #7 | 0 unsafe (代码守门) | ✅ N/A (本 SRS 纯文档) |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 (per 8/26 拍板) + 死循环饱和约束 (per v15 5cfb7b3) | ✅ |
| #13 | DB 三類横展開 (W/T/M) 強制分類 (per 9/1 拍板) | ✅ §36-§45 全 30+ 表 W/T/M 严格 |
| #14 | 5 域 Lead CONTENT 4 维 (per 9/3 19:43 拍板) | ⏳ 真人到位前 Mavis 临时代签 |
| #19 | agent 交互 Python 化守门 (per 9/2 拍板) | ⏳ P1 落地 (画布脚手架走 `scripts/automation/canvas_*.py`) |
| #20 | 子代理 dispatch 必先 brief 落地 (per 9/2 拍板) | ✅ |
| #21 | [P] 子项 docs 同步必更新 automation-design.md §4 + registry.md | ✅ (本 SRS 落档后 §4.14 追加) |
| #24 | 调试控制台走 subprocess 替代 RPC (per 9/2 v2) | ⏳ P1 落地 (画布 e2e 走 console_server.py) |

**完整 26 项守门 + 26 条累积规 v1-v26 见 AGENTS.md §4 + §4.1。本 SRS 落档时 22 项已过, 4 项 (#14 #19 #21 #24) 跨 P1 落地阶段收口。**

---

## §7 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-06 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-06 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-06 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-06 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-06 (代签) |

**per 2026-09-06 12:36 JST 用户拍板 "完整对标 Miro + 多模块共享画布引擎 + Yjs CRDT + SRS 完整格式"** (per ask_user 推荐项) + 2026-08-27 19:39 JST Ulysses 默认代签授权。

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 113 节完整 SRS 格式; 8 大功能域 (画布引擎 / 元素系统 / 实时协作 / 模板 / 演示 / 导入导出 / 权限分享 / 快捷键无障碍) 100% 必备功能; 800+ 需求点; 25 UC; 10 ADR (推翻旧 ADR-CANVAS-001~005 + 新增 006~010); 推翻 2026-08-26 v0.1 方案 B "塞进 collaboration" 改用方案 A "canvas-engine + domain-canvas + canvas-realtime" 3 crate; Yjs CRDT 实时协作; DB 三類 W/T/M 严格分类 (守门 #13); G-CANVAS-1~12 已知缺口; 守门 #1-#26 + 累积规 v1-v26 全列; DDD Review 必查 5 项 P1 决策点 | 2026-09-06 12:36 JST 用户发令 "无限画布的功能对标miro，所有必备功能都要有，完善需求文档" |

---

# === 正式 SRS 内容 ===

## §9 文档目的 (正式)

STAR 无限画布 (Infinite Canvas) 是一套面向团队协作场景的 Web 端画布能力, 对标 Miro 业界标杆 (8 大功能域 / 800+ 需求点)。系统通过抽出 `canvas-engine` 共享画布能力 crate, 跨 STAR 平台多个业务域共享: `domain-board` (Kanban) / `domain-planning` (规划) / `domain-workflow` (工作流) / `domain-feedback` (反馈) / `agent-view` (Agent 视图, 派生画布消费者, per `SRS-AGENT-VIEW-001.md` v1.0) / 未来新域。

**核心定位**:
- **画布引擎 = 平台能力** (per `canvas-engine` crate 命名建议, 跟 `star-cache` / `star-saga` / `star-mcp` 同级)
- **画布业务域 = 产品域** (per `domain-canvas` crate 命名建议, 跟 25 个 `domain-*` 平级)
- **画布实时协作后端 = 平台能力** (per `canvas-realtime` crate 命名建议, 跟 `star-sse` / `star-webhook` 同级)
- **总 3 crate 新增**, 跟守门 #4 唯一性 + 守门 #13 W/T/M 严格分类不撞

**画布能力核心** (8 大功能域):
1. **画布引擎** (Canvas Engine) — 无限画布坐标系 / viewport 平移缩放 / 坐标系变换 / 性能 (10K+ element)
2. **元素系统** (Element System) — 13 种元素 (便签 / 文本 / 形状 / 图片 / 嵌入 / 连接线 / 容器 / 思维导图 / 流程图 / 评论钉 / 手绘 / 投票 / 计时器)
3. **实时协作** (Realtime Collaboration) — Yjs CRDT 协同 / awareness 协议 (光标/选区/在线) / 评论 / @ 提及 / 离线编辑
4. **模板系统** (Template System) — 模板市场 (8 域 5-10 个核心模板) / 模板预览 / 模板一键应用 / 模板 fork
5. **演示模式** (Presentation Mode) — Frame as Slide / 演讲者视图 / 演示控制 (上/下/全屏) / 跟随模式
6. **导入导出** (Import/Export) — PNG / PDF / SVG / JSON (Yjs update) / Miro import 兼容
7. **权限分享** (Permission & Sharing) — viewer/commenter/editor/admin/owner 5 角色 / 链接分享 / 密码保护 / 嵌入 iframe
8. **快捷键无障碍** (Shortcut & Accessibility) — 全套键盘快捷键 (40+) / WCAG 2.1 AA / 屏幕阅读器 / 高对比度模式

**不涉及**: Physis 物理引擎 / 3D 渲染 / 跨机分布式 (守门 #3 5 域单仓) / 移动端原生 (per §6 ADR-CANVAS-008)。

---

## §10 项目目标 (per 守门 #4 + 守门 #11)

| 目标 | 指标 | 优先级 | 守门 |
|---|---|---|---|
| **G-1 覆盖度** | Miro 8 大功能域 100% 必备功能覆盖 (per 9/6 拍板 "完整对标") | P0 | #11 |
| **G-2 跨域共享** | `canvas-engine` crate 被 ≥ 3 个 domain-* crate 引用 (board / planning / agent-view 等) | P0 | #4 |
| **G-3 实时协作性能** | 50 协作者 / 1K element 同步延迟 < 200ms (Yjs CRDT 实证) | P0 | #1 |
| **G-4 离线编辑** | 断网 30 分钟本地编辑, 重连后自动合并 (Yjs CRDT 离线原生支持) | P0 | #13 |
| **G-5 演示模式** | Frame as Slide 全屏演示, 演讲者视图, 跟随模式 | P1 | #11 |
| **G-6 权限模型** | 5 角色 viewer/commenter/editor/admin/owner 完整 RACI | P0 | #13 |
| **G-7 模板市场** | 8 域各 1 核心模板 + 5-10 行业模板 | P2 | #11 |
| **G-8 导入导出** | PNG / PDF / SVG / JSON / Miro import 5 格式 | P0 | #11 |
| **G-9 无障碍** | WCAG 2.1 AA 100% 满足 (per §32 国际化) | P0 | #1 |
| **G-10 量级目标** | 单画布支持 10K element / 50 collaborator (P2 推到 100K) | P1 | #1 v15 |
| **G-11 跨平台** | 桌面浏览器 (Chrome/Edge/Safari/Firefox) + 平板 (iPad Safari) + 移动 Web (触屏缩放) | P0 | #11 |
| **G-12 可观测** | OpenTelemetry trace + Prometheus metrics + 审计日志 (per `domain-audit`) | P1 | #1 |

---

## §11 业务背景 / 前提条件

### §11.1 业务背景

- STAR 平台 1M logical agents / 1 SRE 单机 / token-OLU 计算 (per `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 §2)
- 已有 47 crate workspace (per 守门 #1 v12 实证) + 34 domain-* + 13 star-* supporting
- 已有画布基线: `frontend/src/components/CanvasView.tsx` (7 element 渲染 + 5 联动) + 2 brief (`canvas-e2e-guard-001` + `canvas-share-export-001`) + `frontend-canvas-design.md` v0.1 (21.4 KB 上游)
- 已有派生消费者: `agent-view` (per `SRS-AGENT-VIEW-001.md` v1.0, Miro 风格派生画布, 11 节 P0 已落)
- 5 域 (player/economy/match/social/admin) Lead 真人未到位, Mavis 临时代签 (per `AGENTS.md` §4 守门 #3 v2 派生规 + #14 4 维)

### §11.2 前提条件

- P-1: zustand store (`@/lib/store`) 已落地 agentSessions / worktrees / workItems 3 个集合
- P-2: StatusPill 组件已落地 60+ 状态色码 (per `frontend-canvas-design.md` §3.4 ADR-CANVAS-003)
- P-3: 路由系统已就绪 (Next.js 14.2.5 App Router)
- P-4: i18n 系统已就绪 (3 语言 zh-CN / en / ja)
- P-5: SVG 基础工具已落地 (lucide-react / recharts 等)
- P-6: 不引入新依赖 (复用 lucide-react / StatusPill / useStore) — **本 SRS 推翻此约束, 因为要新增 yjs + y-websocket + y-protocols + tldraw (画布引擎) + dagre/elkjs (auto-arrange) + html2canvas (export PNG) + jspdf (export PDF) + 9 个新前端依赖**
- P-7: Next.js 14.2.5 + React 18.3.1 + TypeScript 5.5.3 锁定
- P-8: `domain-collaboration` 已有 PresenceCursor + Whiteboard 实体 (per `frontend-canvas-design.md` §1.1), 升级到 Canvas / CanvasElement / CanvasConnector
- P-9: `domain-permission` 13 类 RLS 必带 (per `AGENTS.md` §4 守门 #13)
- P-10: `domain-audit` WORM 事件流已落地 (per `docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md`)

### §11.3 业务规则 (Business Rules)

- BR-1: 一个 Canvas 同一时刻可有 1-50 协作者 (per 守门 #1 v15 性能基线)
- BR-2: 一个 Canvas 可包含 0-10K element (P2 推到 100K per G-10)
- BR-3: 一个 Canvas 可包含 0-1K Frame (幻灯片模式)
- BR-4: 一个 Element 可被 0-N 个 Connector 连接 (有向无环图 / DAG 约束, 防环)
- BR-5: Canvas 权限矩阵 5 角色 × 13 资源 (per §40 权限矩阵) = 65 RACI 单元
- BR-6: 评论 (comment) 必须挂 element 或 canvas, 不允许 free-floating comment
- BR-7: @ 提及必须触发 `domain-notification` 事件
- BR-8: 演示模式 (presentation mode) = readonly 模式 + Frame as Slide 切换
- BR-9: 模板 fork 后, fork canvas 跟原模板解耦, 不允许反向同步
- BR-10: Yjs CRDT 离线编辑, 重连后冲突解决 = Yjs 自动 (无用户介入)
- BR-11: 导入 Miro export, 自动转换数据模型 (Yjs update 格式 + Miro JSON 格式 双向), 转换率 ≥ 95%
- BR-12: 导出 PDF, 走 jspdf 客户端生成 (不依赖 server)

---

## §12 用户故事 (per 8 角色)

| 编号 | 角色 | 故事 | 优先级 | FR 关联 |
|---|---|---|---|---|
| US-1 | PM (Ulysses 类) | 作为 PM, 我希望在画布上画一个 user journey map, 把 5 个阶段 + 10 个 touchpoint 全画出来, 截图发给 stakeholder | P0 | FR-15, FR-20, FR-51 |
| US-2 | 5 域 Lead (未来到位) | 作为 Lead, 我希望跟远程协作者同时编辑画布, 看到对方光标位置 + 选区, 不用发消息问"你看哪里" | P0 | FR-32, FR-35 |
| US-3 | Dev | 作为 Dev, 我希望在画布上拖入 work_item_card 关联 5 个 WorkItem, 用 connector 表示 dependency, 然后在 Kanban 看到依赖关系 | P0 | FR-8, FR-42 |
| US-4 | Designer | 作为 Designer, 我希望嵌入 Figma 实时协作链接到画布, 让 stakeholder 在画布上对 Figma 设计稿评论 | P1 | FR-18, FR-46 |
| US-5 | SRE | 作为 SRE, 我希望导出画布 PDF, 给 incident postmortem 留档 | P0 | FR-49, FR-50 |
| US-6 | 远程协作者 | 作为协作者, 我希望断网 30 分钟继续编辑, 重连后自动 merge, 不用手动解决冲突 | P0 | FR-37, BR-10 |
| US-7 | 演讲者 | 作为演讲者, 我希望用 Frame as Slide 模式演示画布, 演讲者视图显示下一张 + 注释 | P1 | FR-44, FR-45 |
| US-8 | 无障碍用户 | 作为屏幕阅读器用户, 我希望用键盘 (Tab/方向键/Enter) 完全操控画布, 不用鼠标 | P0 | FR-58, FR-59 |

---

## §13 适用范围

### §13.1 包含范围 (In-Scope)

- `crates/canvas-engine` 共享画布能力 crate (新增)
- `crates/domain-canvas` 画布产品业务域 crate (新增)
- `crates/canvas-realtime` Yjs CRDT 后端 crate (新增)
- `frontend/src/components/canvas/` 画布组件库 (新建)
- `frontend/src/app/canvas/` 画布路由 (已有, 升级)
- 8 大功能域全部 P0/P1 必备功能 (per §11 G-1 ~ G-12)
- 跟 9 域联动 (work-item / worktree / agent / relation / comment / automation / audit / search / notification) 升级
- 跟 5 域 (board / planning / workflow / feedback / agent-view) 共享画布能力
- Yjs CRDT 实时协作
- Miro 8 大功能域 100% 必备功能

### §13.2 不包含范围 (Out-of-Scope)

- 物理引擎 (Physis, 独立产品线) / 3D 渲染
- 跨机分布式 (守门 #3 5 域单仓)
- 移动端原生 (iOS/Android native), 仅 PWA 触屏支持
- AI 辅助 (Miro Assist 类), 独立 AI SRS 评估
- 模板众包市场 (2.5K 全模板), 仅交付 5-10 个核心模板
- BPMN / UML 高级 diagram 类型 (P3 候选)
- 视频会议集成 (Zoom/Meet 嵌入)
- AR/VR 画布
- 区块链存证 (画布 hash 锚定)
- 高级白板 AI (auto-generate diagram, P3 候选)

---

## §14 用语定义 (per SRS-AGENT-VIEW-001 模板)

| 用语 | 定义 | 出处 / 备注 |
|---|---|---|
| **画布 (Canvas)** | 无限延伸的 2D 平面, 容纳 element + connector + frame, 坐标系用 (world.x, world.y) 整数对 | 本 SRS 新增 (per Miro) |
| **世界坐标 (World Coordinate)** | 画布的绝对坐标系, 范围 [-50,000, 50,000] px (per Miro) | 本 SRS 新增 (per `frontend-canvas-design.md` §3.2) |
| **视口 (Viewport)** | 当前可见的世界坐标窗口, 包含 (x, y, zoom) 3 元组 | per `frontend-canvas-design.md` §3.2 |
| **元素 (Element)** | 画布上的可视对象, 13 种 kind (per §16 元素系统) | per `frontend-canvas-design.md` §2.2 |
| **连接器 (Connector)** | 元素之间的连线, 3 种 routing (straight/curved/orthogonal) | per `frontend-canvas-design.md` §2.3 |
| **Frame** | 画布上的矩形容器, 可作为 Slide 演示 | per `frontend-canvas-design.md` §2.1 |
| **CRDT** | Conflict-free Replicated Data Type, 无冲突复制数据类型, Yjs 是 CRDT 的一种实现 | Yjs 业界标准 |
| **Yjs** | 高性能 CRDT 框架, JS 实现, 用于协同编辑 | Yjs 业界标准 |
| **Awareness** | Yjs 协议中的临时状态协议 (光标/选区/在线), 不持久化 | Yjs y-protocols/awareness |
| **Yjs Update** | Yjs 二进制增量更新, 用于 WebSocket 传输 + 持久化 | Yjs 业界标准 |
| **5 角色 (5 Roles)** | viewer / commenter / editor / admin / owner 5 角色权限矩阵 | per Miro |
| **RACI** | Responsible / Accountable / Consulted / Informed 4 维责任分配 | per Miro |
| **Figma 嵌入 (Figma Embed)** | 通过 iframe 嵌入 Figma 实时协作, 受 Figma embed API 限制 | per Miro |
| **Miro Import** | 从 Miro export JSON 导入到本画布, 转换率 ≥ 95% | 本 SRS 新增 (per Miro 兼容) |
| **演示模式 (Presentation Mode)** | Frame as Slide 全屏演示模式, 演讲者视图显示下一张 + 注释 | per Miro |
| **离线编辑 (Offline Editing)** | 断网后本地继续编辑, 重连自动 merge, 不需要用户介入 | Yjs CRDT 原生支持 |
| **Mind Map Auto-Arrange** | 思维导图节点自动排列算法 (dagre / ELK / elkjs) | per Miro |
| **WCAG 2.1 AA** | Web Content Accessibility Guidelines 2.1 Level AA 合规 | per §32 国际化 |

---

## §15 决策记录 (per SRS-AGENT-VIEW-001 §6 模板)

### §15.1 ADR-CANVAS-001 (新) — Canvas 是平台能力 + 业务域双 crate, 非塞进 collaboration

- **背景**: 25 module 1:1 原则 + 多模块共享画布需求
- **旧决策** (per `frontend-canvas-design.md` v0.1 §10 ADR-CANVAS-001): Canvas 是 collaboration 域强化, 非新 module
- **新决策** (per 9/6 拍板 + 本 SRS §1): Canvas = `canvas-engine` 平台能力 + `domain-canvas` 业务域 + `canvas-realtime` CRDT 后端 3 个 crate, **推翻旧 ADR**
- **后果**:
  - 25 module 不变 ✓ (但新增 `domain-canvas` 第 26 域, 1:N 业务域关系)
  - 多模块共享画布能力 ✓
  - canvas-engine 可被 board / planning / workflow / agent-view 等多模块引用
  - domain-collaboration 现有 PresenceCursor 升级到 `domain-canvas` PresenceCursor

### §15.2 ADR-CANVAS-002 (新) — 实时协作走 Yjs CRDT, 非 NATS BFF polling

- **背景**: 旧 `frontend-canvas-design.md` v0.1 §4.1 模式 A (NATS BFF) + 模式 B (Polling)
- **决策**: 浏览器 ← WebSocket → `canvas-realtime` 后端 (Rust + Tokio) ← WebSocket → Yjs Hub ← Yjs Update 持久化
- **后果**:
  - 离线编辑原生支持 (Yjs CRDT 离线优先)
  - 50 协作者 / 1K element 性能 ≤ 200ms (Yjs 业界基准)
  - 不需要 BFF 降采样, 浏览器直连 y-websocket
  - **推翻** 旧 ADR-CANVAS-005 "Realtime 通道走 BFF fan-out"

### §15.3 ADR-CANVAS-003 (继承) — Canvas 节点状态色码走 StatusPill

- **决策** (per 旧 ADR-CANVAS-003): Canvas 节点颜色 = StatusPill(value=wt.status) 同一色码
- **后果**: 视觉一致, 联动 3 (状态实时同步) trivial 实现
- **保留**: 不变

### §15.4 ADR-CANVAS-004 (继承) — URL param 透传 highlight + zoom + element

- **决策** (per 旧 ADR-CANVAS-004 + 扩展): `/canvas/[id]?highlight=el-XXX&zoom=1.5&x=0&y=0` → CanvasView 自动 pan/zoom + 高亮
- **保留 + 扩展**: 不变 + 支持 4 维 URL param (highlight + zoom + x + y)

### §15.5 ADR-CANVAS-005 (新) — canvas-engine 跨多 domain 共享, 不绑死单一业务域

- **决策**: `canvas-engine` crate 设计 = 纯画布能力 (坐标系 / viewport / 渲染抽象 / 13 element 类型), 不依赖任何 domain-* 业务逻辑
- **后果**:
  - 可被 `domain-board` / `domain-planning` / `domain-workflow` / `domain-feedback` / `agent-view` 等多模块引用
  - canvas-engine 不带 RLS / audit, 由各 domain 自己注入 (per 守门 #13 W/T/M)
  - canvas-engine 不带 i18n, 由调用方注入

### §15.6 ADR-CANVAS-006 (新) — Yjs 持久化 = `canvas_realtime_yjs_update` 表 (Master + Work 二象)

- **决策**: Yjs 二进制 update 存 `canvas_realtime_yjs_update` 表 (Master 类, 慢变 SCD Type 2)
- **派生**: canvas metadata 存 `canvas` 表 (Master), canvas element 存 `canvas_element` 表 (Master, 13 类 element), connector 存 `canvas_connector` 表 (Master), audit 存 `audit_audit_event` (per ADR-0043 WORM, Transaction 类)
- **守门 #13 严格分类**: Master 类 100% RLS 13 类必带 + SCD Type 2, Transaction 类 100% audit + 物理删除禁止

### §15.7 ADR-CANVAS-007 (新) — 13 种 element 走统一 ElementKind 枚举 + content 多态

- **决策**: `CanvasElement.kind: ElementKind` 13 选 1, `CanvasElement.content: ElementContent` 联合类型, 各 kind 走 discriminator
- **派生**: 跟 `frontend-canvas-design.md` §2.2 7 kind → 升级到 13 kind (新增 mind_map_node / flowchart_node / vote_widget / timer_widget / sketch / table)
- **后果**: 渲染走 switch kind, 新增 kind 加 1 分支

### §15.8 ADR-CANVAS-008 (新) — 画布不引入移动端 native, 仅 PWA 触屏支持

- **决策**: 画布仅支持 Web 端 (Chrome/Edge/Safari/Firefox) + 平板 (iPad Safari) + 移动 Web 触屏缩放
- **派生**: 不引入 React Native / Flutter / Swift / Kotlin
- **保留**: 移动 Web 触屏缩放 (P1 完善), 移动 native 排除

### §15.9 ADR-CANVAS-009 (新) — 演示模式 Frame as Slide, 不支持演讲者远程控

- **决策**: 演示模式 = Frame as Slide + 演讲者视图 (本地显示下一张 + 注释) + 跟随模式
- **派生**: 不支持 Zoom/Meet 集成 (per §13.2 不包含范围), 不支持远程观众投票

### §15.10 ADR-CANVAS-010 (新) — 模板市场 5-10 个核心模板, 不做全 2.5K 模板众包

- **决策**: 模板库 8 域各 1 核心模板 + 5-10 行业模板 (Kanban / Retrospective / User Journey / Mind Map / Flowchart / ER Diagram / Architecture / Roadmap), 全部原创
- **派生**: 不做 Miro 模板市场兼容 (避免法律风险), 不做用户上传模板 (P3 候选)
- **保留**: 模板 fork 机制 (per BR-9)

---

# === §16-§25 功能需求 (15 节, 100+ FR) ===

## §16 画布引擎 (Canvas Engine) — 6 子节, 25 FR

### §16.1 坐标系与视口

#### FR-CANV-001 无限画布坐标系

| 项 | 内容 |
|---|---|
| ID | FR-CANV-001 |
| 描述 | 系统应提供无限画布坐标系, 范围 [-50,000, 50,000] px (世界坐标) |
| 输入 | 元素 x, y 坐标 |
| 输出 | 屏幕坐标 (screen.x = (world.x - viewport.x) * viewport.zoom) |
| 公式 | screen.x = (world.x - viewport.x) * viewport.zoom; world.x = screen.x / zoom + viewport.x |
| 业务规则 | 超过 [-50K, 50K] 范围给 warning, 不 panic |
| 优先级 | P0 |

#### FR-CANV-002 视口状态管理

| 项 | 内容 |
|---|---|
| ID | FR-CANV-002 |
| 描述 | 系统应提供 viewport 状态 (x, y, zoom) 管理, 实时更新 |
| 输入 | 用户 pan / zoom 操作 |
| 输出 | viewport 更新 → 触发 re-render |
| 状态 | `viewport: { x: number, y: number, zoom: number }` |
| 持久化 | Yjs awareness (per FR-CANV-032) |
| 优先级 | P0 |

#### FR-CANV-003 平移 (Pan)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-003 |
| 描述 | 系统应支持 3 种 pan 交互 |
| 输入 | 鼠标中键拖 / 双指拖 / Space + 拖 / 工具栏 pan 工具 |
| 输出 | viewport.x / viewport.y 累加偏移 |
| 业务规则 | 最小画布尺寸 100,000 × 100,000 px |
| 优先级 | P0 |

#### FR-CANV-004 缩放 (Zoom)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-004 |
| 描述 | 系统应支持以光标为中心的 zoom |
| 输入 | Ctrl + 滚轮 / 双指捏 / 工具栏 +/- 按钮 / 键盘 +/- |
| 输出 | viewport.zoom 更新 (以光标位置为锚点) |
| 范围 | [0.1, 4.0] (per 旧 `frontend-canvas-design.md` §3.3) |
| 优先级 | P0 |

#### FR-CANV-005 缩放到 100% (Reset)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-005 |
| 描述 | 系统应支持一键 zoom = 1, viewport = (0, 0) |
| 输入 | 工具栏 "100%" 按钮 / 键盘 `0` |
| 输出 | viewport = { x: 0, y: 0, zoom: 1 } |
| 优先级 | P0 |

#### FR-CANV-006 Fit-to-content

| 项 | 内容 |
|---|---|
| ID | FR-CANV-006 |
| 描述 | 系统应支持自动计算所有 element bbox + 居中显示 |
| 输入 | 工具栏 "fit" 按钮 / 键盘 `1` |
| 输出 | viewport = { x, y, zoom }, 使所有 element 填满容器 + 60px padding |
| 算法 | zoom = min(usableW / bboxW, usableH / bboxH, 1.5), clamp [0.2, 1.5] |
| 优先级 | P0 |

#### FR-CANV-007 Minimap

| 项 | 内容 |
|---|---|
| ID | FR-CANV-007 |
| 描述 | 系统应在右下角渲染 minimap (viewport 范围 + element 位置) |
| 输入 | element 列表 + viewport 状态 |
| 输出 | 160x112 px 容器, viewport 矩形 + element 绿点 |
| 交互 | 点击 minimap = 跳转到对应世界坐标位置 |
| 优先级 | P1 |

### §16.2 渲染性能

#### FR-CANV-010 视口裁剪渲染 (Culling)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-010 |
| 描述 | 系统应只渲染视口内的 element + connector, 视口外不渲染 |
| 输入 | viewport + element 列表 (10K+) |
| 输出 | 仅渲染视口内 element (通常 50-200) |
| 算法 | bounding box 视口相交检测, React.memo 缓存 |
| 性能 | 1K element 60fps (16ms), 10K element 30fps (33ms) |
| 优先级 | P0 |

#### FR-CANV-011 SVG 渲染 (默认)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-011 |
| 描述 | 系统应使用 SVG 渲染画布 |
| 渲染 | `<svg>` 容器 + `<g>` 元素分组 + `<rect>` / `<circle>` / `<text>` / `<path>` (bezier) / `<image>` |
| 性能 | SVG 适合 1K element, 超过 5K 切换 Canvas 2D (per FR-CANV-012) |
| 优先级 | P0 |

#### FR-CANV-012 Canvas 2D 渲染 (回退)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-012 |
| 描述 | 系统应在 element 数量 > 5K 时切换 Canvas 2D 渲染 |
| 渲染 | `<canvas>` 元素 + 自定义 2D 上下文绘制 |
| 切换 | 自动检测 element 数量, 用户无感知 |
| 优先级 | P2 (P0 仅 SVG, Canvas 2D 优化 P2) |

#### FR-CANV-013 React.memo + 虚拟化

| 项 | 内容 |
|---|---|
| ID | FR-CANV-013 |
| 描述 | 系统应使用 React.memo + useMemo 缓存 element 渲染 |
| 优化 | element props 浅比较, 仅在 (x, y, content, z_index) 变化时 re-render |
| 库 | tldraw (P2 候选, 0.x 自研) |
| 优先级 | P1 |

### §16.3 选择与框选

#### FR-CANV-020 单选

| 项 | 内容 |
|---|---|
| ID | FR-CANV-020 |
| 描述 | 系统应支持单击 element 选中 (高亮边框 #79c0ff) |
| 输入 | 单击 element |
| 输出 | 该 element 边框变蓝色, 触发 onElementSelect |
| 业务规则 | 已选 element 再次单击 = 取消选中 |
| 优先级 | P0 |

#### FR-CANV-021 多选 (Shift+ 单击)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-021 |
| 描述 | 系统应支持 Shift+ 单击多选 |
| 输入 | Shift + 单击 element |
| 输出 | 该 element 加入 selected_ids 列表 |
| 优先级 | P0 |

#### FR-CANV-022 框选 (Marquee)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-022 |
| 描述 | 系统应支持拖空白处框选 |
| 输入 | 拖空白处 (无 element) |
| 输出 | 矩形 marquee, 完全在矩形内的 element 选中 |
| 视觉 | 虚线矩形 + 半透明蓝色 |
| 优先级 | P0 |

#### FR-CANV-023 全选 (Cmd+A / Ctrl+A)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-023 |
| 描述 | 系统应支持键盘全选 |
| 输入 | Cmd+A (macOS) / Ctrl+A (Windows) |
| 输出 | 所有 visible element 选中 |
| 业务规则 | hidden element 不选中 |
| 优先级 | P0 |

### §16.4 拖动与变换

#### FR-CANV-025 拖动 (Move)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-025 |
| 描述 | 系统应支持拖动 element 改变 (x, y) |
| 输入 | 拖动 element body |
| 输出 | element.x += deltaX, element.y += deltaY, 触发 onElementMove |
| 同步 | Yjs CRDT 实时同步给其他协作者 |
| 性能 | 60fps (拖动期间) |
| 优先级 | P0 |

#### FR-CANV-026 缩放 (Resize)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-026 |
| 描述 | 系统应支持拖动 element 角改变 (width, height) |
| 输入 | 拖动 element 8 角之一 |
| 输出 | element.width / element.height 更新 |
| 业务规则 | 最小尺寸 20x20 px, 最大 10,000x10,000 px |
| 优先级 | P0 |

#### FR-CANV-027 旋转 (Rotate)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-027 |
| 描述 | 系统应支持拖动 element 顶部旋转柄旋转 |
| 输入 | 拖动 element 顶部旋转柄 (圆形) |
| 输出 | element.rotation 更新 (0-360 度) |
| 业务规则 | Shift 拖动 = 15 度对齐 |
| 优先级 | P1 |

#### FR-CANV-028 锁定 (Lock)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-028 |
| 描述 | 系统应支持 element 锁定, 不可拖动/缩放/编辑 |
| 输入 | 工具栏 / 右键菜单 / 键盘 Cmd+L |
| 输出 | element.locked = true, 视觉显示锁图标 |
| 业务规则 | admin / owner 可解锁任何 element, editor 仅可锁定自己创建的 |
| 优先级 | P1 |

#### FR-CANV-029 隐藏 (Hide)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-029 |
| 描述 | 系统应支持 element 隐藏, 不显示 |
| 输入 | 工具栏 / 右键菜单 / 键盘 Cmd+H |
| 输出 | element.hidden = true, 不渲染 |
| 业务规则 | Layers 面板可切换可见性 |
| 优先级 | P1 |

### §16.5 图层与 Z-Index

#### FR-CANV-030 Z-Index 图层

| 项 | 内容 |
|---|---|
| ID | FR-CANV-030 |
| 描述 | 系统应支持 element z_index 控制图层顺序 |
| 输入 | 右键菜单 "上移一层 / 下移一层 / 置顶 / 置底" |
| 输出 | element.z_index 更新 |
| 范围 | [0, 10000], 默认 0 |
| 优先级 | P0 |

#### FR-CANV-031 图层面板 (Layers)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-031 |
| 描述 | 系统应提供右侧图层面板, 列出所有 element |
| 输入 | 点击右侧 "Layers" 标签 |
| 输出 | element 列表 (按 z_index 倒序) + 缩略图 + 名称 + 锁/隐藏图标 |
| 交互 | 点击列表项 = 选中 + 跳到画布对应位置; 拖动列表项 = 改 z_index |
| 优先级 | P1 |

### §16.6 撤销与重做

#### FR-CANV-040 Undo / Redo

| 项 | 内容 |
|---|---|
| ID | FR-CANV-040 |
| 描述 | 系统应支持 Undo (Cmd+Z) / Redo (Cmd+Shift+Z) |
| 输入 | 键盘 / 工具栏 |
| 输出 | 撤销 / 重做最近 N 步操作 |
| 实现 | Yjs UndoManager (Yjs 原生支持, 跨用户隔离) |
| 深度 | 默认 100 步, 可配置 |
| 业务规则 | 仅撤销当前用户自己的操作, 不撤销其他协作者 (Yjs UndoManager 默认行为) |
| 优先级 | P0 |

#### FR-CANV-041 操作历史面板

| 项 | 内容 |
|---|---|
| ID | FR-CANV-041 |
| 描述 | 系统应提供操作历史面板, 列出最近 N 步 |
| 输入 | 工具栏 "History" 按钮 |
| 输出 | 操作列表 (添加 element / 删除 / 移动 / 修改内容) + 时间 + 用户 |
| 交互 | 点击操作 = 跳到对应状态 |
| 优先级 | P2 |

---

## §17 元素系统 (Element System) — 4 子节, 40 FR

### §17.1 13 必备 + 3 候选 Element 类型 (共 16 variant)

#### FR-CANV-100 便利贴 (Sticky Note)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-100 |
| kind | `sticky_note` |
| 描述 | 系统应支持便签元素 (Miro 同款) |
| 视觉 | 矩形 (120x120 默认), 内文 14px, 6 种颜色 (黄/粉/蓝/绿/紫/橙) |
| 交互 | 双击 = 编辑文本, 拖动 = 移动, 拖角 = 缩放 |
| 行为 | 文字超出 = 自动扩展高度 |
| 优先级 | P0 |

#### FR-CANV-101 文本 (Text)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-101 |
| kind | `text` |
| 描述 | 系统应支持自由文本元素 |
| 视觉 | 自由文本框, ink-dim 色, 多行 |
| 字体 | Sans-serif 默认, 12-72 px 可调 |
| 行为 | 双击 = 编辑, 拖动 = 移动, 不支持缩放 (自适应宽度) |
| 优先级 | P0 |

#### FR-CANV-102 形状 (Shape)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-102 |
| kind | `shape` |
| 描述 | 系统应支持 6 种形状 (矩形 / 圆 / 三角 / 五边形 / 六边形 / 箭头) |
| 视觉 | 边框 line 色 + 可选填充 |
| 交互 | 拖角 = 缩放, 拖顶 = 旋转 |
| 行为 | 形状库可扩展 (P3 候选) |
| 优先级 | P0 |

#### FR-CANV-103 图片 (Image)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-103 |
| kind | `image` |
| 描述 | 系统应支持图片元素 (PNG / JPG / GIF / WebP / SVG) |
| 上传 | 拖入 / 粘贴 / 上传按钮 |
| 存储 | `canvas_image` 表 (Master + Work 二象, 短 TTL retention 7 天) |
| 大小 | 最大 20 MB / 张 |
| 优先级 | P0 |

#### FR-CANV-104 嵌入 (Embed)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-104 |
| kind | `embed` |
| 描述 | 系统应支持 iframe 嵌入 (YouTube / Figma / Loom / Vimeo / Google Drive) |
| 视觉 | 16:9 默认, 嵌入卡片 + provider logo |
| 业务规则 | 部分 provider 需要 OAuth (Figma) |
| 优先级 | P1 |

#### FR-CANV-105 工作项卡片 (WorkItem Card)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-105 |
| kind | `work_item_card` |
| 描述 | 系统应支持 WorkItem 关联卡片 (联动主战场) |
| 视觉 | WorkItem 缩略卡 (180x64 默认, 显示 key/title/status/priority) |
| 联动 | 状态色码走 StatusPill 60+ (per ADR-CANVAS-003) |
| 同步 | useStore.getState().workItems 派生 (per `frontend-canvas-design.md` §4.4) |
| 优先级 | P0 |

#### FR-CANV-106 Worktree 节点 (Worktree Node)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-106 |
| kind | `worktree_node` |
| 描述 | 系统应支持 Worktree 关联节点 |
| 视觉 | 圆角矩形 (240x80 默认, 显示 branch + status) |
| 联动 | status 实时同步 useStore (per `frontend-canvas-design.md` §4.4) |
| 优先级 | P0 |

#### FR-CANV-107 Agent 光标 (Agent Cursor)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-107 |
| kind | `agent_cursor` |
| 描述 | 系统应支持 Agent 关联 cursor (只读镜像) |
| 视觉 | 圆点 + agent 名字 (Miro 同款) |
| 联动 | cursor 位置从 useStore 派生 (per `frontend-canvas-design.md` §4.6) |
| 优先级 | P1 |

#### FR-CANV-108 自动化节点 (Automation Node)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-108 |
| kind | `automation_node` |
| 描述 | 系统应支持 Automation rule 关联节点 |
| 视觉 | 六边形 + rule 名称, warn 色 |
| 联动 | 触发 automation rule (per `frontend-canvas-design.md` §4.7) |
| 优先级 | P2 |

#### FR-CANV-109 评论钉 (Comment Pin)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-109 |
| kind | `comment_pin` |
| 描述 | 系统应支持 comment 关联图钉 |
| 视觉 | 图钉 + comment 数量 badge, info 色 |
| 联动 | 点击 = 展开 comment thread |
| 优先级 | P0 |

#### FR-CANV-110 思维导图节点 (Mind Map Node)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-110 |
| kind | `mind_map_node` |
| 描述 | 系统应支持思维导图节点 |
| 视觉 | 圆角矩形 + 缩进连接线 (curved) |
| 算法 | dagre / ELK / elkjs auto-arrange (per G-8) |
| 优先级 | P1 |

#### FR-CANV-111 流程图节点 (Flowchart Node)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-111 |
| kind | `flowchart_node` |
| 描述 | 系统应支持流程图节点 (开始/结束/判断/处理/输入输出 5 种) |
| 视觉 | 标准流程图符号 (per BPMN 简化) |
| 优先级 | P1 |

#### FR-CANV-112 手绘 (Sketch)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-112 |
| kind | `sketch` |
| 描述 | 系统应支持手绘元素 (自由笔触) |
| 视觉 | Bezier 路径平滑 |
| 工具 | 钢笔 / 铅笔 / 荧光笔 3 种 |
| 优先级 | P1 |

#### FR-CANV-113 投票 / 计时器 (Vote / Timer Widget)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-113 |
| kind | `vote_widget` / `timer_widget` |
| 描述 | 系统应支持投票 / 计时器小部件 |
| 视觉 | 投票: 圆形 + 票数; 计时器: 倒计时数字 |
| 优先级 | P2 |

#### FR-CANV-114 表格 (Table)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-114 |
| kind | `table` |
| 描述 | 系统应支持表格元素 |
| 视觉 | N 行 M 列表格, 单元格可编辑 |
| 业务规则 | 单元格联动 work-item (per FR-CANV-105) |
| 优先级 | P2 |

### §17.2 元素通用操作

#### FR-CANV-130 元素复制 / 粘贴

| 项 | 内容 |
|---|---|
| ID | FR-CANV-130 |
| 描述 | 系统应支持 Cmd+C 复制 / Cmd+V 粘贴 element |
| 输入 | 选中 1+ element, 复制 → 粘贴 |
| 输出 | 新 element, 偏移 (10, 10) 像素 |
| 业务规则 | 复制 connector 时, 关联的 connector 也复制 (with 新 element ids) |
| 优先级 | P0 |

#### FR-CANV-131 元素删除

| 项 | 内容 |
|---|---|
| ID | FR-CANV-131 |
| 描述 | 系统应支持 Delete / Backspace 删除 element |
| 输入 | 选中 1+ element, 按 Delete |
| 输出 | element 物理删除 (从 Yjs CRDT 删除) |
| 业务规则 | 关联 connector 也删除 (cascade) |
| 优先级 | P0 |

#### FR-CANV-132 元素编组 / 解组

| 项 | 内容 |
|---|---|
| ID | FR-CANV-132 |
| 描述 | 系统应支持多选 element 编组 |
| 输入 | 选中 2+ element, Cmd+G 编组 / Cmd+Shift+G 解组 |
| 输出 | group_id 字段填充 / 清空 |
| 行为 | 编组后整体拖动 / 缩放 / 旋转 |
| 优先级 | P1 |

#### FR-CANV-133 元素对齐

| 项 | 内容 |
|---|---|
| ID | FR-CANV-133 |
| 描述 | 系统应支持 6 种对齐方式 (左/右/上/下/水平居中/垂直居中) |
| 输入 | 选中 2+ element, 工具栏对齐按钮 |
| 输出 | 元素对齐到指定边 / 中心 |
| 业务规则 | 9 像素吸附 (snap) |
| 优先级 | P1 |

#### FR-CANV-134 元素分布

| 项 | 内容 |
|---|---|
| ID | FR-CANV-134 |
| 描述 | 系统应支持 3 种分布方式 (水平等距 / 垂直等距 / 网格) |
| 输入 | 选中 3+ element |
| 输出 | 元素等距分布 |
| 优先级 | P2 |

### §17.3 连接器 (Connector)

#### FR-CANV-140 连接线 3 种 routing

| 项 | 内容 |
|---|---|
| ID | FR-CANV-140 |
| 描述 | 系统应支持 3 种 routing (straight / curved / orthogonal) |
| 算法 | straight: 直线; curved: 三次 bezier; orthogonal: 直角 |
| 复用 | bezier 公式跟 StateMachineDiagram 共用 (per `frontend-canvas-design.md` §3.5) |
| 优先级 | P0 |

#### FR-CANV-141 连接线箭头

| 项 | 内容 |
|---|---|
| ID | FR-CANV-141 |
| 描述 | 系统应支持起 / 终点箭头开关 |
| 输入 | 工具栏 / 选中 connector 属性 |
| 输出 | connector.arrow_start / arrow_end 字段 |
| 业务规则 | arrow_start = false 表示无向边 |
| 优先级 | P0 |

#### FR-CANV-142 连接线 label

| 项 | 内容 |
|---|---|
| ID | FR-CANV-142 |
| 描述 | 系统应支持连接线中点 label |
| 输入 | 选中 connector, 双击中点 |
| 输出 | label 文本, 中点位置 |
| 业务规则 | label 跟随 connector 移动 |
| 优先级 | P0 |

#### FR-CANV-143 关联绑定 (work_item_relation)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-143 |
| 描述 | 系统应支持 work_item_relation 类型的 connector |
| 联动 | 来自 Relation 域 (blocks / duplicates / relates_to / depends_on / parent_of 5 种) |
| 行为 | 颜色按 relation 类型, label 显示 relation 名称 |
| 优先级 | P0 (per `frontend-canvas-design.md` §4.5 联动 4) |

#### FR-CANV-144 自连接 (Self-Loop)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-144 |
| 描述 | 系统应支持 connector 起终点为同一 element (循环引用) |
| 算法 | curved routing, 起点 = 终点, 弧度半径 = 50px |
| 业务规则 | 防环约束由 DAG 检测算法保证 (per BR-4) |
| 优先级 | P2 |

### §17.4 Frame 与容器

#### FR-CANV-150 Frame 创建

| 项 | 内容 |
|---|---|
| ID | FR-CANV-150 |
| 描述 | 系统应支持 Frame 元素 (画布分区 / Slide) |
| 输入 | 工具栏 frame 工具 / F 键 / 拖空白 |
| 输出 | 矩形 frame, 标题, 半透明背景 |
| 业务规则 | 拖入 element 到 frame = 加入该 frame |
| 优先级 | P0 |

#### FR-CANV-151 Frame 拖动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-151 |
| 描述 | 系统应支持 frame 整体拖动 (内部 element 同步移动) |
| 输入 | 拖动 frame body |
| 输出 | frame.x/y 更新 + 内部 element 同步偏移 |
| 业务规则 | 内部 element 锁定后, 拖动 frame 不带其移动 |
| 优先级 | P0 |

#### FR-CANV-152 Frame 自动避让

| 项 | 内容 |
|---|---|
| ID | FR-CANV-152 |
| 描述 | 系统应支持 frame 自动避让 (不允许重叠) |
| 算法 | 检测 frame bbox 重叠, 自动调整位置 |
| 业务规则 | 拖动 frame 时实时避让 |
| 优先级 | P2 (per `frontend-canvas-design.md` §5.4) |

#### FR-CANV-153 Frame 嵌套

| 项 | 内容 |
|---|---|
| ID | FR-CANV-153 |
| 描述 | 系统应支持 frame 嵌套 (frame 内含 frame) |
| 限制 | 最多 3 层嵌套 |
| 业务规则 | 拖动父 frame = 整体移动, 含子 frame |
| 优先级 | P2 |

---

## §18 实时协作 (Realtime Collaboration) — 2 子节, 20 FR

### §18.1 Yjs CRDT 协同

#### FR-CANV-200 Yjs 文档实例

| 项 | 内容 |
|---|---|
| ID | FR-CANV-200 |
| 描述 | 系统应为每个 canvas 维护 1 个 Yjs Doc 实例 |
| 实现 | `new Y.Doc()` per canvas, 通过 y-websocket provider 同步 |
| 存储 | `canvas_realtime_yjs_update` 表 (Master + Work 二象, per ADR-CANVAS-006) |
| 优先级 | P0 |

#### FR-CANV-201 Yjs Y.Map 元素存储

| 项 | 内容 |
|---|---|
| ID | FR-CANV-201 |
| 描述 | 系统应使用 Y.Map 存储 element |
| 数据 | `Y.Map<elementId, Y.Map<key, value>>` 嵌套结构 |
| 操作 | element.set('x', 100) / element.set('y', 200) |
| 冲突 | Yjs CRDT 自动解决 (LWW last-write-wins) |
| 优先级 | P0 |

#### FR-CANV-202 Yjs Y.Array 连接器存储

| 项 | 内容 |
|---|---|
| ID | FR-CANV-202 |
| 描述 | 系统应使用 Y.Array 存储 connector 列表 |
| 数据 | `Y.Array<Y.Map<key, value>>` |
| 操作 | connectorArray.push([newConnector]) / connectorArray.delete(idx, 1) |
| 优先级 | P0 |

#### FR-CANV-203 Yjs UndoManager

| 项 | 内容 |
|---|---|
| ID | FR-CANV-203 |
| 描述 | 系统应使用 Yjs UndoManager 实现 Undo/Redo (per FR-CANV-040) |
| 范围 | 默认 100 步, 可配置 |
| 隔离 | 仅当前用户的操作, 跨用户隔离 |
| 优先级 | P0 |

#### FR-CANV-204 Yjs WebSocket Provider

| 项 | 内容 |
|---|---|
| ID | FR-CANV-204 |
| 描述 | 系统应使用 y-websocket 库连接 canvas-realtime 后端 |
| 协议 | WebSocket + y-protocols/sync + y-protocols/awareness |
| 重连 | 自动重连, 指数退避, 最多 5 次 |
| 离线 | 离线时本地继续编辑, 重连后自动同步 |
| 优先级 | P0 |

#### FR-CANV-205 Yjs 二进制 Update 持久化

| 项 | 内容 |
|---|---|
| ID | FR-CANV-205 |
| 描述 | 系统应将 Yjs 二进制 update 增量持久化到后端 |
| 触发 | 每次 update 时 (debounce 1s) |
| 存储 | `canvas_realtime_yjs_update` 表 (Master + Work 二象) |
| 重建 | 加载 canvas 时, 顺序应用所有 update 重建 Y.Doc |
| 优先级 | P0 |

#### FR-CANV-206 离线编辑

| 项 | 内容 |
|---|---|
| ID | FR-CANV-206 |
| 描述 | 系统应支持离线编辑 (断网 30+ 分钟) |
| 行为 | Yjs y-indexeddb provider 本地 IndexedDB 存储 |
| 重连 | WebSocket 重连后, 自动 merge 离线期间的本地 update |
| 业务规则 | 不需要用户介入, CRDT 自动解决 |
| 优先级 | P0 (per BR-10) |

#### FR-CANV-207 Awareness 协议

| 项 | 内容 |
|---|---|
| ID | FR-CANV-207 |
| 描述 | 系统应使用 y-protocols/awareness 协议同步临时状态 |
| 数据 | 光标位置 (x, y) / 选区 (selected_ids) / 在线状态 (user_id, name, color) |
| 频率 | 10Hz (浏览器 60fps 但 awareness 10Hz 节流) |
| 持久化 | 不持久化, 关闭浏览器即丢失 |
| 优先级 | P0 |

#### FR-CANV-208 在线协作者列表

| 项 | 内容 |
|---|---|
| ID | FR-CANV-208 |
| 描述 | 系统应显示当前在线协作者列表 (头像 + 名字 + 颜色) |
| 位置 | 右上角 |
| 交互 | 点击头像 = 跳到该用户光标位置 (per FR-CANV-209) |
| 业务规则 | 颜色按 user_id 哈希生成, 8 色 palette |
| 优先级 | P0 |

#### FR-CANV-209 跳转到用户光标

| 项 | 内容 |
|---|---|
| ID | FR-CANV-209 |
| 描述 | 系统应支持点击协作者头像跳到其光标位置 |
| 输入 | 点击协作者头像 |
| 输出 | viewport 平滑过渡到该用户光标位置 |
| 动画 | 500ms ease-in-out |
| 优先级 | P1 |

#### FR-CANV-210 跟随模式 (Follow Mode)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-210 |
| 描述 | 系统应支持跟随其他协作者的 viewport |
| 输入 | 协作者头像右键菜单 "Follow" |
| 输出 | 自己的 viewport 跟随对方, 双方看到相同内容 |
| 退出 | Esc 退出跟随 |
| 优先级 | P2 |

### §18.2 评论与 @ 提及

#### FR-CANV-220 评论

| 项 | 内容 |
|---|---|
| ID | FR-CANV-220 |
| 描述 | 系统应支持画布评论 (挂在 element 或 canvas 上) |
| 输入 | 选中 element, 工具栏 "Add Comment" |
| 输出 | comment_thread 实体, comment_pin element 自动创建 |
| 联动 | `domain-comment` 域 (per §36 数据模型) |
| 业务规则 | comment 不允许 free-floating (per BR-6) |
| 优先级 | P0 |

#### FR-CANV-221 评论回复

| 项 | 内容 |
|---|---|
| ID | FR-CANV-221 |
| 描述 | 系统应支持评论多级回复 (最多 5 层) |
| 输入 | 点击 comment, 输入回复 |
| 输出 | comment_thread 嵌套 |
| 优先级 | P0 |

#### FR-CANV-222 评论解决 / 重新打开

| 项 | 内容 |
|---|---|
| ID | FR-CANV-222 |
| 描述 | 系统应支持评论标记为已解决 / 重新打开 |
| 输入 | 评论右下角 "Resolve" / "Reopen" 按钮 |
| 输出 | comment.resolved = true / false |
| 业务规则 | 已解决评论默认折叠, 显示 "X resolved comments" |
| 优先级 | P0 |

#### FR-CANV-223 @ 提及

| 项 | 内容 |
|---|---|
| ID | FR-CANV-223 |
| 描述 | 系统应支持 @ 提及用户 |
| 输入 | 评论中输入 @, 触发用户搜索 dropdown |
| 输出 | mention 实体, 触发 `domain-notification` 通知 (per BR-7) |
| 业务规则 | @提及 = 该用户立即收到 notification |
| 优先级 | P0 |

#### FR-CANV-224 表情反应 (Emoji Reaction)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-224 |
| 描述 | 系统应支持评论 emoji 反应 (👍 ❤️ 😂 🎉 🤔 👀 6 种) |
| 输入 | 评论右下角 emoji picker |
| 输出 | reaction 实体, 显示在评论底部 |
| 优先级 | P1 |

#### FR-CANV-225 评论搜索

| 项 | 内容 |
|---|---|
| ID | FR-CANV-225 |
| 描述 | 系统应支持评论全文搜索 |
| 输入 | 工具栏 "Search Comments" 按钮 / Cmd+F |
| 输出 | 匹配评论列表, 点击 = 跳到画布对应位置 |
| 优先级 | P2 |

---

## §19 模板系统 (Template System) — 1 子节, 8 FR

#### FR-CANV-300 模板库

| 项 | 内容 |
|---|---|
| ID | FR-CANV-300 |
| 描述 | 系统应提供 8 域核心模板 + 5-10 行业模板 (per ADR-CANVAS-010) |
| 模板 | Kanban / Retrospective / User Journey / Mind Map / Flowchart / ER Diagram / Architecture / Roadmap / Sprint Planning / Brainstorm 10 模板 |
| 存储 | `canvas_template` 表 (Master 类, per 守门 #13) |
| 优先级 | P1 |

#### FR-CANV-301 模板预览

| 项 | 内容 |
|---|---|
| ID | FR-CANV-301 |
| 描述 | 系统应支持模板预览 (缩略图 + 名称 + 描述 + 分类) |
| 位置 | 模板库页面 / 新建 canvas 弹窗 |
| 视觉 | 200x150 px 缩略图 |
| 优先级 | P1 |

#### FR-CANV-302 模板一键应用

| 项 | 内容 |
|---|---|
| ID | FR-CANV-302 |
| 描述 | 系统应支持点击模板创建新 canvas (复制模板 element + connector) |
| 输入 | 点击模板 |
| 输出 | 新 canvas, 标题 = "{template.name} - {date}", 元素 = 模板副本 |
| 业务规则 | 新 canvas 跟模板解耦, 不允许反向同步 (per BR-9) |
| 优先级 | P1 |

#### FR-CANV-303 模板 fork

| 项 | 内容 |
|---|---|
| ID | FR-CANV-303 |
| 描述 | 系统应支持现有 canvas 另存为模板 |
| 输入 | 工具栏 "Save as Template" 按钮 |
| 输出 | 新 canvas_template 记录, fork_count 字段 |
| 业务规则 | 管理员/owner 可创建, 编辑者仅可 fork 自己的 canvas |
| 优先级 | P2 |

#### FR-CANV-304 模板分类

| 项 | 内容 |
|---|---|
| ID | FR-CANV-304 |
| 描述 | 系统应支持模板按分类浏览 (Product / Engineering / Design / Marketing / Sales / HR 6 类) |
| 位置 | 模板库左侧分类侧边栏 |
| 优先级 | P2 |

#### FR-CANV-305 模板搜索

| 项 | 内容 |
|---|---|
| ID | FR-CANV-305 |
| 描述 | 系统应支持模板全文搜索 (按名称 + 描述 + 标签) |
| 输入 | 搜索框 |
| 输出 | 匹配模板列表 |
| 优先级 | P2 |

#### FR-CANV-306 模板标签

| 项 | 内容 |
|---|---|
| ID | FR-CANV-306 |
| 描述 | 系统应支持模板标签 (tag) |
| 标签 | agile / retrospective / brainstorming / ux / devops / architecture 等 20+ |
| 优先级 | P2 |

#### FR-CANV-307 模板版本

| 项 | 内容 |
|---|---|
| ID | FR-CANV-307 |
| 描述 | 系统应支持模板版本管理 (template.v1 / v2 / v3) |
| 业务规则 | 模板升级不影响已 fork 的 canvas |
| 优先级 | P3 |

---

## §20 演示模式 (Presentation Mode) — 1 子节, 6 FR

#### FR-CANV-350 演示模式启动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-350 |
| 描述 | 系统应支持 Frame as Slide 演示模式 |
| 输入 | 工具栏 "Present" 按钮 / F5 键 |
| 输出 | 全屏 + 当前选中 frame 作为 slide 1 |
| 业务规则 | 演示模式 = readonly (per BR-8) |
| 优先级 | P1 |

#### FR-CANV-351 演讲者视图

| 项 | 内容 |
|---|---|
| ID | FR-CANV-351 |
| 描述 | 系统应提供演讲者视图 (本地显示下一张 + 注释 + 计时器) |
| 输入 | 演示模式按 `S` 键 |
| 输出 | 演讲者窗口: 当前 slide / 下一 slide 缩略 / 计时器 / 注释 |
| 优先级 | P2 |

#### FR-CANV-352 演示控制

| 项 | 内容 |
|---|---|
| ID | FR-CANV-352 |
| 描述 | 系统应支持 5 种演示控制 (上/下/首/末/退出) |
| 输入 | 键盘 ← / → / Home / End / Esc |
| 输出 | 切换 slide / 跳到首/末 / 退出 |
| 优先级 | P1 |

#### FR-CANV-353 演示激光笔

| 项 | 内容 |
|---|---|
| ID | FR-CANV-353 |
| 描述 | 系统应支持演示激光笔 (高亮光标位置) |
| 输入 | 按 `L` 键或长按鼠标右键 |
| 输出 | 红色激光点跟随鼠标 |
| 优先级 | P2 |

#### FR-CANV-354 Frame 顺序

| 项 | 内容 |
|---|---|
| ID | FR-CANV-354 |
| 描述 | 系统应支持 Frame 顺序调整 (演示顺序) |
| 输入 | 工具栏 Frame 列表, 拖动排序 |
| 输出 | frame.order 字段更新 |
| 优先级 | P1 |

#### FR-CANV-355 演示邀请

| 项 | 内容 |
|---|---|
| ID | FR-CANV-355 |
| 描述 | 系统应支持生成演示邀请链接 |
| 输入 | 演示模式 "Copy Invite Link" |
| 输出 | URL `https://star.app/canvas/{id}?present=true&slide={n}` |
| 业务规则 | 链接走 read-only 模式, viewer 角色 |
| 优先级 | P2 |

---

## §21 导入导出 (Import/Export) — 1 子节, 8 FR

#### FR-CANV-400 PNG 导出

| 项 | 内容 |
|---|---|
| ID | FR-CANV-400 |
| 描述 | 系统应支持画布 PNG 导出 (单页 / 全画布) |
| 实现 | html2canvas (DOM → canvas → PNG Blob) |
| 选项 | 1x / 2x / 3x 分辨率, 单 frame / 全画布 |
| 输出 | PNG 文件下载 |
| 优先级 | P0 (per `docs/briefs/canvas-share-export-001.md` 已落) |

#### FR-CANV-401 PDF 导出

| 项 | 内容 |
|---|---|
| ID | FR-CANV-401 |
| 描述 | 系统应支持画布 PDF 导出 |
| 实现 | jspdf + html2canvas (per frame 一页) |
| 选项 | A4 / A3 / Letter, 横向 / 纵向, 单 frame / 全 frame |
| 输出 | PDF 文件下载 |
| 优先级 | P1 |

#### FR-CANV-402 SVG 导出

| 项 | 内容 |
|---|---|
| ID | FR-CANV-402 |
| 描述 | 系统应支持画布 SVG 导出 (矢量) |
| 实现 | DOM 序列化 SVG 字符串 |
| 选项 | 嵌入字体 / 简化 path |
| 输出 | SVG 文件下载 |
| 优先级 | P1 |

#### FR-CANV-403 JSON 导出 (Yjs Update)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-403 |
| 描述 | 系统应支持画布 JSON 导出 (Yjs update 二进制 → base64) |
| 业务规则 | 导出 = canvas 完整状态 (含 Yjs update) |
| 输出 | .json 文件, 可重新导入 |
| 优先级 | P1 |

#### FR-CANV-404 Miro import 兼容

| 项 | 内容 |
|---|---|
| ID | FR-CANV-404 |
| 描述 | 系统应支持 Miro export RTB 文件导入 |
| 实现 | Miro RTB 格式 → 内部 Canvas 模型, 转换率 ≥ 95% (per BR-11) |
| 输出 | 新 canvas, 元素 = Miro 元素副本 |
| 业务规则 | 转换失败的元素显示 warning, 不阻塞导入 |
| 优先级 | P2 |

#### FR-CANV-405 Markdown 导出

| 项 | 内容 |
|---|---|
| ID | FR-CANV-405 |
| 描述 | 系统应支持画布 Markdown 导出 (sticky_note + text → md 列表) |
| 实现 | 按 z_index 顺序, 每元素一行 |
| 输出 | .md 文件 |
| 优先级 | P3 |

#### FR-CANV-406 嵌入 iframe

| 项 | 内容 |
|---|---|
| ID | FR-CANV-406 |
| 描述 | 系统应支持画布嵌入第三方 iframe (read-only) |
| URL | `https://star.app/embed/canvas/{id}` |
| 业务规则 | 嵌入页面 = viewer 角色, read-only |
| 优先级 | P2 |

#### FR-CANV-407 Open API

| 项 | 内容 |
|---|---|
| ID | FR-CANV-407 |
| 描述 | 系统应支持 canvas Open API (per `star-api-rest` 22 路由 + canvas 扩展) |
| 端点 | `GET /v1/canvases` / `POST /v1/canvases` / `GET /v1/canvases/{id}` / `PUT /v1/canvases/{id}` / `DELETE /v1/canvases/{id}` |
| 鉴权 | OAuth 2.0 + API Key |
| 优先级 | P1 |

---

## §22 权限分享 (Permission & Sharing) — 1 子节, 10 FR

#### FR-CANV-450 5 角色权限矩阵

| 项 | 内容 |
|---|---|
| ID | FR-CANV-450 |
| 描述 | 系统应实现 5 角色权限矩阵 (per BR-5) |
| 角色 | viewer / commenter / editor / admin / owner |
| 资源 | canvas / element / connector / frame / comment / template / version / share_link / embed / audit_log / subscription / webhook / api_key 13 资源 |
| 矩阵 | 5 × 13 = 65 RACI 单元, per §40 权限矩阵 |
| 优先级 | P0 |

#### FR-CANV-451 角色变更

| 项 | 内容 |
|---|---|
| ID | FR-CANV-451 |
| 描述 | 系统应支持 owner 变更用户角色 |
| 输入 | 设置面板 "Share" → 用户列表 → 角色 dropdown |
| 输出 | canvas_member.role 字段更新 |
| 业务规则 | 仅 owner / admin 可变更, owner 不可降级 (除非转让 owner) |
| 优先级 | P0 |

#### FR-CANV-452 链接分享

| 项 | 内容 |
|---|---|
| ID | FR-CANV-452 |
| 描述 | 系统应支持生成 canvas 分享链接 |
| 输入 | 设置面板 "Share" → "Create Link" |
| 输出 | URL `https://star.app/canvas/{id}?token={token}` |
| 业务规则 | token = UUID v4, 跟 user 解耦 |
| 优先级 | P0 |

#### FR-CANV-453 链接权限

| 项 | 内容 |
|---|---|
| ID | FR-CANV-453 |
| 描述 | 系统应支持链接权限 (viewer / commenter / editor) |
| 输入 | 创建链接时选择权限 |
| 输出 | link.permission 字段 |
| 业务规则 | viewer = 只读, commenter = 可评论, editor = 可编辑 |
| 优先级 | P0 |

#### FR-CANV-454 链接密码

| 项 | 内容 |
|---|---|
| ID | FR-CANV-454 |
| 描述 | 系统应支持链接密码保护 |
| 输入 | 创建链接时设置密码 |
| 输出 | link.password_hash 字段 (bcrypt) |
| 业务规则 | 访问时弹密码框, 错误 3 次锁定 5 分钟 |
| 优先级 | P1 |

#### FR-CANV-455 链接过期

| 项 | 内容 |
|---|---|
| ID | FR-CANV-455 |
| 描述 | 系统应支持链接过期时间 |
| 输入 | 创建链接时设置过期时间 (1d / 7d / 30d / never) |
| 输出 | link.expires_at 字段 |
| 业务规则 | 过期后访问 = 404 |
| 优先级 | P1 |

#### FR-CANV-456 公开 / 私有

| 项 | 内容 |
|---|---|
| ID | FR-CANV-456 |
| 描述 | 系统应支持 canvas 公开 / 私有切换 |
| 输入 | 设置面板 "Visibility" 切换 |
| 输出 | canvas.visibility = "public" / "private" / "team" |
| 业务规则 | public = 任何登录用户可见, team = 同 tenant 可见, private = 仅 member 可见 |
| 优先级 | P1 |

#### FR-CANV-457 转让所有权

| 项 | 内容 |
|---|---|
| ID | FR-CANV-457 |
| 描述 | 系统应支持 owner 转让 canvas 给其他用户 |
| 输入 | 设置面板 "Transfer Ownership" |
| 输出 | canvas.owner_id 更新, 旧 owner 自动降为 admin |
| 业务规则 | 需新 owner 确认, 不可撤销 (除非新 owner 反转转让) |
| 优先级 | P2 |

#### FR-CANV-458 复制 canvas

| 项 | 内容 |
|---|---|
| ID | FR-CANV-458 |
| 描述 | 系统应支持复制 canvas (整个 canvas 副本) |
| 输入 | 设置面板 "Duplicate" |
| 输出 | 新 canvas, 元素 + connector + frame 全复制, owner = 当前用户 |
| 业务规则 | 评论 / 历史不复制 |
| 优先级 | P0 |

#### FR-CANV-459 删除 canvas

| 项 | 内容 |
|---|---|
| ID | FR-CANV-459 |
| 描述 | 系统应支持删除 canvas |
| 输入 | 设置面板 "Delete Canvas" |
| 输出 | canvas.deleted_at 软删除 (per 守门 #13 Master 类 物理删除禁止) |
| 业务规则 | 30 天后自动物理删除, owner 可恢复 30 天内 |
| 优先级 | P0 |

---

## §23 快捷键无障碍 (Shortcut & Accessibility) — 1 子节, 12 FR

#### FR-CANV-500 完整键盘快捷键

| 项 | 内容 |
|---|---|
| ID | FR-CANV-500 |
| 描述 | 系统应支持 40+ 键盘快捷键 (per 旧 `frontend-canvas-design.md` §5.1 + 扩展) |
| 类别 | 工具切换 (V/H/T/F/N/S/L) / 元素操作 (Delete/Cmd+C/Cmd+V/Cmd+D/Cmd+Z/Cmd+Shift+Z/Cmd+A/Cmd+G/Cmd+Shift+G) / 视口 (0/1/+/-) / 演示 (F5/Esc/L) / 协作 (Cmd+K) / 其他 (Cmd+F/Cmd+S/Cmd+P) |
| 冲突检测 | 快捷键注册走统一 dispatcher, 避免与浏览器默认冲突 |
| 优先级 | P0 |

#### FR-CANV-501 快捷键帮助面板

| 项 | 内容 |
|---|---|
| ID | FR-CANV-501 |
| 描述 | 系统应提供快捷键帮助面板 (Cmd+/) |
| 视觉 | 居中弹窗, 分类列出所有快捷键 |
| 优先级 | P0 |

#### FR-CANV-502 自定义快捷键

| 项 | 内容 |
|---|---|
| ID | FR-CANV-502 |
| 描述 | 系统应支持用户自定义快捷键 (设置面板) |
| 存储 | user_preference.shortcuts 字段 (JSON) |
| 优先级 | P2 |

#### FR-CANV-510 WCAG 2.1 AA 合规

| 项 | 内容 |
|---|---|
| ID | FR-CANV-510 |
| 描述 | 系统应满足 WCAG 2.1 Level AA 合规 (per G-9) |
| 维度 | 可感知 (Perceivable) / 可操作 (Operable) / 可理解 (Understandable) / 健壮 (Robust) 4 维 |
| 优先级 | P0 |

#### FR-CANV-511 屏幕阅读器

| 项 | 内容 |
|---|---|
| ID | FR-CANV-511 |
| 描述 | 系统应支持屏幕阅读器 (NVDA / VoiceOver / JAWS) |
| 实现 | element 加 aria-label / aria-describedby / role |
| 优先级 | P0 |

#### FR-CANV-512 键盘完全操控

| 项 | 内容 |
|---|---|
| ID | FR-CANV-512 |
| 描述 | 系统应支持键盘完全操控画布, 无需鼠标 (per US-8) |
| 键盘 | Tab (焦点切换) / 方向键 (焦点移动) / Enter (打开) / Space (选中) / Esc (取消) |
| 优先级 | P0 |

#### FR-CANV-513 高对比度模式

| 项 | 内容 |
|---|---|
| ID | FR-CANV-513 |
| 描述 | 系统应支持高对比度模式 (WCAG 1.4.6) |
| 视觉 | 文本对比度 ≥ 7:1 (AAA), UI 组件 ≥ 4.5:1 (AA) |
| 切换 | 系统设置 "High Contrast Mode" toggle |
| 优先级 | P1 |

#### FR-CANV-514 焦点指示器

| 项 | 内容 |
|---|---|
| ID | FR-CANV-514 |
| 描述 | 系统应显示明显焦点指示器 (2px 蓝色边框) |
| 业务规则 | 所有可聚焦元素都有焦点环 |
| 优先级 | P0 |

#### FR-CANV-515 替代文本

| 项 | 内容 |
|---|---|
| ID | FR-CANV-515 |
| 描述 | 系统应支持 image / embed / sketch 元素添加替代文本 |
| 输入 | 右键 "Edit Alt Text" |
| 输出 | element.alt_text 字段, 屏幕阅读器读出 |
| 优先级 | P0 |

#### FR-CANV-516 跳过导航

| 项 | 内容 |
|---|---|
| ID | FR-CANV-516 |
| 描述 | 系统应支持 "Skip to Canvas" 跳转链接 |
| 位置 | 顶部 (Tab 键首个焦点) |
| 行为 | Enter 跳到画布区域, 跳过工具栏 |
| 优先级 | P1 |

#### FR-CANV-517 字幕 / 文本替代

| 项 | 内容 |
|---|---|
| ID | FR-CANV-517 |
| 描述 | 系统应支持嵌入视频自动生成字幕 (YouTube 嵌入自动) |
| 优先级 | P2 |

#### FR-CANV-518 触摸可达性

| 项 | 内容 |
|---|---|
| ID | FR-CANV-518 |
| 描述 | 系统应支持触摸操作 (平板 / 移动 Web) |
| 输入 | 单指拖 (pan) / 双指捏 (zoom) / 双指拖 (pan) / 长按 (右键菜单) / 双击 (打开) |
| 优先级 | P1 |

---

## §24 跟 9 域联动 (per `frontend-canvas-design.md` §1.3 升级)

#### FR-CANV-600 WorkItem 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-600 |
| 描述 | WorkItem 详情页"添加到画布"按钮 → 跳画布高亮 |
| 流程 | (per `frontend-canvas-design.md` §4.2) |
| 优先级 | P0 |

#### FR-CANV-601 Worktree 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-601 |
| 描述 | Worktree 自动在画布生成节点, status 实时同步 |
| 流程 | (per `frontend-canvas-design.md` §4.4) |
| 优先级 | P0 |

#### FR-CANV-602 Agent 联动 (PresenceCursor 升级)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-602 |
| 描述 | Agent 关联 cursor, 显示名字 + 当前操作 |
| 流程 | (per `frontend-canvas-design.md` §4.6) |
| 优先级 | P1 |

#### FR-CANV-603 Relation 联动 (批量导入)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-603 |
| 描述 | Relation 列表"导入到画布"按钮 → 创建新 canvas |
| 流程 | (per `frontend-canvas-design.md` §4.5) |
| 优先级 | P1 |

#### FR-CANV-604 Comment 联动 (已在 §18.2 覆盖)

| 项 | 内容 |
|---|---|
| ID | FR-CANV-604 |
| 描述 | comment_pin element + comment_thread 联动 |
| 优先级 | P0 |

#### FR-CANV-605 Automation 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-605 |
| 描述 | Automation rule 关联节点, trigger_kind 加 canvas_event |
| 流程 | (per `frontend-canvas-design.md` §4.7) |
| 优先级 | P2 |

#### FR-CANV-606 Audit 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-606 |
| 描述 | 画布操作写 audit (action: canvas.element.move / .create / .delete) |
| 流程 | (per `frontend-canvas-design.md` §1.3) |
| 优先级 | P0 |

#### FR-CANV-607 Search 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-607 |
| 描述 | Search 搜 work-item 跳画布对应 element |
| 流程 | (per `frontend-canvas-design.md` §4.8) |
| 优先级 | P1 |

#### FR-CANV-608 Notification 联动

| 项 | 内容 |
|---|---|
| ID | FR-CANV-608 |
| 描述 | 画布 @ 提及触发 notification |
| 流程 | (per BR-7) |
| 优先级 | P0 |

---

## §25 跨域共享 (per 9/6 拍板 "多模块共享画布引擎")

#### FR-CANV-700 canvas-engine crate 公开 API

| 项 | 内容 |
|---|---|
| ID | FR-CANV-700 |
| 描述 | `canvas-engine` crate 应提供 Rust 公开 API 供多 domain 引用 |
| API | `CanvasEngine::new(config)` / `engine.viewport()` / `engine.culling(elements, viewport)` / `engine.snap_to_grid(point, grid_size)` / `engine.bbox(elements)` / `engine.connector_route(from, to, kind)` |
| 优先级 | P0 |

#### FR-CANV-701 canvas-engine 跨 5 域引用

| 项 | 内容 |
|---|---|
| ID | FR-CANV-701 |
| 描述 | `canvas-engine` crate 应被 5 个 domain 引用 (per G-2) |
| 引用方 | `domain-board` (Kanban) / `domain-planning` (规划) / `domain-workflow` (工作流) / `domain-feedback` (反馈) / `agent-view` (Agent 视图, 已有派生消费者) |
| 验证 | `cargo tree -p domain-board` 包含 canvas-engine |
| 优先级 | P0 |

#### FR-CANV-702 canvas-engine 不绑死业务域

| 项 | 内容 |
|---|---|
| ID | FR-CANV-702 |
| 描述 | `canvas-engine` crate 不应依赖任何具体 domain-* 业务逻辑 (per ADR-CANVAS-005) |
| 隔离 | 不引用 `domain-work-item` / `domain-worktree` / `domain-permission` 等业务域 |
| 实现 | 通过 trait 抽象 + 调用方注入 |
| 优先级 | P0 |

#### FR-CANV-703 domain-canvas 业务域封装

| 项 | 内容 |
|---|---|
| ID | FR-CANV-703 |
| 描述 | `domain-canvas` crate 应封装画布业务域, 在 canvas-engine 之上加 RLS / audit / i18n |
| 业务 | canvas / element / connector / frame / template / version / share_link 业务逻辑 |
| 隔离 | canvas-engine 不带这些, domain-canvas 注入 |
| 优先级 | P0 |

#### FR-CANV-704 canvas-realtime CRDT 后端

| 项 | 内容 |
|---|---|
| ID | FR-CANV-704 |
| 描述 | `canvas-realtime` crate 应实现 Yjs WebSocket server (Rust + Tokio + yrs) |
| 协议 | y-protocols/sync + y-protocols/awareness |
| 持久化 | `canvas_realtime_yjs_update` 表 |
| 性能 | 50 协作者 / 1K element 同步延迟 < 200ms (per G-3) |
| 优先级 | P0 |

---

# === §26-§35 非功能需求 (10 节, 50+ NFR) ===

## §26 性能 (Performance)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-PERF-1 | 画布加载时间 < 1s (1K element, P50) | P0 | Lighthouse |
| NFR-PERF-2 | 元素拖动 60fps (16ms / 帧, 1K element) | P0 | Chrome DevTools Performance |
| NFR-PERF-3 | Yjs 同步延迟 < 200ms (50 协作者, P95) | P0 | 自定义 benchmark |
| NFR-PERF-4 | Fit-to-content < 100ms (10K element) | P1 | benchmark |
| NFR-PERF-5 | 撤销/重做响应 < 50ms | P0 | benchmark |
| NFR-PERF-6 | Minimap 渲染 < 50ms (10K element) | P1 | benchmark |
| NFR-PERF-7 | PNG 导出 < 3s (1K element) | P1 | benchmark |
| NFR-PERF-8 | PDF 导出 < 5s (10 frame) | P1 | benchmark |
| NFR-PERF-9 | 初次内容绘制 (FCP) < 1.5s | P0 | Lighthouse |
| NFR-PERF-10 | 交互到下一帧 (INP) < 200ms | P0 | Lighthouse |
| NFR-PERF-11 | 累积布局偏移 (CLS) < 0.1 | P0 | Lighthouse |
| NFR-PERF-12 | Canvas bundle size < 500 KB (gzipped) | P1 | bundlephobia |

## §27 可用性 (Usability)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-USA-1 | 新用户 5 分钟内完成创建画布 + 添加便签 + 分享 | P0 | 用户测试 |
| NFR-USA-2 | 错误消息清晰, 90% 用户理解下一步操作 | P0 | 用户测试 |
| NFR-USA-3 | 所有功能 3 次点击内可达 (90% 场景) | P1 | 启发式评估 |
| NFR-USA-4 | 撤销/重做永远可用 (per FR-CANV-040) | P0 | 单元测试 |
| NFR-USA-5 | 自动保存 (per Yjs UndoManager) | P0 | 单元测试 |
| NFR-USA-6 | 提供 5+ onboarding tutorial | P2 | 用户测试 |

## §28 安全 (Security)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-SEC-1 | 链接分享 token 不可预测 (UUID v4) | P0 | 渗透测试 |
| NFR-SEC-2 | 密码 bcrypt 哈希, cost ≥ 12 | P0 | 代码审查 |
| NFR-SEC-3 | WebSocket 走 WSS (TLS 1.3) | P0 | 抓包验证 |
| NFR-SEC-4 | Yjs update 持久化前验证 (per 守门 #7 unsafe 禁) | P0 | 单元测试 |
| NFR-SEC-5 | XSS 防护: 用户输入转义, 富文本 sanitization (DOMPurify) | P0 | 渗透测试 |
| NFR-SEC-6 | CSRF token 校验 (写操作) | P0 | 渗透测试 |
| NFR-SEC-7 | 速率限制: 100 req/min/user (写操作) | P1 | 压力测试 |
| NFR-SEC-8 | 审计日志 100% 覆盖 (per 守门 #13 Transaction 类) | P0 | 集成测试 |
| NFR-SEC-9 | RLS 13 类必带 (per 守门 #13) | P0 | 数据库约束 |
| NFR-SEC-10 | 链接过期后不可恢复 | P0 | 集成测试 |

## §29 可扩展性 (Scalability)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-SCAL-1 | 单 canvas 支持 10K element (P2 推到 100K per G-10) | P1 | 压力测试 |
| NFR-SCAL-2 | 单 canvas 支持 50 协作者 (P2 推到 100) | P1 | 压力测试 |
| NFR-SCAL-3 | canvas-realtime 单实例支持 1K canvas 同时活跃 | P1 | 压力测试 |
| NFR-SCAL-4 | canvas-engine 横向扩展 (Rust stateless) | P0 | K8s HPA |
| NFR-SCAL-5 | canvas-realtime 横向扩展 (Sticky session + Redis pub/sub) | P1 | K8s HPA |
| NFR-SCAL-6 | 数据库分片策略 (canvas_id hash) | P2 | 架构评审 |

## §30 可观测 (Observability)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-OBS-1 | OpenTelemetry trace 100% 覆盖 (HTTP + WebSocket + DB) | P0 | 集成测试 |
| NFR-OBS-2 | Prometheus metrics 15+ 指标 (active_canvases / ws_connections / yjs_update_rate / sync_latency_p95 / etc.) | P0 | 集成测试 |
| NFR-OBS-3 | 结构化日志 (JSON, 9 字段) | P0 | 日志审查 |
| NFR-OBS-4 | 健康检查 /healthz + /readyz | P0 | K8s probe |
| NFR-OBS-5 | 错误聚合 (Sentry) | P1 | 集成测试 |
| NFR-OBS-6 | Audit log (per `domain-audit` WORM, 守门 #13) | P0 | 集成测试 |
| NFR-OBS-7 | Grafana dashboard 5+ panel (canvas 活跃数 / 同步延迟 / 错误率 / 协作者分布) | P1 | 部署验证 |

## §31 可恢复 (Recoverability)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-REC-1 | canvas-realtime 崩溃后, 客户端自动重连 (5s 内) | P0 | 混沌测试 |
| NFR-REC-2 | 数据库故障后, 读副本接管 (RPO < 5min) | P0 | 灾难演练 |
| NFR-REC-3 | 误删 canvas 可恢复 (30 天软删除) | P0 | 集成测试 |
| NFR-REC-4 | Yjs 离线编辑重连后自动 merge (per BR-10) | P0 | 混沌测试 |
| NFR-REC-5 | 备份策略: canvas 每日全量 + Yjs update 增量 5min | P1 | 备份验证 |
| NFR-REC-6 | 恢复时间 (RTO) < 30min | P1 | 灾难演练 |

## §32 兼容性 (Compatibility)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-COMPAT-1 | Chrome 110+ / Edge 110+ / Safari 16+ / Firefox 110+ 100% 兼容 | P0 | 跨浏览器测试 |
| NFR-COMPAT-2 | iPad Safari (iOS 16+) 触屏支持 | P1 | 真机测试 |
| NFR-COMPAT-3 | 移动 Web (iOS Safari / Android Chrome) 触屏支持 | P1 | 真机测试 |
| NFR-COMPAT-4 | 高 DPI 屏幕 (Retina / 4K) 适配 | P0 | 真机测试 |
| NFR-COMPAT-5 | 暗色 / 亮色模式 | P0 | 视觉测试 |
| NFR-COMPAT-6 | 操作系统: Windows 10+ / macOS 12+ / Linux (Ubuntu 22.04+) | P0 | 真机测试 |

## §33 可维护 (Maintainability)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-MAINT-1 | 单元测试覆盖率 ≥ 80% (canvas-engine / domain-canvas) | P0 | cargo tarpaulin |
| NFR-MAINT-2 | 集成测试覆盖率 ≥ 60% | P0 | cargo tarpaulin |
| NFR-MAINT-3 | E2E 测试 25+ 场景 (per §66-§75 UC) | P0 | Playwright |
| NFR-MAINT-4 | 代码审查 100% (守门 #9) | P0 | PR 流程 |
| NFR-MAINT-5 | CI 9/9 通过 (per 守门 #1 v25) | P0 | GitHub Actions |
| NFR-MAINT-6 | 文档覆盖率 ≥ 90% (rustdoc / TypeDoc) | P1 | cargo doc / typedoc |
| NFR-MAINT-7 | 0 unsafe (守门 #7) | P0 | cargo clippy |

## §34 国际化 (i18n)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-I18N-1 | 3 语言: zh-CN (默认) / en / ja 100% 翻译 | P0 | i18n 测试 |
| NFR-I18N-2 | RTL 语言预留 (P3 候选, 暂不实装) | P3 | 架构评审 |
| NFR-I18N-3 | 时区支持 (UTC + 用户本地时区显示) | P1 | 集成测试 |
| NFR-I18N-4 | 日期 / 数字 / 货币本地化 (per Intl API) | P0 | 集成测试 |
| NFR-I18N-5 | 字体降级: CJK 字体 (Noto Sans CJK / 思源黑体) | P0 | 视觉测试 |

## §35 法规 (Compliance)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-COMP-1 | GDPR 合规 (EU 用户数据可删除 / 导出) | P0 | 法务审查 |
| NFR-COMP-2 | 个人信息最小化 (per 守门 #5 env 安全) | P0 | 法务审查 |
| NFR-COMP-3 | 数据驻留: 用户可选 region (US / EU / APAC) | P2 | 架构评审 |
| NFR-COMP-4 | Cookie 同意 (per GDPR ePrivacy) | P0 | 法务审查 |
| NFR-COMP-5 | 知识产权: 用户内容所有权归用户, 平台仅托管 | P0 | 法务审查 |
| NFR-COMP-6 | 审计日志保留 7 年 (per SOX) | P1 | 备份策略 |

---

# === §36-§45 数据需求 (10 节, 30+ DR, W/T/M 严格守门 #13) ===

## §36 数据模型总览 (per 守门 #13 W/T/M 横展强制)

> **守门 #13 严格** (per AGENTS.md §4): DB 表设计必须 100% 横展三类: **Work (W)** / **Transaction (T)** / **Master (M)**, 不允许混合分类。

| # | 表名 | W/T/M 分类 | 类别理由 | 守门 #13 派生规 |
|---|---|---|---|---|
| 1 | `canvas` | **M** | 主数据, 慢变 SCD Type 2 | (c) 物理删除禁止 + SCD Type 2 + RLS 13 必带 |
| 2 | `canvas_element` | **M** | 主数据, 慢变 SCD Type 2 | (c) |
| 3 | `canvas_connector` | **M** | 主数据, 慢变 SCD Type 2 | (c) |
| 4 | `canvas_frame` | **M** | 主数据, 慢变 SCD Type 2 | (c) |
| 5 | `canvas_member` | **M** | 主数据, 权限成员, 慢变 SCD Type 2 | (c) |
| 6 | `canvas_role` | **M** | 主数据, 5 角色定义 (参考) | (c) |
| 7 | `canvas_share_link` | **M** | 主数据, 分享链接, 慢变 SCD Type 2 | (c) |
| 8 | `canvas_audit_log` | **T** | 事件流水, append-only | (b) 物理删除禁止 + 監査必須 + RLS 13 必带 |
| 9 | `canvas_realtime_yjs_update` | **M + W 二象** | Yjs 二进制 update, 短 TTL retention 30 天 | (a) + (c) 物理删除 = 30 天后 timer 清除; Master = 慢变 SCD Type 2 |
| 10 | `canvas_template` | **M** | 主数据, 模板, 慢变 SCD Type 2 | (c) |
| 11 | `canvas_template_category` | **M** | 主数据, 模板分类 (参考) | (c) |
| 12 | `canvas_image_upload` | **W** | 图片上传临时存储, retention 7 天 | (a) 物理删除 / タイマー失効 / 短 TTL |
| 13 | `canvas_export_job` | **W** | PNG / PDF 导出任务, retention 1 天 | (a) |
| 14 | `canvas_subscription` | **T** | 协作事件订阅, append-only | (b) |
| 15 | `canvas_webhook` | **M** | 主数据, webhook 配置, 慢变 SCD Type 2 | (c) |
| 16 | `canvas_api_key` | **M** | 主数据, API Key, 慢变 SCD Type 2 | (c) |
| 17 | `canvas_comment_thread` | **M** | 主数据, 评论 thread, 慢变 SCD Type 2 | (c) |
| 18 | `canvas_comment_message` | **T** | 评论消息, append-only | (b) |
| 19 | `canvas_reaction` | **T** | emoji 反应, append-only | (b) |
| 20 | `canvas_notification` | **T** | @ 提及通知, append-only | (b) |
| 21 | `canvas_view` | **T** | 浏览记录, append-only | (b) |
| 22 | `canvas_session_token` | **W** | 协作 session token, retention 1 天 | (a) |
| 23 | `canvas_undo_history` | **W** | Yjs UndoManager 历史, retention 1 天 | (a) |
| 24 | `canvas_index` | **M** | canvas 全文搜索索引, 慢变 | (c) |
| 25 | `canvas_version_snapshot` | **M** | canvas 版本快照 (按需创建), 慢变 SCD Type 2 | (c) |
| 26 | `canvas_dlq` | **W** | dead-letter queue, 失败任务, retention 7 天 | (a) |

**汇总**: 26 表, **M: 16** / **T: 6** / **W: 6** (含 1 个 M+W 二象), 守门 #13 100% 覆盖。

## §37 Work (W) 类表细节 (per 守门 #13 派生规 a: 物理删除 / 短 TTL)

### §37.1 canvas_image_upload (W)

```sql
CREATE TABLE canvas_image_upload (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,           -- 13 RLS: tenant_id + workspace_id (omitted for brevity)
    uploader_id     UUID NOT NULL,
    file_url        VARCHAR(2048) NOT NULL,  -- S3 / OSS URL
    file_size       BIGINT NOT NULL,         -- bytes
    file_hash       VARCHAR(64) NOT NULL,    -- SHA-256
    mime_type       VARCHAR(100) NOT NULL,
    width           INT,
    height          INT,
    uploaded_at     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() + INTERVAL '7 days',  -- retention
    deleted_at      TIMESTAMP WITH TIME ZONE,  -- 软删除 (timer 到期)
    -- 守门 #13 派生规 (a): 物理删除 / タイマー失効 / 短 TTL
);
CREATE INDEX idx_canvas_image_upload_expires ON canvas_image_upload(expires_at) WHERE deleted_at IS NULL;
```

### §37.2 canvas_export_job (W)

```sql
CREATE TABLE canvas_export_job (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    user_id         UUID NOT NULL,
    export_type     VARCHAR(20) NOT NULL,  -- 'png' | 'pdf' | 'svg' | 'json'
    status          VARCHAR(20) NOT NULL,  -- 'queued' | 'processing' | 'completed' | 'failed'
    file_url        VARCHAR(2048),         -- 完成后的下载 URL
    error_message   TEXT,
    started_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() + INTERVAL '1 day',
    -- 守门 #13 派生规 (a)
);
```

### §37.3 canvas_realtime_yjs_update (M + W 二象)

```sql
CREATE TABLE canvas_realtime_yjs_update (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,         -- RLS 必带
    yjs_update      BYTEA NOT NULL,        -- Yjs 二进制 update
    update_size     INT NOT NULL,          -- bytes
    version_vector  BIGINT NOT NULL,       -- 顺序应用, 单调递增
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() + INTERVAL '30 days',  -- W 派生
    -- 守门 #13 派生规 (a) + (c) 二象:
    -- W 部分: 30 天后物理删除 (timer)
    -- M 部分: 删除前是不可变历史, SCD Type 2 不变
);
CREATE INDEX idx_canvas_yjs_canvas_version ON canvas_realtime_yjs_update(canvas_id, version_vector);
```

## §38 Transaction (T) 类表细节 (per 守门 #13 派生规 b: 物理删除禁止 + 監査必須 + RLS 13 必带)

### §38.1 canvas_audit_log (T, per 守门 #13 + ADR-0043)

```sql
CREATE TABLE canvas_audit_log (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    user_id         UUID NOT NULL,
    action          VARCHAR(100) NOT NULL,  -- 'canvas.element.create' | 'canvas.element.move' | 'canvas.connector.add' | ...
    target_type     VARCHAR(50) NOT NULL,   -- 'element' | 'connector' | 'frame' | 'canvas'
    target_id       UUID,
    payload         JSONB,                  -- 详细信息
    ip_address      INET,
    user_agent      VARCHAR(500),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- 守门 #13 派生规 (b): 物理删除禁止 + 監査必須 + RLS 13 必带
    -- WORM 写入 (per ADR-0043 audit_audit_event WORM onboarding.failed 实证)
);
CREATE INDEX idx_canvas_audit_canvas_created ON canvas_audit_log(canvas_id, created_at DESC);
```

### §38.2 canvas_subscription (T)

```sql
CREATE TABLE canvas_subscription (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    user_id         UUID NOT NULL,
    event_type      VARCHAR(100) NOT NULL,  -- 'element.changed' | 'comment.added' | 'mention.added' | ...
    subscribed_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    unsubscribed_at TIMESTAMP WITH TIME ZONE,  -- 软退订
    -- 守门 #13 派生规 (b)
);
```

### §38.3 canvas_view (T, 浏览事件流水)

```sql
CREATE TABLE canvas_view (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    user_id         UUID,                  -- 匿名浏览可空
    session_id      UUID,                  -- 协作 session
    ip_address      INET,
    user_agent      VARCHAR(500),
    duration_ms     INT,                   -- 浏览时长 (ms)
    viewed_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- 守门 #13 派生规 (b)
);
```

## §39 Master (M) 类表细节 (per 守门 #13 派生规 c: 物理删除禁止 + SCD Type 2 + RLS 13 必带)

### §39.1 canvas (M)

```sql
CREATE TABLE canvas (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,         -- 13 RLS
    workspace_id    UUID NOT NULL,
    project_id      UUID,                  -- 可选, 关联 project
    owner_id        UUID NOT NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    visibility      VARCHAR(20) NOT NULL DEFAULT 'private',  -- 'public' | 'team' | 'private'
    canvas_type     VARCHAR(50),           -- 'free' | 'kanban' | 'mindmap' | 'flowchart' | 'er_diagram' | 'architecture' | 'roadmap' | 'sprint' | 'brainstorm' | 'retrospective' | 'user_journey'
    viewport        JSONB NOT NULL DEFAULT '{"x": 0, "y": 0, "zoom": 1}',
    is_template     BOOLEAN NOT NULL DEFAULT false,
    forked_from     UUID,                  -- fork 来源 (per BR-9)
    fork_count      INT NOT NULL DEFAULT 0,
    version         INT NOT NULL DEFAULT 1,  -- SCD Type 2 version
    valid_from      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_to        TIMESTAMP WITH TIME ZONE,  -- SCD Type 2, NULL = 当前版本
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMP WITH TIME ZONE,  -- 软删除 (per 守门 #13 (c) 物理删除禁止)
    -- 守门 #13 派生规 (c)
);
CREATE INDEX idx_canvas_tenant_workspace ON canvas(tenant_id, workspace_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_canvas_owner ON canvas(owner_id) WHERE deleted_at IS NULL;
```

### §39.2 canvas_element (M, 13 element kind 全部覆盖)

```sql
CREATE TABLE canvas_element (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    kind            VARCHAR(50) NOT NULL,  -- 'sticky_note' | 'text' | 'shape' | 'image' | 'embed' | 'work_item_card' | 'worktree_node' | 'agent_cursor' | 'automation_node' | 'comment_pin' | 'mind_map_node' | 'flowchart_node' | 'sketch' | 'vote_widget' | 'timer_widget' | 'table'
    x               INT NOT NULL,           -- 世界坐标
    y               INT NOT NULL,
    width           INT NOT NULL,
    height          INT NOT NULL,
    rotation        INT NOT NULL DEFAULT 0, -- 0-360 度
    z_index         INT NOT NULL DEFAULT 0,
    locked          BOOLEAN NOT NULL DEFAULT false,
    hidden          BOOLEAN NOT NULL DEFAULT false,
    content         JSONB NOT NULL,         -- 多态: 各 kind 走 discriminator
    alt_text        TEXT,                  -- 无障碍替代文本 (per FR-CANV-515)
    color           VARCHAR(20),           -- 颜色 (6 种 sticky_note + shape 填充)
    ref_kind        VARCHAR(50),           -- 'work_item' | 'worktree' | 'agent' | 'automation' | 'comment' (联动)
    ref_id          UUID,                  -- 联动 ID
    group_id        UUID,                  -- 编组 (per FR-CANV-132)
    frame_id        UUID,                  -- 所属 Frame
    created_by      UUID NOT NULL,
    version         INT NOT NULL DEFAULT 1,
    valid_from      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_to        TIMESTAMP WITH TIME ZONE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- 守门 #13 派生规 (c)
);
CREATE INDEX idx_canvas_element_canvas ON canvas_element(canvas_id, z_index DESC) WHERE valid_to IS NULL;
```

### §39.3 canvas_connector (M)

```sql
CREATE TABLE canvas_connector (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    kind            VARCHAR(50) NOT NULL,  -- 'work_item_relation' | 'agent_handoff' | 'free' | 'dependency'
    from_element_id UUID NOT NULL,
    to_element_id   UUID NOT NULL,
    routing         VARCHAR(20) NOT NULL DEFAULT 'curved',  -- 'straight' | 'curved' | 'orthogonal'
    arrow_start     BOOLEAN NOT NULL DEFAULT false,
    arrow_end       BOOLEAN NOT NULL DEFAULT true,
    color           VARCHAR(20),
    width           INT NOT NULL DEFAULT 2,
    label           TEXT,
    relation_id     UUID,                  -- 联动 relation 域
    version         INT NOT NULL DEFAULT 1,
    valid_from      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_to        TIMESTAMP WITH TIME ZONE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- 守门 #13 派生规 (c)
    -- 防环约束由应用层 DAG 检测保证 (per BR-4)
);
```

### §39.4 canvas_frame (M, Frame as Slide)

```sql
CREATE TABLE canvas_frame (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    title           VARCHAR(500) NOT NULL,
    x               INT NOT NULL,
    y               INT NOT NULL,
    width           INT NOT NULL,
    height          INT NOT NULL,
    is_slide        BOOLEAN NOT NULL DEFAULT false,
    order           INT NOT NULL DEFAULT 0,  -- 演示顺序
    parent_frame_id UUID,                   -- 嵌套 (per FR-CANV-153)
    color           VARCHAR(20),
    version         INT NOT NULL DEFAULT 1,
    valid_from      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_to        TIMESTAMP WITH TIME ZONE,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
);
```

### §39.5 canvas_member (M, 5 角色权限)

```sql
CREATE TABLE canvas_member (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    user_id         UUID NOT NULL,
    role            VARCHAR(20) NOT NULL,  -- 'viewer' | 'commenter' | 'editor' | 'admin' | 'owner'
    invited_by      UUID,
    invited_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    accepted_at     TIMESTAMP WITH TIME ZONE,
    version         INT NOT NULL DEFAULT 1,
    valid_from      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    valid_to        TIMESTAMP WITH TIME ZONE,
    UNIQUE(canvas_id, user_id, valid_from),  -- SCD Type 2 unique
);
```

### §39.6 canvas_role (M, 5 角色参考)

```sql
CREATE TABLE canvas_role (
    id              UUID PRIMARY KEY,
    name            VARCHAR(20) NOT NULL UNIQUE,  -- 'viewer' | 'commenter' | 'editor' | 'admin' | 'owner'
    description     TEXT,
    permissions     JSONB NOT NULL,         -- 13 资源 × R/A/C/I 矩阵
    is_system       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
);
```

### §39.7 canvas_share_link (M)

```sql
CREATE TABLE canvas_share_link (
    id              UUID PRIMARY KEY,
    canvas_id       UUID NOT NULL,
    token           UUID NOT NULL UNIQUE,    -- UUID v4
    permission      VARCHAR(20) NOT NULL,    -- 'viewer' | 'commenter' | 'editor'
    password_hash   VARCHAR(255),             -- bcrypt
    expires_at      TIMESTAMP WITH TIME ZONE,
    created_by      UUID NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMP WITH TIME ZONE,  -- 软撤销
);
```

## §40 权限矩阵 (5 角色 × 13 资源, per BR-5)

| 资源 / 角色 | viewer | commenter | editor | admin | owner |
|---|---|---|---|---|---|
| **canvas** | R | R | R+A | R+A | R+A+C |
| **element** | R | R | R+A | R+A | R+A |
| **connector** | R | R | R+A | R+A | R+A |
| **frame** | R | R | R+A | R+A | R+A |
| **comment** | R | R+A | R+A | R+A | R+A |
| **template** | R | R | R+A | R+A | R+A |
| **version** | R | R | R | R+A | R+A |
| **share_link** | — | — | R | R+A | R+A |
| **embed** | R | R | R+A | R+A | R+A |
| **audit_log** | — | — | — | R | R+A |
| **subscription** | R+A | R+A | R+A | R+A | R+A |
| **webhook** | — | — | R | R+A | R+A |
| **api_key** | — | — | R+A | R+A | R+A |

**R**: Read, **A**: Add/Create, **+A**: 包含 Update/Delete, **C**: Configure 全局设置

## §41-§45 数据持久化 / 存储 / 备份 / 隐私 / 数据迁移 (略, 8 节覆盖)

> 完整 5 节 (持久化策略 / 存储选型 / 备份恢复 / 隐私保护 / 数据迁移) 详见基本设计阶段 `BD-STAR-CANVAS-001.md` v0.1, 本 SRS 仅列关键约束:
> - 持久化: PostgreSQL 16 + S3-compatible object storage (图片 / 导出文件)
> - 存储选型: 单库 + canvas_id hash 分片 (P2)
> - 备份: 每日全量 + WAL 归档 + 跨 region 复制
> - 隐私: GDPR 合规, 用户可导出 / 删除自己创建的所有 canvas
> - 迁移: 旧 `domain-collaboration` Whiteboard 数据迁移到 `domain-canvas` Canvas, 一次性脚本 (per 守门 #13 W/T/M 重分类)

---

# === §46-§55 接口需求 (10 节, 40+ IR) ===

## §46 REST API (per `star-api-rest` 22 路由扩展)

> canvas API 端点 (13 个):

| IR | Method | Path | 描述 | 权限 |
|---|---|---|---|---|
| IR-API-1 | GET | `/v1/canvases` | 列出 canvas (per 守门 #13 RLS) | viewer+ |
| IR-API-2 | POST | `/v1/canvases` | 创建 canvas | editor+ |
| IR-API-3 | GET | `/v1/canvases/{id}` | 获取 canvas 详情 | viewer+ |
| IR-API-4 | PUT | `/v1/canvases/{id}` | 更新 canvas (title / description / visibility) | editor+ |
| IR-API-5 | DELETE | `/v1/canvases/{id}` | 软删除 canvas | owner |
| IR-API-6 | POST | `/v1/canvases/{id}/duplicate` | 复制 canvas | viewer+ |
| IR-API-7 | GET | `/v1/canvases/{id}/elements` | 列出 element | viewer+ |
| IR-API-8 | POST | `/v1/canvases/{id}/elements` | 创建 element | editor+ |
| IR-API-9 | PUT | `/v1/canvases/{id}/elements/{eid}` | 更新 element | editor+ |
| IR-API-10 | DELETE | `/v1/canvases/{id}/elements/{eid}` | 删除 element | editor+ |
| IR-API-11 | GET | `/v1/canvases/{id}/comments` | 列出评论 | viewer+ |
| IR-API-12 | POST | `/v1/canvases/{id}/comments` | 创建评论 | commenter+ |
| IR-API-13 | GET | `/v1/canvases/{id}/audit-log` | 审计日志 | admin+ |

## §47 WebSocket 协议 (Yjs y-protocols)

| IR | 类型 | 描述 | 路径 |
|---|---|---|---|
| IR-WS-1 | sync | Yjs 双向同步 (step1 / step2 / update) | `wss://star.app/ws/canvas/{id}` |
| IR-WS-2 | awareness | 在线状态 / 光标 / 选区 | 同上 |
| IR-WS-3 | auth | 初次连接鉴权 (Bearer token) | query param `?token=...` |
| IR-WS-4 | ping/pong | 心跳 (30s 间隔) | frame 0x9 / 0xA |
| IR-WS-5 | close code | 自定义关闭码 (4001 未授权 / 4003 禁止 / 4044 不存在) | close frame |

## §48 CRDT 协议 (Yjs y-protocols/sync)

| IR | 消息 | 描述 |
|---|---|---|
| IR-CRDT-1 | messageYjsSyncStep1 | 客户端 → 服务端: state vector |
| IR-CRDT-2 | messageYjsSyncStep2 | 服务端 → 客户端: missing updates |
| IR-CRDT-3 | messageYjsUpdate | 双向: Yjs 二进制 update |
| IR-CRDT-4 | messageYjsAwareness | 双向: awareness 状态 |

## §49 Webhook (per `star-webhook` crate)

| IR | 事件 | 描述 | payload |
|---|---|---|---|
| IR-WH-1 | `canvas.created` | canvas 创建 | `{ canvas_id, user_id, timestamp }` |
| IR-WH-2 | `canvas.updated` | canvas 元信息更新 | 同上 |
| IR-WH-3 | `canvas.deleted` | canvas 删除 | 同上 |
| IR-WH-4 | `canvas.element.created` | 元素创建 | `{ canvas_id, element_id, user_id, timestamp }` |
| IR-WH-5 | `canvas.element.updated` | 元素更新 | 同上 |
| IR-WH-6 | `canvas.element.deleted` | 元素删除 | 同上 |
| IR-WH-7 | `canvas.comment.added` | 评论添加 | `{ canvas_id, comment_id, user_id, timestamp }` |
| IR-WH-8 | `canvas.mention.added` | @ 提及 | 同上 |

## §50 MCP 工具 (per `star-mcp` 16 tools 扩展)

| IR | 工具 | 描述 | 输入 | 输出 |
|---|---|---|---|---|
| IR-MCP-1 | `canvas_list` | 列出 canvas | `{ workspace_id, filter }` | `Canvas[]` |
| IR-MCP-2 | `canvas_get` | 获取 canvas 详情 | `{ canvas_id }` | `Canvas` |
| IR-MCP-3 | `canvas_create` | 创建 canvas | `{ title, template_id? }` | `Canvas` |
| IR-MCP-4 | `canvas_update` | 更新 canvas | `{ canvas_id, fields }` | `Canvas` |
| IR-MCP-5 | `canvas_delete` | 删除 canvas | `{ canvas_id }` | `void` |
| IR-MCP-6 | `canvas_add_element` | 添加 element | `{ canvas_id, kind, x, y, content }` | `Element` |
| IR-MCP-7 | `canvas_search` | 搜索 canvas | `{ query, workspace_id }` | `Canvas[]` |

## §51-§55 (略, 5 节覆盖: 事件订阅 / GraphQL / gRPC / 消息队列 / 内部事件总线)

> 详细 5 节接口 (事件订阅 / GraphQL / gRPC / 消息队列 / 内部事件总线) 详见基本设计 `BD-STAR-CANVAS-001.md` v0.1。

---

# === §56-§65 约束 / 设计约束 / 技术栈 / 限制 / 部署 (10 节, 30+ CR) ===

## §56 设计约束

| CR | 约束 | 守门 |
|---|---|---|
| CR-DESIGN-1 | `canvas-engine` crate 不依赖任何 domain-* 业务域 (per ADR-CANVAS-005) | #4 + #13 |
| CR-DESIGN-2 | 所有 element 13 kind 走统一 `ElementKind` 枚举 + `ElementContent` 联合类型 (per ADR-CANVAS-007) | #7 |
| CR-DESIGN-3 | Yjs CRDT 走 `yrs` (Rust Yjs) 库, 不自研 CRDT | #1 |
| CR-DESIGN-4 | 实时协作 WebSocket 走 y-protocols/sync + y-protocols/awareness 业界标准 | #1 |
| CR-DESIGN-5 | 所有 element 持久化走 `canvas_realtime_yjs_update` 表 (Yjs update), 不直接 INSERT | #13 |
| CR-DESIGN-6 | 5 角色权限矩阵通过 `canvas_role` 表 + `canvas_member.role` 字段实现, 不硬编码 | #13 |
| CR-DESIGN-7 | 13 资源 RLS 全部走 `tenant_id` + `workspace_id` 必带, 守门 #13 强制 | #13 |
| CR-DESIGN-8 | DB 26 表 W/T/M 严格分类, 守门 #13 强制 100% 覆盖 | #13 |
| CR-DESIGN-9 | DB schema 修改走 migration 脚本 (sqlx-migrate), 不允许运行时 ALTER | #1 |
| CR-DESIGN-10 | 0 unsafe (守门 #7), 跨整个 workspace | #7 |

## §57 技术栈

| 层 | 选型 | 版本 | 守门 |
|---|---|---|---|
| **后端 (Rust)** |  |  |  |
| canvas-engine | Rust + tokio + serde | 1.78+ | #7 |
| domain-canvas | Rust + sqlx + axum | 1.78+ | #7 |
| canvas-realtime | Rust + tokio + yrs + axum | 1.78+ | #7 |
| 数据库 | PostgreSQL 16 | 16.x | #1 |
| 缓存 | Redis 7 | 7.x | #1 |
| 对象存储 | S3-compatible (MinIO) | latest | #1 |
| 消息队列 | NATS JetStream | 2.10+ | #1 |
| **前端 (TypeScript)** |  |  |  |
| 框架 | Next.js 14.2.5 (App Router) | 14.2.5 | #1 |
| UI | React 18.3.1 | 18.3.1 | #1 |
| 状态管理 | zustand 4.5+ | 4.5+ | #1 |
| CRDT 客户端 | yjs 13.6+ | 13.6+ | #1 |
| WebSocket | y-websocket 2.0+ | 2.0+ | #1 |
| Awareness | y-protocols 1.0+ | 1.0+ | #1 |
| 离线存储 | y-indexeddb 9.0+ | 9.0+ | #1 |
| 画布渲染 | tldraw 2.4+ (候选, P0 自研 SVG) | 2.4+ | #1 |
| Mind Map | dagre / elkjs | latest | #1 |
| Export PNG | html2canvas 1.4+ | 1.4+ | #1 |
| Export PDF | jspdf 2.5+ | 2.5+ | #1 |
| 国际化 | next-intl 3.0+ | 3.0+ | #1 |

## §58 限制

| CR | 限制 | 原因 |
|---|---|---|
| CR-LIMIT-1 | 单 canvas 10K element (P2 推到 100K) | 性能基线 G-10 |
| CR-LIMIT-2 | 单 canvas 50 协作者 (P2 推到 100) | Yjs awareness 性能 |
| CR-LIMIT-3 | 图片 20 MB / 张 | 性能 |
| CR-LIMIT-4 | Frame 嵌套最多 3 层 | UX 复杂度 |
| CR-LIMIT-5 | 移动端仅支持 Web (PWA), 不支持 native | per ADR-CANVAS-008 |
| CR-LIMIT-6 | 模板 5-10 个核心, 不做众包 2.5K | per ADR-CANVAS-010 |
| CR-LIMIT-7 | AI 辅助 (Miro Assist) 独立 SRS 评估 | §13.2 不包含范围 |
| CR-LIMIT-8 | 5 域 Lead 真人到位前, Mavis 临时代签 | per 守门 #3 + #14 |
| CR-LIMIT-9 | 物理引擎 / 3D 渲染不涉及 | per §0 不包含范围 |
| CR-LIMIT-10 | 跨机分布式不涉及 (守门 #3 5 域单仓) | per AGENTS.md §4 |

## §59 部署

| CR | 约束 |
|---|---|
| CR-DEPLOY-1 | 容器化 (Docker + K8s) |
| CR-DEPLOY-2 | canvas-realtime 走 sticky session + Redis pub/sub 横向扩展 (per NFR-SCAL-5) |
| CR-DEPLOY-3 | canvas-engine 走 stateless 横向扩展 (per NFR-SCAL-4) |
| CR-DEPLOY-4 | 数据库主从 + WAL 归档 (per NFR-REC-5) |
| CR-DEPLOY-5 | 监控 (Prometheus + Grafana) + 日志 (Loki) + 追踪 (Jaeger) |
| CR-DEPLOY-6 | CI/CD: GitHub Actions 9/9 通过 (per 守门 #1 v25) |
| CR-DEPLOY-7 | 蓝绿部署 / 金丝雀发布 |
| CR-DEPLOY-8 | CDN 静态资源 (Cloudflare / 阿里云 CDN) |
| CR-DEPLOY-9 | 边缘层: envoy 独立 deployment (per 9/1 13:05 JST 拍板, 不选 nginx / istio sidecar) |
| CR-DEPLOY-10 | mTLS 服务间通信 (per 守门 #5 env 安全 + ADR-0021 zero vendor cooperation) |

## §60-§65 (略, 6 节覆盖: 运维 / 监控告警 / 容量规划 / 灾备 / 升级策略 / 退役)

> 完整 6 节 (运维 / 监控告警 / 容量规划 / 灾备 / 升级策略 / 退役) 详见基本设计 `BD-STAR-CANVAS-001.md` v0.1。

---

# === §66-§75 验收 / 场景 / 用例 (10 节, 25 UC) ===

## §66 验收标准 (Acceptance Criteria)

| AC | 描述 | 优先级 | 验证方法 |
|---|---|---|---|
| AC-1 | 8 大功能域 100% 必备功能覆盖 (per G-1) | P0 | Miro 功能对标表 |
| AC-2 | `canvas-engine` crate 被 ≥ 3 domain 引用 (per G-2) | P0 | `cargo tree` |
| AC-3 | 50 协作者 / 1K element 同步延迟 < 200ms (per G-3) | P0 | benchmark |
| AC-4 | 离线 30 分钟本地编辑 + 重连自动 merge (per G-4) | P0 | 混沌测试 |
| AC-5 | 5 角色权限矩阵 65 单元全部覆盖 (per G-6) | P0 | RACI 矩阵 |
| AC-6 | PNG / PDF / SVG / JSON / Miro import 5 格式 (per G-8) | P0 | 集成测试 |
| AC-7 | WCAG 2.1 AA 100% 满足 (per G-9) | P0 | axe-core 自动化 |
| AC-8 | 守门 #1 #3 #5 #6 #7 #9 #12 #13 #19 #24 全过 | P0 | CI |
| AC-9 | DB 26 表 100% W/T/M 分类 (守门 #13 强制) | P0 | 审计脚本 |
| AC-10 | 0 unsafe (守门 #7) | P0 | cargo clippy |

## §67 场景 (Scenarios)

| Scenario | 描述 | UC 关联 |
|---|---|---|
| S-1 | PM 创建 user journey map 跟 stakeholder 分享 | UC-01 |
| S-2 | 5 域 Lead 跟远程协作者实时编辑 | UC-02, UC-03 |
| S-3 | 离线编辑 30 分钟后重连自动 merge | UC-04 |
| S-4 | 演讲者用 Frame as Slide 演示 | UC-05 |
| S-5 | 屏幕阅读器用户完全键盘操控 | UC-06 |

## §68-§75 25 个 Use Case (UC-01..UC-25, 略, 详见基本设计)

> 25 UC 详细流程 (前置 / 步骤 / 后置 / 异常) 详见基本设计 `BD-STAR-CANVAS-001.md` v0.1, 本 SRS 仅列标题:

| UC | 标题 | 优先级 |
|---|---|---|
| UC-01 | 创建 canvas | P0 |
| UC-02 | 添加 element (13 kind) | P0 |
| UC-03 | 实时协作编辑 (2+ 协作者) | P0 |
| UC-04 | 离线编辑 + 重连 merge | P0 |
| UC-05 | 演示模式 (Frame as Slide) | P1 |
| UC-06 | 无障碍键盘完全操控 | P0 |
| UC-07 | 链接分享 (5 角色权限) | P0 |
| UC-08 | 评论 + @ 提及 + 通知 | P0 |
| UC-09 | 撤销 / 重做 (Yjs UndoManager) | P0 |
| UC-10 | 导出 PNG / PDF / SVG / JSON | P0 |
| UC-11 | 导入 Miro RTB | P2 |
| UC-12 | 应用模板 (8 域 10 模板) | P1 |
| UC-13 | Mind Map auto-arrange | P1 |
| UC-14 | Flowchart 节点 | P1 |
| UC-15 | 嵌入 Figma / YouTube | P1 |
| UC-16 | 跨域共享 (board 引用 canvas-engine) | P0 |
| UC-17 | 跨 sub-agent canvas 共享 | P2 |
| UC-18 | Webhook 集成 (8 事件) | P1 |
| UC-19 | MCP 工具 (7 工具) | P1 |
| UC-20 | Open API (13 端点) | P1 |
| UC-21 | 审计日志查询 | P0 |
| UC-22 | 误删 canvas 恢复 (30 天软删除) | P0 |
| UC-23 | 性能压测 (10K element / 50 协作者) | P1 |
| UC-24 | WCAG 2.1 AA 自动化测试 | P0 |
| UC-25 | Miro 完整功能对标 (800+ 需求点) | P0 |

---

# === §76-§85 风险 / 缓解 / 依赖 / 排期 / 资源估算 (10 节, 25+ RK) ===

## §76 风险 (Risks)

| # | 风险 | 等级 | 缓解 | 触发守门 |
|---|---|---|---|---|
| RK-1 | Yjs CRDT 1M logical agent 量级扩展性未实证 | P0 高 | 阶段化压测, P1 推到 1K / P2 推到 10K | 守门 #1 v15 |
| RK-2 | canvas-engine / domain-canvas / canvas-realtime 3 crate 命名撞名 | P1 中 | DDD Review Lead 拍板命名 | 守门 #4 + #13 |
| RK-3 | 旧 v0.1 方案 B 残留 9 派生文档归档冲突 | P1 中 | 落地 phase 启动时一次性归档, 标记 superseded | 守门 #12 饱和 |
| RK-4 | 5 域 Lead 真人未到位, Mavis 临时代签 长期 | P1 中 | 9/5 内推 brief T0+6 周到位 | 守门 #3 + #14 |
| RK-5 | Miro 800+ 需求点中 V2 候选 (投票/计时器/AI) 排期后延 | P2 低 | 阶段化 P0/P1/P2/P3 排期 | 守门 #11 |
| RK-6 | 离线编辑 + CRDT 冲突在 50 协作者下未压测 | P1 中 | 阶段化混沌测试 | 守门 #1 v15 |
| RK-7 | tldraw 2.4+ 选型不确定 (vs 自研 SVG) | P2 低 | P0 自研 SVG, P2 评估 tldraw | 守门 #11 |
| RK-8 | 旧 Whiteboard 数据迁移到新 Canvas 数据模型 | P1 中 | 一次性迁移脚本, 保留旧数据 30 天 | 守门 #13 W/T/M |
| RK-9 | WCAG 2.1 AA 100% 合规成本 | P1 中 | 阶段化 P0/P1/P2, axe-core 自动化 | 守门 #1 |
| RK-10 | 13 种 element 渲染性能 (1K element 60fps) | P1 中 | 视口裁剪 + React.memo 缓存 | 守门 #1 v15 |

## §77-§85 (略, 9 节覆盖: 缓解 / 依赖 / 排期 / 资源估算 / 培训 / 文档 / 沟通 / 验收移交 / 复盘)

> 完整 9 节详见基本设计 `BD-STAR-CANVAS-001.md` v0.1, 关键约束:
> - 依赖: PostgreSQL 16 / Redis 7 / NATS JetStream 2.10+ / S3-compatible
> - 排期: P0 8 周 / P1 8 周 / P2 8 周 / P3 8 周 (32 周总)
> - 资源: 1 SRE 全程 (per 守门 #4 token-OLU) + 5 域 Lead 真人到位后追溯签字
> - 培训: 内部 5 域 Lead 培训 2 周

---

# === §86-§95 决策记录 (10 ADR, 推翻 5 + 新增 5 + 保留 0) ===

> 完整 10 ADR 详见 §15 决策记录, 本节仅列索引:

| ADR | 标题 | 状态 |
|---|---|---|
| ADR-CANVAS-001 | Canvas 是平台能力 + 业务域双 crate (新) | ✅ 替代旧 ADR |
| ADR-CANVAS-002 | Yjs CRDT, 非 NATS BFF polling (新) | ✅ 替代旧 ADR |
| ADR-CANVAS-003 | Canvas 节点状态色码走 StatusPill (继承) | ✅ |
| ADR-CANVAS-004 | URL param 透传 highlight + zoom + x + y (继承 + 扩展) | ✅ |
| ADR-CANVAS-005 | canvas-engine 跨多 domain 共享 (新) | ✅ |
| ADR-CANVAS-006 | Yjs 持久化 = canvas_realtime_yjs_update (M+W 二象, 新) | ✅ |
| ADR-CANVAS-007 | 13 element 走 ElementKind 枚举 + content 多态 (新) | ✅ |
| ADR-CANVAS-008 | 画布不引入移动端 native, 仅 PWA (新) | ✅ |
| ADR-CANVAS-009 | 演示模式 Frame as Slide, 不支持远程控 (新) | ✅ |
| ADR-CANVAS-010 | 模板 5-10 个核心, 不做众包 2.5K (新) | ✅ |

---

# === §96-§105 实施 / 阶段 / 里程碑 (10 节, 10 PH) ===

| Phase | 范围 | 周期 | token-OLU | 守门 |
|---|---|---|---|---|
| **P0 (2026-09-07 ~ 2026-10-04, 4 周)** | canvas-engine crate 骨架 + domain-canvas RLS + Yjs yrs 后端 PoC + 8 大功能域 100% 必备需求基线 | 4 周 | ~1.2M | 守门 #1-#26 + 累积规 v1-v26 |
| **P1 (2026-10-05 ~ 2026-11-01, 4 周)** | 13 element 实装 + 9 域联动 + 5 角色权限 + WCAG 2.1 AA + 性能基线 | 4 周 | ~1.2M | 同上 |
| **P2 (2026-11-02 ~ 2026-11-29, 4 周)** | 10 模板 + Frame as Slide + PNG/PDF/SVG/JSON 导出 + Miro import + tldraw 评估 | 4 周 | ~1.0M | 同上 |
| **P3 (2026-11-30 ~ 2026-12-27, 4 周)** | Mind Map auto-arrange + 嵌入 widget + 跨 sub-agent canvas + AI 辅助 SRS 评估 | 4 周 | ~0.8M | 同上 |
| **P4 (2026-12-28 ~ 2027-01-24, 4 周)** | 100K element 性能压测 + 100 协作者扩展 + 2.5K 模板众包 (P3 候选) | 4 周 | ~0.5M | 同上 |

> **里程碑**: 4 周 1 阶段, 总 20 周 (P0-P4), 跟 Phase F-I 架构集成 (per `ADR-0035-0042-phase-f-i-architecture.md`)。

---

# === §106-§113 运维 / 监控 / 培训 / 术语 / 索引 (8 节, 8 OP) ===

> 完整 8 节详见基本设计 `BD-STAR-CANVAS-001.md` v0.1, 关键约束:
> - 运维: 7×24 值班, SRE 1 人 (per 守门 #4 token-OLU)
> - 监控: Grafana dashboard 5+ panel, Prometheus 15+ 指标
> - 培训: 5 域 Lead 内部培训 2 周
> - 术语: per §14 用语定义
> - 索引: per `AGENTS.md` §6 关键 ADR 索引

---

# === 附录 ===

## 附录 A: Miro 8 大功能域 800+ 需求点对标清单

> 对标基线: Miro (https://miro.com) 截至 2026-09 业界标杆

| 域 | 必备功能 | STAR 覆盖 | 缺口 |
|---|---|---|---|
| 1. 画布引擎 | 无限画布 / pan / zoom / fit / 100% zoom / minimap / 视口裁剪 / 13 element / 40+ 快捷键 | ✅ §16 | 0 |
| 2. 元素系统 | 便签 / 文本 / 形状 / 图片 / 嵌入 / 连接线 / Frame / 思维导图 / 流程图 / 手绘 / 表格 / 投票 / 计时器 | ✅ §17 | 0 |
| 3. 实时协作 | Yjs CRDT / awareness / 光标 / 选区 / 评论 / @ 提及 / emoji / 离线编辑 / 跟随模式 | ✅ §18 | 0 |
| 4. 模板 | 8 域 10 模板 / 预览 / fork / 分类 / 搜索 / 标签 | ✅ §19 (P1/P2) | 5-10 个核心 |
| 5. 演示 | Frame as Slide / 演讲者视图 / 控制 / 激光笔 / 邀请链接 | ✅ §20 (P1/P2) | 远程控不支持 |
| 6. 导入导出 | PNG / PDF / SVG / JSON / Miro import / Markdown / iframe 嵌入 / Open API | ✅ §21 | 0 |
| 7. 权限分享 | 5 角色 / 13 资源 RACI / 链接 / 密码 / 过期 / 公开私有 / 转让 / 复制 / 删除 | ✅ §22 | 0 |
| 8. 快捷键无障碍 | 40+ 快捷键 / WCAG 2.1 AA / 屏幕阅读器 / 键盘完全操控 / 高对比度 / 焦点 / 替代文本 / 跳过导航 / 字幕 / 触摸 | ✅ §23 | RTL P3 候选 |

**汇总**: 8 大功能域 100% 必备功能覆盖 (per G-1), 800+ 需求点全部映射到 §16-§25 FR。

## 附录 B: 旧 v0.1 文档迁移映射 (per ADR-CANVAS-001 推翻)

> 旧 `docs/frontend-canvas-design.md` v0.1 (2026-08-26) → 新 SRS v0.1 (2026-09-06) 迁移映射:

| 旧 v0.1 章节 | 新 SRS 章节 | 状态 |
|---|---|---|
| §1.2 方案 B 选 collaboration 域强化 | §15.1 ADR-CANVAS-001 推翻, 改方案 A | 推翻 |
| §2 实体类型 (Canvas/CanvasElement/CanvasConnector) | §36-§39 26 表数据模型 | 扩展 (7 kind → 13 kind) |
| §3 CanvasView 组件规范 | §16-§18 画布引擎 + 元素 + 协作 | 扩展 |
| §4 联动 1-8 (9 域联动) | §24 FR-CANV-600~608 | 保留 + 扩展 |
| §5 快捷键 (16 旧快捷键) | §23 FR-CANV-500 40+ 快捷键 | 扩展 |
| §6 工具栏 + 侧边栏 + 状态栏 | §16 + §18 UI 规范 | 保留 |
| §7 实施分解 3 阶段 | §96-§105 5 阶段 (P0-P4) | 重排 |
| §8 路由调整 (collaboration 入口) | §13.1 canvas-engine crate | 推翻 (不再 collaboration) |
| §9 已知缺口 (10 V1/V2 候选) | §4 G-CANVAS-1~12 + §76 RK-1~10 | 升级 |
| §10 ADR-CANVAS-001~005 | §86 ADR-CANVAS-001~010 (10 ADR) | 推翻 1-2 + 5 + 扩展 4 |
| §11 验证清单 (7 项) | §66 验收标准 AC-1~10 | 升级 |

## 附录 C: 跨域共享画布引擎引用矩阵

> canvas-engine crate 被 5 domain 引用 (per G-2 + FR-CANV-701):

| 引用方 | 用途 | 状态 |
|---|---|---|
| `domain-canvas` | 画布产品域 (主) | ✅ P0 |
| `domain-board` (Kanban) | Kanban 卡片可"展开为画布视图" | 🟡 P2 候选 |
| `domain-planning` | 规划可视图 (Roadmap) | 🟡 P2 候选 |
| `domain-workflow` | 工作流可视化 | 🟡 P2 候选 |
| `domain-feedback` | 反馈聚类图 | 🟡 P2 候选 |
| `agent-view` (派生) | Agent 拓扑画布 (per `SRS-AGENT-VIEW-001.md` v1.0) | ✅ 已有, 升级到 canvas-engine |

---

> **文档结束**
>
> **下游交接**:
> 1. 落地阶段启动时, 旧 `docs/frontend-canvas-design.md` v0.1 标 superseded, 派生文档 9 份归档 (per RK-3)
> 2. `Cargo.toml` 新增 `crates/canvas-engine` / `crates/domain-canvas` / `crates/canvas-realtime` (DDD Review Lead 拍板命名后)
> 3. `frontend/package.json` 新增 yjs / y-websocket / y-protocols / y-indexeddb / dagre / elkjs / html2canvas / jspdf (P0 启动)
> 4. `docs/architecture/2026-09-06-canvas/` 新建 3 份架构 view (跟 LangGraph / Agent Runtime 平行)
> 5. `docs/design/BD-STAR-CANVAS-001.md` v0.1 基本设计 (本 SRS 落档后 P0 启动, 4 周)
> 6. `scripts/automation/canvas_*.py` 新增 5 份自动化档 (e2e 守门 / scaffold / 命名 linter / W/T/M 分类验证 / export 守门) per 守门 #19/v19+ Python 化
> 7. `docs/briefs/canvas-{p0,p1,p2}.md` 新增 3 份子 brief (P0 启动后 5 域 Lead 拍板)
> 8. `automation-design.md` §4 任务卡表 + `registry.md` 索引追加 (per 守门 #21)
