/*
@cypher schema=1 source_sha256=95529379f02d38988fc6aefcf97d5d840dd8cea7790547cd86d3efcfba165769
MERGE (self:File {path:"frontend/src/app/(app)/nema/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::NemaPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /nema — 2b · 协同（文件即任务 · 多人多 Agent）
// per 2026-11-14 参考 mockup: Docs.dc.html
// 结构: 左 工作集(任务卡/文件列表) + 中 多 Agent 流水线(5 步) +
//      当前文件 .nema 规约 / 生成代码对照 + 协作组(同一文件/独立组) + 租约说明
// =====================================================================
"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import {
  FileCode2,
  FileText,
  Table,
  Plus,
  Users,
  GitBranch,
  Send,
  Lock,
  Terminal,
  UserPlus,
  X,
} from "lucide-react";
import { clsx } from "clsx";
import { useNemaStore } from "@/lib/nema/nemaStore";
import { useNemaWatcher } from "@/lib/nema/nemaWatcher";
import { LocalFolderBrowser } from "@/components/nema/LocalFolderBrowser";

const PIPELINE = ["① 路由拆分", "② 并行编辑", "③ 语义合并", "④ 生成代码", "⑤ 验证"] as const;

const CODE_FILES = [
  { name: "bfs.rs", people: 3, kind: "code" },
  { name: "bfs.rs.nema", people: 3, kind: "code" },
  { name: "bfs_test.rs", people: 1, kind: "code" },
  { name: "graph.rs", people: 0, kind: "code" },
];

const DOC_FILES = [
  { name: "relation-detail-design.md", people: 2, kind: "doc" },
  { name: "单测清单.sheet", people: 1, kind: "sheet" },
];

const NEMA_LINES = [
  { tag: "1", cls: "text-rose-600", text: "!NEMAT1" },
  { tag: "2", cls: "text-blue-600", text: "?1 REV 3" },
  { tag: "3", cls: "text-gray-800", text: "BFS 环检测" },
  { tag: "4", cls: "text-gray-800", text: "防止遍历死循环" },
  { tag: "5", cls: "text-gray-800", text: "访问集 + 路径栈判回边", who: "Agent A1" },
  { tag: "6", cls: "text-gray-800", text: "Agent A1, Agent A2, 林" },
  { tag: "7", cls: "text-blue-600", text: "\\2 REV 2" },
  { tag: "8", cls: "text-gray-700", text: "# 子任务 状态 负责人" },
  { tag: "9", cls: "text-gray-800", text: "回边判定 完成 A1", who: "Agent A1" },
  { tag: "10", cls: "text-gray-800", text: "环路径输出 进行中 A2", who: "Agent A2" },
  { tag: "11", cls: "text-gray-800", text: "深度上限 待定 林", who: "林" },
  { tag: "12", cls: "text-blue-600", text: "\\>3 REV 1" },
  { tag: "13", cls: "text-gray-800", text: "bfs>calls>graph 邻接查询" },
];

const CODE_LINES = [
  { n: 1, text: "pub fn detect_cycle(g: &Graph, start: NodeId) -> Option<Vec<NodeId>> {" },
  { n: 2, text: "    let mut visited = HashSet::new();" },
  { n: 3, text: "    let mut path = Vec::new();", who: "Agent A1" },
  { n: 4, text: "    dfs(g, start, &mut visited, &mut path)" },
  { n: 5, text: "}" },
  { n: 6, text: "" },
  { n: 7, text: "fn on_back_edge(path: &[NodeId], to: NodeId) -> Vec<NodeId> {" },
  { n: 8, text: "    let i = path.iter().position(|n| *n == to).unwrap();", who: "Agent A2" },
  { n: 9, text: "    path[i..].to_vec()" },
  { n: 10, text: "}" },
  { n: 11, text: "" },
  { n: 12, text: "const MAX_DEPTH: usize = 64;" },
];

export default function NemaPage() {
  const router = useRouter();
  const [scope, setScope] = useState<"task" | "other">("task");
  const [showForm, setShowForm] = useState(false);
  const [filePath, setFilePath] = useState("");
  const [title, setTitle] = useState("");
  const [editorsText, setEditorsText] = useState("");

  // ---- 真实功能接线 ----
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const registerSharedFile = useNemaStore((s) => s.registerSharedFile);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);
  const nemaWatcher = useNemaWatcher([""]);

  const taskRows = nemaTasks.filter((x) => x.workItemId).map((x) => ({
    id: x.workItemId ?? x.id,
    title: x.title || x.filePath,
    status: "已开卡",
  }));
  const otherRows = nemaTasks.filter((x) => !x.workItemId).map((x) => ({
    id: x.id,
    title: x.title || x.filePath,
    status: "",
  }));
  const visibleTasks = scope === "task" ? taskRows : otherRows;

  const codeFiles = nemaTasks.map((x) => ({
    name: x.filePath.split("/").pop() ?? x.filePath,
    people: x.editors.length,
    task: x,
  }));
  const docFiles = nemaTasks
    .filter((x) => /\.(md|sheet)$/i.test(x.filePath))
    .map((x) => ({ name: x.filePath.split("/").pop() ?? x.filePath, people: x.editors.length, task: x }));

  const handleRegister = () => {
    const editors = editorsText.split(/[,，\s]+/).map((s) => s.trim()).filter(Boolean);
    if (!filePath.trim() || editors.length === 0) return;
    registerSharedFile({ filePath: filePath.trim(), title: title.trim(), editors });
    setShowForm(false);
    setFilePath("");
    setTitle("");
    setEditorsText("");
  };

  return (
    <div className="flex h-full bg-gray-50">
      {/* ============ 左: 工作集 ============ */}
      <aside data-testid="nema-workset" className="w-72 shrink-0 border-r border-gray-200 bg-white overflow-y-auto">
        <div className="px-3 py-3 border-b border-gray-200">
          <div className="text-[11px] text-gray-500">隶属当前 Worktree · Branch</div>
          <div className="mt-0.5 text-xs font-mono font-semibold text-gray-800 flex items-center gap-1.5">
            <GitBranch size={12} className="text-indigo-600" /> wt-relation-bfs · feat/relation-bfs
          </div>
        </div>

        {/* 按任务卡 / 其他 */}
        <div className="px-3 py-2 flex gap-1">
          {(["task", "other"] as const).map((s) => (
            <button
              key={s}
              type="button"
              onClick={() => setScope(s)}
              className={clsx(
                "flex-1 text-xs px-2 py-1.5 rounded-lg border transition-colors",
                scope === s ? "border-indigo-600 text-indigo-700 bg-indigo-50 font-semibold" : "border-gray-200 text-gray-500",
              )}
            >
              {s === "task" ? "按任务卡" : "其他"}
            </button>
          ))}
        </div>

        <div className="px-3 space-y-3 pb-4">
          {/* 任务卡选择 */}
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-gray-400 mb-1.5">任务卡</div>
            {visibleTasks.length === 0 ? (
              <p className="text-[11px] text-gray-400 px-1 py-2">暂无{scope === "task" ? "已开卡" : "未开卡"}的 .nema 文件</p>
            ) : (
              <ul className="space-y-1">
                {visibleTasks.map((task) => (
                  <li
                    key={task.id}
                    data-testid={`nema-task-${task.id}`}
                    className={clsx(
                      "flex items-center gap-2 px-2.5 py-2 rounded-lg border text-xs",
                      task.status ? "border-indigo-300 bg-indigo-50" : "border-gray-200 hover:border-gray-300",
                    )}
                  >
                    <span className="font-mono font-semibold text-gray-600 truncate w-14">{task.id}</span>
                    <span className="flex-1 truncate text-gray-800">{task.title}</span>
                    {task.status && <span className="text-[10px] text-indigo-700">{task.status}</span>}
                    {task.status && (
                      <button type="button" onClick={() => router.push(`/work-item/${task.id}`)} className="text-[10px] text-gray-400 hover:text-indigo-700">
                        打开
                      </button>
                    )}
                  </li>
                ))}
              </ul>
            )}
            <div className="mt-1.5 text-[11px] font-mono text-indigo-700 flex items-center gap-1">
              <FileCode2 size={11} /> ipa-s05-002 ソースコード作成 · 文件即任务
            </div>
          </div>

          {/* 文件列表 */}
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-gray-400 mb-1.5 flex items-center justify-between">
              <span>代码 <span className="text-gray-300">4</span></span>
            </div>
            {codeFiles.length === 0 ? (
              <p className="text-[11px] text-gray-400 px-1 py-2">暂无代码文件 — 关联共享文件后自动生成 .nema 侧车</p>
            ) : (
              <ul className="space-y-1">
                {codeFiles.map((f) => (
                  <li key={f.name} data-testid={`nema-file-${f.name}`} className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-gray-200 text-xs hover:border-indigo-300">
                    <FileCode2 size={12} className="text-gray-400 shrink-0" />
                    <span className="flex-1 truncate font-mono text-gray-800">{f.name}</span>
                    <span className="text-[10px] text-gray-500 flex items-center gap-0.5">
                      <Users size={10} /> {f.people} 人
                    </span>
                    {!f.task.workItemId && (
                      <button
                        type="button"
                        data-testid={`nema-open-task-${f.name}`}
                        onClick={() => {
                          const wid = openNemaAsStarTask(f.task.id);
                          if (wid) router.push(`/work-item/${wid}`);
                        }}
                        className="text-[10px] text-indigo-700 hover:underline"
                      >
                        开卡
                      </button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-gray-400 mb-1.5">文档与表格 <span className="text-gray-300">2</span></div>
            {docFiles.length === 0 ? (
              <p className="text-[11px] text-gray-400 px-1 py-2">暂无文档/表格</p>
            ) : (
              <ul className="space-y-1">
                {docFiles.map((f) => (
                  <li key={f.name} data-testid={`nema-file-${f.name}`} className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-gray-200 text-xs hover:border-indigo-300">
                    <FileText size={12} className="text-gray-400 shrink-0" />
                    <span className="flex-1 truncate font-mono text-gray-800">{f.name}</span>
                    <span className="text-[10px] text-gray-500 flex items-center gap-0.5">
                      <Users size={10} /> {f.people} 人
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <button type="button" onClick={() => setShowForm((v) => !v)} data-testid="nema-link-file" className="w-full flex items-center justify-center gap-1.5 text-xs py-2 rounded-lg border border-dashed border-gray-300 text-gray-600 hover:border-indigo-400 hover:text-indigo-700">
            <Plus size={12} /> 关联共享文件（自动生成 .nema 侧车）
          </button>

          {showForm && (
            <div data-testid="nema-register-form" className="rounded-lg border border-indigo-200 bg-indigo-50/50 p-2.5 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-indigo-700">注册共享文件（共同编辑 ≥2 人）</span>
                <button type="button" onClick={() => setShowForm(false)} className="text-gray-400 hover:text-gray-600"><X size={12} /></button>
              </div>
              <LocalFolderBrowser onPickFile={(p) => setFilePath(p)} />
              <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="文件路径 (e.g. src/graph/bfs.rs)" className="w-full rounded-lg border border-gray-200 bg-white px-2.5 py-1.5 text-xs font-mono" />
              <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="任务标题 (可选)" className="w-full rounded-lg border border-gray-200 bg-white px-2.5 py-1.5 text-xs" />
              <input value={editorsText} onChange={(e) => setEditorsText(e.target.value)} placeholder="编辑者 (逗号分隔, e.g. 林, Agent A1)" className="w-full rounded-lg border border-gray-200 bg-white px-2.5 py-1.5 text-xs" />
              <button type="button" onClick={handleRegister} className="w-full text-xs py-1.5 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700">
                注册并生成 .nema
              </button>
            </div>
          )}

          <div data-testid="nema-watcher-status" className="text-[10px] text-gray-400 font-mono">
            {nemaWatcher.watching ? "● 文件监听中 (轮询 .nema 变化 → 任务回写)" : "○ 文件监听未启动"}
          </div>

          <p className="text-[11px] text-gray-500 leading-relaxed bg-gray-50 rounded-lg p-2.5 border border-gray-200">
            每个文件可配 .nema 侧车。多人和多个 Agent 优先在 .nema 上并行编辑，汇总后再生成代码。
          </p>
        </div>
      </aside>

      {/* ============ 中: 流水线 + 文件对照 ============ */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4 space-y-3">
        {/* 5 步流水线 */}
        <div data-testid="nema-pipeline" className="flex items-center gap-1.5 text-xs">
          {PIPELINE.map((step, i) => (
            <span key={step} className="flex items-center gap-1.5">
              <span
                className={clsx(
                  "px-2.5 py-1.5 rounded-lg border",
                  i === 1 ? "bg-indigo-600 text-white border-indigo-600 font-semibold" : "bg-white border-gray-200 text-gray-600",
                )}
              >
                {step}
              </span>
              {i < PIPELINE.length - 1 && <span className="text-gray-300">→</span>}
            </span>
          ))}
        </div>

        {/* 当前文件: .nema 规约 + 生成代码 */}
        <div className="text-xs font-semibold text-gray-700">当前文件：bfs.rs</div>
        <div className="grid grid-cols-2 gap-3">
          {/* .nema 规约 */}
          <section data-testid="nema-spec" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
            <header className="px-3 py-2.5 border-b border-gray-200 flex items-center justify-between">
              <span className="font-mono text-xs font-semibold text-gray-800">bfs.rs.nema</span>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-700 font-semibold">权威规约 · REV 3</span>
            </header>
            <div className="font-mono text-xs leading-relaxed">
              {NEMA_LINES.map((line, i) => (
                <div key={i} className={clsx("flex gap-3 px-3 py-0.5 hover:bg-gray-50", line.cls)}>
                  <span className="w-6 text-right text-gray-300 select-none shrink-0">{line.tag}</span>
                  <span className="flex-1 whitespace-pre-wrap break-all">{line.text}</span>
                  {line.who && <span className="text-[10px] text-gray-400 shrink-0 self-center">{line.who}</span>}
                </div>
              ))}
            </div>
          </section>

          {/* 生成代码 */}
          <section data-testid="nema-generated" className="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
            <header className="px-3 py-2.5 border-b border-gray-200 flex items-center justify-between">
              <span className="font-mono text-xs font-semibold text-gray-800">bfs.rs</span>
              <span className="text-[10px] flex items-center gap-1.5">
                <span className="text-gray-500">由 .nema 生成 · SourceMap 对齐</span>
                <span className="px-1.5 py-0.5 rounded bg-amber-100 text-amber-700 font-semibold">与 .nema 有 1 处漂移</span>
              </span>
            </header>
            <div className="font-mono text-xs leading-relaxed">
              {CODE_LINES.map((line, i) => (
                <div key={i} className="flex gap-3 px-3 py-0.5 hover:bg-gray-50">
                  <span className="w-6 text-right text-gray-300 select-none shrink-0">{line.n}</span>
                  <span className="flex-1 whitespace-pre-wrap break-all text-gray-800">{line.text}</span>
                  {line.who && <span className="text-[10px] text-indigo-500 shrink-0 self-center">{line.who}</span>}
                </div>
              ))}
            </div>
          </section>
        </div>

        {/* 租约说明 */}
        <p data-testid="nema-lease-note" className="text-[11px] text-gray-500 flex items-center gap-1.5">
          <Lock size={11} /> 写入需持有文件租约（带递增令牌）；租约过期自动释放，人可随时接管
        </p>

        {/* 协作组 */}
        <div className="grid grid-cols-2 gap-3">
          <section data-testid="nema-group-a" className="bg-white rounded-xl border border-gray-200 shadow-sm p-3">
            <header className="text-xs font-semibold text-gray-800 mb-2">协作组 A · 同一文件 <span className="font-mono text-gray-500">bfs.rs.nema</span>，按段和记录分区并行</header>
            <ul className="space-y-1.5">
              {[
                { who: "A1", tag: "Agent A1", branch: "feat/A1", part: "§1 · 回边判定" },
                { who: "A2", tag: "Agent A2", branch: "feat/A2", part: "§2 · 环路径输出" },
                { who: "林", tag: "林", branch: "直接编辑", part: "深度上限" },
              ].map((m) => (
                <li key={m.who} className="flex items-center gap-2 text-xs">
                  <span className="size-6 rounded-full bg-indigo-100 text-indigo-700 font-bold grid place-items-center text-[10px]">{m.who}</span>
                  <span className="text-gray-700 w-16">{m.tag}</span>
                  <span className="font-mono text-[10px] text-gray-400">{m.branch}</span>
                  <span className="text-gray-500 flex-1 text-right">{m.part}</span>
                </li>
              ))}
            </ul>
          </section>

          <section data-testid="nema-group-b" className="bg-white rounded-xl border border-gray-200 shadow-sm p-3">
            <header className="text-xs font-semibold text-gray-800 mb-2">独立组 B · 不同文件 <span className="text-gray-500 font-normal">各自分支推进，跨文件契约已冻结</span></header>
            <ul className="space-y-1.5">
              {[
                { who: "B1", tag: "Agent B1", branch: "feat/B1", part: "bfs_test.rs" },
                { who: "U", tag: "Ulysses", branch: "直接编辑", part: "relation-detail-design.md" },
              ].map((m) => (
                <li key={m.who} className="flex items-center gap-2 text-xs">
                  <span className="size-6 rounded-full bg-gray-100 text-gray-600 font-bold grid place-items-center text-[10px]">{m.who}</span>
                  <span className="text-gray-700 w-16">{m.tag}</span>
                  <span className="font-mono text-[10px] text-gray-400">{m.branch}</span>
                  <span className="text-gray-500 flex-1 text-right font-mono">{m.part}</span>
                </li>
              ))}
            </ul>
            <p className="mt-2 text-[11px] text-gray-500 leading-relaxed">
              路由 Agent 按文件拆分：同一文件的协作组按 .nema 的段、记录分区并行编辑，独立组在各自分支推进，跨文件调用先冻结契约。
            </p>
          </section>
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          <button type="button" data-testid="nema-advance-merge" className="text-xs px-3 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 flex items-center gap-1.5">
            <Send size={12} /> 推进到 ③ 语义合并
          </button>
          <button type="button" className="text-xs px-3 py-2 rounded-lg border border-gray-300 text-gray-700 hover:border-indigo-400 flex items-center gap-1.5">
            <UserPlus size={12} /> 邀请 Agent 加入
          </button>
          <button type="button" className="text-xs px-3 py-2 rounded-lg border border-gray-300 text-gray-700 hover:border-indigo-400 flex items-center gap-1.5">
            <Terminal size={12} /> 在 CLI 中打开
          </button>
        </div>
      </main>
    </div>
  );
}
