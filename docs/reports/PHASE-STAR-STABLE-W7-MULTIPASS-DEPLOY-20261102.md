# PHASE-STAR-STABLE-W7-MULTIPASS-DEPLOY-20261102

> **状态：W7 OPEN / BLOCKED；稳定版 v1.0.6 NO-GO。** 本阶段把本地 API 构建、镜像导入和部署路径改为显式选择既有 Multipass guest，并补上 K3s 节点 Ready 预检。静态测试通过；未完成真实 guest、镜像或集群验收。

## §0 目的与范围

基线为本地 `dev@09b4eead99b047759be271ade2f5b06c0612ffb1`。此 W7 follow-up 处理本地 K3s 开发/测试 helper 的执行边界：调用者必须选择一个正在运行的 Multipass instance；所有构建、镜像导入、应用和 rollout 查询都定向到该 guest。它不属于公开 Desktop/Web/Vercel/Canvas 发布矩阵，也不实现 Star 原生的 Host/cluster owner、容量预留或 Run infrastructure binding。

本阶段只修改 MAGOS manifest 中的 W7 源文件、任务 brief、阶段报告、review、dispatch JSONL 与稳定版 WBS。未修改 `deploy/k3s-local/README.md`（仍由 D2 owner 持有）、其他 lane、`D:\Star` 主工作树、`main` 或远端。

## §1 改动矩阵

| 项目 | 结果 | 证据与边界 |
|---|---|---|
| Multipass 目标选择 | 实现 | 缺少实例名、名称格式无效、inventory 缺失/重复或实例非 `Running` 时停止；helper 不创建、启动、停止、删除或 purge VM。 |
| Guest 工具与节点就绪 | 实现并有回归用例 | 检查 guest Docker build engine、K3s API，以及所有返回节点的 `Ready=True`；空、格式错误、缺少节点或未就绪均 fail closed。此为本地 preflight，不是 Star 原生 infrastructure admission。 |
| Star-only 构建输入 | 接入既有过滤器 | 继续使用 `star_k3s_context.py` 的 Cargo 闭包、Dockerfile、symlink、路径、digest 和 game/legacy 校验。生产 Cargo metadata 当前无法执行，故真实 context 尚未验收；单元测试的 Cargo projection 是 fixture。 |
| Guest 构建与镜像导入 | 静态实现 | 在被选 guest 内运行 Docker build，通过管道将 guest `docker save` 流式送入同一 guest 的 K3s `k8s.io` containerd namespace，并查询 CRI image inventory。未实跑 guest。 |
| Manifest 与 rollout | 静态实现 | 临时 manifest 锁定本次本地 image tag；经所选 guest 的 K3s kubectl apply、rollout status 与 pod 查询。未实跑 cluster。 |
| 资源与副作用边界 | 静态测试通过 | 使用固定 argv；不暴露任意 shell；不调用宿主 Docker/K3s；不替换既有双实例受控 launcher，也不改变“两条测试实例”的上限。 |
| 文档与验收状态 | 已同步 | brief、dispatch JSONL、review 与 WBS v1.40 记录准确的验证结果和缺口；W7 保持 OPEN/BLOCKED，v1.0.6 保持 NO-GO。 |

## §2 验证摘要

- `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context -v`：18 项运行，17 项通过，1 项跳过。唯一跳过项是 Windows 当前不允许创建 symlink（WinError 1314）。测试对 K3s Ready/NotReady、节点数据格式错误/为空/缺少 Ready 条件、实例缺失/停止、guest Docker/K3s prereq、选定 guest 命令路由、context tamper 和 manifest tag 做断言。
- `rtk proxy py -3 -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py`：通过。
- `rtk proxy bash -n deploy/k3s-local/build-and-deploy.sh`：最终 LF 文件通过。第一次检查因工作副本 CRLF 在 Bash 下报 EOF；最终按 LF 规范化后复跑退出码 0。
- `rtk git diff --check`：通过。
- CypherGraph Guardian `--changed --stamp` 与随后 `--changed`：均通过，检查 3 个 source files。Stamp 后再次规范化 shell LF，随后验证 stamp 仍有效。
- `rtk proxy cargo +1.99.0 --version`：通过；`rtk proxy rustc +1.99.0 --version` 失败，提示 rustc.exe 不适用于已安装的 `1.99.0-x86_64-pc-windows-msvc` toolchain。
- 生产命令 `rtk proxy cargo +1.99.0 metadata --format-version 1 --locked --offline --manifest-path crates/star-api-rest/Cargo.toml --filter-platform x86_64-unknown-linux-gnu`：失败，退出码 101；Cargo 调用 rustc `-vV` 时得到同一 toolchain 错误。因此没有生成并审计真实生产 Cargo context。
- 未运行 Cargo build/test、Docker build、Multipass transfer、K3s apply/rollout、rollback、镜像 digest、SBOM/provenance 或独立 consumer 验证。没有把 mock/fake guest 命令描述为真实部署证据。

