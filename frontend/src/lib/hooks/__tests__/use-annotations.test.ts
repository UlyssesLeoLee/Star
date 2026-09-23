// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/lib/hooks/__tests__/use-annotations.test.ts
//
// Annotation zustand store 单测 (per ULYS-211 / FR-ORCA-035)。
// 直接驱动 store action, 不经 React —— 渲染层覆盖在
// src/components/__tests__/DiffAnnotator.test.tsx。

import { beforeEach, describe, expect, it } from "vitest";

import { AUTHOR_USER, singleLineRange, type DiffAnnotation } from "@/lib/api/annotations";
import { useAnnotationStore } from "@/lib/hooks/use-annotations";
import { DEMO_AGENT_RUN_ID, DEMO_FILE_PATH } from "@/mocks/data/annotations";
import { __resetAnnotationMock } from "@/mocks/handlers/annotations";

const store = () => useAnnotationStore.getState();

function req(body: string, line = 15) {
  return {
    file_path: DEMO_FILE_PATH,
    line_range: singleLineRange(line),
    body,
    agent_run_id: DEMO_AGENT_RUN_ID,
    author: AUTHOR_USER,
  };
}

beforeEach(() => {
  __resetAnnotationMock();
  store().reset();
});

describe("useAnnotationStore", () => {
  it("addAnnotation 把结果并进 byFile, 无需再 loadForFile", async () => {
    const created = await store().addAnnotation(req("改这里"));
    expect(created).not.toBeNull();
    expect(store().byFile[DEMO_FILE_PATH]).toHaveLength(1);
    expect(store().byFile[DEMO_FILE_PATH][0].body).toBe("改这里");
    expect(store().error).toBeNull();
  });

  it("addAnnotation 失败时置 error 并返回 null (不抛)", async () => {
    const created = await store().addAnnotation({ ...req(""), body: "" });
    expect(created).toBeNull();
    expect(store().error).toMatch(/body/);
    expect(store().byFile[DEMO_FILE_PATH] ?? []).toHaveLength(0);
  });

  it("loadForFile 按 created_at ASC 排序 (后端 list_for_file 不保证顺序)", async () => {
    await store().addAnnotation(req("第一条"));
    await store().addAnnotation(req("第二条"));
    await store().addAnnotation(req("第三条"));

    store().reset();
    await store().loadForFile(DEMO_FILE_PATH);

    const bodies = store().byFile[DEMO_FILE_PATH].map((a: DiffAnnotation) => a.body);
    expect(bodies).toEqual(["第一条", "第二条", "第三条"]);
    expect(store().loadingFile[DEMO_FILE_PATH]).toBe(false);
  });

  it("loadForFile 只装该文件的标注", async () => {
    await store().addAnnotation(req("本文件"));
    await store().loadForFile("src/nothing-here.rs");
    expect(store().byFile["src/nothing-here.rs"]).toHaveLength(0);
  });

  it("deleteAnnotation 同时清本地投影", async () => {
    const created = await store().addAnnotation(req("待删"));
    await store().deleteAnnotation(created!.id, DEMO_FILE_PATH);
    expect(store().byFile[DEMO_FILE_PATH]).toHaveLength(0);

    // 服务端也删干净了
    await store().loadForFile(DEMO_FILE_PATH);
    expect(store().byFile[DEMO_FILE_PATH]).toHaveLength(0);
  });

  it("feedToAgent 返回 prompt fragment 并存进 store", async () => {
    await store().addAnnotation(req("unwrap 换成 ?"));
    const fragment = await store().feedToAgent(DEMO_AGENT_RUN_ID);

    expect(fragment).toContain("[Diff Annotation]");
    expect(fragment).toContain(`File: ${DEMO_FILE_PATH}`);
    expect(fragment).toContain("unwrap 换成 ?");
    expect(store().promptFragment).toBe(fragment);
    expect(store().feeding).toBe(false);
  });

  it("feedToAgent 无标注 → null; clearPromptFragment 复位", async () => {
    expect(await store().feedToAgent(DEMO_AGENT_RUN_ID)).toBeNull();
    store().clearPromptFragment();
    expect(store().promptFragment).toBeNull();
  });

  it("feedToAgent 未注册 run → error, 不改 promptFragment", async () => {
    const fragment = await store().feedToAgent("11111111-1111-1111-1111-111111111111");
    expect(fragment).toBeNull();
    expect(store().error).toMatch(/agent run not found/);
    expect(store().feeding).toBe(false);
  });

  it("upsertLocal 幂等 (同 id 覆盖而非追加), removeLocal 跨文件生效", () => {
    const base: DiffAnnotation = {
      id: "ws-1",
      file_path: DEMO_FILE_PATH,
      line_range: singleLineRange(20),
      body: "来自 ws 推送",
      agent_run_id: DEMO_AGENT_RUN_ID,
      created_at: new Date().toISOString(),
      author: AUTHOR_USER,
    };
    store().upsertLocal(base);
    store().upsertLocal({ ...base, body: "改过的 ws 推送" });

    expect(store().byFile[DEMO_FILE_PATH]).toHaveLength(1);
    expect(store().byFile[DEMO_FILE_PATH][0].body).toBe("改过的 ws 推送");

    store().removeLocal("ws-1");
    expect(store().byFile[DEMO_FILE_PATH]).toHaveLength(0);
  });
});
