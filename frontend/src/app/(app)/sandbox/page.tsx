/*
@cypher schema=1 source_sha256=5da05aee701207cbd3ebd1182495ef42a8572bdf01c5151f608b26a2d1d73f05
MERGE (self:File {path:"frontend/src/app/(app)/sandbox/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/sandbox/page.tsx::SandboxPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /sandbox — 2c · 沙盒 (per 2026-11-14 参考 mockup: Sandbox.dc.html)
// 结构: 左 资源树(实体机/虚拟机/Docker + 观测方式) + 主区 沙盒详情
//      (启动/停止/快照/销毁 + 概览/Worktree/终端/Docker/观测 tabs)
//      + 开设 Worktree 表单 + 操作任务列表
// =====================================================================
"use client";

import { useState, useEffect } from "react";
import {
  Plus,
  Boxes,
  Monitor,
  Server,
  Container,
  Play,
  Square,
  Camera,
  Trash2,
  GitBranch,
  CheckCircle2,
  Loader2,
  XCircle,
  Clock,
  ShieldAlert,
  LogIn,
  RefreshCw,
  Terminal,
  ShieldCheck,
} from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";

const SANDBOX_TABS = ["概览", "Worktree", "终端", "Docker", "观测"] as const;

interface Resource {
  name: string;
  sub: string;
  badge: string;
  badgeTone: string;
  online?: boolean;
}

const RESOURCES: { group: string; hint?: string; items: Resource[] }[] = [
  {
    group: "实体机",
    items: [
      { name: "ws-office", sub: "实体机 · Controller", badge: "原生", badgeTone: "bg-ok/10 text-ok" },
      { name: "lab-01", sub: "实体机 · Worker · WireGuard", badge: "执行", badgeTone: "bg-accent/10 text-accent" },
      { name: "mbp-ulysses", sub: "实体机 · 离线 2 小时", badge: "元数据", badgeTone: "bg-bg-soft text-ink-dim" },
    ],
  },
  {
    group: "虚拟机",
    hint: "Multipass 最多保留 2 台",
    items: [
      { name: "star-k3s-1", sub: "Multipass · K3s 节点", badge: "执行", badgeTone: "bg-accent/10 text-accent", online: true },
      { name: "star-k3s-2", sub: "Multipass · 已停止", badge: "元数据", badgeTone: "bg-bg-soft text-ink-dim" },
      { name: "wsb-relation", sub: "Windows Sandbox", badge: "探针", badgeTone: "bg-[var(--color-accent-violet)]/15 text-[var(--color-accent-violet)]" },
    ],
  },
  {
    group: "容器 / 沙箱",
    hint: "Podman 沙箱隔离: 容器内 Worktree 与宿主完全隔离，创建后纳入左侧导航树管理",
    items: [
      { name: "podman", sub: "Podman 沙箱引擎 · 沙箱隔离工作区", badge: "沙箱", badgeTone: "bg-accent/10 text-accent" },
      { name: "docker", sub: "本机 Docker 守护进程 · 6 个容器", badge: "原生", badgeTone: "bg-ok/10 text-ok" },
      { name: "sbx-relation", sub: "Docker 沙盒 · 集成测试", badge: "探针", badgeTone: "bg-[var(--color-accent-violet)]/15 text-[var(--color-accent-violet)]" },
    ],
  },
];

// 沙箱资源树资源列表 (per 2026-11-22: 接入 store.worktrees 真实数据)
const WORKTREES_MOCK = [
  { wt: "wt-relation-bfs", branch: "feat/relation-bfs", path: "~/worktrees/wt-relation-bfs", status: "运行中", tone: "bg-ok/10 text-ok" },
  { wt: "wt-cache-ttl", branch: "fix/cache-ttl", path: "~/worktrees/wt-cache-ttl", status: "空闲", tone: "bg-bg-soft text-ink-dim" },
];

const OP_TASKS = [
  { title: "开设 Worktree", state: "完成", detail: "star-k3s-1 · wt-relation-bfs · 2 分钟前", icon: CheckCircle2, tone: "text-ok" },
  { title: "快照", state: "运行中", detail: "star-k3s-1 · 第 2/3 步 启动虚拟机", icon: Loader2, tone: "text-accent animate-spin" },
  { title: "重启容器", state: "失败", detail: "canvas-realtime · 健康检查超时", icon: XCircle, tone: "text-err" },
];

export default function SandboxPage() {
  // 后端门面数据 (per WBS-BACKEND-SUPPORT-001 W2: sandtree/argo API)
  const storeApi = useStore();
  const worktreeGroupApi = useWorktreeGroupApi();
  const worktrees = storeApi.worktrees;
  const [backendInfo, setBackendInfo] = useState<{
    argoCli?: string[];
    observationLevels?: string[];
  } | null>(null);
  useEffect(() => {
    let alive = true;
    Promise.allSettled([
      fetch("/api/v1/argo/nodes").then((r) => (r.ok ? r.json() : null)),
      fetch("/api/v1/sandtree/resources").then((r) => (r.ok ? r.json() : null)),
    ]).then(([argo, st]) => {
      if (!alive) return;
      const argoData = argo.status === "fulfilled" ? argo.value : null;
      const stData = st.status === "fulfilled" ? st.value : null;
      setBackendInfo({
        argoCli: argoData?.data?.supported_cli,
        observationLevels: stData?.data?.observation_levels,
      });
    }).catch(() => { /* 后端不可达: 保留本地 mock */ });
    return () => { alive = false; };
  }, []);
  const [activeTab, setActiveTab] = useState<(typeof SANDBOX_TABS)[number]>("Worktree");
  const [createWT, setCreateWT] = useState(false);
  const [wtName, setWtName] = useState("");
  const [wtBranch, setWtBranch] = useState("");
  const [wtBaseBranch, setWtBaseBranch] = useState("");
  const [wtHostPath, setWtHostPath] = useState("");
  const [wtBusy, setWtBusy] = useState(false);
  const [wtError, setWtError] = useState<string | null>(null);
  const [wtCreated, setWtCreated] = useState<string | null>(null);

  const handleCreateWT = async () => {
    if (!wtName.trim() || !wtBranch.trim()) return;
    setWtBusy(true); setWtError(null);
    try {
      const newId = storeApi.createWorktree({
        project_id: "physis-project-default",
        name: wtName.trim(),
        branch: wtBranch.trim(),
        base_branch: wtBaseBranch.trim() || "main",
        host_path: wtHostPath.trim() || undefined,
      });
      if (worktreeGroupApi) {
        await worktreeGroupApi.createProjectWorktree(
          "physis-project-default",
          { id: newId, name: wtName.trim(), branch: wtBranch.trim(), base_branch: wtBaseBranch.trim() || "main", status: "initializing", host_path: wtHostPath.trim() || undefined },
          `sandbox-wt-${Date.now()}`,
        );
      }
      setWtCreated(newId);
      setWtName(""); setWtBranch(""); setWtBaseBranch(""); setWtHostPath("");
    } catch (e) {
      setWtError(e instanceof Error ? e.message : String(e));
    } finally {
      setWtBusy(false);
    }
  };
  // 接入实体机表单 (per 2026-11-15: 2c mockup FR-SB-003, 后端未接入 → 本地登记 + 状态说明)
  const [attachOpen, setAttachOpen] = useState(false);
  const [attachForm, setAttachForm] = useState({ host: "", os: "", via: "SSH" });
  const [attachDone, setAttachDone] = useState(false);
  const [attachedHosts, setAttachedHosts] = useState<{ host: string; os: string; via: string; state: string }[]>([]);

  const submitAttach = () => {
    if (!attachForm.host.trim()) return;
    setAttachedHosts((prev) => [...prev, { ...attachForm, host: attachForm.host.trim(), state: "验证中" }]);
    setAttachForm({ host: "", os: "", via: "SSH" });
    setAttachOpen(false);
    setAttachDone(true);
    setTimeout(() => setAttachDone(false), 4000);
  };

  return (
    <div className="flex h-full bg-bg">
      {/* ============ 左: 资源树 ============ */}
      <aside data-testid="sandbox-resource-tree" className="w-72 shrink-0 border-r border-line bg-bg-card overflow-y-auto">
        <div className="px-3 py-3 border-b border-line flex items-center gap-2">
          <button type="button" data-testid="sandbox-new" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg bg-accent text-white hover:bg-accent-soft">
            <Plus size={12} /> 新建沙盒
          </button>
          <button type="button" data-testid="sandbox-attach-bm" onClick={() => setAttachOpen(true)} className="text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim hover:border-accent/60">
            接入实体机
          </button>
        </div>

        <div className="px-3 py-3 space-y-4">
          {attachedHosts.length > 0 && (
            <div data-testid="sandbox-attached-list">
              <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5">已接入实体机</div>
              {attachedHosts.map((h, i) => (
                <div key={h.host + i} className="rounded-lg border border-line bg-bg-soft/50 px-2.5 py-2 text-xs mb-1.5">
                  <div className="flex items-center gap-2">
                    <span className="size-1.5 rounded-full bg-warn" />
                    <span className="font-mono font-semibold text-ink flex-1 truncate">{h.host}</span>
                    <span className="text-[10px] text-warn">{h.state}</span>
                  </div>
                  <div className="text-[10px] text-ink-mute mt-0.5">{h.os || "系统待探测"} · {h.via}</div>
                </div>
              ))}
            </div>
          )}
          {RESOURCES.map((group) => (
            <div key={group.group}>
              <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5">
                {group.group}
                {group.hint && <span className="text-ink-mute normal-case tracking-normal ml-1">· {group.hint}</span>}
              </div>
              <ul className="space-y-1">
                {group.items.map((r) => (
                  <li
                    key={r.name}
                    data-testid={`sandbox-res-${r.name}`}
                    className={clsx(
                      "px-2.5 py-2 rounded-lg border text-xs",
                      r.online ? "border-accent/50 bg-accent/10" : "border-line hover:border-line",
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <span className="font-mono font-semibold text-ink flex-1 truncate">{r.name}</span>
                      <span className={clsx("text-[9px] px-1.5 py-0.5 rounded font-semibold", r.badgeTone)}>{r.badge}</span>
                      {r.online && <span className="size-1.5 rounded-full bg-ok" />}
                    </div>
                    <div className="text-[10px] text-ink-dim mt-0.5">{r.sub}</div>
                  </li>
                ))}
              </ul>
            </div>
          ))}

          <p data-testid="sandbox-observation-note" className="text-xs text-ink-dim leading-relaxed bg-bg rounded-lg p-3 border border-line">
            {backendInfo?.observationLevels && (
              <span data-testid="sandbox-obs-levels" className="block mb-1 text-accent font-mono">
                观测四级: {backendInfo.observationLevels.join(" → ")} (sandtree-core)
              </span>
            )}
            观测方式：原生 → 执行 → 探针 → 元数据。每个值都标出来源、可信度和新鲜度；离线的主机仍列出，不当作空。
          </p>
        </div>
      </aside>

      {/* 接入实体机对话框 (per 2c mockup) */}
      {attachOpen && (
        <div data-testid="sandbox-attach-dialog" className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={() => setAttachOpen(false)}>
          <div className="w-96 rounded-xl border border-line bg-bg-card p-4 shadow-soft" onClick={(e) => e.stopPropagation()}>
            <h3 className="mb-3 text-sm font-bold text-ink">接入实体机</h3>
            <div className="space-y-2.5">
              <label className="block">
                <span className="mb-1 block text-[10px] text-ink-mute">主机地址</span>
                <input
                  value={attachForm.host}
                  onChange={(e) => setAttachForm((f) => ({ ...f, host: e.target.value }))}
                  placeholder="192.168.x.x 或主机名"
                  data-testid="sandbox-attach-host"
                  className="h-9 w-full rounded-lg border border-line bg-bg-soft px-2.5 text-xs focus:border-accent focus:outline-none"
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-[10px] text-ink-mute">系统类型</span>
                <select
                  value={attachForm.os}
                  onChange={(e) => setAttachForm((f) => ({ ...f, os: e.target.value }))}
                  data-testid="sandbox-attach-os"
                  className="h-9 w-full rounded-lg border border-line bg-bg-soft px-2.5 text-xs"
                >
                  <option value="">待探测</option>
                  <option value="Windows 11">Windows 11</option>
                  <option value="Ubuntu 24.04">Ubuntu 24.04</option>
                  <option value="macOS">macOS</option>
                </select>
              </label>
              <label className="block">
                <span className="mb-1 block text-[10px] text-ink-mute">接入方式</span>
                <select
                  value={attachForm.via}
                  onChange={(e) => setAttachForm((f) => ({ ...f, via: e.target.value }))}
                  data-testid="sandbox-attach-via"
                  className="h-9 w-full rounded-lg border border-line bg-bg-soft px-2.5 text-xs"
                >
                  <option value="SSH">SSH</option>
                  <option value="WinRM">WinRM</option>
                  <option value="Agent">Agent (Worker)</option>
                </select>
              </label>
              <p className="rounded border border-warn/40 bg-warn/10 px-2 py-1.5 text-[10px] text-warn">
                凭据与探测由宿主后端执行 — 当前登记本地记录, 后端接入后走真实验证。
              </p>
            </div>
            <div className="mt-3 flex justify-end gap-2">
              <button type="button" onClick={() => setAttachOpen(false)} className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-dim">取消</button>
              <button type="button" onClick={submitAttach} disabled={!attachForm.host.trim()} data-testid="sandbox-attach-submit" className="rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50">接入</button>
            </div>
          </div>
        </div>
      )}
      {attachDone && (
        <div data-testid="sandbox-attach-done" className="fixed top-20 right-4 z-50 rounded-lg border border-ok/50 bg-ok/10 px-3 py-2 text-xs text-ok">
          已登记接入请求 — 验证中 (宿主后端接入后完成探测)
        </div>
      )}

      {/* ============ 主区 ============ */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4 space-y-4">
        {/* 沙盒头部 */}
        <section data-testid="sandbox-detail" className="bg-bg-card rounded-xl border border-line shadow-soft overflow-hidden">
          <header className="px-4 py-3 border-b border-line flex flex-wrap items-center gap-3">
            <div className="size-9 rounded-lg bg-accent/15 grid place-items-center text-accent">
              <Server size={16} />
            </div>
            <div>
              <h1 className="text-base font-bold text-ink">star-k3s-1</h1>
              <p className="text-xs text-ink-dim">Ubuntu 24.04 · 4 核 8 GB</p>
            </div>
            <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-ok/10 text-ok font-semibold flex items-center gap-1">
              <span className="size-1.5 rounded-full bg-ok" /> 在线
            </span>
            <div className="flex-1" />
            {[
              { icon: Play, label: "启动" },
              { icon: Square, label: "停止" },
              { icon: Camera, label: "快照" },
              { icon: Trash2, label: "销毁" },
            ].map((b) => (
              <button key={b.label} type="button" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim hover:border-accent/60">
                <b.icon size={12} /> {b.label}
              </button>
            ))}
          </header>

          {/* tabs */}
          <div data-testid="sandbox-tabs" className="flex items-center gap-1 border-b border-line px-3">
            {SANDBOX_TABS.map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setActiveTab(tab)}
                data-testid={`sandbox-tab-${tab}`}
                className={clsx(
                  "px-3 py-2 text-sm border-b-2 -mb-px transition-colors",
                  activeTab === tab ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
                )}
              >
                {tab}
              </button>
            ))}
          </div>

          {/* Worktree tab 内容 */}
          <div className="p-4">
            <table data-testid="sandbox-worktree-table" className="w-full text-xs">
              <thead>
                <tr className="text-left text-ink-mute border-b border-line">
                  <th className="py-2 pr-3 font-medium">Worktree</th>
                  <th className="py-2 pr-3 font-medium">Branch</th>
                  <th className="py-2 pr-3 font-medium">沙盒内路径</th>
                  <th className="py-2 pr-3 font-medium">状态</th>
                  <th className="py-2 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {worktrees.length > 0 ? worktrees.map((w) => (
                  <tr key={w.id} className="border-b border-line/60">
                    <td className="py-2 pr-3 font-mono font-semibold text-ink">{w.id}</td>
                    <td className="py-2 pr-3 font-mono text-ink-dim flex items-center gap-1">
                      <GitBranch size={11} /> {w.branch}
                    </td>
                    <td className="py-2 pr-3 font-mono text-ink-dim">{w.name}</td>
                    <td className="py-2 pr-3">
                      <span className={clsx("text-[10px] px-1.5 py-0.5 rounded", w.status === "active" ? "bg-ok/10 text-ok" : "bg-bg-soft text-ink-dim")}>{w.status}</span>
                    </td>
                    <td className="py-2">
                      <span className="flex items-center gap-2 text-accent">
                        <button type="button" className="hover:underline flex items-center gap-1"><LogIn size={11} /> 登录</button>
                        <button type="button" className="hover:underline flex items-center gap-1"><RefreshCw size={11} /> 同步</button>
                      </span>
                    </td>
                  </tr>
                )) : (
                  <tr className="border-b border-line/60">
                    <td colSpan={6} className="py-4 text-xs text-ink-mute text-center">暂无 Worktree (点击下方“开设 Worktree”创建)</td>
                  </tr>
                )}
              </tbody>
            </table>

            {/* 开设 Worktree (per 2026-11-22: 接入 store.createWorktree + 服务端 createProjectWorktree) */}
            <section data-testid="sandbox-open-worktree" className="mt-4 rounded-xl border border-line bg-bg p-4 space-y-3">
              <h2 className="text-sm font-bold text-ink">在此沙盒开设 Worktree</h2>
              {wtCreated && (
                <div className="rounded-lg border border-ok/50 bg-ok/10 px-3 py-2 text-xs text-ok mb-2" data-testid="sandbox-wt-created">
                  已创建 Worktree <span className="font-mono font-bold">{wtCreated}</span>，已刷新至左侧导航栏。
                </div>
              )}
              {wtError && (
                <div className="rounded-lg border border-err/50 bg-err/10 px-3 py-2 text-xs text-err mb-2" data-testid="sandbox-wt-error">
                  {wtError}
                </div>
              )}
              <div className="space-y-2">
                <div className="grid grid-cols-2 gap-2">
                  <input
                    data-testid="sandbox-wt-name"
                    type="text"
                    value={wtName}
                    onChange={(e) => setWtName(e.target.value)}
                    placeholder="Worktree 名称 (如 wt-relation-bfs)"
                    className="rounded border border-line bg-bg-soft px-2.5 py-1.5 text-xs text-ink focus:border-accent"
                  />
                  <input
                    data-testid="sandbox-wt-branch"
                    type="text"
                    value={wtBranch}
                    onChange={(e) => setWtBranch(e.target.value)}
                    placeholder="分支名 (如 feat/x)"
                    className="rounded border border-line bg-bg-soft px-2.5 py-1.5 text-xs text-ink focus:border-accent"
                  />
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <input
                    data-testid="sandbox-wt-base-branch"
                    type="text"
                    value={wtBaseBranch}
                    onChange={(e) => setWtBaseBranch(e.target.value)}
                    placeholder="基准分支 (默认 main)"
                    className="rounded border border-line bg-bg-soft px-2.5 py-1.5 text-xs text-ink focus:border-accent"
                  />
                  <input
                    data-testid="sandbox-wt-host-path"
                    type="text"
                    value={wtHostPath}
                    onChange={(e) => setWtHostPath(e.target.value)}
                    placeholder="沙箱挂载路径 (如 D:\\Star\\worktrees\\my-sandbox)"
                    className="rounded border border-line bg-bg-soft px-2.5 py-1.5 text-xs text-ink focus:border-accent"
                  />
                </div>
                <button
                    type="button"
                    data-testid="sandbox-open-worktree-btn"
                    onClick={handleCreateWT}
                    disabled={wtBusy || !wtName.trim() || !wtBranch.trim()}
                    className={`text-xs px-3 py-2 rounded-lg ${wtBusy || !wtName.trim() || !wtBranch.trim() ? "bg-ink-mute/20 text-ink-mute cursor-not-allowed" : "bg-accent text-white hover:bg-accent-soft"}`}
                  >
                    {wtBusy ? "创建中…" : "开设 Worktree"}
                  </button>
              </div>
              <p className="text-[10px] text-ink-dim mt-1">创建后 Worktree 将出现在左侧导航树的 Project 节点下，并可从任务看板、任务卡 CLI、Canvas 等模块读取。</p>
            </section>

            {/* 操作任务 */}
            <section data-testid="sandbox-op-tasks" className="mt-4">
              <h2 className="text-sm font-bold text-ink mb-2">操作任务</h2>
              <ul className="space-y-1.5">
                {OP_TASKS.map((t) => (
                  <li key={t.title + t.state} className="flex items-center gap-2.5 px-3 py-2 rounded-lg border border-line bg-bg-card text-xs">
                    <t.icon size={13} className={t.tone} />
                    <span className="font-semibold text-ink">{t.title}</span>
                    <span className={clsx("text-[10px] px-1.5 py-0.5 rounded font-semibold", t.state === "完成" ? "bg-ok/10 text-ok" : t.state === "失败" ? "bg-err/10 text-err" : "bg-accent/10 text-accent")}>
                      {t.state}
                    </span>
                    <span className="text-ink-dim flex-1 truncate">{t.detail}</span>
                    {t.state === "运行中" && <Clock size={12} className="text-ink-mute" />}
                  </li>
                ))}
              </ul>
            </section>

            {/* 安全说明 */}
            <p data-testid="sandbox-security-note" className="mt-3 text-[11px] text-ink-dim flex items-start gap-1.5 leading-relaxed">
              <ShieldAlert size={12} className="text-warn mt-0.5 shrink-0" />
              Agent、插件和任务卡 CLI 拿不到宿主的 Docker 或虚拟机控制权；这些操作只由受信的宿主管理进程执行，并需要你的授权。
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}
