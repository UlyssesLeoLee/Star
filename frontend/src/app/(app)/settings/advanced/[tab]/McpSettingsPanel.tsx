/*
@cypher schema=1 source_sha256=e38a18e49f63695ae032f4d295bae37c952fd57e66d6a2ba74c58daf59099976
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx::McpSettingsPanel",kind:"component"})
MERGE (registry:File {path:"frontend/src/lib/settings/capabilityRegistry.ts"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:IMPORTS]->(registry)
MERGE (panel)-[:CALLS]->(registry)
@endcypher
*/
"use client";

// =====================================================================
// McpSettingsPanel — MCP 服务器管理 (per WBS §23.33-B, 2026-11-13 用户发令)
// 类似 Claude Code 的 MCP 管理: 连接类型 stdio/sse/http, 作用域 user/project,
// 添加/移除/启用禁用 + 健康状态徽章。
// =====================================================================

import { useMemo, useState } from "react";
import { Plug, Plus, Trash2, Power, Globe, Terminal, Radio, AlertTriangle, CircleCheck, CircleDashed } from "lucide-react";
import { clsx } from "clsx";
import {
  capabilityRegistryApi,
  useCapabilityRegistry,
  type CapabilityHealth,
  type CapabilityScope,
  type McpServerConfig,
  type McpTransport,
} from "@/lib/settings/capabilityRegistry";
import { useTranslation } from "@/lib/i18n";

const TRANSPORT_META: Record<McpTransport, { icon: typeof Terminal; label: string }> = {
  stdio: { icon: Terminal, label: "stdio" },
  sse: { icon: Radio, label: "sse" },
  http: { icon: Globe, label: "http" },
};

export function McpSettingsPanel() {
  const { t, tx } = useTranslation();
  const registry = useCapabilityRegistry();
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState("");
  const [transport, setTransport] = useState<McpTransport>("stdio");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [scope, setScope] = useState<CapabilityScope>("user");
  const [formError, setFormError] = useState<string | null>(null);

  const servers = registry.mcpServers;
  const enabledCount = useMemo(() => servers.filter((s) => s.enabled).length, [servers]);

  function submitAdd() {
    const trimmed = name.trim();
    if (!trimmed) {
      setFormError(t.settingsCapabilities.mcp.nameRequired);
      return;
    }
    if (!command.trim()) {
      setFormError(t.settingsCapabilities.mcp.commandRequired);
      return;
    }
    capabilityRegistryApi.addMcpServer({
      name: trimmed,
      transport,
      command: command.trim(),
      args: args.split(/\s+/).filter(Boolean),
      env: {},
      scope,
      enabled: true,
    });
    setName("");
    setCommand("");
    setArgs("");
    setFormError(null);
    setFormOpen(false);
  }

  return (
    <div className="space-y-4" data-testid="advanced-settings-mcp-panel">
      <div className="card p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold text-ink flex items-center gap-2">
              <Plug size={16} className="text-accent" /> {t.settingsCapabilities.mcp.title}
            </h2>
            <p className="mt-1 text-sm text-ink-dim">{t.settingsCapabilities.mcp.description}</p>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-[11px] font-mono text-ink-mute">
              {tx(t.settingsCapabilities.mcp.enabledCount, { enabled: enabledCount, total: servers.length })}
            </span>
            <button
              type="button"
              data-testid="mcp-add-server"
              onClick={() => setFormOpen((v) => !v)}
              className="btn text-xs"
            >
              <Plus size={12} /> {t.settingsCapabilities.mcp.addServer}
            </button>
          </div>
        </div>
      </div>

      {formOpen && (
        <form
          data-testid="mcp-add-form"
          onSubmit={(e) => { e.preventDefault(); submitAdd(); }}
          className="card p-4 grid gap-3 md:grid-cols-2"
        >
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.settingsCapabilities.mcp.nameLabel}
            <input
              data-testid="mcp-form-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t.settingsCapabilities.mcp.namePlaceholder}
              className="rounded border border-line bg-bg px-3 py-2 text-sm"
            />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.settingsCapabilities.mcp.transportLabel}
            <select
              data-testid="mcp-form-transport"
              value={transport}
              onChange={(e) => setTransport(e.target.value as McpTransport)}
              className="rounded border border-line bg-bg px-3 py-2 text-sm"
            >
              <option value="stdio">stdio — {t.settingsCapabilities.mcp.transportStdio}</option>
              <option value="sse">sse — {t.settingsCapabilities.mcp.transportSse}</option>
              <option value="http">http — {t.settingsCapabilities.mcp.transportHttp}</option>
            </select>
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">
            {tx(t.settingsCapabilities.mcp.commandLabel, { transport })}
            <input
              data-testid="mcp-form-command"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              placeholder={transport === "stdio" ? t.settingsCapabilities.mcp.commandPlaceholderStdio : t.settingsCapabilities.mcp.urlPlaceholder}
              className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
            />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.settingsCapabilities.mcp.argsLabel}
            <input
              data-testid="mcp-form-args"
              value={args}
              onChange={(e) => setArgs(e.target.value)}
              placeholder={t.settingsCapabilities.mcp.argsPlaceholder}
              className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
            />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.settingsCapabilities.mcp.scopeLabel}
            <select
              data-testid="mcp-form-scope"
              value={scope}
              onChange={(e) => setScope(e.target.value as CapabilityScope)}
              className="rounded border border-line bg-bg px-3 py-2 text-sm"
            >
              <option value="user">{t.settingsCapabilities.mcp.scopeUser}</option>
              <option value="project">{t.settingsCapabilities.mcp.scopeProject}</option>
            </select>
          </label>
          {formError && <p className="text-xs text-warn md:col-span-2" role="alert" data-testid="mcp-form-error">{formError}</p>}
          <div className="flex gap-2 md:col-span-2">
            <button type="submit" data-testid="mcp-form-submit" className="btn text-xs">
              <Plus size={12} /> {t.settingsCapabilities.mcp.addButton}
            </button>
            <button type="button" onClick={() => setFormOpen(false)} className="btn-ghost text-xs">
              {t.settingsCapabilities.mcp.cancel}
            </button>
          </div>
        </form>
      )}

      {servers.length === 0 ? (
        <p className="text-sm text-ink-mute" data-testid="mcp-empty">{t.settingsCapabilities.mcp.empty}</p>
      ) : (
        <ul className="space-y-2" data-testid="mcp-server-list">
          {servers.map((server) => (
            <McpServerRow key={server.id} server={server} />
          ))}
        </ul>
      )}
    </div>
  );
}

