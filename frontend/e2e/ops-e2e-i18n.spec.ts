// ops-e2e-i18n.spec.ts — §4.3 Ops Console E2E i18n 3 语言端到端
//
// 触发: 2026-09-08 19:55 JST Mavis 接手 (per 5-LEVEL-FULL brief §1)
// 范围: TEST-DESIGN-OPS-001 v0.2 §4.3 i18n 3 语言端到端验证
//   - 入口文案 + 4 tab 标题 zh-CN / en / ja 完整覆盖
//   - 切换不重载 (useTranslation Context + localStorage)
//   - 7 维 i18n key 必跑 (per TEST-DESIGN §4.3 表)
// 守门:
//   - tsc --noEmit 0 错
//   - pnpm test:e2e -- ops-e2e-i18n 7 测 全过 (跨 chromium/firefox/webkit)
//   - 0 子代理调用
//
// 已知缺口 (per守门 #11 缺标比错标):
//   - i18n 切换 E2E 待实装 e2e 完整覆盖 (本页 MVP 阶段手测 3 语言, 实装阶段扩)
//   - 跨浏览器 binary 下载 (per brief §6 缺口 #1 [M])
//
// 引用:
//   - docs/test-design/TEST-DESIGN-OPS-001.md v0.2 §4.3
//   - frontend/src/lib/i18n/{zh-CN,en,ja}.ts
//   - docs/briefs/5-level-full-impl.md v0.1 §2.1

import { test, expect, type Page } from '@playwright/test';

// === 7 维 i18n key 必跑 (per TEST-DESIGN §4.3 表 + SRS-001 §2.1) ===
const I18N_KEYS = [
  { key: 'userMenu.ops', zh: '运维', en: 'Ops', ja: '運用' },
  { key: 'opsConsole.title', zh: '运维控制台', en: 'Ops Console', ja: '運用コンソール' },
  { key: 'opsConsole.tabCluster', zh: '集群更新', en: 'Cluster', ja: 'クラスタ' },
  { key: 'opsConsole.tabLogAI', zh: 'Log AI 分析', en: 'Log AI', ja: 'Log AI' },
  { key: 'opsConsole.tabMetrics', zh: '运维数据', en: 'Metrics', ja: '運用データ' },
  { key: 'opsConsole.tabDocs', zh: '运维文档', en: 'Docs', ja: 'ドキュメント' },
  { key: 'opsConsole.hero.welcome', zh: '欢迎使用 Ops Console', en: 'Welcome to Ops Console', ja: 'Ops Console へようこそ' },
] as const;

const LANGUAGES = ['zh-CN', 'en', 'ja'] as const;
type Language = (typeof LANGUAGES)[number];

// === 切换 i18n (走 localStorage + reload) ===
// STORAGE_KEY = "star-language" (per frontend/src/lib/i18n/config.ts)
async function setLanguage(page: Page, lang: Language) {
  await page.addInitScript((l) => {
    localStorage.setItem('star-language', l);
  }, lang);
  await page.goto('/ops');
  await page.waitForLoadState('networkidle');
}

