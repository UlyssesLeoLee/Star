/*
@cypher schema=1 source_sha256=fe1ae84cf8f7a9f6f3050ac4a90d0a88f05a2e445ca393d1685ab339d8bb96f2
MERGE (self:File {path:"frontend/src/components/settings/McpSettingsPanel.test.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpSettingsPanel",kind:"component"})
MERGE (readMcpServers:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::readMcpServers",kind:"function"})
MERGE (writeMcpServers:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::writeMcpServers",kind:"function"})
MERGE (validateMcpServerUrl:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::validateMcpServerUrl",kind:"function"})
MERGE (testMcpConnection:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::testMcpConnection",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::McpSettingsPanel MCP 服务器管理面板",kind:"suite"})
MERGE (t1:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::helpers readMcpServers and writeMcpServers roundtrip through localStorage",kind:"test"})
MERGE (t2:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::corrupt localStorage payload falls back to empty list",kind:"test"})
MERGE (t3:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::renders empty state text and honest badge",kind:"test"})
MERGE (t4:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::shows loading state first then settles to ready",kind:"test"})
MERGE (t5:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::lists persisted servers with names urls and action testids",kind:"test"})
MERGE (t6:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::shows failed notice when localStorage read throws",kind:"test"})
MERGE (t7:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::adds a server through the dialog and persists it",kind:"test"})
MERGE (t8:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::validates name url and auth value on submit",kind:"test"})
MERGE (t9:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::shows auth value field only for bearer or api_key auth",kind:"test"})
MERGE (t10:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::edits a server with prefilled dialog and persists changes",kind:"test"})
MERGE (t11:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::toggle disables and re-enables a server",kind:"test"})
MERGE (t12:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::delete with confirm accepted removes the server",kind:"test"})
MERGE (t13:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::delete with confirm declined keeps the server",kind:"test"})
MERGE (t14:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::test connection success shows latency and sends bearer header",kind:"test"})
MERGE (t15:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::test connection falls back to GET when HEAD gets 405",kind:"test"})
MERGE (t16:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::test connection failure shows red error",kind:"test"})
MERGE (t17:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::test button disabled for stdio servers",kind:"test"})
MERGE (t18:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.test.tsx::validateMcpServerUrl unit cases",kind:"test"})
MERGE (self)-[:IMPORTS]->(panel)
MERGE (self)-[:IMPORTS]->(readMcpServers)
MERGE (self)-[:IMPORTS]->(writeMcpServers)
MERGE (self)-[:IMPORTS]->(validateMcpServerUrl)
MERGE (self)-[:IMPORTS]->(testMcpConnection)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(t1)
MERGE (suite)-[:DEFINES]->(t2)
MERGE (suite)-[:DEFINES]->(t3)
MERGE (suite)-[:DEFINES]->(t4)
MERGE (suite)-[:DEFINES]->(t5)
MERGE (suite)-[:DEFINES]->(t6)
MERGE (suite)-[:DEFINES]->(t7)
MERGE (suite)-[:DEFINES]->(t8)
MERGE (suite)-[:DEFINES]->(t9)
MERGE (suite)-[:DEFINES]->(t10)
MERGE (suite)-[:DEFINES]->(t11)
MERGE (suite)-[:DEFINES]->(t12)
MERGE (suite)-[:DEFINES]->(t13)
MERGE (suite)-[:DEFINES]->(t14)
MERGE (suite)-[:DEFINES]->(t15)
MERGE (suite)-[:DEFINES]->(t16)
MERGE (suite)-[:DEFINES]->(t17)
MERGE (suite)-[:DEFINES]->(t18)
MERGE (t3)-[:TESTS]->(panel)
MERGE (t4)-[:TESTS]->(panel)
MERGE (t5)-[:TESTS]->(panel)
MERGE (t6)-[:TESTS]->(panel)
MERGE (t7)-[:TESTS]->(panel)
MERGE (t8)-[:TESTS]->(panel)
MERGE (t9)-[:TESTS]->(panel)
MERGE (t10)-[:TESTS]->(panel)
MERGE (t11)-[:TESTS]->(panel)
MERGE (t12)-[:TESTS]->(panel)
MERGE (t13)-[:TESTS]->(panel)
MERGE (t14)-[:TESTS]->(panel)
MERGE (t15)-[:TESTS]->(panel)
MERGE (t16)-[:TESTS]->(panel)
MERGE (t17)-[:TESTS]->(panel)
MERGE (t1)-[:TESTS]->(readMcpServers)
MERGE (t1)-[:TESTS]->(writeMcpServers)
MERGE (t2)-[:TESTS]->(readMcpServers)
MERGE (t18)-[:TESTS]->(validateMcpServerUrl)
@endcypher
*/
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import {
  DEFAULT_TIMEOUT_MS,
  MCP_STORAGE_KEY,
  McpSettingsPanel,
  readMcpServers,
  testMcpConnection,
  validateMcpServerUrl,
  writeMcpServers,
  type McpServer,
} from "./McpSettingsPanel";

