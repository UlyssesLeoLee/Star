# PHASE-STAR-STABLE-W7-K3S-LOCAL-CONTEXT-20261102

> **状态：W7 OPEN / BLOCKED；v1.0.6 NO-GO。** 部署入口已接入 Star Cargo 闭包过滤器，focused 回归检查通过；独立复核发现 Windows 目录 junction 可使过滤器把仓库外载荷复制进 context，故当前不能认定过滤器完整或 Star-only context 已安全。没有构建镜像或启动 Multipass/K3s。

## §0 目的与范围

固定基线为本地 `dev@7f0b993bd9166ea97f0a04e95c359c50fcf1dea9`。复核发现 `deploy/k3s-local/build-and-deploy.sh` 直接复制完整 `crates/` 到临时 Docker context，并使用未展开的源 Dockerfile；这绕过 `scripts/automation/star_k3s_context.py` 的 Cargo 闭包筛选和 game/legacy 拒绝规则。本阶段只修改已登记的部署脚本、回归测试、阶段报告、review、dispatcher JSONL 与 WBS。

没有修改 context generator、Dockerfile 模板、D2 owner 持有的 `deploy/k3s-local/README.md`、其他 worktree、`D:\Star`、`main`、CI workflow 或任何远端内容。

## §1 改动矩阵

| 项目 | 结果 | 证据/边界 |
|---|---|---|
| 临时 context 生成 | 接线完成；过滤完整性阻塞 | 脚本在专属临时根目录下创建新的子目录，并调用 `star_k3s_context.py --repo ... --output ...`；不再复制整个 `crates/`。生成器按 Star API Cargo 闭包筛选，但 `_copy_local_package` 未拒绝 Windows junction，实测可把仓库外 payload 复制进 context。 |
| Dockerfile 输入 | 完成 | `docker build -f` 指向生成 context 内已展开的 `deploy/k3s-local/Dockerfile`，build context 是该已验证目录。 |
| 本地 K3s 流程 | 保留 | 仍执行 `docker save`、导入 K3s `k8s.io` containerd、校验镜像可见性、apply manifest 和 rollout。未增加 Docker Desktop 前置条件。 |
| 回归保护 | 有限通过；缺 junction 用例 | focused unittest 覆盖闭包选择、路径逃逸、普通 symlink、context 篡改、guest readiness 与部署接线；未覆盖包目录内 junction，因此未拦截本轮发现的逃逸。 |
| D2 README | 未触碰 | MAGOS 发现 D2 worktree 持有 README 未提交改动；该文件不在本 lane manifest 且保持原样。 |

## §2 验证摘要

- `rtk python -m unittest scripts.automation.test_star_k3s_context`：18 项，17 passed、1 skipped；skip 是 Windows symlink 创建权限不足。
- `rtk python -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py`：通过。
- `bash -n deploy/k3s-local/build-and-deploy.sh`：通过；当前文件 LF 格式。
- `rtk git diff --check`：通过。
- 额外隔离临时目录实验调用实际 `_copy_local_package`：Windows `mklink /J` 创建的包内 junction 显示 `is_junction=True`、`is_symlink=False`；函数将 junction 目标目录中的 `payload.txt` 复制到生成 context（`outside_payload_copied=True`）。这确认一个过滤器缺口，不代表修改了仓库源文件。
- 未运行 Cargo metadata、Docker build、Multipass、镜像发布、部署或 rollback。没有生成真实 production context archive、image layers、digest 或 SBOM。

## §3 已知缺口

1. **P1 过滤阻塞：** `_copy_local_package` 只检查 `Path.is_symlink()`；Windows junction 不满足该判定，且 `os.walk(..., followlinks=False)` 仍遍历 junction。已用实际复制函数证明仓库外 payload 可进入生成 context。应在独立代码 lane 里拒绝 junction/reparse point，并加入真实目录 junction 回归测试；本 docs-only lane 不修改代码。
2. 未使用 Rust 1.99.0 在目标环境实际生成 context；先前 W7 记录的 Cargo metadata 因本机工具链无法执行而退出 101。
3. 没有检查真实 context archive、最终镜像层、digest、SBOM、provenance、签名或独立 consumer 验证。
4. 没有 Multipass guest build、K3s cluster rollout/rollback 或两实例容量证据。该本地脚本不证明 Star 原生 Multipass 多集群管理闭环，也不改变 guest VM 归属或资源预留边界。
5. `deploy/k3s-local/README.md` 仍由 D2 owner 持有；其支持范围/退役说明待该 owner 完成后单独审阅。
6. 因过滤缺口及实际制品/运行证据缺失，W7 保持 OPEN / BLOCKED，v1.0.6 保持 NO-GO。

## §4 子代理失败接手清单

- 本 lane 由指定 worker 在独立 worktree 完成；没有派生子代理失败、RPC 失败或待接手产物。
- 外部 README owner 的未提交修改不属于本阶段，也没有读取、复制、覆盖或清理。

## §5 守门规则

1. Docker build 输入必须由 Star Cargo 依赖闭包过滤器生成；不得回退到整个 `crates/` 目录。过滤器须拒绝 symlink、Windows junction/reparse point、越界路径及任何未授权输入。
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
| 平台 | 平台（Mavis 接手审核）— Docker build 已接入生成器与 per-crate Dockerfile；Windows junction 越界仍是 P1 阻塞。 |
| 评审主持 | 评审主持（Mavis 接手审核）— 聚焦测试与 shell syntax 通过；独立 junction 实验复现仓库外 payload 被复制，故发布门保持关闭。 |
| PM | PM（Mavis 接手审核）— W7 OPEN / BLOCKED，v1.0.6 NO-GO。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 让本地 API build-and-deploy 路径使用 Star Cargo 闭包 context 生成器，增加静态回归检查，并保持 W7/稳定发行门 OPEN。 | 检出部署脚本仍直接复制完整 `crates/` 后，按 WBS-W7 登记范围修复 |
| v0.2 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 独立复核发现 `_copy_local_package` 会沿 Windows junction 复制仓库外 payload；新增复现证据、P1 阻塞与修复要求，保持 W7 OPEN/BLOCKED、v1.0.6 NO-GO。 | 对实际复制函数执行隔离 junction 实验并对照 focused tests |
