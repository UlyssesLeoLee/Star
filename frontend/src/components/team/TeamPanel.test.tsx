/*
@cypher schema=1 source_sha256=a9fbab1e22440c56b5e885f9a3b0adc0c1274074d525574bff6a83a28dad710f
MERGE (self:File {path:"frontend/src/components/team/TeamPanel.test.tsx"})
MERGE (panel:File {path:"frontend/src/components/team/TeamPanel.tsx"})
MERGE (api:File {path:"frontend/src/components/team/runtimeApi.ts"})
MERGE (self)-[:TESTS]->(panel)
MERGE (self)-[:TESTS]->(api)
@endcypher
*/

// =====================================================================
// TeamPanel unit tests
//
// 验收:
//   - 降级路径 (fetch 失败/超时): 保留 store seed 行 + "预览数据"/"未接入"
//     徽标, 注册 / 心跳 / 启动 / 停止 / 重启全部禁用
//   - 成功路径 (GET /api/v1/runtime 返回 API 列表): 用 API 数据渲染,
//     移除该区 预览数据/未接入 徽标, 注册 + 心跳启用,
//     启动 / 停止 / 重启仍禁用 (控制通道未接) + title 原因
//   - 动作接线: 注册 → POST /api/v1/runtime/register 后刷新列表;
//     心跳 → POST /api/v1/runtime/{id}/heartbeat 后刷新列表
// =====================================================================

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { TeamPanel } from "./TeamPanel";
import { useStore } from "@/lib/store";
import { localRuntimes as seedLocalRuntimes } from "@/lib/seed";

// ---- fetch mock 工具 --------------------------------------------------

function jsonResponse(body: unknown, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText: String(status),
    json: async () => body,
  } as unknown as Response;
}

interface FetchCall {
  url: string;
  method: string;
}

function makeFetchMock(routes: Array<{ method: string; path: string; body: () => unknown }>) {
  const calls: FetchCall[] = [];
  const fn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    const method = init?.method ?? "GET";
    calls.push({ url, method });
    const route = routes.find((r) => r.method === method && url.includes(r.path));
    if (!route) throw new Error(`unexpected fetch: ${method} ${url}`);
    return jsonResponse(route.body());
  });
  return { fn, calls };
}

// 后端信封形状: RestResponse({data: [...]}) — 验证 runtimeApi 的宽容解析
const API_RUNTIMES_FIXTURE = {
  data: [
    {
      id: "rt-9001",
      tenant_id: "t-1",
      device_id: "dev-9001",
      hostname: "api-host-01",
      status: "online",
      bound_user_id: "usr-001",
      bound_tenant_id: "t-1",
      mount_root: "D:/Dev",
      last_heartbeat_at: "2026-11-16T10:00:00Z",
      policy_violations: 0,
    },
    {
      id: "rt-9002",
      tenant_id: "t-1",
      device_id: "dev-9002",
      hostname: "api-host-02",
      status: "offline",
      bound_user_id: "usr-002",
      bound_tenant_id: "t-1",
      mount_root: "D:/Dev2",
      last_heartbeat_at: "2026-11-16T09:00:00Z",
      policy_violations: 2,
    },
  ],
};

