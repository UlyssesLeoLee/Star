// canvas-view.spec.ts — Miro 模式无限画布 e2e 守门 (per 2026-09-04 拍板)
//
// 触发: 2026-09-04 19:14 JST Ulysses 拍板"无限画布后续做哪一块? e2e 守门补齐 (推荐)"
// 范围: 守门 #1 v3 (check+fmt+clippy 不替代 e2e) 硬约束, 6 项 + 1 minimap
// 数据: canvas-001 (per seed.ts:418) 25 elements + 8 connectors + 4 frames
//
// 守门:
// - tsc --noEmit 0 错
// - vitest 0 失败
// - pnpm test:e2e -- canvas-view 7/7 全过
// - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - 当前 canvas 数据全 mock, 真实数据接入后 e2e 复用同一 spec
// - 双击跳详情仅测 work_item_card, 不覆盖 worktree/agent/automation (4 类同算法)
// - 不测 touch 事件 (mobile 走 cross-domain-5b + remote-mobile 现有 e2e)
// - 滚轮以光标为中心 zoom 数学复杂, 留 P2

import { test, expect, type Locator, type Page } from '@playwright/test';

const CANVAS_URL = '/canvas/canvas-001';

// 工具函数: 读 minimap viewport rect 的 x/y/width/height
async function readMinimapViewportRect(page: Page) {
  const rect = page.locator('[data-testid="canvas-minimap"] rect').first();
  await expect(rect).toBeVisible();
  return await rect.evaluate((el: SVGElement) => ({
    x: Number(el.getAttribute('x') ?? 0),
    y: Number(el.getAttribute('y') ?? 0),
    width: Number(el.getAttribute('width') ?? 0),
    height: Number(el.getAttribute('height') ?? 0),
  }));
}

