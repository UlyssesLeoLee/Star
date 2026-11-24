# DEFECT-WBS — 缺陷管理模块交付工作分解结构

版本：v1.2 · 2026-11-24 · 基于 dev@b1db1c99 · 设计文档 SRS v1.4 / BD v1.6 / DD v1.5  
适用：`crates/domain-defect`、前端 `/defect` 页面、`star-api-rest` 路由、SRS/BD/DD 设计文档（IPA 模板对齐版）  
发布门禁：稳定版在所需门禁通过前维持 NO-GO（per 用户发布门禁）。  
状态判定口径：基本设计 §3.5.2 四项判定（调用可达 / 物理持久化 / 授权真实 / 数据同源）。

---

## 总览

| 阶段 | 目标 | 状态 | 验收标准 |
|------|------|------|----------|
| Phase A | PostgreSQL 持久化（缺陷/验证/建议表 + 迁移） | **部分完成** | `cargo check` 通过 ✓；PG 下跨 tenant/Project 拒绝、状态机、验证闭环测试通过 ✓（领域层 14/14）；**物理迁移文件 + RLS 缺失** ✗（G-11） |
| Phase B | 审计 + Outbox 接线（fail-closed 去掉 Noop） | **部分完成** | owner 事务内处理版本/幂等回执/审计/Outbox ✓（代码级）；重复命令/回执丢失/并发修改/过期授权/跨租户拒绝测试通过（仅内存层 ✓）；**Outbox envelope 物理列缺失** ✗（G-12）、**幂等回执不保真** ✗（G-11） |
| Phase C | `defects.rs` 路由接入 `star-api-rest` | **部分完成** | create/get/list/transition/submit_verification 路由 ✓（6 端点已注册）；**PG 未接线** ✗（G-16）、**会话身份为桩** ✗（G-15）、**HTTP 状态码未区分** ✗（G-18）；API 契约测试 4/4 ✗ |
| Phase D | 前端缺陷页面完善（P0-3） | **部分完成** | 新建按钮打开 Bug 类型窗口 ✗（G-13：实际 kind="task"）；多 Worktree 部分失败显示失败范围 ✗（G-17）；未接入字段显示"未知/未接入" ✗（G-14：仍由 `done` 反推"已验证"）；401/403/草稿恢复场景 ✗ |
| Phase E | 剩余模块整合（P1：Lane/BI/CLI/Agent/通知/插件深链） | 未开始 | Lane Backlog/Sprint/看板收敛；任务详情接通 CLI/Agent/取消/恢复/独立验证；统一统计口径；AI 更新与授权通知事件；Lane 插件页深链修复 |
| Phase F | 测试与交付 | 部分完成 | domain-defect 14/14 测试通过 ✓；vitest 1535/1535 ✓；重跑看板/Lane/任务详情/缺陷/设置深链测试 ✗（仅全量回归）；E2E 验收记录 ✗；SRS/BD/DD/WBS 逐项标记 ✓（本轮） |

---

## Phase A — PostgreSQL 持久化（用户已绑定 Neon，立即开始）

### A.1 依赖与表设计
- [x] A.1.1 `crates/domain-defect/Cargo.toml` 添加 `sqlx`（workspace 版本，features: postgres/uuid/chrono/json/macros/runtime-tokio-rustls）
- [x] A.1.2 新增 `crates/domain-defect/src/pg_schema.rs`：定义 `defect.work_items`、`defect.verifications`、`defect.suggestions`、`defect.command_idempotency`、`defect.audit_events`、`defect.outbox` 表 DDL（按 `multica.*` 命名空间惯例，使用 `defect.*`）
- [ ] A.1.3 **新增迁移脚本**（Neon 兼容）——**未完成**：`db/migrations/` 下无 defect 迁移文件。目标 DDL 见 DD §4.2（含六表、唯一约束、索引、逐表 RLS、审计不可变触发器、Outbox envelope 列）。缺口 **G-11 / G-12**

