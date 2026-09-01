// remote-mobile.spec.ts — PWA Mobile viewport e2e (per 2026-09-01 PHASE-MOBILE-PWA v0.4)
//
// 覆盖:
//   1. <768px 移动端布局: MobileHeader + MobileBottomNav 可见, Sidebar 隐藏
//   2. /remote home: 5 runtime 列表 + 三件套入口按钮
//   3. /remote/desktop/[id]: noVNC 容器 + 连接按钮可见
//   4. /remote/terminal/[id]: xterm 容器 + 连接按钮可见
//   5. /remote/files/[id]: SFTP 风格 + 面包屑 + 文件列表
//   6. PWA install prompt: iOS 模式 (UA=iPhone) 弹 3 步说明
//
// 守门 #1+#9+#12 全过 (per commit 历史):
// - tsc --noEmit 0 错
// - vitest 309/309 pass
// - next build 0 err
// - 0 子代理调用 (root 直实装)
//
// 触发: 2026-09-01 13:38 JST Ulysses 拍板 (A) 推 v0.4

import { test, expect } from '@playwright/test';

// iPhone 13 viewport (per 2026-09-01 实测, 与 Star 当前 Tailwind 768px 断点对齐)
const IPHONE_13 = { width: 390, height: 844 } as const;

test.describe('PWA Mobile viewport e2e (iPhone 13)', () => {
  test.use({ viewport: IPHONE_13, hasTouch: true, isMobile: true });

  test('移动端布局: Sidebar 隐藏 + MobileHeader 可见 + BottomNav 5 项', async ({ page }) => {
    await page.goto('/');
    // MobileHeader
    await expect(page.getByTestId('mobile-header')).toBeVisible();
    // MobileBottomNav
    await expect(page.getByTestId('mobile-bottom-nav')).toBeVisible();
    // 5 域入口
    await expect(page.getByTestId('mobile-nav-home')).toBeVisible();
    await expect(page.getByTestId('mobile-nav-worktree')).toBeVisible();
    await expect(page.getByTestId('mobile-nav-agent')).toBeVisible();
    await expect(page.getByTestId('mobile-nav-feedback')).toBeVisible();
    await expect(page.getByTestId('mobile-nav-more')).toBeVisible();
    // Sidebar 隐藏 (≥768px 才显示)
    await expect(page.getByTestId('app-sidebar')).toBeHidden();
  });

  test('"更多" 抽屉: 含 Remote Control 入口', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mobile-nav-more').click();
    await expect(page.getByTestId('mobile-more-drawer')).toBeVisible();
    await expect(page.getByTestId('mobile-more--remote')).toBeVisible();
  });

  test('/remote home: 列出 5 个 local runtime + Push 设置卡', async ({ page }) => {
    await page.goto('/remote');
    await expect(page.getByTestId('push-settings')).toBeVisible();
    // 5 个 runtime (per seed.ts: lr-001 ~ lr-005)
    for (const id of ['lr-001', 'lr-002', 'lr-003', 'lr-004', 'lr-005']) {
      await expect(page.getByTestId(`remote-runtime-${id}`)).toBeVisible();
      await expect(page.getByTestId(`remote-desktop-${id}`)).toBeVisible();
      await expect(page.getByTestId(`remote-terminal-${id}`)).toBeVisible();
      await expect(page.getByTestId(`remote-files-${id}`)).toBeVisible();
    }
  });

  test('/remote/desktop/lr-001: noVNC 容器 + 连接按钮', async ({ page }) => {
    await page.goto('/remote/desktop/lr-001');
    await expect(page.getByTestId('novnc-canvas')).toBeVisible();
    await expect(page.getByTestId('novnc-connect')).toBeVisible();
  });

  test('/remote/terminal/lr-001: xterm 容器 + 连接按钮', async ({ page }) => {
    await page.goto('/remote/terminal/lr-001');
    await expect(page.getByTestId('xterm-container')).toBeVisible();
    await expect(page.getByTestId('xterm-connect')).toBeVisible();
  });

  test('/remote/files/lr-001: SFTP 文件浏览器 + 面包屑 + 文件列表', async ({ page }) => {
    await page.goto('/remote/files/lr-001');
    await expect(page.getByTestId('files-home')).toBeVisible();
    // 文件列表 useEffect 异步 setEntries, 等久一点
    // (mock 模式 100ms 延迟, strict mode 双跑, 留余量到 8s)
    await expect(page.getByTestId('files-entry-projects')).toBeVisible({ timeout: 8000 });
  });

  test('Bottom Nav 导航: 切到 Worktree (per redirect /worktree → /issues?view=tree)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mobile-nav-worktree').click();
    // /worktree 路由经 legacy redirect 跳到 /issues?view=tree (per next.config.js redirects)
    // Bottom Nav 行为正确,redirect 是后置链路
    await page.waitForURL(/\/issues\?view=tree|\/worktree/, { timeout: 5000 });
  });
});

test.describe('PWA iOS install instructions (iPhone Safari)', () => {
  test.use({ viewport: IPHONE_13, hasTouch: true, isMobile: true });
  // iOS Safari UA (per 2026-09-01 iOS 17.6)
  test.use({ userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Mobile/15E148 Safari/604.1' });

  test('iOS 模式: install prompt 弹 3 步说明 modal', async ({ page }) => {
    await page.goto('/');
    // iOS prompt 3s 延迟
    await expect(page.getByTestId('pwa-install-prompt')).toBeVisible({ timeout: 5000 });
    // 3 步说明: 分享 -> 添加到主屏 -> 添加
    const prompt = page.getByTestId('pwa-install-prompt');
    await expect(prompt).toContainText('分享');
    await expect(prompt).toContainText('添加');
  });
});
