# PHASE-D2-MULTIPASS-MAINTENANCE-20261031

> **状态：PARTIAL / OPEN**（2026-10-31）。四个旧 K3s maintenance 入口已改为 fail-closed；Canvas 部署文档要求使用既有、可达的 K3s context。独立 `star-api-rest` 的旧 `build-and-deploy.sh` 不在本 lane manifest，仍可执行，因此本轮不宣称全部旧部署入口已禁用。

> **当前状态复核（2026-11-06）**：用户已指定 Rust 1.99.1 作为目标基线；该 lane 起始 checkout 固定 1.99.0，后续 rebase 到 dev 后当前 checkout 使用 1.99.1，但 lane 未运行 workspace metadata/build，D4 继续 OPEN。经父级审查，lane 的两笔提交已 rebase 并 fast-forward 到本地 dev；集成提交为 `1ce184aecde0c94bab386b4435c39852d00e66b2`。

## §0 目的

执行 [D2 maintenance brief](../briefs/WBS-STAR-DELIVERY-D2-MULTIPASS-MAINTENANCE-20261031.md)，将被 D4 发现的旧 start/update 入口从 WSL、本机 K3s 和不完整 Helm/服务流程改为 fail-closed；明确当前受支持的 Canvas 应用部署边界，并保留 Multipass 的 2-instance inventory cap、独占锁、fail-closed inventory 和 no-auto-purge 约束。

## §1 改动矩阵

| 改动 | 文件范围 | 验收 / 状态 |
|---|---|---|
| K3s start/update 入口退役 | `maintenance/start-k3s-backend.{bat,ps1}`、`maintenance/update-backend-k3s.{bat,ps1}` | 入口只返回非零状态并说明不执行任何 VM/cluster 操作；更新入口不再执行拉取、Helm、镜像或 rollout 操作；COMPLETE（本范围） |
| 前端入口提示 | `maintenance/start-frontend.ps1`、`maintenance/update-frontend.ps1`、`scripts/start-dev.ps1`、`scripts/start-dev.ps1.deprecated` | 不再推荐旧 updater；引导查阅 Canvas 部署指南，并要求已有、可达的集群；COMPLETE |
| API 旧部署说明 | `deploy/k3s-local/README.md` | 明确为非操作指南，禁止执行旧入口；真实 API image build/publish/deploy 契约仍未提供；PARTIAL |
| Multipass 边界 | `deploy/multipass/README.md`、`docs/deployment/star-canvas-services-deploy.md`、`scripts/automation/multipass_k3s_gitops.ps1` | 文档区分只读预检、新 guest provisioning 与既有集群应用部署；provider 未改动，cap/锁/inventory/no-auto-purge 保持；静态核对通过 |
| 仍待登记的 API 入口 | `deploy/k3s-local/build-and-deploy.sh` | 不在当前 manifest。已读源码确认其调用 `docker build/save`、sudo k3s containerd import、`crictl images`、`kubectl apply/rollout/get pods`；README 已警示不要运行，脚本本身仍可执行；OPEN，需单独登记 lane |
| 旧 Helm NOTES 残留 | `deploy/helm/star/templates/NOTES.txt` | 不在当前 manifest；仍声明 8 services，但当前 chart 缺少相应 Deployment/Service templates。未通过 Helm/K3s 验证。若稳定发行审计确认影响交付，须单独登记；OPEN |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| PowerShell parser | 变更的 6 个 PowerShell 内容文件全部 AST parse PASS（含扩展名为 `.ps1.deprecated` 的兼容文件） |
| Batch 文本 / 引用 | 两个旧 `.bat` 不再调用 PowerShell、WSL、Helm 或其他部署命令；都返回 `exit /b 2` 并指向 Canvas 部署指南；PASS |
| 活动路径扫描 | 四个旧 K3s maintenance bodies 无旧 registry、旧 Helm upgrade、WSL 执行或旧服务矩阵；Canvas 指南只部署 Star Canvas 服务。`deploy/k3s-local/build-and-deploy.sh` 仍可执行；`deploy/helm/star/templates/NOTES.txt` 仍保留未经部署验证的 8 services 陈述。全仓旧入口/旧交付文案扫描未通过，不作通过声明 |
| Multipass invariant 静态审阅 | `MaxTestPhaseInstances = 2`、独占锁覆盖 inventory 检查与 launch、Stopped/Deleted 记录占用 cap、未知/达上限 fail closed、无自动 purge；PASS（源码阅读，未调用 Multipass） |
| CypherGraph Guardian | `--changed --stamp` PASS：5 个受支持源文件格式通过；`.ps1.deprecated` 与 `.bat` 的 header freshness 由 lane 静态核对补验 |
| 独立 CodeRabbit 审阅 | 不可用：Windows shim 启动 `/root/.local/bin/coderabbit` 返回 `Permission denied`；无独立 review 结果，不计为通过 |
| `git diff --check` | PASS；无 whitespace error |
| VM / Docker / K3s / Helm 操作 | 未执行，符合 brief；不宣称构建、镜像发布或 runtime 通过 |

