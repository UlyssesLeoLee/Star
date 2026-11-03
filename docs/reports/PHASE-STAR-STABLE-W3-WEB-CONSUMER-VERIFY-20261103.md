# PHASE-STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103

> **状态：本地 Web consumer gate 已实现，尚未在 GitHub-hosted workflow 运行。W3 仍 OPEN/BLOCKED；v1.0.6 仍 NO-GO。**
> **范围：** 仅修改 Star Web evidence workflow，添加对同一 workflow run 产物的独立下载、校验和 provenance/SBOM attestation consumer verification。该工作流生成 build evidence，不部署 Vercel。

## §0 目的

在 Web evidence workflow 的生产 job 之后增加独立 consumer job。它重新下载 exact run artifact，复验 tag/source identity、归档 SHA-256、SPDX SBOM 文件，并分别验证 SLSA provenance 与 SPDX 2.3 SBOM attestation。消费者 job 仅获 contents:read 和 attestations:read，不授予写权限或 OIDC token。

## §1 改动矩阵

| 文件 | 改动 | 结果 |
|---|---|---|
| .github/workflows/star-stable-web.yml | 新增 verify-consumer job；仅在 APPROVED 且版本 tag 条件下执行；重新解析 exact tag 与 source SHA；下载 exact artifact；校验归档 checksum/SBOM；使用 scripts/release/verify_attested_release.py 验证 SLSA provenance，并用 gh attestation verify --format json 检查 SPDX predicate；解析 JSON 并要求存在成功验证的 SPDX 2.3 attestation；写入明确的非部署摘要。 | 所有缺失文件、tag/source 不符、checksum 或 attestation 失败都会使 job 失败。 |
| .github/workflows/star-stable-web.yml Cypher header | 添加 verifier job、checkout/download actions、release identity resolver、SLSA verifier、GitHub CLI SPDX verifier、JSON predicate assertion、审批 gate、只读权限和 workflow token 的证据关系；更新 source_sha256。 | header 只描述此 workflow 中明确出现的调用及设置。 |
| docs/briefs/STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103.md | 使用本轮隔离 lane 的执行 brief。 | 已随 lane 一并提交。 |
| docs/reports/WBS-DISPATCH-STAR-STABLE-W3-WEB-CONSUMER-VERIFY-20261103.jsonl | 记录此 lane 的 dispatcher completion receipt。 | 可追溯 task id、base、分支与 brief 路径。 |

参考：GitHub 的 attest 文档说明 SPDX SBOM 使用 predicate URI https://spdx.dev/Document/v2.3；GitHub CLI verify 支持限制 source digest/ref、signer workflow 和 predicate type。见 https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations 与 https://github.com/cli/cli/blob/trunk/pkg/cmd/attestation/verify/verify.go。

## §2 验证摘要

- 手工审阅 workflow diff：consumer job 依赖成功的 build-and-attest job；在 checkout 后重新运行 canonical resolver 并比较 tag target 与 GITHUB_SHA；下载 artifact 名称由相同 resolver 输出构成。
- 归档和 .sha256 文件必须非空，sha256sum --check 必须成功，解压目录中的 SPDX 文件必须非空。
- 现有 verifier 将归档 SHA-256、repository、tag/source commit、签名 workflow 和 SLSA predicate 绑定。附加 gh attestation verify 调用使用 source digest、refs/tags/v*、指定 signer workflow 和 SPDX 2.3 predicate，并将 --format json 输出解析为数组；只接受包含 attestation 与 verificationResult.statement.predicateType=https://spdx.dev/Document/v2.3 的成功验证记录。GitHub CLI 文档说明该 JSON 数组每项包含已验证 bundle 和解析后的验证结果。
- 运行了 rtk proxy git diff --check，结果通过。
- 运行 cgg_guard.py --changed --stamp 与对该 YAML 路径的显式 --stamp。工具两次均报告 0 source files checked，因为它没有把此 workflow YAML 作为 source file；随后使用同一脚本的 source_digest 实现更新 freshness stamp。该 guard 结果不代表 YAML schema 或 GitHub workflow 语义验证。
- 读取 scripts/release/verify_attested_release.py --help 以确认当前 CLI 参数。没有运行产品测试、YAML 测试、GitHub-hosted workflow 或真实 attestation consumer 操作。

## §3 已知缺口

1. 此提交只实现验证流程；没有真实 v1.0.6 tag、Web evidence artifact、SLSA/SPDX attestations 或 consumer 成功记录。
2. Web workflow 仍不是 Vercel production deployment；没有 Vercel 目标环境部署、线上版本读取、production smoke test 或 rollback 证据。
3. Desktop 与 Canvas 的 consumer lanes 尚由协调者独立审阅与集成；其余 W0–W9 发布门均未闭合。
4. GH_TOKEN 访问能力和同 run artifact / attestation API 的实际 hosted 行为尚未运行核验。

## §4 子代理失败接手清单

无子代理失败、未提交代码或接手项。协调者对该 lane 做独立 diff review 与集成复核。

## §5 守门规则

1. 必须由用户在 workflow_dispatch 选择 APPROVED，且 ref 是版本 tag。
2. 消费者重新解析 canonical release identity，并要求 peeled tag target 与本次 GITHUB_SHA 一致。
3. 只下载由 tag、版本和 source SHA 组成的 exact artifact 名称。
4. 缺少归档、checksum 或 SPDX 文件时 fail closed；checksum 不符时 fail closed。
5. provenance 验证必须匹配 Star Web workflow signer、tag source ref、source commit 和 SLSA predicate。
6. SBOM 验证必须匹配同一归档 subject、source identity、Star Web signer 与 SPDX 2.3 predicate；CLI 退出码及机器解析后的 verified SPDX JSON 记录都必须通过。
7. consumer job 只有 contents:read 与 attestations:read，不执行部署、发布或写入操作。
8. summary 只声称 build-evidence verification，不声称已部署 Vercel。
9. W3 保持 OPEN/BLOCKED，稳定版 v1.0.6 保持 NO-GO，直到真实 hosted runs、制品、签名与消费者验证证据齐备。

## §6 签字栏

| 角色 | 审核者 | 结论 |
|---|---|---|
| 架构 | 架构师（Mavis 接手审核） | consumer gate 与当前 tag/source identity 合同一致；没有改变 Vercel 部署边界。 |
| SRE Lead | 架构师（Mavis 接手审核） | 权限只读、执行受限；hosted 可用性与时延尚未实测。 |
| 平台 | 架构师（Mavis 接手审核） | 复用已存在的 resolver、artifact action 与 attestation verifier。 |
| 评审主持 | 架构师（Mavis 接手审核） | 待协调者独立审阅 workflow 与权限声明后接受。 |
| PM / Release Owner | Codex/assistant is accountable release owner | 此 lane 不发布产品；全部稳定版门禁仍未通过。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Web 同 run artifact 独立 consumer job，验证 checksum、SLSA provenance 与 SPDX 2.3 SBOM attestation；无测试或 hosted workflow 证据，W3 保持 OPEN/BLOCKED。 |