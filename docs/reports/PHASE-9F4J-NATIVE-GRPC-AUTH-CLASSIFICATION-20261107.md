# PHASE-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION

> **状态：PARTIAL**。native gRPC 与应用适配器的授权错误分类已实现、验证并通过独立审查；基本设计/WBS 同步及本地 dev 集成待完成。  
> Lane：`codex/9f4j-native-auth-errors-20261107`；基线：`dev@8c505f36`；代码提交：`71704ff8`、`ecb4d482`。

## §0 目的

分离永久授权拒绝与可重试传输故障：授权拒绝必须进入现有 fail-closed 停止路径，其他传输类错误继续按既有退避策略处理；错误不得携带服务端详情、凭据或事件内容。

## §1 改动矩阵

| 路径 | 改动与状态 |
|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | `Unauthenticated`/`PermissionDenied` → `Unauthorized`；其他状态 → 不带服务端详情的 `Transport`；pull、origin verification、ACK 共用映射。已实现，提交 `71704ff8`。 |
| `crates/star-api-rest/src/bi_task_event_ingestion.rs` | `Route/Unauthorized` → 应用 `Unauthorized`；`Contract/Transport/Busy` → 应用 `Transport`；三个操作共用映射且有五变体测试。已实现，提交 `ecb4d482`；本轮全文件格式整理提交 5f3de0c。 |
| Brief、阶段报告、dispatch | Brief 已提交；本报告与 dispatch 记录本阶段审查、验证和限制，待提交。 |
| docs/requirements.md、docs/basic-design.md、docs/detailed-design/DD-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION-001.md、Group WBS | 尚未修改。MAGOS 对完整设计范围的登记返回指纹 8f2094a911d4；与保留候选重叠的路径为 requirements.md、basic-design.md、Group WBS，新增详细设计路径无重叠。候选 codex/star-rust-1991-baseline-20261105 及其 worktree/文件保持原样。 |

独立审查确认消费者对授权拒绝会标记 Blocked 并停止，其他错误仍进入既有重试路径；两份 Rust Cypher 头部没有删除既有 MERGE 关系。native 头部为满足校验器大小上限对既有 MERGE 行做了空白压缩，审查确认关系语义保留。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk cargo +1.99.1 test -p platform-messaging-runtime --features native-grpc-adapter --lib native_event_status_mapping_is_fail_closed -j 4` | PASS，1 passed |
| `rtk cargo +1.99.1 test -p star-api-rest --bin star-api-rest native_event_receive_errors_preserve_auth_and_retryable_transport -j 4` | PASS，1 passed。首次尝试遇到 LNK1104 无法打开共享 target exe；确认无目标进程后原命令重试成功，未清理共享缓存。 |
| `rtk cargo +1.99.1 check -p star-api-rest --all-targets -j 4` | PASS，exit 0；有仓库既有 warnings |
| 两个变更 Rust 文件的 `rtk rustfmt +1.99.1 --edition 2021 --check` | PASS；先整理了 adapter 文件既有 import 和断言格式 |
| `cgg_guard.py --changed --stamp` / `rtk git diff --check` | PASS；Guardian 检查 1 个源文件，无空白错误 |
| 组合代码独立审查 | PASS；无 P0/P1；审查者未执行测试 |
| PostgreSQL/RLS、真实 TLS/native RPC、ACK E2E | NOT RUN；无目标数据库和可验证部署环境 |
| Clippy | NOT RUN |

## §3 已知缺口

1. 单测和编译不能替代真实 TLS、服务端授权拒绝、PostgreSQL/RLS、commit-before-ACK 或恢复链路验收。
2. 需求、基本设计、详细设计及 Group WBS 尚未同步。MAGOS 拒绝扩展 manifest，精确指纹为 8f2094a911d4；重叠来源登记了 requirements.md、basic-design.md 与 Group WBS。新增的专属详细设计路径未发生重叠。等待用户授权后只补齐本阶段状态与分类契约。
3. 尚未快进到本地 `dev`；没有修改 `main` 或远端。
4. 应用层 LNK1104 首次失败记录在案；同命令重试通过，未删除共享缓存。

## §4 子代理失败接手清单

- Worker 提交 `71704ff8`，范围仅为原生状态映射与测试。
- 协调者提交 `ecb4d482`，修复 adapter 丢失 native 错误类别的问题。
- 曾用 `--lib` 过滤应用测试，结果为 0 项，未计作通过；随后用 binary target 精确测试并通过。
- 独立代码/规格审查无 P0/P1；reviewer 未运行测试。协调者仍需完成受阻文档同步、最终提交、dev freshness 复核与快进。

## §5 守门规则

1. 未认证或无权限必须停止消费者；不得按网络故障重试。
2. Contract、Transport、Busy 保持 transport 分类并服从既有有界重试。
3. 对外错误不得泄漏 tonic status details、metadata、凭据或事件负载。
4. pull、origin verification、ACK 使用一致的状态映射。
5. 代码级验证不宣称 PostgreSQL、TLS、网络、部署或生产验收通过。
6. 保留 W8/Dependabot lane、用户文件、`main`、远端及共享 Cargo 缓存。
7. 需求/基本设计/详细设计/WBS 同步须先完成 MAGOS manifest 复核；当前状态为 PARTIAL。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核：组合分类行为未发现 P0/P1 |
| SRE Lead | 定向测试与 all-targets 通过；真实运行环境未验证 |
| 平台 | adapter 分类审查通过；PostgreSQL/RLS 未运行 |
| 评审主持 | 范围与 Cypher 关系审查通过 |
| PM | PARTIAL；等待需求、基本设计、详细设计、WBS 登记及本地 dev 集成 |

**审批者**：架构师（Mavis 接手审核）；未作人工部署或生产验收声明。  
**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录错误分类实现、验证、独立审查、未完成设计同步及环境验收缺口 | 完成组合 lane 验证 |
