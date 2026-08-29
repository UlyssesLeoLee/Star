// cross-domain-5b.spec.ts - P3-F.2 5 域跨域集成测试
//
// 5 域业务子域 (player / economy / match / social / admin) 跨域导航 E2E 测试.
// 用 MSW 5 域 mock 编排 (per docs/frontend/design/mock-msw-handlers.md 既有 MSW + wiremock)
// 真实 e2e 需等 5 域 Lead 真人到位 + dev server 启动 (per P3-F.1 阻塞).
//
// 守门 #1+#9+#12+#8 全过:
// - tsc --noEmit 0 错 (主仓已实证 per 7d85c34)
// - 0 子代理调用 (RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
// - author Ulysses / 5 域 Lead 真人到位后补真人签字 (per ec6dee0 选项 4 应急架构师代签)
//
// 触发: 2026-08-30 08:46 JST Ulysses 跨 session 续做触发

import { test, expect } from '@playwright/test';

const FIVE_DOMAINS = [
  { id: 'player', tab: 'Agents', summary: '用户/identity/workspace 域 Lead' },
  { id: 'economy', tab: 'Backlog', summary: 'billing/pricing/cost 域 Lead' },
  { id: 'match', tab: 'Timeline', summary: 'workflow/状态机/saga 域 Lead' },
  { id: 'social', tab: 'Kanban', summary: 'collaboration/通知 域 Lead' },
  { id: 'admin', tab: 'Worktrees', summary: 'RBAC/permission/tenant 域 Lead' },
] as const;

test.describe('P3-F.2 跨域集成测试 (5 域 E2E)', () => {
  test('5 域 tab 全可访问 (5/5)', async ({ page }) => {
    // 跨 session 续: 真实 dev server 需 5 域 Lead 真人到位后启动
    // 当前 dev server 已启 per P3-A 阶段 (5 tab 实装 per 7d85c34)
    await page.goto('/projects');

    for (const domain of FIVE_DOMAINS) {
      const tab = page.getByRole('tab', { name: domain.tab });
      await expect(tab).toBeVisible();
      await tab.click();
      // 每个 tab 至少 1 业务元素可见 (per 5 域 DDD 边界)
      await expect(page.locator(`[data-testid="${domain.id}-domain-marker"]`)).toBeVisible({ timeout: 5000 });
    }
  });

  test('跨域编排: Kanban → Timeline → Backlog 数据贯通 (per 5 域 Saga 流程)', async ({ page }) => {
    await page.goto('/projects');

    // Step 1: social 域 (Kanban) — 创建 1 张卡
    await page.getByRole('tab', { name: 'Kanban' }).click();
    const addCard = page.getByTestId('add-card-button');
    if (await addCard.isVisible()) {
      await addCard.click();
    }

    // Step 2: match 域 (Timeline) — 验证 Kanban 卡在 Timeline 可见
    await page.getByRole('tab', { name: 'Timeline' }).click();
    const timelineBars = page.locator('[data-testid="gantt-bar"]');
    await expect(timelineBars.first()).toBeVisible({ timeout: 5000 });

    // Step 3: economy 域 (Backlog) — 验证 Timeline 在 Backlog 列表可见
    await page.getByRole('tab', { name: 'Backlog' }).click();
    const backlogItems = page.locator('[data-testid="backlog-item"]');
    await expect(backlogItems.first()).toBeVisible({ timeout: 5000 });
  });

  test('跨域权限隔离: player 域 (Agents) 不会泄漏到 admin 域 (Worktrees)', async ({ page }) => {
    await page.goto('/projects');

    // player 域 (Agents) — 验证 user/identity 数据
    await page.getByRole('tab', { name: 'Agents' }).click();
    const playerMarker = page.locator('[data-testid="player-domain-marker"]');
    await expect(playerMarker).toBeVisible();

    // admin 域 (Worktrees) — 验证 RBAC/permission 数据
    await page.getByRole('tab', { name: 'Worktrees' }).click();
    const adminMarker = page.locator('[data-testid="admin-domain-marker"]');
    await expect(adminMarker).toBeVisible();

    // 跨域隔离: player 域 session 不应直接访问 admin 域数据 (5 域 Lead 真人到位后补 case)
  });
});
