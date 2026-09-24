// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/lib/api/__tests__/annotations.test.ts
//
// annotations API client 单测 (per ULYS-211 / FR-ORCA-035)。
// fetch 走 vitest.setup.ts 起的 MSW node server → src/mocks/handlers/annotations.ts。

import { beforeEach, describe, expect, it } from "vitest";

import {
  annotationsApi,
  AnnotationApiError,
  AUTHOR_USER,
  formatLineRange,
  lineSpanRange,
  rangeCoversLine,
  singleLineRange,
  toPromptFragment,
  validateLineRange,
  type CreateAnnotationRequest,
  type DiffAnnotation,
} from "@/lib/api/annotations";
import { DEMO_AGENT_RUN_ID, DEMO_FILE_PATH } from "@/mocks/data/annotations";
import { __resetAnnotationMock } from "@/mocks/handlers/annotations";

function addReq(overrides: Partial<CreateAnnotationRequest> = {}): CreateAnnotationRequest {
  return {
    file_path: DEMO_FILE_PATH,
    line_range: singleLineRange(15),
    body: "请把 unwrap 换成 ?",
    agent_run_id: DEMO_AGENT_RUN_ID,
    author: AUTHOR_USER,
    ...overrides,
  };
}

beforeEach(() => {
  __resetAnnotationMock();
});

// =====================================================================
// line_range 工具 — 1-based 半开区间, off-by-one 的唯一落点
// =====================================================================
describe("line_range helpers", () => {
  it("singleLineRange(N) 产出半开区间 [N, N+1)", () => {
    expect(singleLineRange(15)).toEqual({ start: 15, end: 16 });
  });

  it("lineSpanRange 归一化方向, 闭区间 [a,b] → [min, max+1)", () => {
    expect(lineSpanRange(12, 18)).toEqual({ start: 12, end: 19 });
    expect(lineSpanRange(18, 12)).toEqual({ start: 12, end: 19 });
  });

  it("rangeCoversLine 不含 end (半开)", () => {
    const r = singleLineRange(15);
    expect(rangeCoversLine(r, 14)).toBe(false);
    expect(rangeCoversLine(r, 15)).toBe(true);
    expect(rangeCoversLine(r, 16)).toBe(false);
  });

  it("formatLineRange 单行不显示区间, 多行显示末行 (end-1)", () => {
    expect(formatLineRange({ start: 15, end: 16 })).toBe("15");
    expect(formatLineRange({ start: 12, end: 19 })).toBe("12-18");
    // annotate.rs 只拒 start>end, 故空区间 [15,15) 合法
    expect(formatLineRange({ start: 15, end: 15 })).toBe("15");
  });

  it("validateLineRange 与 annotate.rs::validate_line_range 同规则", () => {
    expect(validateLineRange({ start: 1, end: 1 })).toBeNull();
    expect(validateLineRange({ start: 0, end: 3 })).toMatch(/>= 1/);
    expect(validateLineRange({ start: 9, end: 3 })).toMatch(/> end/);
  });
});

// =====================================================================
// REST — 6 API 对齐 AnnotationRegistry
// =====================================================================
describe("annotationsApi", () => {
  it("add 回传后端 serde 形态: author 内标签对象 + line_range 对象", async () => {
    const created = await annotationsApi.add(addReq());
    expect(created.author).toEqual({ kind: "user" });
    expect(created.line_range).toEqual({ start: 15, end: 16 });
    expect(created.file_path).toBe(DEMO_FILE_PATH);
    expect(created.id).toBeTruthy();
  });

  it("add 空 body 被后端判 invalid_field", async () => {
    await expect(annotationsApi.add(addReq({ body: "" }))).rejects.toMatchObject({
      code: "invalid_field",
      status: 400,
    });
  });

  it("add 非法 line_range 在发请求前就被挡下", async () => {
    const err = await annotationsApi
      .add(addReq({ line_range: { start: 0, end: 3 } }))
      .catch((e: unknown) => e);
    expect(err).toBeInstanceOf(AnnotationApiError);
    expect((err as AnnotationApiError).code).toBe("invalid_field");
  });

  it("add 未注册的 agent_run → agent_run_not_found", async () => {
    await expect(
      annotationsApi.add(addReq({ agent_run_id: "00000000-0000-0000-0000-000000000000" })),
    ).rejects.toMatchObject({ code: "agent_run_not_found", status: 404 });
  });

  it("get / listForFile / listForAgentRun 一致", async () => {
    const a = await annotationsApi.add(addReq());
    const b = await annotationsApi.add(addReq({ body: "这行注释补下 why" }));

    expect(await annotationsApi.get(a.id)).toEqual(a);

    const byFile = await annotationsApi.listForFile(DEMO_FILE_PATH);
    expect(byFile.map((x: DiffAnnotation) => x.id).sort()).toEqual([a.id, b.id].sort());

    const byRun = await annotationsApi.listForAgentRun(DEMO_AGENT_RUN_ID);
    expect(byRun).toHaveLength(2);

    // 别的文件拿不到
    expect(await annotationsApi.listForFile("src/other.rs")).toHaveLength(0);
  });

  it("get 不存在 → not_found", async () => {
    await expect(annotationsApi.get("nope")).rejects.toMatchObject({
      code: "not_found",
      status: 404,
    });
  });

  it("delete 后 listForFile 不再含该条", async () => {
    const a = await annotationsApi.add(addReq());
    await annotationsApi.delete(a.id);
    expect(await annotationsApi.listForFile(DEMO_FILE_PATH)).toHaveLength(0);
  });

  it("feedToAgent 无标注时 prompt_fragment 为 null (镜像 Option::None)", async () => {
    const res = await annotationsApi.feedToAgent(DEMO_AGENT_RUN_ID);
    expect(res.prompt_fragment).toBeNull();
  });

  it("feedToAgent 按 annotate.rs 格式拼接, 多条用 '---' 分隔", async () => {
    const a = await annotationsApi.add(addReq());
    const b = await annotationsApi.add(addReq({ body: "第二条" }));

    const res = await annotationsApi.feedToAgent(DEMO_AGENT_RUN_ID);
    expect(res.agent_run_id).toBe(DEMO_AGENT_RUN_ID);
    expect(res.prompt_fragment).toBe(
      `${toPromptFragment(a)}\n---\n\n${toPromptFragment(b)}`,
    );
    // Lines 用的是原始半开区间端点 (与 Rust 一致, 不是人读的 end-1)
    expect(res.prompt_fragment).toContain("Lines: 15-16");
    expect(res.prompt_fragment).toContain("By: user");
  });
});
