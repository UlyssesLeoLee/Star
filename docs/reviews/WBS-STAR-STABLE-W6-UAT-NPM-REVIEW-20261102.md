# WBS-STAR-STABLE-W6-UAT-NPM-REVIEW-20261102

## 审阅状态

**PENDING_PARENT_REVIEW**。候选已回应 parent review 的 npm cache 回退问题；最终独立审阅由 parent agent 完成，完成前不批准集成。

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

- [ ] 确认 diff 仅含登记路径且两份实现文件以外仅有 lane 文档。
- [ ] 确认 Next 与 Playwright setup 指向 `npm ci` 创建的本地 shim，缺失时不回退到外部包。
- [ ] 核对 PowerShell 调用工作目录为 `frontend`，参数和原命令效果一致。
- [ ] 核实 brief、dispatch audit、报告与本审阅文件准确记录未执行 UAT 和仍阻塞的 W6 范围。
- [ ] 完成独立 review 后再集成；WBS 由其登记 owner 释放后另行更新。

## 判定

尚无独立 parent review 结论。候选未达到集成批准状态。协调者审阅后补充审阅者、发现项及 ACCEPT/REJECT 结论。