// =====================================================================
// Defect Page Test (per G-5 / G-10 / G-14 / G-17 / N-01)
//
// 覆盖:
//   - 三态渲染 (有值 / （未填写）/ （未接入）) + data-field-state 标注
//   - 验证状态绝不从看板 status==="done" 推断 (只认 latest_verification)
//   - REST 读模型优先 (来源标注) + 服务端 degraded 如实呈现
//   - REST 失败 → 显式降级: 「来源：降级路径」+ 顶部横幅 + 多 worktree 部分失败可见
//   - 纯函数: extensionFieldState / verificationState 三态判定
//
// mock 的是网络边界 (fetch / real-mode), 不是业务数据 (per 共享约束前端纪律)。
// =====================================================================

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import DefectPage, {
  extensionFieldState,
  renderExtensionFieldText,
  renderVerificationText,
  verificationState,
  type DefectItem,
} from "./page";
import { I18nProvider } from "@/lib/i18n";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import { useNavStore } from "@/lib/nav/navStore";
import { useStore } from "@/lib/store";

vi.mock("@/lib/auth", () => ({
  getSession: () => ({
    token: "test-jwt",
    payload: {
      sub: "u-1",
      iss: "test",
      aud: "test",
      exp: 4102444800,
      iat: 1,
      tenant_id: "tn-defect-1",
      user_id: "usr-defect-1",
      roles: [],
      scope: "openid",
    },
    isAdmin: false,
  }),
}));

const PROJECT_ID = "prj-defect-1";
const WT_OK = "11111111-1111-4111-8111-111111111111";
const WT_FAIL = "22222222-2222-4222-8222-222222222222";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

function renderPage(groupFetcher?: typeof fetch) {
  return renderWithI18n(
    <WorktreeGroupApiProvider
      getAccessToken={async () => "jwt"}
      sessionKey="defect-test"
      fetcher={groupFetcher}
    >
      <DefectPage />
    </WorktreeGroupApiProvider>,
  );
}

// ---- REST 读模型 fixture (与 DefectListItem 物理契约对齐) ----
function restItem(overrides: Record<string, unknown> = {}) {
  return {
    id: "def-001",
    title: "登录页崩溃",
    status: "Closed",
    severity: "S2",
    priority: "P1",
    reporter_id: "usr-defect-1",
    assignee_id: null,
    lane_id: null,
    worktree_id: null,
    branch_name: null,
    fix_version: "2.1.0",
    lifecycle_version: 3,
    updated_at: "2026-11-25T00:00:00Z",
    ...overrides,
  };
}

function restList(items: unknown[], extra: Record<string, unknown> = {}) {
  return {
    data: {
      items,
      total: items.length,
      page: 1,
      page_size: 200,
      degraded: false,
      degradation_reason: null,
      ...extra,
    },
    meta: {},
  };
}

function groupBugItem(overrides: Record<string, unknown> = {}) {
  return {
    work_item_id: "wi-bug-1",
    item_type: "bug",
    title: "降级路径缺陷",
    description: "来自 worktree 投影的描述",
    priority: "high",
    labels: [],
    reporter_user_id: "usr-defect-1",
    lifecycle: {
      status: "pending",
      review_state: "none",
      active_worktree_id: null,
      version: 1,
      updated_at: new Date().toISOString(),
    },
    metadata_updated_at: new Date().toISOString(),
    ...overrides,
  };
}

function worktreeFixture(id: string, laneId?: string) {
  return {
    id,
    tenant_id: "tn-defect-1",
    project_id: PROJECT_ID,
    lane_id: laneId,
    name: `wt-${id.slice(0, 8)}`,
    branch: "dev",
    base_branch: "main",
    status: "active",
    lock_version: 1,
    last_event_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
  };
}

beforeEach(() => {
  useNavStore.setState({
    selectedProjectId: PROJECT_ID,
    selectedLaneId: "",
    selectedWorktreeId: "",
    selectedBranch: "",
  });
  useStore.setState({ workItems: [], worktrees: [], identities: [], comments: [] });
  // real-mode 开启 → defectApi 走真实 fetch (测试内 stub 全局 fetch)
  window.localStorage.setItem("use_real_api", "true");
  window.localStorage.setItem("real_api_base", "http://localhost:3000");
  window.history.replaceState(null, "", "/defect");
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
  window.localStorage.clear();
});