## §3 已知缺口

1. `deploy/k3s-local/build-and-deploy.sh` 仍可执行；它使用 Docker CLI、本机 k3s/containerd 和 kubectl，部署目标为 Star API。README 的警示不能替代脚本禁用。该文件不在本 lane manifest，需要单独登记。
2. Multipass provider 当前提供受控只读预检和创建新 guest 的 provisioning 流程，没有已验证的启动/恢复既有 guest 操作；旧 start 入口不会被重定向到会创建 VM 的 `Stage all`。
3. `star-api-rest` 没有本 lane 可验证的受支持镜像 build/publish/deploy 工作流；当前 Canvas 部署脚本不覆盖它。
4. 未运行 Rust workspace build、OCI build/publish、K3s apply、rollout 或 endpoint 检查；上游 WBS 的 D4 必须保持 OPEN。
5. `maintenance/start-frontend.bat` 还含一处提及旧 start BAT 的说明性注释；该 `.bat` 不在本 lane manifest，也没有运行调用；本轮未修改。
6. `deploy/helm/star/templates/NOTES.txt` 仍报告 8 个服务，但 chart 当前没有相应 Deployment/Service templates。未通过 Helm/K3s 验证；如稳定发行审计认定影响交付，须单独登记处理。

## §4 子代理失败 / 接手清单

- 本次由独立 worker 接手复核既有 D2 worktree，未重写或清理此前工作；修改限于 WBS 与阶段报告的工具链证据和状态说明。
- 父级已独立审阅 16 个登记路径，检查了 fail-closed 脚本行为、Multipass cap/lock/inventory 静态边界、WBS 与阶段证据，并将 lane rebase/fast-forward 到本地 dev。独立 CodeRabbit 服务未能启动，未计作通过；`deploy/k3s-local/build-and-deploy.sh` 仍在 lane 外，需单独登记审查。

## §5 守门规则

1. 四个旧 K3s maintenance start/update 脚本 fail-closed，不允许静默降级或自动开始 VM/cluster 操作。未登记的 API build/deploy 脚本和旧 Helm NOTES 另列待决，不把它们描述为本 lane 已退役。
2. Canvas 应用部署只对已 provision 且可达的显式 K3s context 开放；部署指南和 script 不会创建 Multipass guest。
3. 保持 Multipass 测试阶段最多 2 个 inventory 实例，包含 Stopped/Deleted 记录；跨进程锁保护 inventory 检查与创建；未知状态 fail closed，不自动 purge。
4. Docker CLI/k3s/containerd API 部署材料不是当前 star-api-rest 支持流程；不得将其描述为 Docker Desktop 前置条件。
5. 本轮未调用 Multipass、Docker、kubectl、Helm 或 VM 操作；未读取凭据。
6. 不触碰历史决策文件，不改 `main`，不 push，不合并或清理 worktree。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构师 | 父级独立审阅完成；16 个登记路径已集成，D2 仍保持 PARTIAL / OPEN |
| SRE Lead | 未进行集群生命周期或部署验收；不声明通过 |
| 平台 | PowerShell 静态解析通过；Rust/镜像 build 未运行 |
| 评审主持 | 差异、PowerShell AST、batch fail-closed 和 whitespace 检查完成；CypherGraph 与活动路径扫描沿用 lane 报告证据 |
| PM | D2 保持 PARTIAL / OPEN，等待 API 遗留脚本另行登记与 D4 运行门 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录四个旧 K3s maintenance 入口 fail-closed、Canvas 部署边界、Multipass inventory invariants 静态核对及 star-api-rest 旧脚本超出 manifest 的已知缺口；未声明 build/runtime 通过 | D2 Multipass maintenance lane 实施与静态审阅 |
| v0.2 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.1 为用户指定目标，但此 lane 起始 checkout 仍固定 1.99.0；保持目标版本 metadata/build、镜像、K3s endpoint、未登记 API 部署脚本及旧 Helm NOTES 的缺口开放；记录本 lane 尚待父级独立审阅和集成 | 用户选择工具链目标与 D2 lane 当前状态复核 |
| v0.3 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录父级已独立审阅 16 个登记路径、重新运行 AST/batch/diff 检查，并将 lane 两笔提交 rebase/fast-forward 到本地 dev；当前 checkout 使用 Rust 1.99.1，但 D2 lane 未运行 workspace gate；D2/D4 仍开放，未登记 API 部署脚本、镜像和实际 Multipass K3s endpoint 仍未验收 | 完成 D2 lane 审查、集成与工具链状态复核 |