### A.2 PG 服务实现（参考 `crates/infrastructure/src/run_task_card_pg.rs` 模式）
- [ ] A.2.1 新增 `PgDefectService` 实现 `DefectCommandPort` / `DefectQueryPort`（在 owner 事务内：`BEGIN` → 设 tenant/actor GUC → 校验授权 → 幂等锁 → 写 defects/verifications/audit/outbox → `COMMIT`）
- [ ] A.2.2 `create_defect`：复用 `WorkItemId` 作为 `defect_id`；按 actor tenant/Project/Lane/Worktree 归属与能力校验；拒绝可直接指定其他 tenant/project 的创建路径
- [ ] A.2.3 `transition_status`：受控状态流转表；禁止任意状态直接跳 Closed；版本乐观锁（`expected_version`）；返回具体拒绝原因
- [ ] A.2.4 `submit_verification`：保存验证主体/修复版本/方法/结果/证据；按策略检查验证独立性（验证人 != 报告人，非管理员）
- [ ] A.2.5 审计写入 + Outbox 投递在同一 owner 事务内；禁止跨 owner 直接写表

### A.3 集成与降级
- [ ] A.3.1 `InMemoryDefectService` 保留为开发/降级实现，生产路由走 `PgDefectService` —— **部分**：`InMemoryDefectService` 已保留，但 `DEFECT_SERVICE` 恒 `DefectService::new(None)` → 生产路由**从未**走 `PgDefectService`。缺口 **G-16**
- [x] A.3.2 `cargo check -p domain-defect` 通过（2 个 dead_code warning，非新增错误）
- [ ] A.3.3 单元/集成测试在 PG 下通过 —— **部分**：领域层 14/14 通过（跨 tenant/Project 拒绝、任意状态跳转拒绝、版本冲突、幂等重试、验证独立性）；**PG 下未跑**（无迁移文件、无 Neon 连接）。缺口 **G-11 / G-16**

---

## Phase B — 审计与 Outbox 接线

### B.1 真实端口实现
- [ ] B.1.1 `AuditPort.record`：写入 `defect_audit_events` 表（event_type、actor_user_id、tenant_id、project_id、target_id、payload、created_at）
- [ ] B.1.2 `OutboxPort.emit`：写入 `defect_outbox` 表（aggregate_type、aggregate_id、event_type、payload，带状态字段供后续投递）
- [ ] B.1.2 去掉 `NoopAuditPort`/`NoopOutboxPort` 的 fail-closed 占位，改为真实 PG 实现

### B.2 事务内幂等与回执
- [x] B.2.1 重复命令：同一 idempotency key 返回原记录，不新增（`pg_service.rs:657` advisory lock + `:665` 幂等查询）
- [ ] B.2.2 回执丢失：幂等表查询后返回缓存的 write receipt（`replayed: true`）—— **部分**：去重成立，但命中时返回 `build_defect_stub`（空标题/空现象/nil reporter），**未回放 `response_body`**；`request_hash` 绑的是 key 本身而非请求摘要，从不比对。缺口 **G-11**
- [x] B.2.3 并发修改：版本乐观锁，`expected_version` 不匹配返回 `Conflict`（`pg_service.rs:287`）
- [x] B.2.4 过期授权：actor tenant/Project 归属校验失败返回 `CrossTenantDenied`/`CrossProjectDenied`（`pg_service.rs:282`、`:285`）
- [x] B.2.5 跨租户/跨 Project 拒绝测试通过（领域层测试覆盖；PG 层未跑）

---

## Phase C — `defects.rs` 路由接入 `star-api-rest`

### C.1 路由与授权
- [x] C.1.1 新增 `crates/star-api-rest/src/routes/defects.rs`：create / get / list / transition / submit_verification + list_verifications（共 6 端点，已注册于 `src/lib.rs:215-220`）
- [ ] C.1.2 复用授权中间件与会话身份提取 —— **部分**：`auth_layer_stub` 为桩（`src/middleware/auth.rs:23`）；`actor_from_headers` 随机生成 user/tenant UUID，与请求 tenant 永不相等 → 跨 tenant 拒绝路径不可达。缺口 **G-15**
- [ ] C.1.3 空 sessionId 路径依赖缺失时保持禁用并返回具体原因 —— **未完成**：缺陷路由暂无 sessionId 相关端点；`DefectService` 的 PG 降级无显式原因标注。缺口 **G-16**

