# RGS 仓 gRPC 测试工具栈 决策报告 (per 2026-09-08 17:53 JST Ulysses "另起 gRPC 工具栈" 拍板)

> **触发**: 2026-09-08 17:51 JST Ulysses 反馈"这套 Playwright 接 RGS 仓 API 能测到位吗" + 17:53 JST 选 A "RGS 仓另起 gRPC 工具栈 (推荐)".
> **范围**: D:\RustGameServer (RGS 仓) gRPC API 测试工具栈选型 + Phase 1 Vertical Slice 测试计划.
> **守门 #11 缺标比错标**: Playwright 仅覆盖 v2 提示词 1/13 组件 (HTTP/HTML 层). RGS 仓 5 域 gRPC API 必须用 gRPC 工具栈.

## 1. 现状 (per 2026-09-08 17:53 JST 探)

RGS 仓 (`D:\RustGameServer`) 已有 25 个 crates, 跟 star 仓 (16 crates) 完全独立:
- `network-gateway` (9/8 14:59 JST 最新) — 推测是 gRPC gateway
- `player-service / economy-service / match-service / guild-service / admin-service` — 5 域 micro-service (per 8/21 9/3 9/5 跟 Ulysses 讨论的 5 域 Lead)
- `battle-service / activity-service / card-service / replay-service` — 游戏玩法
- `pvp-full-service / leaderboard-service` — PvP / 排行榜
- `cluster-ops / gm-backend / gm-extra-service` — GM 跟 ops
- `rgs-arc-olu / rgs-asset-download / rgs-certgen / rgs-hello` — 工具跟入口

RGS 仓 docs 完整 (00-序号 0-10, 8/22 9/4 9/7 多版), 8/30 守门 #5 约束 "Star 仓 + RGS 仓 完全独立" 仍生效.

## 2. 工具栈选型 (per 守门 #11 + v2 提示词 §9 §37)

| 工具 | 用途 | 优劣 |
|---|---|---|
| **grpcurl** | 命令行 gRPC client, 无需 codegen | ✅ 简单, 跟 v2 §9 gRPC 协议一致; ❌ 单调用, 不便集成 |
| **Postman** | GUI gRPC client | ✅ 可视化, 团队友好; ❌ 商业产品 (per 守门 #36 优先开源) |
| **grpc-js** | Node.js gRPC client, 跟 star frontend 同栈 | ✅ 跨项目栈一致 (Node.js); ❌ RGS 仓是 Rust, 跨语言 |
| **tonic + test crate** | Rust 集成测试 (gRPC client + service) | ✅ 同栈 (Rust), v2 §9 内置, 跑 `cargo test`; ❌ 需写 Rust test code |
| **ghz** | gRPC 压测 (跟 wrk 类似) | ✅ 跟 v2 §4 压测 1k/10k/100k 客户端要求一致; ❌ 仅压测, 不做功能测 |
| **Buf / buf curl** | 现代 gRPC client + protobuf lint | ✅ 现代 protoc 替代; 跟 v2 §28 API 版本一致 |

**推荐 (per 9/1 14:58 + 9/8 16:08 守门必带推荐)**: **tonic + test crate (Rust 集成测试) 为主, grpcurl 辅**, 跟 v2 提示词 §37 Phase 1 "Test 阶段" 一致.

## 3. Phase 1 Vertical Slice 测试计划 (per v2 §4 §37)

| 阶段 | 测试目标 | 工具 |
|---|---|---|
| Login | gRPC Login RPC (player-service) | tonic + test crate + grpcurl |
| Game Gateway | gRPC Game Gateway 鉴权 (network-gateway) | tonic + test crate |
| PlayerActor | tokio actor lifecycle + mailbox (player-service) | Rust #[tokio::test] |
| SceneActor | scene sharding (network-gateway) | Rust #[tokio::test] |
| Move | gRPC Move RPC (player-service) | tonic + test crate |
| Combat | gRPC Combat RPC (battle-service) | tonic + test crate |
| Inventory | gRPC Inventory RPC (economy-service) | tonic + test crate |
| PostgreSQL | DB ownership + OCC (per 守门 #13 W/T/M) | sqlx + test + migration |
| Logout | 重新登录恢复 | 端到端 tonic test |

## 4. 后续 session 任务 (per 9/8 15:29 自驱强化)

**本 session 不接 RGS 仓 gRPC 测试** (本 session focus 在 star frontend + 守门 v32-v39 落地, RGS 仓是独立仓). 跨 session 续做清单:
- RGS 仓新 session Mavis 接手: 探 `network-gateway` crate gRPC proto + 写 tonic test + grpcurl 验证
- per v2 §4 必先跑 1k 客户端压测 (ghz)
- 落 `docs/reports/PHASE-RGS-GRPC-TEST-001.md` v0.1

## 5. 守门 v40 候选 (新)

- RGS 仓 gRPC 测试必分两层: (a) 单元 / 集成测试 (tonic + test, Rust 同栈) + (b) 端到端 (ghz 压测, 跨语言 client). Playwright 不可用于 gRPC 测 (协议不匹配).
- 跨 session 续做必在 RGS 仓开新 worktree (per 8/30 09:08 守门 "Star 仓 + RGS 仓 完全独立").
- 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2).

## 6. 决策落档

- 工具栈: tonic + test crate (主) + grpcurl (辅) + ghz (压测)
- Phase 1 测试覆盖: 8 步 (Login → Logout) per v2 §4
- 跨 session 续做清单: 见 §4
- 8/30 守门独立约束: RGS 仓不开 star frontend / star 仓不开 gRPC
- 决策 ID: RGS-GRPC-TEST-001 (跨 session 引用)
