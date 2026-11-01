# PHASE — Star Stable W7 local K3s context filter

**阶段状态：** 实现与夹具验证通过；真实 Cargo 图、Multipass 镜像构建与部署未验证。W7 和 stable release 仍 OPEN / 不可发布。
**工作基线：** `dev@2a57fa0d329be830735d6657677236e195673362`。
**发行范围：** Desktop、Web/Vercel、三份 Star Canvas 服务镜像；本 lane 只处理供本地 K3s 使用的 `star-api-rest` build context。

## §0 目的

为本地 K3s API 镜像提供一个 fail-closed 构建路径：context 仅含 `star-api-rest` 的选定 Cargo 本地依赖闭包，Dockerfile 为每个本地 crate 生成单独的 `COPY`，并且可显式将过滤后的 context 传入指定 Multipass guest 内执行构建。本阶段没有运行真实 Cargo metadata、容器构建或集群部署，不能据此关闭 W7。

## §1 改动矩阵

| 文件/对象 | 改动 | 证据与边界 |
|---|---|---|
| `deploy/k3s-local/Dockerfile` | 锁定 Rust builder image 到 1.99.0；把宽泛 crates 复制替换为由生成器展开的 crate COPY 标记；构建使用 `--locked`。 | 此文件是 context generator 模板，直接用仓库根目录作为构建 context 不受支持。实际 COPY 列表仅在生成 context 中产生。 |
| `scripts/automation/star_k3s_context.py` | 新增 Cargo metadata 解析、普通/build 依赖闭包选择、Cargo workspace/lock 过滤、开发依赖移除、仓库路径与符号链接检查、确定性 context 摘要和显式 Multipass guest build。 | Multipass 构建使用 argv 调用 `transfer` 与 `exec`，没有 host-side Docker 调用；传输前校验生成时摘要和 tar 内容一致，只在用户传入 VM 名和本地 tag 后运行，不创建或清理 VM。 |
| `scripts/automation/test_star_k3s_context.py` | 增加 10 项标准库单元测试。 | 覆盖依赖闭包、开发依赖排除、game 依赖拒绝、缺失节点/manifest、路径越界、输出稳定性、context 篡改拒绝、仓库输入符号链接拒绝及 Multipass 命令构造。Windows 未授予创建符号链接的权限，因此该项被跳过；构造图为 fixture，不是生产依赖证据。 |
| 本 lane brief/report/dispatch | 记录范围、验收、实际测试与门禁状态。 | 本报告不更新主 WBS，也不表示 W7、W2 或稳定版发布门关闭。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context -v` | PASS：9 passed、1 skipped（当前 Windows 主机无创建符号链接权限）。 |
| `rtk proxy py -3 -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py` | PASS。 |
| `rtk proxy py -3 scripts/automation/star_k3s_context.py --help` | PASS：生成 context 与显式 Multipass build 参数可见。 |
| `rtk proxy git diff --check` | PASS（lane 工作树差异）；`cgg_guard.py --changed --stamp` PASS（2 个 Python source file）。 |
| Multipass 命令 | 仅在 mock 中核验命令 argv 指向指定 guest 的 `docker build`；未调用真实 Multipass CLI。Canonical CLI 语法参考已记录在 brief。 |
| Cargo metadata / 实际 context | NOT RUN：启动 Cargo 会在 `rustc -vV` 因 1.99.0 Windows MSVC toolchain 不适用而失败；本阶段未等待或重试 Cargo。 |
| Multipass image build / K3s rollout | NOT RUN；未创建 VM、镜像或部署。 |

## §3 已知缺口

- 主机 Cargo metadata 还未在可工作的 Rust 1.99.0 toolchain 上运行；真实 workspace 的 local closure、过滤锁与生成文件列表尚无实证。
- 未验证生成 Cargo workspace 是否在容器内通过 `cargo build --release --locked -p star-api-rest`；宿主 Linux 镜像 tag、目标平台和网络/依赖缓存也未作真实构建验证。
- 未构建并检查 Multipass guest 中的最终 image layers、digest、SBOM/provenance，也未执行 K3s rollout、健康检查或回滚。
- W7 的完整 Star-only payload 门仍 OPEN；Canvas OCI workflow 的静态筛选证据不能外推到此 local K3s build path。
- stable release W0–W9 均须由主 WBS 与实际目标平台证据另行审核，本 lane 不选择版本、不发布、不推送。

## §4 子代理失败接手清单

W7 lane 曾由子代理实施。首次派发前没有落地 dispatcher brief；brief 与 dispatch 记录在派发后补写，不能满足“先落 brief 再派发”的时序门禁。本报告保留该事实，不回填或声称历史合规。协调者已独立复核代码并删除 workspace 清单函数中 `return` 后的旧不可达实现，补强根输入 symlink 校验、context 摘要与归档一致性校验，并重跑本报告列出的验证。该流程缺口仍记录为治理偏差，不影响对未执行生产门的开放状态判断。

## §5 守门规则

- Cargo metadata 固定 `--locked --offline` 与 Linux target filter；任何无法识别、缺失或越界的图节点均拒绝生成。
- 只遍历 normal/build edge；dev dependency table 从生成 package manifest 中移除。
- 本地 crate 只能由解析到的包清单控制。根 Cargo workspace members 与 Cargo.lock 都经筛选；仅 `.dockerignore` 不构成验收证据。
- 生成文件中必须没有 game/legacy package/path、任意 symlink、未解析 marker 或 broad crate copy；交给 Multipass 前再次扫描以防 context 被修改。
- 镜像构建只能通过用户显式选择的 Multipass 实例由 `multipass exec` 在 guest 内执行；不调用宿主 Docker、不创建/销毁实例、不推送或发布镜像。
- fixture 测试与模拟 Multipass argv 不是实际 dependency/build/deploy 证明；symlink test 在此 Windows 主机因 WinError 1314 被跳过；W7 和 release 保持 OPEN。
- 与主 WBS 的冲突或更优设计需在父级集成前复核；不得修改 WBS、D2 文件或 `D:\Star` 用户内容。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 协调者已完成静态代码、清单范围和 fixture 结果复核；Cargo 图与真实 guest build 仍待验证。 |
| SRE Lead | 不适用：未运行 guest build、集群部署、容量或恢复验收。 |
| 平台 | 不适用：未运行真实 Cargo、镜像构建或制品验证。 |
| 评审主持 | 协调者静态整合审查完成；W7 的生产验收和 dev 集成仍未完成。 |
| PM | 不适用：未修改发行范围、版本或发布授权。 |

**发布审批：** 未申请；W7 与 stable release 仍不可发布。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis lane 实施，协调者复核待完成 | 实现过滤后的本地 K3s build context、Multipass guest 构建调用路径与 fixture 测试；准确记录 Cargo/镜像/集群验证缺口，不关闭 W7。 | stable release WBS 审计发现 local K3s Dockerfile 使用广泛 crates context。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 协调者复核 | 修复不可达的旧清单实现；增加仓库输入 symlink 检查、生成摘要与 Multipass tar 清单/内容复核；测试增至 10 项并如实标记 1 项因主机权限跳过。披露派发 brief 时序偏差；生产构建与部署门保持 OPEN。 | 协调者代码审查发现静态质量和 context 变更可检测性缺口。 |
