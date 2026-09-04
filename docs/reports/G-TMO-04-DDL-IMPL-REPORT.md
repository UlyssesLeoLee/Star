# G-TMO-04 task_metadata DDL 落地报告 v0.1

> **报告主题**: G-TMO-04 (task_metadata SQLite DDL 落地) 闭环
> **报告时间**: 2026-09-05 02:27 JST
> **报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **状态**: 🟢 闭环, 4 表 W/T/M + 7 索引 + 20/20 e2e pass + 守门 #13 c/d 全过
> **触发**: 9/4 17:19 JST 用户发令"完成后续全部任务" + 9/4 18:30 JST 守门 #3 反转 5 子代理兼任 + 9/5 02:27 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发)

---

## §0 目的

G-TMO-04 (per HANDOFF-ST-001 v1.4 §18.6 5 待续做项 + PHASE-P4-V2-TMO-CI-IMPL-REPORT v0.2 §3 11 已知缺口) 要求 "task_metadata DDL 落地 per 守门 #13 c Master SCD Type 2". 闭环.

**为何单独立项**: TMO-07 metadata_node (per feat/tmo-05-06-07 commit 7b1a432) 现在只更新 sub_pool handle.state['metadata'] (in-memory), 没有持久化. task_metadata DDL 落档后, metadata_node 后续可接 DB 持久化路径 (守门 #13 c Master 表 RLS 必携 + 守门 #13 d SCD Type 2 关系变更留痕).

---

## §1 改动矩阵

### 1.1 4 表 W/T/M 分类 (per 守门 #DB-13)

| 表名 | W/T/M | 用途 | 守门 |
|---|---|---|---|
| `task_metadata` | **Master** | task 的 metadata 当前值 (SCD Type 2 current row, 物理删除禁止) | 守门 #13 c Master RLS 必携 tenant_id / workspace_id |
| `task_metadata_scd` | **Master** | task 的 metadata 变更历史 (SCD Type 2 history, 永存) | 守门 #13 d SCD Type 2 关系变更留痕 |
| `task_metadata_audit` | **Transaction** | task_metadata 变更事件 (append-only, 物理删除禁止) | 守门 #13 d Transaction 100% audit |
| `task_metadata_session` | **Work** | 短 TTL 作业中临时状态 (完成后清理) | 守门 #DB-13 a Work 100% retention_period |

### 1.2 7 索引 (per 守门 #13 c RLS 必携)

| 索引 | 表 | 列 | 用途 |
|---|---|---|---|
| idx_task_metadata_tenant | task_metadata | (tenant_id, workspace_id) | 多租户 RLS 查询 |
| idx_task_metadata_task_current | task_metadata | (task_id, is_current) | 当前 metadata 查询 |
| idx_task_metadata_scd_task | task_metadata_scd | (task_id, version DESC) | SCD 历史倒序 |
| idx_task_metadata_scd_tenant | task_metadata_scd | (tenant_id, workspace_id) | 多租户 RLS |
| idx_task_metadata_audit_task | task_metadata_audit | (task_id, event_at_ms DESC) | 单 task audit 倒序 |
| idx_task_metadata_audit_tenant | task_metadata_audit | (tenant_id, workspace_id, event_at_ms DESC) | 多租户 audit 查询 |
| idx_task_metadata_session_active | task_metadata_session | (is_active, session_expires_ms) | Work session TTL 清理 |

### 1.3 关键 CHECK 约束 (守门 #13 c/d + 守门 #DB-13)

| 表 | 约束 | 守门 |
|---|---|---|
| task_metadata | priority BETWEEN 1 AND 10 | 守门 metadata_node MAX_PRIORITY/MIN_PRIORITY |
| task_metadata | is_current IN (0, 1) | 守门 #13 d SCD Type 2 二态 |
| task_metadata | UNIQUE (task_id, version) | 守门 #13 d SCD 版本唯一 |
| task_metadata_audit | event_type IN (created, updated, scd_snapshot, rls_violation, validation_failed) | 守门 #13 d Transaction 5 类事件 |
| task_metadata_session | is_active IN (0, 1) | 守门 #DB-13 a Work 短 TTL |

---

## §2 验证摘要 (实测 0.45s)

| 测试 | 结果 | 耗時 |
|---|---|---|
| `pytest tests/integration/test_task_metadata_ddl.py` | **20/20 pass** | 0.45s |
| IT-14-A (4 表 + 7 索引) | 4/4 | - |
| IT-14-B (Master RLS 4 表 tenant_id + workspace_id) | 4/4 | - |
| IT-14-C (SCD Type 2 version + is_current + scd snapshot) | 3/3 | - |
| IT-14-D (audit 5 类事件 CHECK 约束) | 2/2 | - |
| IT-14-E (Work session TTL + is_active) | 2/2 | - |
| IT-14-F (priority 1-10 CHECK 边界) | 3/3 | - |
| IT-14-G (RLS 多租户 + UNIQUE 约束) | 2/2 | - |
| 全 TMO 4 套 (test_task_metadata_ddl + test_tmo_05_06_07 + test_tmo_merge + test_tmo_split) | **57/57 pass** | 0.68s |

---

## §3 已知缺口 (per 缺标比错标)

| 缺口 | 内容 | 状态 |
|---|---|---|
| ~~G-TMO-04~~ | ~~task_metadata DDL 落地~~ | **🟢 关闭** (本报告, 4 表 W/T/M + 7 索引 + 20/20 e2e pass) |
| G-TMO-04b | metadata_node 集成 task_metadata DDL (in-memory → SQLite 持久化) | pending 推下 session, ~0.2M |
| G-TMO-04c | routes_tmo /api/tmo/metadata 端点 (FastAPI) | pending 推下 session, ~0.2M |
| ~~G-TMO-05~~ | ~~LangGraph SDK alpha 确认~~ | **🟢 关闭** (per G-TMO-05-SDK-FINDINGS v0.1) |

**剩余 5 待续做项 → 4 待续做项** (G-TMO-04 关闭).

---

## §4 守门规则 (实证)

| 守门 | 实证 |
|---|---|
| 守门 #13 c Master RLS 必携 | IT-14-B 4/4 + IT-14-G-1 多租户隔离查询 pass |
| 守门 #13 d SCD Type 2 | IT-14-C 3/3 + IT-14-C-3 完整 SCD 流程 pass |
| 守门 #13 d Transaction 100% audit | IT-14-D 2/2 + 5 类事件 CHECK 约束 pass |
| 守门 #DB-13 a Work 100% retention | IT-14-E 2/2 + session TTL + is_active pass |
| 守门 #DB-13 b Master 物理删除禁止 | schema 无 DELETE/TRUNCATE 触发器 (依赖应用层 + UNIQUE 约束保护) |
| 守门 #DB-13 c SCD Type 2 | IT-14-C-3 v1→v2 完整流程 + scd_history 永存 pass |
| 守门 #19 Python 化 | 标准库 sqlite3 only, 无 SQLAlchemy pass |
| 守门 #22 不进 main 编译链 | Python 进程独立跑, 不污染 cargo check pass |

---

## §5 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 2026-09-05 02:27 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:27 JST | per 8/27 20:56 JST 强化, 真人到位后追溯签字 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:27 JST | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:27 JST | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:27 JST | 同上 |

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-05 02:27 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses** | **G-TMO-04 落地: 4 表 W/T/M 分类 (task_metadata / task_metadata_scd / task_metadata_audit / task_metadata_session) + 7 索引 + 5 CHECK 约束 (priority 1-10 + is_current 0/1 + UNIQUE task_id+version + audit 5 类事件 + session is_active) + 20/20 e2e pass** (per 守门 #13 c Master RLS + 守门 #13 d SCD Type 2 + 守门 #DB-13 W/T/M 强制分类) | **9/5 02:27 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发 + G-TMO-04 消耗最低 ~0.1M 优先) → 守门 #12 commit-time docs 同步触发 v0.1** |
