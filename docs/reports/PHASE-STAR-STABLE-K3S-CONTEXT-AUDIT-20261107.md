# PHASE-STAR-STABLE-K3S-CONTEXT-AUDIT-20261107

> 状态：静态源代码审计完成；生成构建上下文、构建最终镜像及 guest 部署均未运行。
> 基线：`dev@22b92756ad762f27297c493760d95435ef98f2d1`。
> 范围：仅 K3s local image build-context 审计，不修改生成器、Dockerfile、`.dockerignore`、测试或 WBS。

## §0 目的

核对 `deploy/k3s-local` 是否从 Star Cargo 依赖闭包构造受限构建上下文，并确认这条路径遵循“Multipass 管理 Linux guest、guest 内运行 K3s”的目标。区分静态源代码守卫、实际生成的 context inventory 与最终 image inventory；本报告不把静态代码定义写成已运行或已验收的事实。

## §1 改动矩阵与静态证据

本 lane 只新增/更新四份登记审计文档；没有修改产品代码、配置、测试或 WBS。

| 文件/位置 | 静态证据 | 可支持的结论 | 证据边界 |
|---|---|---|---|
| `deploy/k3s-local/Dockerfile:1-10,12-17` | Rust builder 镜像按 digest 固定；根 Cargo manifest、lock、toolchain 是显式输入；crate `COPY` marker 由生成器替换；模板声明不能直接当 build context；运行时仅复制 `star-api-rest` 二进制到 Debian 镜像。 | 模板设计要求生成显式 crate allowlist，避免宽泛复制整个 `crates` 树。 | 未运行生成器或 Docker build；不能据此证明实际 context 或 image 内容。 |
| `.dockerignore:1-13` | 注释定位为 Canvas context 的纵深防御；忽略 `.git`、`target`、`node_modules`、`star-game`、`game-legacy`。 | 这些规则是额外排除层。 | 当前注释明确描述 Canvas context；不能把它单独当作 K3s local context 的过滤证明，也不能代替 inventory。 |
| `scripts/automation/star_k3s_context.py:163-178,265-415` | 定义禁止路径词 `game`/`legacy`；`select_packages` 从 `star-api-rest` 活跃非 dev Cargo 依赖边解析包闭包，并校验本地依赖。 | 生成器源码定义了 Cargo 闭包选择和禁止域检查。 | 此 lane 没有执行 metadata 查询；当前依赖解析结果未观测。 |
| 同文件 `:549-618,620-724` | 生成过滤 workspace manifest；复制已选本地 crate；从 Dockerfile marker 展开每个 crate 的 `COPY`；收集 Cargo metadata、toolchain、license、`.dockerignore` 和部署 Dockerfile 等显式输入。 | 源码定义了逐 crate staging，而不是 `COPY crates ./crates`。 | 不等于本轮产出了具体文件清单。 |
| 同文件 `:729-829` | `_context_digest` 对排序文件清单计算摘要；`_validate_context` 检查必需文件、拒绝禁止路径、符号链接、异常 root/file、非选中 crate 和意外部署文件，并验证展开后的 Dockerfile 不含 marker 或 broad crate COPY。 | 源码包含生成后复验及 allowlist/摘要守卫。 | 守卫没有在本 lane 实际执行；实际返回的路径集合与 digest 未记录。 |
| 同文件 `:965-1043,1093-1139`；`deploy/k3s-local/build-and-deploy.sh:7-34` | helper 通过 Multipass CLI 把校验后的 context 传入明确指定的既有 guest；在 guest 内调用 Docker build、导出镜像并流式导入 K3s；`crictl images` 检查预期 `star-api-rest:<tag>` 可见后再 apply manifest。入口要求所选 guest 已运行，脚本声明不创建/启动/停止/删除 VM，也不使用宿主 Docker/K3s。 | 当前部署实现是 guest-side build/import/deploy 路径，不依赖 Docker Desktop。 | 源码只要求 guest 预先有 Docker build engine 和 K3s；本 lane 未检查任一实际 VM，也未运行这些步骤。CRI 可见性检查不枚举镜像 filesystem/layers。 |
| `scripts/automation/test_star_k3s_context.py:265-284,316-320,626-663` | 测试定义覆盖解析出的 Star closure、game/legacy 断言、拒绝 game dependency、部署脚本调用生成器、拒绝 context 篡改及未列入文件。 | 仓库存在对应验收断言定义。 | **没有运行测试**；不报告测试通过或失败。 |
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md:25-26,44-45,102,130` | WBS 将 W7 列为容器/部署供应链，W8 列为 Multipass/K3s 测试与两实例验收；既有条目已称 K3s local path 为 development-only，并列出实际 guest build、镜像/SBOM/provenance 与 rollout/rollback 缺口。 | 本审计应补充到 W7 证据记录并保留 OPEN；W8 的最多两实例约束继续适用。 | WBS 未在本 lane 修改；其已有 333-file context 记录是 WBS 记录的先前证据，本 lane 没有重新生成或复核该 inventory。 |

### 三类证据状态

1. **静态守卫：** 已检查其源码定义与测试定义；没有运行测试或 helper。
2. **生成 context inventory：** `NOT RUN`。未调用 `star_k3s_context.py`，本报告没有当前文件数、排序路径清单或摘要。
3. **最终 image inventory：** `NOT RUN`。未构建或导出镜像，未检查 image filesystem/layers，也没有本 lane 产生的 image digest、SBOM 或 provenance。源码中的 K3s `crictl images` 逻辑只确认目标引用可见，不是 image 内容盘点。

## §2 验证摘要

- 源码和现有 WBS 的定向只读审阅：完成。
- `git diff --cached --check`：通过；首次检查发现的两处报告尾随空格已移除。
- dispatch JSONL 解析：通过，2 条记录均可解析。
- Python 测试、Cargo、Docker/Podman build、Multipass VM、K3s、cloud、发布/部署：**按任务边界未运行**。
- 生成 context 及 image inventory：**未运行**，不作通过声明。

## §3 已知缺口

1. 当前 dev 的实际过滤 context inventory、文件数和 digest 未在本 lane 生成。既有 WBS 的 333-file 记录属于先前基线/审计记录，本次未重跑核实。
2. 最终 `star-api-rest` image 的文件、layer、digest、SBOM/provenance 未由本 lane 检查；源码的 `crictl images` 检查只确认该 tag 在 K3s CRI 可见。
3. Multipass guest 的实际 Multipass/K3s/Docker build-engine 版本、guest 容量、cluster identity、Rust 1.99.1 build 结果、rollout/health/rollback 证据均未取证。
4. 当前 helper 的 guest 预置条件是 Docker build engine + K3s；此处确认的是 guest 内工具链路径，不是 Docker Desktop 依赖。
5. stable WBS 文件不在本 lane 写入清单且与受保护候选范围重叠；本次只提出同步建议，不声称修改 WBS。

## §4 子代理失败接手清单

- 本 lane 没有再派发子代理；没有子代理失败或散落产出。
- 后续需要单独授权的执行工作：固定 Rust 1.99.1 生成并留存 context inventory/digest；在明确 Multipass guest 内构建；检查最终 image layers、digest、SBOM/provenance；记录 K3s rollout/health/rollback。执行前必须继续遵守 W8 最多保留两条 Multipass inventory 记录的约束。
- 这些执行项不属于当前静态审计权限，不在本 lane 启动。

## §5 守门规则

1. 本 lane 只写 brief、phase report、review、dispatch 四个登记路径。
2. 不更改 Dockerfile、`.dockerignore`、生成器、测试、Cargo workspace 或 WBS。
3. 不运行测试、Cargo、Docker/Podman、数据库、VM、cloud、build、release、deploy。
4. 不读取、复制或修改受保护 Rust 候选工作树；不碰 main、remote refs 或其他 worktree。
5. 静态源码断言、测试定义、生成 context inventory、final image inventory 分开报告。
6. 只把源码中的拒绝逻辑描述为“已定义”，不写成运行验证通过。
7. `.dockerignore` 是额外过滤层，不是实际 context 内容证明。
8. Docker build engine 位于指定 Multipass guest；不声称需要 Docker Desktop。
9. 当前 context/image 两类实际 inventory 均标为 `NOT RUN`。
10. 按 WBS 将 `deploy/k3s-local` 保持 development-only；不扩充已批准的 release image matrix。
11. W7 保持 `OPEN`，直到实际受控构建、image 内容盘点与供应链材料完成审查。
12. W8 测试最多保留两个 Multipass inventory 实例；本 lane 不创建或 purge VM。
13. 不把源码静态检查或测试定义当作 CI、hosted run、image/SBOM/provenance 证据。
14. WBS 不在当前 manifest 中，因此只提供可执行的 follow-up 建议。
15. 未运行的工作明确标注，不用“通过”“完成闭环”等措辞。

## §6 签字栏

本节仅记录本次文档审计范围，不构成产品、集群或发行批准。

| 角色 | 审核记录 | 边界 |
|---|---|---|
| 架构 | 架构师（Mavis 接手审核，待协调者复核） | 复核 Multipass guest 路径与“只按源证据下结论”的边界。 |
| SRE Lead | SRE Lead（Mavis 接手审核，待协调者复核） | 静态审计记录完成；guest readiness、rollout、health、rollback 未验证。 |
| 平台 | 平台（Mavis 接手审核，待协调者复核） | 静态 context allowlist 逻辑已记录；生成 inventory 与最终 image 未检查。 |
| 评审主持 | 评审主持（Mavis 接手审核，待协调者复核） | 复核本 lane 四文件范围与限制。 |
| PM | PM（Mavis 接手审核，待协调者复核） | WBS follow-up 建议已写入 brief；WBS 未修改，W7 仍开放。 |

修订人：`Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 K3s local context 静态审计；区分源码守卫、未生成的 context inventory 与未检查的 final image inventory；提出 W7/W8 后续验收建议。 | 用户要求发布范围只含 Star 内容，并遵循 Multipass 管理 K3s guest 的目标。 |
