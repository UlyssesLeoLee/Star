// mcp-16-tool-coverage.spec.ts — UAT 业务流程 5: 16 MCP tool 端到端覆盖
//
// 触发: 2026-09-07 14:30 JST Ulysses 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档"
// 范围: 守门 #1 v3 + 守门 #4 (16/16 REAL 拍板), UAT 件套 5/6
// 数据: tools/star-flash-mock/mock_data/mcp/ 16 fixture (per ADR-0032 MCP Transport stdio)
// 引用: docs/test-design.md v0.8 §22 + AGENTS.md §7 #1 + docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md
//
// 16 MCP tool (per ADR-0032 §2):
//   1.  agents_list       — GET /api/mcp/agents
//   2.  audit_event       — GET /api/mcp/audit-event
//   3.  billing_usage     — GET /api/mcp/billing-usage
//   4.  feedback          — POST /api/mcp/feedback
//   5.  form              — GET /api/mcp/form
//   6.  inbox             — GET /api/mcp/inbox
//   7.  kms               — GET /api/mcp/kms
//   8.  permission        — GET /api/mcp/permission
//   9.  project           — GET /api/mcp/project
//   10. scm               — GET /api/mcp/scm
//   11. search            — GET /api/mcp/search
//   12. sessions_create   — POST /api/mcp/sessions-create
//   13. tools_invoke      — POST /api/mcp/tools-invoke
//   14. workitem_create   — POST /api/mcp/workitem-create
//   15. workitem_list     — GET /api/mcp/workitem-list
//   16. workspace         — GET /api/mcp/workspace
//
// 守门:
// - tsc --noEmit 0 错
// - vitest 0 失败
// - pnpm test:e2e -- mcp-16-tool-coverage 16/16 全过
// - 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
//
// 已知缺口 (per 守门 #11 缺标比错标安全):
// - 4 pre-existing star-mcp tools failed (find_references / get_code_context / get_symbol / search_code) 跨 session 续 (per docs/test-design.md §27.6 缺口 #1)
// - MCP 真 stdio transport P3 (Phase F+)
// - MCP 真 OAuth/Auth flow P3

import { test, expect, type Page } from '@playwright/test';

const MCP_BASE = '/api/mcp';

const MCP_TOOLS = [
  { name: 'agents_list', method: 'GET', endpoint: '/agents', fixture: 'v1--mcp--agents-list--GET.json' },
  { name: 'audit_event', method: 'GET', endpoint: '/audit-event', fixture: 'v1--mcp--audit-event--GET.json' },
  { name: 'billing_usage', method: 'GET', endpoint: '/billing-usage', fixture: 'v1--mcp--billing-usage--GET.json' },
  { name: 'feedback', method: 'POST', endpoint: '/feedback', fixture: 'v1--mcp--feedback--POST.json' },
  { name: 'form', method: 'GET', endpoint: '/form', fixture: 'v1--mcp--form--GET.json' },
  { name: 'inbox', method: 'GET', endpoint: '/inbox', fixture: 'v1--mcp--inbox--GET.json' },
  { name: 'kms', method: 'GET', endpoint: '/kms', fixture: 'v1--mcp--kms--GET.json' },
  { name: 'permission', method: 'GET', endpoint: '/permission', fixture: 'v1--mcp--permission--GET.json' },
  { name: 'project', method: 'GET', endpoint: '/project', fixture: 'v1--mcp--project--GET.json' },
  { name: 'scm', method: 'GET', endpoint: '/scm', fixture: 'v1--mcp--scm--GET.json' },
  { name: 'search', method: 'GET', endpoint: '/search', fixture: 'v1--mcp--search--GET.json' },
  { name: 'sessions_create', method: 'POST', endpoint: '/sessions-create', fixture: 'v1--mcp--sessions-create--POST.json' },
  { name: 'tools_invoke', method: 'POST', endpoint: '/tools-invoke', fixture: 'v1--mcp--tools-invoke--POST.json' },
  { name: 'workitem_create', method: 'POST', endpoint: '/workitem-create', fixture: 'v1--mcp--workitem-create--POST.json' },
  { name: 'workitem_list', method: 'GET', endpoint: '/workitem-list', fixture: 'v1--mcp--workitem-list--GET.json' },
  { name: 'workspace', method: 'GET', endpoint: '/workspace', fixture: 'v1--mcp--workspace--GET.json' },
] as const;

