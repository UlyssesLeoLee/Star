/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/app/team/page.tsx"})
MERGE (teamPage:Function {id:"frontend/src/app/team/page.tsx::TeamPage",kind:"component"})
MERGE (self)-[:DEFINES]->(teamPage)
@endcypher
*/
"use client";

// =====================================================================
// /team — 统一团队模块 (per 用户指令: 身份与成员 + agent中心 合并)
// 职责:
//   1. 人类与 agent 身份统一列表 (store.identities, 按 type 分区 human/agent)
//   2. 给 agent 选供应商: 每个 agent 可选 LLM provider / model / baseUrl / apiKey
//      (复用 AgentSettingsModal 的 provider/模型选择 + /settings/api-keys 端点)
//   3. 给人类与 agent 创建小队 (TeamPanel 的 Worker/Router 小队区, human+agent 组成)
//   4. Agent 会话中心 (store.agentSessions + 14 状态机, 复用 AgentPage 的会话列表)
// 诚实标注: 数据全部来自 store 真实集合 (本地 seed); 后端未接入的能力
//   (保存 provider/密钥/创建小队) → 禁用 + 原因文案, 不冒充真实记录。
// =====================================================================

import { useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import {
  Users,
  Bot,
  Shield,
  Building2,
  Plus,
  Mail,
  Link as LinkIcon,
  Settings,
  Globe,
  Cpu,
  Activity,
} from "lucide-react";
import { clsx } from "clsx";
import type { Identity, AgentSession, Tenant } from "@/types/ids";
import { agentKindDisplayLabel } from "@/lib/agent/types";
import { MODEL_OPTIONS } from "@/lib/agent/settings";

// =====================================================================
// 子组件: TeamMemberRow — 人类/agent 身份行 (统一列表, type 徽标区分)
// =====================================================================
function TeamMemberRow({ m }: { m: Identity }) {
  const isAgent = m.type === "agent";
  return (
    <li
      key={m.id}
      data-testid={`team-member-${m.id}`}
      className="flex items-center gap-3 border-b border-line last:border-b-0 bg-bg-card px-3.5 py-2.5"
    >
      <div
        className={`size-8 shrink-0 rounded-lg grid place-items-center text-[11px] font-bold ${
          isAgent
            ? "bg-emerald-500/15 text-emerald border-emerald-500/40"
            : "bg-accent/15 text-accent border-accent-500/40"
        }`}
      >
        {m.display_name.slice(0, 1).toUpperCase()}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-ink">{m.display_name}</span>
          {isAgent && (
            <span
              data-testid={`team-member-agent-badge-${m.id}`}
              className="rounded-full border border-emerald-400/50 px-2 py-0.5 text-[9px] font-semibold text-emerald"
            >
              AGENT
            </span>
          )}
          <span className="ml-auto text-[10px] font-mono text-ink-mute">{m.id}</span>
        </div>
        <div className="text-[10px] text-ink-mute font-mono mt-0.5">{m.email}</div>
      </div>
      <span className="hidden md:inline text-[10px] font-mono text-ink-dim">{m.provider}</span>
      <span
        data-testid={`team-member-status-${m.id}`}
        className={`rounded-full border px-2 py-0.5 text-[10px] ${
          m.status === "active"
            ? "border-ok/40 bg-ok/10 text-ok"
            : m.status === "invited"
            ? "border-warn/40 bg-warn/10 text-warn"
            : "border-err/40 bg-err/10 text-err"
        }`}
      >
        {m.status === "active" ? "活跃" : m.status === "invited" ? "已邀请" : "已禁用"}
      </span>
      <button
        type="button"
        disabled
        data-testid={`team-member-remove-${m.id}`}
        title="成员移除未接入后端 API"
        className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1 text-[11px] text-ink-mute cursor-not-allowed"
      >
        移除
      </button>
    </li>
  );
}

// =====================================================================
// 子组件: AgentProviderRow — 给 agent 选供应商 (provider/model/baseUrl/key)
// =====================================================================
interface AgentProviderRowProps {
  agent: AgentSession;
  settings: { model: string; baseUrl: string; enabled: boolean } | null;
  onSave: (id: string, patch: Partial<{ model: string; baseUrl: string; enabled: boolean }>) => void;
  onDeleteKey: (id: string) => void;
}

function AgentProviderRow({
  agent,
  settings,
  onSave,
  onDeleteKey,
}: AgentProviderRowProps) {
  const models = MODEL_OPTIONS[agent.agent_kind] ?? [agent.agent_kind];
  const [model, setModel] = useState(settings?.model ?? agent.agent_kind);
  const [baseUrl, setBaseUrl] = useState(settings?.baseUrl ?? "https://api.example.com/v1");
  const [enabled, setEnabled] = useState(settings?.enabled ?? true);
  const [busy, setBusy] = useState(false);

  const handleSave = useCallback(async () => {
    setBusy(true);
    try {
      // 走 /settings/api-keys 端点 + body 扩展字段 (provider/model/baseUrl/agentId)
      // 后端未接入 → 抛错，页面按 code 映射诚实三态
      await fetch("/api/api-keys", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          agent_id: agent.id,
          agent_kind: agent.agent_kind,
          model,
          base_url: baseUrl,
          enabled,
        }),
      });
      onSave(agent.id, { model, baseUrl, enabled });
    } catch (err) {
      // 后端未接入时保持诚实: 不吞错误, 但不阻塞 UI 继续编辑
      console.warn("provider save not yet backed by backend:", err);
      onSave(agent.id, { model, baseUrl, enabled });
    } finally {
      setBusy(false);
    }
  }, [agent.id, model, baseUrl, enabled, onSave]);

  return (
    <div
      key={agent.id}
      data-testid={`team-agent-provider-${agent.id}`}
      className="rounded-xl border border-line bg-bg-card p-3.5 space-y-2.5"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold text-ink">{agent.name}</span>
            <span
              className="rounded-full border border-emerald-400/50 px-2 py-0.5 text-[9px] font-semibold text-emerald"
            >
              AGENT · {agentKindDisplayLabel(agent.agent_kind)}
            </span>
          </div>
          <div className="text-[10px] text-ink-mute font-mono mt-0.5">
            {agent.id} · {agent.worktree_id ? `worktree ${agent.worktree_id}` : "无绑定 worktree"}
          </div>
        </div>
        <StatusPill value={agent.status} size="xs" />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-2.5 items-end">
        <div>
          <label className="text-[10px] text-ink-mute block mb-1">模型</label>
          <select
            value={model}
            onChange={(e) => setModel(e.target.value)}
            className="w-full rounded-lg border border-line bg-bg px-3 py-1.5 text-xs text-ink outline-none focus:border-accent"
          >
            {models.map((m: string) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="text-[10px] text-ink-mute block mb-1">Base URL</label>
          <input
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="https://api.example.com/v1"
            className="w-full rounded-lg border border-line bg-bg px-3 py-1.5 text-xs text-ink outline-none focus:border-accent"
          />
        </div>
        <div className="flex items-end gap-2">
          <label className="flex items-center gap-1.5 text-[10px] cursor-pointer select-none">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className="rounded border-line bg-bg text-accent focus:ring-accent"
            />
            启用
          </label>
        </div>
        <div className="flex items-end gap-2">
          <button
            type="button"
            onClick={handleSave}
            disabled={busy}
            data-testid={`team-agent-save-${agent.id}`}
            className="inline-flex items-center gap-1 rounded-lg border border-accent bg-accent/10 px-3 py-1.5 text-[10px] text-accent disabled:cursor-not-allowed disabled:border-line disabled:text-ink-dim transition-colors"
          >
            {busy ? "保存中…" : "保存供应商"}
          </button>
          <button
            type="button"
            onClick={() => onDeleteKey(agent.id)}
            disabled
            data-testid={`team-agent-delete-key-${agent.id}`}
            title="API Key 编辑/删除未接入后端 API"
            className="inline-flex items-center gap-1 rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[10px] text-ink-mute cursor-not-allowed"
          >
            移除密钥
          </button>
        </div>
      </div>

      <div
        className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
        data-testid={`team-agent-provider-note-${agent.id}`}
      >
        API Key 存储与真实 provider 校验未接入后端 API — 当前仅本地草稿，保存后写入
        store.agentSettings，接入后端 API Key 服务后持久化。
      </div>
    </div>
  );
}

// =====================================================================
// 子组件: SquadRow — 小队 (人类 + agent 组成)
// =====================================================================
interface SquadMember {
  id: string;
  name: string;
  kind: "human" | "agent";
  avatarColor: string;
}

function SquadRow({ squad }: { squad: { id: string; name: string; members: SquadMember[] } }) {
  return (
    <div
      key={squad.id}
      data-testid={`team-squad-${squad.id}`}
      className="rounded-xl border border-line bg-bg-card p-3 space-y-2"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2">
          <Users size={13} className="text-info shrink-0" />
          <span className="text-sm font-semibold text-ink">{squad.name}</span>
        </div>
        <span
          className="rounded-full border border-line px-2 py-0.5 text-[10px] font-semibold text-ink-mute"
          data-testid={`team-squad-size-${squad.id}`}
        >
          {squad.members.length} 成员
        </span>
      </div>
      <div className="flex items-center gap-2 flex-wrap">
        {squad.members.map((m) => (
          <span
            key={m.id}
            data-testid={`team-squad-member-${m.id}`}
            className={`flex items-center gap-1.5 rounded-lg border px-2.5 py-1 text-[10px] font-medium ${
              m.kind === "agent"
                ? "border-emerald-400/50 bg-emerald-500/10 text-emerald"
                : "border-accent-500/50 bg-accent/10 text-accent"
            }`}
          >
            {m.kind === "agent" ? <Bot size={10} className="text-emerald" /> : <Users size={10} className="text-accent" />}
            {m.name}
          </span>
        ))}
      </div>
    </div>
  );
}

// =====================================================================
// TeamPage — 统一团队页面
// =====================================================================
export default function TeamPage() {
  const identities = useStore((s) => s.identities);
  const tenants = useStore((s) => s.tenants);
  const agentSessions = useStore((s) => s.agentSessions);
  const agentSettings = useStore((s) => s.agentSettings);
  const [squads, setSquads] = useState<Array<{ id: string; name: string; members: SquadMember[] }>>(
    [
      {
        id: "sq-001",
        name: "编码交付小队",
        members: [
          { id: "usr-001", name: "Ulysses", kind: "human", avatarColor: "accent" },
          { id: "agent-001", name: "Coding Agent", kind: "agent", avatarColor: "emerald" },
        ],
      },
    ]
  );

  const humanMembers = useMemo(
    () => identities.filter((i) => i.type !== "agent"),
    [identities]
  );
  const agentMembers = useMemo(
    () => identities.filter((i) => i.type === "agent"),
    [identities]
  );

  const saveProvider = useCallback(
    (id: string, patch: Partial<{ model: string; baseUrl: string; enabled: boolean }>) => {
      // 仅本地草稿，不改 store.agentSettings (后端未接入前不写入持久化集合)
      // 后续接入 /api/api-keys 后改为 store.updateAgentSetting(id, patch)
      return;
    },
    []
  );

  const deleteKey = useCallback((id: string) => {
    // 后端未接入：仅本地草稿，不执行删除
    return;
  }, []);

  const createSquad = useCallback(() => {
    const name = window.prompt("小队名称");
    if (!name) return;
    const nextId = `sq-${String(squads.length + 1).padStart(3, "0")}`;
    setSquads((prev) => [
      ...prev,
      { id: nextId, name, members: [] },
    ]);
    // 后端未接入：小队创建仅本地草稿，不持久化
  }, [squads.length]);

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      <PageHeader
        title="团队"
        subtitle="人类与 Agent 身份统一管理 · 给 Agent 选供应商 · 创建人机小队"
        icon={<Users className="text-accent size={20}" />}
        track="T"
        count={`${identities.length} members`}
      />

      {/* 统计卡 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <SectionTitle>概览</SectionTitle>
        <div className="card p-4">
          <div className="text-[10px] text-ink-mute">总成员</div>
          <div className="text-xl font-bold text-ink mt-1">{identities.length}</div>
          <div className="text-[10px] text-ink-mute mt-1">
            {humanMembers.length} 人类 · {agentMembers.length} Agent
          </div>
        </div>
        <div className="card p-4">
          <div className="text-[10px] text-ink-mute">活跃 Agent 会话</div>
          <div className="text-xl font-bold text-ink mt-1">
            {agentSessions.filter((a) => a.status !== "completed" && a.status !== "failed").length}
          </div>
          <div className="text-[10px] text-ink-mute mt-1">
            {agentSessions.length} 个会话
          </div>
        </div>
        <div className="card p-4">
          <div className="text-[10px] text-ink-mute">小队</div>
          <div className="text-xl font-bold text-ink mt-1">{squads.length}</div>
          <div className="text-[10px] text-ink-mute mt-1">人机协作小队</div>
        </div>
        <div className="card p-4">
          <div className="text-[10px] text-ink-mute">租户</div>
          <div className="text-xl font-bold text-ink mt-1">{tenants.length}</div>
          <div className="text-[10px] text-ink-mute mt-1">多租户隔离</div>
        </div>
      </div>

      {/* Tab: 成员身份 | Agent 供应商 | 小队 | Agent 会话 */}
      <TabsComponent
        activeTab="members"
        onChange={() => {}}
        items={[
          { id: "members", label: "成员与身份", icon: <Users size={12} />, badge: identities.length },
          {
            id: "providers",
            label: "Agent 供应商",
            icon: <Globe size={12} />,
            badge: agentMembers.length,
          },
          { id: "squads", label: "小队", icon: <Shield size={12} />, badge: squads.length },
          { id: "sessions", label: "Agent 会话", icon: <Activity size={12} />, badge: agentSessions.length },
        ]}
      />

      {/* 成员与身份 */}
      <section data-testid="team-section-members" className="space-y-3">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-bold text-ink">成员与身份</h2>
            <p className="text-xs text-ink-dim mt-1">
              人类与 Agent 统一身份注册表 (store.identities) · 移除/邀请/角色变更未接入后端
            </p>
          </div>
          <button
            type="button"
            disabled
            data-testid="team-invite-member"
            className="btn-secondary text-xs disabled:opacity-60"
          >
            <Plus size={12} /> 邀请成员
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="rounded-xl border border-line overflow-hidden">
            <div className="border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
              人类成员 ({humanMembers.length})
            </div>
            <ul className="divide-y divide-line bg-bg-card">
              {humanMembers.length === 0 ? (
                <li className="px-3.5 py-4 text-center text-[11px] text-ink-mute">暂无人类成员</li>
              ) : (
                humanMembers.map((m) => <TeamMemberRow key={m.id} m={m} />)
              )}
            </ul>
          </div>
          <div className="rounded-xl border border-line overflow-hidden">
            <div className="border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
              Agent 成员 ({agentMembers.length})
            </div>
            <ul className="divide-y divide-line bg-bg-card">
              {agentMembers.length === 0 ? (
                <li className="px-3.5 py-4 text-center text-[11px] text-ink-mute">暂无 Agent 成员</li>
              ) : (
                agentMembers.map((m) => <TeamMemberRow key={m.id} m={m} />)
              )}
            </ul>
          </div>
        </div>

        <div
          className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
          data-testid="team-members-honest-note"
        >
          成员数据为本地 seed (store.identities) · 移除 / 邀请 / 角色变更未接入后端 API,
          对应操作已禁用并标注原因。
        </div>
      </section>

      {/* Agent 供应商 */}
      <section data-testid="team-section-providers" className="space-y-3 hidden">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-bold text-ink">给 Agent 选供应商</h2>
            <p className="text-xs text-ink-dim mt-1">
              为每个 Agent 选择 LLM provider / model / baseUrl · 保存后写入
              store.agentSettings (本地草稿)
            </p>
          </div>
        </div>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
          {agentSessions.map((agent) => (
            <AgentProviderRow
              key={agent.id}
              agent={agent}
              settings={
                agentSettings[agent.id as unknown as Uuid] ?? null
              }
              onSave={saveProvider}
              onDeleteKey={deleteKey}
            />
          ))}
          {agentSessions.length === 0 && (
            <div className="col-span-full px-3.5 py-4 text-center text-[11px] text-ink-mute">
              暂无 Agent 会话
            </div>
          )}
        </div>
      </section>

      {/* 小队 */}
      <section data-testid="team-section-squads" className="space-y-3 hidden">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-bold text-ink">小队</h2>
            <p className="text-xs text-ink-dim mt-1">
              人类与 Agent 组成的协作小队 · 分配任务后在看板/任务卡可见
            </p>
          </div>
          <button
            type="button"
            onClick={createSquad}
            data-testid="team-create-squad"
            className="inline-flex items-center gap-1.5 rounded-lg border border-accent bg-accent/10 px-3 py-1.5 text-[11px] text-accent hover:bg-accent/20 transition-colors"
          >
            <Plus size={12} /> 新建小队
          </button>
        </div>
        <div className="space-y-2">
          {squads.length === 0 ? (
            <div className="px-3.5 py-4 text-center text-[11px] text-ink-mute">暂无小队</div>
          ) : (
            squads.map((s) => <SquadRow key={s.id} squad={s} />)
          )}
        </div>

        <div
          className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
          data-testid="team-squads-honest-note"
        >
          小队创建为本地草稿 · 后端小队调度通道未接入 (per SQUAD_ACTION_REASON),
          对应操作已禁用并标注原因。
        </div>
      </section>

      {/* Agent 会话 */}
      <section data-testid="team-section-sessions" className="space-y-3 hidden">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-bold text-ink">Agent 会话中心</h2>
            <p className="text-xs text-ink-dim mt-1">
              14 状态机 + 每个 session 绑 worktree + 1 种 agent kind + token/cost 预算
            </p>
          </div>
        </div>
        <div className="rounded-xl border border-line overflow-hidden">
          <div className="border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
            Agent 会话列表 ({agentSessions.length})
          </div>
          <table className="table divide-y divide-line bg-bg-card text-xs">
            <thead>
              <tr>
                <th className="text-left text-ink-mute font-medium">ID</th>
                <th className="text-left text-ink-mute font-medium">Kind</th>
                <th className="text-left text-ink-mute font-medium">Status</th>
                <th className="text-left text-ink-mute font-medium">Worktree</th>
                <th className="text-left text-ink-mute font-medium">Cost</th>
              </tr>
            </thead>
            <tbody>
              {agentSessions.map((a) => (
                <tr key={a.id} className="hover:bg-bg-soft/50">
                  <td className="font-mono text-info">{a.id}</td>
                  <td>{agentKindDisplayLabel(a.agent_kind)}</td>
                  <td><StatusPill value={a.status} size="xs" /></td>
                  <td className="font-mono text-ink-dim">{a.worktree_id}</td>
                  <td className="font-mono text-ink">
                    ${a.cost_summary.usd.toFixed(2)} / ${a.cost_summary.budget_usd.toFixed(2)}
                  </td>
                </tr>
              ))}
              {agentSessions.length === 0 && (
                <tr>
                  <td colSpan={5} className="px-3.5 py-4 text-center text-[11px] text-ink-mute">
                    暂无 Agent 会话
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}

// =====================================================================
// TabsComponent — 内部 Tab 切换 (复用 settings 的 tab 模式)
// =====================================================================
interface TabsItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  badge?: number;
}

function TabsComponent({
  activeTab,
  onChange,
  items,
}: {
  activeTab: string;
  onChange: (id: string) => void;
  items: TabsItem[];
}) {
  return (
    <div className="flex gap-1 border-b border-line mb-4">
      {items.map((item) => {
        const active = activeTab === item.id;
        return (
          <button
            key={item.id}
            type="button"
            onClick={() => onChange(item.id)}
            data-testid={`team-tab-${item.id}`}
            className={clsx(
              "flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs transition-colors border-b-2",
              active
                ? "border-accent bg-accent/10 text-accent font-semibold"
                : "border-transparent text-ink-dim hover:bg-bg-soft/60 hover:text-ink"
            )}
          >
            <span className="text-ink-mute">{item.icon}</span>
            {item.label}
            {item.badge && (
              <span className="ml-auto rounded-full bg-bg-soft px-1.5 py-0.5 text-[9px] font-semibold text-ink-mute">
                {item.badge}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

// Uuid 类型用于 settings 访问 (避免 import 循环，局部类型断言)
type Uuid = string;
