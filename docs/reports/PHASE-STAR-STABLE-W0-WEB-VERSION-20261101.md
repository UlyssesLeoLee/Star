# PHASE-STAR-STABLE-W0-WEB-VERSION-20261101

## 0. 目的

将稳定 Web 构建证据绑定到唯一规范版本来源：当前被检出的精确 Git 稳定 tag。工作流将解析结果用于构建信息、证据目录、压缩包和上传制品名称；产物仍明确标识为构建证据，不代表 Vercel 部署。

## 1. 改动矩阵

| 范围 | 改动 | 状态 |
|---|---|---|
| `.github/workflows/star-stable-web.yml` | checkout 获取 tag；经 `resolve_star_version.py` 校验稳定 tag、HEAD 和干净工作树；将解析到的 tag/version/source SHA 写入证据包和 build-info，并用于 SBOM 目录、压缩包、artifact 与 attestation 的一致命名 | 已实施 |
| 版本输入 | 继续使用 Git tag 为唯一规范来源；不改 `frontend/package.json` 中的 `0.1.0` | 符合范围 |
| 发布行为 | 保留显式 `APPROVED`、锁定依赖安装、SPDX、provenance 与缺文件失败；不创建 tag/release、不部署到 Vercel | 已保留 |
| 报告与审计 | 记录本 lane 的静态验证及限制 | 本报告与 WBS JSONL |

稳定 tag 使用 `vMAJOR.MINOR.PATCH` 形式。规划中的下一版是 `v1.0.6`；本 lane 没有创建该 tag。

## 2. 验证摘要

| 检查 | 结果 | 证据 |
|---|---|---|
| `rtk git diff --check` | 通过 | 无空白错误 |
| PyYAML 6.0.3 `yaml.safe_load` | 通过 | 工作流 YAML 可解析 |
| 工作流静态断言 | 通过 | 覆盖显式审批、共享 tag resolver、tag 目标与 `GITHUB_SHA` 一致、artifact 身份、`npm ci`、非部署说明 |
| CodeRabbit 自审 | 未能运行（0 次完成） | 两次调用均因 `/root/.local/bin/coderabbit` 权限拒绝退出；未取得 CodeRabbit 发现，已完成本地差异复核 |
| tag resolver 对实际 tag 执行 | 未运行 | lane 当前不是干净的稳定 tag checkout；resolver 按设计会 fail closed。`--repo ..` 从 frontend 工作目录指向仓库根目录，resolver 校验精确 tag peel commit 等于 HEAD，并要求工作树干净 |
| GitHub Actions hosted run、npm build/test、attestation | 未运行 | 需要由受控 stable tag workflow 执行；本地 YAML 检查不等同于 hosted run |
| Vercel 部署 | 未运行 | 本 workflow 仅产生 build-evidence artifact |

## 3. 已知缺口

1. 没有在真实 stable tag 上运行 GitHub Actions；artifact、SBOM 和 attestations 尚无 hosted-run 证据。
2. 未验证 Vercel 生产部署或其运行版本；此 workflow 不执行部署。
3. 此 lane 只绑定 build-evidence 产物版本；未验证或宣称浏览器 UI 会显示该版本，也未修改前端版本展示。
4. `frontend/package.json` 仍为 `0.1.0`，按范围保留为包元数据，不能作为稳定版本来源。
5. 实际发布 tag `v1.0.6` 尚未创建，发布负责人仍是用户指定的 accountable release owner。

## 4. 子代理失败接手清单

本 lane 由协调者分派至隔离 worktree。没有子代理调用失败或需要接手的子项。Orca agent 执行能力不可用时，本任务由当前 Codex worker 接续；无其他 worktree 内容被读取或修改。

## 5. 守门规则

1. Git tag 是唯一规范稳定版本来源。
2. 只接受无前导零、无 prerelease/build metadata 的 `vMAJOR.MINOR.PATCH` tag。
3. 生成证据必须显式选择 `APPROVED`。
4. 必须使用共享 `resolve_star_version.py`，不在 workflow 内自行推导版本。
5. resolver 要求 tag 精确存在并解析到 commit。
6. peeled tag commit 必须与 workflow 的 `GITHUB_SHA` 一致。
7. resolver 检查 repository checkout 干净后才输出 tag/version/source SHA。
8. artifact identity 中的 tag、version、source SHA 来自同一次 resolver 结果。
9. npm 安装继续使用 `npm ci` 和已提交 lockfile。
10. 保留现有前端单元测试步骤；本 lane 未声称测试运行通过。
11. SPDX 必须为 SPDX-2.3 且含非空 package 列表。
12. 压缩包保留确定性时间戳、排序、owner 和 gzip 元数据处理。
13. 压缩包及 SHA-256 文件缺失时上传必须失败。
14. provenance 与 SBOM attestation 指向 versioned archive 和对应 SPDX 文件。
15. workflow summary 明确说明 artifact 是 build evidence，不是 Vercel deployment。
16. 不修改包 manifest 版本、不创建 tag/release、不推送、不部署，也不变更 `main`/`dev`。

## 6. 签字栏

- 架构：架构师（Mavis 接手终审）
- SRE Lead：SRE 负责人（Mavis 接手审核）
- 平台：平台负责人（Mavis 接手审核）
- 评审主持：评审主持人（Mavis 接手审核）
- PM：产品负责人（Mavis 接手审核）

审查范围仅包括本 lane 的 Web build-evidence 工作流和本报告所述静态检查；Hosted workflow、真实发布及 Vercel 生产部署不在已验证范围内。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 stable Git tag 到 Web build-evidence 的版本绑定、静态验证与未完成的 hosted/Vercel 证据 | STAR-STABLE-W0-WEB-VERSION-20261101 |
