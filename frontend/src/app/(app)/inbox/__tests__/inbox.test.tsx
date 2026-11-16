/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/inbox/__tests__/inbox.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/inbox/__tests__/inbox.test.tsx::inbox 真实通知 API",kind:"suite"})
MERGE (page:File {path:"frontend/src/app/(app)/inbox/page.tsx"})
MERGE (i18n:File {path:"frontend/src/lib/i18n"})
MERGE (server:File {path:"frontend/src/mocks/server.ts"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(page)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (self)-[:IMPORTS]->(server)
@endcypher
*/
// =====================================================================
// inbox.test.tsx — 收件箱真实通知 API 行为测试 (per 2026-11-17)
//
// 覆盖 (任务书验收清单):
//   1. real-mode 未开启 → 诚实未接入态, 不渲染 mock 通知
//   2. 加载中态
//   3. 真实拉取成功渲染 + 分类 tabs 过滤
//   4. 空数据 → 诚实空态
//   5. 后端不可达 → "未接入" + 原因, 且不出现 mock 内容
//   6. 已读切换失败 → 乐观更新回滚
//   7. 已读切换成功 → 真实 API 生效
//
// 方式: real-mode 经 localStorage 开启 (use_real_api / real_api_base /
//   real_api_key), 真实请求走 realFetch → MSW 拦截 /api/v1/notifications*。
//   (per 守门 #25: 集成测试不发真 HTTP)
// =====================================================================

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup, fireEvent, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { http, HttpResponse, delay } from "msw";
import InboxPage from "../page";
import { I18nProvider } from "@/lib/i18n";
import { server } from "@/mocks/server";
import type { Notification } from "@/types/notifications";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

/** 开启 real-mode (localStorage 三档优先级最高) */
function enableRealMode() {
  window.localStorage.setItem("use_real_api", "true");
  window.localStorage.setItem("real_api_base", "http://localhost:3000");
  window.localStorage.setItem("real_api_key", "test-key");
}

const REAL_NOTIFS: Notification[] = [
  {
    id: "notif-1",
    tenant_id: "tenant-1",
    kind: "review_request",
    subject: "请评审 PR #50 关系权重实现",
    body: "Agent A 提交了关系权重实现，等待人工评审",
    subject_url: "/pr/50",
    actor_id: "usr-1",
    actor_name: "Agent A",
    target_type: "merge_request",
    target_id: "pr-50",
    read: false,
    created_at: "2026-11-17T02:00:00.000Z",
  },
  {
    id: "notif-2",
    tenant_id: "tenant-1",
    kind: "comment",
    subject: "评论 TASK-142 BFS 环检测",
    body: "自环判定已补，需要确认是否输出完整环路径",
    subject_url: null,
    actor_id: "usr-2",
    actor_name: "Agent B",
    target_type: "work_item",
    target_id: "wi-142",
    read: true,
    created_at: "2026-11-17T01:00:00.000Z",
  },
  {
    id: "notif-3",
    tenant_id: "tenant-1",
    kind: "system",
    subject: "CI failed: Star-frontend #117",
    body: "typecheck exit 1 — see log",
    subject_url: null,
    actor_id: null,
    actor_name: null,
    target_type: "agent",
    target_id: "ci-117",
    read: false,
    created_at: "2026-11-17T00:00:00.000Z",
  },
];

/** 后端 RestResponse envelope (与 crates/star-api-rest response.rs 对齐) */
function ok(data: unknown) {
  return HttpResponse.json({ data, meta: {} });
}

const MOCK_FIXTURE_MARKERS = [/PR #44/, /n-00[1-9]/, /wt-context-cache/, /budget_alert/];

describe("inbox 真实通知 API", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });
  afterEach(() => {
    cleanup();
  });

  it("real-mode 未开启 → 诚实未接入态, 不渲染 mock 通知", async () => {
    renderWithI18n(<InboxPage />);
    // 未接入 StatusNotice + 原因说明
    expect(await screen.findByTestId("status-notice-disconnected")).toBeInTheDocument();
    expect(screen.getByText(/未接入 — 真实通知服务不可用/)).toBeInTheDocument();
    expect(screen.getByTestId("inbox-error-detail")).toHaveTextContent(/real-mode 未开启/);
    // 不渲染列表, 不出现任何 mock 内容
    expect(screen.queryByTestId("inbox-list")).toBeNull();
    for (const marker of MOCK_FIXTURE_MARKERS) {
      expect(screen.queryByText(marker)).toBeNull();
    }
  });

  it("加载中态: 初始为 loading, 不预填 mock 数据", async () => {
    enableRealMode();
    server.use(
      http.get("/api/v1/notifications", async () => {
        await delay(120);
        return ok({ notifications: REAL_NOTIFS, total: 3, limit: 200, offset: 0 });
      }),
    );
    renderWithI18n(<InboxPage />);
    // 初始态 = 加载中 (不是 mock 数据)
    expect(screen.getByTestId("status-notice-loading")).toBeInTheDocument();
    expect(screen.queryByTestId("inbox-list")).toBeNull();
    // 请求完成后进入就绪态
    expect(await screen.findByTestId("inbox-item-notif-1")).toBeInTheDocument();
  });

  it("成功渲染真实通知 + 分类 tabs 过滤", async () => {
    enableRealMode();
    server.use(
      http.get("/api/v1/notifications", () =>
        ok({ notifications: REAL_NOTIFS, total: 3, limit: 200, offset: 0 }),
      ),
    );
    renderWithI18n(<InboxPage />);

    expect(await screen.findByTestId("inbox-item-notif-1")).toBeInTheDocument();
    expect(screen.getByTestId("inbox-item-notif-2")).toBeInTheDocument();
    expect(screen.getByTestId("inbox-item-notif-3")).toBeInTheDocument();
    // 未读徽标 = 2 (notif-1 / notif-3)
    expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("2");
    // 默认选中第一条 → 详情面板 + 跳转链接 (subject_url 存在)
    expect(screen.getByTestId("inbox-detail")).toBeInTheDocument();
    expect(screen.getByTestId("inbox-open-branch")).toHaveAttribute("href", "/pr/50");

    // 分类过滤: 评论 tab 只显示 notif-2
    fireEvent.click(screen.getByTestId("inbox-category-comment"));
    expect(screen.getByTestId("inbox-item-notif-2")).toBeInTheDocument();
    expect(screen.queryByTestId("inbox-item-notif-1")).toBeNull();
    expect(screen.queryByTestId("inbox-item-notif-3")).toBeNull();

    // 全部 tab 恢复 3 条
    fireEvent.click(screen.getByTestId("inbox-category-all"));
    expect(screen.getByTestId("inbox-item-notif-1")).toBeInTheDocument();
    expect(screen.getByTestId("inbox-item-notif-3")).toBeInTheDocument();
  });

  it("空数据 → 诚实空态", async () => {
    enableRealMode();
    server.use(
      http.get("/api/v1/notifications", () =>
        ok({ notifications: [], total: 0, limit: 200, offset: 0 }),
      ),
    );
    renderWithI18n(<InboxPage />);

    expect(await screen.findByTestId("status-notice-empty")).toBeInTheDocument();
    expect(screen.getByText("暂无通知")).toBeInTheDocument();
    expect(screen.queryByTestId("inbox-list")).toBeNull();
  });

  it("后端不可达 → 未接入 + 原因, 不出现 mock 内容", async () => {
    enableRealMode();
    server.use(
      http.get("/api/v1/notifications", () => HttpResponse.error()),
    );
    renderWithI18n(<InboxPage />);

    expect(await screen.findByTestId("status-notice-disconnected")).toBeInTheDocument();
    expect(screen.getByTestId("inbox-error-detail")).toHaveTextContent(/不可达/);
    expect(screen.getByTestId("inbox-retry")).toBeInTheDocument();
    // 关键纪律: 失败绝不回落 mock 数据
    expect(screen.queryByTestId("inbox-list")).toBeNull();
    for (const marker of MOCK_FIXTURE_MARKERS) {
      expect(screen.queryByText(marker)).toBeNull();
    }
  });

  it("已读切换失败 → 乐观更新后回滚", async () => {
    enableRealMode();
    server.use(
      http.get("/api/v1/notifications", () =>
        ok({ notifications: [REAL_NOTIFS[0]], total: 1, limit: 200, offset: 0 }),
      ),
      http.patch("/api/v1/notifications/:id", async () => {
        await delay(100);
        return HttpResponse.json({ error: { code: "internal" } }, { status: 500 });
      }),
    );
    renderWithI18n(<InboxPage />);
    await screen.findByTestId("inbox-item-notif-1");
    expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("1");

    // 点击标为已读 → 乐观更新 (徽标 0) → 失败 → 回滚 (徽标 1)
    fireEvent.click(screen.getByTestId("inbox-mark-read"));
    await waitFor(() => expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("0"));
    await waitFor(() => expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("1"));
    // 回滚提示展示
    expect(screen.getByTestId("status-notice-failed")).toBeInTheDocument();
    expect(screen.getByText(/已回滚/)).toBeInTheDocument();
  });

  it("已读切换成功 → 走真实 API 并更新状态", async () => {
    enableRealMode();
    let patchCount = 0;
    server.use(
      http.get("/api/v1/notifications", () =>
        ok({ notifications: [REAL_NOTIFS[0]], total: 1, limit: 200, offset: 0 }),
      ),
      http.patch("/api/v1/notifications/:id", async ({ request }) => {
        patchCount += 1;
        const body = (await request.json()) as { read?: boolean };
        return ok({ notification: { ...REAL_NOTIFS[0], read: body.read ?? true } });
      }),
    );
    renderWithI18n(<InboxPage />);
    await screen.findByTestId("inbox-item-notif-1");
    expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("1");

    fireEvent.click(screen.getByTestId("inbox-mark-read"));
    await waitFor(() => expect(screen.getByTestId("inbox-unread-count")).toHaveTextContent("0"));
    expect(patchCount).toBe(1);
    // 按钮切换为 标为未读
    expect(screen.getByTestId("inbox-mark-read")).toHaveTextContent("标为未读");
  });
});
