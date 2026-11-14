/*
@cypher schema=1 source_sha256=1e1748c98442a94d9026641911089d837f5b6e7644f1c9dea14a58c9164c523c
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

import { useState } from "react";
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
} from "lucide-react";
import { clsx } from "clsx";

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
      { name: "ws-office", sub: "实体机 · Controller", badge: "原生", badgeTone: "bg-emerald-100 text-emerald-700" },
      { name: "lab-01", sub: "实体机 · Worker · WireGuard", badge: "执行", badgeTone: "bg-blue-100 text-blue-700" },
      { name: "mbp-ulysses", sub: "实体机 · 离线 2 小时", badge: "元数据", badgeTone: "bg-gray-100 text-gray-500" },
    ],
  },
  {
    group: "虚拟机",
    hint: "Multipass 最多保留 2 台",
    items: [
      { name: "star-k3s-1", sub: "Multipass · K3s 节点", badge: "执行", badgeTone: "bg-blue-100 text-blue-700", online: true },
      { name: "star-k3s-2", sub: "Multipass · 已停止", badge: "元数据", badgeTone: "bg-gray-100 text-gray-500" },
      { name: "wsb-relation", sub: "Windows Sandbox", badge: "探针", badgeTone: "bg-violet-100 text-violet-700" },
    ],
  },
  {
    group: "Docker",
    items: [
      { name: "docker", sub: "本机 Docker 守护进程 · 6 个容器", badge: "原生", badgeTone: "bg-emerald-100 text-emerald-700" },
      { name: "sbx-relation", sub: "Docker 沙盒 · 集成测试", badge: "探针", badgeTone: "bg-violet-100 text-violet-700" },
    ],
  },
];

const WORKTREES = [
  { wt: "wt-relation-bfs", branch: "feat/relation-bfs", path: "~/worktrees/wt-relation-bfs", status: "运行中", tone: "bg-emerald-100 text-emerald-700" },
  { wt: "wt-cache-ttl", branch: "fix/cache-ttl", path: "~/worktrees/wt-cache-ttl", status: "空闲", tone: "bg-gray-100 text-gray-500" },
];

const OP_TASKS = [
  { title: "开设 Worktree", state: "完成", detail: "star-k3s-1 · wt-relation-bfs · 2 分钟前", icon: CheckCircle2, tone: "text-emerald-600" },
  { title: "快照", state: "运行中", detail: "star-k3s-1 · 第 2/3 步 启动虚拟机", icon: Loader2, tone: "text-blue-600 animate-spin" },
  { title: "重启容器", state: "失败", detail: "canvas-realtime · 健康检查超时", icon: XCircle, tone: "text-rose-600" },
];

export default function SandboxPage() {
  const [activeTab, setActiveTab] = useState<(typeof SANDBOX_TABS)[number]>("Worktree");

  return (
    <div className="flex h-full bg-gray-50">
      {/* ============ 左: 资源树 ============ */}
      <aside data-testid="sandbox-resource-tree" className="w-72 shrink-0 border-r border-gray-200 bg-white overflow-y-auto">
        <div className="px-3 py-3 border-b border-gray-200 flex items-center gap-2">
          <button type="button" data-testid="sandbox-new" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700">
            <Plus size={12} /> 新建沙盒
          </button>
          <button type="button" className="text-xs px-2.5 py-1.5 rounded-lg border border-gray-300 text-gray-700 hover:border-indigo-400">
            接入实体机
          </button>
        </div>

        <div className="px-3 py-3 space-y-4">
          {RESOURCES.map((group) => (
            <div key={group.group}>
              <div className="text-[10px] font-mono uppercase tracking-wider text-gray-400 mb-1.5">
                {group.group}
                {group.hint && <span className="text-gray-300 normal-case tracking-normal ml-1">· {group.hint}</span>}
              </div>
              <ul className="space-y-1">
                {group.items.map((r) => (
                  <li
                    key={r.name}
                    data-testid={`sandbox-res-${r.name}`}
                    className={clsx(
                      "px-2.5 py-2 rounded-lg border text-xs",
                      r.online ? "border-indigo-300 bg-indigo-50" : "border-gray-200 hover:border-gray-300",
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <span className="font-mono font-semibold text-gray-800 flex-1 truncate">{r.name}</span>
                      <span className={clsx("text-[9px] px-1.5 py-0.5 rounded font-semibold", r.badgeTone)}>{r.badge}</span>
                      {r.online && <span className="size-1.5 rounded-full bg-emerald-500" />}
                    </div>
                    <div className="text-[10px] text-gray-500 mt-0.5">{r.sub}</div>
                  </li>
                ))}
              </ul>
            </div>
          ))}

          <p data-testid="sandbox-observation-note" className="text-[11px] text-gray-500 leading-relaxed bg-gray-50 rounded-lg p-2.5 border border-gray-200">
            观测方式：原生 → 执行 → 探针 → 元数据。每个值都标出来源、可信度和新鲜度；离线的主机仍列出，不当作空。
          </p>
        </div>
      </aside>

      {/* ============ 主区 ============ */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4 space-y-4">
        {/* 沙盒头部 */}
        <section data-testid="sandbox-detail" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
          <header className="px-4 py-3 border-b border-gray-200 flex flex-wrap items-center gap-3">
            <div className="size-9 rounded-lg bg-indigo-100 grid place-items-center text-indigo-700">
              <Server size={16} />
            </div>
            <div>
              <h1 className="text-base font-bold text-gray-900">star-k3s-1</h1>
              <p className="text-xs text-gray-500">Ubuntu 24.04 · 4 核 8 GB</p>
            </div>
            <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-emerald-100 text-emerald-700 font-semibold flex items-center gap-1">
              <span className="size-1.5 rounded-full bg-emerald-500" /> 在线
            </span>
            <div className="flex-1" />
            {[
              { icon: Play, label: "启动" },
              { icon: Square, label: "停止" },
              { icon: Camera, label: "快照" },
              { icon: Trash2, label: "销毁" },
            ].map((b) => (
              <button key={b.label} type="button" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border border-gray-300 text-gray-700 hover:border-indigo-400">
                <b.icon size={12} /> {b.label}
              </button>
            ))}
          </header>

          {/* tabs */}
          <div data-testid="sandbox-tabs" className="flex items-center gap-1 border-b border-gray-200 px-3">
            {SANDBOX_TABS.map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setActiveTab(tab)}
                data-testid={`sandbox-tab-${tab}`}
                className={clsx(
                  "px-3 py-2 text-sm border-b-2 -mb-px transition-colors",
                  activeTab === tab ? "border-indigo-600 text-indigo-700 font-semibold" : "border-transparent text-gray-500 hover:text-gray-900",
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
                <tr className="text-left text-gray-400 border-b border-gray-200">
                  <th className="py-2 pr-3 font-medium">Worktree</th>
                  <th className="py-2 pr-3 font-medium">Branch</th>
                  <th className="py-2 pr-3 font-medium">沙盒内路径</th>
                  <th className="py-2 pr-3 font-medium">状态</th>
                  <th className="py-2 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {WORKTREES.map((w) => (
                  <tr key={w.wt} className="border-b border-gray-100">
                    <td className="py-2 pr-3 font-mono font-semibold text-gray-800">{w.wt}</td>
                    <td className="py-2 pr-3 font-mono text-gray-600 flex items-center gap-1">
                      <GitBranch size={11} /> {w.branch}
                    </td>
                    <td className="py-2 pr-3 font-mono text-gray-500">{w.path}</td>
                    <td className="py-2 pr-3">
                      <span className={clsx("text-[10px] px-1.5 py-0.5 rounded", w.tone)}>{w.status}</span>
                    </td>
                    <td className="py-2">
                      <span className="flex items-center gap-2 text-indigo-700">
                        <button type="button" className="hover:underline flex items-center gap-1"><LogIn size={11} /> 登录</button>
                        <button type="button" className="hover:underline flex items-center gap-1"><RefreshCw size={11} /> 同步</button>
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {/* 开设 Worktree */}
            <section data-testid="sandbox-open-worktree" className="mt-4 rounded-xl border border-gray-200 bg-gray-50 p-4 space-y-3">
              <h2 className="text-sm font-bold text-gray-900">在此沙盒开设 Worktree</h2>
              <div className="flex items-center gap-2 text-xs flex-wrap">
                <span className="text-gray-500">仓库</span>
                <span className="px-2 py-1 rounded border border-gray-200 bg-white font-mono">Physis / GVPE · star</span>
              </div>
              <div className="flex items-center gap-2 text-xs flex-wrap">
                <span className="text-gray-500">Branch</span>
                <span className="px-2 py-1 rounded border border-gray-200 bg-white font-mono flex items-center gap-1">
                  <GitBranch size={11} /> feat/relation-bfs（当前）
                </span>
                <span className="px-2 py-1 rounded border border-gray-200 bg-white text-gray-400">新建 Branch…</span>
              </div>
              <div className="flex items-center gap-2 text-xs">
                <span className="text-gray-500">关联任务卡</span>
                <span className="px-2 py-1 rounded border border-gray-200 bg-white font-mono">T-142 BFS 环检测</span>
              </div>
              <div className="space-y-1.5">
                <div className="text-xs text-gray-500">代码进入方式</div>
                {[
                  { title: "在沙盒内克隆", desc: "沙盒内独立克隆仓库并检出 Branch，与宿主完全隔离，推荐。", checked: true },
                  { title: "限路径同步（可审计）", desc: "", checked: false },
                  { title: "只读挂载", desc: "", checked: false },
                ].map((opt) => (
                  <label key={opt.title} className={clsx("flex items-start gap-2 p-2.5 rounded-lg border text-xs cursor-pointer", opt.checked ? "border-indigo-400 bg-white" : "border-gray-200 bg-white")}>
                    <input type="radio" name="code-entry" defaultChecked={opt.checked} className="mt-0.5 accent-indigo-600" />
                    <span>
                      <span className="font-semibold text-gray-800">{opt.title}</span>
                      {opt.desc && <span className="block text-gray-500 mt-0.5">{opt.desc}</span>}
                    </span>
                  </label>
                ))}
              </div>
              <button type="button" data-testid="sandbox-open-worktree-btn" className="text-xs px-3 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700">
                开设 Worktree
              </button>
            </section>

            {/* 操作任务 */}
            <section data-testid="sandbox-op-tasks" className="mt-4">
              <h2 className="text-sm font-bold text-gray-900 mb-2">操作任务</h2>
              <ul className="space-y-1.5">
                {OP_TASKS.map((t) => (
                  <li key={t.title + t.state} className="flex items-center gap-2.5 px-3 py-2 rounded-lg border border-gray-200 bg-white text-xs">
                    <t.icon size={13} className={t.tone} />
                    <span className="font-semibold text-gray-800">{t.title}</span>
                    <span className={clsx("text-[10px] px-1.5 py-0.5 rounded font-semibold", t.state === "完成" ? "bg-emerald-100 text-emerald-700" : t.state === "失败" ? "bg-rose-100 text-rose-700" : "bg-blue-100 text-blue-700")}>
                      {t.state}
                    </span>
                    <span className="text-gray-500 flex-1 truncate">{t.detail}</span>
                    {t.state === "运行中" && <Clock size={12} className="text-gray-400" />}
                  </li>
                ))}
              </ul>
            </section>

            {/* 安全说明 */}
            <p data-testid="sandbox-security-note" className="mt-3 text-[11px] text-gray-500 flex items-start gap-1.5 leading-relaxed">
              <ShieldAlert size={12} className="text-amber-600 mt-0.5 shrink-0" />
              Agent、插件和任务卡 CLI 拿不到宿主的 Docker 或虚拟机控制权；这些操作只由受信的宿主管理进程执行，并需要你的授权。
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}
