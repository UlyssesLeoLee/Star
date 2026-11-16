/*
@cypher schema=1 source_sha256=9b00b1b4f03df1c383a1dcc7df07e20ffebc9e5cfd86fd70cca1938558fb6a57
MERGE (self:File {path:"frontend/src/components/settings/McpSettingsPanel.tsx"})
MERGE (mcpSettingsPanel:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpSettingsPanel",kind:"component"})
MERGE (mcpServerFormDialog:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpServerFormDialog",kind:"component"})
MERGE (readMcpServers:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::readMcpServers",kind:"function"})
MERGE (writeMcpServers:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::writeMcpServers",kind:"function"})
MERGE (validateMcpServerUrl:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::validateMcpServerUrl",kind:"function"})
MERGE (testMcpConnection:Symbol {id:"frontend/src/components/settings/McpSettingsPanel.tsx::testMcpConnection",kind:"function"})
MERGE (mcpServer:Type {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpServer"})
MERGE (mcpServerInput:Type {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpServerInput"})
MERGE (mcpConnectionTestResult:Type {id:"frontend/src/components/settings/McpSettingsPanel.tsx::McpConnectionTestResult"})
MERGE (statusNotice:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::StatusNotice",kind:"component"})
MERGE (react:ExternalService {name:"react"})
MERGE (lucide:ExternalService {name:"lucide-react"})
MERGE (localStorage:ExternalService {name:"localStorage"})
MERGE (self)-[:DEFINES]->(mcpSettingsPanel)
MERGE (self)-[:DEFINES]->(mcpServerFormDialog)
MERGE (self)-[:DEFINES]->(readMcpServers)
MERGE (self)-[:DEFINES]->(writeMcpServers)
MERGE (self)-[:DEFINES]->(validateMcpServerUrl)
MERGE (self)-[:DEFINES]->(testMcpConnection)
MERGE (self)-[:DEFINES]->(mcpServer)
MERGE (self)-[:DEFINES]->(mcpServerInput)
MERGE (self)-[:DEFINES]->(mcpConnectionTestResult)
MERGE (self)-[:IMPORTS]->(react)
MERGE (self)-[:IMPORTS]->(lucide)
MERGE (self)-[:IMPORTS]->(statusNotice)
MERGE (readMcpServers)-[:READS]->(localStorage)
MERGE (writeMcpServers)-[:WRITES]->(localStorage)
MERGE (mcpSettingsPanel)-[:CALLS]->(readMcpServers)
MERGE (mcpSettingsPanel)-[:CALLS]->(writeMcpServers)
MERGE (mcpSettingsPanel)-[:CALLS]->(statusNotice)
MERGE (mcpSettingsPanel)-[:CALLS]->(testMcpConnection)
MERGE (mcpSettingsPanel)-[:CALLS]->(mcpServerFormDialog)
@endcypher
*/
"use client";

// =====================================================================
// McpSettingsPanel — 设置 › MCP 服务器管理面板 (本地可用版)
// 诚实原则: MCP 注册表 API 未接入 → 服务器配置本地持久化到 localStorage
//   (key: star-mcp-servers) 并显式标注。
// ① 列表渲染 readMcpServers(); ② 新增/编辑弹窗 (URL 格式校验,
//    authType 选 bearer/api_key 时显示对应认证值字段);
// ③ 测试连接: fetch HEAD (405/网络错误回退 GET), 超时默认 5s,
//    成功绿标 + 延迟, 失败红标 + 错误; stdio 传输无 HTTP 端点禁用并说明;
// ④ 启用/禁用 toggle + window.confirm 删除确认;
// ⑤ 空态/加载/错误三态复用 StatusNotice;
// ⑥ 底部诚实徽标 '本地持久化 — MCP 注册表 API 接入后同步';
// ⑦ 导出 readMcpServers / writeMcpServers / testMcpConnection /
//    validateMcpServerUrl 供测试与宿主页复用。
// =====================================================================
import { useEffect, useState } from "react";
import { clsx } from "clsx";
import {
  CheckCircle2,
  Loader2,
  Pencil,
  Plus,
  Server,
  Trash2,
  X,
  XCircle,
  Zap,
} from "lucide-react";

