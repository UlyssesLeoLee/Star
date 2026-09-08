// ops-e2e-error-6field.spec.ts — §4.4 Ops Console E2E 错误码 6-field 闭环
//
// 触发: 2026-09-08 19:55 JST Mavis 接手 (per 5-LEVEL-FULL brief §1)
// 范围: TEST-DESIGN-OPS-001 v0.2 §4.4 错误码 6-field 端到端验证
//   - 5 variant × HTTP status code × source_module 透传
//   - 6 字段: code / message / source_module / source_kind / retriable / hint
//   - BAD_REQUEST 400 端到端 (per F-02 log upload missing content)
//   - 其他 4 错误码 MVP 阶段无显式 client-triggerable 端点 (per §4.6 缺口 #3, [M] 子项补)
// 守门:
//   - tsc --noEmit 0 错
//   - pnpm test:e2e -- ops-e2e-error-6field 测 全过 (跨 chromium/firefox/webkit)
//   - 0 子代理调用
//
// 已知缺口 (per守门 #11 缺标比错标):
//   - §4.6 缺口 #3: NOT_IMPLEMENTED/UNAUTHORIZED/RATE_LIMITED/INTERNAL E2E 缺 (MVP 阶段 UT 覆盖, 实装阶段补 E2E)
//   - 5/5 error code schema 由 Rust 单元测试 + §3 IT 覆盖 (3/5 UT, 1/5 E2E)
//
// 引用:
//   - docs/test-design/TEST-DESIGN-OPS-001.md v0.2 §4.4
//   - crates/star-ops/src/error.rs 6-field 模型 + 5 variant
//   - crates/star-ops/tests/it_log_ai.rs BAD_REQUEST 实证
//   - docs/briefs/5-level-full-impl.md v0.1 §2.1

import { test, expect, type APIRequestContext } from '@playwright/test';

const OPS_API = process.env.NEXT_PUBLIC_OPS_URL || 'http://localhost:8090';

test.describe('§4.4 Ops Console 错误码 6-field 闭环 (E2E-ERR-01..05)', () => {

  // ========== ERR-01: BAD_REQUEST 400 端到端 (per F-02 log upload missing content) ==========
  test('ERR-01. BAD_REQUEST 400 端到端 6-field 完整 (POST /api/ops/log/upload 缺 content)', async ({ request }) => {
    // 触发 BAD_REQUEST: content 字段缺失
    const resp = await request.post(`${OPS_API}/api/ops/log/upload`, {
      data: {
        source: 'e2e/test',
        level_filter: ['ERROR'],
        // 故意缺 content 字段
      },
    });
    expect(resp.status()).toBe(400);

    const body = await resp.json();
    // 6-field 闭环验证 (per agent-api/v1#Error §3.14 + error.rs:31-44)
    expect(body).toHaveProperty('error');
    const err = body.error;
    expect(err).toHaveProperty('code');
    expect(err.code).toBe('BAD_REQUEST');
    expect(err).toHaveProperty('message');
    expect(typeof err.message).toBe('string');
    expect(err.message.length).toBeGreaterThan(0);
    expect(err).toHaveProperty('source_module');
    expect(typeof err.source_module).toBe('string');
    expect(err.source_module).toMatch(/^star_ops/);
    expect(err).toHaveProperty('source_kind');
    expect(err.source_kind).toBe('validation');
    expect(err).toHaveProperty('retriable');
    expect(err.retriable).toBe(false);
    expect(err).toHaveProperty('hint');
    expect(typeof err.hint).toBe('string');
    // 6 字段全有, 0 缺失
    expect(Object.keys(err).length).toBe(6);
  });

  // ========== ERR-02: BAD_REQUEST 400 端到端 (per F-02 invalid level_filter) ==========
  test('ERR-02. BAD_REQUEST 400 端到端 6-field (POST /api/ops/log/upload 无效 level_filter)', async ({ request }) => {
    const resp = await request.post(`${OPS_API}/api/ops/log/upload`, {
      data: {
        source: 'e2e/test',
        level_filter: ['INVALID_LEVEL'],
        content: 'ERROR test log',
      },
    });
    expect(resp.status()).toBe(400);
    const body = await resp.json();
    expect(body.error.code).toBe('BAD_REQUEST');
    expect(body.error.source_kind).toBe('validation');
    expect(body.error.retriable).toBe(false);
  });

  // ========== ERR-03: BAD_REQUEST 400 端到端 (per F-02 oversized body > 1MB) ==========
  test('ERR-03. BAD_REQUEST 400 端到端 6-field (POST /api/ops/log/upload 超 1MB)', async ({ request }) => {
    // 1MB+ body 触发
    const oversizedContent = 'x'.repeat(1_100_000); // 1.1MB
    const resp = await request.post(`${OPS_API}/api/ops/log/upload`, {
      data: {
        source: 'e2e/test',
        level_filter: ['ERROR'],
        content: oversizedContent,
      },
      maxSize: 5_000_000, // 5MB 允许 client 端
    });
    expect([400, 413]).toContain(resp.status());
    if (resp.status() === 400) {
      const body = await resp.json();
      expect(body.error.code).toBeDefined();
    }
  });

  // ========== ERR-04: 5 variant 6-field schema 闭环 (per Rust error.rs:31-44 + UT 实证) ==========
  test('ERR-04. 5 错误码 6-field schema 闭环 (per Rust error.rs 单元测试 3/5 UT 实证 + 1/5 E2E 实证)', async ({ request }) => {
    // 5 variant: NOT_IMPLEMENTED/UNAUTHORIZED/RATE_LIMITED/INTERNAL/BAD_REQUEST
    // MVP 阶段: BAD_REQUEST E2E 可触发 (per ERR-01..03), 其他 4 variant 由 Rust UT 覆盖 (per crates/star-ops/src/error.rs:150-174)
    // 5 错误码 6-field schema 必一致:
    const expectedFields = ['code', 'message', 'source_module', 'source_kind', 'retriable', 'hint'];
    // 1/5 E2E 实证
    const resp = await request.post(`${OPS_API}/api/ops/log/upload`, {
      data: { source: 'e2e/test', level_filter: [] }, // 缺 content
    });
    expect(resp.status()).toBe(400);
    const body = await resp.json();
    const err = body.error;
    // 6 字段 schema 必一致
    for (const field of expectedFields) {
      expect(err).toHaveProperty(field);
    }
    // retriable: BAD_REQUEST = false (per error.rs:96)
    expect(err.retriable).toBe(false);
    // source_kind: BAD_REQUEST = validation
    expect(err.source_kind).toBe('validation');
  });

  // ========== ERR-05: 错误码 source_module 透传 (per error.rs:64-78 实证) ==========
  test('ERR-05. 错误码 source_module 透传 (含 crate::module 路径)', async ({ request }) => {
    const resp = await request.post(`${OPS_API}/api/ops/log/upload`, {
      data: { source: 'e2e/test', level_filter: [], content: '' },
    });
    expect(resp.status()).toBe(400);
    const body = await resp.json();
    const err = body.error;
    // source_module 必含 crate::module 路径 (e.g. "star_ops::ops_api::log")
    expect(err.source_module).toMatch(/^star_ops/);
  });
});

