/*
@cypher schema=1 source_sha256=261b46a334e157e3d9c86e8128a3cef54b34045e1fc86d9a895dedd505eaaefa
MERGE (self:File {path:"frontend/src/app/(app)/kanban/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::KanbanPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /kanban — 1b · 任务 · IPA 看板 (per 2026-11-14 参考 mockup: TasksIpa.dc.html)
// 结构: 面包屑(Project→LANE→Worktree→Branch) + 6 视图 tabs + 任务生命周期条
//      + Sprint 卡片区 + Backlog 开卡区 + 右侧 Worktree 状态/聊天/Team rail
// =====================================================================
"use client";

import { useState } from "react";
import {
  ChevronDown,
  GitBranch,
  Plus,
  FileText,
  MessageSquare,
  Users,
  ListChecks,
  GitCommitHorizontal,
} from "lucide-react";
import { clsx } from "clsx";

const VIEW_TABS = ["Sprint", "任务看板", "IPA 流程图", "按文件开卡", "时间线", "日历"] as const;

const LIFECYCLE = ["Backlog", "待执行", "进行中", "评审", "完成", "归档"] as const;

interface TaskCard {
  id: string;
  title: string;
  ipa: string;
  ipaLabel: string;
  files: number;
  points: number;
  status: string;
}

const SPRINT_12: TaskCard[] = [
  { id: "T-121", title: "关系图服务边界与契约", ipa: "ipa-s03-003", ipaLabel: "アーキテクチャ設計", files: 0, points: 5, status: "评审" },
  { id: "T-122", title: "关系查询 API 契约", ipa: "ipa-s03-007", ipaLabel: "API設計", files: 1, points: 3, status: "完成" },
  { id: "T-123", title: "分页与游标 API 契约", ipa: "ipa-s03-007", ipaLabel: "API設計", files: 0, points: 2, status: "待执行" },
  { id: "T-126", title: "BFS 模块接口与状态", ipa: "ipa-s04-002", ipaLabel: "モジュール設計", files: 1, points: 3, status: "进行中" },
  { id: "T-127", title: "遍历器与访问集类设计", ipa: "ipa-s04-003", ipaLabel: "クラス設計", files: 0, points: 2, status: "待执行" },
  { id: "T-128", title: "BFS 详细测试设计", ipa: "ipa-s04-014", ipaLabel: "テスト設計（詳細）", files: 0, points: 2, status: "待执行" },
  { id: "T-131", title: "BFS 基础实现", ipa: "ipa-s05-002", ipaLabel: "ソースコード作成", files: 0, points: 5, status: "完成" },
  { id: "T-142", title: "BFS 环检测", ipa: "ipa-s05-002", ipaLabel: "ソースコード作成", files: 3, points: 5, status: "进行中" },
  { id: "T-143", title: "关系权重", ipa: "ipa-s05-002", ipaLabel: "ソースコード作成", files: 0, points: 3, status: "评审" },
  { id: "T-133", title: "BFS 基础实现代码评审", ipa: "ipa-s05-006", ipaLabel: "コードレビュー", files: 0, points: 1, status: "完成" },
  { id: "T-151", title: "BFS 单测规格", ipa: "ipa-s06-ut-002", ipaLabel: "試験仕様書作成", files: 1, points: 2, status: "进行中" },
];

const SPRINT_13: TaskCard[] = [
  { id: "T-152", title: "结果分页", ipa: "ipa-s05-002", ipaLabel: "ソースコード作成", files: 0, points: 3, status: "Backlog" },
  { id: "T-150", title: "环检测边界用例", ipa: "ipa-s06-ut-003", ipaLabel: "試験ケース作成", files: 1, points: 2, status: "Backlog" },
];

const BACKLOG: TaskCard[] = [
  { id: "T-118", title: "关系查询的性能与规模约束", ipa: "ipa-s02-003", ipaLabel: "システム要件定義", files: 0, points: 3, status: "Backlog" },
  { id: "T-160", title: "关系服务集成环境", ipa: "ipa-s06-it-003", ipaLabel: "試験環境構築", files: 0, points: 3, status: "Backlog" },
];

const STATUS_TONE: Record<string, string> = {
  评审: "bg-violet-100 text-violet-700 border-violet-300",
  完成: "bg-emerald-100 text-emerald-700 border-emerald-300",
  待执行: "bg-slate-100 text-slate-600 border-slate-300",
  进行中: "bg-blue-100 text-blue-700 border-blue-300",
  Backlog: "bg-gray-100 text-gray-500 border-gray-300",
};

function TaskRow({ card, target }: { card: TaskCard; target: string }) {
  return (
    <li
      data-testid={`kanban-task-${card.id}`}
      className="flex items-center gap-3 px-3 py-2.5 bg-white border border-gray-200 rounded-lg hover:border-blue-400 transition-colors"
    >
      <span className="font-mono text-xs font-semibold text-gray-500 w-12 shrink-0">{card.id}</span>
      <span className="flex-1 text-sm text-gray-900 truncate">{card.title}</span>
      <span className="text-[11px] font-mono text-indigo-700 w-44 shrink-0 truncate" title={`${card.ipa} ${card.ipaLabel}`}>
        {card.ipa} <span className="text-gray-400">{card.ipaLabel}</span>
      </span>
      <span className="text-[11px] text-gray-500 w-12 shrink-0 flex items-center gap-1">
        <FileText size={11} /> {card.files}
      </span>
      <span className="text-[11px] font-semibold text-gray-700 w-8 text-right shrink-0">{card.points}</span>
      <span className={clsx("text-[10px] px-1.5 py-0.5 rounded border w-14 text-center shrink-0", STATUS_TONE[card.status])}>
        {card.status}
      </span>
      <button
        type="button"
        className="text-[11px] text-blue-700 hover:bg-blue-50 rounded px-1.5 py-0.5 border border-transparent hover:border-blue-200 shrink-0"
        data-testid={`kanban-task-${card.id}-move`}
      >
        → {target}
      </button>
    </li>
  );
}

export default function KanbanPage() {
  const [activeView, setActiveView] = useState<(typeof VIEW_TABS)[number]>("Sprint");

  return (
    <div className="flex h-full bg-gray-50">
      {/* ============ 主区 ============ */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4 space-y-4">
        {/* 面包屑: Project → LANE → Worktree → Branch */}
        <nav data-testid="kanban-breadcrumb" className="flex items-center gap-1.5 text-xs text-gray-600 flex-wrap">
          <button type="button" className="flex items-center gap-1 hover:text-gray-900">
            <span className="font-medium">Physis / GVPE</span> <ChevronDown size={12} />
          </button>
          <span className="text-gray-300">▸</span>
          <button type="button" className="flex items-center gap-1 hover:text-gray-900">
            LANE · auth-refactor <ChevronDown size={12} />
          </button>
          <span className="text-gray-300">▸</span>
          <button type="button" className="flex items-center gap-1 hover:text-gray-900">
            wt-relation-bfs <ChevronDown size={12} />
          </button>
          <button type="button" className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-gray-200 bg-white font-mono hover:border-gray-400">
            <GitBranch size={11} /> feat/relation-bfs
          </button>
          <button type="button" className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-gray-200 bg-white font-mono hover:border-gray-400">
            <GitBranch size={11} /> fix/bfs-cycle
          </button>
        </nav>

        {/* 视图 tabs */}
        <div data-testid="kanban-view-tabs" className="flex items-center gap-1 border-b border-gray-200">
          {VIEW_TABS.map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => setActiveView(tab)}
              data-testid={`kanban-view-tab-${tab}`}
              className={clsx(
                "px-3 py-2 text-sm border-b-2 -mb-px transition-colors",
                activeView === tab
                  ? "border-blue-700 text-blue-700 font-semibold"
                  : "border-transparent text-gray-500 hover:text-gray-900",
              )}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* 生命周期条 */}
        <div data-testid="kanban-lifecycle" className="flex items-center gap-2 text-[11px] text-gray-500 flex-wrap">
          {LIFECYCLE.map((stage, i) => (
            <span key={stage} className="flex items-center gap-2">
              <span className={clsx("px-2 py-0.5 rounded-full border", i === 2 ? "bg-blue-50 border-blue-300 text-blue-700 font-semibold" : "border-gray-200 bg-white")}>
                {stage}
              </span>
              {i < LIFECYCLE.length - 1 && <span className="text-gray-300">→</span>}
            </span>
          ))}
        </div>

        {/* Sprint 12 */}
        <section data-testid="kanban-sprint-12" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
          <header className="px-4 py-3 border-b border-gray-200 flex flex-wrap items-center gap-3">
            <h2 className="text-base font-bold text-gray-900">Sprint 12</h2>
            <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-blue-100 text-blue-700 font-semibold">进行中</span>
            <span className="text-xs text-gray-500 font-mono">11-11 → 11-25 · 11 张卡 · 33 点</span>
            <span className="text-xs text-gray-600 flex-1">目标：关系图环检测上线</span>
            <span className="text-xs text-gray-500">
              完成 <span className="font-bold text-gray-900">9 / 33</span> 点
            </span>
            <button type="button" data-testid="kanban-finish-sprint" className="text-xs px-2.5 py-1.5 rounded-lg bg-blue-700 text-white hover:bg-blue-800">
              完成冲刺
            </button>
          </header>
          <ul className="divide-y divide-gray-100 p-1.5">
            {SPRINT_12.map((card) => (
              <TaskRow key={card.id} card={card} target="S13" />
            ))}
          </ul>
        </section>

        {/* Sprint 13 */}
        <section data-testid="kanban-sprint-13" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
          <header className="px-4 py-3 border-b border-gray-200 flex flex-wrap items-center gap-3">
            <h2 className="text-base font-bold text-gray-900">Sprint 13</h2>
            <span className="text-[11px] px-1.5 py-0.5 rounded-full bg-gray-100 text-gray-500 font-semibold">计划中</span>
            <span className="text-xs text-gray-500 font-mono">11-25 → 12-09 · 2 张卡 · 5 点</span>
            <span className="text-xs text-gray-600 flex-1">目标：结果分页与集成测试</span>
            <button type="button" data-testid="kanban-start-sprint" className="text-xs px-2.5 py-1.5 rounded-lg border border-blue-700 text-blue-700 hover:bg-blue-50">
              启动冲刺
            </button>
          </header>
          <ul className="divide-y divide-gray-100 p-1.5">
            {SPRINT_13.map((card) => (
              <TaskRow key={card.id} card={card} target="S12" />
            ))}
          </ul>
        </section>

        {/* Backlog */}
        <section data-testid="kanban-backlog" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
          <header className="px-4 py-3 border-b border-gray-200 flex items-center gap-3">
            <h2 className="text-base font-bold text-gray-900">Backlog</h2>
            <span className="text-xs text-gray-500">2 张 · 未排入任何冲刺</span>
            <div className="flex-1" />
            <button type="button" className="text-xs px-2.5 py-1.5 rounded-lg border border-gray-300 text-gray-700 hover:border-blue-400 flex items-center gap-1">
              <ListChecks size={12} /> 从 IPA 流程建卡
            </button>
            <button type="button" className="text-xs px-2.5 py-1.5 rounded-lg border border-gray-300 text-gray-700 hover:border-blue-400 flex items-center gap-1">
              <FileText size={12} /> 按文件开卡
            </button>
          </header>

          {/* 开卡表单 */}
          <div data-testid="kanban-new-card-form" className="px-4 py-3 bg-gray-50 border-b border-gray-200 space-y-2">
            <div className="flex items-center gap-2 flex-wrap text-xs">
              <span className="text-gray-500">IPA：</span>
              <span className="font-mono px-2 py-0.5 rounded bg-white border border-gray-200">ipa-s05-002 ソースコード作成</span>
              <span className="font-mono px-2 py-0.5 rounded bg-white border border-gray-200">ipa-s06-ut-005 単体試験実施</span>
              <span className="font-mono px-2 py-0.5 rounded bg-white border border-gray-200 text-gray-400">不关联 IPA</span>
            </div>
            <div className="flex items-center gap-2 flex-wrap text-xs">
              <span className="text-gray-500">工作流：</span>
              <span className="px-2 py-0.5 rounded bg-white border border-gray-200">标准研发</span>
              <span className="px-2 py-0.5 rounded bg-white border border-gray-200">缺陷修复</span>
              <span className="px-2 py-0.5 rounded bg-white border border-gray-200">文档评审</span>
            </div>
            <div className="flex items-center gap-2 flex-wrap text-xs">
              <span className="text-gray-500">文件：</span>
              <span className="font-mono px-2 py-0.5 rounded bg-white border border-gray-200">bfs.rs</span>
              <span className="font-mono px-2 py-0.5 rounded bg-white border border-gray-200 text-gray-400">无</span>
              <span className="px-2 py-0.5 rounded bg-white border border-gray-200 text-gray-400 flex items-center gap-1">
                <GitBranch size={11} /> 从 Branch 选择…
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs">
              {["3 点", "1 点", "5 点", "8 点"].map((p) => (
                <button key={p} type="button" className="px-2 py-0.5 rounded border border-gray-300">{p}</button>
              ))}
              <button type="button" data-testid="kanban-create-card" className="ml-auto px-3 py-1 rounded-lg bg-blue-700 text-white hover:bg-blue-800 flex items-center gap-1">
                <Plus size={12} /> 开卡
              </button>
            </div>
          </div>

          <ul className="divide-y divide-gray-100 p-1.5">
            {BACKLOG.map((card) => (
              <TaskRow key={card.id} card={card} target="S12" />
            ))}
          </ul>
        </section>
      </main>

      {/* ============ 右侧 rail ============ */}
      <aside data-testid="kanban-right-rail" className="w-72 shrink-0 border-l border-gray-200 bg-white flex flex-col">
        <div className="px-3 py-2.5 border-b border-gray-200 flex items-center gap-2 text-xs font-semibold text-gray-700">
          <GitCommitHorizontal size={13} className="text-emerald-600" />
          Worktree 状态 · <span className="text-emerald-600 font-mono">干净</span>
        </div>

        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          <div className="flex items-center gap-2 text-xs font-semibold text-gray-700">
            <MessageSquare size={13} className="text-blue-600" /> 聊天
          </div>
          <div data-testid="kanban-chat-panel" className="rounded-lg border border-gray-200 bg-gray-50 p-3 space-y-2">
            <p className="text-xs text-gray-800 leading-relaxed">详细设计还有哪些流程没建卡？</p>
            <p className="text-xs text-gray-600 leading-relaxed">
              P4 共 20 个流程，已建卡 3 个。建议先建 <span className="font-mono">ロジック設計</span> 和{" "}
              <span className="font-mono">DB詳細設計</span>，要我放进 Backlog 吗？
            </p>
            <button type="button" data-testid="kanban-chat-ask" className="text-[11px] px-2 py-1 rounded border border-gray-300 bg-white hover:border-blue-400 text-gray-600">
              问问这个 Branch 的进展
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs font-semibold text-gray-700">
            <Users size={13} className="text-gray-500" /> Team
          </div>
          <div className="flex -space-x-1.5">
            {["U", "林", "A", "B"].map((who, i) => (
              <span
                key={i}
                className="size-7 rounded-full border-2 border-white bg-indigo-100 text-indigo-700 text-[10px] font-bold grid place-items-center"
              >
                {who}
              </span>
            ))}
            <span className="size-7 rounded-full border-2 border-white bg-gray-100 text-gray-500 text-[9px] font-bold grid place-items-center">
              4 在线
            </span>
          </div>
        </div>
      </aside>
    </div>
  );
}