import { StatusNotice } from "../status/StatusNotice";

export const MCP_STORAGE_KEY = "star-mcp-servers";
export const DEFAULT_TIMEOUT_MS = 5000;

export type McpAuthType = "bearer" | "api_key" | "none";
export type McpTransport = "stdio" | "http";

export interface McpServer {
  id: string;
  name: string;
  url: string;
  authType: McpAuthType;
  authValue?: string;
  enabled: boolean;
  transport: McpTransport;
  timeoutMs: number;
  createdAt: string;
}

/** 新增/编辑表单提交载荷 (id/enabled/createdAt 由面板补全) */
export type McpServerInput = {
  name: string;
  url: string;
  transport: McpTransport;
  authType: McpAuthType;
  authValue?: string;
  timeoutMs: number;
};

export interface McpConnectionTestResult {
  ok: boolean;
  latencyMs?: number;
  status?: number;
  error?: string;
}

const AUTH_LABEL: Record<McpAuthType, string> = {
  bearer: "Bearer",
  api_key: "API Key",
  none: "无认证",
};

// ── localStorage helpers ─────────────────────────────────────────────

function isMcpServerShape(v: unknown): v is McpServer {
  if (typeof v !== "object" || v === null) return false;
  const s = v as Record<string, unknown>;
  return (
    typeof s.id === "string" &&
    typeof s.name === "string" &&
    typeof s.url === "string" &&
    (s.authType === "bearer" || s.authType === "api_key" || s.authType === "none") &&
    (s.authValue === undefined || typeof s.authValue === "string") &&
    typeof s.enabled === "boolean" &&
    (s.transport === "stdio" || s.transport === "http") &&
    typeof s.timeoutMs === "number" &&
    typeof s.createdAt === "string"
  );
}

/**
 * 读取本地 MCP 服务器列表。localStorage 不可用 (隐私模式) 时 getItem
 * 会抛异常 — 由调用方 (面板) 捕获进入错误态; 数据缺失/损坏返回 []。
 */
export function readMcpServers(): McpServer[] {
  if (typeof window === "undefined") return [];
  const raw = window.localStorage.getItem(MCP_STORAGE_KEY);
  if (raw == null) return [];
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return [];
  }
  if (!Array.isArray(parsed)) return [];
  return parsed.filter(isMcpServerShape);
}

/** 写入本地 MCP 服务器列表; localStorage 不可用返回 false。 */
export function writeMcpServers(servers: McpServer[]): boolean {
  if (typeof window === "undefined") return false;
  try {
    window.localStorage.setItem(MCP_STORAGE_KEY, JSON.stringify(servers));
    return true;
  } catch {
    return false;
  }
}

/** URL 格式校验; 合法返回 null, 否则返回中文错误信息。stdio 仅要求非空。 */
export function validateMcpServerUrl(url: string, transport: McpTransport): string | null {
  const trimmed = url.trim();
  if (!trimmed) return "请输入 URL";
  if (transport === "stdio") return null; // 本地命令字符串, 不做 URL 校验
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return "URL 格式无效 — 需以 http:// 或 https:// 开头";
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return "仅支持 http:// 或 https:// 协议";
  }
  if (!parsed.hostname) return "URL 缺少主机名";
  return null;
}

function describeFetchError(err: unknown): string {
  if (err instanceof TypeError) return `网络错误: ${err.message}`;
  if (err instanceof Error) return err.message;
  return String(err);
}

/**
 * 测试 MCP 服务器连接: 发 HTTP HEAD (405/网络错误回退 GET), 带认证头,
 * timeoutMs 超时中止。stdio 传输无 HTTP 端点, 返回失败说明。
 */