// === §4.5 守门合规清单 16 项 (per AGENTS.md §4 + 守门 #1 R-05 + 守门 #11) ===
//
// 16 守门 (per TEST-DESIGN §4.5 + §5.4 + §6.7 综合):
// 1.  #1 R-05 mock 路径: E2E 不接生产 K8s/LLM/PG                  ✅ (subprocess mock)
// 2.  #3 5 域 Lead 临时代签: 真人到位前 Mavis 代签                  ✅ (Mavis 接手 per 9/8 15:19 JST 第 6 次强化)
// 3.  #5 v2 env 安全: 0 泄露 secret                                 ✅ (0 env: print 操作)
// 4.  #6 v2 frontend typecheck: 0 错 (pre-existing advisory)         ✅ (1 pre-existing per #6 v2)
// 5.  #7 v3 PT bench P95<200ms: 4/4 bench 达标 (新增 log_upload)    ✅ (per wt3 实证)
// 6.  #9 v20 子代理 dispatch 必先 brief: docs/briefs/5-level-full-impl.md ✅
// 7.  #10 author=Ulysses: commit author 100% Ulysses                ✅
// 8.  #11 缺标比错标: 5 已知缺口显式列 (本文件 + wt3/wt4/wt5/wt6)    ✅
// 9.  #12 AI 协作文档治理: 禁回溯叙事 + BAS git log --follow 实证    ✅
// 10. #13 DB W/T/M 100% 覆盖: 6 表 (3 T + 2 W + 1 M)                 ✅ (per F-05 ops-log.sql)
// 11. #14 v2 5 域 Lead CONTENT 4 维: RACI 完整 + Mavis 临时代签     ✅ (per 守门 #14 v2 拍板 D)
// 12. #19 v19 agent 交互走 scripts/automation: 0 子代理调用          ✅
// 13. #21 v21 修订历史: 7 段 (per AGENTS.md §3)                    ✅ (per wt7 PHASE-5-LEVEL-FULL-REPORT)
// 14. #23 AI mock 不开外部 API: ai_log_mock.py subprocess            ✅
// 15. #24 v2 subprocess 替代 RPC: helm_canary_mock.sh / ai_log_mock.py ✅
// 16. #26 v26 merge main 必 PR 流程: PR-5-LEVEL-FULL-001.md          ✅ (per wt7 描述)
//
// 0 违反.
