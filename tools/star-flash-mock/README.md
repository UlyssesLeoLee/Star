# Star Mock Project (star-flash-mock)

> **版本**: v0.2 (2026-09-05 JST 升版, per 2026-09-04 17:47 JST "测试脚本+数据归入 mock 项目" + 2026-09-05 06:50 JST user 拍板 "全栈覆盖 v0.7" + 2026-09-05 06:50 JST "推进" + P5 DB W/T/M 100% 表覆蓋)
> **范围**: Star 项目全栈测试用 mock 项目 (LangGraph + Agent Runtime + 16 MCP tools + Streamable HTTP + DB W/T/M 100% 覆蓋 + 5 域业务 + OpenClaw v1 既有 fixture)
> **守门**: 守门 #5 环境变量安全 / 守门 #11 缺标比错标 / 守门 #13 a W=短 TTL 物理删 / b T=append-only / c M=RLS+SCD / d T 100% audit / e M 100% RLS + 派生守門 10 条 CW-01~CW-10

---

## 0. 目的 (Purpose)

集中 Star 项目的所有测试脚本和测试数据到独立 mock 项目, 满足 2026-09-04 17:47 JST 用户拍板"所有开发过程中的测试脚本和测试数据都应归入 mock 项目以备回归测试"。本项目是 Star 主仓 (`D:\Star`) 内部工具, **不**独立发布, 但**镜像** `D:\RustGameServer\tools\rgs-flash-mock` 治理结构。

## 1. 目录结构 (Directory Layout)

```
D:\Star\tools\star-flash-mock\
├── README.md                              # 本文件
├── scripts/                               # 测试脚本 (回归)
│   ├── smoke-test.sh                      # 1. smoke (5 类基础设施)
│   ├── regression-test-langgraph.sh       # 2. LangGraph TMO + 9 SA + SA-10
│   ├── regression-test-agent-runtime.sh   # 3. Agent Runtime L0/L1/L2
│   ├── regression-test-mcp.sh             # 4. 16 MCP tool
│   ├── regression-test-streamable-http.sh # 5. Streamable HTTP
│   ├── regression-test-db-wtm.sh          # 6. DB W/T/M 三类横展 (P5 升版: 12 fixture basic)
│   ├── regression-test-db-wtm-100.sh      # 6b. DB W/T/M 100% 表覆蓋 (P5 升版: 100 表 + 9 段走查 + 派生守門 10 条)
│   ├── regression-test-five-domain.sh     # 7. 5 域业务 (player/economy/match/social/admin)
│   ├── regression-test-openclaw.sh        # 8. OpenClaw v1 既有 fixture
│   └── run-all.sh                         # 9. 一键跑全部
├── mock_data/                             # 测试数据
│   ├── openclaw/                          # OpenClaw v1 端点 (20 份, 9/1 迁移自 docs/reports/wiremock-openclaw/)
│   ├── langgraph/                         # Star-LG LangGraph 統合架构
│   │   ├── tmo/                           #   TMO 7 节点 (M-N1..M-N7) - 21 份
│   │   ├── sa-10/                         #   SA-10 task-orchestrator - 6 份
│   │   └── sa-01..09/                     #   9 SA 类型 (SA-01..SA-09) - 27 份
│   ├── agent-runtime/                     # STAR Agent Runtime (L0/L1/L2)
│   │   ├── l0-dispatcher/                 #   L0 派发 (Tokio + SQLite) - 6 份
│   │   ├── l1-ecs/                        #   L1 ECS (bevy_ecs / flecs) - 9 Archetype - 9 份
│   │   └── l2-pools/                      #   L2 业务共享池 (LLM/MCP/HTTP/Tool/RAG/Token/Rate/CB) - 8 份
│   ├── mcp/                               # 16 MCP tool 端点 - 48 份
│   ├── streamable-http/                   # Streamable HTTP (session 重连/Server-push/Last-Event-ID/DELETE) - 8 份
│   └── db-wtm/                            # DB W/T/M 三类横展 (per 守门 #13) - 100 表 100% 覆蓋
│       ├── work/                          #   Work (短 TTL 物理删, retention 显式) - 16 份 (P5 升版: 14 表 + 2 domain-specific)
│       ├── transaction/                   #   Transaction (append-only, audit, RLS 13 類必携) - 49 份 (P5 升版: 47 表 + 2 domain-specific)
│       └── master/                        #   Master (SCD Type 2, RLS 13 類必携, 不物理删) - 45 份 (P5 升版: 33 表 + 12 domain-specific)
├── docs/                                  # 回归测试报告
│   ├── regression-report-2026-09-05.md
│   └── README.md
└── k3s/                                   # k3s 部署 yaml (envoy 独立 deployment 模式 per 9/1 13:03+13:05 JST 偏好)
    ├── envoy-deployment.yaml              # envoy 独立 deployment + ClusterIP
    └── star-mock-service.yaml             # star-mock ClusterIP service
```

