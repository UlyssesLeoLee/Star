# PHASE — Stable release MIT metadata and policy slice

**状态：PARTIAL / OPEN**

## §0 目的

将用户选定的 MIT 首方许可同步到可修改的 Helm chart 元数据，更新 cargo-deny 配置说明，并把当前 stable-hardening 并行计划写入稳定版 WBS。本阶段不作完整许可合规批准，也不代表发行物已可发布。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| deploy/helm/star/Chart.yaml | 将首方 chart license annotation 从 Apache-2.0 改为 MIT | 已修改 |
| deny.toml | 将内部 crate 注释改为反映 canonical MIT；不改变许可白名单 | 已修改 |
| docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md | 记录四条隔离 lane、精确基线、owner 冲突和未关闭门禁 | 已修改 |
| docs/briefs/STABLE-HARDENING-LICENSE-POLICY-20261031.md | dispatcher 落档的任务范围与限制 | 已存在 |
| docs/reports/WBS-DISPATCH-STABLE-HARDENING-LICENSE-POLICY-20261031.jsonl | dispatcher brief 事件 | 已存在 |
| docs/reports/PHASE-STABLE-HARDENING-LICENSE-POLICY-20261031.md | 本阶段证据、限制和剩余工作 | 本报告 |

## §2 验证摘要

- 最终变更集执行 rtk git diff --check：PASS。
- Chart YAML 使用 PyYAML 解析，并核对 annotations.licenses 为 MIT：PASS。
- Python 3.13 `tomllib` 对 deny.toml 中既有的多行 inline-table 形式在第 145 行报错；这不是本 lane 改动。随后用 `rtk cargo +1.99.1 deny check` 验证实际消费者能够读取配置：`bans`、`licenses`、`sources` 为 OK，`advisories` 因现有 `webpki-roots 1.0.9` 命中 `RUSTSEC-2026-0381` 失败。输出还提示若干未命中许可白名单项和重复 lock entries；本 lane 未改 Cargo manifest 或 lock。
- 默认 pinned Rust 1.99.0 下的 `rtk cargo deny check` 在 `rustc -vV` 阶段失败，错误为 rustc.exe 不适用于当前 x86_64-pc-windows-msvc toolchain。改用已安装的 Rust 1.99.1 仅作诊断，不能替代 Rust 1.99.0 release baseline 验收。
- CodeRabbit 独立审阅尝试失败：Windows shim 调用 /root/.local/bin/coderabbit 时返回 Permission denied；因此本阶段以本地 diff/配置/WBS 复核作为审查证据，不把 CodeRabbit 失败视为通过。

## §3 已知缺口

1. 根 Cargo.toml 当前仍声明 Apache-2.0；MAGOS 证明该文件与活跃 W8 lane 及依赖分支重叠，本 lane 没有触碰。
2. .github/workflows/license-gate.yml 与远端依赖更新分支重叠；cargo-audit placeholder digest、inventory artifact 上传失败关闭等事项仍待 owner-safe lane。
3. 完整依赖/发行物许可闭包、版权、专利、NOTICE、修改和再分发义务尚未逐项复核。
4. RustGameServer adapter 的权属及可分发边界没有本阶段证据。
5. Helm chart 是 deprecated placeholder，元数据变为 MIT 不代表有实际 workload 或部署能力。
6. cargo deny、license workflow、artifact inventory 与签名证明均未通过最终门禁。

## §4 子代理失败接手清单

本 lane 由协调者执行，没有子代理失败接手。Desktop、Web/Vercel 和 Canvas OCI 是同一 stable-hardening set 的独立并行 lane；它们的结论不属于本报告，须分别审阅。

## §5 守门规则

1. Star 自有项目的 canonical license 使用用户确认的 MIT。
2. 首方 metadata 与第三方依赖许可审查分开记录。
3. 不以 GPL、AGPL 或 LGPL 标签自动认定不可商用。
4. 非商业、用途限制或 source-available 许可仍须按项目规则阻断。
5. 对依赖按实际发行目标和完整传递闭包审查。
6. 每份审查记录准确版本和构建 feature/target。
7. 逐项核对版权、NOTICE、专利与源码义务。
8. 不把上游主许可证当作完整分发闭包证明。
9. RustGameServer adapter 权属保持未决，未确认前不作批准陈述。
10. Chart 声明不替代 Cargo manifest、二进制或容器内容检查。
11. 不触碰 MAGOS 已识别的 Cargo.toml、license-gate.yml owner 文件。
12. 不改写、覆盖或清理活跃 W8、D2、D4 worktree。
13. 稳定版 W1 只有完整证据闭环后才能关闭。
14. W9 继续依赖 W0–W8 的通过证据。
15. 不提前创建稳定 tag 或公开 release。
16. 不提前发布镜像、部署或 push。
17. 保留主工作树的用户改动，并只把审查通过的 lane 集成到 dev。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis 接手审核；许可边界和 owner 冲突已复核 |
| SRE Lead | Mavis 接手审核；完整运行期发行物证据仍待补 |
| 平台 | Mavis 接手审核；cargo-deny 运行门未通过 |
| 评审主持 | Mavis 接手审核；报告仅覆盖本 lane manifest |
| PM | Mavis 接手审核；稳定版整体保持 OPEN |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新 Helm MIT metadata、policy 注释及稳定版 WBS；如实保留解析、toolchain 和依赖权属缺口。 |
