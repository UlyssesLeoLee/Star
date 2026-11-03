# PHASE — Star Stable W4 cargo-audit Asset SHA Repair

Date: 2026-11-03
Lane: `codex/star-stable-w4-audit-sha-20261103`
Baseline: `dev@234ea49eafa9f8a6acf466b32f134f9518aeebec`

## §0 目的与范围

修正 `.github/workflows/license-gate.yml` 中 cargo-audit v0.22.2 的官方归档名、下载 URL 与 SHA-256，使已核验资产能够通过现有完整性校验。RustSec 官方 release API 与直接下载摘要核对见[上游资产审阅](../reviews/WBS-STAR-STABLE-W4-UPSTREAM-ASSET-REVIEW-20261102.md)。

本 lane 不修改 Rust toolchain、其他 workflow、PR、main、远端或 WBS 主文件；不下载或执行归档内容。

## §1 改动矩阵

| 项 | 修改 | 结果 |
|---|---|---|
| SHA 配置 | `CARGO_AUDIT_SHA256` 设置为 `7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428` | 与 cargo-audit v0.22.2 官方 asset digest 一致 |
| 下载 asset | 路径使用 `cargo-audit-x86_64-unknown-linux-musl-v${CARGO_AUDIT_VERSION}.tgz` | 与官方 asset 名称一致 |
| 防护与证据 | 保留占位值拒绝、`set -euo pipefail` 和 `sha256sum -c -`；注释链接到上游核查报告 | 仍在摘要未配置或校验失败时停止 |
| WBS | 未修改 | WBS 主文件由协调者 lane 管理 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| PyYAML 解析及断言 | 通过：版本、摘要、asset 名、旧 URL 排除、fail-closed 检查和 checksum 命令均符合预期 |
| Bash 语法 | `rtk proxy bash -n -c <advisory-gate Fetch step>` 通过；仅解析，不执行脚本 |
| `rtk proxy git diff --check` | 通过 |
| `rtk proxy actionlint --version` | 未运行验证：`actionlint` 不在 PATH，RTK 返回 program not found |
| cargo-audit / hosted workflow | 未运行；本地未下载或执行二进制，也没有 hosted run 证据 |

## §3 已知缺口

1. 本地 YAML/Bash 静态检查不证明 GitHub Actions hosted workflow 成功。
2. 本 lane 未核验 cargo-audit 二进制签名或独立 attestation；消费者验证仍缺。
3. cargo-audit 对当前 Rust 依赖图的审计结果尚无运行记录。
4. W4 仍 OPEN/BLOCKED，v1.0.6 继续 NO-GO；本修复不关闭其他 CI、许可证、SBOM、签名或发行门禁。

## §4 子代理失败接手清单

本 lane 未派发子代理，因此无失败子代理待接手。协调者仍需独立复核本次 diff、WBS 同步和目标 dev 集成状态。

## §5 守门规则

1. 保持 cargo-audit 固定在 v0.22.2 与 Rust 1.99.0 toolchain。
2. 保留强制 SHA-256 校验；不得以 `continue-on-error`、占位摘要或忽略失败替代。
3. 不据静态检查推断 hosted CI、二进制签名、attestation、消费者验证或稳定发行通过。
4. 仅接纳本 lane 登记的最小文件清单，不吸收同路径 Dependabot 分支的其他改动。
5. 保留未授权分支、工作树、`main` 和 `D:\Star` 用户内容。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 仅修复 W4 的 asset 身份校验输入 |
| SRE Lead | SRE Lead（Mavis 接手审核）— hosted workflow 与运行态未验证 |
| 平台 | 平台（Mavis 接手审核）— YAML/Bash 静态检查通过；actionlint 不可用 |
| 评审主持 | 评审主持（Mavis 接手审核）— lane 自审完成，协调者独立复核待执行 |
| PM | PM（Mavis 接手审核）— W4 仍 OPEN/BLOCKED，v1.0.6 NO-GO |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 cargo-audit v0.22.2 官方归档名与摘要配置，记录静态验证和未闭合门禁。 | W4 上游资产审阅确认 workflow 文件名与 SHA 配置错误 |