**总 fixture 估算**: 20 (openclaw) + 21 (tmo) + 6 (sa-10) + 27 (sa-01..09) + 23 (agent-runtime) + 48 (mcp) + 8 (streamable-http) + 110 (db-wtm, P5 升版) = **263 份 mock fixture** (含 100 表 W/T/M 100% 覆蓋) + 10 份回归脚本 (P5 升版: +1 db-wtm-100) + 4 份 k3s yaml + 3 份 docs (P5 升版: +1 W-T-M-100-COVERAGE-REPORT)

## 2. mock_data fixture 命名规则 (Naming Convention)

per 守门 #11 缺标比错标:

```
<version>--<module>--<sub-module>--<method>--<scenario>.json
```

例:
- `v1--tmo--m-n1-merge--POST-merge-2-tasks.json` (TMO M-N1 merge 正常 case)
- `v1--tmo--m-n1-merge--POST-merge-cyclic-dep.json` (TMO M-N1 merge cycle 异常 case)
- `v1--db--wtm--work--session-cache--GET.json` (DB Work 类 session_cache 端点)
- `v1--db--wtm--transaction--audit-event--POST.json` (DB Transaction 类 audit_event POST)

## 3. 守门 (per AGENTS.md §4)

| 守门 | 应用 | fixture 体现 |
|---|---|---|
| **#1** R-05 推 origin (反转 9/3 已落地) | N/A | — |
| **#3** 5 域独立 Lead (不映射 DDD) | 5 域业务 fixture 走历史治理命名 (player/economy/match/social/admin) | mock_data/{openclaw,langgraph}/five-domain/ |
| **#4** AI 协作 token-OLU | fixture 大小受限 (per fixture ≤ 5KB JSON) | 单 fixture < 100 rows |
| **#5** 环境变量安全 | 无 secret in fixture; 凭据走 env (per 守门 #5 硬 ban) | fixture 引用 `$env:VAR` 占位 |
| **#7** 0 unsafe | fixture JSON 严格 schema (zod-style 验证) | 每 fixture 头 5 行含 schema 注释 |
| **#9** 子代理 status="succeeded" ≠ 实际成功 | 回归脚本基于 `git log --follow` 实证 | scripts/run-all.sh 收尾跑 git 实证 |
| **#10** 代签规则应用 | scripts/ header 写 author=Ulysses | scripts/ 文件头 5 行 author 注释 |
| **#11** 缺标比错标安全 | fixture 含 "missing" / "edge" / "error" 子集 | 每个 module 必含 edge-case fixture |
| **#12** AI 协作文档治理 | README 实证每 fixture 来源 (commit / ADR) | fixture 头 commit 引用 |
| **#13** DB 三類横展開 (W/T/M) | mock_data/db-wtm/{work,transaction,master}/ 100% 覆盖 | 4 表 W + 4 表 T + 4 表 M = 12 fixture |
| **#14** 5 域 Lead CONTENT 4 维 | 5 域 fixture 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) | 5 域 fixture header 含 4 维 |

