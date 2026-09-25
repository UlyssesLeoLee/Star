// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/e2e/diff-annotation.spec.ts
//
// FR-ORCA-035 前端 diff 标注 E2E (per ULYS-211, 父 ULYS-201)。
//
// 3 条链路 (per issue 产出物 "3 E2E Playwright 跑通 add → list → feed_to_agent"):
//   annotation-01-add   : 在 diff 行 drop comment → 列表 + 行角标出现
//   annotation-02-list  : 刷新后 list_for_file 把标注读回来 (含多行半开区间)
//   annotation-03-feed  : feed_to_agent 产出下一轮 prompt fragment
//
// 为什么用 page.route 而不是 MSW browser worker:
//   MSW 走 service worker, 首次注册与页面首帧存在竞态 (既有 e2e 全部
//   test.skip 的历史原因之一); page.route 在 Node 侧拦截, 无竞态且不依赖
//   NEXT_PUBLIC_API_MOCKING 环境变量。拦截的 wire 契约与
//   src/mocks/handlers/annotations.ts 完全一致 (即 annotate.rs 的 serde 形态)。
//
// 跑法 (config 的 webServer 起在 3000 而 baseURL 是 3001, 属既有配置问题,
// 不在本 issue 范围内改):
//   npx next build && npx next start -p 3001
//   PLAYWRIGHT_SKIP_WEBSERVER=1 npx playwright test e2e/diff-annotation.spec.ts --project=chromium

import { test, expect, type Page } from "@playwright/test";

const PAGE_URL = "/development/annotate";
const FILE_PATH = "crates/agent-bridge/src/annotate.rs";
const AGENT_RUN_ID = "3f1c8a02-5b7e-4c21-9d6a-8e0f12b4c7d5";

interface LineRange {
  start: number;
  end: number;
}

interface StubAnnotation {
  id: string;
  file_path: string;
  line_range: LineRange;
  body: string;
  agent_run_id: string;
  created_at: string;
  author: { kind: "user" | "agent" };
}

/** mirrors DiffAnnotation::to_prompt_fragment (crates/agent-bridge/src/annotate.rs) */
function toPromptFragment(a: StubAnnotation): string {
  return (
    `[Diff Annotation]\n` +
    `File: ${a.file_path}\n` +
    `Lines: ${a.line_range.start}-${a.line_range.end}\n` +
    `By: ${a.author.kind}\n` +
    `At: ${a.created_at}\n` +
    `Body: ${a.body}\n`
  );
}

/**
 * 在 Node 侧起一个 AnnotationRegistry 等价桩 (in-memory, 每个 test 独立),
 * 并挡掉 onboarding 弹窗。
 */
async function stubAnnotationApi(page: Page): Promise<StubAnnotation[]> {
  const store: StubAnnotation[] = [];
  let seq = 0;

  await page.addInitScript(() => {
    // OnboardingGuard 首启会弹引导遮罩, 会吃掉 gutter 的点击
    window.localStorage.setItem("star:onboarding-completed", "true");
  });

  await page.route("**/api/v2/annotations**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const method = request.method();

    // POST /annotations/feed?agent_run_id=
    if (method === "POST" && url.pathname.endsWith("/annotations/feed")) {
      const runId = url.searchParams.get("agent_run_id") ?? "";
      const list = store.filter((a) => a.agent_run_id === runId);
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          agent_run_id: runId,
          prompt_fragment:
            list.length === 0 ? null : list.map(toPromptFragment).join("\n---\n\n"),
        }),
      });
      return;
    }

    // POST /annotations
    if (method === "POST") {
      const req = request.postDataJSON() as Omit<StubAnnotation, "id" | "created_at">;
      seq += 1;
      const created: StubAnnotation = {
        ...req,
        id: `e2e-ann-${seq}`,
        created_at: new Date(Date.now() + seq).toISOString(),
      };
      store.push(created);
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(created),
      });
      return;
    }

    // DELETE /annotations/:id
    if (method === "DELETE") {
      const id = url.pathname.split("/").pop();
      const idx = store.findIndex((a) => a.id === id);
      if (idx >= 0) store.splice(idx, 1);
      await route.fulfill({ status: 204, body: "" });
      return;
    }

    // GET /annotations?file_path= | ?agent_run_id=
    const filePath = url.searchParams.get("file_path");
    const runId = url.searchParams.get("agent_run_id");
    const list = store.filter((a) =>
      filePath !== null ? a.file_path === filePath : a.agent_run_id === runId,
    );
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(list),
    });
  });

  return store;
}

