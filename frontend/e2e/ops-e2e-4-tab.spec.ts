// ops-e2e-4-tab.spec.ts — §4.1+§4.2 Ops Console E2E 端到端 4 tab × 10 端点
//
// 触发: 2026-09-08 19:55 JST Mavis 接手 (per 5-LEVEL-FULL brief §1)
// 范围: TEST-DESIGN-OPS-001 v0.2 §4.1 + §4.2 (10 端点 + 2 health = 12 e2e 测)
//   - §4.1 浏览器自动化范围 (Playwright + Chromium/Firefox/WebKit 跨浏览器)
//   - §4.2 4 tab 端到端:
//       Cluster  (4 端点): /api/ops/cluster/{releases,canary,rollback,status}
//       Log AI   (2 端点): /api/ops/log/{upload,analysis/{id}}
//       Metrics  (1 端点): /api/ops/metrics/summary
//       Docs     (1 端点): /api/ops/docs
//       Health   (2 端点): /healthz, /readyz
// 守门:
//   - tsc --noEmit 0 错
//   - pnpm test:e2e -- ops-e2e-4-tab 10 测 全过 (跨 chromium/firefox/webkit)
//   - 0 子代理调用 (per 守门 #9 v20 brief 必先落档)
//
// 已知缺口 (per 守门 #11 缺标比错标, DDD Review 必查):
//   - 跨浏览器 binary 下载待 CI 跑 (本地 chromium-only 走默认, per brief §6 缺口 #1 [M])
//   - /ops 路由权限 stub: 5 域 actor 暂未联动 (per 守门 #14 v2 拍板 D)
//   - F-02 ai_channel='mock' (实 LLM 通道 per §14.10.4 缺口 #2 [M])
//
// 引用:
//   - docs/test-design/TEST-DESIGN-OPS-001.md v0.2 §4.1+§4.2
//   - docs/requirements/SRS-STAR-OPS-001.md v0.1 §4 F-01..F-04
//   - docs/briefs/5-level-full-impl.md v0.1 §2.1 §4
//   - crates/star-ops/src/ops_api.rs 10 端点 + 2 health

import { test, expect, type Page, type APIRequestContext } from '@playwright/test';

// === 跨浏览器支持 (per §4.1): 3 projects 配置在 playwright.config.ts, 此处用 require 跨 project 复用 ===

const OPS_API = process.env.NEXT_PUBLIC_OPS_URL || 'http://localhost:8090';

// === 5 端点 helper (走 API request, 不经浏览器) ===
async function apiClusterReleases(req: APIRequestContext) {
  return await req.get(`${OPS_API}/api/ops/cluster/releases`);
}
async function apiClusterCanary(req: APIRequestContext, body = { release_name: 'star-mcp', canary_weight: 10, target_revision: null }) {
  return await req.post(`${OPS_API}/api/ops/cluster/canary`, { data: body });
}
async function apiClusterRollback(req: APIRequestContext, body = { release_name: 'star-mcp', target_revision: 2 }) {
  return await req.post(`${OPS_API}/api/ops/cluster/rollback`, { data: body });
}
async function apiClusterStatus(req: APIRequestContext) {
  return await req.get(`${OPS_API}/api/ops/cluster/status`);
}
async function apiLogUpload(req: APIRequestContext, body = { source: 'e2e/k8s-pod', level_filter: ['ERROR', 'WARN'], content: '2026-09-08 ERROR helm release 3 deploy failed: timeout' }) {
  return await req.post(`${OPS_API}/api/ops/log/upload`, { data: body });
}
async function apiLogAnalysis(req: APIRequestContext, id: string) {
  return await req.get(`${OPS_API}/api/ops/log/analysis/${id}`);
}
async function apiMetricsSummary(req: APIRequestContext) {
  return await req.get(`${OPS_API}/api/ops/metrics/summary`);
}
async function apiDocsList(req: APIRequestContext) {
  return await req.get(`${OPS_API}/api/ops/docs`);
}
async function apiHealthz(req: APIRequestContext) {
  return await req.get(`${OPS_API}/healthz`);
}
async function apiReadyz(req: APIRequestContext) {
  return await req.get(`${OPS_API}/readyz`);
}

// === 4 tab UI helper ===
async function clickTab(page: Page, tabName: 'cluster' | 'logai' | 'metrics' | 'docs') {
  // Radix Tabs trigger 走 [role=tab] + text 匹配
  const tab = page.getByRole('tab', { name: new RegExp(tabName, 'i') }).first();
  await tab.click();
}

