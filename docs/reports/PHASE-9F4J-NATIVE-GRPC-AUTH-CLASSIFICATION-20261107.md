# PHASE-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION

> **状态：PARTIAL**。native gRPC 与应用适配器的授权错误分类已实现、验证并通过独立审查，已选择性集成到本地 dev；基本设计/WBS 同步及数据库、TLS、端到端验收仍未完成。
> 来源 lane：`codex/9f4j-native-auth-errors-20261107`；源基线：`dev@8c505f36`；源提交：`71704ff8`、`ecb4d482`、`5f3de0c1`、`944ea273`、`a356f239`、`205de9da`。已重放到 `dev` 的对应提交：`b83dfd07`、`2911d103`、`0bfbbfa0`、`3cb1b983`、`383efffa`、`5859027b`。

## §0 目的

分离永久授权拒绝与可重试传输故障：授权拒绝必须进入现有 fail-closed 停止路径，其他传输类错误继续按既有退避策略处理；错误不得携带服务端详情、凭据或事件内容。

## §1 改动矩阵

| 路径 | 改动与状态 |
|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | `Unauthenticated`/`PermissionDenied` → `Unauthorized`；其他状态 → 不带服务端详情的 `Transport`；pull、origin verification、ACK 共用映射。已实现，提交 `71704ff8`。 |
| `crates/star-api-rest/src/bi_task_event_ingestion.rs` | `Route/Unauthorized` → 应用 `Unauthorized`；`Contract/Transport/Busy` → 应用 `Transport`；三个操作共用映射且有五变体测试。已实现，提交 `ecb4d482`；本轮全文件格式整理提交 5f3de0c。 |
| Brief、阶段报告、dispatch | 已提交并集成到本地 `dev`；最终状态仍为 PARTIAL。 |
| docs/requirements.md、docs/basic-design.md、docs/detailed-design/DD-9F4J-NATIVE-GRPC-AUTH-CLASSIFICATION-001.md、Group WBS | 尚未修改。集成前的完整设计范围检查曾返回指纹 `8f2094a911d4`，当时登记路径与保留候选重叠；此后 dev 已前进，该指纹已失效。本轮尚未对新基线重新规划完整设计清单。保留候选 `codex/star-rust-1991-baseline-20261105` 及其 worktree/文件原样。 |

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
2. 需求、基本设计、详细设计及 Group WBS 尚未同步。代码集成前对完整设计范围的 MAGOS 检查返回过指纹 `8f2094a911d4`；由于 dev 已前进，该指纹已失效。当前基线下的完整设计范围尚未重新规划或修改；此前检测到的重叠是 requirements.md、basic-design.md 与 Group WBS，新增的专属详细设计路径当时无重叠。
3. 六个来源提交已逐项重放到本地 `dev`（提交映射见报告头部）；`main` 与远端均未修改或 push。重放没有重新运行 Rust 构建/测试；验证证据仍来自来源 lane 对相同补丁的实测。
4. 应用层 LNK1104 首次失败记录在案；同命令重试通过，未删除共享缓存。

## §4 子代理失败接手清单

- Worker 提交 `71704ff8`，范围仅为原生状态映射与测试。
- 协调者提交 `ecb4d482`，修复 adapter 丢失 native 错误类别的问题。
- 曾用 `--lib` 过滤应用测试，结果为 0 项，未计作通过；随后用 binary target 精确测试并通过。
- 独立代码/规格审查无 P0/P1；reviewer 未运行测试。协调者已复核目标 `dev` clean/inactive、MAGOS 登记范围无冲突，并完成六个提交的选择性重放；仍需完成受阻设计/WBS 同步和真实环境验收。

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
| PM | PARTIAL；本地 dev 集成完成，需求/基本设计/详细设计/WBS 同步及真实环境验收待办 |

**审批者**：架构师（Mavis 接手审核）；未作人工部署或生产验收声明。  
**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录错误分类实现、验证、独立审查、未完成设计同步及环境验收缺口 | 完成组合 lane 验证 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录源 lane 六个提交已选择性重放到本地 dev；更新当前设计门禁说明，不将旧指纹当作有效授权，也不把代码级检查提升为生产验收 | 代码 lane 已集成到 dev |
