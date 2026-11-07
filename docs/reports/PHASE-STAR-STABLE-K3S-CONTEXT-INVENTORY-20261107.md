# PHASE-STAR-STABLE-K3S-CONTEXT-INVENTORY-20261107

> **阶段**：STAR-STABLE-W7 / K3s-local build context inventory
> **状态**：静态构建上下文核查已完成；独立审查与发布制品验收未完成
> **基线**：`dev@6c6dfa9d79acb4198d4f1071c68503363fe10439`
> **生成上下文**：`C:\Users\leo19\AppData\Local\Temp\star-k3s-context-inventory-dev-6c6dfa9d-20261107`

## §0 目的

为 K3s-local 容器构建上下文建立可复核的逐文件证据，检查过滤后的目录、Cargo workspace 与 lockfile 以及生成 Dockerfile 的显式 crate 复制范围。本阶段只证明已生成的 build context 及其静态映射；不将其描述为已构建或已发布的容器制品。

上下文生成器的既有报告为 **334 files**、摘要 `e894ebd1e5cd4dcc76a44bc80d32e269827e93dd30254f693eb38fd8ca020100`。本阶段为 334 个文件另行生成排序 JSONL 清单，并逐文件比对实际长度和 SHA-256。生成器摘要作为既有记录，不将其说成由本报告中的 per-file 清单摘要算法重算所得。

## §1 改动与证据矩阵

| 项目 | 检查与证据 | 结果 | 范围说明 |
|---|---|---|---|
| 逐文件清单 | [`STAR-K3S-CONTEXT-INVENTORY-20261107.jsonl`](data/STAR-K3S-CONTEXT-INVENTORY-20261107.jsonl)：相对路径、字节数、SHA-256；按相对路径排序 | 334 行；路径有序；与生成目录逐文件核对后长度或哈希不符为 0 | 仅涵盖指定临时 build context |
| Dispatcher brief 记录 | `WBS-DISPATCH-STAR-STABLE-K3S-CONTEXT-INVENTORY-20261107.jsonl` | 已记录 `action=brief`，输出指向本 lane brief，`comments_count=0` | 沿用 dispatcher API 已写入的 append-only 记录；本阶段不伪造额外 dispatcher 事件 |
| 文件路径与链接 | 清点生成目录中的文件、路径名称和链接 | 334 个普通文件；符号链接 0；包含 `game` 或 `legacy` 的文件路径 0 | 不以源码注释中的词语作为业务载荷判据 |
| Context 根目录 | 对实际生成目录进行枚举 | `.dockerignore`、`Cargo.lock`、`Cargo.toml`、`LICENSE`、`crates/`、`deploy/`、`rust-toolchain.toml` | 记录观察结果，不据此推断最终镜像内容 |
| Cargo workspace | 解析生成后的根 `Cargo.toml` | 35 个 workspace member；exclude 为 `crates/platform-messaging-proto` 与 `target` | `platform-messaging-proto` 作为本地路径依赖被 Dockerfile 显式复制 |
| Cargo.lock 与本地依赖 | 解析生成后的 `Cargo.lock` 及根清单和 36 个复制 crate 清单 | 405 个 lock package；package 名称含 `game`/`legacy` 为 0；静态发现的 68 条本地依赖路径均解析到包含 `Cargo.toml` 的目录 | 未执行 Cargo 解析、编译或依赖下载 |
| 生成 Dockerfile | 检查 `deploy/k3s-local/Dockerfile` | 36 条 `COPY crates/<crate>/ crates/<crate>/`，与 35 个 workspace member 加 1 个 supplemental excluded local crate 完全对应；未发现 `COPY crates ./crates`；运行阶段显式复制 `star-api-rest` 二进制 | Dockerfile 文本检查不等价于 Docker build 或镜像层审计 |
| Docker 忽略规则 | 检查 context 内 `.dockerignore` | 规则排除 `.git`、`target`、`node_modules` | 过滤范围结论来自已生成目录的实际逐文件盘点，不单独依赖 `.dockerignore` |

## §2 验证摘要