/** 在第 line 行 drop 一条 comment */
async function dropComment(page: Page, line: number, body: string) {
  await page.getByTestId(`annotation-gutter-${line}`).click();
  await page.getByTestId("annotation-body-input").fill(body);
  await page.getByTestId("annotation-submit").click();
}

test.describe("FR-ORCA-035 diff annotation", () => {
  test("annotation-01-add-comment-on-diff-line", async ({ page }) => {
    const store = await stubAnnotationApi(page);
    await page.goto(PAGE_URL);

    await expect(page.getByTestId("diff-annotator")).toBeVisible();
    await expect(page.getByTestId("annotation-empty")).toBeVisible();

    await dropComment(page, 15, "now_v7 需要 uuid 的 v7 feature, 确认 Cargo.toml 开了");

    const items = page.getByTestId("annotation-item");
    await expect(items).toHaveCount(1);
    await expect(items.first()).toContainText("now_v7 需要 uuid 的 v7 feature");
    await expect(items.first()).toContainText("L15");
    await expect(page.getByTestId("annotation-marker-15")).toHaveText("1");
    await expect(page.getByTestId("annotation-composer")).toHaveCount(0);

    // 发到后端的 payload 就是 annotate.rs 的 serde 形态
    expect(store).toHaveLength(1);
    expect(store[0].file_path).toBe(FILE_PATH);
    expect(store[0].agent_run_id).toBe(AGENT_RUN_ID);
    expect(store[0].line_range).toEqual({ start: 15, end: 16 }); // 1-based 半开
    expect(store[0].author).toEqual({ kind: "user" });
  });

  test("annotation-02-list-for-file-after-reload", async ({ page }) => {
    await stubAnnotationApi(page);
    await page.goto(PAGE_URL);
    await expect(page.getByTestId("annotation-empty")).toBeVisible();

    await dropComment(page, 15, "第一条: 单行标注");
    await expect(page.getByTestId("annotation-item")).toHaveCount(1);

    // shift+click 扩选 16..17 → 半开区间 [16, 18)
    await page.getByTestId("annotation-gutter-16").click();
    await page.getByTestId("annotation-gutter-17").click({ modifiers: ["Shift"] });
    await expect(page.getByTestId("composer-range")).toContainText("行 16-17");
    await page.getByTestId("annotation-body-input").fill("第二条: 这两行合并");
    await page.getByTestId("annotation-submit").click();
    await expect(page.getByTestId("annotation-item")).toHaveCount(2);

    // reload → 走 list_for_file 重新装载
    await page.reload();
    await expect(page.getByTestId("annotation-item")).toHaveCount(2);
    await expect(page.getByTestId("annotation-count")).toHaveText("2 条标注");

    const bodies = await page.getByTestId("annotation-body").allTextContents();
    expect(bodies).toEqual(["第一条: 单行标注", "第二条: 这两行合并"]); // created_at ASC

    await expect(page.getByTestId("annotation-marker-16")).toBeVisible();
    await expect(page.getByTestId("annotation-marker-17")).toBeVisible();
    await expect(page.getByTestId("annotation-marker-18")).toHaveCount(0); // end 不含
  });

  test("annotation-03-feed-to-agent-prompt-fragment", async ({ page }) => {
    await stubAnnotationApi(page);
    await page.goto(PAGE_URL);
    await expect(page.getByTestId("annotation-empty")).toBeVisible();

    // 没标注时不给 feed
    await expect(page.getByTestId("feed-to-agent-button")).toBeDisabled();

    await dropComment(page, 15, "这里要处理 None 分支");
    await dropComment(page, 19, "created_at 用注入的 clock, 别直接 Utc::now()");
    await expect(page.getByTestId("annotation-item")).toHaveCount(2);

    await page.getByTestId("feed-to-agent-button").click();

    const fragment = page.getByTestId("prompt-fragment");
    await expect(fragment).toBeVisible();
    await expect(fragment).toContainText("[Diff Annotation]");
    await expect(fragment).toContainText(`File: ${FILE_PATH}`);
    await expect(fragment).toContainText("Lines: 15-16");
    await expect(fragment).toContainText("By: user");
    await expect(fragment).toContainText("这里要处理 None 分支");
    // 多条之间用 '---' 分隔 (per AnnotationRegistry::feed_to_agent)
    await expect(fragment).toContainText("---");
    await expect(fragment).toContainText("created_at 用注入的 clock");
  });
});