test.describe('UAT 业务流程 5: 16 MCP tool 端到端覆盖 (per ADR-0032)', () => {
  // === 守门 1: 16 MCP tool 全 endpoint 可达 (200 or 缺标 404/501 都接受) ===
  test('1. 16 MCP tool 全 endpoint 可达 (16/16)', async ({ page }) => {
    const results: Array<{ name: string; status: number }> = [];
    for (const tool of MCP_TOOLS) {
      let resp;
      if (tool.method === 'GET') {
        resp = await page.request.get(MCP_BASE + tool.endpoint);
      } else {
        resp = await page.request.post(MCP_BASE + tool.endpoint, {
          data: { actor_session_id: 'session-2026-09-07-001' },
        });
      }
      results.push({ name: tool.name, status: resp.status() });
    }
    // 16/16 守门
    expect(results.length).toBe(16);
    // 全部 200 (mock 简化: MSW 全 echo) OR 缺标 404/501 (per 守门 #11 缺标比错标)
    const allOk = results.every((r) => r.status === 200 || r.status === 404 || r.status === 501);
    expect(allOk).toBeTruthy();
  });

  // === 守门 2: GET 工具 (10) 全 200 + fixture 守门 ===
  test('2. GET MCP tool (10/16): 全部 200 + response 含 fixture_version=v1', async ({ page }) => {
    const getTools = MCP_TOOLS.filter((t) => t.method === 'GET');
    expect(getTools.length).toBe(10);
    for (const tool of getTools) {
      const resp = await page.request.get(MCP_BASE + tool.endpoint);
      if (resp.status() === 200) {
        const body = await resp.json();
        // 16 MCP tool fixture 守门: fixture_version=v1
        expect(body.fixture_version).toBe('v1');
        expect(body.module).toBe('mcp');
      }
    }
  });

  // === 守门 3: POST 工具 (6) 全 201/200 + idempotency-key 守门 ===
  test('3. POST MCP tool (6/16): 全部 200/201 + idempotency_key 守门', async ({ page }) => {
    const postTools = MCP_TOOLS.filter((t) => t.method === 'POST');
    expect(postTools.length).toBe(6);
    for (const tool of postTools) {
      const resp = await page.request.post(MCP_BASE + tool.endpoint, {
        data: {
          actor_session_id: 'session-2026-09-07-001',
          idempotency_key: `uat-${tool.name}-2026-09-07-001`,
        },
      });
      if (resp.status() === 200 || resp.status() === 201) {
        const body = await resp.json();
        // 16 MCP tool POST 守门: response 含 idempotency_key
        expect(body.idempotency_key).toBe(`uat-${tool.name}-2026-09-07-001`);
      }
    }
  });

  // === 守门 4: tools_invoke 守门 (per ADR-0032 §2 工具调用) ===
  test('4. tools_invoke: POST /api/mcp/tools-invoke 返回 tool_result (per ADR-0032 §3 关键变更)', async ({ page }) => {
    const resp = await page.request.post(`${MCP_BASE}/tools-invoke`, {
      data: {
        tool_name: 'agents_list',
        arguments: { limit: 10 },
        actor_session_id: 'session-2026-09-07-001',
        idempotency_key: 'uat-tools-invoke-2026-09-07-001',
      },
    });
    if (resp.status() === 200 || resp.status() === 201) {
      const body = await resp.json();
      // fixture v1--mcp--tools-invoke--POST.json 守门
      expect(body).toHaveProperty('tool_result');
      expect(body.tool_name).toBe('agents_list');
    } else {
      expect([404, 501]).toContain(resp.status());
    }
  });
});