## 4. 已知缺口 (per 守门 #11 缺标比错标)

- **缺口 #1**: 16 MCP tool 全 fixture 仅含 6 tool (workitem_list / workitem_create / tools_invoke / agents_list / sessions_create / billing_usage), 剩 10 tool (audit_event / scm / workspace / feedback / inbox / project / permission / kms / form / search) 等 P3-B 实装
- **缺口 #2**: Agent Runtime L1 ECS 9 Archetype 仅 2 fixture (SA-01 + lifecycle), 缺 SA-02..SA-09 + System 12 类 + Component 详细
- **缺口 #3**: Streamable HTTP 仅 4 fixture (session-create/reconnect/server-push/delete-session), 缺 5xx 错误 + retry 完整 case
- **缺口 #4**: 5 域 fixture 仅 0 份独立 (复用 frontend/src/mocks/data/five-domain.ts per test-design v0.6 §17.4)
- **缺口 #5** ✅ P5 升版闭合: DB W/T/M 12 fixture → **110 fixture 100% 覆蓋 100 表** (per regression-test-db-wtm-100.sh 9/9 段 PASS + 派生守門 10/10)
- **缺口 #6**: k3s/ 2 yaml 缺 ConfigMap + Secret (envoy + 服务配置)
- **缺口 #7**: docs/ 2 份回归报告 (P5 升版: +1 W-T-M-100-COVERAGE-REPORT), 缺每次跑出的 commit-time 报告
- **缺口 #8** (P5 升版新增): frontend TS Schema 同步 (Zustand store / MSW mock 状态分类), 等 P3-B 拍板
- **缺口 #9** (P5 升版新增): V2 候補フィールド 暫定 T (symbol_index_snapshot / forgejo provider / Squad V2), V2 化时降格 W
| **缺口 #10** (P5 升版新增): 19 Module 混在 W/T/M 運用設計での TTL 差異明示 (各 fixture retention_period 已显式, 监控 + 削除ジョブ落地待 v0.3)

## 5. ACI 接口 (LLM-可读断言契约, per ULYS-191 §4.1.2 brief v0.1)

### 5.1 目的

让 mock 项目的所有 fixture 输出**提示词型断言** (per ULYS-191 §1), 方便 LLM agent
(Claude Code / Codex / 后续 agent) 一行指令读懂 mock 结果 (PASS/FAIL + 为什么 + 怎么修)。

对比传统 boolean assertion vs ACI assertion:

| 维度 | 传统 (`fixture_assertion`) | ACI (`aci_assertion`) |
|---|---|---|
| 结构 | `{guard_check_pass: true, elapsed_under_10ms: true}` | 10-13 字段 dict (assertion_id / expect / actual / reasoning / suggested_fix / ...) |
| 严重程度 | 0 (无) | 5 档 (`critical/high/medium/low/info`) |
| 修复建议 | 0 | `suggested_fix` 字段 (自然语言, LLM 可直接采纳) |
| 时间维度 | 0 | `captured_at` RFC3339 (LLM 判读趋势) |
| tags | 0 | 可选 (perf/kms/concurrency/...) 帮 LLM 分类 |

### 5.2 顶层配置 `.aci.json`

`.aci.json` 是 mock 项目根的 schema 契约声明文件 (per ACI schema v0.1 §3.1):
- `aci_version`: schema 版本 (LLM 看到 v0.2 知道字段含义可能已扩展)
- `supported_layers`: `["ut", "it", "st", "e2e"]`
- `severity_levels`: `["critical", "high", "medium", "low", "info"]`
- `assertion_statuses`: `["PASS", "FAIL", "WARN", "SKIP"]`
- `expect_value_types`: 17 种 (response_within_ms / field_equals / workflow_completes / ...)
- `schema_required_fields`: 10 个必填字段

### 5.3 Fixture 字段 `aci_assertion` (并存扩展, 不删 fixture_assertion)

