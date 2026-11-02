# WBS-STAR-STABLE-W6-UAT-NPM-REVIEW-20261102

## 审阅状态

**ACCEPT（仅限本 lane 范围）**。候选已回应 parent review 的 npm cache 回退问题；parent agent 已独立复核实现差异、manifest 与验证结果。此结论不表示 W6 或 v1.0.6 稳定发行门通过。

## 审阅基线与范围

- 基线：`dev@d3611583cacc06770cee23f2ffb49c67f8b56df9`
- 候选：`codex/star-stable-w6-uat-npm-20261102`
- 只检查 `tools/star-flash-mock/scripts/README.md` 与 `tools/star-flash-mock/scripts/uat-3000-regression.ps1` 的 package-manager 命令替换，以及 brief、dispatch audit、阶段报告和本审阅记录。
- 不包括 CI、`frontend/package.json`、任何 lockfile、WBS 或 Vercel/Neon 运行态。

## 差异与结论

README 安装示例使用 `npm ci`，然后直接调用本地 `./node_modules/.bin/playwright install chromium`；测试通过现有 `npm run test:e2e -- ...` script。PowerShell 自动启动在 `frontend` 目录执行本地 `node_modules/.bin/next.cmd dev -p 3001`，Playwright 测试通过 `npm run test:e2e -- ...`。监听端口、spec 路径、结果提取与原服务检测保持不变。

## Parent review 发现与修正

| 发现 | 修正 | 证据范围 |
|---|---|---|
| `npm exec --offline` 在缺少项目本地依赖时仍可能选择 npm cache 中的二进制，不能证明执行来源是本项目 lockfile | Next 启动改为 `cmd` 调用 `frontend/node_modules/.bin/next.cmd`；README browser setup 改为 `./node_modules/.bin/playwright`；测试保留现有 `npm run test:e2e` | 静态检查命令和路径；未运行服务或 browser setup |

实现路径在本地 dependency shim 缺失时直接失败，不会由 npm 命令解析或安装缓存包。README 明确先运行 `npm ci`，再执行项目内 Playwright shim。

## 静态验证证据

- PowerShell AST `Parser.ParseFile`：0 parse errors。
- `rtk rg -n -i 'pnpm|npm exec'` 两份目标文件：无命中（rg exit 1，符合零匹配结果）。
- `rtk git diff --check`：通过。
- 未安装依赖、未启动 Next/K3s/WSL/3001，也未执行 UAT 或 browser setup。

## 已知风险 / 不可外推项

1. 两份 lockfile 仍并存，P1E workflow 和通用 CI 仍未统一；MAGOS 因 Dependabot refs 重叠拒绝 wider manifest，本 lane 未覆盖。
2. Node 22/24 对齐未处理。
3. UAT 脚本仍依赖宿主服务和 WSL/K3s 路径，当前宿主运行状态未验证。
4. Vercel/Neon/Owner 的真实部署和运行验收仍开放。
5. 因以上限制，W6 与 v1.0.6 不可标为通过。

## 协调者复核清单

- [x] diff 仅含登记路径，两份实现文件以外只有 lane brief、dispatch audit、阶段报告和本审阅记录。
- [x] Next 与 Playwright setup 指向 `npm ci` 创建的本地 shim，缺失时不会回退到 npm cache 包。
- [x] PowerShell 调用在 `frontend` 工作目录运行，保留原端口、测试 spec 路径、报告提取与服务检测。
- [x] brief、dispatch audit、报告与本审阅文件准确记录未执行 UAT 和仍阻塞的 W6 范围。
- [x] WBS 文件仍由 Star Delivery lane claim 持有；另行更新，不扩大本 lane 范围。

Parent agent 独立复核还确认：`Parser.ParseFile` 0 errors、两份实现文件搜索 `pnpm|npm exec` 无命中、候选与 dev 的 `git diff --check` 通过。CodeRabbit CLI 未能在此主机启动（执行入口返回 permission denied）；本结论基于上述实际差异及静态验证，没有把它记作外部自动审查通过。

## 判定

**ACCEPT（仅本 lane）。** Parent agent 于 2026-11-02 完成独立审阅。初版曾使用 `npm exec --offline`，因 npm 仍可能解析缓存二进制而被退回；修订后改为直接调用本地依赖 shim，验证通过。可将本 lane 两份 UAT 文件及四份配套文档集成到 dev。W6 其余 package manager/lockfile/CI/Node/Vercel/Neon/Owner 验收仍开放；WBS 更新须待其登记 owner 释放。
