# PHASE-STAR-STABLE-W4-AUDIT-EXECUTION-20261106

**状态：本地审计已执行；W4 OPEN / BLOCKED；v1.0.6 稳定版 NO-GO**
**基线：** `dev@a01c66ab93d9884891d0ea885bf74117a5d22d56`
**范围：** 只读扫描三个已登记 Rust lockfile，并核对现行 cargo-audit workflow 的静态资产配置。
**Brief：** [W4-CARGO-AUDIT-EXECUTION-20261106.md](../briefs/W4-CARGO-AUDIT-EXECUTION-20261106.md)

## §0 目的

按 W4 brief 使用 Rust/Cargo 1.99.1 与 cargo-audit 0.22.2，扫描稳定版范围内的三个 lockfile。记录审计库版本、实际发现和 lockfile 未变证据，并将现行 workflow 与已登记的 RustSec 上游资产审查作静态对照。本报告不更改依赖、workflow、WBS 或产品代码。

结论：审计命令和 advisory 数据库均可运行，但根 workspace 与 BFF lockfile 存在未解决的安全 advisory。ACI emitter lockfile 未发现漏洞。宿主 CI 运行、必需检查保护及签名/来源证明未在本 lane 验证。因此 W4 仍 OPEN / BLOCKED，v1.0.6 稳定版保持 NO-GO。

## §1 改动矩阵

| 路径/对象 | 本阶段动作 | 结果 |
|---|---|---|
| `Cargo.lock` | cargo-audit 只读扫描 | 12 个漏洞发现；SHA-256 前后相同 |
| `bff/Cargo.lock` | cargo-audit 只读扫描 | 8 个漏洞发现；SHA-256 前后相同 |
| `tools/aci-emitter/Cargo.lock` | cargo-audit 只读扫描 | 0 个漏洞发现；SHA-256 前后相同 |
| `.github/workflows/license-gate.yml` | 只读静态核对 | 版本、资产文件名及 SHA 与现有上游资产审查一致；没有执行 hosted workflow |
| 本阶段报告与审查 | 新建 | 仅新增已登记的 report/review 文件 |
| WBS、源代码、依赖、workflow | 未修改 | 保持原状 |

## §2 验证摘要

### 工具与 advisory 数据库

通过 RTK 运行的版本命令返回：

```text
rustc +1.99.1 --version
rustc 1.99.1 (58cf8f9c3 2026-10-13)

cargo +1.99.1 --version
cargo 1.99.1 (5f94df478 2026-08-27)

cargo +1.99.1 audit --version
cargo-audit-audit 0.22.2
```

首个 root 扫描允许 advisory database 刷新并成功读取数据库：1,422 条 advisories；数据库提交为 `309392b18f6b17022f82c701f300433126901068`，数据库更新时间 `2026-11-06T16:34:32+01:00`。后续两个扫描使用 `--no-fetch` 固定同一数据库快照，避免不同扫描结果使用不同 advisory 数据。

### 扫描结果

以下命令均通过 `rtk proxy` 执行；退出码为 cargo-audit 子命令退出码。

| Lockfile | 命令 | 退出码 | 审计结果 | 失败分类 |
|---|---|---:|---|---|
| 根 workspace | `cargo +1.99.1 audit --file Cargo.lock --format json` | 1 | 884 个依赖；12 个漏洞；5 个被当前 cargo-audit 配置允许的 warning | 命令和数据库成功；非工具/网络故障，退出码 1 表示发现漏洞 |
| BFF | `cargo +1.99.1 audit --file bff/Cargo.lock --format json --no-fetch` | 1 | 334 个依赖；8 个漏洞；6 个被当前 cargo-audit 配置允许的 warning | 命令成功；非工具/网络故障，退出码 1 表示发现漏洞 |
| ACI emitter | `cargo +1.99.1 audit --file tools/aci-emitter/Cargo.lock --format json --no-fetch` | 0 | 72 个依赖；0 个漏洞；没有 informational warning | 通过 |

漏洞 advisory ID 与受影响包：

| Lockfile | Advisory ID / package |
|---|---|
| 根 workspace | `RUSTSEC-2026-0387` / `crypto-bigint`；`RUSTSEC-2026-0385` / `ctutils`；`RUSTSEC-2026-0418` / `libz-sys`；`RUSTSEC-2026-0414`、`RUSTSEC-2026-0433` / `pulldown-cmark`；`RUSTSEC-2023-0071` / `rsa`；`RUSTSEC-2026-0346`、`RUSTSEC-2026-0347`、`RUSTSEC-2026-0348`、`RUSTSEC-2026-0349` / `rustls`；`RUSTSEC-2026-0382` / `rustls-webpki`；`RUSTSEC-2026-0381` / `webpki-roots` |
| BFF | `RUSTSEC-2026-0418` / `libz-sys`；`RUSTSEC-2023-0071` / `rsa`；`RUSTSEC-2026-0346`、`RUSTSEC-2026-0347`、`RUSTSEC-2026-0348`、`RUSTSEC-2026-0349` / `rustls`；`RUSTSEC-2026-0382` / `rustls-webpki`；`RUSTSEC-2026-0381` / `webpki-roots` |