每个 fixture 文件加一个 `aci_assertion` dict, 字段示例 (per `.aci.json` schema):

```json
{
  "aci_assertion": {
    "assertion_id": "star-flash-mock:guards:g-1",
    "aci_version": "0.1.0-draft",
    "layer": "ut",
    "scope": {
      "project": "star-flash-mock",
      "module": "guards",
      "domain": "admin",
      "operation": "task_queue_check",
      "http_method": "GET"
    },
    "expect": {
      "type": "response_within_ms",
      "value": 10,
      "description": "task_queue_no_persistence_gap 检查应在 10ms 内返回"
    },
    "actual": {
      "type": "response_within_ms",
      "value": 5,
      "description": "实测 5ms 完成, 快于预期 1 倍"
    },
    "status": "PASS",
    "severity": "info",
    "reasoning": "actual 5ms 远低于 expect 10ms 阈值 (50% 余量), guard G-1 通过",
    "tags": ["perf", "l0", "scheduler", "smoke"],
    "captured_at": "2026-09-23T10:00:00Z"
  }
}
```

**字段语义**:
- `expect`/`actual`: 人类可读 (`{type, value, description}`), LLM 必读
- `status`: PASS/FAIL/WARN/SKIP
- `severity`: critical/high/medium/low/info (LLM 优先处理 critical)
- `reasoning`: FAIL 时必填, 「为什么 fail」自然语言
- `suggested_fix`: FAIL 时建议填, 「怎么修」自然语言
- `captured_at`: RFC3339 时间戳

### 5.4 emitter helper (`scripts/_lib_aci_emit.py`)

Python emitter (`AciEmitter` class) + bash wrapper (`_lib_aci_emit.sh`)。

**Python 用法**:

```python
from pathlib import Path
from _lib_aci_emit import AciEmitter

em = AciEmitter(layer="it")
a = em.build(
    assertion_id="star-flash-mock:guards:g-1",
    scope={"project": "star-flash-mock", "module": "guards", "domain": "admin"},
    expect={"type": "response_within_ms", "value": 2000,
            "description": "API should respond within 2s"},
    actual={"type": "response_within_ms", "value": 5,
            "description": "Measured 5ms response"},
    status="PASS",
    severity="info",
    reasoning="actual 5ms << expect 2000ms threshold, by 400x margin",
    tags=["perf", "smoke"],
)
em.write(a, Path("/tmp/out.aci.json"))
```

**CLI 用法**:

```bash
python3 scripts/_lib_aci_emit.py emit \
  --layer it \
  --assertion-id "star-flash-mock:guards:g-1" \
  --scope project=star-flash-mock module=guards domain=admin \
  --expect-type response_within_ms --expect-value 2000 --expect-description "..." \
  --actual-type response_within_ms --actual-value 5 --actual-description "..." \
  --status PASS --severity info --reasoning "..." \
  --tags smoke,perf \
  --output /tmp/out.aci.json
```

### 5.5 聚合 CLI (`tools/aci-summary/aci_summary.py`)

对目录递归收集 `aci_assertion`, 聚合 critical/high/medium 统计 + Top N issues:

```bash
python3 tools/aci-summary/aci_summary.py tools/star-flash-mock/mock_data/agent-runtime/guards/ --top 5
# 输出 critical/high/medium 统计 + Top 5 issues (severity-sorted)
```

LLM agent 主用例: 一行指令获取 mock 结果概览, 不必逐个 fixture 解析。

### 5.6 Stage 1 落地状态 (per brief v0.1)

- ✅ `.aci.json` (本项目根)
- ✅ `_lib_aci_emit.py` (Python emitter, AciEmitter class)
- ✅ `_lib_aci_emit.sh` (bash wrapper)
- ✅ 5 sample fixture 加 `aci_assertion` 字段 (G-1/3/5/7/9, 都 PASS)
- ✅ `aci_summary.py` (CLI 雏形, MVP 阶段)
- ⏳ Stage 2: 全量 175 fixture 加 `aci_assertion` (per G-ACI-06 b/c, 第 2 笔 brief)
- ⏳ Stage 3: IDE1.0 / RustGameServer / 4 个项目适配 (per §4.2 + §4.3)
- ⏳ Stage 4: 跨项目统一验证 (per §4.5)