export async function testMcpConnection(server: McpServer): Promise<McpConnectionTestResult> {
  if (server.transport !== "http") {
    return { ok: false, error: "stdio 传输无 HTTP 端点 — 测试连接仅支持 http 传输" };
  }
  const headers: Record<string, string> = { Accept: "*/*" };
  if (server.authType === "bearer" && server.authValue) {
    headers.Authorization = `Bearer ${server.authValue}`;
  }
  if (server.authType === "api_key" && server.authValue) {
    headers["X-API-Key"] = server.authValue;
  }
  const timeoutMs = server.timeoutMs > 0 ? server.timeoutMs : DEFAULT_TIMEOUT_MS;
  const started = performance.now();
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    let res: Response;
    try {
      res = await fetch(server.url, { method: "HEAD", headers, signal: ctrl.signal });
      // 部分服务器不支持 HEAD → 回退 GET
      if (res.status === 405) {
        res = await fetch(server.url, { method: "GET", headers, signal: ctrl.signal });
      }
    } catch (err) {
      if (ctrl.signal.aborted) throw err;
      // 网络错误也回退 GET 一次 (HEAD 被中间层拒绝的常见形态)
      res = await fetch(server.url, { method: "GET", headers, signal: ctrl.signal });
    }
    const latencyMs = Math.round(performance.now() - started);
    if (res.status < 400) {
      return { ok: true, latencyMs, status: res.status };
    }
    return {
      ok: false,
      latencyMs,
      status: res.status,
      error: `HTTP ${res.status}${res.statusText ? ` ${res.statusText}` : ""}`,
    };
  } catch (err) {
    const latencyMs = Math.round(performance.now() - started);
    if (ctrl.signal.aborted) {
      return { ok: false, latencyMs, error: `连接超时 (${timeoutMs} ms)` };
    }
    return { ok: false, latencyMs, error: describeFetchError(err) };
  } finally {
    window.clearTimeout(timer);
  }
}

