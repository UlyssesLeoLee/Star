# PHASE — Star Stable W7 local K3s context filter

**阶段状态：** Star-only context 过滤实现与静态审阅已通过 fast-forward 集成到本地 dev，集成提交 `42082bccd9ae6fd24cf609f5697b2b964a568002`。Rust 1.99.0 基线、Multipass 镜像构建、最终镜像审查与部署未验证；W7 和 stable release 仍 OPEN / 不可发布。
**工作基线：** 原始实现基于 `dev@2a57fa0d329be830735d6657677236e195673362`；集成前 dev 基线 `646e1e3e354be2eddd3345c928adf8c591d5ac49`；W7 集成提交与当前 dev tip `42082bccd9ae6fd24cf609f5697b2b964a568002`。
**发行范围：** Desktop、Web/Vercel、三份 Star Canvas 服务镜像；本 lane 只处理供本地 K3s 使用的 `star-api-rest` build context。

**当前审阅结论：** 所修改的本地 K3s API build path 已从宽泛 crates 输入改为 Cargo 解析闭包生成的逐 crate context；该静态改动已集成到 dev。`deploy/k3s-local` 仍属开发/测试路径，不加入本次稳定版镜像矩阵。D2 worktree 中同路径未提交的 README 草稿保持原样，不将其视为已接受的支持范围决定。

## §0 目的

为本地 K3s API 镜像提供一个 fail-closed 构建路径：context 仅含 `star-api-rest` 的选定 Cargo 本地依赖闭包，Dockerfile 为每个本地 crate 生成单独的 `COPY`，并且可显式将过滤后的 context 传入指定 Multipass guest 内执行构建。本阶段已在 Rust 1.99.1 诊断工具链运行真实 Cargo metadata、生成并审查 333 文件 context，且对输出 lock 运行 locked metadata；未运行 pinned 1.99.0、容器构建或集群部署，不能据此关闭 W7。

## §1 改动矩阵