describe("DefectPage REST 读模型 (N-01)", () => {
  it("优先走 REST 读模型: 来源标注 + 缺陷扩展字段来自读模型", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/v1/defects")) {
        return new Response(
          JSON.stringify(
            restList([
              restItem({ id: "def-a", title: "登录页崩溃 (已关闭)" }),
              restItem({
                id: "def-b",
                title: "接口超时",
                status: "InProgress",
                severity: "S3",
                fix_version: null,
              }),
            ]),
          ),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);

    renderPage();

    const source = await screen.findByTestId("defect-data-source");
    expect(source.textContent).toContain("来源：REST 读模型");
    expect(await screen.findByText("登录页崩溃 (已关闭)")).toBeInTheDocument();
    expect(screen.getByText("接口超时")).toBeInTheDocument();
    // 列表行严重度来自读模型 (不再硬编码 "unknown")
    expect(screen.getByText("S2")).toBeInTheDocument();
    // REST 成功 → 无降级横幅 / 无部分失败
    expect(screen.queryByTestId("defect-source-unavailable-banner")).not.toBeInTheDocument();
    expect(screen.queryByTestId("defect-partial-failure")).not.toBeInTheDocument();
  });

  it("服务端 degraded=true → 顶部横幅如实标注 InMemory 降级", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/v1/defects")) {
        return new Response(
          JSON.stringify(
            restList(
              [restItem({ id: "def-a" })],
              { degraded: true, degradation_reason: "InMemory 降级: 数据不持久" },
            ),
          ),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);

    renderPage();

    await screen.findByText("登录页崩溃");
    expect(screen.getByTestId("defect-data-source").textContent).toContain("服务端标注降级");
    const banner = screen.getByTestId("defect-source-unavailable-banner");
    expect(banner.textContent).toContain("InMemory 降级: 数据不持久");
  });
});

describe("DefectPage 扩展字段三态渲染 (G-14 / G-10)", () => {
  async function renderWithItems(items: unknown[]) {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes("/api/v1/defects")) {
        return new Response(JSON.stringify(restList(items)), { status: 200 });
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);
    renderPage();
    await screen.findByTestId("defect-data-source");
  }

  it("有值 / 未填写 / 未接入 三态 + 读模型缺失列不得推断", async () => {
    await renderWithItems([
      restItem({
        id: "def-a",
        title: "已修复缺陷",
        status: "Closed",
        fix_version: "2.1.0",
      }),
      restItem({
        id: "def-b",
        title: "未填版本缺陷",
        status: "InProgress",
        fix_version: null,
      }),
    ]);

    // 选中 def-a: fix_version 有值; actual_result 列缺失 → 未接入;
    // 看板列 done (Closed) 但验证状态绝不推断为「已验证」
    fireEvent.click(screen.getByText("已修复缺陷"));
    await waitFor(() =>
      expect(screen.getByTestId("defect-field-fix-version")).toHaveTextContent("2.1.0"),
    );
    expect(screen.getByTestId("defect-field-fix-version")).toHaveAttribute(
      "data-field-state",
      "value",
    );
    expect(screen.getByTestId("defect-field-actual-result")).toHaveAttribute(
      "data-field-state",
      "unavailable",
    );
    expect(screen.getByTestId("defect-field-actual-result")).toHaveTextContent("（未接入）");
    expect(screen.getByTestId("defect-field-verification")).toHaveTextContent("（未接入）");
    // 旧实现会在此处显示「已验证」(status==="done"), 必须不存在
    expect(screen.queryByText("已验证")).not.toBeInTheDocument();
    expect(screen.queryByText(/已验证（/)).not.toBeInTheDocument();
    // 读模型缺列 → 扩展区块顶部标注来源不可用 (per D-DI-06)
    expect(screen.getByTestId("defect-extension-source-unavailable")).toBeInTheDocument();

    // 选中 def-b: fix_version 列已接入但为空 → 未填写
    fireEvent.click(screen.getByText("未填版本缺陷"));
    await waitFor(() =>
      expect(screen.getByTestId("defect-field-fix-version")).toHaveTextContent("（未填写）"),
    );
    expect(screen.getByTestId("defect-field-fix-version")).toHaveAttribute(
      "data-field-state",
      "empty",
    );
  });

  it("验证状态只来自真实字段 latest_verification (passed / null)", async () => {
    const verification = {
      verifier_id: "usr-verify-1",
      fix_version: "3.0.0",
      method: "manual",
      result: "passed",
      evidence_id: null,
      created_at: "2026-11-25T01:00:00Z",
    };
    await renderWithItems([
      restItem({
        id: "def-c",
        title: "已验证缺陷",
        status: "Closed",
        latest_verification: verification,
      }),
      restItem({
        id: "def-d",
        title: "未验证缺陷",
        status: "Closed",
        latest_verification: null,
      }),
    ]);

    fireEvent.click(screen.getByText("已验证缺陷"));
    await waitFor(() =>
      expect(screen.getByTestId("defect-field-verification")).toHaveTextContent("已验证（3.0.0）"),
    );
    expect(screen.getByTestId("defect-field-verification")).toHaveAttribute(
      "data-field-state",
      "passed",
    );

    fireEvent.click(screen.getByText("未验证缺陷"));
    await waitFor(() =>
      expect(screen.getByTestId("defect-field-verification")).toHaveTextContent("（未提交）"),
    );
    expect(screen.getByTestId("defect-field-verification")).toHaveAttribute(
      "data-field-state",
      "unsubmitted",
    );
  });
});

