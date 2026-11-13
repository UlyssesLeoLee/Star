# PHASE-STAR-STABLE-W3-WBS-CONSUMER-SYNC-20261103

- 日期：2026-11-03
- Lane：`codex/star-stable-w3-wbs-sync-20261103`
- 基线：本地 `dev@fc08efa38c5738e24c410ca88c1a730b3dc8ef76`
- 状态：WBS 仅同步已集成的本机 consumer verifier 证据；W3 仍 OPEN/BLOCKED，v1.0.6 仍 NO-GO。

## §0 目的与范围

将已集成到本地 dev 的 W3 consumer verifier 能力和验证边界同步到 Stable Release 主 WBS。仅修改 W3 行、追加一条 WBS 修订历史，以及新增本阶段报告；不修改其他 WBS 行、代码、review、brief、dispatch、main、远端或其他 lane。

## §1 改动矩阵

| 文件/范围 | 修改 | 结果 |
|---|---|---|
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` | 更新 W3 状态证据并追加 v1.49 历史；其余行与历史快照保留。 | 记录本地 verifier 的能力、提交、既有验证及实际 hosted 缺口；W3 OPEN/BLOCKED、v1.0.6 NO-GO。 |
| `docs/reports/PHASE-STAR-STABLE-W3-WBS-CONSUMER-SYNC-20261103.md` | 新增本阶段报告。 | 记录来源、变更和验证边界。 |
| 消费端实现 | 未修改；来源为 dev 中的 `4ed2dc2725de57213d5271ff853bd2cc909c7c11` 与 `75cc9b72e8fad3500d80176e22d97e5542246e90`。 | verifier 能检查 Desktop/Web 文件和固定 digest 的批准 Canvas OCI 镜像的发布身份/provenance 约束，并对 gh 子进程/输出及本地文件哈希变化 fail closed。 |

WBS 链接到已在 dev 的[consumer verifier 阶段报告](PHASE-STAR-STABLE-W3-CONSUMER-VERIFY-20261103.md)与[独立审查](../reviews/STAR-STABLE-W3-CONSUMER-VERIFY-REVIEW-20261103.md)。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 来源证据核对 | brief 指定的两个代码提交、consumer verifier 阶段报告与独立 review 均位于基线 `dev@fc08efa38c5738e24c410ca88c1a730b3dc8ef76` 的历史。 |
| verifier 单测 | 11 项本机模拟测试通过；这是既有阶段报告及协调者独立重跑记录，不是本 WBS 文档 lane 新运行的测试。 |
| `py_compile` | 通过；这是既有阶段报告及协调者独立重跑记录。 |
| consumer 实际运行 | 未验证真实 GitHub CLI、服务端 attestation 或 hosted v1.0.6 artifact。 |
| 制品链 | 未验证 hosted build、实际 SPDX BOM、attestation bundle、最终镜像/digest、安装、部署或回滚。 |
| 本 lane `rtk proxy git diff --cached --check` | 两份登记文件均通过；命令无错误输出，exit code 0。 |
| Cypher Guardian | 本 lane 仅修改文档，未运行。 |

## §3 已知缺口

1. 消费端单测使用模拟 CLI 进程，不能证明真实 GitHub CLI 版本、认证、网络、workflow signer 或 hosted attestation 可用。
2. 当前没有 v1.0.6 Desktop/Web/Canvas hosted 制品及其真实 SHA/digest、SPDX BOM、provenance/attestation bundle 和成功消费记录。
3. verifier 不生成 BOM，不归档 attestation bundle，也不证明同一 release ID 下全部 artifact、BOM、镜像与证明材料完整绑定。
4. 安装、部署、回滚和 Multipass/K3s 运行验收仍缺失。
5. 因上述缺口，W3 必须保持 OPEN/BLOCKED，v1.0.6 必须保持 NO-GO；不得将本地 mock 测试提升为 hosted 验收。

## §4 子代理失败接手清单

- 本文档 lane 未派发子代理；无失败子代理或遗失提交需要接手。
- consumer verifier 实现由先前独立 lane 完成并集成到基线；其阶段报告记录 11 项聚焦测试、Python 编译和代码 diff 检查结果，协调者独立复跑确认。
- 本 lane 不重写实现 lane 的 review，也不把未执行的 hosted 验收交由模拟测试替代。

## §5 守门规则

1. 本 lane 的写入范围只含 Stable Release WBS 与本阶段报告。
2. 不修改 brief、dispatch JSONL、协调者 review、代码、workflow 或其他 WBS 行。
3. 保留其他 lane/worktree 的 owner 与内容，不清理其分支、缓存或文件。
4. 以 `dev@fc08efa38c5738e24c410ca88c1a730b3dc8ef76` 为文档证据基线。
5. 只登记 brief 列出的 consumer verifier commits 与现有报告/review。
6. 11 项单测仅记为本机模拟 consumer 测试，不等同真实 GitHub CLI 验证。
7. 不声称本 lane 重新运行了来源阶段的测试或 guardian 检查。
8. verifier 的支持范围限定为 Desktop/Web 本地文件和 allowlist 中固定 digest 的 Canvas OCI 镜像。
9. signer workflow、仓库、tag、source commit/ref 和 predicate 约束由工具按制品类型执行。
10. gh 子进程必须有限时、有限输出并在错误时停止；不得把未受限缓冲描述为已验证。
11. 本地文件在验证后重新哈希；这不等于 hosted artifact 的完整供应链证明。
12. 不推断 SPDX BOM、attestation bundle 或最终制品已生成。
13. 不推断 hosted CI、release、安装、部署、回滚或 K3s 验收通过。
14. 不以历史快照代替当前线上/远端 release 状态。
15. W3 状态保持 OPEN/BLOCKED，除非真实 artifact、BOM、attestation 与独立消费验收完整闭合。
16. v1.0.6 保持 NO-GO；不创建 tag/release，不发布或部署。
17. 不修改 `main`、远端或 `D:\Star` 用户工作区内容。

## §6 签字栏

| 角色 | 审查记录 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）：WBS 同步与 verifier 的局部能力边界分开表述。 |
| SRE Lead | Mavis 接手审核：未把模拟消费测试扩写为运行态或 hosted 证据。 |
| 平台 | Mavis 接手审核：记录有限子进程/输出与本地文件复哈希能力；真实供应链仍待验证。 |
| 评审主持 | Mavis 接手审核：本次仅改两个登记文件；diff 检查后提交。 |
| PM / Release owner | Ulysses 为 accountable release owner；本同步不构成 W3 或 v1.0.6 发布批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发/证据 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W3 consumer verifier 的 WBS 同步报告；W3 与稳定发行状态保持开放。 | dev 基线中的 verifier commits、阶段报告与独立 review |