使用临时、只读的 Python 标准库检查代码（未另存辅助脚本，以保持登记文件范围）：`pathlib` 枚举文件，`hashlib.sha256` 对照 JSONL 逐文件哈希和字节数，`tomllib` 解析 Cargo 清单与 lockfile，`re` 检查 Dockerfile COPY 语句及依赖路径。通过 RTK 调用该检查后，汇总输出如下：

- 实际普通文件 334；清单记录 334；路径排序正确；逐文件长度/哈希不符 0。
- 符号链接 0；文件路径名含 `game`/`legacy` 的条目 0。
- workspace member 35；Cargo.lock package 405；package 名称含 `game`/`legacy` 的条目 0。
- 明确 crate COPY 路径 36，集合与 35 个 workspace member 及 `platform-messaging-proto` 一致；宽泛 `COPY crates ./crates` 检查为 false。
- 本地路径依赖检查覆盖根 Cargo 清单与 36 个显式复制 crate 清单，发现 68 条路径引用，未解析引用 0。
- `star-api-rest` 运行二进制有显式 builder 到 runtime stage 的 COPY。

本阶段没有运行 Cargo、Docker、Multipass、测试、部署或回滚操作。以上静态结果没有验证编译可用性、依赖供应链、基础镜像内容或运行时行为。

## §3 已知缺口

以下验收状态均为 **NOT RUN / OPEN**：

1. Docker build、最终镜像清单、镜像 layer 与实际文件内容审计。
2. SBOM、provenance、签名及其制品 digest 绑定核验。
3. Multipass guest 与 K3s 中的镜像导入、部署和真实运行验证。
4. K3s rollout、故障注入、回滚及恢复验证。
5. 最终镜像和发布载荷仅含获批 Star 内容的证明。

因此，这份清单只能支持“指定生成 build context 的路径与文件哈希已盘点”，不能支持“容器镜像已证明只含 Star 内容”或“稳定版发布门已通过”。

## §4 子代理失败接手清单

| 工作项 | 状态 | 接手/后续动作 |
|---|---|---|
| 本 lane 逐文件静态盘点 | 已完成，待独立审查 | 协调者复核 inventory、workspace/COPY 映射及本报告结论 |
| 独立 reviewer lane | 报告编写时仍在执行；未收到结论 | 协调者收到后记录 review disposition；不得将当前结果标记为独立审查通过 |
| Docker / SBOM / 签名 / K3s 验收 | 未执行 | 后续稳定版阶段按发布 WBS 建立独立运行证据；保持当前状态 OPEN |

无失败 worker 产物被并入。本报告和清单由本 lane 直接从已生成 context 整理，不触及保留中的候选 worktree。

## §5 守门规则

1. 范围固定为本 brief、阶段报告、本 dispatch JSONL 和逐文件数据 JSONL；不修改稳定版 WBS 主文件。
2. 不触碰 `main`、远端、其他分支/worktree 或 `D:\Star` 用户文件；保留中的 Rust candidate 及其文件不读取、不覆盖、不清理。
3. 不运行 Cargo、Docker、Multipass、测试、部署或回滚；本阶段只做生成 context 的只读静态核查。
4. 只对实际检查的生成 context 作结论；不把路径名称检查扩展为对最终容器或整个产品仓库的保证。
5. `platform-messaging-proto` 虽位于 workspace exclude，仍因已解析的本地依赖而被显式复制；不得将其误报为未登记 crate，也不得把该事实扩大为全部镜像依赖均已审计。
6. 仅由 Dockerfile 文本与 build context 静态证据，不宣称 build reproducibility、供应链可信、镜像安全或 K3s 可运行。

## §6 签字栏

本阶段签字表示 review/责任状态，不授予稳定版最终发布批准。

| 角色 | 状态 |
|---|---|
| 架构审核 | 待独立审查 |
| SRE Lead | 未签；部署/回滚验收未运行 |
| 平台审核 | 待协调者复核 |
| 评审主持 | 待独立 reviewer 返回 |
| PM / accountable release owner | Ulysses 保持最终稳定版责任；本报告不构成 release approval |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次整理 K3s-local 已生成 build context 的逐文件清单和静态依赖/COPY 证据；明确镜像、供应链及 K3s 验收均未运行。 |
