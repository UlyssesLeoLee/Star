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

**说明**:
- 末次 commit 列填 `TBD` = 本批次 v0.1 初版, commit 落地后回填
- 状态 🟡 stub = 框架落地, 真实对接 (Mavis task 调度 / cargo 子命令 / git stash) 待续
- 状态 🟢 完成 = 框架 + smoke + 真实对接都完成 (本批次 4 份: __init__ × 2 + judge + smoke_test + registry_check)

---

## 2. 子代理任务索引 (per dispatcher.py 落档)

| task_id | brief 路径 | output 路径 | status.json 路径 | 调用方 |
|---|---|---|---|---|
| (待续) | `docs/briefs/<task_id>.md` | `docs/briefs/<task_id>.output.md` | `docs/briefs/<task_id>.status.json` | (待续) |

**说明**: 本表 v0.1 初版为空, 跨 session 续做 (H2 强类型重构 / P3-B.5/B.6 真实 e2e 等) 落档后回填。

---

## 3. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 8 份脚本索引 (__init__.py × 2 + dispatcher + cli_helper/base + refactor_template + judge + smoke_test + registry_check), 任务卡调用方映射, 状态 🟢/🟡 分档 | 2026-09-02 00:39 JST 拍板 "新建 docs/automation-design.md + scripts/automation/ 落档" + 守门 #12 派生 v2 |

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

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 8 份脚本索引 | 2026-09-02 00:39 JST 拍板 |
| v0.2 | 2026-09-03 | 架构师 (Mavis 接手 agent per DEC-008) | 新增 §5 P3-G Agent Jira 化阶段索引: 5 段 20 子项 G.1-G.20, W1 1.9M + W2-W5 4.0M, 命名空间 P3-G 跟 P3-B (OpenClaw) 9 子项共存 | 2026-09-03 11:50 JST Ulysses Jira 化指令 + 3 步 ask_user 拍板 + 守门 #21 [P] docs 同步 |
| v0.3 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 脚本索引表 新增 `kanban_sprint_gen.py` (KANBAN-SPRINT-001 P1 Sprint 视图 验证, 43/43 pass) | 2026-09-03 13:25 JST P1 收官, 守门 #1 v19 + #21 v21 实证 |
| v0.4 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 索引说明更新 (kanban_sprint_gen.py 43→55 项) + KANBAN-SPRINT-001 落地 P1 v0.2 Jira 設計 (per docs/briefs/kanban-sprint-view-001.md v0.2) + P2 度量 (Velocity/Burndown/History/Capacity) | 2026-09-03 13:55 JST P1 v0.2 + P2 收官, commit `947c0ef` 落地 |
| v0.5 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §1 索引说明更新 (kanban_sprint_gen.py 55→93 项) + KANBAN-SPRINT-001 P3 仪式 收官 (Goal + Standup + Review + Retrospective + Markdown 导出) | 2026-09-03 14:05 JST P3 拍板 + 14:20 JST 收官, KANBAN-SPRINT-001 三阶段全部收官 |