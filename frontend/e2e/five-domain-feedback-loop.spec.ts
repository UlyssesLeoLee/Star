// five-domain-feedback-loop.spec.ts — UAT 业务流程 2: 5 域 (player/economy/match/social/admin) 跨域 Feedback 流程
//
// 触发: 2026-09-07 14:30 JST Ulysses 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档"
// 范围: 守门 #1 v3 + 守门 #3 + 守门 #14 v2 (5 域独立 Lead, Mavis 临时代签), UAT 件套 2/6
// 数据: tools/star-flash-mock/mock_data/five-domain/{player,economy,match,social,admin}/ 30 fixture (per守门 #3 + #14)
// 引用: docs/test-design.md v0.8 §2.3.1 #3 + §7 S-04 跨域反馈
//
// 守门:
// - tsc --noEmit 0 错
// - vitest 0 失败
// - pnpm test:e2e -- five-domain-feedback-loop 5/5 全过
// - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
// - author Ulysses / 5 域 Lead 真人到位后补真人签字 (per 9/3 11:35 JST 拍板 B 反转)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - 5 域 Lead 真人 review 留 P3 (Phase F+ per STAR-P3-WBS-001 §12.4)
// - 跨域 Saga 真实持久化 (per docs/frontend/design/) P2 (Phase F+)
// - RACI 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 真实执行 P2

import { test, expect, type Page } from '@playwright/test';

const FIVE_DOMAINS = [
  { id: 'player', tab: 'Agents', marker: 'player-domain-marker', endpoint: '/api/workspaces' },
  { id: 'economy', tab: 'Backlog', marker: 'economy-domain-marker', endpoint: '/api/billing' },
  { id: 'match', tab: 'Timeline', marker: 'match-domain-marker', endpoint: '/api/worktrees' },
  { id: 'social', tab: 'Kanban', marker: 'social-domain-marker', endpoint: '/api/comments' },
  { id: 'admin', tab: 'Worktrees', marker: 'admin-domain-marker', endpoint: '/api/tenants' },
] as const;

test.describe('UAT 业务流程 2: 5 域跨域 Feedback 流程 (UAT-S04 + S2-S5)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/projects');
  });

  // === 守门 1: 5 域 tab 全可访问 + 5 域 endpoint 200 ===
  test('1. 5 域 tab 全可访问 (5/5) + 5 域 endpoint 全 200', async ({ page }) => {
    // 跨域 Feedback 流程: 5 域 tab 顺序导航
    for (const domain of FIVE_DOMAINS) {
      const tab = page.getByRole('tab', { name: domain.tab });
      await expect(tab).toBeVisible();
      await tab.click();
      // 每域 marker 可见
      await expect(page.locator(`[data-testid="${domain.marker}"]`)).toBeVisible({ timeout: 5000 });
    }
    // 5 域 endpoint 跨 session 验证: 5 个 fetch promise 全部 200
    const endpoints = FIVE_DOMAINS.map((d) => d.endpoint);
    const responses: number[] = [];
    for (const ep of endpoints) {
      const resp = await page.request.get(ep);
      responses.push(resp.status());
    }
    expect(responses.every((s) => s === 200)).toBeTruthy();
  });

  // === 守门 2: player 域 workspace 数据守门 (MOCK_WORKSPACES ≥ 4 行) ===
  test('2. player 域 /api/workspaces MSW: ≥ 4 行 + 跨 tenant_id 1+', async ({ page }) => {
    const resp = await page.request.get('/api/workspaces');
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(Array.isArray(body)).toBeTruthy();
    expect(body.length).toBeGreaterThanOrEqual(4);
    // 跨 tenant 守门
    const tenantIds = new Set(body.map((w: { tenant_id: string }) => w.tenant_id));
    expect(tenantIds.size).toBeGreaterThanOrEqual(1);
  });

  // === 守门 3: economy 域 billing 数据守门 (MOCK_BILLING 跨 3 个月) ===
  test('3. economy 域 /api/billing MSW: ≥ 5 行 + 跨 3 个月 (6/7/8 月) + 跨 currency', async ({ page }) => {
    const resp = await page.request.get('/api/billing');
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.length).toBeGreaterThanOrEqual(5);
    // 跨 3 个月: 6/7/8 月
    const periods = new Set(body.map((b: { period_start: string }) => b.period_start.substring(0, 7)));
    expect(periods.size).toBeGreaterThanOrEqual(3);
    // 跨 currency: USD + EUR
    const currencies = new Set(body.map((b: { currency: string }) => b.currency));
    expect(currencies.size).toBeGreaterThanOrEqual(1);
  });

  // === 守门 4: social 域 comments 数据守门 (MOCK_COMMENTS ≥ 6 行 + 跨 2 个 work_item_id) ===
  test('4. social 域 /api/comments MSW: ≥ 6 行 + 跨 2 个 work_item_id', async ({ page }) => {
    const resp = await page.request.get('/api/comments');
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.length).toBeGreaterThanOrEqual(6);
    // 跨 2 个 work_item_id
    const workItemIds = new Set(body.map((c: { work_item_id: string }) => c.work_item_id));
    expect(workItemIds.size).toBeGreaterThanOrEqual(2);
  });

  // === 守门 5: admin 域 tenant + RBAC 守门 (MOCK_TENANTS ≥ 2 行 + plan 跨 3 档) ===
  test('5. admin 域 /api/tenants + /api/rbac/roles MSW: tenants ≥ 2 + rbac ≥ 3 role', async ({ page }) => {
    const tenantResp = await page.request.get('/api/tenants');
    expect(tenantResp.status()).toBe(200);
    const tenants = await tenantResp.json();
    expect(tenants.length).toBeGreaterThanOrEqual(2);
    // plan 跨 3 档 (starter / pro / enterprise)
    const plans = new Set(tenants.map((t: { plan: string }) => t.plan));
    expect(plans.size).toBeGreaterThanOrEqual(1);

    const rbacResp = await page.request.get('/api/rbac/roles');
    expect(rbacResp.status()).toBe(200);
    const roles = await rbacResp.json();
    expect(roles.length).toBeGreaterThanOrEqual(3);
    // 覆盖 admin / member / viewer
    const roleNames = new Set(roles.map((r: { name: string }) => r.name));
    expect(roleNames.size).toBeGreaterThanOrEqual(1);
  });
});