describe("DefectPage 显式降级 + 多 worktree 部分失败 (G-17 / N-01)", () => {
  it("REST 失败 → 「来源：降级路径」+ 顶部横幅 + 逐 worktree 失败可见", async () => {
    // REST 读模型不可用
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(JSON.stringify({ error: "boom" }), { status: 500 })) as unknown as typeof fetch,
    );

    useStore.setState({
      worktrees: [worktreeFixture(WT_OK), worktreeFixture(WT_FAIL)],
    });

    const groupFetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes(`/api/v1/worktrees/${WT_OK}/work-items`)) {
        return new Response(
          JSON.stringify({
            worktree_id: WT_OK,
            project_id: PROJECT_ID,
            work_items: [groupBugItem()],
          }),
          { status: 200 },
        );
      }
      if (url.includes(`/api/v1/worktrees/${WT_FAIL}/work-items`)) {
        return new Response(
          JSON.stringify({ error: { code: "worktree_unavailable" } }),
          { status: 503 },
        );
      }
      return new Response(JSON.stringify({ error: { code: "not_found" } }), { status: 404 });
    }) as unknown as typeof fetch;

    renderPage(groupFetcher);

    // 来源显式标注降级路径
    const source = await screen.findByTestId("defect-data-source");
    expect(source.textContent).toContain("来源：降级路径");
    // 顶部横幅: 读模型失败原因可见
    expect(screen.getByTestId("defect-source-unavailable-banner")).toBeInTheDocument();
    // 部分失败指示器: 1/2, 失败 worktree 与原因可见
    const partial = await screen.findByTestId("defect-partial-failure");
    expect(partial.textContent).toContain("1/2");
    expect(partial.textContent).toContain("当前列表不完整");
    const failedRow = screen.getByTestId(`defect-worktree-failure-${WT_FAIL}`);
    expect(failedRow.textContent).toContain(WT_FAIL);
    // 成功 worktree 的 bug 出现在列表 (局部结果 + 失败范围同时可见)
    expect(await screen.findByText("降级路径缺陷")).toBeInTheDocument();
  });

  it("REST 与 worktreeGroupApi 均不可用 → 显式错误, 不呈现空列表冒充完整", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(JSON.stringify({ error: "boom" }), { status: 500 })) as unknown as typeof fetch,
    );
    useStore.setState({ worktrees: [worktreeFixture(WT_OK)] });

    // 不提供 group fetcher → useWorktreeGroupApi 为 null → 降级路径不可达
    renderWithI18n(<DefectPage />);

    const source = await screen.findByTestId("defect-data-source");
    expect(source.textContent).toContain("来源：降级路径");
    expect(screen.getByTestId("defect-source-unavailable-banner")).toBeInTheDocument();
    expect(screen.getByText(/加载失败/)).toBeInTheDocument();
  });
});

describe("三态渲染纯函数 (G-14)", () => {
  it("extensionFieldState / renderExtensionFieldText", () => {
    expect(extensionFieldState("有值", true)).toBe("value");
    expect(extensionFieldState("", true)).toBe("empty");
    expect(extensionFieldState(null, true)).toBe("empty");
    expect(extensionFieldState("有值", false)).toBe("unavailable");
    expect(extensionFieldState(undefined, false)).toBe("unavailable");
    expect(extensionFieldState(undefined, true)).toBe("empty");

    expect(renderExtensionFieldText("有值", true)).toBe("有值");
    expect(renderExtensionFieldText(null, true)).toBe("（未填写）");
    expect(renderExtensionFieldText(null, false)).toBe("（未接入）");
  });

  it("verificationState 绝不从看板状态推断", () => {
    const verification: NonNullable<DefectItem["latest_verification"]> = {
      verifier_id: "v-1",
      fix_version: "1.0.0",
      method: null,
      result: "passed",
      evidence_id: null,
      created_at: "2026-11-25T00:00:00Z",
    };
    expect(verificationState(undefined)).toBe("unavailable");
    expect(verificationState(null)).toBe("unsubmitted");
    expect(verificationState({ ...verification, result: "passed" })).toBe("passed");
    expect(verificationState({ ...verification, result: "failed" })).toBe("failed");

    expect(renderVerificationText(undefined)).toBe("（未接入）");
    expect(renderVerificationText(null)).toBe("（未提交）");
    expect(renderVerificationText({ ...verification, fix_version: "2.0.0" })).toBe("已验证（2.0.0）");
    expect(
      renderVerificationText({ ...verification, result: "failed", fix_version: "1.5.0" }),
    ).toBe("验证失败（1.5.0）");
  });
});
