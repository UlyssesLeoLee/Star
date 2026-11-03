# PHASE-STAR-STABLE-W3-CONSUMER-VERIFY-20261103

- 日期：2026-11-03
- Lane：`codex/star-stable-w3-consumer-verify-20261103`
- 基线：本地 `dev@fedc3dc12ec16235394a7c46d2603507231f0ee6`
- 状态：消费者验证工具通过本地实现门；W3 仍为 OPEN/BLOCKED，稳定版仍 NO-GO。

## §0 目的

为稳定发行候选提供一个标准库 Python 消费端工具，校验下载的 Desktop 文件、Web 证据归档或固定 digest 的 Canvas OCI 镜像是否有与 Star 仓库、稳定 tag、源 commit、批准的 workflow signer 及 SLSA provenance v1 相符的 GitHub artifact attestation。该工具补上 W3 的本机消费验证能力；它不生成制品、BOM、attestation，也不替代 hosted 发布门。

## §1 改动矩阵

| 范围 | 结果 | 证据 |
|---|---|---|
| `scripts/release/verify_attested_release.py` | 校验稳定 tag、仓库、40/64 位小写 commit、期望 SHA-256；Desktop/Web 本地文件流式哈希；Canvas 仅接收三种批准的 Star GHCR 镜像且必须使用不可变 `@sha256` digest。调用参数绑定 repo、`refs/tags/<tag>`、source digest、按制品类型固定的 workflow signer 与 SLSA v1 predicate；输出机器可读验证记录。 | Commit `4ed2dc27`、`75cc9b72` |
| 子进程与输出边界 | 使用参数数组与 `shell=False`；stderr 丢弃；限时等待、超限停止并回收 CLI 子进程；stdout 流式保留不超过 4 MiB（读块最多额外暂存 64 KiB）；本地文件在 CLI 验证后再次计算 SHA-256，变化时失败关闭。 | `_run_bounded_gh`；限流、超时及变更测试 |
| `scripts/release/tests/test_verify_attested_release.py` | 覆盖 Desktop/Web/Canvas 成功路径、摘要不匹配、非法身份、CLI 缺失/非零/超时、畸形或空 JSON、输出超限与验证中本地文件变化。 | 11 项聚焦测试 |
| 阶段文档与审查 | 记录实现范围、局部通过证据、未关闭的发行验收项与本轮审查修正。 | 本报告及配套 review |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy py -3 -m unittest scripts.release.tests.test_verify_attested_release -v` | 11 项通过（协调者独立重跑） |
| `rtk proxy py -3 -m py_compile scripts/release/verify_attested_release.py scripts/release/tests/test_verify_attested_release.py` | 通过（协调者独立重跑） |
| `rtk proxy git diff --check dev...HEAD` | 通过（协调者独立重跑；文档提交前代码范围） |
| CypherGraph Guardian | worker 报告 PASS，2 个源文件；本轮没有独立重复运行 |
| GitHub hosted attestation / v1.0.6 真实制品 | 未运行；无对应已发布候选制品，本工具测试使用模拟的 CLI 进程 |

## §3 已知缺口

1. 本地模拟测试不证明安装的 GitHub CLI 版本、凭据、网络、服务端 attestation 或实际 workflow 输出有效；真实消费仍需对最终 release artifact/image 执行本工具并保存输出。
2. 工具只校验调用者指定的单个制品及其 provenance 身份；不生成/校验 SPDX BOM，不下载或归档 attestation bundle，也不自动关联同一 release ID 下的 artifact、BOM、Canvas 三镜像、bundle 与验证记录。
3. 当前没有 v1.0.6 Desktop/Web/Canvas hosted build、最终 SHA/digest、签名/attestation、真实安装升级/回滚与部署证据。因此 W3 与稳定发行不得标记完成。
4. CLI 成功 JSON 只按官方文档规定的顶层记录结构（非空数组，记录含 `attestation` 和 `verificationResult` 对象）检查；不依赖未承诺的嵌套字段。批准约束由 CLI 参数执行。
5. 当前 WBS 主文件被 D2 活动 lane 持有，本 lane 未修改 WBS；集成后应在该 claim 释放时补写 W3 记录并保持 OPEN/BLOCKED。
6. 本地文件采用验证前后重新哈希的方式发现持久变化；该实现不是对同一用户并发恶意改写后恢复原字节的文件系统快照证明。

## §4 子代理失败接手清单

- Worker `/root/w3_consumer_verifier` 完成两份登记代码文件，先提交实现，再按协调者审查补充输出流限、文件复哈希及可审计身份字段。
- worker 报告 11 项测试、编译、diff 与 Cypher Guardian 结果；协调者独立重跑 11 项测试、Python 编译及代码 diff 检查，均通过。
- 本轮没有子代理失败或遗失的提交；没有将真实 hosted 验证或发布结果交由 mock 代替。

## §5 守门规则

1. W3 lane 只写登记的两个代码文件、brief、dispatch、阶段报告和 review。
2. 不修改 WBS 主表、发布 workflow、D2/W8 内容、`main`、远端或用户脏文件。
3. Python 标准库实现，不增加依赖。
4. 不能把 mock/单测或静态参数检查写成真实 hosted 验证。
5. 只接受 canonical stable `vMAJOR.MINOR.PATCH` tag。
6. source commit 必须是完整小写 40/64 位 Git object ID。
7. 文件和 OCI 期望 digest 必须为 `sha256:<64 lowercase hex>`。
8. Desktop/Web 对输入文件分块计算 SHA-256，并拒绝链接文件及摘要不符。
9. Web 归档文件名必须绑定所选 tag、版本及 source commit。
10. Canvas 镜像必须是 allowlist 中的 Star GHCR 镜像并固定 `@sha256`。
11. signer workflow 由制品类型在代码中固定，调用者不能自由选择 signer。
12. gh 命令使用 argv 参数数组，`shell=False`，stdin/stderr 不承载交互或日志内容。
13. gh 运行超时设上限；stdout 在读取阶段限流，超限即终止并回收进程。
14. 非零退出、CLI 缺失、超时、无效 JSON、空 attestations 和不完整顶层记录一律失败关闭。
15. 文件在验证结束后再次哈希；其内容发生变化则不返回成功记录。
16. 验证记录包含制品 digest、仓库、tag、source ref/commit、predicate、signer workflow 与 attestation 记录数。
17. W3/稳定版验收状态以真实 artifact、BOM、bundle、hosted workflow 与部署证据为准，当前仍 OPEN/BLOCKED / NO-GO。

## §6 签字栏

| 角色 | 审查记录 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）：局部消费验证边界已审查；不能替代发行 owner 验收。 |
| SRE Lead | Mavis 接手审核：超时、输出上限、失败关闭与资源回收边界已复核。 |
| 平台 | Mavis 接手审核：标准库依赖、跨平台 subprocess 参数与文件流式哈希已复核；真实 Windows/CI hosted 执行仍待验收。 |
| 评审主持 | Mavis 接手审核：先指出并推动修正输出完整缓冲问题，再复核修订实现与测试。 |
| PM / Release owner | Ulysses 为最终发行 accountable owner；本报告仅接受该实现 lane，不构成 v1.0.6 发布批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发/证据 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增消费者验证工具阶段报告；将实现与 W3 release gate 分开记录。 | `4ed2dc27`、`75cc9b72`；11 项测试及本地检查 |
