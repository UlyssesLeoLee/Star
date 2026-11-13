# PHASE-STAR-ARTIFACT-AUDIT-20261106 — Star 发布制品边界审计

> 审计结论 **已验证两条路径的本地过滤输出，稳定发布仍未放行。** 三份 Canvas context 各自仅含一个批准服务，但当前稳定发布的许可证门禁会拦截，且三份独立 Cargo 锁文件均无法通过 `--locked`。K3s 本地 API 的真实过滤 context 已通过 Cargo 检查。镜像与部署验收待办。

## §0 目的

以本地 `dev@17943a750877e887cb6808c662af65831f8de39b` 为唯一源码基线。按 [AGENTS.md §0](../../AGENTS.md) 核对批准的 Canvas 三镜像、独立的 `star-api-rest` 本地开发入口、Multipass/GitOps 与废弃 Helm 占位的边界。将真实过滤目录、策略测试、配置声明、镜像和集群运行证据分别记账。审计日期为 2026-11-06（JST），基线由本地 Git 解析，未刷新远端引用。

## §1 核对矩阵

| 面向 | 基线实证 | 结论 |
|---|---|---|
| Canvas 发布入口 | [发布工作流](../../.github/workflows/publish-star-canvas-services.yml#L95) 仅允许已批准的稳定标签手动调度，按 `canvas-engine`、`domain-canvas`、`canvas-realtime` 建立独立矩阵，校验 resolver 身份与 checkout SHA。许可证门禁位于 context 步骤之前。实际 `Cargo.toml` 的 workspace license 是 `Apache-2.0`，根 `LICENSE` 首行是 `MIT License`。 | 当前源码组合会被许可一致性门禁拒绝。 |
| Canvas 源码与 game/legacy 排除 | [context 生成](../../.github/workflows/publish-star-canvas-services.yml#L138) 只复制所选服务的 `Cargo.toml`、`src/` 和 Dockerfile，并输出专用 workspace/lock 及根文件。[共享策略](../../scripts/release/star_canvas_context_policy.py) 检查路径、包身份、复制源及祖先目录的 symlink/junction。实际三个暂存目录各 9 个文件，分别有 119、119、136 条锁记录，均仅含自身一个本地包。未发现包含 `game` 或 `legacy` 的文件路径或包名。 | 暂存目录的正向样本通过。复用已有目录时，中性文件名的未批准文件能穿过当前门禁。独立 Cargo 锁文件校验也失败，见 §2。 |
| Canvas 最终 OCI | 三份 Dockerfile 在 builder 里使用 `cargo build --locked`，runtime 仅显式复制所选 binary 和根 `LICENSE`。工作流把生成目录交给 Buildx，配置按服务缓存、SBOM、max provenance，推送后生成 digest 记录和 GitHub provenance，消费作业校验三份 SLSA provenance 再渲染部署清单。 [Dockerfile 样本](../../crates/canvas-engine/Dockerfile) · [Buildx 与证明](../../.github/workflows/publish-star-canvas-services.yml#L394) | 配置检查成立。缺少实际 BuildKit 输入、缓存、最终镜像文件系统、第三方依赖闭包及证明的验收结果。消费作业的 SLSA 验证不覆盖独立 SBOM 内容核验。 |
| 本地 API Dockerfile 和入口 | 源 Dockerfile 带逐 crate `COPY` 标记，脚本要求现有运行中的指定 Multipass guest，将构建交给生成器。基线确实生成 333 个文件与 36 条定向 `COPY`。README 的无参数调用会被脚本拒绝，其仓库根 context 命令绕过生成器，源模板缺少构建所需的逐 crate 复制。 [Dockerfile](../../deploy/k3s-local/Dockerfile) · [脚本](../../deploy/k3s-local/build-and-deploy.sh#L19) · [README](../../deploy/k3s-local/README.md#L41) | [收敛台账](PHASE-WBS-INTEGRATION-LEDGER-20261105.md#L24) 的宽泛 `COPY crates` 说法应更正。保留 README 旁路、镜像和部署缺口。 |
| Multipass 本地部署 | [生成器](../../scripts/automation/star_k3s_context.py#L837) 在传输前检查指定 guest、Docker daemon、K3s API 和节点 Ready，校验归档成员和 digest，再定向传输。在 guest 内构建，将镜像导入 `k8s.io`，检查 CRI，应用单服务 manifest 并等待 rollout。 [guest 执行](../../scripts/automation/star_k3s_context.py#L965) | 测试覆盖调用边界与失败路径。唯一 inventory 条目为 Stopped，未进入 guest 执行。 |
| Multipass 宿主实验与 GitOps | [PowerShell 启动器](../../scripts/automation/multipass_k3s_gitops.ps1#L255) 解析 inventory，在跨进程独占文件锁内检查两实例上限并 launch，无自动 purge。Stage 2 会开启 privileged mounts 并挂载整个仓库，cloud-init 另有 apply 整套 `deploy/k3s-local` 的入口。实际静态 Kustomize 渲染含 `star-api-rest:local`、`bff:local` 和两个可变 Envoy tag。 [整仓挂载](../../scripts/automation/multipass_k3s_gitops.ps1#L430) · [cloud-init](../../deploy/multipass/cloud-init-k3s-gitops.yaml#L75) | 该实验入口的挂载范围和镜像集合比单 API 入口更宽。其静态配置不能作为受限 Worktree bridge、批准的发布清单或 Star 原生多集群能力的证据。 |
| Helm 占位与引用 | [Chart.yaml](../../deploy/helm/star/Chart.yaml) 明示退役。版本库仅跟踪 Chart/values、NOTES、helper 和条件 Secret，未见工作负载模板。 [staging Application](../../deploy/gitops/argocd/argocd-applications.yaml#L99) 仍以该 chart 为源并采用手动同步， [维护脚本](../../maintenance/update-backend-k3s.ps1#L310) 仍调用 Helm upgrade， [根 README](../../README.md#L170) 仍把 YAML 解析错误写成现状。 | 占位没有应用交付证据，仍存在调用路径。此次只核验 dev 的清单和脚本，未核验 staging 目标分支的 chart 内容或远端 ArgoCD 同步。 |
| Canvas 部署模板 | [Canvas manifest](../../deploy/star-canvas-services-k3s.yaml) 保留三份 digest 占位符和版本占位符，额外包含 digest 固定的 Envoy 与 `ServiceMonitor`。 | 必须使用发布作业渲染的清单并在目标集群核验所需 CRD 和实际 image ID。模板或本地 API rollout 均不能代替此项。 |

## §2 验证摘要

| 命令或操作 | 结果 |
|---|---|
| `git rev-parse HEAD dev` | 两者均为 `17943a750877e887cb6808c662af65831f8de39b`。 |
| `py -3 -B -m unittest discover -s scripts/release/tests -v` | 64/64 通过，0 跳过。覆盖版本解析、context 策略、内联脚本 fixture、manifest 渲染和消费证明验证的契约。 |
| `py -3 -B -m unittest -v scripts.automation.test_star_k3s_context` | 20 项运行，19 通过，1 跳过。Windows symlink fixture 缺创建权限（WinError 1314），真实 junction 回归通过。 |
| 实际 Canvas context | 用 `py -3 -B -` 从 workflow 提取并原样运行唯一 PowerShell here-string 中的 Python。`GITHUB_WORKSPACE` 为本 worktree，`RUNNER_TEMP` 与 `GITHUB_OUTPUT` 位于新建的临时目录。逐服务设置 `CANVAS_IMAGE`、当前 SHA 和模拟 `v1.0.6`/`1.0.6`。三个目录分别核对根文件、唯一服务 crate、workspace 及锁身份。此操作绕过调度、版本解析和许可证步骤，未创建 tag。 |
| Canvas context 负对照 | 在已生成的 `canvas-engine` context 根下写入中性名称 `unapproved-payload.txt` 后重跑同一段 Python。脚本成功且报告 10 个文件，该文件仍存在。普通 hosted runner 上未观察到已有目录污染。 |
| 独立 Canvas Cargo 锁 | 在每份真实生成目录运行 `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu`，Cargo 1.99.1 三次均退出 101，明确要求更新 `Cargo.lock`。在另外三份临时副本用同命令去掉 `--locked` 让 Cargo 复算，锁条数变为 102、102、113，未新增包身份，依赖边发生调整。随后三份 `--locked` 检查均通过。源码锁文件没有参与写入。 |
| 实际 K3s context | `py -3 -B scripts/automation/star_k3s_context.py --repo . --output <fresh-temp-dir>`，使用固定的 Cargo 1.99.1，退出 0。333 文件，35 个 workspace members，36 个本地锁包，405 条总锁记录。补充包 `platform-messaging-proto` 排除在 workspace members 之外，其四个文件随 context 复制。实际 Dockerfile 有 36 条定向 `COPY`，无整树 `COPY`。复算同一摘要 `32c42de70b2df291aa2d46cac02420365613655bab945e2b882333c3e5204e1b`，实际文件路径和锁包名称均无 game/legacy 命中。对生成目录执行上述 `--locked --offline` Cargo metadata，退出 0。 |
| 静态 manifest 与宿主脚本 | `kubectl kustomize deploy/k3s-local` 退出 0，输出 3 份 Deployment 和 4 个 image 引用。PowerShell AST parser 解析 `scripts/automation/multipass_k3s_gitops.ps1` 无错误。 |
| 宿主只读状态 | `multipass list --format json` 返回一个 `Stopped` 实例。`docker info --format '{{.ServerVersion}}'` 在宿主返回 `29.9.2`，该 daemon 未用于镜像构建。 |
| 复核范围 | 直接复核 Git 差异和报告。该隔离树的 `.codegraph` 只有 `.gitignore`，CodeGraph 返回未初始化，故以固定 checkout 的工作流、源码和执行结果作为证据。此次改动仅为审计报告。 |

Canvas 暂存目录的复算摘要为已排序的路径到文件 SHA-256 映射的 JSON 摘要，用于比对本次 Windows checkout。它不等于 BuildKit 传输摘要或 OCI digest。

| 服务 | 文件数 | 过滤锁记录 | 暂存摘要 SHA-256 |
|---|---:|---:|---|
| `canvas-engine` | 9 | 119 | `a88d321a8e121533cb662a0c381b1908ea6cedb2cfe2854ca4bf5c5d4917f43b` |
| `domain-canvas` | 9 | 119 | `926a34f5bd6f8870a6e16c29d28f4f1ed61d98b98150effc287d3bf48680ea89` |
| `canvas-realtime` | 9 | 136 | `37838b0cd5b092257370d817dc09c1d3e7b8821eac3d66bb540bb4b005bbd238` |

## §3 已知缺口与最小修复

| 优先级 | 缺口 | 后续最小修复与验收 |
|---|---|---|
| 发布阻断 | Canvas license 门禁会先于 context 阶段拦截。 | 在许可证责任 lane 核对首方许可证决定、Cargo workspace metadata 和根 LICENSE，再通过现有一致性门禁。单独核对最终第三方 SPDX/NOTICE 闭包。 |
| 发布阻断 | 从全 workspace 锁依赖边截取的三份锁，对独立服务都不是 Cargo 接受的 `--locked` 输入。 | 保留按服务筛选和锁定构建。以原锁作为种子，让固定版本 Cargo 在独立 workspace 内重算，校验 registry 身份和 checksum 均来自原锁、仅有所选服务本地包，并把 `cargo metadata --locked` 加入实际 context 门禁。现有 K3s 生成器已有同类实现可核对。 |
| 高 | Canvas context 复用已存在的目录，门禁允许无敏感命名的额外根文件。K3s 生成器拒绝已存在的输出，但会复制批准 crate 下的普通文件，路径检查不能判别中性文件名下的实际内容。 | Canvas 要求新建空目录，校验输出清单只含明确根文件和该服务的 manifest/source，添加已复现的额外文件回归。K3s 以已审查 checkout 生成并记录实际文件清单，明确 build/runtime 所需文件，拒绝不在发布清单内的附带文件。 |
| 高 | README 仍提供仓库根 context 和无 guest 的旁路。废弃 Helm chart 仍被 staging Application 和维护脚本引用。 | 更新 README 和 WBS，给出指定现有运行中 guest 的支持入口及单独生成 context 的命令，移除直接仓库根构建指令。停用 Helm 活跃引用，或在已有受验 digest 清单后显式切换。复核 dev/staging 实际引用与根 README。 |
| 高 | 没有 BuildKit 传输、镜像、缓存及独立 SBOM 内容验收。 | 在同一经批准源码上捕获并审查实际传输文件清单。对三份 Canvas 发布镜像及单独的本地 API 镜像核验最终文件系统、digest、依赖和 SPDX/NOTICE。审计 BuildKit 缓存及 provenance/SBOM 绑定。若拟分发本地 API 镜像，先固定 Debian runtime 及 apt 依赖来源，并确认产物许可材料。 |
| 高 | Multipass 只有停止态 inventory。实验 Stage 2 允许整仓 privileged mount，GitOps dev 入口包含本地 tag 和浮动依赖。 | 使用明确选择并授权的现有 guest，在测试容量约束内验收 API/节点/CRI、归档和镜像 digest、rollout、health 与回滚。若升级宿主实验为支持路径，先替换整仓挂载为限路径 bridge，并固定、核验部署清单里的每个镜像。 |
| 待验 | Canvas digest 模板、ServiceMonitor 与 GitOps 晋级链路缺集群证据。 | 用真实发布证明渲染并核验三份 digest，核对目标集群 CRD 和镜像 ID，执行 rollout 和回滚验证。将集群层结果与 Star 原生 Run 绑定、owner 预留、sandbox readiness 和多集群性能另行记账。 |

此次没有执行镜像构建、缓存导出、镜像扫描、GHCR 推送、真实证明核验、Helm 渲染、VM 生命周期变更或集群部署。两套真实过滤目录及本地 Kustomize 结果仅足以支持上述局部判断。

## §4 子代理失败接手清单

本隔离审计 lane 未派生子代理，无失败接手项。K3s symlink 测试保留已知跳过，junction 与失败关闭测试结果单列。发布与部署待验事项交协调者按 §3 排序。

## §5 守门规则

1. 以同一源码 SHA、tag、镜像 digest 串联发布证据。
2. 发布只允许批准的三份 Canvas 服务镜像。
3. 每份 Canvas workspace、锁文件及 context 仅含所选服务的本地产品代码。
4. 以固定 Cargo 工具链验证生成后的锁文件可供 `--locked` 使用。
5. 校验实际文件清单、包身份和 checksum。
6. 拒绝 game、legacy、未批准工作区源、symlink 和 junction。
7. 新建空输出目录，并拒绝未批准的附带文件。
8. 分别审查实际 BuildKit 输入、缓存、镜像和部署制品。
9. 发布许可元数据、LICENSE、第三方 SPDX/NOTICE 与实际制品一致。
10. 消费作业须验证 digest 绑定的证明及实际 SBOM 内容。
11. 本地 API 只用生成的逐 crate context 并绑定明确 guest。
12. Multipass 创建遵守独占锁、两实例上限和保留约束。
13. Worktree 进入 guest 只走显式、限路径、可审计的 bridge。
14. GitOps 与部署清单只引用已验证的镜像与受支持入口。
15. Helm 占位在现行交付与维护路径中保持退役。
16. 用集群验收记录 API/节点/CRI、rollout 与回滚，原生 Run/Runtime 能力另作验收。

## §6 签字栏

| 角色 | 审核范围 | 记录 |
|---|---|---|
| 架构 | Star-only 范围与原生边界 | 架构师 (Mavis 接手 agent per DEC-008)，审计接受，发布待验 |
| SRE Lead | OCI、集群和回滚证据 | Mavis 接手审核，运行验收待办 |
| 平台 | Cargo、过滤与 Multipass 入口 | Mavis 接手审核，过滤结果已核实 |
| 评审主持 | 基线、范围与差异核对 | Mavis 接手审核，提交限本审计报告 |
| PM | 发布状态和待办 | Mavis 接手审核，稳定发布保持未放行 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容与触发 |
|---|---|---|---|
| v0.1 | 2026-11-06 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据当前 dev 审计任务记录真实 Canvas/K3s 过滤目录、独立锁检查、README/Helm 入口差异与发布缺口。 |
