// =====================================================================
// design-artifacts.test.ts — MSW handler 端点测试 (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 覆盖 (>= 6 个):
//   1. GET /api/design-artifacts 不带 filter 返全部
//   2. GET /api/design-artifacts?work_item_id=X 过滤
//   3. GET /api/design-artifacts/:id 单条
//   4. GET /api/design-artifacts/guard/:work_item_id 正面 (全 approved)
//      + 负面 (有 draft, 指出 pending)
//   5. POST /api/design-artifacts/:id/review approve 流程
//      (status → approved, version+1, review_record_id 有值)
//   6. POST /api/design-artifacts/:id/review request_changes 流程
//      (status → draft)
//
// 设计 (per mock-msw-handlers.md §2.7 + handlers.test.ts 实证):
//   - jsdom + MSW 2.x 下 fetch 走真实网络 (EACCES ::1:80), 拦截失败
//   - 改测 inner pure resolvers (listDesignArtifacts / getDesignArtifact
//     / guardDesignArtifacts / applyReview / nextStatusFromDecision)
//   - 跟 handlers.test.ts 风格一致: 不依赖 MSW 实际拦截, 数据完整性等价
//   - handler 模块导出 (4 endpoint + 2 状态机) 由 handlers.test.ts
//     server.listHandlers() 验证 (跨所有 handler 集合)
// =====================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import {
  designArtifactHandlers,
  listDesignArtifacts,
  getDesignArtifact,
  guardDesignArtifacts,
  applyReview,
  nextStatusFromDecision,
  __resetDesignArtifactStore,
  __getDesignArtifactStore,
} from "@/mocks/handlers/design-artifacts";
import { MOCK_DESIGN_ARTIFACTS } from "@/mocks/data/design-artifacts";
import {
  isDesignArtifact,
  isDesignArtifactStatus,
} from "@/mocks/schemas/design-artifact";
import type { DesignArtifact } from "@/types/ids";

// =====================================================================
// Schema + data integrity
// =====================================================================

describe("DesignArtifact schema (data integrity)", () => {
  it("[schema.1] all 9 mock artifacts pass isDesignArtifact type guard", () => {
    for (const a of MOCK_DESIGN_ARTIFACTS) {
      expect(isDesignArtifact(a)).toBe(true);
    }
  });

  it("[schema.2] 5 status enum fully covered by fixture", () => {
    const statuses = new Set(MOCK_DESIGN_ARTIFACTS.map((a) => a.status));
    expect(statuses.has("draft")).toBe(true);
    expect(statuses.has("in_review")).toBe(true);
    expect(statuses.has("approved")).toBe(true);
    expect(statuses.has("rejected")).toBe(true);
    expect(statuses.has("superseded")).toBe(true);
  });

  it("[schema.3] isDesignArtifactStatus type guard 5 值 + 拒绝", () => {
    expect(isDesignArtifactStatus("draft")).toBe(true);
    expect(isDesignArtifactStatus("in_review")).toBe(true);
    expect(isDesignArtifactStatus("approved")).toBe(true);
    expect(isDesignArtifactStatus("rejected")).toBe(true);
    expect(isDesignArtifactStatus("superseded")).toBe(true);
    expect(isDesignArtifactStatus("invalid")).toBe(false);
    expect(isDesignArtifactStatus(123)).toBe(false);
  });
});

// =====================================================================
// Fixture integrity
// =====================================================================

