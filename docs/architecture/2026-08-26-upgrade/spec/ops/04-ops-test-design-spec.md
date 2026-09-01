# SRE Ops 后台 — 测试设计 (Test Design)

> **状态**: Draft v0.1 (2026-09-01)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **关联**: `01-ops-requirements-spec.md` v0.1 + `02-ops-basic-design-spec.md` v0.1 + `03-ops-detailed-design-spec.md` v0.1
> **基线**: `docs/test-design.md` v0.2 (Q1-D + UAT 已落地)

---

## 1. 测试原则 (per `test-design.md` §1)

1. **测试金字塔**: 单元 60% / 集成 30% / E2E 10% (SRE 后台偏集成 + E2E, 0% 单元)
2. **真人到位 (SRE Lead) 才能跑端到端** (per 守门 #1 派生 v1, 强 WebAuthn)
3. **审计日志** 所有写操作必有测试 (per NFR-OPS-002 强约束)
4. **5 域命名 disclaimer** 必在所有报表 label (per REQ-OPS-012)
5. **跨域数据隔离** 测试: 后台操作不污染业务数据 (per §4.5)

---

## 2. 测试层级 (per `test-design.md` §2)

### 2.1 单元测试 (60% — 但 SRE 后台 0% 单元, 全部 Mock + 集成)
- **覆盖**: API request/response schema, 状态机转移, 错误码映射
- **位置**: `crates/domain-ops/src/**/tests.rs` + `frontend/src/mocks/__tests__/ops/`
- **mock**: 现有 MSW + UAT 测试数据 (per UAT-1 拍板 vitest + MSW e2e)
- **目标覆盖率**: 80% (per `test-design.md` §11.1)

### 2.2 集成测试 (30%)
- **覆盖**: domain-ops + domain-audit + domain-permission + OpenTelemetry 集成
- **位置**: `crates/domain-ops/tests/integration/`
- **mock**: OpenTelemetry collector / Prometheus / k3s API 用 testcontainers-rs 或 in-process stub

### 2.3 E2E 测试 (10% — Playwright, per `test-design.md` §5)
- **覆盖**: SRE Lead 真机 4 步二次确认流程 + 限流/缓存/K8s 写操作
- **位置**: `frontend/e2e/ops/*.spec.ts` (复用现有 Playwright 配置)
- **前置**: SRE Lead WebAuthn 凭据 + 真人现场操作

---

## 3. 测试用例 (per 12 REQ + 5 NFR + 4 步状态机)

### 3.1 REQ-OPS-001 跨域日志查询 (12 用例)
| # | 场景 | 输入 | 期望 |
|---|---|---|---|
| T-001 | 正常关键字查询 | query="error", time_range 1h | 200 + events 数组 |
| T-002 | 多维度过滤 | query + tenant + domain + level | 200 + 过滤后 events |
| T-003 | trace_id 精确查询 | trace_id="abc123" | 200 + 1 个 event |
| T-004 | 时间窗口反向 | from > to | 400 INVALID_QUERY |
| T-005 | limit > 1000 | limit=5000 | 400 INVALID_QUERY |
| T-006 | 1M events 范围 | time_range 30d | 200 + took_ms < 5000 |
| T-007 | 跨 domain 联合 | 22 domain 都过滤 | 200 + 聚合结果 |
| T-008 | 后端不可达 | OTel collector 模拟 down | 502 BACKEND_UNREACHABLE |
| T-009 | 权限缺失 | actor.role != SreLead | 403 PERMISSION_DENIED |
| T-010 | PII 脱敏 | message 含 PII | 200 + message hash 后展示 |
| T-011 | 大查询性能 | 1M events 5 维度过滤 | 200 + took_ms < 5s |
| T-012 | 5 域 disclaimer | 报表 label | 含 "5 域是历史治理命名" 文本 |

### 3.2 REQ-OPS-005 限流调控 (15 用例, 4 步状态机)
- T-101 ~ T-105: 4 步状态机 happy path (preview / challenge / apply / done)
- T-106 ~ T-110: 异常 path (cancel / WebAuthn failed / 二次确认超时 / 限流冲突 / 跨租户)
- T-111 ~ T-115: 安全 (SRE 角色缺失 / WebAuthn 缺失 / 全局限流 special case / 临时限流过期 / audit log 写入失败)

### 3.3 REQ-OPS-006 缓存清理 (10 用例)
- T-201 ~ T-205: redis 清理 (by domain / by pattern / by key)
- T-206 ~ T-208: llm_response 清理 (by model / by workspace / by age)
- T-209 ~ T-210: rag_vector 清理 + 5min verify hit rate

### 3.4 REQ-OPS-009 Kubernetes 运维 (12 用例)
- T-301 ~ T-304: 读 (pod list / deployment / HPA / service mesh)
- T-305 ~ T-310: 写 (scale / restart / cordon / drain / rollout undo / 2-step 确认)
- T-311 ~ T-312: 安全 (k3s API 不可达 / SRE 角色缺失)

### 3.5 NFR-OPS-001/002 权限 + 审计 (8 用例)
- T-401: WebAuthn 缺失
- T-402: WebAuthn 失败
- T-403: 跨租户写操作
- T-404: audit log 写入失败 (强制 rollback)
- T-405: audit log 不可篡改 (append-only)
- T-406: 强 MFA 强制 (per 8/21 JST 拒绝兼任硬约束)
- T-407: WebAuthn 凭据过期
- T-408: 多个 SRE Lead 并发 (互斥)

### 3.6 NFR-OPS-003 性能 (6 用例, per 守门 #1 派生 v5 P95)
- T-501: 列表页 P95 < 1s
- T-502: 详情查询 P95 < 5s
- T-503: 写操作 P95 < 2s
- T-504: 限流 (per NFR-OPS-005)
- T-505: 并发 (10 SRE 同时操作)
- T-506: 大数据 (1M events 5min 持续查询)

### 3.7 REQ-OPS-012 5 域命名 disclaimer (5 用例)
- T-601: UI 顶部 banner 显示
- T-602: 报表 label 包含
- T-603: 5 域指标 label 不映射 22 domain
- T-604: 错误信息不混用 5 域 / DDD 命名
- T-605: ST 测试报告 disclaimer (per HANDOFF-ST-001 v0.3 §1.1)

---

## 4. 跨域测试 (per 守门 #9 实证: 子代理 RPC 不可靠, 必须 git log)

- 跨域一致性: 22 domain 业务接口不污染 SRE 后台接口
- 跨 session 测试: SRE 后台不依赖任何 5 域 Lead 真人 (但 5 域报表依赖, 跨 session 续)
- 跨域 audit: 任何 domain 的写操作都进 domain-audit, SRE 后台不能跨

---

## 5. 已知缺口 (per 守门 #11 缺标比错标安全)

1. **SRE Lead 真人到位** (per 守门 #5): WebAuthn 二次验证 + 强 MFA 测试依赖真人凭据, 跨 session 续
2. **5 域 Lead 真人到位** (per HANDOFF v0.6 §5.3 Blocker 4): 5 域报表 (REQ-OPS-008) 测试需要 5 域 Lead 配合, 跨 session 续
3. **WebAuthn 集成规范** (per NFR-OPS-001): 当前缺, 测试需等集成, 跨 session 续
4. **限流风暴场景** (per 已知缺口 #3): 一个 SRE 操作触发 22 domain 联锁限流, 测试需细化
5. **数据脱敏规则** (per 已知缺口 #4): REQ-OPS-007 PII 脱敏规则未细化, 测试需拍板

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 4 测试层级 + 67 测试用例覆盖 12 REQ + 5 NFR + 5 已知缺口 | 2026-09-01 13:04 JST |