### C.2 契约验证
- [ ] C.2.1 后端 API 契约测试 4/4 通过 —— **未完成**：`cargo test -p star-api-rest` 现有集成测试因缺 `star_api_rest` crate 链接而无法编译（`E0463 can't find crate for star_api_rest`）。缺口 **G-15 / G-16 / G-18**

---

## Phase D — 前端缺陷页面完善（P0-3）

### D.1 新建与详情
- [ ] D.1.1 缺陷新建按钮实际打开 Bug 类型创建窗口 —— **未完成**：`WorkItemDetailDrawer` 声明 `defaultKind`/`lockKind` prop 但函数签名未解构；缺陷页 `handleCreate` 只传 `{kind:"new",defaultStatus:"todo"}` → draft kind 落到 `"task"`；Kind 下拉未按 `lockKind` 禁用。目标方案见 DD §3.2.3。缺口 **G-13**
- [ ] D.1.2 创建成功后使用 owner 回执 + 授权投影刷新列表 —— **未完成**：三处新建均走本地 `store.addWorkItem`（`localOnly=true`），无 owner 回执、无 revision 驱动刷新。缺口 **G-13**
- [ ] D.1.3 缺陷详情接入统一 `WorkItemDetailDrawer`；缺陷扩展字段接入真实来源 —— **部分**：抽屉已接入 ✓；扩展字段仍为占位值（`severity` 恒 `"unknown"`、`actual_result`/`expected_result`/`fix_version` 恒 `undefined`），且 `GroupWorkItemView` 不含缺陷扩展字段 → 需新增缺陷专属读模型。缺口 **G-14**

### D.2 上下文与多 Worktree
- [x] D.2.1 保留 Project/Lane/Worktree/Branch 上下文；四级树选择联动过滤（`defect/page.tsx` 读 `navStore.selectedLaneId`/`selectedWorktreeId`/`selectedProjectId`）
- [ ] D.2.2 多 Worktree 部分失败时显示失败范围和原因 —— **未完成**：`defect/page.tsx:138` 单 worktree 失败被 `catch {}` 吞掉。目标契约见 DD §3.2.4。缺口 **G-17**

### D.3 权限与异常场景
- [ ] D.3.1 覆盖：无 Worktree 的 Lane、两个 Project、同名 Branch、不同权限用户、切换上下文、迟到响应、401/403、草稿恢复 —— **未完成**：无对应测试
- [ ] D.3.2 未授权/未接入时创建/移动按钮禁用并说明原因 —— **未完成**：当前无诚实三态实现；验证状态仍由 `status==="done"` 反推“已验证”。缺口 **G-14**

---

## Phase E — 剩余模块整合（P1）

### E.1 Lane/看板收敛
- [ ] E.1.1 Lane Backlog、Sprint、独立看板入口收敛，保留原有深链、看板列、拖动、Sprint 与任务详情能力
- [ ] E.1.2 统一授权范围和写入路径（与缺陷模块共享同一 WorkItem ID/状态规则）

### E.2 任务详情
- [ ] E.2.1 任务详情接通已授权的 CLI、Agent、取消/恢复、独立验证
- [ ] E.2.2 空 sessionId 路径核对，依赖缺失时保持禁用并说明原因

### E.3 统一统计口径
- [ ] E.3.1 任务列表、日历、时间线、燃尽图、BI 统一范围与统计口径
- [ ] E.3.2 用不同 Project/Lane/Worktree 和未绑定任务核对列表、计数与钻取

### E.4 AI 更新与授权通知
- [ ] E.4.1 接通 AI 更新与授权通知事件；验证断线补缺、重复投递、慢消费者、权限撤销
- [ ] E.4.2 核对 Team 的 Runtime/Daemon 和 Worker/Router 管理操作是否真实接入