**完整设计稿**: `docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md` v0.2
**Brief v0.1**: `docs/briefs/ulys-191-star-mock-aci-stage1.md`

## 6. Mock Cluster Switch (per ULYS-190 §4.1 brief v0.1)

### 6.1 目的

ULYS-190 落地阶段 1: 给 Star mock 加 **L1 cluster_switch** 雛形, 控制整個 cluster 啟用 / 關閉 / 模式 (offline / passthrough / proxy)。
配合 ULYS-191 ACI emit 帶 `mock_switch_trace` 字段 (per §5.3 + design-analysis §3.4), LLM 一眼看出「是 mock 模擬的 X 功能 vs 真實的 Y 服務」失敗。

### 6.2 顶层配置 `.mock-cluster.json`

```json
{
  "$schema": "https://ulysses-star.local/schemas/mock-cluster/v0.1.0-draft.json",
  "cluster_version": "0.1.0-draft",
  "project": "star-flash-mock",
  "cluster_id": "star-flash-mock--dev-cluster",
  "enabled": true,
  "mode": "offline",
  "fallback_to_real": false,
  "aci_compat_version": "0.1.0-draft",
  "supported_layers": ["ut", "it", "st", "e2e"],
  "tags_taxonomy": ["perf", "kms", "rbac", "auth", "concurrency", "regression"]
}
```

**字段說明**:

| 字段 | 必填 | 含義 |
|---|---|---|
| `cluster_version` | ✅ | schema 版本 (semver, 跟 `.aci.json` `aci_version` 對齊) |
| `project` | ✅ | mock 項目名 (跟 `.aci.json` `project` 字段一致) |
| `cluster_id` | ✅ | cluster 全局唯一 ID (e.g. `<project>--<env>`) |
| `enabled` | ✅ | bool, 整個 cluster 啟用 / 關閉; false = 全部 fixture 不 dispatch |
| `mode` | ✅ | `offline` / `passthrough` / `proxy` (per §6.3 模式說明) |
| `fallback_to_real` | 🟡 | bool, mock 失敗時是否 fallback 真實服務 (預設 false, mock 失敗暴露給測試) |
| `aci_compat_version` | ✅ | 對齊 `.aci.json` `aci_version` (e.g. `"0.1.0-draft"`); 不一致則 dispatcher 報錯 |

### 6.3 三種 mode 說明

| mode | 含義 | LLM 看到的 mock_switch_trace |
|---|---|---|
| `offline` | mock 完全本地模擬, 不連真實服務 | `cluster.mode=offline` |
| `passthrough` | mock 轉發到真實服務, 只 mock 部分欄位 | `cluster.mode=passthrough` |
| `proxy` | mock 作為 reverse proxy, mock 記錄所有進 / 出 (trace 用) | `cluster.mode=proxy` |

### 6.4 reader helper (`scripts/_lib_mock_switch.py`)

```bash
# 1. is-enabled: exit 0=enabled, 1=disabled
python3 scripts/_lib_mock_switch.py is-enabled --cluster-config .mock-cluster.json
# 期望: CLUSTER_ENABLED=true (exit 0)

# 2. get-mode: print cluster mode
python3 scripts/_lib_mock_switch.py get-mode --cluster-config .mock-cluster.json
# 期望: CLUSTER_MODE=offline (exit 0)

# 3. trace: print mock_switch_trace 字符串 (per §5.3 ACI emit 用)
python3 scripts/_lib_mock_switch.py trace --cluster-config .mock-cluster.json
# 期望: cluster.enabled=true, cluster.mode=offline, cluster.aci_compat_version=0.1.0-draft, [TBD plugins section 落地後擴充]

# 4. validate-compat: 校验 .aci.json aci_version 跟 .mock-cluster.json aci_compat_version 一致
python3 scripts/_lib_mock_switch.py validate-compat \
    --cluster-config .mock-cluster.json --aci-config .aci.json
# 期望: ACI_COMPAT=OK (cluster=0.1.0-draft) (exit 0); 不一致 exit 3
```