describe("MOCK_DESIGN_ARTIFACTS fixture integrity", () => {
  beforeEach(() => __resetDesignArtifactStore());

  it("[data.1] 9 artifacts across 3 work_item_ids (per 任务 spec: >= 6 条, 跨 >= 3 wi)", () => {
    expect(MOCK_DESIGN_ARTIFACTS.length).toBeGreaterThanOrEqual(6);
    const wiSet = new Set(MOCK_DESIGN_ARTIFACTS.map((a) => a.work_item_id));
    expect(wiSet.size).toBeGreaterThanOrEqual(3);
  });

  it("[data.2] 1 wi has all approved (Guard 正面用例)", () => {
    const wiAllApproved = "wi-physis-001";
    const items = MOCK_DESIGN_ARTIFACTS.filter(
      (a) => a.work_item_id === wiAllApproved,
    );
    expect(items.length).toBeGreaterThan(0);
    for (const a of items) {
      expect(a.status).toBe("approved");
    }
  });

  it("[data.3] 1 wi has 1 draft + 2 approved (Guard 负面用例)", () => {
    const wiHasDraft = "wi-physis-002";
    const items = MOCK_DESIGN_ARTIFACTS.filter(
      (a) => a.work_item_id === wiHasDraft,
    );
    const drafts = items.filter((a) => a.status === "draft");
    const approved = items.filter((a) => a.status === "approved");
    expect(drafts).toHaveLength(1);
    expect(approved).toHaveLength(2);
  });
});

// =====================================================================
// Pure resolver — 5 状态机迁移
// =====================================================================

describe("nextStatusFromDecision — state machine migration", () => {
  it("[sm.1] draft + approve → approved", () => {
    expect(nextStatusFromDecision("draft", "approve")).toBe("approved");
  });

  it("[sm.2] in_review + approve → approved", () => {
    expect(nextStatusFromDecision("in_review", "approve")).toBe("approved");
  });

  it("[sm.3] rejected + approve → approved", () => {
    expect(nextStatusFromDecision("rejected", "approve")).toBe("approved");
  });

  it("[sm.4] approved + request_changes → draft", () => {
    expect(nextStatusFromDecision("approved", "request_changes")).toBe("draft");
  });

  it("[sm.5] superseded 任何 decision → 保持 superseded", () => {
    expect(nextStatusFromDecision("superseded", "approve")).toBe("superseded");
    expect(nextStatusFromDecision("superseded", "request_changes")).toBe("superseded");
  });
});

// =====================================================================
// Pure resolvers — endpoint behavior
// =====================================================================

describe("Endpoint behavior (via pure resolvers)", () => {
  beforeEach(() => __resetDesignArtifactStore());

  it("[handler.1] GET /api/design-artifacts 不带 filter → 返全部 (mutable store 视角)", () => {
    const all = listDesignArtifacts(null);
    expect(all.length).toBe(MOCK_DESIGN_ARTIFACTS.length);
    for (const a of all) {
      expect(isDesignArtifact(a)).toBe(true);
    }
  });

  it("[handler.2] GET /api/design-artifacts?work_item_id=wi-physis-001 → 3 条全 approved", () => {
    const filtered = listDesignArtifacts("wi-physis-001");
    expect(filtered).toHaveLength(3);
    for (const a of filtered) {
      expect(a.work_item_id).toBe("wi-physis-001");
      expect(a.status).toBe("approved");
    }
  });

  it("[handler.3] GET /api/design-artifacts/:id 单条", () => {
    const a = getDesignArtifact("da-001");
    expect(a).not.toBeNull();
    expect(a?.id).toBe("da-001");
    expect(a?.title).toBe("Physis 引擎架构总览 (v3)");
    expect(isDesignArtifact(a)).toBe(true);

    // 不存在
    const missing = getDesignArtifact("da-nonexistent");
    expect(missing).toBeNull();
  });

  it("[handler.4] GET /api/design-artifacts/guard/:work_item_id 正面 (全 approved)", () => {
    const r = guardDesignArtifacts("wi-physis-001");
    expect(r.all_approved).toBe(true);
    expect(r.pending).toEqual([]);
  });

  it("[handler.4b] GET /api/design-artifacts/guard/:work_item_id 负面 (有 draft)", () => {
    const r = guardDesignArtifacts("wi-physis-002");
    expect(r.all_approved).toBe(false);
    expect(r.pending).toHaveLength(1);
    expect(r.pending[0].id).toBe("da-004");
    expect(r.pending[0].status).toBe("draft");
  });

  it("[handler.5] POST /api/design-artifacts/:id/review approve → status approved + version+1 + review_record_id", () => {
    // 用 da-004 (draft) → approve 走完整路径
    const res = applyReview("da-004", {
      decision: "approve",
      reviewer_id: "u-rev-001",
      comment: "Looks good",
    });
    expect(res.ok).toBe(true);
    expect(res.status).toBe(200);
    expect(res.artifact).not.toBeNull();
    expect(res.artifact!.id).toBe("da-004");
    expect(res.artifact!.status).toBe("approved");
    expect(res.artifact!.version).toBe(2); // 原 1 + 1
    expect(res.artifact!.review_record_id).toBeTruthy();
    expect(typeof res.artifact!.review_record_id).toBe("string");

    // 验证 mutable store 实际更新
    const store = __getDesignArtifactStore();
    const updated = store.find((a) => a.id === "da-004");
    expect(updated?.status).toBe("approved");
    expect(updated?.version).toBe(2);
  });

  it("[handler.6] POST /api/design-artifacts/:id/review request_changes → status draft", () => {
    // 用 da-005 (approved) → request_changes: approved → draft (per nextStatusFromDecision)
    const res = applyReview("da-005", {
      decision: "request_changes",
      reviewer_id: "u-rev-001",
      comment: "Need more detail",
    });
    expect(res.ok).toBe(true);
    expect(res.status).toBe(200);
    expect(res.artifact!.id).toBe("da-005");
    expect(res.artifact!.status).toBe("draft");
    expect(res.artifact!.version).toBe(3); // 原 2 + 1
    expect(res.artifact!.review_record_id).toBeTruthy();
  });

  it("[handler.7] POST /api/design-artifacts/:id/review 400 on invalid body", () => {
    const res = applyReview("da-001", { decision: "bogus", reviewer_id: "u-rev" });
    expect(res.ok).toBe(false);
    expect(res.status).toBe(400);
    expect(res.artifact).toBeNull();
  });

  it("[handler.8] POST /api/design-artifacts/:id/review 404 on missing id", () => {
    const res = applyReview("da-missing", {
      decision: "approve",
      reviewer_id: "u-rev",
    });
    expect(res.ok).toBe(false);
    expect(res.status).toBe(404);
  });
});

