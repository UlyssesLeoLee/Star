# Brief: P3-CHARTS-P0

**Agent**: Mavis 自写 (无子代理 dispatch, per 守门 #9 + 守门 #20 子代理 status 实证不可靠, 自写可控)
**Phase**: P3-CHARTS (新 phase, per docs/automation-design.md §4 新增)
**Created**: 2026-09-02 11:00 JST
**Status**: 🟡 running (per docs/automation-design.md §2.3 [P] 判定)

---

## 0. 触发

per 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira, 各个图表设计要完善" + 11:00 JST 拍板 (ask_user):
- 方向 = A 先基础设施 + 示例全跑通
- 深度 = I 完整可跑
- 测试 = α 单元 + 集成

依赖:
- docs/requirements/charts-and-reports.md v1.0 (commit c2836a7)
- docs/basic-design/charts-and-reports.md v1.0 (commit c2836a7)
- docs/specs/domain-report-spec.md v1.0 (commit c2836a7)
- 22 份详细设计 (commit c2836a7)

base: c2836a7 (commit [CHARTS] 26 文档 v1.0 落地)

---

## 1. 分阶段 (per 守门 #19 v19 升档 [P])

跨 stage 累计消耗主上下文预估 ≥ 5K token (8 图表 × ~600 行 Rust + ~250 行 TSX + ~150 行测试 = ~8K 行), 自动升档 [P]。

按守门 #19 v19 派生规 + 守门 #21 v21:
- 任何 [P] 子项落档后必更新 `docs/automation-design.md` §4 任务卡表 + `scripts/automation/registry.md` 索引
- commit message 引用 `automation-design.md §N.M` 章节号

### 阶段 1 — 基础设施 + C01 Burndown 完整跑通 (本次)

**scope**:
1. frontend `package.json` 加 Recharts ^2.12.0 + d3-scale ^4.0.2 + d3-scale-chromatic ^3.1.0
2. `crates/domain-report/` 完整 crate 骨架 (Cargo.toml + src/lib.rs + 14 .rs)
3. 22 图表注册表 (PHF map 编译期穷举, 阶段 1 注册 8 P0 + 14 stub)
4. 5min TTL in-memory cache trait (Redis 实现留 V2, per 阶段限制)
5. 4 个 Port trait + 内存实现 (WorkItemQueryPort / SprintQueryPort / UserQueryPort / PermissionPort)
6. C01 Burndown 完整链路:
   - 后端: SQL + Port impl + 缓存 + 错误处理 + 单元测试 5 case + 集成测试 (RLS + cache invalidation)
   - 前端: Chart01Burndown.tsx (Recharts 完整 + Tooltip + i18n zh-CN + a11y)

**交付 (本次 commit)**:
- `frontend/package.json` 加 3 依赖
- `crates/domain-report/Cargo.toml`
- `crates/domain-report/src/lib.rs` (模块导出)
- `crates/domain-report/src/domain/{chart_type, chart_config, chart_registry}.rs`
- `crates/domain-report/src/domain/c01_burndown.rs` (C01 聚合)
- `crates/domain-report/src/application/{cache, ports, command_service, query_service}.rs`
- `crates/domain-report/src/infrastructure/{in_memory_cache, port_impls}.rs`
- `crates/domain-report/src/api/rest_handlers.rs`
- `crates/domain-report/tests/e2e_c01.rs`
- `frontend/src/components/charts/Chart01Burndown.tsx`
- `frontend/src/components/charts/shared/ChartFrame.tsx`
- `frontend/src/lib/chart-data-schema.ts`
- `frontend/src/i18n/charts/zh-CN.json` (C01 子集)

**守门 (per AGENTS.md §4 #1)**:
- `cargo check --workspace --lib` 0 err
- `cargo clippy --workspace --lib` 0 err
- `cargo test --workspace --lib -p domain-report` ≥ 5 单元 + 1 集成 全过
- `cd frontend && pnpm typecheck` 0 err
- frontend `package.json` Recharts 加完 + 后续 `pnpm install` (网络依赖, 留用户跑)

**docs 同步**:
- commit message 含 `scripts/automation/charts_p0_setup.py` + `scripts/automation/charts_p0_c01.py` 路径
- 更新 `docs/automation-design.md` §4 任务卡表 (新增 P3-CHARTS-P0 行)
- 更新 `scripts/automation/registry.md` 索引 (新增 2 脚本行)

### 阶段 2 — P0 剩余 7 图表 (C02-C05, C06, C07, C13) 批量

**scope**: 复用 C01 模板, 7 图表各 ~150 行 Rust + ~150 行 TSX + ~80 行测试

**守门**: 同阶段 1, 7 图表 × 测试 ≥ 35 case + 7 集成

### 阶段 3 — 守门收尾 + commit + 守门员汇报

---

## 2. 守门

(per AGENTS.md §4 #1 + 守门 #19 派生 v19 + 守门 #20 v20 + 守门 #21 v21)

### 2.1 守门员 (per 守门 #6 PowerShell only)

全部 PowerShell 命令, 不混 bash:
- `cargo check --workspace --lib` (用 ; 不 &&)
- `cargo clippy --workspace --lib`
- `cargo test --workspace --lib -p domain-report`
- `Get-ChildItem` 替 `ls -la`
- `Select-String` 替 `grep`

### 2.2 守门 #1 累积规 v1-v14

- v1: `cargo check --workspace --lib` 0 err
- v2: `--all-targets` 含 tests 0 err
- v3: check + fmt + clippy 不替代 cargo test
- v4: 单 crate 100% pass ≠ workspace pass
- v5: release + doc + bench `--no-run` 与 debug build 等价守门
- v6: release mode test 100% pass (单 crate)
- v7: multi-crate test 守门覆盖率持续提升

### 2.3 守门 #12 死循环饱和边界 (per 5cfb7b3)

本次有明确新事件 (Ulysses 拍板 + 4 维回复 + 3 维 A+I+α), 满足触发条件, 允许 docs 同步 commit。

### 2.4 守门 #13 DB W/T/M 三類横展開

`crates/domain-report/migrations/` (新建):
- W 表: `report_generation_task` (24h retention)
- T 表: `report_view_audit` (append-only, RLS 必携)
- M 表: `report_definition` (SCD Type 2, RLS 必携)
- M 表: `report_subscription` (SCD Type 2, RLS 必携)
- 全部含 `tenant_id` 必携
- migration 落地: 阶段 1 仅 schema 描述, 实际 migration 文件留实施时生成 (per 已知缺口)

### 2.5 守门 #19 v19 派生 + 守门 #20 v20 + 守门 #21 v21

- ✅ 本 brief 落 `docs/briefs/P3-CHARTS-P0.md`
- ✅ commit message 含 `scripts/automation/charts_p0_setup.py` + `charts_p0_c01.py` 路径
- ✅ 更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md`
- 不走子代理 dispatch (Mavis 自写, 守门 #20 子代理 status 实证不可靠)

---

## 3. 已知缺口 (per 守门 #11 缺标比错标安全)

1. Redis 实际连接留 V2, 阶段 1 走 in-memory cache (per 守门 #1 验证最小)
2. 实际 migration 文件 (refinery/sqlx) 待实施时生成
3. `pnpm install` 网络依赖, 用户后续跑 (Mavis 无 npm registry 凭证, per AGENTS.md §4 #5 env 变量安全)
4. 22 图表路由表阶段 1 仅注册 8 P0, P1/P2 stub (后续阶段补)
5. C01 测试骨架 5 case, 后续阶段补全 (per 详细设计 §8)
6. i18n 阶段 1 仅 zh-CN, en-US/ja-JP 后续补

---

## 4. git 实证要求 (per 守门 #1 禁回溯叙事)

- commit author: `Ulysses <ulysses@mavis.local>` (per AGENTS.md §2.1)
- commit message 含本 brief 路径 `docs/briefs/P3-CHARTS-P0.md`
- commit message 含脚本相对路径 `scripts/automation/charts_p0_setup.py` + `charts_p0_c01.py`
- 修订人 / 审批者 按 AGENTS.md §2.2 / §2.3 代签

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 11:00 JST | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 阶段 1 基础设施 + C01 完整链路, 3 阶段分批 | 2026-09-02 11:00 JST Ulysses 拍板 A+I+α |
