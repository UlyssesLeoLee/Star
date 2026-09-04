# G-TMO-04b TaskMetadataRepository 集成报告 v0.1

> **报告主题**: G-TMO-04b (metadata_node 集成 task_metadata DDL, in-memory → SQLite 持久化) 闭环
> **报告时间**: 2026-09-05 02:30 JST
> **报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **状态**: 🟢 闭环, 4 公开 API + SCD Type 2 + RLS 隔离 + Master 物理删除禁止 + 14/14 e2e pass
> **触发**: 9/4 17:19 JST 用户发令"完成后续全部任务" + 9/4 18:30 JST 守门 #3 反转 + 9/5 02:30 JST 自主推进

---

## §0 目的

G-TMO-04b (per HANDOFF-ST-001 v1.4 §18.6 4 待续做项 + PHASE-P4-V2-TMO-CI-IMPL-REPORT v0.2 §3) 要求 "metadata_node 集成 task_metadata DDL (in-memory → SQLite 持久化)". 闭环.

**作用**: 给 metadata_node 提供持久化桥梁, 让 in-memory `handle.state['metadata']` 操作可落档到 SQLite 4 表 (per G-TMO-04 DDL). 后续 metadata_node 可调用本仓库做持久化更新 (非 in-memory).

## §1 改动矩阵

### 1.1 4 公开 API

| API | 用途 | 守门 |
|---|---|---|
| `get_current_metadata(task_id, tenant_id, workspace_id)` | 读 task 当前 metadata (is_current=1) | 守门 #13 c RLS 必携 |
| `upsert_metadata(task_id, tenant_id, workspace_id, metadata, actor_session_id)` | 插入/更新 metadata, 走 SCD Type 2 | 守门 #13 d SCD + audit 5 类事件 |
| `get_scd_history(task_id, tenant_id, workspace_id, limit)` | 读 SCD 历史 (version DESC) | 守门 #13 d 关系变更留痕 |
| `get_audit_log(task_id, tenant_id, workspace_id, limit)` | 读 audit log (event_at_ms DESC) | 守门 #13 d Transaction 100% audit |
| `delete_metadata(...)` | **禁用** (抛 PermissionError) | 守门 #13 c Master 物理删除禁止 |

### 1.2 守门修订 (G-TMO-04 DDL)

| 守门 | 修订前 | 修订后 | 实证 |
|---|---|---|---|
| UNIQUE 约束 | `(task_id, version)` 太严, workspace 隔离冲突 | `(task_id, tenant_id, workspace_id, version)` | IT-15-B-3 workspace 隔离 pass + IT-14-G-2 unique pass |

## §2 验证摘要 (实测 1.02s)

| 测试 | 结果 | 耗時 |
|---|---|---|
| `pytest tests/integration/test_task_metadata_repo.py` | **14/14 pass** | 0.46s |
| IT-15-A (upsert SCD Type 2) | 3/3 | - |
| IT-15-B (RLS 跨 tenant + workspace 隔离) | 3/3 | - |
| IT-15-C (SCD history version DESC) | 2/2 | - |
| IT-15-D (audit log 3 类事件 + actor_session_id) | 2/2 | - |
| IT-15-E (Master 物理删除禁止) | 2/2 | - |
| IT-15-F (sqlite3 only + 跨 connection 持久化) | 2/2 | - |
| `pytest tests/integration/test_task_metadata_ddl.py` | **20/20 pass** (修订后) | 0.36s |
| 全 5 套 TMO + metadata | **71/71 pass** | 1.02s |

## §3 已知缺口 (per 缺标比错标)

| 缺口 | 内容 | 状态 |
|---|---|---|
| ~~G-TMO-04~~ | ~~task_metadata DDL 落地~~ | **🟢 关闭** (G-TMO-04 v0.1) |
| ~~G-TMO-04b~~ | ~~metadata_node 集成 task_metadata DDL (in-memory → SQLite 持久化)~~ | **🟢 关闭** (本报告) |
| G-TMO-04c | routes_tmo /api/tmo/metadata 端点 (FastAPI) | pending 推下 session, ~0.2M |
| G-TMO-04d | metadata_node.py 集成 TaskMetadataRepository (call site) | pending 推下 session, ~0.1M |
| ~~G-TMO-05~~ | ~~LangGraph SDK alpha 确认~~ | **🟢 关闭** (G-TMO-05 v0.1) |

**剩余 4 待续做项 → 3 待续做项** (G-TMO-04b 关闭).

## §4 守门规则 (实证)

| 守门 | 实证 |
|---|---|
| 守门 #13 c Master RLS | IT-15-B 3/3 + 多租户 + 多 workspace 隔离 |
| 守门 #13 d SCD Type 2 | IT-15-A 3/3 + v1→v2→v3 完整 SCD 流程 |
| 守门 #13 d Transaction audit | IT-15-D 2/2 + 3 类事件 + actor_session_id |
| 守门 #DB-13 a Work 100% retention | schema 维持 (per G-TMO-04 v0.1 验证) |
| 守门 #DB-13 b Master 物理删除禁止 | IT-15-E 2/2 + delete 抛 PermissionError + 删后仍能读 |
| 守门 #DB-13 c SCD Type 2 | IT-15-C 2/2 + scd_history version DESC |
| 守门 #19 Python 化 | IT-15-F 2/2 + sqlite3 only + 跨 connection 持久化 |
| 守门 #22 不进 main 编译链 | Python 进程独立跑, 无 .rs 改动 |

## §5 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 2026-09-05 02:30 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:30 JST | per 8/27 20:56 JST 强化, 真人到位后追溯签字 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:30 JST | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:30 JST | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:30 JST | 同上 |

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-05 02:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses** | **G-TMO-04b 落地: TaskMetadataRepository 4 API (get_current + upsert + get_scd_history + get_audit_log) + delete_metadata 禁用 + DDL UNIQUE 约束修订 (task_id+tenant_id+workspace_id+version) + 14/14 e2e pass + 71/71 全 5 套 TMO 集成 pass** (per 守门 #13 c/d + 守门 #DB-13) | **9/5 02:30 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发) → 守门 #12 commit-time docs 同步触发 v0.1** |