### E.5 Lane 插件页深链
- [ ] E.5.1 修复 Lane 插件页 `/settings?tab=integrations` 落到 MCP 页的问题
- [ ] E.5.2 Hooks 保持"高级设置"；验证插件及策略深链

---

## Phase F — 测试与交付

### F.1 domain-defect 测试（已完成 14/14，持续维护）
- [x] F.1.1 跨 tenant/Project 拒绝
- [x] F.1.2 任意状态跳转拒绝（返回具体原因）
- [x] F.1.3 首次提交修复版本（fix_version 字段）
- [x] F.1.4 独立验证（验证人 != 报告人，fix_version 匹配）—— 注意：比较对象是**报告人**，非实际修复者，见缺口 G-20
- [x] F.1.5 版本冲突（expected_version 不匹配）
- [x] F.1.6 幂等重试（同一 key 返回原记录）

### F.2 前端与 Rust 检查
- [x] F.2.1 重跑 vitest 全量 —— 170 文件 / 1535 用例全通过（`dev@b1db1c99`）；**定向重跑**（看板/Lane 导航/任务详情/缺陷页面/设置深链）未单独执行
- [ ] F.2.2 使用真实 NEMA WASM 路径单独验证 —— 未执行
- [ ] F.2.3 运行相关 Rust 定向测试、`cargo fmt`、`cargo clippy` 静态检查 —— `cargo check -p domain-defect` / `-p star-api-rest` 通过 ✓；`cargo fmt`/`clippy` 未执行；`cargo test -p star-api-rest` 因集成测试缺 crate 链接而失败

### F.3 浏览器端到端验收
- [ ] F.3.1 真实授权与持久化环境完成一条：缺陷创建 → 修复 → 独立验证 → 关闭
- [ ] F.3.2 记录提交、环境、命令和证据

### F.4 设计文档逐项标记
- [x] F.4.1 SRS-DEFECT-INTELLIGENCE-001 / BD / DD / WBS：每项标明 已完成 / 部分完成 / 未运行 / 阻塞
- [x] F.4.2 最后报告每项修改、测试结果与仍未接入的能力

### F.5 缺口总表（2026-11-24 物理事实复核 + IPA 自审，与 BD §6 / DD §7 一致，共 26 项）

