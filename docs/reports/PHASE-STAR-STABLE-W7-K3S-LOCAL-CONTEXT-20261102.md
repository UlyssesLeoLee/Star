# PHASE-STAR-STABLE-W7-K3S-LOCAL-CONTEXT-20261102

> **状态：W7 OPEN / BLOCKED；v1.0.6 NO-GO。** 本阶段修复了本地 API 部署脚本绕过已存在 Star 闭包过滤器的问题；静态回归验证通过，但没有构建镜像或启动 Multipass/K3s。

## §0 目的与范围

固定基线为本地 `dev@7f0b993bd9166ea97f0a04e95c359c50fcf1dea9`。复核发现 `deploy/k3s-local/build-and-deploy.sh` 直接复制完整 `crates/` 到临时 Docker context，并使用未展开的源 Dockerfile；这绕过 `scripts/automation/star_k3s_context.py` 的 Cargo 闭包筛选和 game/legacy 拒绝规则。本阶段只修改已登记的部署脚本、回归测试、阶段报告、review、dispatcher JSONL 与 WBS。

没有修改 context generator、Dockerfile 模板、D2 owner 持有的 `deploy/k3s-local/README.md`、其他 worktree、`D:\Star`、`main`、CI workflow 或任何远端内容。

## §1 改动矩阵

| 项目 | 结果 | 证据/边界 |
|---|---|---|
| 临时 context 生成 | 完成 | 脚本在专属临时根目录下创建新的子目录，并调用 `star_k3s_context.py --repo ... --output ...`；不再复制整个 `crates/`。生成器负责解析 Star API Cargo 闭包、校验输入并拒绝 game/legacy。 |
| Dockerfile 输入 | 完成 | `docker build -f` 指向生成 context 内已展开的 `deploy/k3s-local/Dockerfile`，build context 是该已验证目录。 |
| 本地 K3s 流程 | 保留 | 仍执行 `docker save`、导入 K3s `k8s.io` containerd、校验镜像可见性、apply manifest 和 rollout。未增加 Docker Desktop 前置条件。 |
| 回归保护 | 完成 | 新增 focused unittest，断言部署脚本调用过滤器、使用生成 Dockerfile/context、无 broad crates copy，并保留 image import/deploy 步骤。 |
| D2 README | 未触碰 | MAGOS 发现 D2 worktree 持有 README 未提交改动；该文件不在本 lane manifest 且保持原样。 |

## §2 验证摘要

- `StarK3sContextTests.test_deploy_script_uses_filtered_context_generator`：1 passed。
- `bash -n deploy/k3s-local/build-and-deploy.sh`：通过。脚本已按仓库 LF 策略规范化（git ls-files --eol 显示 i/lf w/lf attr/text eol=lf）；最初 CRLF 工作副本触发 Bash EOF 诊断，LF 临时副本与最终工作副本均通过。未改变脚本语法逻辑。
- `rtk proxy git diff --check`：通过。
- CypherGraph Guardian `--changed --stamp`：2 个 source files checked，PASS。
- 未运行 Cargo、Docker、Multipass、镜像发布、部署或 rollback。当前证据只说明调用路径与脚本语法；没有生成实际 context archive、image layers 或 SBOM。

## §3 已知缺口

1. 未使用 Rust 1.99.0 在当前目标环境实际生成 context；Rust 1.99.0 的本机 compiler/runtime 门此前仍未通过。
2. 没有检查真实 Docker archive/context、最终镜像层、digest、SBOM、provenance、签名或独立 consumer 验证。
3. 没有 Multipass guest build、K3s cluster rollout/rollback 或两实例容量证据。该本地脚本不证明 Star 原生 Multipass 多集群管理闭环，也不改变 guest VM 归属或资源预留边界。
4. `deploy/k3s-local/README.md` 仍由 D2 owner 持有；其支持范围/退役说明待该 owner 完成后单独审阅。
5. 因上述缺口，W7 保持 OPEN / BLOCKED，v1.0.6 保持 NO-GO。

## §4 子代理失败接手清单

- 本 lane 由指定 worker 在独立 worktree 完成；没有派生子代理失败、RPC 失败或待接手产物。
- 外部 README owner 的未提交修改不属于本阶段，也没有读取、复制、覆盖或清理。

## §5 守门规则

1. Docker build 输入必须由 Star Cargo 依赖闭包过滤器生成；不得回退到整个 `crates/` 目录。
2. 不得使用未展开的 Dockerfile 模板作为构建文件。
3. 过滤器拒绝 game/legacy 输入、symlink、越界路径或上下文篡改时，部署失败关闭。
4. 保留现有 K3s containerd `k8s.io` 导入、manifest apply 与 rollout 检查。
5. 本地 host Docker CLI 用法不构成 Docker Desktop 依赖声明，也不证明 Multipass 多集群支持。
6. 本阶段静态测试不替代真实 filtered context、image、SBOM、provenance、Multipass/K3s 和 consumer/rollback 验收。
7. `deploy/k3s-local/README.md` 由 D2 owner 持有，必须保持原样。
8. W7 状态仍 OPEN / BLOCKED，稳定版 v1.0.6 仍 NO-GO。
9. 未修改 main、未 push、未 fetch、未发布镜像、未创建 tag/release、未部署。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— filter 与实际本地 build script 已接线；多集群证明仍缺。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 真实镜像、Multipass、K3s rollout/rollback 未运行。 |
| 平台 | 平台（Mavis 接手审核）— Docker build 使用生成的 Star 闭包 context 与 per-crate Dockerfile。 |
| 评审主持 | 评审主持（Mavis 接手审核）— 聚焦回归测试与 shell syntax 通过；未声称发布门关闭。 |
| PM | PM（Mavis 接手审核）— W7 OPEN / BLOCKED，v1.0.6 NO-GO。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 让本地 API build-and-deploy 路径使用 Star Cargo 闭包 context 生成器，增加静态回归检查，并保持 W7/稳定发行门 OPEN。 | 检出部署脚本仍直接复制完整 `crates/` 后，按 WBS-W7 登记范围修复 |
