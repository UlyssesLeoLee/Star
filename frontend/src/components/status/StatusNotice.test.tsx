/*
@cypher schema=1 source_sha256=3f59a9bf5c824dfb5d0f8d0847a5b1feb43585dcaae76b00952c71b5cf6727ca
MERGE (self:File {path:"frontend/src/components/status/StatusNotice.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::StatusNotice",kind:"suite"})
MERGE (component:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::StatusNotice",kind:"component"})
MERGE (infer:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::inferNoticeState",kind:"function"})
MERGE (t01:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders loading notice with action hint",kind:"test"})
MERGE (t02:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders empty notice with action hint",kind:"test"})
MERGE (t03:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders disconnected notice with action hint",kind:"test"})
MERGE (t04:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders unauthorized notice with action hint",kind:"test"})
MERGE (t05:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders failed notice with action hint",kind:"test"})
MERGE (t06:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders recovering notice with action hint",kind:"test"})
MERGE (t07:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders success notice with action hint",kind:"test"})
MERGE (t08:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders custom title and hint",kind:"test"})
MERGE (t09:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::renders compact badge without hint",kind:"test"})
MERGE (t10:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::uses alert role for error states and status role otherwise",kind:"test"})
MERGE (t11:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState returns empty for no error and no data",kind:"test"})
MERGE (t12:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState returns success for data without error",kind:"test"})
MERGE (t13:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState classifies network errors as disconnected",kind:"test"})
MERGE (t14:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState classifies 401/403 responses as unauthorized",kind:"test"})
MERGE (t15:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState classifies service-unavailable statuses as disconnected",kind:"test"})
MERGE (t16:Symbol {id:"frontend/src/components/status/StatusNotice.test.tsx::inferNoticeState classifies other errors as failed",kind:"test"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(t01)
MERGE (suite)-[:DEFINES]->(t02)
MERGE (suite)-[:DEFINES]->(t03)
MERGE (suite)-[:DEFINES]->(t04)
MERGE (suite)-[:DEFINES]->(t05)
MERGE (suite)-[:DEFINES]->(t06)
MERGE (suite)-[:DEFINES]->(t07)
MERGE (suite)-[:DEFINES]->(t08)
MERGE (suite)-[:DEFINES]->(t09)
MERGE (suite)-[:DEFINES]->(t10)
MERGE (suite)-[:DEFINES]->(t11)
MERGE (suite)-[:DEFINES]->(t12)
MERGE (suite)-[:DEFINES]->(t13)
MERGE (suite)-[:DEFINES]->(t14)
MERGE (suite)-[:DEFINES]->(t15)
MERGE (suite)-[:DEFINES]->(t16)
MERGE (self)-[:IMPORTS]->(component)
MERGE (self)-[:IMPORTS]->(infer)
MERGE (t01)-[:TESTS]->(component)
MERGE (t02)-[:TESTS]->(component)
MERGE (t03)-[:TESTS]->(component)
MERGE (t04)-[:TESTS]->(component)
MERGE (t05)-[:TESTS]->(component)
MERGE (t06)-[:TESTS]->(component)
MERGE (t07)-[:TESTS]->(component)
MERGE (t08)-[:TESTS]->(component)
MERGE (t09)-[:TESTS]->(component)
MERGE (t10)-[:TESTS]->(component)
MERGE (t11)-[:TESTS]->(infer)
MERGE (t12)-[:TESTS]->(infer)
MERGE (t13)-[:TESTS]->(infer)
MERGE (t14)-[:TESTS]->(infer)
MERGE (t15)-[:TESTS]->(infer)
MERGE (t16)-[:TESTS]->(infer)
@endcypher
*/

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  NOTICE_STATES,
  StatusNotice,
  inferNoticeState,
  type NoticeState,
} from "./StatusNotice";

