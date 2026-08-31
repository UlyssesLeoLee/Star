// =====================================================================
// workitem-guard.test.ts — Guard 纯函数测试 (per wt-test-t2-dsg 2026-08-31)
// =====================================================================
// 覆盖 (>= 12 个):
//   1. 全 approved → allowed + reason "all_approved"
//   2. 1 draft + 2 approved → 不允许 + reason "pending_artifacts" + pending 含 draft
//   3. 1 in_review + 1 approved → 不允许 + pending 含 in_review
//   4. 1 rejected + 1 approved → 不允许 + pending 含 rejected
//   5. 1 superseded + 1 approved → 允许 (superseded 视为已批准)
//   6. 0 artifacts + requireApproval=true → 不允许 + reason "no_artifacts_attached"
//   7. 0 artifacts + requireApproval=false → 允许 + reason "no_artifacts_required"
//   8. artifacts 含异 workItemId 时的行为 (test 演示)
//   9-12. 4 Status (draft/in_review/rejected/single) boundary 各 1 测试
//   13. version 字段不影响 Guard (即使 version=99 仍按 status 判定)
//
// 守门 #1: 跑 vitest 前 tsc --noEmit 必 exit 0
// =====================================================================

import { describe, it, expect } from "vitest";
import { checkAllArtifactsApproved } from "@/lib/workitem-guard";
import type { DesignArtifact, Uuid } from "@/types/ids";

const WI: Uuid = "wi-001";
const OTHER_WI: Uuid = "wi-002";
const AUTHOR: Uuid = "u-author";

const T = "2026-08-30T10:00:00Z";

/** factory: 减重复, 默认 status=approved */
function mkArtifact(
  id: string,
  work_item_id: Uuid,
  status: DesignArtifact["status"],
  version: number = 1,
  title: string = `Artifact ${id}`,
): DesignArtifact {
  return {
    id,
    work_item_id,
    title,
    status,
    version,
    author_id: AUTHOR,
    created_at: T,
    updated_at: T,
    review_record_id: status === "approved" ? `rr-${id}` : null,
  };
}

describe("checkAllArtifactsApproved — positive cases", () => {
  it("[1] all approved → allowed + reason all_approved", () => {
    const artifacts = [
      mkArtifact("a1", WI, "approved"),
      mkArtifact("a2", WI, "approved"),
      mkArtifact("a3", WI, "approved"),
    ];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(true);
    expect(r.reason).toBe("all_approved");
    expect(r.pending).toEqual([]);
  });

  it("[5] 1 superseded + 1 approved → allowed (superseded 视为已批准)", () => {
    const artifacts = [
      mkArtifact("a1", WI, "superseded"),
      mkArtifact("a2", WI, "approved"),
    ];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(true);
    expect(r.reason).toBe("all_approved");
    expect(r.pending).toEqual([]);
  });

  it("[7] 0 artifacts + requireApproval=false → allowed + no_artifacts_required", () => {
    const r = checkAllArtifactsApproved(WI, [], false);
    expect(r.allowed).toBe(true);
    expect(r.reason).toBe("no_artifacts_required");
    expect(r.pending).toEqual([]);
  });
});

describe("checkAllArtifactsApproved — negative cases", () => {
  it("[2] 1 draft + 2 approved → 不允许 + pending_artifacts + pending 含 draft", () => {
    const draft = mkArtifact("a-draft", WI, "draft", 1, "Draft Design Doc");
    const artifacts = [
      draft,
      mkArtifact("a-app1", WI, "approved"),
      mkArtifact("a-app2", WI, "approved"),
    ];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
    expect(r.pending).toHaveLength(1);
    expect(r.pending[0].id).toBe("a-draft");
    expect(r.pending[0].title).toBe("Draft Design Doc");
    expect(r.pending[0].status).toBe("draft");
  });

  it("[3] 1 in_review + 1 approved → 不允许 + pending 含 in_review", () => {
    const inReview = mkArtifact("a-ir", WI, "in_review");
    const artifacts = [inReview, mkArtifact("a-app", WI, "approved")];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
    expect(r.pending).toHaveLength(1);
    expect(r.pending[0].status).toBe("in_review");
  });

  it("[4] 1 rejected + 1 approved → 不允许 + pending 含 rejected", () => {
    const rejected = mkArtifact("a-rej", WI, "rejected");
    const artifacts = [rejected, mkArtifact("a-app", WI, "approved")];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
    expect(r.pending).toHaveLength(1);
    expect(r.pending[0].status).toBe("rejected");
  });

  it("[6] 0 artifacts + requireApproval=true → 不允许 + no_artifacts_attached", () => {
    const r = checkAllArtifactsApproved(WI, [], true);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("no_artifacts_attached");
    expect(r.pending).toEqual([]);
  });

  it("[6b] 0 artifacts + requireApproval=true default → 不允许 (省略第 3 参)", () => {
    const r = checkAllArtifactsApproved(WI, []);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("no_artifacts_attached");
  });
});

describe("checkAllArtifactsApproved — boundary & edge cases", () => {
  it("[8] artifacts 数组含异 workItemId → 仍按 status 判定 (函数不主动过滤)", () => {
    // 设计: caller 应预先按 workItemId 过滤; 但若传入异 wi 的 artifact,
    // 由于函数不强制 workItemId 匹配, 异 wi artifact 会按其 status 进入判定.
    // 这里: WI 有 1 approved, OTHER_WI 有 1 draft (会被算作 pending)
    const artifacts = [
      mkArtifact("a-wi-app", WI, "approved"),
      mkArtifact("a-other-draft", OTHER_WI, "draft", 1, "Other WI Draft"),
    ];
    const r = checkAllArtifactsApproved(WI, artifacts);
    // draft (异 wi) 仍被视为未批准, 进入 pending
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
    expect(r.pending).toHaveLength(1);
    expect(r.pending[0].id).toBe("a-other-draft");
    expect(r.pending[0].work_item_id).toBe(OTHER_WI);
  });

  it("[9] draft single → 不允许 (boundary: 单一 draft)", () => {
    const artifacts = [mkArtifact("a1", WI, "draft")];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
    expect(r.pending).toHaveLength(1);
  });

  it("[10] in_review single → 不允许 (boundary: 单一 in_review)", () => {
    const artifacts = [mkArtifact("a1", WI, "in_review")];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
  });

  it("[11] rejected single → 不允许 (boundary: 单一 rejected)", () => {
    const artifacts = [mkArtifact("a1", WI, "rejected")];
    const r = checkAllArtifactsApproved(WI, artifacts);
    expect(r.allowed).toBe(false);
    expect(r.reason).toBe("pending_artifacts");
  });

  it("[12] version 字段不影响 Guard (即使 version=99 仍按 status 判定)", () => {
    // approved 就算 version=99 也通过
    const okArtifacts = [
      mkArtifact("a1", WI, "approved", 99),
      mkArtifact("a2", WI, "superseded", 50),
    ];
    const okR = checkAllArtifactsApproved(WI, okArtifacts);
    expect(okR.allowed).toBe(true);
    expect(okR.reason).toBe("all_approved");

    // draft 就算 version=1 也失败
    const badArtifacts = [mkArtifact("a1", WI, "draft", 1)];
    const badR = checkAllArtifactsApproved(WI, badArtifacts);
    expect(badR.allowed).toBe(false);
    expect(badR.reason).toBe("pending_artifacts");
  });
});
