/*
@cypher schema=1 source_sha256=0bbe4b39279775a20f2465d5029646f6bf31d2b23145d31983ffce1185f9dfd8
MERGE (self:File {path:"frontend/src/app/(app)/settings/developer/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/settings/developer/page.tsx::DeveloperSettingsPage",kind:"component"})
MERGE (webhooks:Symbol {id:"frontend/src/app/(app)/settings/developer/page.tsx::WebhooksTab",kind:"component"})
MERGE (api:Symbol {id:"frontend/src/app/(app)/settings/developer/page.tsx::ApiTab",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:DEFINES]->(webhooks)
MERGE (self)-[:DEFINES]->(api)
@endcypher
*/
"use client";

// =====================================================================
// /settings/developer — Developer API + 出站 Webhook 管理
// per 2026-11-12 用户发令 (导航完善): registry 里 developer-console
// (/settings/developer) 与 webhooks (/settings/developer?tab=webhooks)
// 此前指向不存在的路由 → 404。本页把两个入口整合为一个双 tab 页面,
// webhook CRUD 走 localStorage, API 参考走静态展示 (后端接入见已知缺口)。
// =====================================================================

import { Suspense, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "next/navigation";
import { Plus, Trash2, Zap, Webhook, Key, Code2, RefreshCw } from "lucide-react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";

export interface OutboundWebhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  active: boolean;
  created_at: string;
}

const STORAGE_KEY = "star.outbound-webhooks.v1";

function loadWebhooks(): OutboundWebhook[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as OutboundWebhook[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveWebhooks(list: OutboundWebhook[]) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
  } catch {
    /* quota / privacy mode */
  }
}