const DEFAULT_TITLE: Record<NoticeState, string> = {
  loading: "加载中",
  empty: "暂无数据",
  disconnected: "未连接",
  unauthorized: "无权限",
  failed: "失败",
  recovering: "恢复中",
  success: "操作成功",
};

describe("StatusNotice", () => {
  it.each(NOTICE_STATES)("renders %s notice with action hint", (state) => {
    const { container } = render(<StatusNotice state={state} />);
    const el = container.querySelector(`[data-testid="status-notice-${state}"]`);
    expect(el).not.toBeNull();
    const text = el?.textContent ?? "";
    expect(text).toContain(DEFAULT_TITLE[state]);
    // 每态必须带用户可执行的下一步提示 (中文祈使句)
    expect(text).toMatch(/请|重试|继续|查看|新建/);
  });

  it("renders custom title and hint", () => {
    render(<StatusNotice state="failed" title="自定义标题" hint="点击这里重试。" />);
    expect(screen.getByTestId("status-notice-failed").textContent).toContain("自定义标题");
    expect(screen.getByTestId("status-notice-failed").textContent).toContain("点击这里重试。");
    expect(screen.getByTestId("status-notice-failed").textContent).not.toContain(DEFAULT_TITLE.failed);
  });

  it("renders compact badge without hint", () => {
    render(<StatusNotice state="disconnected" compact />);
    const el = screen.getByTestId("status-notice-disconnected");
    expect(el.textContent).toContain(DEFAULT_TITLE.disconnected);
    // 密集模式只渲染图标 + 标题, 不渲染提示文案
    expect(el.textContent).not.toContain("请确认服务已启动");
  });

  it("uses alert role for error states and status role otherwise", () => {
    for (const state of ["failed", "unauthorized", "disconnected"] as const) {
      const view = render(<StatusNotice state={state} />);
      expect(view.getByRole("alert")).not.toBeNull();
      view.unmount();
    }
    for (const state of ["loading", "empty", "recovering", "success"] as const) {
      const view = render(<StatusNotice state={state} />);
      expect(view.getByRole("status")).not.toBeNull();
      view.unmount();
    }
  });

  it("inferNoticeState returns empty for no error and no data", () => {
    expect(inferNoticeState()).toBe("empty");
    expect(inferNoticeState(undefined, false)).toBe("empty");
    expect(inferNoticeState(null)).toBe("empty");
  });

  it("inferNoticeState returns success for data without error", () => {
    expect(inferNoticeState(undefined, true)).toBe("success");
  });

  it("inferNoticeState classifies network errors as disconnected", () => {
    expect(inferNoticeState(new Error("Failed to fetch"))).toBe("disconnected");
    expect(inferNoticeState("connect ECONNREFUSED 127.0.0.1:8081")).toBe("disconnected");
    expect(inferNoticeState({ code: "ECONNREFUSED" })).toBe("disconnected");
    expect(inferNoticeState(new Error("The operation timed out"))).toBe("disconnected");
  });

  it("inferNoticeState classifies 401/403 responses as unauthorized", () => {
    expect(inferNoticeState(new Error("Request failed with status code 401"))).toBe("unauthorized");
    expect(inferNoticeState({ status: 403 })).toBe("unauthorized");
    expect(inferNoticeState({ response: { status: 401 } })).toBe("unauthorized");
    expect(inferNoticeState(new Error("Invalid token"))).toBe("unauthorized");
  });

  it("inferNoticeState classifies service-unavailable statuses as disconnected", () => {
    expect(inferNoticeState(new Error("Bad Gateway: status 502"))).toBe("disconnected");
    expect(inferNoticeState({ status: 503 })).toBe("disconnected");
  });

  it("inferNoticeState classifies other errors as failed", () => {
    expect(inferNoticeState(new Error("Internal Server Error"))).toBe("failed");
    expect(inferNoticeState("boom")).toBe("failed");
    expect(inferNoticeState(500)).toBe("failed");
  });
});
