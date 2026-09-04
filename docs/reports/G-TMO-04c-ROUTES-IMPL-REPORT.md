# G-TMO-04c routes_tmo /api/tmo/metadata 5 端点落地报告 v0.1

> **报告主题**: G-TMO-04c (routes_tmo /api/tmo/metadata 端点 FastAPI 落档) 闭环
> **报告时间**: 2026-09-05 02:33 JST
> **报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **状态**: 🟢 闭环, 5 端点 + 11/11 e2e pass + 82/82 全 6 套 TMO 集成 pass
> **触发**: 9/4 17:19 JST 用户发令"完成后续全部任务" + 9/4 18:30 JST 守门 #3 反转 + 9/5 02:33 JST 自主推进

---

## §0 目的

G-TMO-04c (per HANDOFF-ST-001 v1.4 §18.6 3 待续做项) 要求 "routes_tmo /api/tmo/metadata 端点 (FastAPI)". 闭环.

**作用**: 暴露 G-TMO-04b TaskMetadataRepository 给外部 HTTP client (FastAPI TestClient + 后续 Next.js frontend), 让 metadata_node 持久化可经 HTTP 触发.

## §1 改动矩阵

### 1.1 5 端点 (per 02 §5.2 TMO 8 端点规范, 本次新加 5 个)

| 端点 | 方法 | 用途 | 守门 |
|---|---|---|---|
| `/api/tmo/metadata` | POST | M-N7 metadata_node 持久化 (走 SCD Type 2) | 守门 #13 c + 守门 #13 d SCD + audit |
| `/api/tmo/metadata/{task_id}` | GET | 读 task 当前 metadata (is_current=1) | 守门 #13 c RLS Query 必携 |
| `/api/tmo/metadata/{task_id}/history` | GET | 读 SCD Type 2 历史 (version DESC) | 守门 #13 d 关系变更留痕 |
| `/api/tmo/metadata/{task_id}/audit` | GET | 读 audit log (event_at_ms DESC) | 守门 #13 d Transaction 100% audit |
| `/api/tmo/metadata/_health` | GET | repo 状态 (db_path + ts) | 守门 #1 + 守门 #22 |

### 1.2 Pydantic 模型 (5 类)

| 模型 | 用途 | 守门 |
|---|---|---|
| `MetadataUpsertRequest` | POST 请求体 (task_id + tenant_id + workspace_id + metadata + actor_session_id) | min_length=1 强制 RLS |
| `MetadataUpsertResponse` | POST 响应 (version + is_current + name/labels/notes/priority + updated_at_ms) | - |
| `MetadataGetResponse` | GET current 响应 (含 created_at_ms + updated_at_ms) | - |
| `MetadataHistoryResponse` | GET history 响应 (history list) | - |
| `MetadataAuditResponse` | GET audit 响应 (audit_events list) | - |

### 1.3 守门修订 (FastAPI 0.135)

| 守门 | 修订前 | 修订后 | 实证 |
|---|---|---|---|
| 路由优先级 | `/metadata/{task_id}` 注册在前 → `_health` 被解析成 task_id | `/metadata/_health` 注册在前 → 优先匹配 | smoke test 9/9 + e2e 11/11 |
| Query 必携 | `tenant_id: str` 默认 required (fastapi 0.135 默认) | 显式 `Query(..., min_length=1)` + 描述 | IT-16-B-3 422 pass |

## §2 验证摘要 (实测 1.57s)

| 测试 | 结果 | 耗時 |
|---|---|---|
| `pytest tests/integration/test_routes_tmo_metadata.py` | **11/11 pass** | 0.80s |
| IT-16-A (POST upsert + Pydantic 422) | 4/4 | - |
| IT-16-B (GET current + RLS 隔离 + 422) | 3/3 | - |
| IT-16-C (GET history SCD) | 2/2 | - |
| IT-16-D (GET audit 3 类事件) | 1/1 | - |
| IT-16-E (GET _health) | 1/1 | - |
| 全 6 套 TMO + metadata | **82/82 pass** | 1.57s |

## §3 已知缺口 (per 缺标比错标)

| 缺口 | 内容 | 状态 |
|---|---|---|
| ~~G-TMO-04c~~ | ~~routes_tmo /api/tmo/metadata 端点 (FastAPI)~~ | **🟢 关闭** (本报告) |
| G-TMO-04d | metadata_node.py 集成 TaskMetadataRepository (call site) | pending 推下 session, ~0.1M |

**剩余 3 待续做项 → 2 待续做项** (G-TMO-04c 关闭).

## §4 守门规则 (实证)

| 守门 | 实证 |
|---|---|
| 守门 #13 a L0 协调 | 5 端点全部经 TaskMetadataRepository L0 唯一入口 |
| 守门 #13 c Master RLS | IT-16-A-3 + IT-16-A-4 + IT-16-B-3 Pydantic 422 + IT-16-B-2 RLS 404 |
| 守门 #13 d SCD Type 2 | IT-16-A-2 v2 + IT-16-C-2 history 派生 1 snapshot |
| 守门 #13 d Transaction audit | IT-16-D-1 3 类事件 |
| 守门 #19 Python 化 | FastAPI + Pydantic, 无 .rs 改动 |
| 守门 #22 不进 main 编译链 | routes_tmo.py 是 Python, 走 port 8080 console_server.py 独立进程 |

## §5 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 2026-09-05 02:33 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:33 JST | per 8/27 20:56 JST 强化, 真人到位后追溯签字 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:33 JST | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:33 JST | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:33 JST | 同上 |

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-05 02:33 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses** | **G-TMO-04c 落地: 5 端点 (POST upsert + GET current/history/audit/_health) + 5 Pydantic 模型 + 2 守门修订 (路由优先级 + Query 必携) + 11/11 e2e pass + 82/82 全 6 套 TMO pass** (per 守门 #13 a/c/d + 守门 #19 + 守门 #22) | **9/5 02:33 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发) → 守门 #12 commit-time docs 同步触发 v0.1** |
