# STAR Stable K3s Build Context Inventory Review

- **基线**：dev@6c6dfa9d79acb4198d4f1071c68503363fe10439
- **审查对象**：C:\Users\leo19\AppData\Local\Temp\star-k3s-context-inventory-dev-6c6dfa9d-20261107
- **审查类型**：独立、静态、限范围的生成构建上下文审查
- **结论**：已检查的上下文路径、crate 清单、Cargo 元数据和 Dockerfile 输入未发现 game/legacy 载荷路径或宽泛 crate COPY；这不构成最终镜像或发行放行。

## 0. 目的

独立核对已生成的 K3s-local build context 是否与 brief 中登记的 334 文件摘要一致，并检查实际上下文中的根目录、crate、Cargo workspace/lock 与生成 Dockerfile 的复制输入。审查以生成后的文件为对象，未仅依赖上下文生成器的 digest 方法，也未把静态结果当作镜像验收证据。

## 1. 审查矩阵

| 项目 | 独立证据 | 结果 |
|---|---|---|
| Context 文件集合 | 单独遍历实际目录，按 POSIX 相对路径排序后计算路径和文件内容 SHA-256 | 334 个文件；摘要与 brief 的 e894ebd1e5cd4dcc76a44bc80d32e269827e93dd30254f693eb38fd8ca020100 一致 |
| 根目录及路径 | 根项为 .dockerignore、Cargo.lock、Cargo.toml、LICENSE、crates/、deploy/、rust-toolchain.toml；检查文件和目录路径段 | 未发现包含 game 或 legacy 的路径 |
| Crate 清单 | 36 个 crate 目录和 36 份 crate Cargo.toml；35 个 workspace member manifest 均存在 | 另有 1 个显式排除的 crate crates/platform-messaging-proto，其目录和 manifest 均在 context 内 |
| Package 与 Lock | 36 个本地 package 均在 Cargo.lock 中有匹配的 name/version；锁文件含 405 个 package | package 名称及锁依赖名未发现 game 或 legacy |
| 本地路径依赖 | 独立解析根 workspace dependencies、各 crate 的 dependencies/dev/build dependencies 与 target-specific 依赖 | 68 条 path dependency：67 指向 workspace member，1 指向显式排除但存在且已复制的 platform-messaging-proto；缺失、越界或其他未列出目标为 0 |
| Docker COPY | 逐条检查生成的 deploy/k3s-local/Dockerfile 的 38 条 COPY | 36 条 crate COPY 均为精确 crate 路径且与 36 个目录一一匹配；没有宽泛或未解析的 crates COPY |
| Runtime 输入 | 检查 multi-stage COPY | 最终阶段只从 builder 复制 star-api-rest 可执行文件；最终镜像未构建 |

## 2. 验证摘要

- 使用 Python 标准库独立枚举实际文件；对每个排序后的 POSIX 相对路径依次写入 UTF-8 路径、NUL、原始文件字节、NUL，再计算 SHA-256。计算结果为 334 文件、e894ebd1e5cd4dcc76a44bc80d32e269827e93dd30254f693eb38fd8ca020100，与已登记摘要一致。
- 36 个 crate 目录均有可解析的 Cargo.toml。35 个 workspace member 全部存在；额外的 crates/platform-messaging-proto 明确列在 workspace exclude 中，但被本地 path dependency 使用、在 context 中有 manifest，并且对应 package/version 存在于 Cargo.lock。
- 检查了 36 个本地 package 的 name/version、Cargo.lock 的 405 个 package 及依赖名称；本地 package 均能在锁文件中匹配，未发现 game 或 legacy package/dependency 名称。
- 在本地 manifest、根 workspace dependencies 和 target-specific dependency sections 中找到 68 条 path dependency。67 条落在 workspace members 中，1 条落在显式排除的 platform-messaging-proto；68 条都指向 context 内存在的 crate manifest。
- 生成 Dockerfile 有 38 条 COPY：一条复制根 Cargo/toolchain 文件、36 条逐 crate COPY、一条从 builder 复制最终二进制。36 个 crate 源路径唯一、完整匹配 context 中的 crate 目录；未发现 COPY crates ./crates、COPY . . 或其他宽泛/未解析 crates COPY。
- 检查路径属性未发现 symlink、Windows reparse point 或无法读取的路径。
- 未运行 Cargo、Docker、Multipass、测试、部署或发布命令；以下均为 **NOT RUN**：最终镜像内容/layer 检查、SBOM、provenance、签名、K3s 部署及回滚。