function newId() {
  return `whk-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

export default function DeveloperSettingsPage() {
  // per 2026-11-12: useSearchParams 必须包 Suspense (Next 16 静态 prerender bailout)
  return (
    <Suspense fallback={null}>
      <DeveloperSettingsInner />
    </Suspense>
  );
}

function DeveloperSettingsInner() {
  const searchParams = useSearchParams();
  const initialTab = searchParams?.get("tab") === "webhooks" ? "webhooks" : "api";
  const [tab, setTab] = useState<"api" | "webhooks">(initialTab);

  return (
    <div className="max-w-5xl mx-auto">
      <PageHeader
        title="Developer Console"
        subtitle="Developer API (REST + MCP) 参考与出站 Webhook 管理 (per 2026-09-02 拍板)"
        icon={<Code2 className="text-accent" size={20} />}
        track="D"
        count={tab === "webhooks" ? "webhooks" : "api"}
      />

      <div className="flex gap-2 mb-5" data-testid="developer-tabs">
        <button
          data-testid="developer-tab-api"
          data-active={tab === "api" ? "true" : "false"}
          onClick={() => setTab("api")}
          className={`px-3 py-1.5 text-xs font-mono border rounded ${
            tab === "api" ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim hover:text-ink"
          }`}
        >
          <Key size={11} className="inline mr-1" /> Developer API
        </button>
        <button
          data-testid="developer-tab-webhooks"
          data-active={tab === "webhooks" ? "true" : "false"}
          onClick={() => setTab("webhooks")}
          className={`px-3 py-1.5 text-xs font-mono border rounded ${
            tab === "webhooks" ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim hover:text-ink"
          }`}
        >
          <Webhook size={11} className="inline mr-1" /> Outbound Webhooks
        </button>
      </div>

      {tab === "api" ? <ApiTab /> : <WebhooksTab />}
    </div>
  );
}

// =====================================================================
// ApiTab — REST / MCP / JWT 参考 (只读展示, 后端接入见已知缺口)
// =====================================================================
function ApiTab() {
  const endpoints = useMemo(
    () => [
      { method: "GET", path: "/api/v1/health", note: "就绪探针" },
      { method: "GET", path: "/api/v1/workspace-files", note: "本地文件夹桥接 (白名单只读)" },
      { method: "GET", path: "/api/v1/projects", note: "项目列表" },
      { method: "POST", path: "/api/v1/auth/google", note: "Google OIDC 登录" },
      { method: "GET", path: "/api/v1/admin/status", note: "管理员状态 (lidian727@gmail.com)" },
    ],
    []
  );
  return (
    <div className="card" data-testid="developer-api-tab">
      <SectionTitle>Developer API 参考</SectionTitle>
      <p className="text-xs text-ink-dim mb-3">
        REST 基址 /api/v1 · MCP 经同一网关暴露 · JWT 由 Google OIDC 签发 (RS256)。
      </p>
      <table className="table text-xs" data-testid="developer-api-table">
        <thead>
          <tr>
            <th className="text-left">Method</th>
            <th className="text-left">Path</th>
            <th className="text-left">说明</th>
          </tr>
        </thead>
        <tbody>
          {endpoints.map((e) => (
            <tr key={`${e.method}${e.path}`}>
              <td>
                <span className="font-mono text-[10px] px-1.5 py-0.5 rounded border border-accent/40 text-accent">{e.method}</span>
              </td>
              <td className="font-mono text-[11px] text-ink">{e.path}</td>
              <td className="text-ink-dim">{e.note}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="text-[10px] text-ink-mute font-mono mt-3">
        已知缺口: OpenAPI 3.1 spec 自动生成 (v0.2 utoipa) / MCP endpoint 鉴权目录 UI。
      </div>
    </div>
  );
}

// =====================================================================
// WebhooksTab — 出站 Webhook CRUD (localStorage), 5 域预置事件模板
// =====================================================================
const EVENT_OPTIONS = [
  "work_item.created",
  "work_item.status_changed",
  "worktree.state_changed",
  "feedback.created",
  "agent.session_failed",
  "project.member_joined",
  "notification.mentioned",
];

function WebhooksTab() {
  const [webhooks, setWebhooks] = useState<OutboundWebhook[]>([]);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [events, setEvents] = useState<string[]>([EVENT_OPTIONS[0]]);
  const [notice, setNotice] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, string>>({});

  useEffect(() => {
    setWebhooks(loadWebhooks());
  }, []);

  const persist = (next: OutboundWebhook[]) => {
    setWebhooks(next);
    saveWebhooks(next);
  };

  const addWebhook = () => {
    if (!name.trim() || !url.trim()) {
      setNotice("name / url 必填");
      return;
    }
    const item: OutboundWebhook = {
      id: newId(),
      name: name.trim(),
      url: url.trim(),
      events,
      active: true,
      created_at: new Date().toISOString(),
    };
    persist([item, ...webhooks]);
    setName("");
    setUrl("");
    setEvents([EVENT_OPTIONS[0]]);
    setNotice("✓ Webhook 已创建 (本地存储)");
  };

  const removeWebhook = (id: string) => {
    persist(webhooks.filter((w) => w.id !== id));
  };

  const toggleWebhook = (id: string) => {
    persist(webhooks.map((w) => (w.id === id ? { ...w, active: !w.active } : w)));
  };

  const testWebhook = (id: string) => {
    // 本地 demo: 不做真实网络投递 (CORS/凭据安全), 只回显 payload 草稿
    const w = webhooks.find((x) => x.id === id);
    if (!w) return;
    setTestResult((r) => ({
      ...r,
      [id]: `demo payload → ${w.url} (events: ${w.events.join(", ")}) — 真实投递待后端出站 Webhook 接入`,
    }));
  };

  return (
    <div className="card" data-testid="developer-webhooks-tab">
      <SectionTitle>出站 Webhook ({webhooks.length})</SectionTitle>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mt-3">
        <input
          data-testid="webhook-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="名称 (e.g. Feishu 通知)"
          className="input text-xs"
        />
        <input
          data-testid="webhook-url"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="https://…"
          className="input text-xs"
        />
        <button data-testid="webhook-add" className="btn text-xs" onClick={addWebhook}>
          <Plus size={12} /> 添加 Webhook
        </button>
      </div>

      <div className="flex flex-wrap gap-1.5 mt-2" data-testid="webhook-events">
        {EVENT_OPTIONS.map((ev) => {
          const on = events.includes(ev);
          return (
            <button
              key={ev}
              data-testid={`webhook-event-${ev}`}
              data-active={on ? "true" : "false"}
              onClick={() => setEvents((cur) => (on ? cur.filter((x) => x !== ev) : [...cur, ev]))}
              className={`text-[10px] font-mono px-2 py-0.5 rounded border ${
                on ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-mute hover:text-ink"
              }`}
            >
              {ev}
            </button>
          );
        })}
      </div>

      {notice && (
        <div data-testid="webhook-notice" className="text-xs font-mono mt-2 text-ok">
          {notice}
        </div>
      )}

      <ul className="divide-y divide-line/50 mt-3" data-testid="webhook-list">
        {webhooks.map((w) => (
          <li key={w.id} data-testid={`webhook-row-${w.id}`} className="py-2 flex items-center gap-2 flex-wrap">
            <button
              data-testid={`webhook-toggle-${w.id}`}
              onClick={() => toggleWebhook(w.id)}
              title={w.active ? "停用" : "启用"}
              className={`size-5 grid place-items-center rounded border ${
                w.active ? "border-ok bg-ok/15 text-ok" : "border-line text-ink-mute"
              }`}
            >
              <Zap size={10} />
            </button>
            <div className="min-w-0 flex-1">
              <div className="text-xs text-ink font-semibold truncate">{w.name}</div>
              <div className="text-[10px] font-mono text-ink-mute truncate">{w.url}</div>
              <div className="flex flex-wrap gap-1 mt-0.5">
                {w.events.map((ev) => (
                  <span key={ev} className="text-[9px] font-mono px-1 py-0.5 rounded bg-bg-soft border border-line/50 text-ink-dim">
                    {ev}
                  </span>
                ))}
              </div>
            </div>
            <button data-testid={`webhook-test-${w.id}`} className="btn-ghost p-1 rounded" onClick={() => testWebhook(w.id)} title="测试">
              <RefreshCw size={11} />
            </button>
            <button data-testid={`webhook-delete-${w.id}`} className="btn-ghost p-1 rounded text-err" onClick={() => removeWebhook(w.id)} title="删除">
              <Trash2 size={11} />
            </button>
            {testResult[w.id] && (
              <div data-testid={`webhook-test-result-${w.id}`} className="w-full text-[10px] font-mono text-ink-dim pl-6">
                {testResult[w.id]}
              </div>
            )}
          </li>
        ))}
        {webhooks.length === 0 && (
          <li className="py-6 text-center text-xs text-ink-mute">
            暂无 Webhook。添加一个出站 Webhook (本地存储, 后端投递接入见已知缺口)。
          </li>
        )}
      </ul>
    </div>
  );
}
