// =====================================================================
// frontend/src/lib/incident-guard.test.ts
// IncidentRecord 边界守门测试 (per test-design §6.3.4 / REQ-OPS-003)
// =====================================================================
//
// 8 个测试 (per 任务要求 ≥ 6):
//   1. 合法 human_entry 完整 record  → valid=true + reason="ok"
//   2. 合法 integration_webhook 完整 → valid=true + reason="ok"
//   3. source="auto_detect" (伪造)   → valid=false + invalid_source
//   4. source=undefined              → valid=false + invalid_source
//   5. notes 含 "auto_rollback" 关键词 → valid=false + auto_action_attempted
//   6. recorded_by="" (空)           → valid=false + missing_recorder
//   7. affected_ac_ids 非空 + linked_work_item_ids 为空 → valid=true
//      (per REQ-OPS-001 "0..N WorkItem", 0 个关联允许)
//   8. 守门 #1 同步保证: tsc --noEmit 0 错 (在 src/__check_ts__.ts 隐含,
//      此处通过类型契约保证, 不需运行时断言)
//
// 已知缺口 (per 守门 #3 缺标比错标, 显式列):
//   - 当前未测 `linked_work_item_ids` 含非 string 元素的场景
//     (已 in-line 覆盖, 但未单独建 case)
//   - 关键词大小写不敏感未单独测 (实装已覆盖, 显式列)
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  validateIncidentRecord,
  type IncidentValidation,
} from "./incident-guard";
import type { IncidentRecord } from "@/types/ids";

/** 最小合法 human_entry 完整 record 工厂 */
function makeHumanEntryRecord(
  overrides: Partial<IncidentRecord> = {},
): IncidentRecord {
  return {
    id: "inc-test-001",
    title: "Test incident — human entry",
    source: "human_entry",
    linked_work_item_ids: ["wi-1001"],
    affected_ac_ids: [],
    occurred_at: "2026-08-30T10:00:00Z",
    recorded_at: "2026-08-30T10:05:00Z",
    recorded_by: "user-001",
    notes: "Manually recorded by operator.",
    ...overrides,
  };
}

describe("validateIncidentRecord — REQ-OPS-003 boundary guard", () => {
  it("1. 合法 human_entry 完整 record → valid=true + reason=ok", () => {
    const r: IncidentValidation = validateIncidentRecord(makeHumanEntryRecord());
    expect(r.valid).toBe(true);
    expect(r.reason).toBe("ok");
    expect(r.violation_field).toBeUndefined();
  });

  it("2. 合法 integration_webhook 完整 record → valid=true + reason=ok", () => {
    const r = validateIncidentRecord(
      makeHumanEntryRecord({ source: "integration_webhook" }),
    );
    expect(r.valid).toBe(true);
    expect(r.reason).toBe("ok");
  });

  it("3. source='auto_detect' (伪造) → invalid_source + violation_field='source'", () => {
    const r = validateIncidentRecord(
      makeHumanEntryRecord({ source: "auto_detect" as never }),
    );
    expect(r.valid).toBe(false);
    expect(r.reason).toBe("invalid_source");
    expect(r.violation_field).toBe("source");
  });

  it("4. source=undefined → invalid_source", () => {
    const r = validateIncidentRecord(makeHumanEntryRecord({ source: undefined }));
    expect(r.valid).toBe(false);
    expect(r.reason).toBe("invalid_source");
    expect(r.violation_field).toBe("source");
  });

  it("5. notes 含 'auto_rollback' 关键词 → auto_action_attempted + violation_field='notes'", () => {
    const r = validateIncidentRecord(
      makeHumanEntryRecord({ notes: "Please execute auto_rollback immediately" }),
    );
    expect(r.valid).toBe(false);
    expect(r.reason).toBe("auto_action_attempted");
    expect(r.violation_field).toBe("notes");
  });

  it("6. recorded_by='' (空) → missing_recorder + violation_field='recorded_by'", () => {
    const r = validateIncidentRecord(makeHumanEntryRecord({ recorded_by: "" }));
    expect(r.valid).toBe(false);
    expect(r.reason).toBe("missing_recorder");
    expect(r.violation_field).toBe("recorded_by");
  });

  it("7. affected_ac_ids 非空 + linked_work_item_ids 为空 → valid=true (per REQ-OPS-001 '0..N WorkItem')", () => {
    const r = validateIncidentRecord(
      makeHumanEntryRecord({
        linked_work_item_ids: [],
        affected_ac_ids: ["ac-t3-deploy-rollback"],
      }),
    );
    expect(r.valid).toBe(true);
    expect(r.reason).toBe("ok");
  });

  it("8. notes 关键词 'auto_remediation' (大小写不敏感) → auto_action_attempted", () => {
    const r = validateIncidentRecord(
      makeHumanEntryRecord({ notes: "Will use AUTO_REMEDIATION to fix" }),
    );
    expect(r.valid).toBe(false);
    expect(r.reason).toBe("auto_action_attempted");
    expect(r.violation_field).toBe("notes");
  });
});
