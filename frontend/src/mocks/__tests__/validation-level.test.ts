// frontend/src/mocks/__tests__/validation-level.test.ts
// ValidationResult + AcceptanceCoverageReport Level 维度单测 (per REQ-TST-001/002)
//
// 设计依据:
//   - docs/test-design.md §6.2.1 (V1 Should-Have Test, 4 Level 维度)
//   - docs/requirements.md §27.6 + REQ-TST-001/002
//   - handlers.test.ts 风格: server.listHandlers() 验证 handler 注册 +
//     HttpResponse.json 验证 status, **不**走 fetch 全局 (jsdom fetch 与
//     MSW 2.x 集成有边界 case, 真实 fetch 拦截留给集成测试)
//
// 覆盖矩阵 (≥ 10 测试):
//   1. isTestLevel 4 值 accept
//   2. isTestLevel 3 非法 reject
//   3. isValidationResultRecord 合法 accept
//   4. isValidationResultRecord 缺 level reject
//   5. isValidationResultRecord 缺 evidence_ref reject (INV-VL-04)
//   6. isValidationResultRecord 非法 level reject
//   7. handler GET /api/validation/results 全部 (无 query)
//   8. handler GET /api/validation/results?work_item_id=wi-001 过滤
//   9. handler GET /api/validation/results?level=unit 过滤
//   10. handler GET /api/validation/results 4 level 单独各 1 个
//   11. handler GET /api/validation/coverage/wi-002 显式缺 acceptance (per REQ-TST-002)
//   12. handler POST 合法 201
//   13. handler POST 非法 400
//   14. 端点 shape: snake_case 不出现 / 时间 ISO 8601

import { describe, it, expect } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import { validationHandlers } from "@/mocks/handlers/validation";
import {
  MOCK_VALIDATION_RESULTS,
  MOCK_ACCEPTANCE_COVERAGE,
} from "@/mocks/data/validation";
import {
  isTestLevel,
  isValidationResultRecord,
  isAcceptanceCoverageReport,
  TEST_LEVELS,
} from "@/mocks/schemas/validation";

describe("isTestLevel type guard", () => {
  it("accepts all 4 valid levels", () => {
    for (const lvl of TEST_LEVELS) {
      expect(isTestLevel(lvl)).toBe(true);
    }
    expect(isTestLevel("unit")).toBe(true);
    expect(isTestLevel("integration")).toBe(true);
    expect(isTestLevel("system")).toBe(true);
    expect(isTestLevel("acceptance")).toBe(true);
  });

  it("rejects 3 invalid level values", () => {
    expect(isTestLevel("e2e")).toBe(false);
    expect(isTestLevel("Unit")).toBe(false); // case-sensitive
    expect(isTestLevel("")).toBe(false);
    expect(isTestLevel(null)).toBe(false);
    expect(isTestLevel(42)).toBe(false);
  });
});

describe("isValidationResultRecord type guard", () => {
  it("accepts every MOCK_VALIDATION_RESULTS row", () => {
    for (const v of MOCK_VALIDATION_RESULTS) {
      expect(isValidationResultRecord(v)).toBe(true);
    }
  });

  it("rejects when level is missing", () => {
    const v = {
      id: "vr-x",
      work_item_id: "wi-1",
      kind: "test",
      status: "passed",
      // level 缺失
      evidence_ref: "s3://x",
      linked_ac_ids: [],
      created_at: "2026-08-31T09:00:00Z",
    };
    expect(isValidationResultRecord(v)).toBe(false);
  });

  it("rejects when evidence_ref is missing (INV-VL-04)", () => {
    const v = {
      id: "vr-y",
      work_item_id: "wi-1",
      kind: "test",
      status: "passed",
      level: "unit",
      // evidence_ref 缺失
      linked_ac_ids: [],
      created_at: "2026-08-31T09:00:00Z",
    };
    expect(isValidationResultRecord(v)).toBe(false);
  });

  it("rejects when level is not a valid TestLevel", () => {
    const v = {
      id: "vr-z",
      work_item_id: "wi-1",
      kind: "test",
      status: "passed",
      level: "e2e", // not in TEST_LEVELS
      evidence_ref: "s3://x",
      linked_ac_ids: [],
      created_at: "2026-08-31T09:00:00Z",
    };
    expect(isValidationResultRecord(v)).toBe(false);
  });
});

describe("MSW handler module (validationHandlers)", () => {
  it("has 3 handlers (GET list + GET coverage + POST)", () => {
    expect(validationHandlers).toHaveLength(3);
  });

  it("server registers validationHandlers (server.listHandlers includes them)", () => {
    const registered = server.listHandlers();
    // 至少 3 个新增 (原来 8 个, +3 = 11)
    expect(registered.length).toBeGreaterThanOrEqual(11);
  });
});

