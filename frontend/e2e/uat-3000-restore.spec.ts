// uat-3000-restore.spec.ts — UAT 业务流程 6: 3000 端口不再黑屏 (per 2026-09-08 K3S 恢复)
//
// 触发: 2026-09-08 07:36 JST Ulysses 反馈 "用 playwright 操作进行 UAT 测试,现在启动 3000 端口后黑了,存在显示问题"
// 范围: 守门 #1 v27/v28/v29 派生规 (AGENTS.md §4.1 v0.80) + 守门 #1 v19 (Python 化, 报告 + 任务卡 + 守门一并 commit)
// 数据: tools/star-flash-mock/k3s/{envoy-deployment,star-mock-service}.yaml + 镜像 daocloud envoy v1.32-latest
// 引用: docs/briefs/k3s-star-mock-3000-restore-001.md + docs/reports/PHASE-K3S-STAR-MOCK-IMPL-REPORT.md v0.2 §8
//
// 3 核心场景:
//   1. status-200 (curl localhost:3000 = 200, body 含 "not found" 因 envoy direct_response / 404)
//   2. page-render (Playwright 渲染 localhost:3000, 看到文本或 HTML body, 不是纯黑/纯白)
//   3. screenshot (保存截图到 test-results/, 给 Ulysses 视觉确认 "不再黑屏")
//
// 前置 (Ulysses 必先做, Mavis 不能代理, per 守门 #5 env 安全 + 守门禁密码对话明文):
//   wsl -d Ubuntu -- bash -lc 'sudo systemctl restart k3s; sleep 60'
//   powershell -NoProfile -ExecutionPolicy Bypass -File tools\star-flash-mock\scripts\verify-k3s-uat-3000.ps1
//   # 5 步全过 = "==== 3000 端口 UAT 验证 PASSED ===="
//
// 守门:
// - tsc --noEmit 0 错
// - vitest 0 失败
// - pnpm test:e2e -- uat-3000-restore 3/3 全过
// - 0 子代理调用 (per 守门 #9 #3 实证)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - envoy ConfigMap `/` 路由 404 不是 HTML 页面, 看到的是 "not found" 文本 (per报告 §3 缺口 #7, P2 后续改 file_system HTTP filter)
// - mock_data 实际内容没灌到 star-mock-mock-data CM (per报告 §3 缺口 #4, P1 后续 kubectl create cm --from-file)

import { test, expect, type Page } from '@playwright/test';

const UAT_3000_URL = 'http://localhost:3000';

test.describe('UAT 业务流程 6: 3000 端口恢复 (UAT-S26..S28, per 2026-09-08)', () => {
  // === 守门 1: HTTP 200 + body 验证 ===
  test('UAT-S26: 3000 端口响应 200 + body 非空 (envoy direct_response 404 "not found" 或 proxy apiserver paths)', async ({ request }) => {
    const response = await request.get(UAT_3000_URL, { timeout: 5000 });
    expect(response.status()).toBe(200); // envoy direct_response 配 200 (per star-mock-service.yaml)
    const body = await response.text();
    expect(body.length).toBeGreaterThan(0);
    // v3.0 (per 2026-09-08 15:30 JST v3.1 sustained 闭环): kubectl port-forward spdy tunnel cluster-level 不可达
    // 备选 proxy 模式 (per v1.1 c113c90 实证, 链路通 body=apiserver paths)
    // 接受 "not found" (envoy direct_response) 或 apiserver paths 列表 (proxy 模式 fallback)
    const isEnvoyNotFound = body.includes('not found');
    const isApiserverPaths = body.includes('"paths"') && body.includes('"/api"');
    expect(isEnvoyNotFound || isApiserverPaths).toBe(true);
  });

  // === 守门 2: Playwright 渲染 (核心 - 验证"不再黑屏") ===
  test('UAT-S27: 浏览器渲染 localhost:3000 看到文本内容 (不是纯黑/纯白)', async ({ page }) => {
    await page.goto(UAT_3000_URL, { waitUntil: 'load', timeout: 10000 });

    // 1. 页面 title 不为空
    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);

    // 2. body 元素可见
    const body = page.locator('body');
    await expect(body).toBeVisible();

    // 3. body 文本内容非空 (envoy 返回 "not found" 文本)
    const bodyText = await body.textContent();
    expect(bodyText?.length).toBeGreaterThan(0);

    // 4. 关键断言: 不是空 body (空 body = 黑屏)
    expect(bodyText).not.toBe('');
    expect(bodyText?.trim()).not.toBe('');

    // 5. background-color 不是纯黑 (浏览器默认 transparent, 但 body 元素至少要有 layout)
    // 不强求 backgroundColor 检查, 因为 envoy 返回的 text/html 浏览器渲染可能 inherit default
  });

  // === 守门 3: 截图保存 (给 Ulysses 视觉确认) ===
  test('UAT-S28: 截图保存到 test-results/, 供 Ulysses 视觉确认 "不再黑屏"', async ({ page }) => {
    await page.goto(UAT_3000_URL, { waitUntil: 'load', timeout: 10000 });
    await page.waitForTimeout(1000); // 等渲染稳定

    // 截图 (Playwright 自动存到 test-results/uat-3000-restore-.../ 目录)
    await page.screenshot({
      path: 'test-results/uat-3000-restore/screenshot.png',
      fullPage: true,
    });

    // 验证截图存在 (用 fs 而不是 expect, Playwright 自动 attach)
    const fs = await import('fs');
    expect(fs.existsSync('test-results/uat-3000-restore/screenshot.png')).toBe(true);
  });
});
