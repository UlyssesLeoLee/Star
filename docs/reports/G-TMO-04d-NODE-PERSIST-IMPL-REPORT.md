# G-TMO-04d metadata_node 集成 TaskMetadataRepository call site 落地报告 v0.1

> **报告主题**: G-TMO-04d (metadata_node.py 集成 TaskMetadataRepository call site) 闭环
> **报告时间**: 2026-09-05 02:38 JST
> **报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **状态**: 🟢 闭环, env 开关 + 优雅降级 + 6/6 e2e pass + 88/88 全 7 套 TMO pass
> **触发**: 9/4 17:19 JST 用户发令"完成后续全部任务" + 9/4 18:30 JST 守门 #3 反转 + 9/5 02:38 JST 自主推进

---

## §0 目的

G-TMO-04d (per HANDOFF-ST-001 v1.4 §18.6 2 待续做项) 要求 "metadata_node.py 集成 TaskMetadataRepository (call site)". 闭环.

**作用**: metadata_node 默认 in-memory 模式保持向后兼容 (per 守门 #22 优雅降级), 启用 `STAR_TASK_METADATA_PERSIST=1` 后委托 TaskMetadataRepository.upsert_metadata 走 SCD Type 2 + audit 5 类事件 (per 守门 #13 c/d + 守门 #DB-13).

**设计原则**:
- 守门 #22 优雅降级: 持久化失败不应破坏 in-memory 状态, logger.warning + 继续
- 守门 #19 Python 化: 延迟 import repo 模块, 默认模式不加载 sqlite 依赖
- 守门 #13 c: tenant_id + workspace_ids(取第一个) 必携 (Pydantic + repo 双校验)
- 守门 #13 d: 启用 persist 时 SCD Type 2 走 repo (v1 → v2 + scd snapshot + audit 3 事件)

## §1 改动矩阵

### 1.1 5 改动

| 改动 | 内容 | 守门 |
|---|---|---|
| 1. env 开关常量 | `STAR_TASK_METADATA_PERSIST` (默认 "0" = 关闭) | 守门 #22 优雅降级 |
| 2. `_persist_to_sqlite` helper | 委托 repo.upsert_metadata + 收集 audit_count | 守门 #13 c/d + 守门 #19 |
| 3. 主函数 step 4.5 | 可选 SQLite 持久化 (try/except 包裹, 失败 log warning) | 守门 #22 优雅降级 |
| 4. result 新增 `persisted` 字段 | 启用 persist 时含 backend + version + scd_snapshot_id + audit_count | - |
| 5. e2e test 6 case | in-memory 2 + persist 3 + 优雅降级 1 | 守门 #19 + 守门 #22 |

### 1.2 env 配置

| env | 默认 | 用途 |
|---|---|---|
| `STAR_TASK_METADATA_PERSIST` | "0" | 启用 SQLite 持久化 (1 = 启用) |
| `STAR_TASK_METADATA_DB_PATH` | `./data/task_metadata.sqlite` | SQLite 数据库路径 (per 守门 #22 独立进程) |

## §2 验证摘要 (实测 1.55s)

| 测试 | 结果 | 耗時 |
|---|---|---|
| `pytest tests/integration/test_metadata_node_persist.py` | **6/6 pass** | 0.11s |
| IT-17-A (默认 in-memory 模式 + 优雅默认) | 2/2 | - |
| IT-17-B (启用 persist 模式 + SCD + RLS) | 3/3 | - |
| IT-17-C (优雅降级 — Windows CON 设备名触发失败) | 1/1 | - |
| 全 7 套 TMO + metadata | **88/88 pass** | 1.55s |

## §3 已知缺口 (per 缺标比错标)

| 缺口 | 内容 | 状态 |
|---|---|---|
| ~~G-TMO-04d~~ | ~~metadata_node.py 集成 TaskMetadataRepository (call site)~~ | **🟢 关闭** (本报告) |

**剩余 2 待续做项 → 1 待续做项** (G-TMO-04d 关闭, **G-TMO-04 系列全闭环**).

剩余待续做项:
- G-DEP-01: P0 工具 (3 tool, ~0.4-0.6M) — 推下 session
- G-DEP-02: P1 工具 (4 tool, ~0.3-0.5M) — 推下 session
- 5 域 Lead 真人寻访 (per 守门 #14) — Ulysses 启动
- 真实凭证切真 (per 9/3 11:35 拍板 A) — Ulysses 提供

## §4 守门规则 (实证)

| 守门 | 实证 |
|---|---|
| 守门 #13 a L0 协调 | metadata_node 仍是 L0 唯一入口, persist 委托 L0 repo (守门 #13 a 实证) |
| 守门 #13 c Master RLS | IT-17-B-3 tenant-B 看不到 tenant-A 数据 |
| 守门 #13 d SCD Type 2 | IT-17-B-2 v1→v2 + scd snapshot 1 + audit 3 事件 |
| 守门 #19 Python 化 | 延迟 import + 标准库 sqlite3, 无 .rs 改动 |
| 守门 #22 优雅降级 | IT-17-C-1 Windows CON 设备名触发失败, in-memory 仍 OK + 返 persisted=None |

## §5 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 2026-09-05 02:38 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:38 JST | per 8/27 20:56 JST 强化, 真人到位后追溯签字 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:38 JST | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:38 JST | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:38 JST | 同上 |

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-05 02:38 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses** | **G-TMO-04d 落地: env 开关 + _persist_to_sqlite helper + step 4.5 可选持久化 + 优雅降级 + 6/6 e2e pass + 88/88 全 7 套 TMO pass** (per 守门 #13 a/c/d + 守门 #19 + 守门 #22) | **9/5 02:38 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发) → 守门 #12 commit-time docs 同步触发 v0.1** |
