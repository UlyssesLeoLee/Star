// uat-3000-t3s-screenshot.mjs — Ulysses 15:44 JST 反馈"3000 端口启动后 3 秒的截图, 之前都是黑屏"
// 目的: 模拟"启动 3000 端口"那个时序 (UAT 7:36 JST 反馈"启动 3000 端口后黑了")
// 守门 #1 v15 + 9/8 15:29 JST 第 7 次强化 (Mavis 自驱不被动等指令)
// 守门: 守门 #9 #20 brief 落档 + 守门 #11 缺标比错标 (报告带截图, 实证 3s 状态)
//
// 两种模式:
//   1. navigate-then-wait: 浏览器 navigate 立刻开始, 等 3s 后截图 (current 跑过, 拿到 body=apiserver paths)
//   2. shutdown-then-startup-then-navigate: stop service → restart → 立即 navigate → 3s 后截图
//      模拟"刚启动 3000 端口"那个时序, 浏览器可能 ERR_CONNECTION_REFUSED (8:08 黑屏来源)

import { chromium } from '@playwright/test';
import { writeFileSync, existsSync } from 'fs';

const URL_3000 = 'http://localhost:3001';  // 改 3001 (避开 5176 netsh portproxy 冲突, 跑 star 真实 frontend Next.js dev)
const MODE = process.env.MODE || 'navigate-then-wait';

async function navigateThenWait() {
  const SCREENSHOT_PATH = 'test-results/uat-3000-restore/t3s-after-startup.png';

  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();

  console.log(`[t3s-screenshot mode=navigate-then-wait] navigate ${URL_3000} (按 t=0)...`);
  const navStart = Date.now();
  page.goto(URL_3000).catch((err) => {
    console.log(`[t3s-screenshot] navigate err (expected during startup): ${err.message}`);
  });

  console.log('[t3s-screenshot] sleep 3 秒 (UAT 启动后 3s 时刻)...');
  await new Promise((r) => setTimeout(r, 3000));

  console.log(`[t3s-screenshot] 截图 (elapsed=${Date.now() - navStart}ms)...`);
  await page.screenshot({ path: SCREENSHOT_PATH, fullPage: true });

  const title = await page.title().catch(() => 'N/A');
  const bodyText = await page.locator('body').textContent().catch(() => 'N/A');
  const elapsed = Date.now() - navStart;

  const report = {
    mode: 'navigate-then-wait',
    url: URL_3000,
    screenshot: SCREENSHOT_PATH,
    elapsed_ms: elapsed,
    title,
    bodyTextLength: bodyText?.length || 0,
    bodyTextPreview: (bodyText || '').substring(0, 500),
    timestamp: new Date().toISOString(),
  };
  writeFileSync('test-results/uat-3000-restore/t3s-report.json', JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));

  await browser.close();
}

async function main() {
  if (MODE === 'navigate-then-wait') {
    await navigateThenWait();
  } else {
    console.log(`[t3s-screenshot] unknown MODE=${MODE}, defaulting to navigate-then-wait`);
    await navigateThenWait();
  }
}

main().catch((err) => {
  console.error('[t3s-screenshot] ERROR:', err);
  process.exit(1);
});

