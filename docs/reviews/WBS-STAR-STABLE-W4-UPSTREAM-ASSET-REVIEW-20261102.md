# W4 cargo-audit 上游资产审阅 — 2026-11-02

## 决定

**证据项通过核对，实施门仍 HOLD / BLOCKED。** RustSec 官方 GitHub release API 对 cargo-audit/v0.22.2 返回 asset id 439292222，文件名 cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz，大小 6,648,968 字节，digest 为 sha256:7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428。2026-11-02 对该直接下载归档重新计算 SHA-256，结果与 API digest 一致。

当前 .github/workflows/license-gate.yml 仍将 CARGO_AUDIT_SHA256 设为 REPLACE_WITH_UPSTREAM_SHA256_FOR_0.22.2，并尝试下载 cargo-audit-v0.22.2-x86_64-unknown-linux-musl.tar.gz，与官方 asset 名称不符。因此实际 gate 仍 fail closed 且没有通过证据。上游摘要核对不能替代 workflow 修复、二进制签名/独立 attest、cargo-audit 执行或 hosted run。

## 来源与边界

- 本次官方元数据来自 [RustSec release API](https://api.github.com/repos/rustsec/rustsec/releases/tags/cargo-audit%2Fv0.22.2)，下载来自其 asset browser_download_url；未执行归档内容。
- 只读审阅固定 candidate commit f6bede9301572ddd249ffd8c2baa39af4f106c0e，其提交统计为 8 个 workflow、19 个引用从 Rust 1.99.0 改为 1.120.0。当前仓库 rust-toolchain.toml 和根 Cargo.toml 声明 Rust 1.99.0；该 candidate 不适合当前基线。
- MAGOS 因 ci.yml / license-gate.yml 与该 candidate 的文件重叠而拒绝 workflow 实施 lane。未登记 override、未改工作流、未 cherry-pick 或清理 source。
- 本报告不验证 live PR、当前远端 ref、签名、attestation、workflow run 或 v1.0.6 release。W4 保持 OPEN/BLOCKED，稳定发行 NO-GO。

## 审阅检查

| 检查 | 结果 |
|---|---|
| API asset id/name/size/digest | 与本次下载的目标一致 |
| 直接下载 SHA-256 | 与 API digest 一致 |
| workflow digest 设置 | 仍是占位符，失败关闭 |
| workflow URL 构造 | 文件名与上游 asset 不同 |
| Rust baseline candidate | 1.120.0 与当前 1.99.0 不一致 |
| 实际 workflow/产品运行 | 未执行 |
| W4 / v1.0.6 状态 | OPEN/BLOCKED / NO-GO |

## 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| v0.1 | 2026-11-02 | 记录官方 API 元数据和直接下载 hash 的一致性，同时保留 workflow 与发布门禁缺口。 |