describe("TeamPanel", () => {
  beforeEach(() => {
    // store 初始态已是诚实空态 (per 2026-11-16 seed 清除) → 测试显式注入 fixture
    useStore.setState({ localRuntimes: seedLocalRuntimes as never });
    // 默认: 后端不可达 → 组件走降级路径 (确定性, 不依赖真实网络)
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("backend unreachable (test)")));
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the panel root and runtime rows from the store (降级)", async () => {
    render(<TeamPanel />);
    await act(async () => {}); // flush 挂载时 fetch 的失败落定

    expect(screen.getByTestId("team-panel")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-001")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-002")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-005")).toBeTruthy();
  });

  it("renders squad preview rows (Worker and Router)", async () => {
    render(<TeamPanel />);
    await act(async () => {});

    expect(screen.getByTestId("team-squad-row-sq-001")).toBeTruthy();
    expect(screen.getByTestId("team-squad-row-sq-002")).toBeTruthy();
    expect(screen.getByTestId("team-squad-row-sq-003")).toBeTruthy();
  });

  it("disables every runtime action button and exposes the reason via title", async () => {
    render(<TeamPanel />);
    await act(async () => {});

    for (const action of ["start", "stop", "restart"] as const) {
      const button = screen.getByTestId(`team-action-${action}-lr-001`) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      expect(button.title).toContain("未接入");
    }
  });

  it("disables every squad action button and exposes the reason via title", async () => {
    render(<TeamPanel />);
    await act(async () => {});

    for (const action of ["assign", "recall"] as const) {
      const button = screen.getByTestId(`team-action-${action}-sq-001`) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      expect(button.title).toContain("未接入");
    }
  });

  it("marks each row with an explicit 未接入 badge (降级)", async () => {
    render(<TeamPanel />);
    await act(async () => {});

    for (const rowId of [
      "team-runtime-row-lr-001",
      "team-runtime-row-lr-005",
      "team-squad-row-sq-001",
      "team-squad-row-sq-003",
    ]) {
      const row = screen.getByTestId(rowId);
      expect(within(row).getByText("未接入")).toBeTruthy();
    }
  });

  it("降级路径: fetch 失败时保留预览徽标并禁用注册/心跳", async () => {
    render(<TeamPanel />);

    // 失败落定 → 头部徽标 title 变为"不可达"
    await waitFor(() => {
      expect(screen.getByText("预览数据").title).toContain("不可达");
    });

    expect(screen.getByTestId("team-runtime-row-lr-001")).toBeTruthy();
    const register = screen.getByTestId("team-action-register") as HTMLButtonElement;
    expect(register.disabled).toBe(true);
    expect(register.title).toContain("未接入");
    const heartbeat = screen.getByTestId("team-action-heartbeat-lr-001") as HTMLButtonElement;
    expect(heartbeat.disabled).toBe(true);
    expect(heartbeat.title).toContain("未接入");
  });

  it("成功路径: GET /api/v1/runtime 返回后用 API 数据渲染并移除预览/未接入徽标", async () => {
    const { fn } = makeFetchMock([
      { method: "GET", path: "/api/v1/runtime", body: () => API_RUNTIMES_FIXTURE },
    ]);
    vi.stubGlobal("fetch", fn);

    render(<TeamPanel />);

    // API 行渲染 (不再是 store seed)
    const apiRow = await screen.findByTestId("team-runtime-row-rt-9001");
    expect(screen.getByTestId("team-runtime-row-rt-9002")).toBeTruthy();
    expect(screen.queryByTestId("team-runtime-row-lr-001")).toBeNull();

    // 该区 预览数据 / 行尾 未接入 徽标移除
    expect(screen.queryByText("预览数据")).toBeNull();
    expect(within(apiRow).queryByText("未接入")).toBeNull();
    // 数据源标注切换为实时数据
    expect(screen.getByText("star-api-rest /api/v1/runtime · 实时数据")).toBeTruthy();

    // 注册 + 心跳启用
    const register = screen.getByTestId("team-action-register") as HTMLButtonElement;
    expect(register.disabled).toBe(false);
    const heartbeat = screen.getByTestId("team-action-heartbeat-rt-9001") as HTMLButtonElement;
    expect(heartbeat.disabled).toBe(false);

    // 启动/停止/重启仍禁用 (控制通道未接) + title 原因
    for (const action of ["start", "stop", "restart"] as const) {
      const button = screen.getByTestId(`team-action-${action}-rt-9001`) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      expect(button.title).toContain("控制通道");
    }
  });

  it("注册: 点击后 POST /api/v1/runtime/register 并刷新列表", async () => {
    let listBody: unknown = API_RUNTIMES_FIXTURE;
    const { fn, calls } = makeFetchMock([
      { method: "GET", path: "/api/v1/runtime", body: () => listBody },
      { method: "POST", path: "/api/v1/runtime/register", body: () => ({ ok: true }) },
    ]);
    vi.stubGlobal("fetch", fn);

    render(<TeamPanel />);
    await screen.findByTestId("team-runtime-row-rt-9001");

    // 注册成功后再挂一台新设备 → 刷新后应出现新行
    listBody = {
      data: [
        ...API_RUNTIMES_FIXTURE.data,
        {
          id: "rt-9003",
          tenant_id: "t-1",
          device_id: "dev-9003",
          hostname: "api-host-03",
          status: "registered",
          bound_user_id: "usr-003",
          bound_tenant_id: "t-1",
          mount_root: "D:/Dev3",
          last_heartbeat_at: "2026-11-16T11:00:00Z",
          policy_violations: 0,
        },
      ],
    };

    fireEvent.click(screen.getByTestId("team-action-register"));

    await waitFor(() => {
      expect(calls.some((c) => c.method === "POST" && c.url.includes("/api/v1/runtime/register"))).toBe(true);
    });
    // 注册后刷新列表 → 新行出现
    await screen.findByTestId("team-runtime-row-rt-9003");
    expect(calls.filter((c) => c.method === "GET" && c.url.includes("/api/v1/runtime")).length).toBe(2);
  });

  it("心跳: 点击后 POST /api/v1/runtime/{id}/heartbeat 并刷新列表", async () => {
    const { fn, calls } = makeFetchMock([
      { method: "GET", path: "/api/v1/runtime", body: () => API_RUNTIMES_FIXTURE },
      { method: "POST", path: "/heartbeat", body: () => ({ ok: true }) },
    ]);
    vi.stubGlobal("fetch", fn);

    render(<TeamPanel />);
    await screen.findByTestId("team-runtime-row-rt-9001");

    fireEvent.click(screen.getByTestId("team-action-heartbeat-rt-9001"));

    await waitFor(() => {
      expect(
        calls.some((c) => c.method === "POST" && c.url.includes("/api/v1/runtime/rt-9001/heartbeat")),
      ).toBe(true);
    });
    // 心跳后刷新列表
    await waitFor(() => {
      expect(calls.filter((c) => c.method === "GET" && c.url.includes("/api/v1/runtime")).length).toBe(2);
    });
  });
});
