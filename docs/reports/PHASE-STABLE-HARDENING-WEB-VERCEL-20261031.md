# PHASE — Stable Web/Vercel Configuration

## §0 目的

为 Star Web 前端提供可复现的 Vercel 项目配置。本阶段只明确 install/build 命令，不创建或修改 Vercel 项目设置，不执行部署。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `frontend/vercel.json` | 显式声明 Next.js framework、`npm ci` 和现有 `npm run build` | 完成 |
| `docs/briefs/STABLE-HARDENING-WEB-VERCEL-20261031.md` | dispatcher 记录的工作 brief | 已存在 |
| `docs/reports/WBS-DISPATCH-STABLE-HARDENING-WEB-VERCEL-CONFIG-20261031.jsonl` | brief 与本阶段静态核验事件 | 本阶段记录 |

## §2 验证摘要

- 使用 PowerShell JSON parser 检查 `frontend/vercel.json` 和 `frontend/package.json` 的语法。
- 静态核对 Vercel `installCommand` 为 `npm ci`，并且 `buildCommand` 对应现有 package script `build: next build`。
- 查阅 [Vercel Node.js dependency detection](https://vercel.com/docs/functions/runtimes/node-js)：自动检测存在多个锁文件时优先识别 pnpm，再识别 npm；[Vercel project configuration](https://vercel.com/docs/project-configuration/vercel-json) 确认 `framework`、`buildCommand` 与 `installCommand` 可在项目根 `vercel.json` 覆盖。
- 未运行 Next.js build，也未执行 Vercel 部署；静态核验不代表部署已成功。

## §3 已知缺口

1. 本仓库当前同时有 `frontend/package-lock.json` 和 `frontend/pnpm-lock.yaml`。本配置显式选择 `npm ci`，但 lockfile 的唯一权威仍需由活动依赖维护 lane 决定；本 lane 未改动 package manifest 或任一 lockfile。
2. Vercel Project Root Directory 必须配置为 `frontend`，这样该目录下的 `vercel.json` 和 `package.json` 才是项目根输入。本阶段没有 Vercel dashboard/API 证据，未设置或声称该值已生效。
3. 缺少真实 Vercel deployment record、source commit、生产/预览域名、消费侧验证及 rollback 证据。
4. 稳定版其他门禁（MIT 完整依赖权利闭包、SBOM/provenance、CI required checks、桌面和镜像构建、Multipass/K3s 验收）仍保持 OPEN。

## §4 子代理失败接手清单

- 本 lane 未派生子代理；无失败接手项。

## §5 守门规则

1. 仅修改 MAGOS manifest 登记的 `frontend/vercel.json` 与本 lane 报告/dispatch 文件。
2. 保留活动依赖维护分支所拥有的 `frontend/package.json` 和 npm/pnpm lockfiles。
3. 不虚构项目设置、凭据、域名或生产状态。
4. 不部署、不发布、不打 tag、不 push。
5. 只把可复现静态核验记为本阶段证据，不把它升级成 build/deploy 证明。

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | 架构边界已复核；真实 Vercel 项目配置待 owner 提供证据 |
| SRE Lead | 未完成生产部署与 rollback 演练 |
| 平台 | 静态配置已核对；远端平台状态未核验 |
| 评审主持 | 审查范围限于本 lane manifest |
| PM | 本阶段仅完成配置切片；稳定版仍未就绪 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增显式 Vercel install/build 配置与本阶段验证/缺口记录；不宣称真实部署完成。 |
