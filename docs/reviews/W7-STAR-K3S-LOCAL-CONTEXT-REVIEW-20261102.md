# W7 本地 K3s Star context 接线审阅 — 2026-11-02

## 决定

**本地 build-path 修正通过；W7 发布门仍 OPEN / BLOCKED。** `build-and-deploy.sh` 不再复制整个 `crates/`，而是调用现有生成器，build 使用生成的 allowlist Dockerfile/context。该改动修复了被发现的 Star-only 输入绕行，但没有实建 image，也没有完成发布矩阵的 Multipass/K3s 验收。

## 审阅矩阵

| 检查项 | 结果 | 说明 |
|---|---|---|
| 完整 crate tree 进入 build context | 已修复 | 删除 `cp -r ${REPO_ROOT}/crates`，使用 `star_k3s_context.py` 生成 Cargo closure context。 |
| Dockerfile 是否展开 | 已修复 | build 命令使用生成目录中的 `deploy/k3s-local/Dockerfile`。 |
| K3s 本地导入与部署步骤 | 保留 | 镜像 save/import、kubelet 镜像可见性、manifest apply 与 rollout 保留。 |
| 新回归检查 | 通过 | 单项 unittest 断言 generator invocation、Dockerfile/context 和原部署步骤。 |
| Shell 语法 | 通过 | 最终 LF 工作副本 `bash -n` 通过；初始 CRLF 副本的 EOF 诊断在规范化后消失。 |
| 真实 context/image/MultiPass/K3s | 未运行 | 不在本 lane 可证明范围；发布门保持打开。 |
| D2 README | 保留 | owner 的未提交内容未触碰。 |

## 风险与未关闭项

- 运行生成器依赖本机 Cargo metadata 能力；Rust 1.99.0 可用性尚未验收。
- 没有真实 archive、final image、digest、SBOM/provenance 或 consumer verification。
- 本地 API script 不构成 Multipass 多实例调度、隔离或资源预留验收。
- `deploy/k3s-local/README.md` 支持范围由 D2 owner 后续确认。
- W7 保持 OPEN / BLOCKED；稳定版 v1.0.6 保持 NO-GO。

## 检查结果

| 检查 | 结果 |
|---|---|
| Focused Python regression | PASS — 1 test |
| Bash syntax | PASS — final LF working copy; initial CRLF EOF diagnostic resolved by LF normalization |
| CypherGraph Guardian | PASS — 2 source files |
| `git diff --check` | PASS |
| Docker/Multipass/cluster release acceptance | NOT RUN |

## 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| v0.1 | 2026-11-02 | 审阅过滤上下文是否接入本地 build-and-deploy 路径，并保持实际制品/基础设施门禁开放。 |