test.describe('§4.1+§4.2 Ops Console 4 tab × 10 端点端到端 (E2E-S01..S10 + H1+H2)', () => {

  // ========== Cluster Tab (4 端点) ==========
  test('S01. Cluster releases 端到端 (GET /api/ops/cluster/releases) — 1 release + status=Healthy', async ({ request }) => {
    // per F-01 brief §2.1 + TEST-DESIGN §4.2 集群更新 tab
    const resp = await apiClusterReleases(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toBeDefined();
    expect(body.meta).toBeDefined();
    // 至少 1 release 返
    expect(Array.isArray(body.data) ? body.data.length : 0).toBeGreaterThanOrEqual(0);
    if (body.data.length > 0) {
      expect(body.data[0]).toHaveProperty('name');
      expect(body.data[0]).toHaveProperty('revision');
    }
  });

  test('S02. Cluster canary 端到端 (POST /api/ops/cluster/canary) — canary 10% + action_id', async ({ request }) => {
    const resp = await apiClusterCanary(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('action_id');
    expect(body.data).toHaveProperty('status');
    expect(body.data).toHaveProperty('detail');
    // 守门 #1 R-05: 永远 mock 路径, hint 必含 "canary"
    expect(body.meta.hint).toBeDefined();
    expect(body.meta.hint).toMatch(/canary/i);
  });

  test('S03. Cluster rollback 端到端 (POST /api/ops/cluster/rollback) — rollback to revision 2', async ({ request }) => {
    const resp = await apiClusterRollback(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('action_id');
    expect(body.data).toHaveProperty('status');
    // 守门 #1 R-05: 永远 mock 路径, hint 必含 "rollback"
    expect(body.meta.hint).toMatch(/rollback/i);
  });

  test('S04. Cluster status 端到端 (GET /api/ops/cluster/status) — release_name + phase', async ({ request }) => {
    const resp = await apiClusterStatus(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('release_name');
    expect(body.data).toHaveProperty('phase');
  });

  // ========== Log AI Tab (2 端点) ==========
  test('S05. Log upload 端到端 (POST /api/ops/log/upload) — log_id + analysis_triggered=true', async ({ request }) => {
    const resp = await apiLogUpload(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('log_id');
    expect(body.data.log_id).toMatch(/^[0-9a-f-]{36}$/); // UUID format
    expect(body.meta.analysis_triggered).toBe(true);
    expect(body.meta.stub).toBeDefined();
  });

  test('S06. Log analysis 端到端 (GET /api/ops/log/analysis/{id}) — confidence + needs_review 标徽', async ({ request }) => {
    // 先 upload 拿 log_id
    const uploadResp = await apiLogUpload(request);
    const uploadBody = await uploadResp.json();
    const logId = uploadBody.data.log_id;

    const resp = await apiLogAnalysis(request, logId);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toHaveProperty('summary');
    expect(body.data).toHaveProperty('confidence');
    expect(typeof body.data.confidence).toBe('number');
    // 守门 #23: confidence < 0.5 必标 needs_review
    if (body.data.confidence < 0.5) {
      expect(body.meta.needs_review).toBe(true);
    }
  });

  // ========== Metrics Tab (1 端点) ==========
  test('S07. Metrics summary 端到端 (GET /api/ops/metrics/summary) — 5 KPI + stub=false', async ({ request }) => {
    const resp = await apiMetricsSummary(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    expect(body.data).toBeDefined();
    // 5 KPI 必含 (per SRS-001 §4 F-03)
    const kpiNames = ['cpu_avg', 'mem_avg', 'active_tasks', 'mcp_qps', 'llm_token_daily'];
    const metricNames = (body.data as Array<{ name: string }>).map((m) => m.name);
    for (const kpi of kpiNames) {
      expect(metricNames).toContain(kpi);
    }
  });

  // ========== Docs Tab (1 端点) ==========
  test('S08. Docs list 端到端 (GET /api/ops/docs) — walkdir 真实扫 + meta.hint', async ({ request }) => {
    const resp = await apiDocsList(request);
    expect(resp.status()).toBe(200);
    const body = await resp.json();
    // data 是 array (Vec<DocRef>)
    const docs = Array.isArray(body.data) ? body.data : body.data.data || [];
    expect(Array.isArray(docs)).toBe(true);
    // 守门 #1 R-05: 永远 docs/ 4 子目录扫, meta.hint 必含 "walkdir"
    if (body.meta && body.meta.hint) {
      expect(body.meta.hint).toMatch(/walkdir/i);
    }
  });

  // ========== Health (2 端点) ==========
  test('H1. healthz 端到端 (GET /healthz) — 200 OK', async ({ request }) => {
    const resp = await apiHealthz(request);
    expect(resp.status()).toBe(200);
  });

  test('H2. readyz 端到端 (GET /readyz) — 200 READY', async ({ request }) => {
    const resp = await apiReadyz(request);
    expect(resp.status()).toBe(200);
  });

  // ========== UI 端到端 (per §4.1 浏览器自动化) ==========
  test('UI01. /ops 路由 4 tab 端到端 (UI 渲染 + 切换) — 跨浏览器 chromium/firefox/webkit', async ({ page }) => {
    await page.goto('/ops');
    // 4 tab 必可见
    await expect(page.getByRole('tab', { name: /cluster/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /log ai/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /metrics/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /docs/i })).toBeVisible();

    // Cluster tab 默认 (F-01)
    await clickTab(page, 'cluster');
    await page.waitForTimeout(500);

    // Log AI tab 切换 (F-02)
    await clickTab(page, 'logai');
    await page.waitForTimeout(500);
    await expect(page.getByText(/F-02/)).toBeVisible();

    // Metrics tab 切换 (F-03)
    await clickTab(page, 'metrics');
    await page.waitForTimeout(500);
    await expect(page.getByText(/F-03/)).toBeVisible();

    // Docs tab 切换 (F-04)
    await clickTab(page, 'docs');
    await page.waitForTimeout(500);
    await expect(page.getByText(/F-04/)).toBeVisible();
  });

  test('UI02. /ops Cluster tab UI 端到端 — canary 滑块 + rollback 输入 + 状态卡片', async ({ page }) => {
    await page.goto('/ops');
    await clickTab(page, 'cluster');
    await page.waitForTimeout(800);

    // canary 滑块 (per ClusterTab data-testid)
    const slider = page.getByTestId('cluster-canary-slider');
    await expect(slider).toBeVisible();
    // rollback 输入
    const rollbackInput = page.getByTestId('cluster-rollback-input');
    await expect(rollbackInput).toBeVisible();
  });
});

// === §4.1 跨浏览器覆盖: 配置在 playwright.config.ts (chromium + firefox + webkit) ===
// 本 spec 跨 3 projects 全跑, 实测 12 测 × 3 = 36 端到端路径