// =====================================================================
// MSW integration — handler registration sanity (per handlers.test.ts 风格)
// =====================================================================

describe("MSW handler module structure (registration sanity)", () => {
  it("[reg.1] designArtifactHandlers has 4 handlers (1 list + 1 guard + 1 single + 1 review)", () => {
    expect(designArtifactHandlers).toHaveLength(4);
  });

  it("[reg.2] server.listHandlers contains all 4 designArtifactHandlers", () => {
    // 注意: vitest 跨文件 setupFiles 已调 server.listen, 这里直接 list
    const all = server.listHandlers();
    // 我们注册的 4 个 path 必须在
    const paths = all.map((h) => {
      const info = h.info as { method: string; path: string };
      return `${info.method} ${info.path}`;
    });
    expect(paths).toContain("GET /api/design-artifacts");
    expect(paths).toContain("GET /api/design-artifacts/guard/:work_item_id");
    expect(paths).toContain("GET /api/design-artifacts/:id");
    expect(paths).toContain("POST /api/design-artifacts/:id/review");
  });

  it("[reg.3] HttpResponse.json({...}) shape sanity (HttpResponse 200 + 400 + 404)", () => {
    const okRes = HttpResponse.json({ id: "da-001" }, { status: 200 });
    expect(okRes.status).toBe(200);
    const badRes = HttpResponse.json({ error: "Invalid" }, { status: 400 });
    expect(badRes.status).toBe(400);
    const nfRes = HttpResponse.json({ error: "Not found" }, { status: 404 });
    expect(nfRes.status).toBe(404);
  });

  it("[reg.4] http.get / http.post produce handlers with info.path (per msw 2.x)", () => {
    const g = http.get("/api/__test_path", () => HttpResponse.json({}));
    const p = http.post("/api/__test_path2", () => HttpResponse.json({}));
    expect(g.info.path).toBe("/api/__test_path");
    expect(p.info.path).toBe("/api/__test_path2");
  });
});
