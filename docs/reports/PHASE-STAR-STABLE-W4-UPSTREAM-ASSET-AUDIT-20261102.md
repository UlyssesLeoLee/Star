# PHASE-STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102

> **状态：OPEN / BLOCKED。** 本审计确认了 cargo-audit 上游资产元数据与下载摘要，但没有修改或执行 CI 工作流；W4 与 v1.0.6 稳定发行均未关闭。

## §0 目的

按已提交的 brief 对 license/advisory workflow 的 cargo-audit 0.22.2 下载资产做窄范围证据复核，并同步 W4 与稳定发行 WBS。固定基线为本地 dev@2b3aab3366f82f0b5e12603d351e32128db5fe41；lane 起始提交为 2340d2e845e8bc146ff72e7ff0698bc67c1355ab，仅含 brief 与 dispatcher 记录。范围限于该 lane 登记的报告、review、JSONL 与 WBS。没有修改 workflow、Rust/Cargo 配置、产品代码或其他 lane；没有 fetch、build、run workflow、deploy、publish、push、创建 tag/release，也没有改动 main 或 D:\Star 用户文件。

## §1 上游资产与工作流审查矩阵

| 检查项 | 结论 | 证据 |
|---|---|---|
| RustSec 官方 release | 已确认 | 官方 GitHub release API 对 tag cargo-audit/v0.22.2 返回 release id 334918644。对应 asset id 439292222，名称 cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz，大小 6,648,968 字节，API digest 为 sha256:7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428。[官方 release API](https://api.github.com/repos/rustsec/rustsec/releases/tags/cargo-audit%2Fv0.22.2) |
| 下载摘要复核 | 一致 | 2026-11-02 UTC 重新从 asset 的 GitHub browser_download_url 下载相同归档；本机 Get-FileHash SHA-256 与 API digest 完全相同。临时下载文件未执行并已删除。 |
| 当前 workflow 输入 | 仍失效 / fail closed | .github/workflows/license-gate.yml 固定 CARGO_AUDIT_VERSION=0.22.2，但 CARGO_AUDIT_SHA256 仍是 REPLACE_WITH_UPSTREAM_SHA256_FOR_0.22.2；脚本明确在该占位值下退出 2。该报告没有将已验证摘要写入 workflow。 |
| 当前 workflow URL | 与上游资产不匹配 | workflow 组合出的文件名为 cargo-audit-v0.22.2-x86_64-unknown-linux-musl.tar.gz；官方资产为 cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz。URL/名称错误与 SHA 占位是两个独立缺陷。 |
| Rust baseline 来源 | 保持 1.99.0 | rust-toolchain.toml 的 channel 与根 Cargo.toml 的 rust-version 均为 1.99.0。不得以 Dependabot 候选把 CI toolchain 提到 1.120.0。 |
| workflow 候选来源 | 不接纳 / 保留 | 本地 remote-tracking candidate commit f6bede9301572ddd249ffd8c2baa39af4f106c0e 的提交 patch 将 8 个 workflow 的 19 个 dtolnay/rust-toolchain 引用从 @1.99.0 改为 @1.120.0。它与 Star 声明的 Rust 1.99.0 基线冲突；本报告只记录该固定提交 patch，不代表 live PR 或 live remote 状态。MAGOS 因 ci.yml 与 license-gate.yml 来源文件重叠而阻止修改；没有登记覆盖，也没有改动或 cherry-pick 来源。 |

## §2 验证摘要

- 对官方 GitHub release API 进行只读 GET，核对 release id、tag、asset id/name/size/digest。
- 直接下载该 asset 到唯一命名的临时路径，用 Get-FileHash -Algorithm SHA256 核对 API digest；hash 相同。下载归档没有执行，临时文件已删除。
- 读取当前 license-gate.yml、rust-toolchain.toml 与根 Cargo.toml，确认占位值、文件名差异和 Rust 1.99.0 baseline。
- 用 rtk proxy git show / git diff 只读检查候选 commit f6bede9301572ddd249ffd8c2baa39af4f106c0e 的提交统计与 workflow patch；确认 8 files / 19 replacements。没有切换、改写或合并该来源。
- 只运行文档 lane 的 JSONL、WBS、报告章节与 whitespace 检查；没有运行 Cargo、产品测试、构建或 hosted workflow。

本节中的 API 与本机下载结果是 2026-11-02 的快照。它证明 API 声明的资产身份与本次下载字节摘要匹配，不证明发布者签名、独立 attestation、归档内容安全、可执行程序行为或 workflow/CI 成功。

## §3 已知缺口与下一步工作

| 优先级 | 缺口 | 后续验收 |
|---|---|---|
| P0 | workflow digest 仍是占位符，下载路径也与实际 asset 文件名不匹配；因此 advisory gate 仍会失败关闭且无法取得该二进制。 | 先取得针对当前登记清单的 MAGOS 许可并由 owner 审阅来源，随后以最小 workflow 修改同时对齐上游资产 URL/name 与固定摘要；在目标 Rust baseline 上验证下载、hash、解包与 cargo-audit 执行，并保存实际 hosted run。 |
| P0 | W4 所需的 CI/license workflow、Action、权限/失败策略、required-check 与外部保护仍未全部满足。 | 按 W4 WBS 单独解决来源冲突与其余工作项，取得受保护的 hosted workflow 与 repo protection evidence。 |
| P0 | v1.0.6 的全平台 artifact、Web/Vercel、三份 Canvas 镜像、BOM/SBOM、provenance、签名/消费验证和 rollback 证据仍不完整。 | W0–W8 全部门禁闭合后进行最终 release-candidate 复核；此前保持 NO-GO。 |
| 限制 | 本次检查使用未认证 GitHub API 与 public release asset。 | 未验证签名、tag signer、checksum 文件的独立发布通道或 GitHub Actions hosted run；需要这些证明的门槛仍开放。 |

## §4 子代理失败接手清单

- 本 lane 未再派生子代理；没有其他 worker 产出需要接手。
- 协调阶段尝试通过 Orca 启动 Gemini terminal，但该 CLI 进入账户认证确认画面。没有继续认证；随后检查 lane 的 Git 状态仍干净，未观察到文档或工作流写入。文档工作在本隔离 lane 内继续完成。
- 本 lane 工作由当前指定 worker 完成；未读取、移植或覆盖其他工作树内容。

## §5 守门规则

1. 只写本 lane 登记的四个文档/数据文件，不修改 brief 或 workflow。
2. 区分上游 API 元数据、本机下载摘要、签名/attestation 与 hosted CI 证据。
3. 只核验精确的 release tag、asset id、asset name、size 和 digest。
4. 下载归档仅计算摘要，不执行或解包。
5. 不能把本次一致的 digest 描述成 workflow 已修复。
6. 必须保留 workflow 中的 placeholder 和错误 URL 作为当前阻塞事实，不得隐含写成成功状态。
7. 将 URL/文件名错配与 checksum 占位分开追踪。
8. cargo-audit W4 子项保持 OPEN/BLOCKED。
9. v1.0.6 稳定发行继续 NO-GO。
10. candidate commit 只以完整 SHA 标识，不用 branch 名推断 live PR 状态。
11. 不将对 candidate 的只读审计描述为接受、合并或运行 workflow。
12. Rust 1.99.0 toolchain/channel 与 Cargo MSRV 是当前基线；1.120.0 candidate 不兼容，来源保持原样。
13. MAGOS source overlap 未获覆盖，不能绕过或修改重叠 workflow。
14. 不修改、清理或覆盖 D:\Star 用户内容、main 或其他 lane/worktree。
15. 不 fetch、push、build、publish、deploy、tag 或 release。
16. WBS 只更新 W4 的新证据和版本历史，不重写其他 WBS 状态。
17. 静态文档校验不代替 Cargo、workflow、artifact 或 release 验收。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 上游资产 API digest 与本次下载摘要相符；workflow 未改，W4 OPEN。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 本 lane 未执行 cargo-audit 或取得当前候选的 hosted run；运行门仍阻塞。 |
| 平台 | 平台（Mavis 接手审核）— URL/asset filename mismatch 与 SHA placeholder 分别登记。 |
| 评审主持 | 评审主持（Mavis 接手审核）— candidate commit 的 8 workflow/19 引用变化只作来源重叠证据，不纳入。 |
| PM | PM（Mavis 接手审核）— v1.0.6 仍 NO-GO，不创建 release。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复核 cargo-audit 0.22.2 官方资产与直接下载 SHA-256；记录 workflow filename mismatch、占位摘要和 1.120.0 来源冲突；W4/稳定版保持 OPEN/NO-GO。 | STAR-STABLE-W4-UPSTREAM-ASSET-AUDIT-20261102 brief |
