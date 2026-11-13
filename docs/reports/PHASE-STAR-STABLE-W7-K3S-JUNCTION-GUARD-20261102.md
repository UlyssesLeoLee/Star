# PHASE-STAR-STABLE-W7-K3S-JUNCTION-GUARD-20261102

## §0 目的

修复 Star K3s filtered build context 在 Windows 下跟随 package 子目录 junction、复制仓库外内容的缺口。本报告只覆盖生成器的路径安全与本机回归测试；W7 与 v1.0.6 发行门仍开放。

基线：`dev@789392b67a90998bfa59ed49c3deaab31a012269`。
实现提交经 rebase 后为：`86368147`（reparse-point 检查与真实 junction 回归），`a1eb5821`（缺失 Windows 元数据回归）；两项已随 `0d3b9f8a` 首次快进到本地 dev。原始提交 `ca61ab93`、`83bf3f96` 已由 rebase 替换。
实现工作树：`D:\\orcaWork\\Star\\star-stable-w7-k3s-junction-guard-20261102`。

## §1 改动矩阵

| 项目 | 结果 | 证据 / 边界 |
|---|---|---|
| reparse 检测 | 完成 | 新 `_is_reparse_point` 使用不跟随链接的 `lstat`，跨平台拒绝 symlink；Windows 检查 `FILE_ATTRIBUTE_REPARSE_POINT`。缺少 `st_file_attributes` 或元数据读取失败时拒绝继续。 |
| 仓库输入路径 | 完成 | `_repo_relative` 对从仓库根到输入的每一级路径检查 reparse point，避免 junction 祖先被 `resolve()` 消去。 |
| 包目录复制 | 完成 | `_copy_local_package` 在保留子目录、进入下一轮遍历及复制文件前检查 reparse point；发现链接即停止。 |
| Windows junction 回归 | 完成 | 对选中包目录建立指向仓库外的真实 junction；调用生产 context 复制流程并确认 `ContextError`，外部 sentinel 未复制；测试清理只移除 junction 本身。 |
| 不支持的元数据回归 | 完成 | 模拟 Windows `lstat` 元数据没有 `st_file_attributes`，确认失败关闭。 |
| WBS 与设计同步 | 完成 | 新增测试设计；在稳定版 WBS 记录 W6 npm 本地验收与 W7 junction 修复，并保留各发布门缺口。 |

## §2 验证摘要

| 命令 | 结果 |
|---|---|
| `rtk proxy py -3 --version` | Python 3.13.7 |
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context.StarK3sContextTests.test_rejects_junction_inside_selected_package -v` | 1 passed；真实 Windows junction 测试未跳过 |
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context.StarK3sContextTests.test_fails_closed_without_windows_reparse_metadata -v` | 1 passed |
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context -v` | 20 tests run：19 passed，1 skipped。唯一 skip 是已有 symlink fixture 因 WinError 1314 缺少创建 symlink 的权限；新的 junction 与缺失元数据回归均通过。 |
| `rtk proxy py -3 -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py` | 通过 |
| CypherGraph Guardian `--changed --stamp` | 通过；先后检查 source 与 test header，最后状态无未校验的源文件改动。 |
| `rtk proxy git diff --check` / staged diff check | 通过 |

## §3 已知缺口

1. 未在 Rust 1.99.0 下运行 Cargo metadata 或生成实际项目 context；当前代码验证使用 Python 3.13.7。
2. 未运行 Docker 或生成 context archive / image；没有 image digest、最终层内容、SBOM、provenance、签名或 attestation 证据。
3. 未运行 Multipass guest build、K3s rollout/rollback 或 consumer pull；没有两实例容量验收，也没有 Star-native cluster binding/reservation/readiness 证据。
4. 本机既有 symlink fixture 因 WinError 1314 跳过；新 junction 回归本轮真实执行并通过。
5. 因上述缺口，W7 保持 OPEN / BLOCKED，v1.0.6 保持 NO-GO。

## §4 子代理失败接手清单

W7 实现代理在开始编辑前因平台用量上限退出，没有留下源码或文档改动。协调者接手同一 MAGOS 隔离工作树和已登记文件清单，重新检查工作树仅有 dispatcher brief 与 JSONL 回执后继续；未覆盖或清理任何其他 worktree。首次测试夹具调用 `cmd /c mklink` 的参数形式不正确，未创建 junction；改为 PowerShell `New-Item -ItemType Junction` 后，真实 junction 回归通过。

## §5 守门规则

1. 保持输入边界为 Star Cargo 闭包；不得把全仓 `crates/` 复制回构建上下文。
2. symlink、junction 或其他 Windows reparse point 必须在遍历/复制前拒绝。
3. Windows 文件元数据缺失或不可读取时 fail closed。
4. 不依赖 Python 3.12 才出现的 `Path.is_junction()`；支持旧版本使用 `lstat` 文件属性识别。
5. junction 测试的外部 sentinel 绝不进入 output；清理只删除 junction 本身。
6. 记录原 symlink 测试的权限 skip，不将其算作通过。
7. 20 项 focused tests 中真实 junction 与缺失元数据用例必须通过。
8. Cypher header 必须与实现和测试调用关系一致。
9. 本地 context 过滤测试不等于 Docker image 或发布制品验收。
10. W7/W6 及总体稳定版状态在剩余镜像、SBOM/provenance、consumer、hosted 和 rollback 证据未通过前保持 OPEN。
11. 不修改 `main`、远端、D2 README、Multipass inventory 或其他 lane。
12. 不 build/deploy/publish 镜像，不创建 tag/release，不 push。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— reparse point guard 限于路径输入与 context copier。 |
| SRE Lead | 未验证：本阶段未执行 VM、Docker、K3s 或 hosted 环境。 |
| 平台 | 本地代码审阅通过；实际镜像、SBOM 与 provenance 未验收。 |
| 评审主持 | 协调者独立审阅代码差异与 focused test 结果。 |
| PM / Release Owner | Ulysses 为 accountable release owner；本报告不是发行批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 K3s filtered context 的 Windows junction 绕过，加入真实 junction 与缺失元数据测试；明确 W7 与 release gate 仍开放 | STAR-STABLE-W7-K3S-JUNCTION-GUARD-20261102 brief |
| v0.2 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将代码提交号更新为 rebase 后的 `86368147` / `a1eb5821`，记录其已快进到 `dev@0d3b9f8a`；复验 20 项测试为 19 通过、1 项既有 symlink 权限 skip，W7 与 v1.0.6 发布门仍开放。 | W7 review 文档进入 dev 后按用户要求 rebase 并集成实现 lane |