test.describe('Miro 无限画布 e2e 守门 (canvas-001, 25 elements)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(CANVAS_URL);
    // 等 CanvasView mount: 主 svg 出现 + 至少 1 element 渲染
    await expect(page.locator('[data-testid="canvas-svg"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-element-el-wi-001"]')).toBeVisible();
  });

  // === 守门 1: 入口可用性 ===
  test('1. /canvas/canvas-001 路由 200 + 14 element + 4 frame + minimap 渲染', async ({ page }) => {
    // Page header
    await expect(page.getByText('Physis Sprint 23 — Worktree + Agent 工作流')).toBeVisible();
    // 14 elements (per seed.ts:453-473: 2 wi + 3 wt + 3 ag + 2 fb + 1 sn + 2 au + 1 tx = 14)
    const elementCount = await page.locator('[data-testid^="canvas-element-el-"]').count();
    expect(elementCount).toBeGreaterThanOrEqual(14);
    // 4 frames (per seed.ts:425-429)
    await expect(page.locator('[data-testid="canvas-frame-frame-001"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-frame-frame-002"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-frame-frame-003"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-frame-frame-004"]')).toBeVisible();
    // Toolbar + minimap + svg
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-minimap"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-svg"]')).toBeVisible();
    // Toolbar zoom % 显示 100 (默认 viewport.zoom = 1)
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('100%');
  });

  // === 守门 2: pan (shift+drag 主 svg) ===
  test('2. pan: shift+drag 主 svg, viewport.x / viewport.y 变化', async ({ page }) => {
    const rect0 = await readMinimapViewportRect(page);

    const svg = page.locator('[data-testid="canvas-svg"]');
    const box = await svg.boundingBox();
    if (!box) throw new Error('canvas-svg no bounding box');

    // shift+drag: 从 (400, 300) 到 (200, 100) 拖 -200,-200
    await page.keyboard.down('Shift');
    await page.mouse.move(box.x + 400, box.y + 300);
    await page.mouse.down({ button: 'left' });
    await page.mouse.move(box.x + 200, box.y + 100, { steps: 8 });
    await page.mouse.up({ button: 'left' });
    await page.keyboard.up('Shift');

    // viewport.x 变化 (drag -200px 屏幕 / zoom=1 → world dx=+200, 实际 world x -= dx_screen/zoom, per CanvasView.onMouseMove line 86)
    const rect1 = await readMinimapViewportRect(page);
    // rect1.x 应大于 rect0.x (因为 pan 方向反向, viewport.x 减 → viewport rect x 增)
    expect(rect1.x).toBeGreaterThan(rect0.x);
  });

  // === 守门 3: zoom (工具栏 ZoomIn + 滚轮 双路径) ===
  test('3a. zoom: 工具栏 ZoomIn 按钮, zoom 1 → 1.2', async ({ page }) => {
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('100%');
    await page.locator('[data-testid="canvas-toolbar"] button[title="Zoom in (+)"]').click();
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('120%');
  });

  test('3b. zoom: 滚轮向上 (svg 内部 hover 触发), zoom 1 → 1.1', async ({ page }) => {
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('100%');
    const svg = page.locator('[data-testid="canvas-svg"]');
    // dispatch wheel 事件 on svg element (避免 foreignObject 拦截)
    await svg.dispatchEvent('wheel', { deltaY: -100, bubbles: true, cancelable: true });
    // wait for re-render: 100% * 1.1 = 110%
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('110%');
  });

  test('3c. zoom: 上限 400% 边界 (连点 30 次 ZoomIn, 不超过 400%)', async ({ page }) => {
    const zoomIn = page.locator('[data-testid="canvas-toolbar"] button[title="Zoom in (+)"]');
    for (let i = 0; i < 30; i++) {
      await zoomIn.click();
    }
    // 100% * 1.2^30 ≈ 23.7x, 但上限 400% (Math.min(4, ...))
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('400%');
  });

  // === 守门 4: fit-to-content ===
  test('4. fit-to-content: 工具栏 Maximize2 按钮, viewport 重置到 minX/minY', async ({ page }) => {
    // 先 pan + zoom 让 viewport 偏离默认
    const zoomIn = page.locator('[data-testid="canvas-toolbar"] button[title="Zoom in (+)"]');
    await zoomIn.click();
    await zoomIn.click();
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('144%'); // 1.2^2 = 1.44
    // pan 一下
    const svg = page.locator('[data-testid="canvas-svg"]');
    const box = await svg.boundingBox();
    if (!box) throw new Error('canvas-svg no bounding box');
    await page.keyboard.down('Shift');
    await page.mouse.move(box.x + 600, box.y + 400);
    await page.mouse.down({ button: 'left' });
    await page.mouse.move(box.x + 100, box.y + 100, { steps: 8 });
    await page.mouse.up({ button: 'left' });
    await page.keyboard.up('Shift');

    // fit-to-content
    await page.locator('[data-testid="canvas-toolbar"] button[title="Fit to content (1)"]').click();
    // toolbar zoom% 应 ≤ 100% (per CanvasView.tsx:390 Math.min(1200/..., 800/..., 1))
    const toolbar = page.locator('[data-testid="canvas-toolbar"]');
    const text = await toolbar.textContent();
    const match = text?.match(/(\d+)%/);
    expect(match).toBeTruthy();
    const zoomPct = Number(match![1]);
    expect(zoomPct).toBeLessThanOrEqual(100);
    expect(zoomPct).toBeGreaterThan(0);
  });

  // === 守门 5: ?highlight= URL 自动 pan/zoom (per CanvasView line 62-71 useEffect + design doc §3.5) ===
  test('5. ?highlight=el-wi-002 URL: minimap viewport rect x 偏离 0 (useEffect auto-pan 触发)', async ({ page }) => {
    // 直接 navigate with ?highlight=el-wi-002 (CanvasView line 62-71 自动 pan/zoom 到 element 中心)
    await page.goto(CANVAS_URL + '?highlight=el-wi-002');
    await expect(page.locator('[data-testid="canvas-svg"]')).toBeVisible();
    await expect(page.locator('[data-testid="canvas-element-el-wi-002"]')).toBeVisible();
    // 默认 viewport=(0,0,1) 时 minimap viewport rect x=0. highlight 后 CanvasView useEffect
    //   setViewport({ x: targetX - 600/zoom/2, y: targetY - 400/zoom/2 })
    //   el-wi-002 center = (30+100, 180+45) = (130, 225)
    //   new viewport.x = 130 - 300 = -170
    // 等 useEffect 跑完 (React 18 微任务, wait 500ms 余量)
    await page.waitForTimeout(500);
    const rect = await readMinimapViewportRect(page);
    // viewport.x = -170, minimap rect 起点 = viewport.x
    expect(rect.x).toBeLessThan(0);
  });

  // === 守门 6: 选区删除 ===
  test('6. 选区删除: 选中 1 element + 工具栏 trash → store 0 该 id', async ({ page }) => {
    const wiCard = page.locator('[data-testid="canvas-element-el-wi-001"]');
    await expect(wiCard).toBeVisible();
    // 单击选中 (CanvasView line 117 setSelected([el.id]))
    await wiCard.click();
    // 工具栏出现 trash 按钮 (CanvasView line 396 conditional)
    const trashBtn = page.locator('[data-testid="canvas-toolbar"] button[title="Delete"]');
    await expect(trashBtn).toBeVisible();
    await trashBtn.click();
    // element 被删 (store.deleteCanvasElement → DOM unmount)
    await expect(wiCard).toHaveCount(0);
  });

  // === 守门 7: minimap viewport rect 跟随 viewport 变化 ===
  test('7. minimap viewport rect: zoom 1 → 1.2 后 width/height 缩小 (因可见范围更窄)', async ({ page }) => {
    const rect0 = await readMinimapViewportRect(page);
    // 默认 1200x800 / zoom=1 = 1200/1=1200 宽
    expect(rect0.width).toBeCloseTo(1200, 0);
    expect(rect0.height).toBeCloseTo(800, 0);

    // zoom in 一次 → 1.2x → 1200/1.2=1000 宽
    await page.locator('[data-testid="canvas-toolbar"] button[title="Zoom in (+)"]').click();
    await expect(page.locator('[data-testid="canvas-toolbar"]')).toContainText('120%');

    const rect1 = await readMinimapViewportRect(page);
    expect(rect1.width).toBeCloseTo(1000, 0); // 1200/1.2=1000
    expect(rect1.height).toBeCloseTo(666.67, 0); // 800/1.2 ≈ 666.67
  });
});
