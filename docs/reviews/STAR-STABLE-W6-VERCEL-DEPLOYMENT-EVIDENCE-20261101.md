# STAR Stable W6 Vercel Deployment Evidence Review

## 审查范围
本文件记录 lane 内部复核，提交到 dev 前仍需协调者进行集成审查。基线为 dev@3726c977984c69792dc21f0be97228e7a33e7659；范围限于本地 Web/Vercel 配置和 GitHub 可见部署/workflow API。

## 证据核对
| 核对项 | 结果 | 限定 |
|---|---|---|
| Vercel project config | frontend/vercel.json 指定 Next.js、npm ci、npm run build | 静态配置不证明生产部署存在 |
| Lockfile | package-lock.json 与 pnpm-lock.yaml 均跟踪 | package manager 尚未唯一化 |
| Stable Web workflow | 本地候选 workflow 只构建并上传证据包，明确不部署 Vercel；要求 tag 与 APPROVED 输入 | 配置不证明 workflow 曾实际运行 |
| GitHub deployment objects | 当前 API endpoint 返回空列表 | 不覆盖 Vercel 外部控制面 |
| GitHub workflow catalog | 14 项，未含 star-stable-web.yml；按该路径查询 runs 返回 404 | 以本次 API 可见 catalog 为限 |
| Exact local baseline runs | SHA 3726c977984c69792dc21f0be97228e7a33e7659 查询返回 total_count=0 | 无该 SHA hosted workflow 证据 |
| Production evidence | 没有 deployment ID/domain/status/source SHA、Neon/Owner profile、runtime、consumer 或 rollback record | 没有已连接 Vercel provider，无法独立查询外部部署 |

## 差异与风险
- 远端 dev@40a84c1 与本地 dev@3726c977 分离。
- GitHub repo API 未返回部署记录；不能以此断言 Vercel 外部控制面没有部署。
- 当前本地候选的 Web workflow 只生成 build evidence，不执行 production deploy。
- npm 和 pnpm lockfile 并存，W6 的唯一 package manager 条件未满足。
- 未得到实际 Vercel deployment、Neon/Owner runtime、协作路径、consumer 或 rollback 证据。

## 验证
- 对照 GitHub branch、deployments、workflow catalog 与 exact SHA runs API 响应。
- 对照 frontend/vercel.json、package.json、两个已跟踪 lockfile、next.config.js 与 star-stable-web.yml。
- diff check、§0–§7 标题检查和 dispatcher JSONL parse 通过；没有运行产品 build/test/deploy。

## 结论
HOLD — W6 维持 ADAPT / OPEN。lane 自查认为证据记录与边界一致；协调者集成审查待完成。当前资料只证明本地候选有 Vercel 构建配置和 build-evidence workflow，不证明 production deployment 已发生。不得据此批准发布 v1.0.6。