### 6.5 跟 ACI emit 拼接 (per §5.4 `_lib_aci_emit.py`)

`_lib_aci_emit.py` 加 `mock_switch_trace` 字段 (per ULYS-190 §3.4 + brief v0.1 §1.1 item 5):

```bash
# 推荐: 从 _lib_mock_switch.py 拼接 trace, 再传入 emit
TRACE=$(python3 scripts/_lib_mock_switch.py trace --cluster-config .mock-cluster.json)
python3 scripts/_lib_aci_emit.py emit \
    --assertion-id "star-flash-mock:test:g-99" \
    --layer it \
    --scope project=star-flash-mock,module=guards,domain=admin \
    --expect-type response_within_ms --expect-value 2000 --expect-description "expect 2s" \
    --actual-type response_within_ms --actual-value 5 --actual-description "actual 5ms" \
    --status PASS --severity info \
    --reasoning "test emit with mock_switch_trace" \
    --mock-switch-trace "$TRACE" \
    --output /tmp/test.aci.json
```

### 6.6 Stage 1 落地状态 (per brief v0.1)

- ✅ `.mock-cluster.json` (本项目根, L1 cluster_switch 契约)
- ✅ `_lib_mock_switch.py` (Python reader, MockSwitchReader class + 4 CLI 子命令)
- ✅ `.aci.json` 加 `aci_compat_version` 字段 (per brief §1.1 item 3)
- ✅ G-1 sample fixture 加 `mock_switch_trace` 字段 (per brief §1.1 item 4)
- ✅ `_lib_aci_emit.py` 加 `mock_switch_trace` 参数 + `--mock-switch-trace` CLI flag (per brief §1.1 item 5)
- ⏳ Stage 2: 全量 5 sample fixture 加 `mock_switch_trace` (per §4.4 + G-MS-04 跨項目 plugin_id 命名, 第 2 笔 brief)
- ⏳ Stage 3: L2 plugin_switch + L3 module_switch (per design-analysis §4.2-§4.4, 6 项目 brief)
- ⏳ Stage 4: 跨项目 CI 加 `mock-switch-validate` (per design-analysis §4.5)

**完整设计稿**: `docs/architecture/2026-09-22-mock-switches/00-design-analysis.md` v0.2
**Brief v0.1**: `docs/briefs/ulys-190-star-mock-cluster-switch-stage1.md`

## 7. 跨项目引用 (per 守门 #12 + AGENTS.md §5 仓库拓扑)