function healthIcon(health: CapabilityHealth) {
  switch (health) {
    case "healthy": return <CircleCheck size={13} className="text-ok" aria-label="healthy" />;
    case "error": return <AlertTriangle size={13} className="text-warn" aria-label="error" />;
    case "connecting": return <CircleDashed size={13} className="text-accent animate-spin" aria-label="connecting" />;
    default: return <CircleDashed size={13} className="text-ink-mute" aria-label="unknown" />;
  }
}

function McpServerRow({ server }: { server: McpServerConfig }) {
  const { t, tx } = useTranslation();
  const TransportIcon = TRANSPORT_META[server.transport].icon;
  return (
    <li
      data-testid={`mcp-server-${server.id}`}
      className={clsx(
        "card p-3 flex flex-wrap items-center gap-3",
        !server.enabled && "opacity-60",
      )}
    >
      <span className="grid size-8 shrink-0 place-items-center rounded border border-line bg-bg-soft text-accent">
        <TransportIcon size={14} />
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-semibold text-ink font-mono">{server.name}</span>
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-line text-ink-mute">
            {TRANSPORT_META[server.transport].label}
          </span>
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-accent/40 bg-accent/10 text-accent">
            {server.scope === "user" ? t.settingsCapabilities.mcp.scopeUserShort : t.settingsCapabilities.mcp.scopeProjectShort}
          </span>
        </div>
        <p className="mt-0.5 truncate text-[11px] font-mono text-ink-mute">
          {server.command} {server.args.join(" ")}
        </p>
        {server.last_error && (
          <p className="mt-0.5 text-[11px] text-warn font-mono" role="alert">
            {tx(t.settingsCapabilities.mcp.lastError, { error: server.last_error })}
          </p>
        )}
      </div>
      <span title={t.settingsCapabilities.mcp.healthLabel} aria-label={`health ${server.health}`}>
        {healthIcon(server.health)}
      </span>
      <button
        type="button"
        data-testid={`mcp-toggle-${server.id}`}
        onClick={() => capabilityRegistryApi.toggleMcpServer(server.id)}
        aria-pressed={server.enabled}
        title={server.enabled ? t.settingsCapabilities.mcp.disable : t.settingsCapabilities.mcp.enable}
        className={clsx(
          "rounded border px-2 py-1 text-[10px] font-mono flex items-center gap-1 transition-colors",
          server.enabled ? "border-ok/50 bg-ok/10 text-ok" : "border-line text-ink-mute hover:text-ink",
        )}
      >
        <Power size={11} />
        {server.enabled ? t.settingsCapabilities.mcp.on : t.settingsCapabilities.mcp.off}
      </button>
      <button
        type="button"
        data-testid={`mcp-remove-${server.id}`}
        onClick={() => capabilityRegistryApi.removeMcpServer(server.id)}
        aria-label={tx(t.settingsCapabilities.mcp.removeAria, { name: server.name })}
        className="rounded border border-line p-1.5 text-ink-mute hover:text-danger hover:border-danger/50 transition-colors"
      >
        <Trash2 size={13} />
      </button>
    </li>
  );
}
