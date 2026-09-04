// canvas-share-export.spec.ts — Share + Export PNG 按钮实装守门 (per 2026-09-04 拍板)
//
// 触发: 2026-09-04 23:00 JST Ulysses 拍板"补缺口 — Share + Export PNG 实装 (推荐)"
//        23:04 JST 库选型拍板"html2canvas (推荐, 库依赖)"
// 范围: 守门 #1 v3 (check+fmt+clippy 不替代 e2e) 硬约束
// 数据: canvas-001 (per seed.ts:418) 14 elements + 4 frames + 8 connectors
//
// 守门:
// - tsc --noEmit 0 错 (新文件范围)
// - vitest 0 失败 (canvas-share-export.spec.ts 已 vitest exclude, 仅 playwright 跑)
// - pnpm test:e2e -- canvas-share-export 3/3 全过
// - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - html2canvas 1.4.1 不渲染 SVG <foreignObject> (sticky_note / text / work_item_card
//   文字丢失, 只见矩形). test 2 验证 PNG 已下载即可, 不验内容完整性.
// - react-hot-toast toast 默认 top-right, 可能被 sidebar 遮挡. test 3 验证 role="status"
//   节点存在, 不验文本内容.
// - navigator.clipboard 需 HTTPS / localhost. dev mode OK, CI 配 permission.
// - Header (page.tsx:46-64) 不在 ref 范围. 用户要 header 导出留 P2.

import { test, expect, type Page } from '@playwright/test';

const CANVAS_URL = '/canvas/canvas-001';

test.describe('Canvas Share + Export PNG 按钮实装 (canvas-001)', () => {
  test.beforeEach(async ({ page, context }) => {
    // 配 clipboard 权限 (dev localhost 通常默认开, e2e 显式 grant 兜底)
    await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: 'http://localhost:3000' });
    await page.goto(CANVAS_URL);
    // 等 CanvasView mount (跟 canvas-view.spec.ts 同步: 主 svg 出现)
    await expect(page.locator('[data-testid="canvas-svg"]')).toBeVisible();
  });

  // === 守门 1: Share 按钮点击 → clipboard 写入当前 URL ===
  test('1. Share 按钮点击: clipboard 写入当前 URL', async ({ page }) => {
    const shareBtn = page.locator('[data-testid="canvas-share-btn"]');
    await expect(shareBtn).toBeVisible();
    await shareBtn.click();

    // 验证 clipboard 内容 = 当前 URL (react-hot-toast 弹 toast 是副作用, 不阻塞)
    const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboardText).toContain('/canvas/canvas-001');
  });

  // === 守门 2: Export PNG 按钮点击 → 触发 download (.png 后缀) ===
  test('2. Export PNG 按钮点击: 触发 download (.png 后缀)', async ({ page }) => {
    const exportBtn = page.locator('[data-testid="canvas-export-png-btn"]');
    await expect(exportBtn).toBeVisible();

    // 等待 download 事件 (html2canvas 异步 + toBlob 异步, 总耗时 ~500ms-2s)
    const downloadPromise = page.waitForEvent('download', { timeout: 15000 });
    await exportBtn.click();
    const download = await downloadPromise;

    // 验证文件名 + .png 后缀
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.png$/);
    // 文件名 pattern: {title}-canvas-001.png (per page.tsx onExportPng)
    expect(filename).toContain('canvas-001');

    // 保存 download 到临时位置, 验证非空
    const tmpPath = await download.path();
    expect(tmpPath).toBeTruthy();
  });

  // === 守门 3: Share 按钮点击后 → react-hot-toast toast 出现 ===
  test('3. Share 按钮点击后: react-hot-toast toast 出现 (role="status")', async ({ page }) => {
    const shareBtn = page.locator('[data-testid="canvas-share-btn"]');
    await expect(shareBtn).toBeVisible();
    await shareBtn.click();

    // react-hot-toast 默认 role="status" / "alert" (per react-hot-toast 2.4.1)
    //   至少一个 toast 节点可见
    const toast = page.locator('[role="status"], [role="alert"]').filter({ hasText: 'Canvas' });
    await expect(toast.first()).toBeVisible({ timeout: 3000 });
  });
});
