# PHASE — Star Stable W6 Vercel Deployment Evidence Recheck

## §0 目的
核对 WBS-W6 要求的实际 Web/Vercel 发布闭环证据，区分本地 dev 候选与 GitHub 当前可见远端状态。此阶段只审阅配置和公开 API；不运行构建或部署，不读取密钥，不修改 WBS。W6 状态维持 ADAPT / OPEN。

## §1 改动矩阵
| 文件 | 改动 | 验收 |
|---|---|---|
| docs/reports/PHASE-STAR-STABLE-W6-VERCEL-DEPLOYMENT-EVIDENCE-20261101.md | 记录本地配置与远端部署/API证据 | 按 §0–§7 完整报告 |
| docs/reviews/STAR-STABLE-W6-VERCEL-DEPLOYMENT-EVIDENCE-20261101.md | 另附审查结论与限制 | 不把静态配置当作生产部署证明 |
| docs/reports/WBS-DISPATCH-STAR-STABLE-W6-VERCEL-DEPLOYMENT-EVIDENCE-20261101.jsonl | 记录证据、review、validation 事件 | JSONL 可解析 |
| docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md | 不修改 | 按协调者要求由父 lane 汇总更新 |

## §2 验证摘要
### 本地候选配置
- 基线：local dev@3726c977984c69792dc21f0be97228e7a33e7659；lane 起点 935f382cfddfe703b01862eb76762fff6801fd2b。初始工作树干净。
- [frontend/vercel.json](../../frontend/vercel.json) 声明 Next.js framework，installCommand 为 npm ci，buildCommand 为 npm run build。
- frontend/package.json 为 star-frontend 0.1.0，提供 build 与 start 脚本。frontend/package-lock.json 和 frontend/pnpm-lock.yaml 都是已跟踪文件；当前配置因此没有满足 W6 的唯一 package manager/lockfile 要求。
- [.github/workflows/star-stable-web.yml](../../.github/workflows/star-stable-web.yml) 是本地候选中的 workflow。它只接受 tag 上的 workflow_dispatch 和显式 APPROVED 输入，验证 tag target SHA 后执行 npm ci、前端测试、Next.js build、SPDX SBOM、打包和 GitHub artifact attestation；artifact retention 为 14 天。workflow 注释及 summary 明确说明它生成的是 build evidence bundle，不会向 Vercel 部署。
- frontend/next.config.js 仍设置 typescript.ignoreBuildErrors=true。该静态配置不证明部署运行正常，也不满足独立类型检查证据。

### GitHub API 只读观察（2026-11-01 23:10–23:20 UTC）
- GitHub branch API 当前返回 dev@40a84c1aaf804267ee25e02384e99276c3151906，commit time 为 2026-10-24T23:16:08Z；该 ref 不等于本地候选。
- [GitHub deployments API](https://api.github.com/repos/UlyssesLeoLee/Star/deployments?per_page=10) 返回空数组（count=0）。这是仓库 GitHub Deployments 证据，不覆盖 Vercel 服务端独立部署记录。
- [GitHub workflow catalog](https://api.github.com/repos/UlyssesLeoLee/Star/actions/workflows?per_page=100) 返回 14 项，当前 catalog 中没有 .github/workflows/star-stable-web.yml。按该路径查询 workflow runs 返回 HTTP 404。
- 对 local dev 基线 SHA 3726c977984c69792dc21f0be97228e7a33e7659 查询 [Actions runs](https://api.github.com/repos/UlyssesLeoLee/Star/actions/runs?head_sha=3726c977984c69792dc21f0be97228e7a33e7659&per_page=10)，total_count=0。
- 当前可用工具中没有 Vercel 连接器或该项目的部署记录接口；本审查没有 Vercel project/production deployment ID、production domain/runtime、deployment source SHA、Neon/Owner profile、真实协作路径、消费者验收或 rollback 证据。

## §3 已知缺口
1. 当前可见 GitHub dev 仍在 2026-10-24 的 40a84c1；本地 dev@3726c977 没有对应的 hosted Action run。
2. 远端 GitHub Deployments API 没有 deployment 记录，当前 workflow catalog 未列出本地候选的 stable Web evidence workflow。
3. 仓库没有 Vercel 服务端 API 访问或 project/deployment 证据；因此不能确认 Vercel production 是否已部署，也不能填写 deployment ID、URL、environment、status、source commit 或运行时配置。
4. 本地 Vercel 配置调用 npm，而 frontend 同时保留 npm 和 pnpm lockfiles；需统一并由锁文件驱动安装。
5. W6 要求的 Neon/Owner profile、协作路径运行证明、artifact/BOM 绑定、消费者验证与 rollback 均缺少实测证据。稳定发行仍 NO-GO。

## §4 子代理失败接手清单
本阶段为单一 W6 evidence audit，没有派生子代理或待接手 worker 产物。Vercel API 查询不可用不是工具运行失败的误报：本环境没有已连接的 Vercel provider；报告将其记为证据缺口。

## §5 守门规则
1. GitHub 的 branch ref 不代表本地 dev 已推送。
2. 远端 branch SHA 与 local dev SHA 分开记录。
3. GitHub Deployments 为空只表示仓库 endpoint 当前没有返回记录，不推断 Vercel 外部控制面状态。
4. workflow catalog 没有 stable Web workflow，不等于该本地工作流失败或通过。
5. path-specific workflow runs 的 404 不是成功结果。
6. exact SHA runs total_count=0 表示 API 未返回该候选的 hosted runs。
7. workflow 文件静态说明 build evidence only，不得称作 Vercel deployment。
8. artifact upload/attestation 的 workflow 配置不等于真实产物或 attestation 已生成。
9. 14 天 artifact retention 不满足长期 release distribution。
10. npm 与 pnpm lockfile 并存，W6 package manager/lockfile 门仍 OPEN。
11. frontend 版本字符串 0.1.0 不替代用户确认的 v1.0.6 Git tag 版本源。
12. 不读取本机环境变量、API token、Vercel secret 或连接凭证。
13. 不运行 build、test、deploy、release 或外部写操作。
14. 不修改 source/config、WBS、main 或其他 worktree。
15. 不 fetch、不 push，不创建 tag/release。
16. 未提供运行时、消费者、升级或 rollback 证据时，W6 与总体发行保持 OPEN/NO-GO。

## §6 签字栏
| 角色 | 结论 | 责任边界 |
|---|---|---|
| 架构 | Mavis 接手审核：接受证据范围 | 仅核对 Web/Vercel 部署证据边界 |
| SRE Lead | OPEN | production runtime、health/rollback 证据缺失 |
| 平台 | OPEN | Vercel deployment owner/profile 与 source mapping 缺失 |
| 评审主持 | Mavis 接手审核 | 只接受本阶段只读审查结果 |
| PM / Release owner | HOLD / NO-GO | W6 与 v1.0.6 release gate 未关闭 |

## §7 修订历史
| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 只读复核本地 Vercel 配置、远端 GitHub deployment/workflow 状态与 W6 验收缺口；未修改生产实现或发布状态。 |
