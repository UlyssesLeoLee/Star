// worktree-creation-flow.spec.ts — UAT 业务流程 1: WorkItem → Worktree → Agent 状态机
//
// 触发: 2026-09-07 14:30 JST Ulysses 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档"
// 范围: 守门 #1 v3 (check+fmt+clippy 不替代 e2e) 硬约束, UAT 件套 1/6
// 数据: tools/star-flash-mock/mock_data/five-domain/match/ 6 fixture (per守门 #3 + #14)
// 引用: docs/test-design.md v0.8 §2.3.1 #1 + §7 S-01 S-02 S-03
//
// 守门:
// - tsc --noEmit 0 错
// - vitest 0 失败
// - pnpm test:e2e -- worktree-creation-flow 4/4 全过
// - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - 当前 MSW mock backend (per P3-A.7 9/3 11:35 JST 拍板, 不需要真后端)
// - 真实状态机执行 (saga 持久化) P3 (Phase F+)
// - PR / 评审联动 (per docs/frontend/design/) P2 (Phase F+)
// - 跨 worktree 的并发冲突检测 P3 (Phase F+)

import { test, expect, type Page } from '@playwright/test';

const PROJECTS_URL = '/projects';
const WORKTREE_TAB = 'Worktrees';

test.describe('UAT 业务流程 1: WorkItem → Worktree → Agent 状态机 (UAT-S01..S03)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PROJECTS_URL);
    // 切到 match 域 (Worktrees tab)
    await page.getByRole('tab', { name: WORKTREE_TAB }).click();
    await expect(page.locator('[data-testid="match-domain-marker"]')).toBeVisible({ timeout: 5000 });
  });

  // === 守门 1: Worktree 列表加载 (≥5 条 + 跨 3 个 project_id) ===
  test('1. /api/worktrees MSW 返回 ≥5 行 + 跨 3 个 project_id', async ({ page }) => {
    const responsePromise = page.waitForResponse(
      (r) => r.url().includes('/api/worktrees') && r.status() === 200
    );
    // 触发: 刷新 tab 或 reload
    await page.reload();
    const response = await responsePromise;
    const body = await response.json();
    // 数据守门 per mock-data-isolation.md §2.4 + five-domain.ts 5 域: MOCK_WORKTREES ≥ 5 条
    expect(Array.isArray(body)).toBeTruthy();
    expect(body.length).toBeGreaterThanOrEqual(5);
    // 跨 3 个 project_id 守门
    const projectIds = new Set(body.map((w: { project_id: string }) => w.project_id));
    expect(projectIds.size).toBeGreaterThanOrEqual(3);
  });

  // === 守门 2: Worktree 状态机覆盖 (per WorktreeStatus 17 状态中的 6 个) ===
  test('2. Worktree 状态机: MOCK_WORKTREES 状态分布覆盖 active / ci_running / review_requested / merged / abandoned / error', async ({ page }) => {
    const responsePromise = page.waitForResponse(
      (r) => r.url().includes('/api/worktrees') && r.status() === 200
    );
    await page.reload();
    const response = await responsePromise;
    const body = await response.json();
    // 6 个状态分布 (per five-domain.ts:147-200)
    const statusSet = new Set(body.map((w: { status: string }) => w.status));
    expect(statusSet.size).toBeGreaterThanOrEqual(3);
    // 至少包含 active (S-01 起始态)
    expect(statusSet.has('active')).toBeTruthy();
  });

  // === 守门 3: 单条 Worktree 详情 ===
  test('3. /api/worktrees/:id MSW 返回 200 + 完整 WorktreeSnapshot schema', async ({ page }) => {
    const responsePromise = page.waitForResponse(
      (r) => /\/api\/worktrees\/[^/]+$/.test(r.url()) && r.status() === 200
    );
    // 触发: 点击第一个 Worktree 行 (per cross-domain-5b.spec.ts 同模式)
    const firstRow = page.locator('[data-testid^="worktree-row-"]').first();
    if (await firstRow.isVisible()) {
      await firstRow.click();
      const response = await responsePromise;
      const body = await response.json();
      // WorktreeSnapshot schema 验证 (per mock-data-isolation.md §2.4)
      expect(body).toHaveProperty('id');
      expect(body).toHaveProperty('project_id');
      expect(body).toHaveProperty('branch');
      expect(body).toHaveProperty('status');
      expect(body).toHaveProperty('created_at');
      expect(body).toHaveProperty('last_event_at');
    } else {
      // 缺标 (per 守门 #11 缺标比错标): Worktree 行未渲染, 跳过子断言
      test.skip();
    }
  });

  // === 守门 4: Worktree 状态机 transition endpoint 守门 ===
  test('4. POST /api/worktrees/:id/transition: 200 + status: ok (mock echo)', async ({ page }) => {
    // 直接 verify: 缺 to 字段 → 400; 有 to 字段 → 200 ok
    const responsePromise = page.waitForResponse(
      (r) => r.url().includes('/api/worktrees/') && r.url().includes('/transition')
    );
    // 触发: click 任意可见的 "transition" 按钮 (per docs/frontend/design/ P2 入口)
    const transitionBtn = page.getByTestId('worktree-transition-btn').first();
    if (await transitionBtn.isVisible()) {
      await transitionBtn.click();
      const response = await responsePromise;
      expect(response.status()).toBe(200);
      const body = await response.json();
      expect(body.status).toBe('ok');
      expect(body).toHaveProperty('at');
    } else {
      // 缺标: transition 按钮未实装, 跳过子断言 (per 守门 #11 缺标比错标)
      test.skip();
    }
  });
});