| 文件/对象 | 改动 | 证据与边界 |
|---|---|---|
| `deploy/k3s-local/Dockerfile` | 锁定 Rust builder image 到 1.99.0；把宽泛 crates 复制替换为由生成器展开的 crate COPY 标记；构建使用 `--locked`。 | 此文件是 context generator 模板，直接用仓库根目录作为构建 context 不受支持。实际 COPY 列表仅在生成 context 中产生。 |
| `scripts/automation/star_k3s_context.py` | 新增 Cargo metadata 解析、普通/build 依赖闭包选择、workspace 白名单、开发依赖移除、仓库路径与符号链接检查、确定性 context 摘要和显式 Multipass guest build。对 Cargo 必须读取但未启用的可选 path crate，只复制经验证的 manifest 依赖并排除在 workspace members 之外。 | 输出目录中以源 `Cargo.lock` 为种子让 Cargo 离线生成新锁文件，再逐项校验 registry package identity/checksum 和本地 package identity；拒绝新增 registry 身份、未批准本地 crate 和 game/legacy 包。Multipass 构建使用 argv 调用 `transfer` 与 `exec`，不调用宿主 Docker；传输前校验生成时摘要和归档内容一致，不创建或清理 VM。 |
| `scripts/automation/test_star_k3s_context.py` | 增加 11 项标准库单元测试。 | 覆盖依赖闭包、开发依赖排除、game 依赖拒绝、缺失节点/manifest、路径越界、输出稳定性、源 Cargo.lock identity/checksum 漂移拒绝、context 篡改拒绝、仓库输入符号链接拒绝及 Multipass 命令构造。Windows 未授予创建符号链接的权限，因此该项被跳过；构造图为 fixture，不是生产依赖证据。 |
| 本 lane brief/report/dispatch/WBS | 记录范围、验收、实际测试、当前 dev 基线和门禁状态。 | 本次同步主 WBS 只记录本 lane 的静态实现与证据；不表示 W7、W2 或稳定版发布门关闭。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context -v`（`RUSTUP_TOOLCHAIN=1.99.1`） | PASS：10 passed、1 skipped（当前 Windows 主机无创建符号链接权限）。 |
| `rtk proxy py -3 -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py` | PASS。 |
| `rtk proxy py -3 scripts/automation/star_k3s_context.py --help` | PASS：生成 context 与显式 Multipass build 参数可见。 |
| `rtk proxy git diff --check` | PASS（lane 工作树差异）；CypherGraph Guardian `--changed --stamp` PASS（2 个 Python source files）。 |
| Multipass 命令 | 仅在 mock 中核验命令 argv 指向指定 guest 的 `docker build`；未调用真实 Multipass CLI。Canonical CLI 语法参考已记录在 brief。 |
| 源 workspace Cargo metadata | PASS（诊断）：Rust 1.99.1 下 `--locked --offline --filter-platform x86_64-unknown-linux-gnu` 完成；Rust 1.99.0 本机 toolchain 仍无法启动 `rustc -vV`，因此 pinned baseline 未验证。 |
| 生成 context / lock | PASS（诊断）：生成 333 个文件，context SHA-256 `e8316e07d4a7e5851f2cc94dde9792b2306ca7f6bf4276c4b705cb5b5b012fdc`；输出无 game/legacy 输入。生成 context 的 Cargo lock 再通过 `cargo metadata --locked --offline --filter-platform x86_64-unknown-linux-gnu`。 |
| Multipass image build / K3s rollout | NOT RUN；未创建 VM、镜像或部署。 |

## §3 已知缺口

- Rust 1.99.1 诊断已证明真实 workspace closure、生成文件清单和输出锁文件可被 Cargo 校验；这不能替代 Rust 1.99.0 pinned baseline，需修复/恢复该 toolchain 后重跑相同门禁。
- 未验证生成 Cargo workspace 是否在容器内通过 `cargo build --release --locked -p star-api-rest`；宿主 Linux 镜像 tag、目标平台和网络/依赖缓存也未作真实构建验证。
- 未构建并检查 Multipass guest 中的最终 image layers、digest、SBOM/provenance，也未执行 K3s rollout、健康检查或回滚。
- W7 的完整 Star-only payload 门仍 OPEN；Canvas OCI workflow 的静态筛选证据不能外推到此 local K3s build path。当前只证明生成的 Cargo 输入闭包排除 game/legacy 项，不证明容器镜像最终层、SBOM/provenance 或活动部署载荷已合格。
- `D:\\orcaWork\\Star\\star-delivery-d2-multipass-maintenance-20261031` 中 `deploy/k3s-local/README.md` 的未提交退役草稿仍由原 worktree 保留且未合入；本 lane 不修改该草稿，也不把它当作已批准的产品支持决策。生成器作为开发/测试路径的静态防护可独立集成，但该路径的长期支持/退役决定仍待 D2 设计审查。
- stable release W0–W9 均须由主 WBS 与实际目标平台证据另行审核，本 lane 不选择版本、不发布、不推送。

## §4 子代理失败接手清单

W7 lane 曾由子代理实施。首次派发前没有落地 dispatcher brief；brief 与 dispatch 记录在派发后补写，不能满足“先落 brief 再派发”的时序门禁。本报告保留该事实，不回填或声称历史合规。协调者已独立复核代码并删除 workspace 清单函数中 `return` 后的旧不可达实现，补强根输入 symlink 校验、context 摘要与归档一致性校验，并重跑本报告列出的验证。该流程缺口仍记录为治理偏差，不影响对未执行生产门的开放状态判断。

## §5 守门规则

- 源 Cargo metadata 固定 `--locked --offline` 与 Linux target filter；输出锁文件由 Cargo 在隔离 context 中离线生成，并验证 registry identity/checksum 均来自源锁、本地 package identity 均来自批准的闭包；任何无法识别、缺失或越界的图节点均拒绝生成。
- 只遍历 normal/build edge；dev dependency table 从生成 package manifest 中移除。
- 本地 crate 只能由解析到的包清单控制。根 Cargo workspace members 与 Cargo.lock 都经筛选；仅 `.dockerignore` 不构成验收证据。
- 生成文件中必须没有 game/legacy package/path、任意 symlink、未解析 marker 或 broad crate copy；交给 Multipass 前再次扫描以防 context 被修改。
- 镜像构建只能通过用户显式选择的 Multipass 实例由 `multipass exec` 在 guest 内执行；不调用宿主 Docker、不创建/销毁实例、不推送或发布镜像。
- fixture 测试与模拟 Multipass argv 不是实际 dependency/build/deploy 证明；symlink test 在此 Windows 主机因 WinError 1314 被跳过；W7 和 release 保持 OPEN。
- 与主 WBS 的冲突或更优设计需在父级集成前复核；本次只在已登记 WBS 路径内同步 W7 状态，不修改 D2 文件或 `D:\Star` 用户内容。
- D2 脏 README 草稿与该 API 路径的长期支持状态有关，但不要求放宽 Star-only 输入过滤；保留草稿原样，并将支持/退役裁决与本次安全构建上下文改动分开记录。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 协调者已完成静态代码、清单范围和 fixture 结果复核；Cargo 图与真实 guest build 仍待验证。 |
| SRE Lead | 不适用：未运行 guest build、集群部署、容量或恢复验收。 |
| 平台 | 不适用：未运行真实 Cargo、镜像构建或制品验证。 |
| 评审主持 | 协调者完成静态整合审查并快进集成到本地 dev@42082bcc；W7 的生产验收仍未完成。 |
| PM | 不适用：未修改发行范围、版本或发布授权。 |

**发布审批：** 未申请；W7 与 stable release 仍不可发布。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis lane 实施，协调者复核待完成 | 实现过滤后的本地 K3s build context、Multipass guest 构建调用路径与 fixture 测试；准确记录 Cargo/镜像/集群验证缺口，不关闭 W7。 | stable release WBS 审计发现 local K3s Dockerfile 使用广泛 crates context。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 协调者复核 | 修复不可达的旧清单实现；增加仓库输入 symlink 检查、生成摘要与 Multipass tar 清单/内容复核；测试增至 10 项并如实标记 1 项因主机权限跳过。披露派发 brief 时序偏差；生产构建与部署门保持 OPEN。 | 协调者代码审查发现静态质量和 context 变更可检测性缺口。 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 协调者复核 | 移除不完整的自制 Cargo lock 依赖投影；改为在隔离输出内由 Cargo 离线生成 lock，并验证源锁 identity/checksum、本地包白名单。Rust 1.99.1 下真实 workspace metadata、333 文件 context 和输出 `--locked --offline` metadata 通过；明确 Rust 1.99.0、Multipass build 与 K3s rollout 仍未验证。 | 真实 Cargo 工作区发现按单平台 resolve graph 直接裁剪 lock 不能通过生成 context 的 locked metadata。 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 协调者复核 | 增加源 lock package identity/checksum 漂移拒绝测试；当前单测 10 passed、1 skipped，代码图守门通过。更新 brief/report 的锁文件算法和证据边界；release context digest 不变。 | 独立复核要求覆盖 lock 校验拒绝路径并同步测试数量。 |
| v0.5 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 协调者复核 | 记录 D2 活动 worktree 的未提交 README 将 `deploy/k3s-local` 标为退役，与 W7 API image candidate 的支持范围冲突；W7 暂不集成到 dev，等待 owner 完成范围裁决。 | D2 worktree 只读检查发现同一路径的活动 owner 改动。 |
| v0.6 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将候选 rebase 到 `dev@646e1e3e` 后复核：`deploy/k3s-local/Dockerfile` 不再包含宽泛 crates COPY，生成器仅从解析的 Star API 本地依赖闭包组装 context；同步主 WBS。D2 未提交 README 草稿继续保留，API 路径支持/退役仍需独立裁决。Rust 1.99.0、Multipass 镜像构建、最终镜像/SBOM/provenance、K3s rollout/rollback 未验证，W7 与稳定版保持 OPEN/NO-GO。 | 用户要求 Docker/build context 不携带 game 等遗留内容；对候选的当前 dev 集成审查。 |
| v0.7 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录本 lane 的 8 个登记文件已从 `dev@646e1e3e` fast-forward 到 `dev@42082bcc`；W6 审阅文档已在 dev ancestry 中，因此未重复 cherry-pick。复验 11 项 context tests：10 passed、1 skipped（WinError 1314）；CodeRabbit CLI 因操作系统权限错误不可用，协调者完成本地静态审查。Rust 1.99.0、真实 Multipass image build、最终 payload/SBOM/provenance、K3s rollout/rollback 仍未通过，W7 和稳定版保持 OPEN/NO-GO。 | W7 lane 成功快进集成后补记 dev 结果与未关闭门禁。 |
