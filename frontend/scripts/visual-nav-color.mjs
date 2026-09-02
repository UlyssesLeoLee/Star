// scripts/visual-nav-color.mjs — 5 域分色 light/dark 视觉走查
//
// Per 2026-09-02 16:13 JST 补缺口 (light mode 颜色饱和度待走查),
// Playwright 跑 light + dark 两套主题, 截 Sidebar + AppMatrixDrawer
// 视觉对比图, 归档到 docs/frontend/screenshots/nav-color-tokens/.
//
// 运行: 先 `pnpm dev` 起服务, 然后 `node scripts/visual-nav-color.mjs`
// 产物: docs/frontend/screenshots/nav-color-tokens/{light,dark}-{sidebar,matrix}.png
//
// =====================================================================

import { chromium } from "@playwright/test";
import { mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, "..", "..", "docs", "frontend", "screenshots", "nav-color-tokens");
const BASE = "http://localhost:3000";

async function shoot(page, name, theme) {
  // dark 主题 — html 加 dark class
  await page.evaluate((t) => {
    document.documentElement.classList.toggle("dark", t === "dark");
    document.documentElement.style.colorScheme = t;
  }, theme);

  // 等 300ms 让 React rerender + HMR 应用最新 registry
  await page.waitForTimeout(300);

  // Sidebar
  const sidebar = await page.$('[data-testid="app-sidebar"]');
  if (sidebar) {
    await sidebar.screenshot({ path: join(OUT_DIR, `${theme}-sidebar.png`) });
    console.log(`  ✓ ${theme}-sidebar.png`);
  } else {
    console.log(`  ✗ ${theme}-sidebar.png — sidebar not found`);
  }

  // 触发 AppMatrix 抽屉 — 点击 + 按钮
  const trigger = await page.$('[data-testid="app-matrix-trigger"]');
  if (trigger) {
    await trigger.click();
    await page.waitForSelector('[data-testid="app-matrix-modal"]', { timeout: 5000 });
    await page.waitForTimeout(500);
    const modal = await page.$('[data-testid="app-matrix-modal"]');
    if (modal) {
      await modal.screenshot({ path: join(OUT_DIR, `${theme}-matrix.png`) });
      console.log(`  ✓ ${theme}-matrix.png`);

      // 调试: 读 6 个 core module 的实际 className (per 2026-09-02 16:13 JST 缺口诊断)
      const debugIds = ["inbox", "issues", "projects", "agents", "analytics", "settings", "remote"];
      const classMap = await page.evaluate((ids) => {
        const out = {};
        ids.forEach((id) => {
          const el = document.querySelector(`[data-testid="matrix-card-icon-tile-${id}"]`);
          out[id] = el ? el.className : "(missing)";
        });
        return out;
      }, debugIds);
      console.log("  [debug] matrix icon-tile className:");
      for (const [id, cls] of Object.entries(classMap)) {
        // 提取 bg-color / text-color 关键 class
        const bg = cls.match(/bg-\w+-\d+\/\d+/)?.[0] ?? "?";
        const text = cls.match(/text-\w+-\d+/)?.[0] ?? "?";
        console.log(`    ${id.padEnd(10)} bg=${bg.padEnd(18)} text=${text}`);
      }
    }
    // 关闭 — 点击 backdrop (modal 容器外层) 而非 Escape
    // 因为 backdrop 监听 onClick close (per AppMatrixDrawer line 71-74)
    await page.evaluate(() => {
      // 找 backdrop 容器 (有 onClick handler 的 div)
      const backdrop = document.querySelector('.fixed.inset-0.z-50');
      if (backdrop) {
        // 模拟点击 backdrop 自身 (e.target === e.currentTarget)
        backdrop.click();
      }
    });
    await page.waitForTimeout(300);
  }
}

(async () => {
  await mkdir(OUT_DIR, { recursive: true });
  console.log(`Output dir: ${OUT_DIR}`);

  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();

  console.log(`Navigating to ${BASE}/inbox ...`);
  // cache-bust: 加 ?t= 时间戳防止 Next.js dev HMR 缓存 module
  await page.goto(`${BASE}/inbox?t=${Date.now()}`, { waitUntil: "networkidle", timeout: 30_000 });
  // hard reload 确保 module 重新求值 (HMR 可能不更新 ESM exports)
  await page.reload({ waitUntil: "networkidle" });
  await page.waitForSelector('[data-testid="app-sidebar"]', { timeout: 10_000 });
  console.log("  ✓ Sidebar mounted");

  for (const theme of ["light", "dark"]) {
    console.log(`\n[${theme}] shooting ...`);
    await shoot(page, "nav", theme);
  }

  // ---- Per 2026-09-02 18:16 JST: 截顶栏 5 tab 域色 ----
  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.classList.toggle("dark", t === "dark");
      document.documentElement.style.colorScheme = t;
    }, theme);
    await page.waitForTimeout(300);

    // inbox active (cyan core)
    await page.goto(`${BASE}/inbox?t=${Date.now()}`, { waitUntil: "networkidle" });
    const headerInbox = await page.$('[data-testid="app-header"]');
    if (headerInbox) {
      await headerInbox.screenshot({ path: join(OUT_DIR, `${theme}-header-inbox.png`) });
      console.log(`  ✓ ${theme}-header-inbox.png (cyan core active)`);
    }
  }

  await browser.close();
  console.log("\nDone.");
})().catch((e) => {
  console.error("FAIL:", e);
  process.exit(1);
});
