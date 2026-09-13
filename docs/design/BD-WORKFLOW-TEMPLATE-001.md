# BD-WORKFLOW-TEMPLATE-001

> **瀑布式开发工作流模板 — 基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟡 Draft v0.1 — 待 5 角色签字栏拍板
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) **v0.1** (8 子能力 WT-1~WT-8 × 33 FR × 14 US × 5 NFR × 7 张表 W/T/M 100% 覆盖 × 9 已知缺口)
> - **重要溯源说明 (per 守门 #1 禁回溯叙事)**: SRS v0.1 于 2026-09-14 在 commit `b6bb4681` 落档 (branch `agent/minimaxm3/4cf9f76a7607` 推到 origin, PR #42 OPEN)。本 BD 派生自该 SRS, 与 SRS 在同一 branch 续作 (本 BD commit 将位于 SRS commit 之上)。后续任何读者若在别处只看到 SRS 但未见本 BD, 请先确认所在 branch 是否为本任务 worktree, 避免按 SRS 直接跳到 stage 3 DD 而跳过 stage 2 BD。
> - 上位总册: [`docs/design/BD-CANVAS-001.md`](./BD-CANVAS-001.md) (root 写总册 BD, 5 view × 14 表跨域汇总) + [`docs/design/BD-CANVAS-AGENT-001.md`](./BD-CANVAS-AGENT-001.md) (专题 1: agent 管理) + [`docs/design/BD-CANVAS-GAMIFY-001.md`](./BD-CANVAS-GAMIFY-001.md) (专题 2: 游戏化) + [`docs/design/BD-CANVAS-WORKFLOW-001.md`](./BD-CANVAS-WORKFLOW-001.md) (专题 3: 自动化流程)
> - 关联 V0.1 既有代码: `frontend/src/components/UserMenu.tsx` (右上角入口扩展点, per FR-WT-8.1) + `frontend/src/lib/nav/registry.ts` (路由登记, per §2.4 消歧) + `frontend/src/components/board/KanbanBoard.tsx` (既有 4 列看板, 本 BD §3.2 SCR-WT-03 复用其 props) + `frontend/src/components/board/constants.ts` (兜底列逻辑) + `frontend/src/mocks/data/kanban.ts` (KANBAN_COLUMNS 4 列常量) + `frontend/src/types/ids.ts:322-347` (Workflow 状态机类型, 本 BD §2.4 消歧) + `frontend/src/lib/store.ts` (zustand store 扩展点)
> - 撰写者: MinimaxM3 (agent, per Multica ULYS-35 takeover, 原 assignee dc14a111 因 2 次 session limit 失败, per 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 永久代签)
> - 修订人/审批: §11 签字栏 5 角色（架构师/SRE Lead/平台工程师/评审主持/PM）由 Mavis 接手 agent 代签 (per 守门 #14 v4 反转, 真人代签流程永久 obsolete)
> - 日期: 2026-09-14
> - 受众: 詳細設計エンジニア / 実装エンジニア / UI/UX 设计师 / アーキテクト / SRE / 5 域 Lead
> - **dual-use 提醒**: 本 BD 不重复总册 BD (`BD-CANVAS-001.md`) 跨域共享部分, 聚焦 WT-1 ~ WT-8 33 项详细设计; 本 BD 亦不裁决 SRS §10 已列的 9 项已知风险与各 FR "已知缺口" — 详见 §9 TBD 追踪矩阵, 一律标注待拍板, 不自行假设

---

## §0 目的 (Purpose)

本文档基于 [`SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) v0.1 的需求, 定义 **瀑布式开发工作流模板** 的基本設計:

- **系统架构** (UI / API / 数据流 / 同步事件) 覆盖 WT-1 ~ WT-8 33 项 FR
- **组件一览 + 画面设计** (模板列表页 / 详情页 / 编辑器 / 模板化看板 / 写锁角标)
- **数据设计** (7 张表 W/T/M 100% 覆盖, SCD2/RLS/审计策略)
- **接口设计** (REST + SSE 端点, 写锁机制, 跨界面同步, 错误处理)
- **5 View 详细** (機能/データ/動作/モジュール/ネットワーク) 覆盖 33 项 FR
- **NFR** (性能/可用性/安全/可观测/可维护, 量化指标待拍板)
- **安全设计** (RBAC, 写锁 token, SCD2 审计, 跨 tenant 隔离, 输入校验)
- **守门合规 + TBD/已知缺口追踪矩阵** (9 项 SRS 已知风险 + 逐 FR 已知缺口, 全部显式标注待拍板, 不自行裁决)
- **需求→设计→测试→验收追溯矩阵** (33 项 FR 全覆盖)
- **5 角色签字栏 + 修订履历**

**关于 SRS 中【已知缺口】/【待定】/【风险 #1-#9】的处理原则 (per issue 明确指示 + 守门 #11 缺标比错标)**: 本 BD **不**对 SRS 标注为"待 Design Doc 决策"的事项自行拍板 (例如: 作废列映射 `wontfix` 是否新增 enum、模板列映射唯一性约束 vs 17 列预置冲突、软删物理恢复策略、模板跨 project 范围等)。本 BD 在相应章节给出**候选方案**并明确标注为**【TBD】**, 附影响面说明, 汇总于 §9.2; 最终选型需 5 域 Lead / Ulysses 拍板后回填。

---

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope, WT-1 ~ WT-8 8 子能力, 33 项 FR 逐条对应)

#### 1.1.1 WT-1 模板数据模型与持久化 (5 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-1.1 | Dev/PM (模板编辑者) | 已进入模板管理页面 `/workflow-templates` | 模板创建请求 (name, is_builtin=false) | 创建 1 个 `WorkflowTemplate` 记录 (含 17 列预置) | name 重复/为空 → form 校验拒绝 | `workflow_templates` 表新增 1 行, version=1 | P0 |
| FR-WT-1.2 | 系统 (模板初始化) | 已创建模板 (FR-WT-1.1 成功后) | template_id | 创建 17 个 `TemplateColumn` 记录 (按 §3.1 17 列预置数据) | — | `template_columns` 表新增 17 行 (关联 template_id) | P0 |
| FR-WT-1.3 | Dev/PM | 已创建模板 | template_id | 读取 17 列预置数据 (含 builtin/width_factor/panel/mapping_status) | template_id 不存在 → 404 | 返回完整 17 列结构 | P0 |
| FR-WT-1.4 | Dev/PM | 模板已存在, 持有写锁 | 列结构变更请求 (增/删/重排/重命名/列宽改/映射改) | 生成新 version, 历史快照追加 `template_versions` | 写锁超时/被抢 → 拒绝; 乐观锁冲突 → conflict dialog | `workflow_templates.version` +1, `template_versions` 追加 1 行 | P0 |
| FR-WT-1.5 | 系统 (跨 tenant 守卫) | 任意模板操作请求 | tenant_id + template_id | 校验 template.tenant_id == request.tenant_id | 不匹配 → 403 | 同 tenant 才能操作 | P0 |

#### 1.1.2 WT-2 17 列结构定义与不可违反规则 (6 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-2.1 | Dev/PM | 17 列预置已加载 | 删除列请求 (builtin=true) | UI 按钮置灰, **不发请求**; 后端双重校验: builtin=true → 403 | — | 17 列预置保留 | P0 |
| FR-WT-2.2 | Dev/PM | 17 列预置已加载 | 调节作废列宽 (builtin=true, width_factor=0.5) | UI 调节器置灰禁用; 后端 PATCH 时若 builtin 作废列 width_factor ≠ 0.5 → 422 | — | 作废列宽固定 0.5 | P0 |
| FR-WT-2.3 | Dev/PM | 拖动 Todo 列 | 拖动到非 position=0 位置 | UI toast 拒绝 "Todo 列必须保持在位置 0"; 后端 PATCH 时校验 | — | Todo 位置固定 0 | P0 |
| FR-WT-2.4 | Dev/PM | 拖动 P1-P9 列 | 拖动违反 P1 < P2 < ... < P9 顺序 | UI toast 拒绝; 后端 PATCH 时校验 | — | 13 列相对顺序保留 | P0 |
| FR-WT-2.5 | Dev/PM | 拖动审核/完成列 | 拖到 main panel | UI toast 拒绝; 后端 PATCH 时校验 panel 字段 | — | bottom panel 列只能在 bottom | P0 |
| FR-WT-2.6 | Dev/PM | 重命名列请求 | 新 name 为空/重名/仅空白 | form 校验失败, 不发请求; 后端双重校验 | — | name 非空且唯一 | P0 |

#### 1.1.3 WT-3 模板 CRUD UI (5 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-3.1 | Dev/PM | 已登录, tenant 上下文 | 访问 `/workflow-templates` | 渲染模板列表 (含 builtin 「瀑布式开发」) | API 失败 → 空态 + 重试 | 列表展示 | P0 |
| FR-WT-3.2 | Dev/PM | 在模板列表页 | 点击「+ 创建模板」按钮 → 弹 modal 输入 name | POST `/api/workflow-templates` 创建新模板 | 网络/权限失败 → 错误提示 | 新模板进入列表 | P0 |
| FR-WT-3.3 | Dev/PM | 已登录 | 点击列表项 | 跳 `/workflow-templates/{id}` 详情页 | template_id 不存在 → 404 | 详情页渲染 | P0 |
| FR-WT-3.4 | Dev/PM | 在详情页 | 点击「删除」按钮 (仅 is_builtin=false 显示) | DELETE `/api/workflow-templates/{id}` 软删 | builtin 模板不显示按钮 (前端隐藏 + 后端 403 双重保护) | 模板从列表消失 (deleted_at ≠ null) | P0 |
| FR-WT-3.5 | Dev/PM | 在详情页 | 点击「复制」按钮 | POST 创建新模板 (is_builtin=false, name="原名 (Copy)") | — | 新模板进入列表 | P1 |

#### 1.1.4 WT-4 模板应用到看板 (4 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-4.1 | Dev/PM | 在模板详情页 | 点击「应用到看板」按钮 | 跳 `/workflow-templates/{id}/board?project_id={current}` | — | 跳路由 + query string | P0 |
| FR-WT-4.2 | Dev/PM | 已到模板化看板路由 | 模板 id | 渲染 17 列 (从模板 columns 读取), 不渲染默认 4 列 | 模板不存在 → 404; 模板列数 ≠ 17 → 用 17 列预置回退 | 看板渲染 | P0 |
| FR-WT-4.3 | Dev/PM | 在模板化看板 | 拖 WorkItem 到模板列 | 写 `WorkItem.status = column.mapping_status`, 触发既有状态机校验 | 状态机拒绝 → toast | WorkItem.status 更新 | P0 |
| FR-WT-4.4 | Dev/PM | 在模板化看板 | 进入页面 | 右上角显示「当前模板: {name} v{version}」indicator | — | indicator 显示 | P1 |

#### 1.1.5 WT-5 模板列可编辑边界 (5 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-5.1 | Dev/PM | 在模板编辑器 | 点击「+ 增加列」 | 弹 form 强制选 panel (main / bottom), 输入 name, width_factor | panel 未选 → form 校验失败 | 新增 1 列 (builtin=false) | P0 |
| FR-WT-5.2 | Dev/PM | 在模板编辑器 | 点击「删除列」 (仅 builtin=false 显示) | 弹确认 modal → DELETE 列 | builtin 列按钮置灰 + 后端 403 | 列被删除 | P0 |
| FR-WT-5.3 | Dev/PM | 在模板编辑器 | 拖动列边缘改 width_factor (仅 builtin=false + 非作废列) | UI 实时更新列宽 | builtin 作废列调节器置灰 + 后端 422 | 列宽持久化 | P0 |
| FR-WT-5.4 | Dev/PM | 在模板编辑器 | 拖动列重排 | 实时重排 | 违反 WT-2.3/2.4/2.5 约束 → toast 拒绝并回滚 | 顺序持久化 | P0 |
| FR-WT-5.5 | Dev/PM | 在模板编辑器 | 双击列名进入编辑 | form 改 name, blur 提交 | 空/重名 → 校验失败 | name 更新 | P0 |
| (FR-WT-5.6, P1) | Dev/PM | 在模板编辑器 | 改 mapping_status 弹 picker | 4 状态单选 (todo/in_progress/review/done) | 同一状态已被其他列映射 → picker 禁用该选项 | mapping 更新 | P1 |

#### 1.1.6 WT-6 同步事件与跨界面联动 (3 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-6.1 | 系统 (BFF) | 模板 CRUD 成功 | 事件 payload `{template_id, version, event_type}` | SSE 推送 `template_changed` 事件 | SSE 不可用 → fallback polling | 客户端收到事件 | P0 |
| FR-WT-6.2 | 客户端 | SSE 不可用 | 5s polling `GET /api/workflow-templates` | 拉取最新列表, 对比本地缓存 | 网络失败 → 退避到 10s | 缓存 invalidate | P0 |
| FR-WT-6.3 | 客户端 | 收到 `template_changed` 事件 | template_id | 所有打开的 `/workflow-templates/{id}*` 路由 invalidate 缓存 + 重渲染 | — | UI 反映最新状态 | P0 |

#### 1.1.7 WT-7 多人并发编辑与排他锁 (3 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-7.1 | Dev/PM | 进入模板编辑模式 | template_id | POST `/api/workflow-templates/{id}/lock` 申请写锁 (5min 超时) | 锁被他人持有 → 返回 holder info | 锁写入 `template_locks` 表 (WORM) | P0 |
| FR-WT-7.2 | 客户端 | 锁申请结果 | 锁状态 (持有/被占) | 若被占 → 编辑按钮置灰 + 显示「{user} 正在编辑 (剩余 {N} 分钟)」 | — | UI 反映锁状态 | P0 |
| FR-WT-7.3 | Dev/PM | 提交模板更新 | template_id + version (客户端期望) + payload | 后端比对 server.version vs request.version | 不一致 → 409, 弹 conflict dialog 含 3 选项: 强制覆盖 / 放弃 / 回滚到 version N | 用户选其一, 重新提交 | P0 |

#### 1.1.8 WT-8 右侧工作流入口 (2 项 FR)

| FR ID | Actor | 前置条件 | 输入 | 输出 | 异常处理 | 后置条件 | 优先级 |
|---|---|---|---|---|---|---|---|
| FR-WT-8.1 | Dev/PM | 在任何页面 TopBar | 点击 UserMenu 头像 | 看到第 5 类入口「工作流模板」 (icon=Workflow, href=`/workflow-templates`) | — | 入口可见 | P0 |
| FR-WT-8.2 (P1) | Dev/PM | UserMenu 展开 | 看到工作流模板入口 | 右侧 badge「当前激活模板: {name}」, 点击展开下拉显示全部模板可快速切换 | 未选激活模板 → badge 显示「未激活」 | badge 显示 | P1 |

### 1.2 不包含 (Out-of-Scope, 与 SRS §1.4 完全对齐, 不重复裁决)

per SRS §1.4 列 11 项, 本 BD 不重写:

- ❌ 不修改 `Workflow` 状态机类型 / `/workflow` 单页
- ❌ 不修改现有 4 列 `KanbanBoard` / `/board` 路由 (本 BD §3.2 SCR-WT-03 复用其 props, 但走新独立路由)
- ❌ 不新增 `WorkItem.status` 枚举值 (模板列通过 §4.5 映射层映射到既有 4 状态)
- ❌ 不引入 CRDT (v1 走传统乐观锁 + write lock, per §3.2 A12 依赖声明)
- ❌ 不做模板导入/导出 (JSON/YAML) — 留 P2+
- ❌ 不做模板继承 / 模板参数化 — 留 P2+
- ❌ 不做模板自动归档 — 留 P2+
- ❌ 不引入全局权限 scheme — v1 同 tenant 即可编辑 (留 P1+ per-role)
- ❌ 不生成签字 PDF / 文档 — 审核/完成 panel 仅 UI + 字段写存储
- ❌ 不动 4 专题 SRS — 平行不重叠
- ❌ 不实现后端持久化 API — v1 前端 mock + zustand persist (留 P0 真实后端)

### 1.3 文档结构 (per issue 章节指示 + BD-CANVAS-WORKFLOW-001 v1.0.3 12 段模板)

见目录 (§0~§12)

### 1.4 派生映射

| 上位 SRS 章节 | 本 BD 派生章节 |
|---|---|
| §1 文档目的 / 适用范围 | §0 + §1 |
| §2 用语定义 (含 §2.4 三词消歧) | §2.4 (本 BD 沿用) |
| §3 业务背景 (ULYS-30 原文 + 4 差距 + 10 BR-WT + 3 A12) | §2.5 + §4.6 + §6.3 |
| §4 功能需求 33 FR (WT-1 ~ WT-8) | §1.1.1 ~ §1.1.8 (8 子能力逐条展开) |
| §5 NFR 5 项 | §7 |
| §6 约束 / 风险 | §6.3 + §8 |
| §7 验收条件 33 AC + 3 e2e | §10 追溯矩阵 (Test Case 列待回填) |
| §8 接口需求 (6 BFF + 4 组件 + 25 module 联动) | §3 + §5 |
| §9 数据需求 (7 张表 W/T/M 100%) | §4 |
| §10 已知缺口 9 项 | §9.2 TBD 追踪矩阵 (继承 + BD 新增) |
| §11 关联文档 | §附录 |
| §12 签字栏 | §11 |

---

## §2 系统架构 (System Architecture)

### 2.1 总体架构图 (5-tier, 对齐 BD-CANVAS-AGENT-001 §2.1 tier 划分)

```
┌─────────────────────────────────────────────────────────────────┐
│ Tier 1: UI (Frontend - Next.js 14 + React 18)                   │
│ ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│ │ UserMenu (右上角)│  │ /workflow-      │  │ /workflow-      │ │
│ │ 第 5 入口       │  │ templates 列表  │  │ templates/{id}  │ │
│ │ (FR-WT-8.1)    │  │ + 详情 + 编辑器 │  │ /board 模板化看板│ │
│ └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│          │                    │                    │          │
│          │ SSE + REST         │ SSE + REST         │ REST     │
└──────────┼────────────────────┼────────────────────┼──────────┘
           │                    │                    │
┌──────────▼────────────────────▼────────────────────▼──────────┐
│ Tier 2: BFF (BFF layer - Node.js 22 / Fastify)                 │
│ ┌──────────────────────────────────────────────────────────┐  │
│ │ 6 REST endpoints (per §5):                               │  │
│ │   GET    /api/workflow-templates                         │  │
│ │   GET    /api/workflow-templates/{id}                    │  │
│ │   POST   /api/workflow-templates                         │  │
│ │   PATCH  /api/workflow-templates/{id}                    │  │
│ │   DELETE /api/workflow-templates/{id}                    │  │
│ │   POST   /api/workflow-templates/{id}/lock               │  │
│ │ + SSE channel /api/workflow-templates/stream              │  │
│ └──────────────────────────────────────────────────────────┘  │
└──────────┬─────────────────────────────────────────────────────┘
           │
┌──────────▼─────────────────────────────────────────────────────┐
│ Tier 3: Domain Service (Backend - Rust / starboard crate)      │
│ ┌──────────────────────────────────────────────────────────┐  │
│ │ templateStore (新模块)                                   │  │
│ │   - list_templates(tenant_id)                            │  │
│ │   - get_template(id)                                     │  │
│ │   - create_template(input)                               │  │
│ │   - update_template(id, version, payload)  // 乐观锁    │  │
│ │   - delete_template(id)                                  │  │
│ │   - acquire_lock(id, user_id)                            │  │
│ │   - release_lock(id, user_id)                            │  │
│ │ + EventBus.publish(template_changed)                     │  │
│ └──────────────────────────────────────────────────────────┘  │
└──────────┬─────────────────────────────────────────────────────┘
           │
┌──────────▼─────────────────────────────────────────────────────┐
│ Tier 4: Database (PostgreSQL 16 + 7 tables per §4)             │
│ workflow_templates (Master) | template_columns (Master)        │
│ template_versions (Master WORM) | template_locks (Trans WORM)  │
│ template_change_events (Trans WORM)                            │
│ work_item_template_links (Master) | template_audit_logs (Trans)│
└────────────────────────────────────────────────────────────────┘
```

### 2.2 UI/API/数据流概述

**正向流** (CRUD):
UI 组件 → REST POST/PATCH/DELETE → BFF 路由 → Domain Service → Database → Domain Service 返回 → BFF 序列化 → UI 更新

**反向同步流** (跨界面):
任意 UI 操作 → Domain Service 写 DB → EventBus.publish → BFF SSE → 所有订阅的 UI 客户端 invalidate 缓存 → 重渲染

**写锁流**:
UI 进入编辑 → POST `/lock` → Domain Service 写 `template_locks` → 返回 token → UI 持锁编辑 → 提交 PATCH (含 token + version) → Domain Service 校验 token + version → 成功 / 409 conflict dialog

### 2.3 数据流场景

#### 2.3.1 场景 1: 用户在模板列表页创建模板

```
UserMenu (Tier 1)
  │ click 「+ 创建模板」按钮
  ▼
TemplateList 组件 (Tier 1)
  │ open modal, input name="我的模板"
  │ click 确认
  │ POST /api/workflow-templates
  ▼
BFF route (Tier 2)
  │ validate body
  │ call templateStore.create_template()
  ▼
Domain Service (Tier 3)
  │ insert workflow_templates (version=1)
  │ insert template_columns × 17 (按 §3.1 预置数据)
  │ insert template_versions (snapshot v1)
  │ EventBus.publish(template_changed)
  │ return new template
  ▼
BFF 序列化 JSON → 返回 201
  │
  ▼
TemplateList 收到 201
  │ update zustand store
  │ re-render list (新模板出现)
  │
  ▼
SSE 推送 template_changed 事件
  │
  ▼
其他打开的 TemplateList 客户端收到事件
  │ invalidate cache
  │ re-render list
```

#### 2.3.2 场景 2: 多人并发编辑冲突

```
用户 A 进入 /workflow-templates/123/edit
  │ POST /api/workflow-templates/123/lock
  ▼
template_locks 表新增 1 行 (holder=A, expires_at=+5min)
  │ 返回 {token: "abc123", expires_at: ...}
  │
用户 A 持锁编辑, 客户端显示 🟢 角标
  │
用户 B 进入同一模板编辑
  │ POST /api/workflow-templates/123/lock
  ▼
Domain Service 检查 template_locks
  │ 发现已被 A 持有且未过期
  │ 返回 409 {holder: A, remaining_seconds: 240}
  │
用户 B 客户端显示「A 正在编辑 (剩余 4 分钟)」, 编辑按钮置灰
  │
用户 A 编辑完成, PATCH /api/workflow-templates/123
  │ (含 token="abc123" + version=1)
  ▼
Domain Service 校验
  │ token 匹配 + version 匹配
  │ 写入新 version=2
  │ 释放 lock (DELETE template_locks)
  │
用户 A 编辑成功, 客户端显示 v2 已保存
  │
SSE 推送 template_changed 事件
  │
用户 B 客户端收到事件, invalidate cache, 解锁角标
```

### 2.4 §2.4 三词命名消歧 (per SRS §2.4 沿用)

| 用语 | 实际所指 | 来源 |
|---|---|---|
| **Workflow (Workflow 状态机)** | WorkItem 状态流转状态机, 字段 `Workflow`/`WorkflowState`/`WorkflowTransition` | `frontend/src/types/ids.ts:322-347` |
| **/workflow 单页** | Workflow Engine 入口, 用户配置状态机用 | `frontend/src/lib/nav/registry.ts:345-352` |
| **n8n 式自动化流程 (Automation Flow)** | BD-CANVAS-WORKFLOW-001 定义, 可执行节点图 | `docs/design/BD-CANVAS-WORKFLOW-001.md` |
| **WorkflowTemplate (本 BD 工作流模板)** | 用户可命名的 17 列结构, 模板维度管理 | SRS §1.3 + 本 BD §1.1 |
| **/workflow-templates 路由 (本 BD 提议)** | 模板管理入口, 跟 `/workflow` 单页**不同路由不同语义** | SRS §4.3 + 本 BD §3.2 |

### 2.5 17 列预置结构 (per SRS §4.1.3, 在 BD 阶段的落地形式)

| # | name | builtin | width_factor | panel | position | mapping_status |
|---|---|---|---|---|---|---|
| 0 | Todo | true | 1.0 | main | 0 | `todo` |
| 1 | 作废 | true | 0.5 | main | 1 | `wontfix` (【TBD】#1) |
| 2 | P1 超上流工程 | true | 1.0 | main | 2 | `in_progress` |
| 3 | P2 要件定義 | true | 1.0 | main | 3 | `in_progress` |
| 4 | P3 基本設計 | true | 1.0 | main | 4 | `in_progress` |
| 5 | P4 詳細設計 | true | 1.0 | main | 5 | `in_progress` |
| 6 | P5 実装 | true | 1.0 | main | 6 | `in_progress` |
| 7 | P6 テスト工程 | true | 1.0 | main | 7 | `in_progress` |
| 8 | P6.1 テスト計画 | true | 1.0 | main | 8 | `in_progress` |
| 9 | P6.2 テスト設計 | true | 1.0 | main | 9 | `in_progress` |
| 10 | P6.3 テスト実装 | true | 1.0 | main | 10 | `in_progress` |
| 11 | P6.4 テスト実行 | true | 1.0 | main | 11 | `in_progress` |
| 12 | P7 移行・リリース | true | 1.0 | main | 12 | `in_progress` |
| 13 | P8 運用・保守 | true | 1.0 | main | 13 | `in_progress` |
| 14 | P9 廃止 | true | 1.0 | main | 14 | `in_progress` |
| 15 | 审核 | true | 0.6 | bottom | 0 | `review` |
| 16 | 完成 | true | 0.4 | bottom | 1 | `done` |

**注**: 13 列 (P1-P9 + P6.1-P6.4) 均映射到 `in_progress` — 这与 SRS §10 #7 「同一状态不允许被多列映射」存在冲突, 需 stage 2/3 拍板 (a) 放宽约束 (允许 N 列 → 1 状态) 或 (b) 17 列拆 13 个独立 enum。本 BD 暂按 (a) 假设实现 (允许多列 → 1 状态), 待 §9.2 T-02 拍板。

---

## §3 组件一览与画面设计 (Component List & Screen Design)

> per `ipa-screen-design` skill: 以下画面均分配唯一 Screen ID, 未确认的文案/尺寸/交互细节一律标注【TBD】, 不编造。

### 3.1 组件总览 (新增组件)

| 组件 | 说明 | 对应 FR |
|---|---|---|
| `TemplateList` (新组件) | 路由 `/workflow-templates`, 列表展示模板 (含 builtin) | WT-3.1, WT-3.2 |
| `TemplateDetail` (新组件) | 路由 `/workflow-templates/{id}`, 详情 + 编辑器入口 | WT-3.3, WT-5.1~5.5 |
| `TemplateBoard` (新组件) | 路由 `/workflow-templates/{id}/board`, 模板化看板 | WT-4.1, WT-4.2, WT-4.3 |
| `TemplateColumnEditor` (新组件) | 列编辑器 (列宽/重命名/重排/增删), 嵌入 TemplateDetail | WT-5.1 ~ WT-5.6 |
| `LockIndicator` (新组件) | 写锁状态角标, 嵌入 TemplateDetail | WT-7.1, WT-7.2 |
| `ConflictDialog` (新组件) | 乐观锁冲突弹窗, 3 选项 (强制覆盖 / 放弃 / 回滚) | WT-7.3 |
| `UserMenuWorkflowEntry` (扩展既有 UserMenu) | 第 5 入口「工作流模板」+ 当前激活模板 badge | WT-8.1, WT-8.2 |
| `templateStore` (新 zustand store) | 模板 CRUD + 锁状态 + 当前激活管理 | (跨组件共享) |
| `boardStore` 扩展 (既有) | 新增 `templateColumns` prop 支持, 不破坏既有 4 列逻辑 | WT-4.2 |

### 3.2 Screen ID 一览

| Screen ID | 画面名称 | 目的 | 主要 Actor |
|---|---|---|---|
| SCR-WT-01 | 模板列表页 | 浏览 + 创建 + 删除模板 | Dev/PM |
| SCR-WT-02 | 模板详情页 | 查看 + 编辑模板列结构 | Dev/PM |
| SCR-WT-03 | 模板化看板 | 渲染模板列, WorkItem 拖入列 | Dev/PM/SRE |
| SCR-WT-04 | 锁状态指示器 | 显示当前锁持有者 + 剩余时间 | (跨页面组件) |
| SCR-WT-05 | 冲突弹窗 | 乐观锁冲突 3 选项 | Dev/PM |

#### SCR-WT-01 模板列表页

| 项目 | 内容 |
|---|---|
| 显示条件 | 用户导航到 `/workflow-templates` 或点击 UserMenu 第 5 入口 |
| 画面构成 | 顶部「+ 创建模板」按钮 + 模板卡片网格 (含 builtin 「瀑布式开发」) |
| 输入项目 | 卡片点击 → 跳 SCR-WT-02; 「+ 创建」按钮 → 弹 modal 输入 name |
| 必填/校验 | name 非空且 tenant 内唯一 |
| 按钮行为 | 「+ 创建」→ POST; 卡片点击 → 跳详情; 卡片右上角菜单 (内置模板不显示) → 删除/复制 |
| 权限控制 | 同 tenant 即可创建; 内置模板 (is_builtin=true) 仅查看, 不可删/不可改 |
| 关联 API | `GET /api/workflow-templates` (列表), `POST /api/workflow-templates` (创建), `DELETE /api/workflow-templates/{id}` (软删) |
| 异常表现 | 加载失败 → 空态 + 重试; 创建失败 → form 内联错误; 删除失败 → toast |

#### SCR-WT-02 模板详情页

| 项目 | 内容 |
|---|---|
| 显示条件 | 列表卡片点击 → 跳 `/workflow-templates/{id}` |
| 画面构成 | 顶部模板名 + 版本 indicator + 「应用到看板」按钮 + 锁状态角标 (SCR-WT-04) + 列编辑器 (SCR-WT-01 内嵌) |
| 输入项目 | 列名双击编辑 / 列宽拖动 / 列拖动重排 / 「+ 增加列」 / 列删除按钮 (仅 builtin=false) |
| 必填/校验 | 详见 FR-WT-2.6 |
| 按钮行为 | 「应用到看板」→ 跳 SCR-WT-03; 「保存」→ 触发 PATCH (含 token + version) |
| 权限控制 | 写锁持有者才能编辑; 非持有者编辑按钮置灰 (前端) + 后端 403 双重保护 |
| 关联 API | `GET /api/workflow-templates/{id}`, `PATCH /api/workflow-templates/{id}`, `POST /lock` |
| 异常表现 | 加载失败 → 错误占位; 写锁超时 → 自动释放 + 通知; 乐观锁冲突 → SCR-WT-05 弹窗 |

#### SCR-WT-03 模板化看板

| 项目 | 内容 |
|---|---|
| 显示条件 | 详情页「应用到看板」按钮点击 → 跳 `/workflow-templates/{id}/board?project_id={current}` |
| 画面构成 | 17 列横向排列 (前 15 列 main panel 占比 2/3, 后 2 列 bottom panel 占比 1/3) + 右上角「当前模板: {name} v{version}」indicator + WorkItem 卡片 (沿用既有 KanbanCard 组件) |
| 输入项目 | WorkItem 卡片拖动到不同模板列 |
| 必填/校验 | 拖到模板列时, `WorkItem.status = column.mapping_status`, 触发既有状态机校验 (WORKITEM_SM, per `frontend/src/types/ids.ts`); 校验失败 → toast 拒绝 |
| 按钮行为 | 拖 WorkItem → 触发 status 更新; 卡片点击 → 既有 WorkItemDetailDrawer |
| 权限控制 | 复用既有 WorkItem 编辑权限 |
| 关联 API | 复用既有 `/api/work-items/{id}` PATCH (status 字段更新); 不新增模板化看板专属 API |
| 异常表现 | 模板列加载失败 → 错误占位 + 「用 17 列预置回退」; WorkItem 拖动失败 → toast |

#### SCR-WT-04 锁状态指示器

| 项目 | 内容 |
|---|---|
| 显示条件 | SCR-WT-02 编辑模式开启 |
| 画面构成 | 角标: 🟢 「您正在编辑 (剩余 {N} 分钟)」 / 🔴 「{user} 正在编辑 (剩余 {N} 分钟, 您为只读)」 / ⚪ 「未锁定」 |
| 输入项目 | 无 |
| 必填/校验 | — |
| 按钮行为 | 🟢 状态 → 显示「释放锁」按钮 (手动提前释放) |
| 权限控制 | — |
| 关联 API | `POST /api/workflow-templates/{id}/lock`, 客户端轮询锁状态 (5s) |
| 异常表现 | 锁过期 → 自动切回 ⚪ 状态 + 通知 |

#### SCR-WT-05 冲突弹窗

| 项目 | 内容 |
|---|---|
| 显示条件 | PATCH 返回 409 (乐观锁冲突) |
| 画面构成 | 弹窗: 「模板在您编辑期间已被他人更新到 v{N}」+ 3 按钮 |
| 输入项目 | 3 选项: (a) 「强制覆盖」 (你的改动覆盖 server); (b) 「放弃」 (关闭弹窗, 你的改动丢失); (c) 「回滚到 v{N}」 (将 server 回滚到指定版本, 也产生新 version) |
| 必填/校验 | 选项 (c) 需要 version N (默认显示 server 当前 version) |
| 按钮行为 | (a) → 重发 PATCH 但 version=server.version (强制覆盖); (b) → 关闭弹窗; (c) → POST 回滚端点 (留 P2+) |
| 权限控制 | 写锁持有者才能操作 |
| 关联 API | (a) `PATCH /api/workflow-templates/{id}` 含 server.version; (c) 【TBD】回滚端点 |
| 异常表现 | 强制覆盖后再次冲突 → 重复弹窗 (最多 3 次, 然后强制只读) |

### 3.3 画面迁移图

```
UserMenu 第 5 入口 → SCR-WT-01 模板列表页
                        │
                        ├─→ 点击「+ 创建」 → modal → POST → 列表更新
                        ├─→ 点击卡片 → SCR-WT-02 模板详情页
                        │                  │
                        │                  ├─→ 「应用到看板」 → SCR-WT-03 模板化看板
                        │                  ├─→ 进入编辑模式 → POST /lock → SCR-WT-04 锁角标
                        │                  ├─→ 列编辑 → PATCH (含 token + version)
                        │                  │      ├─→ 成功 → 更新本地
                        │                  │      └─→ 409 → SCR-WT-05 冲突弹窗 → 3 选项
                        │                  └─→ 「释放锁」 → DELETE /lock
                        └─→ 卡片菜单「删除」 → DELETE (软删) → 列表更新
```

---

## §4 数据设计 (Data Design)

> per `ipa-database-design` skill: 以下字段均取自 SRS §9 已声明的 schema 增项, 不新增未在 SRS 中出现的业务字段; 索引/约束为**基本设计阶段建议**, 非最终定案, 详细设计阶段可调整。

### 4.1 数据模型总览 (7 张表, W/T/M 覆盖)

| # | 表名 | 分类 (W/T/M) | WORM | 行数预估 | 说明 |
|---|---|---|---|---|---|
| 1 | `workflow_templates` | Master (W) | — | 100 / tenant | 模板主表 |
| 2 | `template_columns` | Master (W) | — | 1700 / tenant (100 模板 × 17 列) | 列定义 |
| 3 | `template_versions` | Master (W) | WORM (append-only) | ~1000 / tenant (10 模板 × 100 变更) | SCD2 历史快照 |
| 4 | `template_locks` | Transaction (T) | WORM (append-only) | 10-100 / tenant (并发上限) | 写锁记录 |
| 5 | `template_change_events` | Transaction (T) | WORM (append-only) | 1000+ / tenant / day | 同步事件 |
| 6 | `work_item_template_links` | Master (W) | — | 10000+ / tenant | WorkItem 与模板列关联 |
| 7 | `template_audit_logs` | Transaction (T) | WORM (append-only) | 10000+ / tenant / day | 审计日志 |

**W/T/M 验证**: 6 Master (W) + 3 Transaction (T, 全部 WORM) — 0 混合分類, 100% 覆盖 ✅ (per 守门 #13)

**Work 类 0 张理由**: SRS 对本域 7 张表均未提出"临时性、会话级、需 TTL 自动清理"的业务需求 — 模板主表 (Master) 需长期保留并支持版本回溯, 锁/事件/审计 (Transaction) 需长期保留供审计, `work_item_template_links` (Master) 关联 WorkItem 需永久保留。无 Work 类表, 全部数据均无 TTL 自动清理策略。

### 4.2 表定义 (DDL 建议, 详细设计阶段可调整)

#### 4.2.1 `workflow_templates` (Master, WORM: —)

```sql
CREATE TABLE workflow_templates (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id     UUID NOT NULL,                         -- per §5 RLS 策略
  project_id    UUID,                                  -- NULL = 跨项目模板 (per FR-WT-1.1)
  name          VARCHAR(255) NOT NULL,
  is_builtin    BOOLEAN NOT NULL DEFAULT FALSE,       -- TRUE = 预置不可删
  version       INTEGER NOT NULL DEFAULT 1,            -- SCD2 当前 version, 递增
  columns_count INTEGER NOT NULL DEFAULT 17,           -- 冗余字段, 便于快速校验
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by    UUID NOT NULL,
  deleted_at    TIMESTAMPTZ,                           -- 软删时间戳
  CONSTRAINT uq_workflow_templates_tenant_name UNIQUE (tenant_id, name) WHERE deleted_at IS NULL
);

CREATE INDEX idx_workflow_templates_tenant ON workflow_templates(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_workflow_templates_builtin ON workflow_templates(tenant_id, is_builtin) WHERE deleted_at IS NULL;
```

**字段说明**:
- `project_id NULL`: 模板可跨项目 (per FR-WT-1.1), 但 v1 不实装跨项目模板, 字段保留供未来扩展
- `columns_count`: 冗余字段, 避免 SELECT COUNT(*) JOIN; 模板保存时同事务更新
- 唯一约束 `(tenant_id, name) WHERE deleted_at IS NULL`: 同 tenant 内模板名唯一 (软删的不参与)

#### 4.2.2 `template_columns` (Master, WORM: —)

```sql
CREATE TABLE template_columns (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
  position        INTEGER NOT NULL,                     -- 0-based, main + bottom 各自 0-based
  name            VARCHAR(255) NOT NULL,
  builtin         BOOLEAN NOT NULL DEFAULT FALSE,       -- TRUE = 17 列预置
  width_factor    NUMERIC(4,2) NOT NULL DEFAULT 1.0,    -- 0.5 / 1.0 / ...
  panel           VARCHAR(16) NOT NULL DEFAULT 'main',  -- 'main' / 'bottom'
  mapping_status  VARCHAR(32) NOT NULL,                 -- WorkItemStatus enum 值
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT chk_panel CHECK (panel IN ('main', 'bottom')),
  CONSTRAINT chk_mapping_status CHECK (mapping_status IN ('todo', 'in_progress', 'review', 'done')),
  CONSTRAINT uq_template_columns_position UNIQUE (template_id, panel, position),
  CONSTRAINT uq_template_columns_name UNIQUE (template_id, name)
);

CREATE INDEX idx_template_columns_template ON template_columns(template_id);
```

**字段说明**:
- `position`: main panel 0-14, bottom panel 0-1 (per §2.5 17 列预置)
- `mapping_status`: 必须引用 `WorkItemStatus` 枚举值; v1 含 `todo`/`in_progress`/`review`/`done` (4 值); 作废列 mapping `wontfix` 待 §9.2 T-01 拍板
- `CHECK` 约束: panel 和 mapping_status 枚举值限定 (防止前端 PATCH 绕过类型)
- `position` 唯一性: 同一模板同一 panel 内 position 唯一 (防止拖动重排产生冲突)

#### 4.2.3 `template_versions` (Master, WORM: append-only)

```sql
CREATE TABLE template_versions (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id    UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
  version        INTEGER NOT NULL,                      -- 历史 version 快照
  snapshot_json  JSONB NOT NULL,                        -- 完整模板 + columns 快照
  change_summary TEXT,                                  -- 修订摘要 (人类可读)
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by     UUID NOT NULL,
  CONSTRAINT uq_template_versions UNIQUE (template_id, version)
);

CREATE INDEX idx_template_versions_template ON template_versions(template_id, version DESC);
```

**字段说明**:
- `snapshot_json`: 完整快照含模板字段 + 全部 columns 字段 (JSONB 便于跨版本对比)
- `change_summary`: "WT-5.4 重排顺序: P3 → P2" 类可读描述, 由前端 PATCH 请求时携带
- WORM append-only: 不允许 UPDATE/DELETE (per 应用层约束 + 数据库 trigger)

#### 4.2.4 `template_locks` (Transaction, WORM: append-only)

```sql
CREATE TABLE template_locks (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
  holder_user_id  UUID NOT NULL,
  acquired_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at      TIMESTAMPTZ NOT NULL,                 -- acquired_at + 5min
  released_at     TIMESTAMPTZ,                          -- NULL = 持有中; 非 NULL = 已释放
  lock_token      VARCHAR(64) NOT NULL,                 -- 32-byte random hex
  CONSTRAINT uq_template_locks_active UNIQUE (template_id) WHERE released_at IS NULL  -- 同一 template 同时只 1 个 active lock
);

CREATE INDEX idx_template_locks_active ON template_locks(template_id, expires_at) WHERE released_at IS NULL;
```

**字段说明**:
- `released_at`: NULL 表示锁仍有效; 非 NULL 表示已释放 (正常释放或过期清理)
- 唯一约束 `(template_id) WHERE released_at IS NULL`: 同一模板同时只能有 1 个 active lock (防并发申请)
- `lock_token`: 32-byte 随机 hex, PATCH 提交时必须校验 token 匹配
- **WORM append-only**: 不允许 UPDATE (只能 INSERT + UPDATE released_at); 但 `released_at` 的 UPDATE 是允许的 (释放动作), 这跟"严格 WORM"略有差异 — 在 DD 阶段需拍板 (a) 严格 WORM (释放走新 INSERT row), 或 (b) 允许 UPDATE released_at (本 BD 默认, 实务便利)

#### 4.2.5 `template_change_events` (Transaction, WORM: append-only)

```sql
CREATE TABLE template_change_events (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id  UUID NOT NULL,
  version      INTEGER NOT NULL,                        -- 变更后 version
  event_type   VARCHAR(32) NOT NULL,                    -- 'created' / 'updated' / 'deleted' / 'rolled_back'
  payload_json JSONB,                                   -- 变更详情 (列增删/重排等)
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT chk_event_type CHECK (event_type IN ('created', 'updated', 'deleted', 'rolled_back'))
);

CREATE INDEX idx_template_change_events_template ON template_change_events(template_id, created_at DESC);
```

**字段说明**:
- `payload_json`: 含具体变更内容 (例: `{"columns_added": [...], "columns_removed": [...]}`)
- WORM: append-only, 不允许 UPDATE/DELETE (per 应用层约束)
- 用于 SSE 推送 + 离线重放 (客户端断网期间的事件回放)

#### 4.2.6 `work_item_template_links` (Master, WORM: —)

```sql
CREATE TABLE work_item_template_links (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  work_item_id UUID NOT NULL,
  template_id  UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
  column_id    UUID NOT NULL REFERENCES template_columns(id) ON DELETE RESTRICT,
  linked_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT uq_witl UNIQUE (work_item_id, template_id)
);

CREATE INDEX idx_witl_work_item ON work_item_template_links(work_item_id);
CREATE INDEX idx_witl_template ON work_item_template_links(template_id, column_id);
```

**字段说明**:
- 记录 WorkItem 当前所在模板列 (WorkItem 可同时在 1 个模板化看板, 不允许多模板关联)
- `ON DELETE RESTRICT`: WorkItem 或模板被删除时, 关联记录需先清理 (前端软删时不动 FK)
- 派生字段: WorkItem.status 应与 column.mapping_status 一致 (应用层校验, per §4.6)

#### 4.2.7 `template_audit_logs` (Transaction, WORM: append-only)

```sql
CREATE TABLE template_audit_logs (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  template_id  UUID NOT NULL,
  user_id      UUID NOT NULL,
  action       VARCHAR(32) NOT NULL,                    -- 'create' / 'update' / 'delete' / 'lock_acquire' / 'lock_release' / 'conflict_resolve'
  before_json  JSONB,                                   -- 变更前快照 (NULL for create)
  after_json   JSONB,                                   -- 变更后快照 (NULL for delete)
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT chk_action CHECK (action IN ('create', 'update', 'delete', 'lock_acquire', 'lock_release', 'conflict_resolve'))
);

CREATE INDEX idx_template_audit_logs_template ON template_audit_logs(template_id, created_at DESC);
```

**字段说明**:
- 审计所有变更, 含 5 类操作 (CRUD + 锁相关 + 冲突解决)
- `before_json` / `after_json`: 用于事后追溯 "改了什么"
- WORM: append-only

### 4.3 既有表字段扩展

无既有表需扩展字段。本 BD 不修改 `workflow_templates` 以外的任何既有表 (per SRS §1.4 Out-of-Scope)。

> 注: `WorkItem.status` 字段无需扩展, 因为模板列通过 §4.2.2 `mapping_status` 引用既有 4 状态; `WorkItem` 与模板的关联通过新建的 `work_item_template_links` (派生关系) 而非修改 `WorkItem` 自身。

### 4.4 枚举扩展

无新增枚举值。本 BD 复用既有 4 个 `WorkItemStatus` 枚举值 (`todo`/`in_progress`/`review`/`done`)。

> 注: 作废列 mapping_status 候选 `wontfix` 待 §9.2 T-01 拍板; 若拍板 (a) 新增 `wontfix` 则需扩展 `WorkItemStatus` enum。

### 4.5 SCD / RLS / 审计策略

#### 4.5.1 SCD Type 2 版本管理

- 模板结构性变更 (列增删/重排/重命名/列宽改/映射改) 触发新 `version` (per FR-WT-1.4)
- 历史快照写入 `template_versions` (append-only WORM)
- 一键回滚 (per BR-WT-9) 从 `template_versions` 读取快照, 写入新 version, 不覆盖历史

#### 4.5.2 RLS 策略 (per 守门 #13a 100% tenant_id 必携)

```sql
ALTER TABLE workflow_templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_columns ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_locks ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_change_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_item_template_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE template_audit_logs ENABLE ROW LEVEL SECURITY;

-- 7 张表统一 RLS 策略: 仅同 tenant 可读可写
CREATE POLICY tenant_isolation_policy ON workflow_templates
  USING (tenant_id = current_setting('app.current_tenant')::UUID);
-- (其余 6 张表类似, 通过 template_id 间接关联 tenant_id, 应用层 join 校验)
```

#### 4.5.3 审计策略

- 所有变更写 `template_audit_logs` (per FR-WT-1.4 / WT-5.x)
- 审计记录保留周期: 【TBD, 待 §9.2 T-08 拍板】 (v1 暂定永久保留, per 一人公司审计要求)
- 审计查询接口 (留 P1+): `GET /api/workflow-templates/{id}/audit-logs?from={date}&to={date}`

### 4.6 数据完整性约束 (与 SRS §7.3 对齐)

| 约束 | 描述 | 实现位置 |
|---|---|---|
| 17 列预置不可删 | 17 列 `builtin=true`, DELETE 请求后端 403 | DB CHECK + 应用层 |
| 作废列宽固定 0.5 | `template_columns` 上 CHECK 约束 + 应用层守卫 | DB CHECK |
| Todo + 作废顺序固定 | `position=0/1` 唯一约束 (main panel) + 应用层校验 | DB UNIQUE + 应用层 |
| P1-P9 相对顺序保留 | 应用层校验 (DB CHECK 无法表达相对顺序) | 应用层 |
| 审核/完成在 bottom panel | `panel='bottom'` CHECK + 应用层 | DB CHECK |
| 同 tenant name 唯一 | `UNIQUE (tenant_id, name) WHERE deleted_at IS NULL` | DB UNIQUE |
| 列名非空 | `name VARCHAR(255) NOT NULL` | DB NOT NULL |
| mapping_status 引用 WorkItemStatus | `CHECK (mapping_status IN (...))` | DB CHECK |
| 锁同时只能 1 个 active | `UNIQUE (template_id) WHERE released_at IS NULL` | DB UNIQUE |
| WorkItem.status 与 column.mapping_status 一致 | 应用层校验 (拖动时) | 应用层 |
| builtin 模板 `deleted_at` 永远 NULL | 应用层守卫 + DB partial UNIQUE (name) 防软删重名 | 应用层 + DB |

---

## §5 接口设计 (Interface Design)

### 5.1 REST API (6 端点, BFF)

| 方法 | 路径 | 用途 | 鉴权 |
|---|---|---|---|
| `GET` | `/api/workflow-templates` | 列表 (含 builtin) | session token |
| `GET` | `/api/workflow-templates/{id}` | 详情 | session token |
| `POST` | `/api/workflow-templates` | 创建 | session token |
| `PATCH` | `/api/workflow-templates/{id}` | 更新 (含乐观锁 version) | session token + lock token |
| `DELETE` | `/api/workflow-templates/{id}` | 软删 | session token |
| `POST` | `/api/workflow-templates/{id}/lock` | 申请写锁 (返回 lock token) | session token |
| `GET` | `/api/workflow-templates/stream` | SSE 通道 (订阅 template_changed 事件) | session token (via SSE Upgrade) |

> 注: 实际是 7 个端点 (含 SSE), 与 SRS §8.1 列 6 个对齐 (SSE 是单独的 channel, 不算 REST 端点)。详细契约见 §5.2。

### 5.2 API 详细契约 (核心端点示例, 详细设计阶段补齐余下)

#### 5.2.1 `POST /api/workflow-templates`

**请求体**:
```json
{
  "name": "我的瀑布式变体",
  "project_id": null
}
```

**响应 201**:
```json
{
  "id": "01a09cae-1234-...",
  "tenant_id": "01a09cae-5678-...",
  "name": "我的瀑布式变体",
  "is_builtin": false,
  "version": 1,
  "columns": [
    {"position": 0, "panel": "main", "name": "Todo", "builtin": true, "width_factor": 1.0, "mapping_status": "todo"},
    {"position": 1, "panel": "main", "name": "作废", "builtin": true, "width_factor": 0.5, "mapping_status": "wontfix"},
    ... (15 more)
  ],
  "created_at": "2026-09-14T07:00:00Z",
  "updated_at": "2026-09-14T07:00:00Z"
}
```

**错误码**:
- 400: name 为空 / 超长 / 非法字符
- 409: 同 tenant 内 name 重复
- 500: 数据库错误

#### 5.2.2 `PATCH /api/workflow-templates/{id}`

**请求体** (含乐观锁 version):
```json
{
  "version": 1,
  "lock_token": "abc123def456",
  "changes": {
    "columns_renamed": [{"id": "...", "new_name": "P3 设计"}],
    "columns_reordered": [{"id": "...", "new_position": 5}]
  }
}
```

**响应 200**:
```json
{
  "id": "...",
  "version": 2,
  "updated_at": "2026-09-14T07:30:00Z",
  "changes_applied": [...]
}
```

**错误码**:
- 400: changes 字段非法 (空 / 超长)
- 403: lock_token 不匹配
- 409: version 冲突 (server.version ≠ request.version), 响应含 server 最新 version 让客户端选处理方式
- 422: changes 违反 BR-WT-1~10 (尝试删除 builtin 列 / 改作废列宽等)
- 500: 数据库错误

#### 5.2.3 `POST /api/workflow-templates/{id}/lock`

**请求体**: (空)

**响应 200** (成功获取锁):
```json
{
  "lock_token": "abc123def456",
  "expires_at": "2026-09-14T07:35:00Z",
  "remaining_seconds": 300
}
```

**错误码**:
- 404: template_id 不存在
- 409: 锁被他人持有, 响应含 holder info: `{"holder_user_id": "...", "remaining_seconds": 240}`

### 5.3 实时更新: SSE 通道

**端点**: `GET /api/workflow-templates/stream` (SSE Upgrade)

**事件格式**:
```
event: template_changed
data: {"template_id": "...", "version": 2, "event_type": "updated", "actor_user_id": "..."}
```

**客户端订阅**:
- 进入 `/workflow-templates` 或 `/workflow-templates/{id}*` 路由时建立 SSE 连接
- 退出路由时关闭连接
- SSE 不可用时 fallback 到 §5.4 polling 5s

### 5.4 与既有 25 module 联动接口

| 既有 module | 联动接口 | 本 BD 改动 |
|---|---|---|
| `UserMenu` (Tier 1) | 新增第 5 入口, 加 `href="/workflow-templates"` | `frontend/src/components/UserMenu.tsx` 第 5 入口行 (per FR-WT-8.1) |
| `nav/registry.ts` (Tier 1) | 新增 `/workflow-templates` 路由登记 | `frontend/src/lib/nav/registry.ts` 新增 entry (id="workflow-templates", code="WT", category="work") |
| `KanbanBoard` (Tier 1) | 扩展支持 `columns` prop (替代硬编码 KANBAN_COLUMNS) | `frontend/src/components/board/KanbanBoard.tsx` props 新增 `columns?: TemplateColumn[]` (per FR-WT-4.2) |
| `boardStore` (Tier 2) | 新增 `templateColumns` 字段 + 支持 prop 注入 | `frontend/src/store/boardStore.ts` |
| `WorkItem.status` (Tier 4) | 模板列拖动触发 status 更新 | 既有 API `/api/work-items/{id}` PATCH (无新增) |
| `WorkItemDetailDrawer` (Tier 1) | 模板化看板内点击卡片时复用 | `frontend/src/components/board/WorkItemDetailDrawer.tsx` (无改动, 复用既有) |
| `automation` module (Tier 3) | **不联动** — 命名消歧 (per §2.4) | 无 |

> 共联动 5 个既有 module, 0 跨域 Rust crate 引用 (per 总册约束)。

### 5.5 错误处理总则

| 错误类别 | 处理策略 |
|---|---|
| 4xx 客户端错误 | form 内联错误提示 / toast, 不阻塞 UI |
| 5xx 服务端错误 | toast + Sentry 上报 + 触发 fallback polling |
| 409 乐观锁冲突 | 弹 SCR-WT-05 冲突弹窗 |
| 403 锁被他人持有 | UI 角标显示 + 编辑按钮置灰 |
| 网络中断 | 自动重连 SSE, 失败 3 次后切 polling 5s |
| SSE 连接超时 (>30s 无消息) | 客户端主动断开, 重连 |
| WorkItem.status 状态机拒绝 (拖动到非法列) | toast 显示 "WorkItem 状态机不允许此转换" |

---

## §6 5-View 详细设计 (機能・データ・動作・モジュール・ネットワーク)

### 6.1 機能 View (Functional)

| 功能块 | 对应 FR 组 | 核心机能 |
|---|---|---|
| 模板数据模型与持久化 | WT-1 (FR-WT-1.1~1.5, 5 项) | WorkflowTemplate entity + 17 列预置 + SCD2 + 租户隔离 + 软删 |
| 17 列结构约束 | WT-2 (FR-WT-2.1~2.6, 6 项) | 不可删/不可改宽/顺序锁/panel 锁/列名校验 — 7 项不可违反规则 |
| 模板 CRUD UI | WT-3 (FR-WT-3.1~3.5, 5 项) | 列表 + 创建 + 详情 + 删除 + 复制 |
| 模板应用到看板 | WT-4 (FR-WT-4.1~4.4, 4 项) | 跳路由 + 模板列渲染 + WorkItem.status 映射 + indicator |
| 模板列可编辑边界 | WT-5 (FR-WT-5.1~5.6, 5+1 项) | 增/删/改宽/重排/重命名 + mapping 改 (P1) |
| 同步事件与跨界面联动 | WT-6 (FR-WT-6.1~6.3, 3 项) | SSE + polling 5s 兜底 + 缓存 invalidate |
| 多人并发编辑与排他锁 | WT-7 (FR-WT-7.1~7.3, 3 项) | 写锁 5min + 乐观锁 + 一键回滚 |
| 右侧工作流入口 | WT-8 (FR-WT-8.1~8.2, 1+1 项) | UserMenu 第 5 入口 + 当前激活模板 badge |

33 项 FR 的逐条 Actor/输入/输出/异常映射已在 §1.1.1-§1.1.8 给出, 本节不重复罗列, 仅做功能块级归类以支撑 §10 追溯矩阵按块索引。

### 6.2 データ View (Data)

复用 §4 全部内容 (7 张表, W/T/M = 6/3/0, 见 §4.1)。跨 View 补充:

- **模板主表** (`workflow_templates` + `template_columns`) 是**静态配置数据**, 长期保留
- **版本快照** (`template_versions`) 是**append-only 历史**, 不可删除 (审计 + 回滚)
- **锁/事件/审计** (`template_locks` + `template_change_events` + `template_audit_logs`) 是**运行时数据**, append-only 但 `released_at` 字段允许 UPDATE
- **WorkItem 关联** (`work_item_template_links`) 是**派生关系**, 跟随 WorkItem 或模板的生命周期
- 删除/软删模板不级联删除 `template_locks`/`template_change_events`/`template_audit_logs`/`template_versions` (审计要求, per SRS §7.3)

### 6.3 動作 View (Behavior / State Machine)

#### 6.3.1 模板生命周期状态机 (FR-WT-1.4 + WT-5.x)

```
[Draft] --创建--> [Active v1] --列结构变更--> [Active v2] ... [Active vN]
                          │                       │
                          └---------回滚到 vi-------┘ (产生新 version vi+1, 不覆盖)
                          │
                          └---------软删---------→ [Deleted] (deleted_at ≠ null, 不再 active)
```

**状态转换约束**:
- 创建模板 → 自动进入 Active v1 (无 Draft 状态, 创建即可用)
- 列结构变更 → version +1 (per FR-WT-1.4 SCD2)
- 回滚 → 读取历史 version 快照 → 写入新 version (per BR-WT-9, 不覆盖历史)
- 软删 → deleted_at = NOW(), 不物理删除 (per FR-WT-1.5)
- builtin 模板 → 永远在 Active 状态, 不可软删 (per BR-WT-5)

#### 6.3.2 锁状态机 (FR-WT-7.1 + WT-7.2)

```
[未锁定 (⚪)] --申请写锁--> [持有中 (🟢)] --主动释放--> [未锁定 (⚪)]
                          │ --5min 超时--> [未锁定 (⚪)] (自动释放 + 通知)
                          │ --PATCH 成功--> [未锁定 (⚪)] (释放)
                          │ --PATCH 失败--> [未锁定 (⚪)]
                          │
                          └── [被他人持有 (🔴)] <--其他人申请--> [未锁定 (⚪)]
```

**关键约束**:
- 同时只能 1 个 active lock (per `template_locks` UNIQUE 约束)
- 持有者 token 必须匹配 PATCH 请求 token
- 超时自动释放 (应用层 cron 任务扫描 `expires_at < NOW() AND released_at IS NULL`)

#### 6.3.3 WorkItem 拖动状态机 (FR-WT-4.3 + 既有 WORKITEM_SM)

```
[模板列 A] --拖 WorkItem W--> [模板列 B]
                                    │
                                    ├─ WorkItem.status = B.mapping_status
                                    │
                                    └─ WORKITEM_SM 状态机校验 (既有逻辑, per `frontend/src/types/ids.ts`)
                                          │
                                          ├─ 允许 → W.status 更新
                                          └─ 拒绝 → toast "状态机不允许此转换"
```

**关键约束**:
- 模板列 mapping_status 必须引用既有 4 状态 (per §4.2.2 CHECK 约束)
- WorkItem.status 更新走既有 `/api/work-items/{id}` PATCH, 不新增模板化看板专属 API
- 13 列 (P1-P9 + P6.1-P6.4) 均映射 `in_progress` — 这意味着 P1→P2 拖动 status 不变, 仅 column_id 变 (per `work_item_template_links` 派生)

### 6.4 モジュール View (Module)

复用总册 `BD-CANVAS-001.md` §1.1.4 模块 view 索引, 本 BD 新增/扩展的模块边界:

| 模块 | 关系 |
|---|---|
| `workflow-template-engine` (新增) | 承载 WT-1~WT-8 模板 CRUD + 列编辑 + 锁管理 + 同步事件, 是本 BD 的核心新模块 |
| `board` (既有, 扩展) | `KanbanBoard` 扩展 `columns` prop 支持, 复用既有 4 列逻辑不变 |
| `work-item` (既有, 扩展) | 复用既有 status 字段, 不扩展字段, 仅扩展逻辑 (拖动到模板列时校验 mapping) |
| `user-menu` (既有, 扩展) | UserMenu 新增第 5 入口 (per FR-WT-8.1) |
| `nav-registry` (既有, 扩展) | 新增 `/workflow-templates` 路由登记 (per §5.4) |
| `template-store` (新增, zustand) | 模板 CRUD + 锁状态 + 当前激活管理 (客户端) |

### 6.5 ネットワーク View (Network / Deployment)

复用 `BD-CANVAS-AGENT-001` §2.1 既定 5-tier 部署拓扑（UI / BFF / Domain Service / DB), 本 BD 不引入新的网络拓扑层或新部署单元。

- 新增 1 个 BFF 端点组 (6 REST + 1 SSE) 在既有 BFF pod 内
- 新增 1 个 Domain Service 模块 (`workflow-template-engine`) 在既有 starboard crate 内
- 7 张新表加入既有 PostgreSQL (per §4.2 DDL)
- 前端新增 5 组件 + 2 扩展 (per §3.1)
- SSE 通道复用既有 SSE 服务 (`frontend/src/lib/realtime/sse.ts`)

---

## §7 非功能要件 (NFR)

> per `ipa-nonfunctional-requirements` skill: 以下数值为**基本设计提案值（待拍板）**, 非项目已批准基线; SRS 未量化处一律标【TBD】而非编造。验收方法逐条给出。

| NFR ID | 类别 | 要求（提案值, 待拍板） | 验收方法 |
|---|---|---|---|
| NFR-WT-01 | 性能 | 模板列表页首屏渲染 ≤ 500ms (10 模板以内, mock 数据) | Lighthouse / WebPageTest |
| NFR-WT-02 | 性能 | 模板化看板渲染 17 列 + 50 WorkItem ≤ 1s | 性能测试脚本 + APM 采样 |
| NFR-WT-03 | 性能 | 列宽拖动实时响应 ≤ 16ms (60fps) | Chrome Performance API |
| NFR-WT-04 | 性能 | PATCH 模板响应 P95 ≤ 300ms (含 version 校验 + SCD2 历史写入) | APM |
| NFR-WT-05 | 可用性 | 键盘可达 (Tab 顺序遵循视觉顺序) | a11y 测试 (axe-core) |
| NFR-WT-06 | 可用性 | 屏读器 announce 列名 + 锁状态 | NVDA / VoiceOver 测试 |
| NFR-WT-07 | 可用性 | 错误恢复: 写锁超时自动释放 + 用户通知 (toast + 邮件, 【TBD】邮件) | e2e |
| NFR-WT-08 | 安全 | 跨 tenant 访问返回 403 (mock 阶段前端校验) | 单元测试 |
| NFR-WT-09 | 安全 | 写锁持有者变更校验: 提交时验证 token = 当前持有者 | 单元测试 |
| NFR-WT-10 | 安全 | 软删模板跨 session 不可恢复 (per §10 #6 待决, 暂定禁止) | 单元测试 |
| NFR-WT-11 | 可观测 | 模板 CRUD 走既有埋点 (`frontend/src/lib/analytics.ts`) | e2e |
| NFR-WT-12 | 可观测 | 锁冲突事件埋点, 用于运营分析 | e2e |
| NFR-WT-13 | 可观测 | SSE 连接状态暴露到 `/health` | 单元测试 |
| NFR-WT-14 | 可维护 | 模板结构变更走 zustand persist 版本号, 旧版本自动迁移 | 单元测试 |
| NFR-WT-15 | 可维护 | 新增 builtin 列需更新 §2.5 表 + 跑 §10 AC 回归 | 流程性约束 |

---

## §8 安全设计 (Security Design)

> per `ipa-security-design` skill: 按实际攻击面逐项列出, 服务端权限校验与前端显示隐藏严格区分; 资料不足处标记【安全确认必要】而非假设已有防护。

### 8.1 认证与授权

- **认证 (Authentication)**: 所有 API-WT-* 复用既有 Session Token 机制, 本 BD 不新增独立认证方式。
- **授权 (Authorization)**: 复用既有 RBAC, 模板的读写权限与所属 Workspace 权限一致 (per §4.5.2 RLS `tenant_id` 策略); 前端画面按钮的显示/隐藏（如 SCR-WT-02 编辑按钮）**仅为体验优化**, 真正的写权限校验必须在 BFF 层复核, 不得仅依赖前端隐藏。

### 8.2 写锁 token 校验 (FR-WT-7.1 + WT-7.3) — 攻击面重点

- **机制**: PATCH 请求必须含 `lock_token`, 后端校验 `lock_token` 匹配 `template_locks` 表中 active lock 的 token。
- **现状**: 仅有静态 token 字符串比对, **无 token 定期轮换机制、无强制 HTTPS 校验**（v1 mock 阶段可暂缓, 真实生产前必补）。
- **【安全确认必要】** 真实生产前需补: (a) HTTPS-only 强制 (BFF 层), (b) token 长度 ≥ 32 byte (cryptographically secure), (c) token 仅在响应中返回一次 (不写日志)。本 BD 不预先假设评审结论, 上述四项在评审前均按【TBD】处理, 不建议在评审完成前接入外部生产系统。

### 8.3 乐观锁冲突防护 (FR-WT-7.3)

- 服务端强制校验: PATCH 请求的 `version` 必须等于服务端当前 `workflow_templates.version`, 否则返回 409。
- 冲突响应含 `server.version` + `server.snapshot`, 客户端可选择强制覆盖 (用 server.version 重发) / 放弃 / 回滚 (留 P2+)。
- 此校验必须在 BFF/Domain Service 层强制执行, 不得仅由前端 UI 阻止提交, 防止绕过前端直接调用 API。

### 8.4 输入校验与常见攻击面

| 攻击面 | 现状 | 备注 |
|---|---|---|
| 模板 name (`name`, WT-1) | 前端 form 校验 (非空/长度 ≤ 255) + 后端双重校验 | 防止 XSS 注入: 后端返回前转义 |
| 列名 (WT-5.5) | 前端 form 校验 + 后端双重校验 | 同上 |
| 列宽 `width_factor` (WT-5.3) | DB CHECK 约束 + 应用层守卫 | 防止注入非法数值 |
| 写锁 token (WT-7) | 32-byte 随机 hex + 仅响应中返回一次 | **【安全确认必要】** HTTPS-only + 不写日志 |
| `change_summary` (模板版本) | 长度 ≤ 500 chars + 防止 XSS | 应用层守卫 |
| SSE 通道 (WT-6.1) | 复用既有 SSE 鉴权机制, 同源策略 | 不引入新攻击面 |

### 8.5 审计与敏感数据

- `template_audit_logs` 记录所有变更 (含 lock acquire/release / conflict resolve), 永久保留 (per 一人公司审计要求)。
- `template_versions.snapshot_json` 含完整模板快照, 含 name/columns/mapping_status 等非敏感字段, 无 PII。
- Secret/密钥管理（如未来写锁 token 的存储）复用总册既有密钥管理机制, 本 BD 不新建独立密钥库。

---

## §9 守门合规 (Guard Compliance) + TBD 追踪矩阵

### 9.1 守门 #13 W/T/M 三分类声明

已在 §4.1 声明: Master 6 张 / Transaction 3 张 / **Work 0 张**。

**Work 类 0 张理由**: SRS 对本域 7 张表均未提出"临时性、会话级、需 TTL 自动清理"的业务需求 — 模板主表/版本 (Master) 需长期保留并支持版本回溯, 锁/事件/审计 (Transaction) 需长期保留供审计, `work_item_template_links` (Master) 关联 WorkItem 需永久保留。无 Work 类表, 全部数据均无 TTL 自动清理策略 (留 §9.2 T-08 拍板是否需审计周期)。

### 9.2 守门 #13a L1↔L1 通信禁止派生约束

`template-store` 模块不直接调用 L1 Agent, 复用既有 `automation` 模块的 BFF 通道。模板化看板的 WorkItem 拖动走既有 `/api/work-items/{id}` PATCH, 不新增旁路。

### 9.3 TBD 追踪矩阵 (全量汇总, 按来源分类)

> 本表汇总本 BD 全文出现的所有【TBD】/【安全确认必要】标记, 以及 SRS §10 已知风险 #1-#9 的继承状态。任何一项在本 BD 中均未被擅自假设或裁决。

| # | 来源 | 内容 | 影响 | 归属阶段 |
|---|---|---|---|---|
| T-01 | SRS §10 #5 | 作废列 mapping_status `wontfix` 是否新增 `WorkItemStatus` enum 值 | 影响 §2.5 17 列预置表 + §4.2.2 `mapping_status` CHECK 约束 + §4.4 枚举扩展声明 | DD 阶段拍板: (a) 新增 wontfix / (b) 复用 todo |
| T-02 | SRS §10 #7 | 17 列预置有 13 列映射 `in_progress`, 与 FR-WT-5.6「同状态不允许被多列映射」冲突 | 影响 §2.5 + §6.3.3 WorkItem 拖动逻辑 | DD 阶段拍板: (a) 放宽约束 (允许多列 → 1 状态) / (b) 17 列拆 13 个独立 enum |
| T-03 | SRS §10 #6 | 软删物理恢复策略 (永久禁止 vs 30 天保留后真删) | 影响 §5.5 NFR-WT-10 + §8.5 审计策略 | DD 阶段拍板 |
| T-04 | SRS §10 #8 | 模板跨 project 范围 (`project_id` 字段允许 null vs 必填) | 影响 §4.2.1 `workflow_templates.project_id` 字段 + §1.1 Out-of-Scope 决策 | DD 阶段拍板 |
| T-05 | SRS §10 #1 | polling 5s 兜底 vs 真实生产建议 SSE/WebSocket | 影响 §5.3 SSE 通道设计 + NFR-WT-04 性能指标 | 真实生产前拍板 |
| T-06 | SRS §10 #2 | 后端持久化 API 真实生产前必补 | 影响整个 v1 mock → 真实生产过渡路径 | P0 阻塞项, 真实后端实装前拍板 |
| T-07 | SRS §10 #3 | 签字 PDF/文档 (v1 不做, 留 P2+) | 影响 UI 是否需签字动作 | P2+ 拍板 |
| T-08 | 本 BD §4.5.3 | 审计记录保留周期 (v1 暂定永久, vs 留 P2+ 周期化) | 影响 §4.5.3 + NFR-WT-11~13 存储容量 | DD 阶段拍板 |
| T-09 | 本 BD §4.2.4 | `template_locks.released_at` 允许 UPDATE 是否破坏 WORM 严格性 | 影响 §4.1 WORM 声明 (a) 严格 WORM / (b) 允许 UPDATE released_at | DD 阶段拍板 |
| T-10 | 本 BD §8.2 | 写锁 token 安全强化 (HTTPS-only / 不写日志 / 长度 / 轮换) | 影响真实生产前的安全评审 | 安全评审阶段拍板 |
| T-11 | 本 BD §7 NFR | NFR-WT-01~15 量化数值为提案值, 待产品/SRE 拍板 | 影响容量规划与运维实装 | DD 阶段拍板 |
| T-12 | 本 BD §1.1.5 / §5.4 | `KanbanBoard.columns` prop 扩展是否破坏既有 4 列逻辑 | 影响既有看板回归测试 | DD 阶段回归测试覆盖 |
| T-13 | 本 BD §5.2.3 | 锁申请冲突响应格式 (holder_user_id 是否暴露给所有用户) | 影响 §8.2 RBAC 设计 | DD + 安全评审拍板 |
| T-14 | 本 BD §6.3.1 | 模板状态机是否需要 Draft 状态 (v1 默认 Active v1, 暂不实装) | 影响 §3.2 SCR-WT-02 编辑模式入口 | DD 阶段拍板 |
| T-15 | 本 BD §3.2 SCR-WT-05 | 「回滚到 version N」按钮 (c 选项) 是否本期实装 (留 P2+) | 影响 §5.2.2 PATCH 错误处理 | DD 阶段拍板 |
| T-16 | 本 BD §3.2 SCR-WT-04 | 锁状态轮询 5s (前端) vs SSE 推送 (复用 §5.3) | 影响 §5.4 联动接口 (新增 polling 还是复用 SSE) | DD 阶段拍板 |
| T-17 | 本 BD §8.4 | `template_versions.snapshot_json` 含 name 是否需加密 (防内部窥探) | 影响 §4.5.3 审计策略 + §8.5 敏感数据声明 | 安全评审阶段拍板 |
| T-18 | 本 BD §6.3.3 | WorkItem 拖动到模板列时若 mapping_status = 当前 status, 是否仍触发 PATCH (no-op) | 影响 §6.3.3 状态机逻辑 | DD 阶段拍板 |
| T-19 | 本 BD §4.6 | 17 列预置表与 FR-WT-1.3 "读取 17 列预置数据" 接口契约一致性 | 影响 §5.2 详细契约设计 | DD 阶段回填 |
| T-20 | 本 BD §6.1 | WT-5.6 (FR-WT-5.6) 改 mapping_status 是 P1, 是否本期实装 | 影响 §3.2 SCR-WT-02 列编辑器是否含 mapping picker | DD 阶段拍板 |
| T-21 | 本 BD §9.1 | 是否需要新增 Work 类表 (e.g. 临时模板草稿) | 影响 §4.1 W/T/M 分类 | DD 阶段拍板 |
| T-22 | 本 BD §3.2 SCR-WT-02 | 「应用到看板」按钮在 builtin 模板上是否可点击 (预览模式?) | 影响 §3.3 画面迁移图 | DD 阶段拍板 |

---

## §10 追溯矩阵 (Traceability Matrix)

> 覆盖全部 33 项 FR (WT-1 ~ WT-8), 而非仅 SRS 新增项。Design/Test 列为本 BD 交付时点的映射, Test Case ID 留待测试设计阶段（`ipa-test-case` skill）编写后回填。

| FR 组 | FR 数 | 对应设计章节 | 对应表/API/画面 | Test Case (待补) |
|---|---|---|---|---|
| WT-1 模板数据模型与持久化 | 5 | §4.2.1~4.2.2 + §6.1 | `workflow_templates`, `template_columns`, §5.2.1 POST | 【TBD, 测试设计阶段】 |
| WT-2 17 列结构约束 | 6 | §2.5 + §4.6 + §6.1 | `template_columns` (CHECK + UNIQUE), SCR-WT-02 校验 | 【TBD】 |
| WT-3 模板 CRUD UI | 5 | §3.2 SCR-WT-01 + §5.1 | `GET/POST/DELETE /api/workflow-templates` | 【TBD】 |
| WT-4 模板应用到看板 | 4 | §3.2 SCR-WT-03 + §5.4 | `KanbanBoard.columns` prop, `/api/work-items/{id}` PATCH | 【TBD】 |
| WT-5 模板列可编辑边界 | 5+1 | §3.2 SCR-WT-02 + §5.2.2 PATCH | `PATCH /api/workflow-templates/{id}` | 【TBD】 |
| WT-6 同步事件与跨界面联动 | 3 | §5.3 + §6.1 | SSE `/api/workflow-templates/stream` + polling fallback | 【TBD】 |
| WT-7 多人并发编辑与排他锁 | 3 | §4.2.4 + §6.3.2 + §5.2.3 | `template_locks` (WORM), `POST /lock`, SCR-WT-05 | 【TBD】 |
| WT-8 右侧工作流入口 | 1+1 | §3.1 + §5.4 | `UserMenu` 第 5 入口, `nav/registry.ts` 新增路由 | 【TBD】 |
| **合计** | **33** | — | — | — |

---

## §11 签字栏 (Signature Block)

| 角色 | 姓名/代签 | 状态 | 日期 |
|---|---|---|---|
| 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |

**本文档为 ULYS-35 issue 委托的设计文档交付物, 由 agent MinimaxM3 (接手 dc14a111 因 2 次 session limit 失败) 撰写。5 个角色（架构师/SRE Lead/平台工程师/评审主持/PM）由 Mavis 接手 agent 代签（per 守门 #14 v4 反转 + 2026-09-10 12:45 JST「真人代签流程永久 obsolete」+ 8/27 19:39 JST 用户授权代签）。§9.3 TBD 追踪矩阵中的全部 22 项为设计层面的候选方案标注, 不因签字栏完成而自动裁决, 仍需在详细设计阶段逐项拍板。**

---

## §12 修订履历 (Revision History)

| 版本 | 日期 | 变更摘要 | 作者 |
|---|---|---|---|
| v0.1 | 2026-09-14 | 初版交付, 覆盖 SRS-WORKFLOW-TEMPLATE-001 v0.1 全部 33 项 FR (WT-1~WT-8), §0-§12 完整章节结构, 7 张表 W/T/M=6/3/0, 6 REST + 1 SSE API, 22 项 TBD 追踪矩阵; 接管 dc14a111 因 2 次 session limit 失败, per 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 永久代签 + 守门 #14 v4 反转 | MinimaxM3 (agent, per Multica ULYS-35 takeover) |

---

## 附录

- 附录 A：本文档所引用的上位文档 —— [`docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) v0.1 (本 BD 派生自该 SRS, commit `b6bb4681` 落档, branch `agent/minimaxm3/4cf9f76a7607` 推到 origin, PR #42 OPEN)
- 附录 B：本文档所引用的同级文档 —— [`docs/design/BD-CANVAS-001.md`](./BD-CANVAS-001.md)（5 view 跨域汇总 / 5-tier 架构 / 25 module 矩阵均直接复用其既定设计, 本 BD 不重复定义）+ [`docs/design/BD-CANVAS-WORKFLOW-001.md`](./BD-CANVAS-WORKFLOW-001.md)（章节结构 12 段模板 / FR 表 8 列格式 / 守门合规章节结构均参照此文档）+ [`docs/design/BD-CANVAS-AGENT-001.md`](./BD-CANVAS-AGENT-001.md)（5-tier 架构引用源）
- 附录 C：本文档所引用的总册文档 —— [`docs/requirements/SRS-CANVAS-WORKFLOW-001.md`](../requirements/SRS-CANVAS-WORKFLOW-001.md) v1.1 (n8n 自动化流程, 与本 BD 通过 §2.4 命名消歧区分)
- 附录 D：与本 BD 配套的测试设计文档 —— 【TBD, 尚未创建, 建议依据 §10 追溯矩阵与 `ipa-test-case` skill 后续产出】
- 附录 E：本文档所引用的代码 —— `frontend/src/components/UserMenu.tsx` (扩展点) / `frontend/src/lib/nav/registry.ts` (路由) / `frontend/src/components/board/KanbanBoard.tsx` (扩展 props) / `frontend/src/components/board/constants.ts` (兜底列逻辑) / `frontend/src/mocks/data/kanban.ts` (KANBAN_COLUMNS 4 列常量) / `frontend/src/types/ids.ts:322-347` (Workflow 状态机类型, §2.4 消歧) / `frontend/src/lib/store.ts` (zustand store 扩展点) / `frontend/src/lib/realtime/sse.ts` (SSE 通道复用)