function makeServer(overrides: Partial<McpServer> = {}): McpServer {
  return {
    id: "srv-1",
    name: "GitHub MCP",
    url: "https://mcp.github.example.com",
    authType: "none",
    enabled: true,
    transport: "http",
    timeoutMs: 5000,
    createdAt: "2026-11-16T00:00:00.000Z",
    ...overrides,
  };
}

function seedServers(servers: McpServer[]) {
  window.localStorage.setItem(MCP_STORAGE_KEY, JSON.stringify(servers));
}

function fetchResponse(init: Partial<Response> = {}): Response {
  return { ok: true, status: 200, statusText: "OK", ...init } as Response;
}

function stubFetch(impl: (url: RequestInfo | URL, init?: RequestInit) => Promise<Response>) {
  const mock = vi.fn(impl);
  vi.stubGlobal("fetch", mock);
  return mock;
}

describe("McpSettingsPanel MCP 服务器管理面板", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("helpers readMcpServers and writeMcpServers roundtrip through localStorage", () => {
    expect(readMcpServers()).toEqual([]);
    const servers = [makeServer(), makeServer({ id: "srv-2", name: "Filesystem", transport: "stdio" })];
    expect(writeMcpServers(servers)).toBe(true);
    expect(window.localStorage.getItem(MCP_STORAGE_KEY)).toBe(JSON.stringify(servers));
    expect(readMcpServers()).toEqual(servers);
    expect(writeMcpServers([])).toBe(true);
    expect(readMcpServers()).toEqual([]);
  });

  it("corrupt localStorage payload falls back to empty list", () => {
    window.localStorage.setItem(MCP_STORAGE_KEY, "{not-json");
    expect(readMcpServers()).toEqual([]);
    window.localStorage.setItem(MCP_STORAGE_KEY, JSON.stringify([{ id: "x" }]));
    expect(readMcpServers()).toEqual([]); // 缺字段的对象被过滤
  });

  it("renders empty state text and honest badge", async () => {
    render(<McpSettingsPanel />);
    expect(screen.getByTestId("mcp-servers-panel")).toBeInTheDocument();
    expect(await screen.findByTestId("status-notice-empty")).toHaveTextContent("暂无 MCP 服务器 — 点击新增添加");
    expect(screen.getByTestId("mcp-honest-badge")).toHaveTextContent("本地持久化 — MCP 注册表 API 接入后同步");
  });

  it("shows loading state first then settles to ready", async () => {
    render(<McpSettingsPanel />);
    expect(screen.getByTestId("status-notice-loading")).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByTestId("status-notice-loading")).not.toBeInTheDocument());
    expect(screen.getByTestId("status-notice-empty")).toBeInTheDocument();
  });

  it("lists persisted servers with names urls and action testids", async () => {
    seedServers([makeServer(), makeServer({ id: "srv-2", name: "Slack", url: "https://mcp.slack.test", enabled: false })]);
    render(<McpSettingsPanel />);
    const row1 = await screen.findByTestId("mcp-server-item-srv-1");
    expect(row1).toHaveTextContent("GitHub MCP");
    expect(row1).toHaveTextContent("https://mcp.github.example.com");
    expect(screen.getByTestId("mcp-action-test-srv-1")).toBeEnabled();
    expect(screen.getByTestId("mcp-action-toggle-srv-1")).toHaveAttribute("aria-checked", "true");
    expect(screen.getByTestId("mcp-action-delete-srv-1")).toBeInTheDocument();
    const row2 = screen.getByTestId("mcp-server-item-srv-2");
    expect(row2).toHaveTextContent("Slack");
    expect(screen.getByTestId("mcp-action-toggle-srv-2")).toHaveAttribute("aria-checked", "false");
  });

  it("shows failed notice when localStorage read throws", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError: storage denied");
    });
    render(<McpSettingsPanel />);
    const failed = await screen.findByTestId("status-notice-failed");
    expect(failed).toHaveTextContent("读取本地存储失败");
    expect(failed).toHaveTextContent("SecurityError");
  });

  it("adds a server through the dialog and persists it", async () => {
    render(<McpSettingsPanel />);
    await screen.findByTestId("status-notice-empty");
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    expect(screen.getByTestId("mcp-form-dialog")).toBeInTheDocument();

    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "PostgreSQL" } });
    fireEvent.change(screen.getByTestId("mcp-form-url"), { target: { value: "https://pg.mcp.example.com" } });
    fireEvent.change(screen.getByTestId("mcp-form-auth-type"), { target: { value: "bearer" } });
    fireEvent.change(screen.getByTestId("mcp-form-auth-value"), { target: { value: "tok-123" } });
    fireEvent.change(screen.getByTestId("mcp-form-timeout"), { target: { value: "8000" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));

    await waitFor(() => expect(screen.queryByTestId("mcp-form-dialog")).not.toBeInTheDocument());
    const stored = readMcpServers();
    expect(stored).toHaveLength(1);
    expect(stored[0]).toMatchObject({
      name: "PostgreSQL",
      url: "https://pg.mcp.example.com",
      authType: "bearer",
      authValue: "tok-123",
      enabled: true,
      transport: "http",
      timeoutMs: 8000,
    });
    expect(stored[0].id).toBeTruthy();
    expect(stored[0].createdAt).toBeTruthy();
    expect(screen.getByTestId(`mcp-server-item-${stored[0].id}`)).toHaveTextContent("PostgreSQL");
  });

  it("validates name url and auth value on submit", async () => {
    render(<McpSettingsPanel />);
    await screen.findByTestId("status-notice-empty");
    fireEvent.click(screen.getByTestId("mcp-add-server"));

    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(screen.getByTestId("mcp-form-error-name")).toHaveTextContent("请输入服务器名称");
    expect(screen.getByTestId("mcp-form-error-url")).toHaveTextContent("请输入 URL");

    fireEvent.change(screen.getByTestId("mcp-form-url"), { target: { value: "not-a-url" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(screen.getByTestId("mcp-form-error-url")).toHaveTextContent("URL 格式无效");

    fireEvent.change(screen.getByTestId("mcp-form-url"), { target: { value: "ftp://files.example.com" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(screen.getByTestId("mcp-form-error-url")).toHaveTextContent("仅支持 http:// 或 https:// 协议");

    fireEvent.change(screen.getByTestId("mcp-form-url"), { target: { value: "https://ok.example.com" } });
    fireEvent.change(screen.getByTestId("mcp-form-auth-type"), { target: { value: "api_key" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(screen.getByTestId("mcp-form-error-auth")).toHaveTextContent("请输入 API Key");

    fireEvent.change(screen.getByTestId("mcp-form-auth-value"), { target: { value: "k-1" } });
    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "Validated" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    await waitFor(() => expect(screen.queryByTestId("mcp-form-dialog")).not.toBeInTheDocument());
    expect(readMcpServers()[0]).toMatchObject({ name: "Validated", authType: "api_key" });
  });

  it("shows auth value field only for bearer or api_key auth", async () => {
    render(<McpSettingsPanel />);
    await screen.findByTestId("status-notice-empty");
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    expect(screen.queryByTestId("mcp-form-auth-value")).not.toBeInTheDocument();

    fireEvent.change(screen.getByTestId("mcp-form-auth-type"), { target: { value: "bearer" } });
    expect(screen.getByTestId("mcp-form-auth-value")).toBeInTheDocument();

    fireEvent.change(screen.getByTestId("mcp-form-auth-type"), { target: { value: "api_key" } });
    expect(screen.getByTestId("mcp-form-auth-value")).toBeInTheDocument();

    fireEvent.change(screen.getByTestId("mcp-form-auth-type"), { target: { value: "none" } });
    expect(screen.queryByTestId("mcp-form-auth-value")).not.toBeInTheDocument();
  });

  it("edits a server with prefilled dialog and persists changes", async () => {
    seedServers([makeServer({ authType: "bearer", authValue: "tok-1" })]);
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-edit-srv-1"));

    expect(screen.getByTestId("mcp-form-name")).toHaveValue("GitHub MCP");
    expect(screen.getByTestId("mcp-form-url")).toHaveValue("https://mcp.github.example.com");
    expect(screen.getByTestId("mcp-form-auth-type")).toHaveValue("bearer");
    expect(screen.getByTestId("mcp-form-auth-value")).toHaveValue("tok-1");

    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "GitHub MCP v2" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));

    await waitFor(() => expect(screen.queryByTestId("mcp-form-dialog")).not.toBeInTheDocument());
    const stored = readMcpServers();
    expect(stored).toHaveLength(1);
    expect(stored[0]).toMatchObject({
      id: "srv-1",
      name: "GitHub MCP v2",
      url: "https://mcp.github.example.com",
      authType: "bearer",
      authValue: "tok-1",
      enabled: true,
      createdAt: "2026-11-16T00:00:00.000Z",
    });
    expect(screen.getByTestId("mcp-server-item-srv-1")).toHaveTextContent("GitHub MCP v2");
  });

  it("toggle disables and re-enables a server", async () => {
    seedServers([makeServer()]);
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-toggle-srv-1"));
    await waitFor(() => expect(screen.getByTestId("mcp-action-toggle-srv-1")).toHaveAttribute("aria-checked", "false"));
    expect(readMcpServers()[0].enabled).toBe(false);
    expect(screen.getByTestId("mcp-server-item-srv-1")).toHaveTextContent("已禁用");

    fireEvent.click(screen.getByTestId("mcp-action-toggle-srv-1"));
    await waitFor(() => expect(screen.getByTestId("mcp-action-toggle-srv-1")).toHaveAttribute("aria-checked", "true"));
    expect(readMcpServers()[0].enabled).toBe(true);
  });

  it("delete with confirm accepted removes the server", async () => {
    seedServers([makeServer(), makeServer({ id: "srv-2", name: "Slack" })]);
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-delete-srv-1"));
    expect(confirm).toHaveBeenCalledWith(expect.stringContaining("GitHub MCP"));
    await waitFor(() => expect(screen.queryByTestId("mcp-server-item-srv-1")).not.toBeInTheDocument());
    expect(screen.getByTestId("mcp-server-item-srv-2")).toBeInTheDocument();
    expect(readMcpServers().map((s) => s.id)).toEqual(["srv-2"]);
  });

  it("delete with confirm declined keeps the server", async () => {
    seedServers([makeServer()]);
    vi.spyOn(window, "confirm").mockReturnValue(false);
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-delete-srv-1"));
    expect(screen.getByTestId("mcp-server-item-srv-1")).toBeInTheDocument();
    expect(readMcpServers()).toHaveLength(1);
  });

  it("test connection success shows latency and sends bearer header", async () => {
    seedServers([makeServer({ authType: "bearer", authValue: "tok-123" })]);
    const fetchMock = stubFetch(() => Promise.resolve(fetchResponse({ status: 200 })));
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-test-srv-1"));

    await waitFor(() => expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("连接成功"));
    expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("ms");
    expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("HTTP 200");
    expect(fetchMock).toHaveBeenCalledWith(
      "https://mcp.github.example.com",
      expect.objectContaining({
        method: "HEAD",
        headers: expect.objectContaining({ Authorization: "Bearer tok-123" }),
      }),
    );
  });

  it("test connection falls back to GET when HEAD gets 405", async () => {
    seedServers([makeServer()]);
    const fetchMock = stubFetch((_url, init) => {
      const method = init?.method ?? "GET";
      return Promise.resolve(method === "HEAD" ? fetchResponse({ status: 405, ok: false }) : fetchResponse({ status: 200 }));
    });
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-test-srv-1"));

    await waitFor(() => expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("连接成功"));
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock).toHaveBeenNthCalledWith(2, "https://mcp.github.example.com", expect.objectContaining({ method: "GET" }));
  });

  it("test connection failure shows red error", async () => {
    seedServers([makeServer()]);
    stubFetch(() => Promise.reject(new TypeError("Failed to fetch")));
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-1");
    fireEvent.click(screen.getByTestId("mcp-action-test-srv-1"));

    await waitFor(() => expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("连接失败"));
    expect(screen.getByTestId("mcp-test-result-srv-1")).toHaveTextContent("Failed to fetch");
  });

  it("test button disabled for stdio servers", async () => {
    seedServers([makeServer({ id: "srv-3", transport: "stdio", url: "npx -y @modelcontextprotocol/server-filesystem" })]);
    render(<McpSettingsPanel />);
    await screen.findByTestId("mcp-server-item-srv-3");
    expect(screen.getByTestId("mcp-action-test-srv-3")).toBeDisabled();
    expect(screen.getByTestId("mcp-server-item-srv-3")).toHaveTextContent("stdio 传输无 HTTP 端点");
  });

  it("validateMcpServerUrl unit cases", () => {
    expect(validateMcpServerUrl("", "http")).toBe("请输入 URL");
    expect(validateMcpServerUrl("not-a-url", "http")).toMatch(/URL 格式无效/);
    expect(validateMcpServerUrl("ftp://x.example.com", "http")).toBe("仅支持 http:// 或 https:// 协议");
    expect(validateMcpServerUrl("https://ok.example.com", "http")).toBeNull();
    expect(validateMcpServerUrl("http://localhost:8080/mcp", "http")).toBeNull();
    expect(validateMcpServerUrl("", "stdio")).toBe("请输入 URL");
    expect(validateMcpServerUrl("npx -y some-server", "stdio")).toBeNull();
  });

  it("testMcpConnection reports timeout on abort", async () => {
    stubFetch(
      (_url, init) =>
        new Promise((_resolve, reject) => {
          const signal = init?.signal;
          const onAbort = () => reject(new DOMException("aborted", "AbortError"));
          if (signal?.aborted) {
            onAbort();
            return;
          }
          signal?.addEventListener("abort", onAbort);
        }),
    );
    const result = await testMcpConnection(makeServer({ timeoutMs: 50 }));
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(/连接超时/);
  });

  it("testMcpConnection refuses stdio transport", async () => {
    const result = await testMcpConnection(makeServer({ transport: "stdio", url: "npx x" }));
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(/stdio 传输无 HTTP 端点/);
  });

  it("testMcpConnection applies api_key header", async () => {
    const fetchMock = stubFetch(() => Promise.resolve(fetchResponse({ status: 200 })));
    const result = await testMcpConnection(makeServer({ authType: "api_key", authValue: "k-9" }));
    expect(result.ok).toBe(true);
    expect(fetchMock).toHaveBeenCalledWith(
      "https://mcp.github.example.com",
      expect.objectContaining({ headers: expect.objectContaining({ "X-API-Key": "k-9" }) }),
    );
  });

  it("DEFAULT_TIMEOUT_MS is 5000", () => {
    expect(DEFAULT_TIMEOUT_MS).toBe(5000);
  });
});
