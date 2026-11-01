# PHASE-WBS-STAR-STABLE-UPDATE-20261101

**阶段状态：** WBS 快照与依赖证据已同步；稳定发行仍为 OPEN / 不可发布。
**工作基线：** dev@bde4e9d8e9dac0e25f7d983814d37fb27bc9313e（记录 W1/W2 证据快照时的目标基线，不表示后续 dev HEAD）。
**采用证据快照：** agent/star-stable-followup/integration@cdb66b5b4445aa57c7d1c60d3c5e6acf0f50a9fe2。记录该快照时，目标 dev@bde4e9d8e9dac0e25f7d983814d37fb27bc9313e 尚未包含此集成候选或本 WBS lane。

## §0 目的

将被接受的 W1 license metadata 与 W2 Schedule PostgreSQL fixture lane 结果写回 stable-release WBS，更新受影响的 Docker payload 证据，并按依赖顺序列出后续工作。此文档同步不关闭任何 W0–W9 门，不表示发行已就绪。

## §1 改动矩阵

| 文件/对象 | 变更 | 证据与边界 |
|---|---|---|
| docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md | 新增最新 accepted integration 快照；刷新 W1、W2、W7 状态；重排后续工作顺序；新增 v1.2 修订记录。 | W1 源提交 9083742a63306f310976b820faaa02a400a17ab8，纠正提交 ab7b0389ba68422725348bbe5811aff89efd1637，集成提交 393c98ce3b5562e64ef5e82e58dd46e09fe6c3f2。W2 源提交 eb1e91f47a3b6d19483990942114caa742407b9d，集成候选 cdb66b5b4445aa57c7d1c60d3c5e6acf0f50a9fe2。W1 remains BLOCKED，W2 remains OPEN/BLOCKED。 |
| W1 报告 PHASE-W1-MIT-METADATA-20261101.md | 13 个 Star 自有 Cargo package 的 license metadata 已改为 MIT；ACI emitter 保留独立项目的双许可，BFF 源文件 SPDX 仍为 MIT OR Apache-2.0。 | 报告记载 13 份 TOML 解析及 MIT 字段断言通过、git diff --check 通过。Rust 1.99.0 rustc 不可用，Cargo metadata 未验证；根 Cargo 声明、NOTICE、第三方权属/授权、SBOM 与实际 Desktop/Web/Vercel/Canvas 制品均未闭合。 |
| W2 报告 PHASE-W2-SCHEDULE-PG-FIXTURE-20261101.md | seed_directory 与 seed_task_binding 在构造 SQL 前要求具体远端 Branch UUID，保留原 fixture 约束。 | 报告记载 git diff --check 通过；Rust 1.99.0 all-targets check 未开始；1.99.1 诊断等待既有 Cargo 锁 60 秒后停止，没有编译结论；rustfmt 未安装；STAR_RUN_OWNER_TEST_DATABASE_URL 未设置，PG 集成未运行。 |
| W7 静态发行与部署输入 | 记录 Canvas 专用 build-context 静态保护，以及独立 local K3s Dockerfile 的未验证上下文边界。 | Canvas 发布 workflow allowlist 三个服务并生成筛选上下文，拒绝非 Canvas 本地依赖和含 game 的路径，配置 SBOM/provenance；三份 Canvas Dockerfile 只复制对应 crate 与生成的 workspace/lock/license。deploy/k3s-local/Dockerfile 仍 COPY crates；Cargo workspace 含 star-game，而 .dockerignore 排除 star-game。该本地上下文兼容性/排除效果未 build 验证。helm/star 为弃用占位 Chart，活动 Canvas manifest 保留 digest placeholder。 |
| main/dev 审计 | 未发现可安全摘取到 dev 的 main 独有内容。 | git cherry -v dev main 对 main 的五个提交全部标为 -（patch-equivalent）：62605314fc087cef212a017d6b2d2da76670cf24、94fdc837d19bbeb2d91b85042c06074b174fcb97、21416a7e727d57f878faefaf8ff39871c53e6ec4、08e883cf8be8433a2966a1ed1d8a15b60f5d7d5d、02e0cbd8f87a8422100d4fdd83d54e7838ab1fff。没有 cherry-pick；main 未修改。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| W1/W2 来源报告与 brief | 已从已接受的 integration@cdb66b5b 读取并核对阶段范围、验证结果与剩余缺口。 |
| main/dev 提交比较 | git cherry -v dev main：5 个 main-only commits 全部 patch-equivalent；git rev-list --left-right --count dev...main 为 278 5；未改写或修改 main。 |
| WBS 一致性 | PASS：检查确认 W0–W9 编号各出现一次且顺序正确，W1/W2 状态与报告一致，稳定版总状态保持 OPEN，未将任何未验证门标记为完成；v1.2 修订行唯一。 |
| rtk git diff --check | PASS：工作树与四文件暂存差异均通过 rtk git diff --check。 |
| 测试、Cargo、数据库或构建 | 本 WBS 文档 lane 未运行；仅记录依赖 lane 自己的报告，不把它们扩写成稳定版验收。 |