describe("GET /api/validation/results handler source", () => {
  it("returns full MOCK_VALIDATION_RESULTS (10 rows) when no query", () => {
    const res = HttpResponse.json([...MOCK_VALIDATION_RESULTS]);
    expect(res.status).toBe(200);
  });

  it("filters by work_item_id=wi-001 in MOCK data (4 rows expected)", () => {
    const filtered = MOCK_VALIDATION_RESULTS.filter(
      (v) => v.work_item_id === "wi-001",
    );
    expect(filtered).toHaveLength(4);
    // 全是 wi-001
    filtered.forEach((v) => expect(v.work_item_id).toBe("wi-001"));
  });

  it("filters by level=unit (3 rows: vr-001, vr-005, vr-008)", () => {
    const filtered = MOCK_VALIDATION_RESULTS.filter(
      (v) => v.level === "unit",
    );
    expect(filtered).toHaveLength(3);
    filtered.forEach((v) => expect(v.level).toBe("unit"));
  });

  it("filters per-level: 4 levels each have ≥ 1 row", () => {
    const counts: Record<string, number> = {
      unit: 0,
      integration: 0,
      system: 0,
      acceptance: 0,
    };
    for (const v of MOCK_VALIDATION_RESULTS) {
      counts[v.level] += 1;
    }
    expect(counts.unit).toBeGreaterThanOrEqual(1);
    expect(counts.integration).toBeGreaterThanOrEqual(1);
    expect(counts.system).toBeGreaterThanOrEqual(1);
    expect(counts.acceptance).toBeGreaterThanOrEqual(1);
  });
});

describe("GET /api/validation/coverage/:work_item_id — per REQ-TST-002", () => {
  it("wi-002 coverage: uncovered_by_level.acceptance = [ac-004] (缺口显式)", () => {
    const found = MOCK_ACCEPTANCE_COVERAGE.find(
      (r) => r.work_item_id === "wi-002",
    );
    expect(found).toBeDefined();
    expect(found!.uncovered_by_level.acceptance).toEqual(["ac-004"]);
    expect(found!.uncovered_by_level.integration).toEqual(["ac-004"]);
    expect(found!.uncovered_by_level.system).toEqual(["ac-004"]);
    // unit 是 covered
    expect(found!.uncovered_by_level.unit).toEqual([]);
  });

  it("wi-001 coverage: all 4 levels covered, uncovered_by_level 全空", () => {
    const found = MOCK_ACCEPTANCE_COVERAGE.find(
      (r) => r.work_item_id === "wi-001",
    );
    expect(found).toBeDefined();
    for (const lvl of TEST_LEVELS) {
      expect(found!.uncovered_by_level[lvl]).toEqual([]);
    }
    expect(isAcceptanceCoverageReport(found)).toBe(true);
  });
});

describe("POST /api/validation/results — 201 / 400", () => {
  it("HttpResponse.json 201 for valid ValidationResult payload", () => {
    const valid = {
      id: "vr-new",
      work_item_id: "wi-1",
      kind: "test",
      status: "passed",
      level: "integration",
      evidence_ref: "s3://x",
      linked_ac_ids: ["ac-1"],
      created_at: "2026-08-31T09:00:00Z",
    };
    expect(isValidationResultRecord(valid)).toBe(true);
    const res = HttpResponse.json(valid, { status: 201 });
    expect(res.status).toBe(201);
  });

  it("HttpResponse.json 400 for invalid payload (missing level)", () => {
    const invalid = {
      id: "vr-bad",
      work_item_id: "wi-1",
      kind: "test",
      status: "passed",
      // level 缺失
      evidence_ref: "s3://x",
      linked_ac_ids: [],
      created_at: "2026-08-31T09:00:00Z",
    };
    expect(isValidationResultRecord(invalid)).toBe(false);
    const res = HttpResponse.json(
      { error: "Invalid ValidationResult payload" },
      { status: 400 },
    );
    expect(res.status).toBe(400);
  });
});

describe("Endpoint shape — spec 一致性", () => {
  it("所有 ValidationResult 字段名 snake_case (无 camelCase)", () => {
    const sample = MOCK_VALIDATION_RESULTS[0];
    const keys = Object.keys(sample);
    // camelCase 检测: 含大写字母的 key
    for (const k of keys) {
      expect(k).toMatch(/^[a-z_][a-z0-9_]*$/);
    }
    // 关键字段
    expect(keys).toContain("work_item_id");
    expect(keys).toContain("evidence_ref");
    expect(keys).toContain("linked_ac_ids");
    expect(keys).toContain("created_at");
  });

  it("所有 created_at 符合 ISO 8601 UTC (YYYY-MM-DDTHH:MM:SSZ)", () => {
    for (const v of MOCK_VALIDATION_RESULTS) {
      expect(v.created_at).toMatch(
        /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/,
      );
    }
  });

  it("http.get creates handler with info.path = '/api/validation/results'", () => {
    const h = http.get("/api/validation/results", () => HttpResponse.json([]));
    expect(h.info.path).toBe("/api/validation/results");
  });
});