## 3. 已知缺口

1. 本结论只覆盖 brief 中这一个已生成 context 和实际检查的文件范围。CodeGraph index 状态为 up-to-date，但本会话没有 check_index_coverage 工具；因此不声称已审计仓库所有 Docker 制品、生成路径或 source coverage。
2. Docker builder 镜像 rust:1.99.1-slim-bookworm 固定了 digest；runtime debian:bookworm-slim 仍使用可变 tag。建议在后续发行审查中固定 runtime digest 或记录可复现的镜像解析结果。本 lane 不改 Dockerfile。
3. 未构建最终镜像，因此没有验证最终镜像层、镜像内文件清单、SBOM/provenance、签名或运行时内容。
4. 未在 Multipass/K3s 环境执行部署、健康检查或回滚；VM/cluster 数量、容量和隔离行为不在本次证据范围。
5. platform-messaging-proto 作为 excluded path dependency 被显式纳入 context 和 Dockerfile COPY。当前 manifest、路径与 lock 检查闭合；后续若生成器/依赖图变化，应继续把此例外纳入审查。
6. WBS 主文件未在本 lane 更新，因为该路径受保留候选的重叠保护，且不属于本次登记清单。

## 4. 子代理失败与交接

本 lane 未另行派发子代理，也没有子代理失败。交接给协调者的审查结论是：**静态 build-context 边界通过本次限定检查；稳定发行仍未关闭。** 已检查的 context 范围与限制见 §1—§3；不得把 NOT RUN 的镜像、SBOM、签名或 K3s 运维门描述为通过。

## 5. 守门结果

| # | 守门项 | 结果 |
|---:|---|---|
| 1 | 基线固定为指定 dev commit | PASS |
| 2 | 仅审查登记的生成 context | PASS |
| 3 | 文件数与独立重算摘要 | PASS：334 / digest 匹配 |
| 4 | 根目录输入集合 | PASS：7 个登记根项 |
| 5 | 全部路径的 game/legacy 分段检查 | PASS：0 命中 |
| 6 | Symlink/reparse/unreadable 路径 | PASS：0 |
| 7 | Workspace members 的 manifest 存在性 | PASS：35/35 |
| 8 | Crate 目录与 manifest 一致性 | PASS：36/36 |
| 9 | Package name/version 与 Cargo.lock 对应 | PASS：36/36 |
| 10 | 本地依赖路径闭包 | PASS：68 条均在 context 内 |
| 11 | 显式排除 path dependency 的处置 | PASS：1 个有 manifest、被复制并在 lock 中 |
| 12 | Cargo.lock package/dependency 命名过滤 | PASS：405 packages，0 命中 |
| 13 | Dockerfile COPY 语句总量与分类 | PASS：38 条已逐条检查 |
| 14 | 每个 crate COPY 显式、唯一且完整 | PASS：36/36 |
| 15 | Broad/unparsed crates COPY 禁止项 | PASS：0 |
| 16 | Runtime 只复制最终 Star 二进制；base pin 状态 | PASS（复制边界）；跟踪项（runtime tag 可变） |
| 17 | 最终镜像、SBOM/provenance、签名、K3s 部署/回滚 | NOT RUN；不构成发行放行 |

## 6. 签批栏

本签批仅覆盖 §1—§2 的静态构建上下文盘点和 §3 的缺口记录，**不是稳定版、镜像或部署审批**。最终镜像、供应链证明、签名、K3s 部署与回滚必须由后续 release gate 实测。

| 角色 | 审核记录 | 边界 |
|---|---|---|
| 架构 | 🟢 架构师（Mavis 接手审核） | 确认本报告的构建上下文范围与结论边界 |
| SRE Lead | 🟢 SRE Lead（Mavis 接手审核） | 明确部署、运行与回滚均未执行 |
| 平台 | 🟢 平台（Mavis 接手审核） | 仅确认 Cargo/路径/Dockerfile 输入检查 |
| 评审主持 | 🟢 评审主持（Mavis 接手审核） | 确认独立摘要与限制披露 |
| PM | 🟢 PM（Mavis 接手审核） | 记录静态检查结果；不将其标为稳定版可发布 |

## 7. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 K3s-local 生成 context 的独立路径、Cargo 与 Dockerfile 输入审查及 NOT RUN 门禁 | 用户要求稳定版制品只包含 Star 产品内容 |