- **不**引用 RGS 仓 (`D:\RustGameServer\tools\rgs-flash-mock`): 仅治理结构镜像, fixture 不双向同步
- **不**引用 RGS 5 域 Lead 真人: Star 仓 5 域 Lead 临时代签 per AGENTS.md §4 #3 反转
- **不**建立业务子域↔DDD bounded context 映射: fixture 用 module 维度 (per 守门 #3)

## 8. 修订历史 (per 守门 #12)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 脚手架 (scripts/ + mock_data/ + docs/ + k3s/) + 165 份 fixture 估算 + 9 份回归脚本 + 7 份守门落档; 迁移 docs/reports/wiremock-openclaw 20 份 → mock_data/openclaw/ | 2026-09-05 06:50 JST user 拍板 (单文件 v0.6 → v0.7 + 新建 tools/star-flash-mock/ + 全栈覆盖) |
| v0.2 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | P5 升版: 110 fixture 落地 (45 M + 49 T + 16 W) 100% 覆蓋 100 表; +98 fixture 透过 _generate_100_fixtures.py 可再生; +regression-test-db-wtm-100.sh 9 段走查 PASS; +W-T-M-100-COVERAGE-REPORT.md v0.1; 派生守門 10 条 CW-01~CW-10 全部 PASS; 守门 #5/#11/#12/#13 a/b/c/d 0 违反 | 2026-09-05 06:50 JST user 拍板 "推进" + P5 DB W/T/M 100% 表覆蓋 (推荐) |
| v0.3 | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | ULYS-190 §4.1 brief Stage 1: +`.mock-cluster.json` (L1 cluster_switch 契约) +`_lib_mock_switch.py` (MockSwitchReader class + 4 CLI 子命令: is-enabled/get-mode/trace/validate-compat) +`.aci.json` 加 `aci_compat_version` 字段 +`mock_data/agent-runtime/guards/v1--guard--g-1.json` 加 `mock_switch_trace` 字段 +`_lib_aci_emit.py` 加 `mock_switch_trace` 参数 + `--mock-switch-trace` CLI flag + README §6 Mock Cluster Switch (6.1 目的 / 6.2 .mock-cluster.json / 6.3 mode / 6.4 reader helper / 6.5 ACI emit 拼接 / 6.6 Stage 1 状态); regression-test-system.sh 13/13 PASS + regression-test-agent-runtime.sh PASS + validate-compat OK; 守门 #1+#5+#7+#10+#11+#13+#14v4+#15+#19v19+#20+#24 0 违反 | 2026-09-23 23:02 JST user 拍板「推进」(per reply `01a0d081`) T1 派工触發 + brief v0.1 (per commit `0f199757` ULYS-190 v0.2 approved) |
| v0.4 | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | ULYS-190 §4.5 mock-switch-validate + §4.6 G-MS-01 Ada 盘點落地: +`tools/mock-switch-validate.py` (244 LOC, 3 CLI 子命令 validate-one/validate-all/report + DEFAULT_PROJECT_ROOTS 7 项目) +design-analysis v0.3 (G-MS-01 ✅ 解決 + Ada 降級模式 L1 only) +`docs/briefs/ulys-190-g-ms-01-ada-inventory.md` (113 行 G-MS-01 盤點落地 brief, Ada testkit scaffold 屬性確认) +`docs/briefs/ulys-190-mock-switch-validate.md` (117 行 §4.5 跨项目 CI mock-switch-validate brief) +`docs/regression-report-stage2-mock-switch-validate-2026-09-23.md` (97 行 Stage 2 落地报告, 5 验收脚本全过); regression-test-system.sh PASSED; 守门 #1+#5+#6+#7+#9+#10+#11+#12+#13+#14v4+#15+#19v19+#20+#24 0 违反 | 2026-09-23 23:02 JST user 拍板「完成所有后续工作」(per reply `01a0d0e1`) T1 派工触發 |
| v0.5 | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | ULYS-190 §4.2/§4.3/§4.6 跨项目 cluster_switch + plugin_switch 落地: IM1.0 (im-testkit 5 plugin: assertions/fixtures/mock_grpc/mock_rest/mock_ws_frames) + RGS (rgs-flash-mock 5 plugin: player/economy/match/social/admin) + CATs (cats-mock 4 plugin: data/db/http/infra) + IDE1.0 (2 plugin: ide-cli/ide-kernel-core) + GitGit (gm-console MSW 3 plugin: health/repo/vault) + Ada (testkit scaffold 降級模式 L1 only, enabled=false 預設) — 6 commits 跨 6 repos (IM1.0 d637348 / RGS cdbc17d / CATs 97c7a17 / IDE1.0 c7621cc / GitGit 2ea... / Ada 9e901a8); +`.github/workflows/mock-switch-validate.yml` 跨项目 CI 校验 workflow (per §4.5 followup); 13 守门合规全綠 | 2026-09-23 23:02 JST user 拍板「推进到完成」(per reply `01a0d306`) T1 派工触發 |
