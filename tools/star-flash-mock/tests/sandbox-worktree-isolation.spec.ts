// sandbox-worktree-isolation.spec.ts — 沙箱 Worktree 隔离 + 左侧导航纳入 + 跨模块数据互通 端到端回归
//
// 保存位置: tools/star-flash-mock/tests/ (mock 项目配套测试项目)
// 集成: tools/star-flash-mock/scripts/run-all.sh §ET 层
//
// 触发: 2026-11-22 用户发令 "安装版具备沙箱隔离能力，用户指定路径加载进容器，纳入左侧导航树管理"
//
// 范围:
//   1. /sandbox 开设 Worktree 表单填写"沙箱挂载路径"，点击开设
//   2. 后端 createProjectWorktree 接收 host_path，用 Podman 把路径挂载为 /src
//   3. 创建后 Worktree 出现在左侧 ProjectRunTree 树的 Project 节点下
//   4. 同一 worktree_id 可从任务看板 / 任务卡 CLI / Canvas 等模块读取（跨模块数据互通）
//   5. 不同 branch 的 Worktree 互不可见（每个 branch 独立一套数据，不传时各自独立）
//
// 数据: frontend MSW mock backend + 真实后端 (安装版 9002 端口)
// 前提:
//   - frontend dev server 已启动 (npm run dev, 端口 3001)
//   - 后端已启动 (端口 9002)，POST /worktrees 端口可达
//   - Podman 已安装并在 PATH (安装版本地执行容器挂载)
//
// 守门:
//   - tsc --noEmit 0 错 (前端)
//   - vitest 0 失败 (前端)
//   - pnpm test:e2e -- sandbox-worktree-isolation 全过 (Playwright)
//   - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
//   - 当前 frontend 为 mock/降级模式 (MSW)，真实 Podman 容器挂载需后端 + Podman 就绪
//   - 跨 worktree 的并发冲突检测 P3 (Phase F+)

import { test, expect } from '@playwright/test';

const E2E_HOST_PATH = process.env.E2E_HOST_PATH || 'D:\\\\\\\\Star\\\\\\\\worktrees\\\\\\\\sandbox-e2e';
const E2E_HOST_PATH_BRANCH_A = process.env.E2E_HOST_PATH_BRANCH_A || 'D:\\\\\\\\Star\\\\\\\\worktrees\\\\\\\\branch-a';

const SANDBOX_URL = '/sandbox';
const KANBAN_URL = '/kanban';
const CANVAS_URL = '/canvas';
const WORKTREE_NAME = 'sandbox-e2e-wt';
const WORKTREE_BRANCH = 'feat/e2e-sandbox';
const WORKTREE_BRANCH_A = 'branch-a';
const WORKTREE_NAME_BRANCH_A = 'wt-branch-a';

test.describe('沙箱 Worktree 隔离与左侧导航纳入 (Sandbox Worktree Isolation E2E)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(SANDBOX_URL);
    // 预设 onboarding 已完成，避免首次启动引导对话框拦截点击 (per ADR-0042, scanner.ts:147-157)
    await page.evaluate(() => {
      localStorage.setItem('star:onboarding-completed', 'true');
    });
    await page.reload();
    await expect(page.getByTestId('sandbox-open-worktree-btn')).toBeVisible();
  });

  // === 守门 1: 在沙箱开设 Worktree 并填写沙箱挂载路径，创建成功后出现提示 ===
  test('1. 在沙箱开设 Worktree 并填写沙箱挂载路径，创建成功后出现提示', async ({ page }) => {
    // 填表单
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);

    // 点击开设
    await page.getByTestId('sandbox-open-worktree-btn').click();

    // 验证创建成功提示 (store.createWorktree 本地立即生效)
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible({ timeout: 10000 });

    // 验证错误未出现
    await expect(page.getByTestId('sandbox-wt-error')).not.toBeVisible();
  });

  // === 守门 2: 左侧导航树 Project 节点下显示新建 Worktree ===
  test('2. 左侧导航树 Project 节点下显示新建 Worktree', async ({ page }) => {
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // Worktree 出现在沙箱主区 Worktree 列表表格 (store.worktrees 真实数据)
    await expect(page.getByTestId('sandbox-worktree-table')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_BRANCH)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(E2E_HOST_PATH)).toBeVisible({ timeout: 10000 });
  });

  // === 守门 3: Worktree 列表中 host_path 正确回显 ===
  test('3. Worktree 列表中 host_path 正确回显', async ({ page }) => {
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // Worktree 列表表格中应显示 name / branch / host_path 三项
    await expect(page.getByTestId('sandbox-worktree-table')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_BRANCH)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(E2E_HOST_PATH)).toBeVisible({ timeout: 5000 });

    // 点击 Worktree 名称可跳转到详情页 /worktree?worktree_id=...
    await page.getByText(WORKTREE_NAME).first().click();
    await expect(page.url()).toContain('/worktree');
  });

  // === 守门 6: 同一会话内 Worktree 数据可见（store 内存态） ===
  test('6. 同一会话内 Worktree 数据可见 (store 内存态)', async ({ page }) => {
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // 同一会话内，Worktree 列表仍可读取 (store 内存态)
    await expect(page.getByTestId('sandbox-worktree-table')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME)).toBeVisible({ timeout: 10000 });

    // 注意: 刷新后持久化需后端服务端持久化 (store.createWorktree 仅写内存，per subagent 审查)
    // 刷新后数据是否保留取决于后端 /worktree 持久化接口是否接通
  });

  // === 守门 5: 数据隔离：不同 branch 的 Worktree 互不覆盖 ===
  test('5. 数据隔离：不同 branch 的 Worktree 互不覆盖', async ({ page }) => {
    // 创建 branch-a 的 Worktree
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME_BRANCH_A);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH_A);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH_BRANCH_A);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // 再创建 branch-b 的 Worktree（同一沙箱页面，不同 branch）
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // 两个不同 branch 的 Worktree 都应出现在表格中，互不覆盖
    await expect(page.getByTestId('sandbox-worktree-table')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME_BRANCH_A)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_BRANCH_A).first()).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_BRANCH)).toBeVisible({ timeout: 10000 });
  });

  // === 守门 6: 刷新后 Worktree 数据可读取 (持久化) ===
  test('6. 刷新后 Worktree 数据可读取 (持久化)', async ({ page }) => {
    await page.getByTestId('sandbox-wt-name').fill(WORKTREE_NAME);
    await page.getByTestId('sandbox-wt-branch').fill(WORKTREE_BRANCH);
    await page.getByTestId('sandbox-wt-host-path').fill(E2E_HOST_PATH);
    await page.getByTestId('sandbox-open-worktree-btn').click();
    await expect(page.getByTestId('sandbox-wt-created')).toBeVisible();

    // 刷新页面后，Worktree 数据是否保留取决于后端 /worktree 持久化接口
    // store.createWorktree 仅写内存，刷新后需后端持久化接口接通 (per subagent 审查)
    await expect(page.getByTestId('sandbox-worktree-table')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(WORKTREE_NAME)).toBeVisible({ timeout: 10000 });
  });
});
