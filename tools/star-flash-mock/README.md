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
- **缺口 #10** (P5 升版新增): 19 Module 混在 W/T/M 運用設計での TTL 差異明示 (各 fixture retention_period 已显式, 监控 + 削除ジョブ落地待 v0.3)

## 5. 跨项目引用 (per 守门 #12 + AGENTS.md §5 仓库拓扑)

- **不**引用 RGS 仓 (`D:\RustGameServer\tools\rgs-flash-mock`): 仅治理结构镜像, fixture 不双向同步
- **不**引用 RGS 5 域 Lead 真人: Star 仓 5 域 Lead 临时代签 per AGENTS.md §4 #3 反转
- **不**建立业务子域↔DDD bounded context 映射: fixture 用 module 维度 (per 守门 #3)

## 6. 修订历史 (per 守门 #12)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 脚手架 (scripts/ + mock_data/ + docs/ + k3s/) + 165 份 fixture 估算 + 9 份回归脚本 + 7 份守门落档; 迁移 docs/reports/wiremock-openclaw 20 份 → mock_data/openclaw/ | 2026-09-05 06:50 JST user 拍板 (单文件 v0.6 → v0.7 + 新建 tools/star-flash-mock/ + 全栈覆盖) |
| v0.2 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | P5 升版: 110 fixture 落地 (45 M + 49 T + 16 W) 100% 覆蓋 100 表; +98 fixture 透过 _generate_100_fixtures.py 可再生; +regression-test-db-wtm-100.sh 9 段走查 PASS; +W-T-M-100-COVERAGE-REPORT.md v0.1; 派生守門 10 条 CW-01~CW-10 全部 PASS; 守门 #5/#11/#12/#13 a/b/c/d 0 违反 | 2026-09-05 06:50 JST user 拍板 "推进" + P5 DB W/T/M 100% 表覆蓋 (推荐) |