| # | 缺口 | 阻塞的阶段 | 关闭条件摘要 |
|---|------|-----------|------------|
| G-11 | `defect.*` 六表物理迁移文件 + RLS + 审计不可变触发器；幂等回执保真 | A / B | 新增迁移文件（含六表 DDL、逐表 ENABLE/FORCE RLS、tenant 策略、索引、审计禁改触发器）；幂等命中回放 `response_body`；`request_hash` 改为真实请求摘要并比对 |
| G-12 | `defect.outbox` 增补 envelope 物理列 | B | DDL 增补 `event_id`/`stream_id`/`source_sequence`/`correlation_id`/`event_digest`/`schema_version`/`occurred_at` + `(stream_id, source_sequence)` 唯一约束 + digest 长度约束 |
| G-13 | 缺陷新建统一走 owner 写入路径 + kind 锁定 | D | 三处新建统一调 owner 创建命令并以回执/revision 刷新；抽屉真正解构 `defaultKind`/`lockKind`，Kind 下拉锁定时禁用 |
| G-14 | 缺陷扩展字段与验证状态接真实来源 | D | 新增缺陷专属读模型（含 `latest_verification`）；三态渲染（值/未填写/未接入）；删除 `done` → “已验证” 的反推 |
| G-15 | REST 路由使用真实会话身份 | C | 从已验证会话取 `ActorContext`；租户/Project/能力校验基于真实身份；401/403 可区分 |
| G-16 | `DefectService` 注入真实 `PgPool` | A / C | 启动流程注入 Neon pool；保留显式降级开关与降级原因标注 |
| G-17 | 多 Worktree 部分失败显式化 | D | 逐 worktree 结果（成功/失败范围/原因）进入 UI；`complete` 标志驱动“部分结果”提示 |
| G-18 | 缺陷错误码 → HTTP 状态码映射 | C | 补 5 个缺陷错误码的 status 映射 + 契约测试 |
| G-19 | `fix_version` 与状态流转同语句原子写 | B | 合并为单条 `UPDATE ... SET status, lifecycle_version, fix_version` |
| G-20 | 验证独立性按实际修复者/执行者校验 | B | 引入修复者/执行者身份事实并按策略校验，例外须显式记录授权理由 |
| G-21 | 缺陷域自有 4 表缺 DDL（`defect_field_value`/`defect_relation`/`defect_comment`/`defect_draft`） | D | 补齐 4 表 DDL（含 W 类 TTL 列），或经决策确认并入既有表 |
| G-22 | `defect_status_event` 无独立表（现由 `defect.audit_events` 合并承载） | D | 按 D-DI-05 拍板结果建独立表或确认 `event_type` 枚举可满足检索 |
| G-23 | 5 张外部 owner 表的协作契约未定（证据附件/工作流策略/通知订阅/质量投影/SLA 策略） | E | 逐表确认版本化契约与授权范围并记录于 DD §4.3 |
| G-24 | W 类表 retention 未定义（`defect_draft`/`defect_quality_projection`） | D / E | 定义 TTL、容量上限与清理触发方式并写入 DD §4.2 |
| G-25 | `transition` 命令可直接置 `closed`，绕过"须有通过的独立验证" | B | 在 `transition` 入口显式校验"存在最新通过的验证且 `fix_version` 匹配"，否则拒绝并返回阻断原因 |
| G-26 | 字段可写性策略与枚举↔PG 映射测试缺失 | B / D | 实现字段可写性策略校验 + 补 UT-DI-03 映射测试 |

### F.6 发布门禁
- [ ] F.6.1 Vercel 生产分支（旧域名 `starcanvas.vercel.app`）构建通过并载入最新代码
- [ ] F.6.2 稳定版在所需门禁通过前维持 NO-GO
- [ ] F.6.3 缺口 G-1～G-26 全部关闭并留证（per 守门 #11 缺标比错标）

---

## 当前执行指令

按缺口依赖顺序推进，前置缺口不关闭则后置无法验证：

1. **G-11**（迁移文件 + RLS + 幂等回执保真）—— 解锁 PG 层一切验证
2. **G-15 / G-16**（真实会话身份 + PgPool 注入）—— 解锁授权与持久化验收
3. **G-13 / G-14 / G-17**（前端 kind 锁定 + 扩展字段真实来源 + 部分失败）—— 解锁 AC-DI-01
4. **G-12**（Outbox envelope 列）→ **G-8**（投递回路）—— 解锁通知验收
5. **G-19 / G-20**（原子写 + 实际修复者独立性）—— 解锁 AC-DI-03

严格按 `git add -- <显式文件列表>` 提交，不整体暂存或清理用户工作树改动（`snapshot.test.ts.snap` 修改与 `鍏朵腑` 等保留不动）。
稳定版在所需门禁通过前维持 **NO-GO**。

---

## 修订历史

| 日期 | 版本 | 内容 |
|------|------|------|
| 2026-11-23 | v1.0 | 建立缺陷管理模块交付工作分解结构：Phase A～F、依赖顺序、验收标准与门禁。 |
| 2026-11-24 | v1.1 | 按物理事实复核重标各阶段状态，新增缺口 G-11～G-20 与发布门禁。 |
| 2026-11-24 | v1.2 | 设计文档 IPA 模板对齐后同步：新增缺口 G-21～G-26（4 表缺 DDL / 状态事件表 / 外部 owner 契约 / W 类 retention / closed 前置校验 / 字段可写性策略）；头部登记 SRS v1.4 / BD v1.6 / DD v1.5 版本口径。稳定版维持 NO-GO。 |
