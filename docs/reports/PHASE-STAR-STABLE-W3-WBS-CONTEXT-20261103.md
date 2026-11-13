# W3 WBS Docker Context Fact Sync

> **状态：候选文档修订，等待协调者独立复核与集成。** 本次只同步 WBS 中 k3s-local Dockerfile 的过时描述；不关闭 W3、W7 或稳定版 v1.0.6。

## §0 目的与范围

固定核对基线为 `dev@234ea49eafa9f8a6acf466b32f134f9518aeebec`。WBS 高层载荷说明仍称 `deploy/k3s-local/Dockerfile` 直接复制整个 `crates` 目录，但当前工作流已将它作为带标记的 Dockerfile 模板，由 Star Cargo 闭包生成器展开选定 crate 的 `COPY`。本次纠正 WBS 的当前事实，并保留实际容器构建与部署验收缺口。

本 lane 仅修改 WBS、brief、dispatcher receipt、本报告与 review。没有修改 Dockerfile、context generator、部署脚本、测试、D2 owner 文件、main、远端或 `D:\Star` 用户内容。

## §1 改动矩阵

| 项 | 结果 | 证据与边界 |
|---|---|---|
| WBS 高层 Docker 描述 | 删除“k3s-local 仍直接 `COPY crates ./crates`”的过时断言；说明 source Dockerfile 是模板，`build-and-deploy.sh` 调用生成器，生成器展开显式 crate allowlist 并拒绝 broad copy、game/legacy 与未授权路径。 | 当前 source `deploy/k3s-local/Dockerfile` 含 `@star-local-crates-begin/end` 标记；`deploy/k3s-local/build-and-deploy.sh` 调用 `scripts/automation/star_k3s_context.py --build-in-multipass`；生成器展开 marker 并检查 broad copy 与允许路径。 |
| 已有过滤回归证据 | 明确 fixture 对 per-crate `COPY`、broad-copy 缺失、lock 内容和 game/legacy 排除有断言。 | 仅记录当前 dev 已有测试与 W7 review；本 lane 没有重跑 tests。 |
| 发布验收状态 | 明确保留 OPEN/BLOCKED 与 NO-GO；将源代码/fixture 证据与真实 BuildKit archive、最终镜像层和部署验收分开。 | W7 junction review 记录既有 suite 20 项为 19 通过、1 项因 symlink 权限跳过；它没有生成 image、digest、SBOM 或部署证据。 |

## §2 验证摘要

- 只读核对基线 `dev@234ea49eafa9f8a6acf466b32f134f9518aeebec` 的 Dockerfile 模板、部署入口、过滤生成器、allowlist fixture 和已集成 W7 junction review。
- 既有 W7 review 的历史记录为 20 项 focused tests 中 19 项通过、1 项 symlink 权限 skip；这是此前集成的证据，本 lane 未重跑测试。
- 本 lane 验证：`rtk proxy git diff --check` 通过；Python UTF-8 whitespace / EOF-newline 检查覆盖 manifest 五份文件且无 trailing whitespace 或缺少末尾换行。
- 未运行 Docker/BuildKit、Cargo、Multipass、K3s、SBOM、provenance、consumer 或 rollback 验证。

## §3 已知缺口

1. 没有检查生成后的实际 BuildKit context archive，也没有检查最终 image layers。
2. W7 过滤生成仍未在准确 Rust 1.99.0 工具链下完成真实 context/build 验收。
3. Dockerfile base image 使用可变 tag，没有 digest pinning 证据。
4. 没有最终镜像 digest、SBOM、provenance、签名、attestation 或独立 consumer 验证。
5. 没有 Multipass/K3s rollout、health、rollback 或 workload 无 game 验收。
6. 本文档修订不构成 W3/W7 完成声明；稳定版 v1.0.6 仍为 NO-GO。

## §4 子代理失败接管清单

本 lane 未继续派发下级代理；没有失败子代理产出需要接管。协调者仍需在集成前核对 WBS 文案与本报告引用的代码事实。

## §5 守门规则

1. 构建必须通过 Star Cargo 闭包生成器提供过滤后的 context；Dockerfile source template 不是直接 build input。
2. 保留显式每 crate `COPY` allowlist 与 broad-copy/game/legacy 拒绝规则；不得以根 `.dockerignore` 单独证明载荷边界。
3. 静态实现和测试记录不得代替真实 BuildKit archive、最终 image/layers、digest、SBOM/provenance、Multipass/K3s、consumer 与 rollback 验收。
4. 未完成所有发布门禁前，W3/W7 保持 OPEN/BLOCKED，v1.0.6 保持 NO-GO；不创建 tag/release，不构建、发布、部署或 push。
5. 只记录本 lane 真实完成的文档改动和验证，不把旧报告基线写成当前代码行为。

## §6 签批栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核 — 文档边界和当前源码映射；待协调者独立复核 |
| SRE Lead | Mavis 接手审核 — 无运行时签批；Multipass/K3s 与 rollback 未验证 |
| 平台 | Mavis 接手审核 — 仅核对 source template 与生成器接线 |
| 评审主持 | Mavis 接手审核 — doc diff 自检完成；协调者审查待进行 |
| PM | Mavis 接手审核 — W3/W7 OPEN/BLOCKED，v1.0.6 NO-GO |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 纠正 WBS 中 `deploy/k3s-local/Dockerfile` 仍 broad-copy 的过时描述；区分当前模板/生成器的静态证据与未完成的真实容器/部署验收。 | 对照 dev@234ea49e 的 source、allowlist fixture 与已集成 W7 junction review |