function newServerId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `mcpsrv-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

// ── 新增/编辑弹窗 ────────────────────────────────────────────────────

type DialogErrors = { name?: string; url?: string; authValue?: string; timeout?: string };

function McpServerFormDialog({
  server,
  onCancel,
  onSave,
}: {
  server?: McpServer;
  onCancel: () => void;
  onSave: (input: McpServerInput) => void;
}) {
  const [name, setName] = useState(server?.name ?? "");
  const [url, setUrl] = useState(server?.url ?? "");
  const [transport, setTransport] = useState<McpTransport>(server?.transport ?? "http");
  const [authType, setAuthType] = useState<McpAuthType>(server?.authType ?? "none");
  const [authValue, setAuthValue] = useState(server?.authValue ?? "");
  const [timeoutMs, setTimeoutMs] = useState(String(server?.timeoutMs ?? DEFAULT_TIMEOUT_MS));
  const [errors, setErrors] = useState<DialogErrors>({});

  const handleSubmit = () => {
    const next: DialogErrors = {};
    const trimmedName = name.trim();
    if (!trimmedName) next.name = "请输入服务器名称";
    const urlError = validateMcpServerUrl(url, transport);
    if (urlError) next.url = urlError;
    if (authType !== "none" && !authValue.trim()) {
      next.authValue = authType === "bearer" ? "请输入 Bearer Token" : "请输入 API Key";
    }
    const parsedTimeout = Number(timeoutMs);
    if (!Number.isFinite(parsedTimeout) || parsedTimeout < 1000 || parsedTimeout > 60000) {
      next.timeout = "超时需为 1000–60000 之间的数值 (毫秒)";
    }
    setErrors(next);
    if (Object.keys(next).length > 0) return;
    onSave({
      name: trimmedName,
      url: url.trim(),
      transport,
      authType,
      authValue: authType === "none" ? undefined : authValue.trim(),
      timeoutMs: Math.round(parsedTimeout),
    });
  };

  const inputCls =
    "rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent";

  return (
    <div
      data-testid="mcp-form-dialog"
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center p-3"
    >
      <div
        data-testid="mcp-form-backdrop"
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onCancel}
      />
      <div className="popover-solid zoom-in-95 relative z-10 w-full max-w-md rounded-xl border-2 border-line p-4 shadow-soft">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-ink">{server ? "编辑 MCP 服务器" : "新增 MCP 服务器"}</h3>
          <button
            type="button"
            data-testid="mcp-form-cancel"
            onClick={onCancel}
            aria-label="关闭"
            className="rounded p-1 text-ink-mute hover:bg-bg-soft"
          >
            <X size={14} />
          </button>
        </div>

        <div className="mt-3 flex flex-col gap-2.5">
          <label className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-ink">名称</span>
            <input
              data-testid="mcp-form-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如: GitHub MCP"
              className={inputCls}
            />
            {errors.name && (
              <span data-testid="mcp-form-error-name" className="text-[10px] text-err">
                {errors.name}
              </span>
            )}
          </label>

          <label className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-ink">传输方式</span>
            <select
              data-testid="mcp-form-transport"
              value={transport}
              onChange={(e) => setTransport(e.target.value as McpTransport)}
              className={inputCls}
            >
              <option value="http">HTTP (远程 URL)</option>
              <option value="stdio">stdio (本地命令)</option>
            </select>
          </label>

          <label className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-ink">
              {transport === "http" ? "服务器 URL" : "命令 (stdio)"}
            </span>
            <input
              data-testid="mcp-form-url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder={
                transport === "http" ? "https://mcp.example.com" : "npx -y @modelcontextprotocol/server-filesystem"
              }
              className={inputCls}
            />
            {errors.url && (
              <span data-testid="mcp-form-error-url" className="text-[10px] text-err">
                {errors.url}
              </span>
            )}
          </label>

          <label className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-ink">认证方式</span>
            <select
              data-testid="mcp-form-auth-type"
              value={authType}
              onChange={(e) => setAuthType(e.target.value as McpAuthType)}
              className={inputCls}
            >
              <option value="none">无认证</option>
              <option value="bearer">Bearer Token</option>
              <option value="api_key">API Key</option>
            </select>
          </label>

          {authType !== "none" && (
            <label className="flex flex-col gap-1">
              <span className="text-[11px] font-semibold text-ink">
                {authType === "bearer" ? "Bearer Token" : "API Key"}
              </span>
              <input
                data-testid="mcp-form-auth-value"
                type="password"
                value={authValue}
                onChange={(e) => setAuthValue(e.target.value)}
                placeholder={authType === "bearer" ? "sk-…" : "key-…"}
                className={inputCls}
              />
              {errors.authValue && (
                <span data-testid="mcp-form-error-auth" className="text-[10px] text-err">
                  {errors.authValue}
                </span>
              )}
              <span className="text-[9px] text-ink-mute">仅本地持久化, 测试连接时通过请求头发送</span>
            </label>
          )}

          <label className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-ink">连接超时 (毫秒)</span>
            <input
              data-testid="mcp-form-timeout"
              type="number"
              min={1000}
              max={60000}
              step={500}
              value={timeoutMs}
              onChange={(e) => setTimeoutMs(e.target.value)}
              className={inputCls}
            />
            {errors.timeout && (
              <span data-testid="mcp-form-error-timeout" className="text-[10px] text-err">
                {errors.timeout}
              </span>
            )}
          </label>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-xs text-ink-mute hover:text-ink"
          >
            取消
          </button>
          <button
            type="button"
            data-testid="mcp-form-submit"
            onClick={handleSubmit}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white hover:bg-accent-soft"
          >
            {server ? "保存" : "新增"}
          </button>
        </div>
      </div>
    </div>
  );
}

// ── 面板 ─────────────────────────────────────────────────────────────

type TestSlot = { running: boolean; result?: McpConnectionTestResult };
type PanelPhase = "loading" | "ready" | "error";

export function McpSettingsPanel() {
  const [phase, setPhase] = useState<PanelPhase>("loading");
  const [servers, setServers] = useState<McpServer[]>([]);
  const [storageError, setStorageError] = useState("");
  const [testSlots, setTestSlots] = useState<Record<string, TestSlot>>({});
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<McpServer | null>(null);

  useEffect(() => {
    let cancelled = false;
    const timer = window.setTimeout(() => {
      if (cancelled) return;
      try {
        setServers(readMcpServers());
        setPhase("ready");
      } catch (err) {
        setStorageError(err instanceof Error ? err.message : String(err));
        setPhase("error");
      }
    }, 0);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, []);

  const persist = (next: McpServer[]) => {
    setServers(next);
    setStorageError(writeMcpServers(next) ? "" : "本地存储不可用 — 修改未持久化, 刷新页面后丢失");
  };

  const handleSave = (input: McpServerInput) => {
    if (editing) {
      persist(servers.map((s) => (s.id === editing.id ? { ...s, ...input } : s)));
    } else {
      persist([
        ...servers,
        { id: newServerId(), createdAt: new Date().toISOString(), enabled: true, ...input },
      ]);
    }
    setDialogOpen(false);
    setEditing(null);
  };

  const handleDelete = (server: McpServer) => {
    if (!window.confirm(`删除 MCP 服务器「${server.name}」? 此操作不可撤销。`)) return;
    persist(servers.filter((s) => s.id !== server.id));
  };

  const handleToggle = (server: McpServer) => {
    persist(servers.map((s) => (s.id === server.id ? { ...s, enabled: !s.enabled } : s)));
  };

  const handleTest = async (server: McpServer) => {
    setTestSlots((prev) => ({ ...prev, [server.id]: { running: true } }));
    const result = await testMcpConnection(server);
    setTestSlots((prev) => ({ ...prev, [server.id]: { running: false, result } }));
  };

  const openCreate = () => {
    setEditing(null);
    setDialogOpen(true);
  };

  const openEdit = (server: McpServer) => {
    setEditing(server);
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditing(null);
  };

  return (
    <div data-testid="mcp-servers-panel" className="flex flex-col gap-3">
      {/* 头部: 标题 + 新增按钮 */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="min-w-0">
          <h2 className="text-base font-bold text-ink">MCP 服务器</h2>
          <p className="mt-0.5 text-[11px] text-ink-mute">
            管理本地 MCP 服务器配置 — 新增、编辑、测试连接、启用/禁用
          </p>
        </div>
        <button
          type="button"
          data-testid="mcp-add-server"
          onClick={openCreate}
          className="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white hover:bg-accent-soft"
        >
          <Plus size={13} /> 新增
        </button>
      </div>

      {/* ⑤ 加载/错误/空三态 (复用 StatusNotice) */}
      {phase === "loading" && (
        <StatusNotice state="loading" title="加载 MCP 服务器配置" hint="正在从本地存储读取已保存的服务器列表…" />
      )}
      {phase === "error" && (
        <StatusNotice
          state="failed"
          title="读取本地存储失败"
          hint={storageError || "无法读取本地存储, 请检查浏览器隐私设置后重试。"}
        />
      )}
      {phase === "ready" && servers.length === 0 && (
        <StatusNotice
          state="empty"
          title="暂无 MCP 服务器 — 点击新增添加"
          hint="点击右上角「新增」按钮, 添加第一个 MCP 服务器并测试连接。"
        />
      )}

      {/* ① 服务器列表 */}
      {phase === "ready" && servers.length > 0 && (
        <ul data-testid="mcp-servers-list" className="flex flex-col gap-2">
          {servers.map((server) => {
            const slot = testSlots[server.id];
            const result = slot?.result;
            const canTestHttp = server.transport === "http";
            return (
              <li
                key={server.id}
                data-testid={`mcp-server-item-${server.id}`}
                className="rounded-xl border border-line bg-bg-card px-3.5 py-3"
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="grid size-8 shrink-0 place-items-center rounded-lg bg-accent/15 text-accent">
                    <Server size={14} />
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-xs font-semibold text-ink">{server.name}</p>
                    <p className="truncate font-mono text-[10px] text-ink-mute">{server.url}</p>
                  </div>
                  <span className="rounded border border-line bg-bg-soft px-1.5 py-0.5 font-mono text-[9px] font-semibold text-ink-mute">
                    {server.transport.toUpperCase()}
                  </span>
                  <span className="rounded border border-line bg-bg-soft px-1.5 py-0.5 font-mono text-[9px] text-ink-mute">
                    {AUTH_LABEL[server.authType]}
                  </span>
                  <StatusNotice
                    compact
                    state={server.enabled ? "success" : "disconnected"}
                    title={server.enabled ? "已启用" : "已禁用"}
                  />
                  <div className="flex shrink-0 items-center gap-1.5">
                    <button
                      type="button"
                      data-testid={`mcp-action-test-${server.id}`}
                      onClick={() => void handleTest(server)}
                      disabled={!canTestHttp || slot?.running}
                      className={clsx(
                        "inline-flex items-center gap-1 rounded-lg border px-2 py-1 text-[10px] font-semibold",
                        canTestHttp && !slot?.running
                          ? "border-accent/50 bg-accent/10 text-accent hover:bg-accent/20"
                          : "cursor-not-allowed border-line bg-bg-soft/60 text-ink-mute",
                      )}
                    >
                      {slot?.running ? <Loader2 size={11} className="animate-spin" /> : <Zap size={11} />}
                      测试连接
                    </button>
                    <button
                      type="button"
                      data-testid={`mcp-action-edit-${server.id}`}
                      onClick={() => openEdit(server)}
                      className="inline-flex items-center gap-1 rounded-lg border border-line bg-bg-soft/60 px-2 py-1 text-[10px] font-semibold text-ink-mute hover:text-ink"
                    >
                      <Pencil size={11} /> 编辑
                    </button>
                    <button
                      type="button"
                      role="switch"
                      aria-checked={server.enabled}
                      aria-label="启用/禁用"
                      data-testid={`mcp-action-toggle-${server.id}`}
                      onClick={() => handleToggle(server)}
                      className={clsx(
                        "relative h-5 w-9 shrink-0 rounded-full transition-colors",
                        server.enabled ? "bg-accent" : "bg-bg-soft",
                      )}
                    >
                      <span
                        className={clsx(
                          "absolute top-0.5 size-4 rounded-full bg-white shadow transition-all",
                          server.enabled ? "left-[18px]" : "left-0.5",
                        )}
                      />
                    </button>
                    <button
                      type="button"
                      data-testid={`mcp-action-delete-${server.id}`}
                      onClick={() => handleDelete(server)}
                      className="inline-flex items-center gap-1 rounded-lg border border-err/40 bg-err/5 px-2 py-1 text-[10px] font-semibold text-err hover:bg-err/15"
                    >
                      <Trash2 size={11} /> 删除
                    </button>
                  </div>
                </div>

                {/* ③ 测试连接结果 */}
                {slot?.running && (
                  <p
                    data-testid={`mcp-test-result-${server.id}`}
                    className="mt-2 flex items-center gap-1.5 text-[11px] text-ink-mute"
                  >
                    <Loader2 size={11} className="animate-spin" /> 测试连接中…
                  </p>
                )}
                {result && !slot?.running && (
                  <p
                    data-testid={`mcp-test-result-${server.id}`}
                    className={clsx(
                      "mt-2 flex items-center gap-1.5 text-[11px] font-medium",
                      result.ok ? "text-ok" : "text-err",
                    )}
                  >
                    {result.ok ? (
                      <>
                        <CheckCircle2 size={12} /> 连接成功 · {result.latencyMs ?? 0} ms
                        {result.status != null ? ` · HTTP ${result.status}` : ""}
                      </>
                    ) : (
                      <>
                        <XCircle size={12} /> 连接失败 · {result.error}
                      </>
                    )}
                  </p>
                )}
                {!canTestHttp && !slot?.running && (
                  <p className="mt-2 text-[10px] text-ink-mute">
                    stdio 传输无 HTTP 端点 — 测试连接仅支持 http 传输的服务器
                  </p>
                )}
              </li>
            );
          })}
        </ul>
      )}

      {/* 保存失败提示 (localStorage 不可写) */}
      {phase === "ready" && storageError && <StatusNotice compact state="failed" title={storageError} />}

      {/* ⑥ 底部诚实徽标 */}
      <div
        data-testid="mcp-honest-badge"
        className="flex w-fit items-center rounded-full border border-warn/40 bg-warn/10 px-2.5 py-1 text-[10px] font-semibold text-warn"
      >
        本地持久化 — MCP 注册表 API 接入后同步
      </div>

      {/* ② 新增/编辑弹窗 */}
      {dialogOpen && <McpServerFormDialog server={editing ?? undefined} onCancel={closeDialog} onSave={handleSave} />}
    </div>
  );
}
