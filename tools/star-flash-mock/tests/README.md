# Star Mock Project — 沙箱 Worktree 隔离 E2E 回归测试

本目录存放 `tools/star-flash-mock` 配套的 Playwright 端到端回归测试，专门验证沙箱隔离功能：

1. `/sandbox` 开设 Worktree 并填写“沙箱挂载路径”
2. 后端 `POST /worktrees` 接收 `host_path`，用 Podman 把用户指定路径挂载为 `/src`
3. 创建后 Worktree 出现在左侧 `ProjectRunTree` 树的 Project 节点下
4. 同一 `worktree_id` 可从任务看板 / 任务卡 CLI / Canvas 等模块读取（跨模块数据互通）
5. 不同 branch 的 Worktree 互不可见（每个 branch 独立一套数据）

## 文件

| 文件 | 说明 |
|------|------|
| `sandbox-worktree-isolation.spec.ts` | Playwright E2E 回归脚本（6 个守门） |
| `playwright.config.ts` | Playwright 配置（chromium，baseURL localhost:3001） |
| `README.md` | 本文档 |

## 运行方式

### 方法 1：一键回归（推荐）

```bash
cd D:/Star
bash tools/star-flash-mock/scripts/run-all.sh
```

`run-all.sh` §ET 层会自动调用 `tests/run-e2e.sh` 运行 Playwright 脚本。

### 方法 2：单独运行 E2E

前提条件：
- frontend dev server 已启动（`npm run dev`，端口 3001）
- 后端已启动（端口 9002，`POST /worktrees` 端口可达）
- Podman 已安装并在系统 PATH 中（安装版本地执行容器挂载）

```bash
cd D:/Star/tools/star-flash-mock/tests
npx playwright test --config playwright.config.ts
```

或：

```bash
bash run-e2e.sh
```

## 守门准则

| 守门 | 说明 |
|------|------|
| tsc --noEmit | frontend `tsc --noEmit` 0 错 |
| vitest | frontend `vitest run` 0 失败 |
| Playwright | `pnpm test:e2e -- sandbox-worktree-isolation` 全过 |
| 0 子代理 | per 守门 #9 #3 实证 5/5 RPC 不可靠 |

## 已知缺口（per 守门 #11 缺标比错标安全）

- 当前 frontend 为 mock/降级模式（MSW），真实 Podman 容器挂载需后端 + Podman 就绪
- 跨 worktree 的并发冲突检测 P3（Phase F+）

## 集成

本目录的测试已集成进 `tools/star-flash-mock/scripts/run-all.sh` 的 §ET 层，与现有 UT/IT/ST 三层并行跑。