## §3 已知缺口

- W0 仍为 PARTIAL：发行范围明确，但 canonical version、source commit、release owner 和包版本映射未定。
- W1 仍 BLOCKED：根 Cargo workspace license 与用户所选 MIT 冲突尚未由当前 owner 解决；根 NOTICE 缺失；第三方授权/归属、完整依赖闭包和 SPDX/SBOM 尚未审查。13 个包的 metadata 修改不证明实际发行物的授权或 notice closure。
- W2 仍 OPEN/BLOCKED：Rust 1.99.0 编译门未开始，1.99.1 诊断没有编译结论，rustfmt 缺失，PostgreSQL 测试端点 unset。fixture 变更的静态格式检查不等同于编译或数据库验收。
- W3/W4 仍 OPEN/BLOCKED：真实 Rust SBOM、平台运行、artifact/image digest、签名与消费侧 attestation 验证及 hosted required-check 配置未完成。
- W5/W6 仍 OPEN：没有三平台 Desktop release、Vercel 部署消费验证、签名安装升级/回滚和真实制品证据。
- W7 仍 OPEN：Canvas allowlist/context 只有静态证据，没有 Buildx 镜像或 K3s 部署验证。deploy/k3s-local/Dockerfile 的宽泛 crates context 和 star-game workspace 成员排除兼容性未验证；不得据此声称所有 Docker 输入均为 Star-only。
- W8 仍 OPEN：最多两实例的 Multipass/K3s 创建、容量和恢复验收没有可复核记录。
- W9 仍 BLOCKED：依赖 W0–W8。没有创建 tag/release、构建或发布 artifact、部署或 push。

## §4 子代理失败接手清单

本 WBS lane 未派生新子代理，也未发生子代理失败。W1 后续接手点为当前 root Cargo/依赖 owner 解决 MIT metadata、补 root NOTICE 并完成各发布目标权属/notice/SBOM；W2 后续接手点为准备可用 Rust 1.99.0、rustfmt 与 PostgreSQL endpoint 后重新运行 MSRV 和 PG 门。Cargo lock 诊断命令已由 W2 lane 停止其自身等待中的进程；不触碰其他 Cargo owner。

## §5 守门规则

- 只改登记的 WBS、brief、phase report 与 dispatcher audit 四个文件。
- WBS 采用记录 W1/W2 证据快照时的 dev@bde4e9d8 基线和 coordinator accepted integration@cdb66b5b 的提交/报告证据；该历史快照不表示当前 dev HEAD，也不将候选 integration 描述为已进入 dev。
- W0–W9 均未关闭，所有静态或局部证据不替代发布构建、目标平台、许可证、签名、部署和 PostgreSQL 验收。
- 发行面仍是 Desktop、Web/Vercel、三份 Star Canvas 镜像；Multipass guest image 与开发工具不作为公开制品。
- Canvas 工作流的过滤上下文不外推到其他 Dockerfile；local K3s context 未验证前不宣称全仓 Docker payload 已排除 game/legacy 内容。
- 五个 main-only commits 均 patch-equivalent；不 cherry-pick、不修改 main、不 push、不创建 tag/release、不发布或部署。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis/协调者独立复核待完成 |
| SRE Lead | Mavis/协调者独立复核待完成 |
| 平台 | Mavis/协调者独立复核待完成 |
| 评审主持 | Mavis/协调者独立复核待完成 |
| PM | Mavis/协调者独立复核待完成 |

**修订人：** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
**发布审批：** 未申请；WBS 不可发布。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按已接受的 W1/W2 证据更新 stable-release WBS、W1/W2/W7 当前状态、Docker context 边界与后续依赖顺序；保持 W0–W9 未关闭。 | W1 metadata 与 W2 fixture lanes 已被接受，需要同步发布计划 |
