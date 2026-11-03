# Review — Star Stable W4 cargo-audit Asset SHA Repair

## 审阅状态

**LANE SELF-REVIEW ACCEPT**，仅限下述文件和静态验收。协调者仍需独立复核后再集成 dev。

## 基线与范围

- 基线：`dev@234ea49eafa9f8a6acf466b32f134f9518aeebec`
- 候选：`codex/star-stable-w4-audit-sha-20261103`
- 修改：`.github/workflows/license-gate.yml` 与本 lane brief、阶段报告、review、dispatcher JSONL。
- 未修改 WBS 主文件、Rust toolchain、其他 workflows、PR #292、main、远端或其他工作树。

## 发现与判定

| 检查 | 结果 |
|---|---|
| Version | 固定为 v0.22.2 |
| 官方 asset 名称 | `cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz` |
| 下载 URL | Release tag 路径保留 `cargo-audit%2Fv0.22.2`，asset basename 与官方名称一致 |
| SHA-256 | `7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428` |
| Fail closed | 保留占位值拒绝、严格 shell 选项及 checksum 失败传播 |
| YAML 与 Bash | PyYAML parse/assertions 通过；提取的 step 用 `rtk proxy bash -n -c` 语法解析通过 |
| Actionlint | 不可用：RTK 无法解析 PATH 中的 `actionlint` |
| 运行态 | 未执行 cargo-audit、未下载二进制、未运行 hosted Actions |

候选修复了上游资产核验报告指出的 URL/文件名与摘要配置问题。静态验证不证明 asset 下载成功、二进制签名有效或 hosted workflow 通过。

## 已知缺口

W4/WBS 和稳定发行门禁尚未关闭。仍缺 hosted workflow 结果、cargo-audit 实际执行结果、二进制签名或独立 attestation，以及消费者验证记录。协调者需把本 lane 证据同步到 WBS 并完成独立审阅。

## 判定

接受本 lane 候选，等待协调者复核及本地 dev 集成。不得将本 review 解读为 PR #292 已接受，也不得解读为 W4 或 v1.0.6 通过。

## 修订历史

| 版本 | 日期 | Reviewer | 结果 |
|---|---|---|---|
| v0.1 | 2026-11-03 | 架构师（Mavis 接手审核）— lane self-review | ACCEPT lane 内静态修复；协调者独立复核待完成 |