## §3 已知缺口

1. Windows 上 Rust 1.99.0 的 `rustc.exe` 当前不可运行，导致生产 Cargo metadata 与 filtered context generation 未验证；测试只验证以 fixture 提供 lock projection 的策略逻辑。
2. 没有真实 Multipass guest、guest 内 Docker 构建、image streaming/import、CRI 可见性、manifest apply、rollout、日志/消费者检查或 rollback 证据。
3. 没有验证 2 条 Multipass inventory 上限、锁定启动器行为或多 guest 资源预算；本 lane 不修改实例管理器。
4. 没有证明最终 image layers 只含 Star payload，也没有 digest、SBOM、provenance、signature 或 consumer verify 证据。`deploy/k3s-local` 仍为开发/测试路径，不关闭 Canvas 发布载荷门禁。
5. 本地 guest 节点 Ready 检查不等同于 Star 原生 Infrastructure owner reservations、cluster identity/generation binding、Host/cluster 容量核算或 TaskExecutionRun readiness admission。
6. D2 owner 的 `deploy/k3s-local/README.md` 草稿保持未读取、未修改、未覆盖，须由其 owner 后续协调。
7. 上述缺口未闭合前，W7 与稳定版均不得标记通过；W7 保持 OPEN / BLOCKED，v1.0.6 保持 NO-GO。

## §4 子代理失败接手清单

- Standards reviewer 未报告其他明确代码缺陷；要求真实环境证据保持未完成。
- Spec reviewer 指出原实现只检查 K3s 节点列表非空，不能证明节点 Ready。已改为检查所有节点 `Ready=True`，新增 NotReady fail-before-transfer 回归测试并复跑完整聚焦套件。
- 审阅 worker 未改动 W3、WBS 或其他 worktree；W3 lane 保持独立。

## §5 守门规则

1. 必须显式指定 Multipass instance；不允许默认选 guest 或回退宿主 Docker/K3s。
2. Guest 必须已存在且为 `Running`，guest Docker 与 K3s API 可用，且所有返回节点 Ready；不满足时在 build/transfer 前 fail closed。
3. 构建输入仅来自现有 Star Cargo closure generator；每次重新校验 context inventory 与 digest，不得复制整个 `crates/`。
4. Multipass guest 的构建、导入与部署仅使用固定参数数组；不得向任务、插件或 agent 暴露任意宿主/guest shell。
5. 不创建、启动、停止、删除或 purge VM；不变更当前最多 2 个 Multipass inventory 记录的限制或既有锁定 launcher。
6. Node Ready preflight 是本地 helper 检查，不宣称 Star 原生 host/cluster admission、reservation、Run binding 或生产部署已经实现。
7. W7 local helper 不得替代真实 Rust 1.99.0 context/build、镜像闭包、SBOM/provenance、consumer、rollback 与发布 gate。
8. `deploy/k3s-local/README.md` 仍归 D2 owner；不修改 `main`、不 push、不创建 tag/release、不发布镜像。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— guest preflight 与 Star-native infrastructure ownership 边界明确。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 无真实 guest/rollout/rollback 证据，W7 保持开放。 |
| 平台 | 平台（Mavis 接手审核）— 固定 argv、过滤 context 与 no-host-fallback 检查保留；生产 Cargo context 被 toolchain 阻断。 |
| 评审主持 | 评审主持（Mavis 接手审核）— 独立 review 发现的 NotReady 缺口已修复并通过回归测试。 |
| PM | PM（Mavis 接手审核）— 稳定版仍 NO-GO；不发布或推送。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Multipass guest deploy helper、Ready-node fail-closed preflight、聚焦验证和仍未完成的实际发行/集群证据。 | W7 Multipass 实施与独立 review 完成后同步验收边界 |