The two `pulldown-cmark` findings report CVSS 7.5 (High); `rsa` reports CVSS 5.9 (Medium) and the audit output did not identify a fixed upgrade. Other findings are recorded by advisory ID above; this phase did not change dependencies or decide remediation/exception policy.

### Lockfile immutability

SHA-256 before and after the scans matched:

| File | SHA-256 |
|---|---|
| `Cargo.lock` | `28101371550bd9e9e70c47ec5555f3fae84c24e28c9f77207ed983d9fb118527` |
| `bff/Cargo.lock` | `fee5ffd786378c3cecc5a84afda7945b5f2f65d4f0820ed642865112c1d37ccc` |
| `tools/aci-emitter/Cargo.lock` | `d8c8e1b85ec2ed6237b780d4c0f9ea7e8aff57e47da0a1055de799414641e3eb` |

### Workflow asset comparison

现行 `.github/workflows/license-gate.yml` 静态配置为 cargo-audit `0.22.2`、SHA-256 `7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428`，下载文件名模板为 `cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz`。这些值与 [W4 上游资产审查](../reviews/WBS-STAR-STABLE-W4-UPSTREAM-ASSET-REVIEW-20261102.md) 记录的 RustSec release asset 元数据一致。Nov 2 审查中“现有 workflow 仍为占位值”的状态描述已过时；当前 workflow 文件已经不再使用该占位值。本阶段只做本地静态对照，没有下载 hosted workflow 中固定的二进制，也没有证明 hosted workflow 实际运行或其签名/来源 provenance。

## §3 已知缺口

1. 根 workspace 与 BFF 有分别 12、8 项漏洞发现，需由依赖 owner 评估升级、隔离或有期限的风险接受；本阶段没有改 lockfile，也没有将发现豁免。
2. 未查询或核实该 workflow 的 hosted run、分支保护中的 required check 状态或 release 运行证据。
3. 没有收集固定二进制的签名、attestation 或可验证构建 provenance；本机 cargo-audit 子命令版本只说明本地工具版本，不能证明 hosted 固定二进制的来源。
4. 没有验证 v1.0.6 的桌面、Web/Vercel 与 Star Canvas 镜像制品。
5. 未执行测试、构建、容器检查、SBOM、性能或 Multipass/K3s 验收；这些不在本阶段的只读审计范围。

## §4 子代理失败接手清单

| 项目 | 状态 |
|---|---|
| 本 lane 的子代理 dispatch | 未派发子代理 |
| 子代理失败或未回传的结果 | 无 |
| 需接手事项 | Hosted run、required-check protection、签名/来源证明及漏洞处置由后续 W4 owner 继续核验 |

## §5 守门规则

- 三个登记 lockfile 的 audit 已运行；数据库刷新在 root 扫描成功，后续复用固定快照。
- 两个退出码 1 是 cargo-audit 检出漏洞，不是命令或网络错误；ACI 扫描退出码 0。
- 三个 lockfile 的扫描前后 SHA-256 一致。
- workflow 资产版本、下载名及摘要只做静态对照，没有修改 workflow。
- 没有把本地插件版本当作 hosted 二进制 provenance，也没有把静态配置当作 hosted run 证据。
- 由于漏洞发现及 hosted/provenance 证据缺失，W4 OPEN / BLOCKED；v1.0.6 NO-GO。
- 本阶段未执行 cargo test、clippy、构建或部署；不得据此声称这些门禁通过。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 只读审计范围符合稳定版发布边界；协调者终审待完成 |
| SRE Lead | hosted run 与分支保护证据未取得；待 owner 核验 |
| 平台 | workflow 资产静态对照完成；固定二进制 provenance 未验证 |
| 评审主持 | 本 worker 已完成自检；独立协调者复核待完成 |
| PM / Release Owner | 最终发行候选、tag 与发布审批责任由用户担任；本阶段未请求或推断发布批准 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 记录三个 lockfile 的 cargo-audit 执行、advisory 结果、lockfile 哈希和 workflow 静态资产对照；维持 W4 OPEN / BLOCKED 与 v1.0.6 NO-GO | W4-CARGO-AUDIT-EXECUTION brief |