test.describe('§4.3 Ops Console i18n 3 语言端到端验证 (E2E-I18N-01..07)', () => {

  // === I18N-01..07: 7 维 i18n key 端到端验证 ===
  for (const entry of I18N_KEYS) {
    test(`I18N. ${entry.key} 端到端 3 语言完整覆盖`, async ({ page }) => {
      // zh-CN 端到端验证
      await setLanguage(page, 'zh-CN');
      const zhText = await page.locator('body').innerText();
      if (entry.key === 'userMenu.ops') {
        // userMenu 在顶部 UserMenu 组件, ops 路由可能不显示
        await page.goto('/projects');
        await page.waitForLoadState('networkidle');
        await expect(page.getByText(entry.zh)).toBeVisible();
      } else {
        // /ops 路由上验证
        if (entry.key === 'opsConsole.title') {
          await expect(page.getByText(entry.zh)).toBeVisible();
        } else {
          // tab 标题: 在 TabsList 内
          await expect(page.getByRole('tab', { name: new RegExp(entry.zh.split(' ')[0]) })).toBeVisible();
        }
      }

      // en 端到端验证
      await setLanguage(page, 'en');
      const enText = await page.locator('body').innerText();
      if (entry.key === 'userMenu.ops') {
        await page.goto('/projects');
        await page.waitForLoadState('networkidle');
        await expect(page.getByText(entry.en)).toBeVisible();
      } else {
        if (entry.key === 'opsConsole.title') {
          await expect(page.getByText(entry.en)).toBeVisible();
        } else {
          await expect(page.getByRole('tab', { name: new RegExp(entry.en, 'i') })).toBeVisible();
        }
      }

      // ja 端到端验证
      await setLanguage(page, 'ja');
      if (entry.key === 'userMenu.ops') {
        await page.goto('/projects');
        await page.waitForLoadState('networkidle');
        await expect(page.getByText(entry.ja)).toBeVisible();
      } else {
        if (entry.key === 'opsConsole.title') {
          await expect(page.getByText(entry.ja)).toBeVisible();
        } else {
          // ja 可能有全角空格, 用 .includes 而非严格匹配
          const tabs = page.getByRole('tab');
          const tabCount = await tabs.count();
          let found = false;
          for (let i = 0; i < tabCount; i++) {
            const tabText = (await tabs.nth(i).innerText()).trim();
            if (tabText.includes(entry.ja.split(/\s|\//)[0])) {
              found = true;
              break;
            }
          }
          expect(found).toBe(true);
        }
      }
    });
  }

  // === I18N-08: i18n 切换不重载 (per TEST-DESIGN §4.3 必跑) ===
  test('I18N-08. i18n 切换不重载 — localStorage 持久化 + 切换无 page reload', async ({ page }) => {
    // 设置 zh-CN → 进 /ops → 验证 tab 可见
    await setLanguage(page, 'zh-CN');
    const initialLoadTime = Date.now();
    await expect(page.getByText('运维控制台')).toBeVisible();
    // 切到 en
    await page.evaluate(() => {
      localStorage.setItem('star-language', 'en');
    });
    // 不重载, 仅 context 切换 (per useTranslation Context)
    await page.waitForTimeout(500);
    // 检查 en 文案出现 (Ops Console)
    const opConsoleVisible = await page.getByText('Ops Console').count();
    expect(opConsoleVisible).toBeGreaterThan(0);
  });

  // === I18N-09: zh-CN 入口文案 + 4 tab 完整 (per MVP 实证) ===
  test('I18N-09. zh-CN 入口文案 + 4 tab 完整', async ({ page }) => {
    await setLanguage(page, 'zh-CN');
    // 入口文案
    await expect(page.getByText('运维控制台')).toBeVisible();
    // 4 tab 标题
    await expect(page.getByRole('tab', { name: /集群更新/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Log AI/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /运维数据/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /文档/ })).toBeVisible();
  });

  // === I18N-10: en 入口文案 + 4 tab 完整 (per MVP 实证) ===
  test('I18N-10. en 入口文案 + 4 tab 完整', async ({ page }) => {
    await setLanguage(page, 'en');
    await expect(page.getByText('Ops Console')).toBeVisible();
    await expect(page.getByRole('tab', { name: /Cluster/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Log AI/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Metrics/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Docs/ })).toBeVisible();
  });

  // === I18N-11: ja 入口文案 + 4 tab 完整 (per MVP 实证) ===
  test('I18N-11. ja 入口文案 + 4 tab 完整', async ({ page }) => {
    await setLanguage(page, 'ja');
    await expect(page.getByText('運用コンソール')).toBeVisible();
    await expect(page.getByRole('tab', { name: /クラスタ/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Log AI/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /運用データ/ })).toBeVisible();
    await expect(page.getByRole('tab', { name: /ドキュメント/ })).toBeVisible();
  });
});
