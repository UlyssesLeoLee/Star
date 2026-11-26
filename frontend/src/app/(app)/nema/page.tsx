/*
@cypher schema=1 source_sha256=237b390b0c482698e888a0682cc4893e5ae7b330db4e598c297345f5a3f16661
MERGE (self:File {path:"frontend/src/app/(app)/nema/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::NemaPage",kind:"component"})
MERGE (load_profiles:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::loadInterpreterProfiles",kind:"effect"})
MERGE (format_preview:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::formatProfilePreview",kind:"effect"})
MERGE (add_editor_handler:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::handleAddEditor",kind:"function"})
MERGE (remove_editor_handler:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::handleRemoveEditor",kind:"function"})
MERGE (trail_loader:Symbol {id:"frontend/src/app/(app)/nema/page.tsx::loadEditTrail",kind:"effect"})
MERGE (nema:File {path:"frontend/src/lib/nema/nema.ts"})
MERGE (store:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (watcher:File {path:"frontend/src/lib/nema/nemaWatcher.ts"})
MERGE (collab:File {path:"frontend/src/lib/nema/nemaCollaboration.ts"})
MERGE (main_store:File {path:"frontend/src/lib/store.ts"})
MERGE (status_notice:File {path:"frontend/src/components/status/StatusNotice.tsx"})
MERGE (update_editors:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::updateNemaEditors",kind:"function"})
MERGE (read_trail:Symbol {id:"frontend/src/lib/nema/nema.ts::readNemaEditTrail",kind:"function"})
MERGE (presence:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::nemaEditorPresence",kind:"function"})
MERGE (resolve_actor:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::resolveNemaActor",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:DEFINES]->(add_editor_handler)
MERGE (self)-[:DEFINES]->(remove_editor_handler)
MERGE (self)-[:DEFINES]->(trail_loader)
MERGE (self)-[:IMPORTS]->(nema)
MERGE (self)-[:IMPORTS]->(store)
MERGE (self)-[:IMPORTS]->(watcher)
MERGE (self)-[:IMPORTS]->(collab)
MERGE (self)-[:IMPORTS]->(main_store)
MERGE (self)-[:IMPORTS]->(status_notice)
MERGE (page)-[:CALLS]->(load_profiles)
MERGE (page)-[:CALLS]->(format_preview)
MERGE (page)-[:CALLS]->(store)
MERGE (page)-[:CALLS]->(watcher)
MERGE (page)-[:CALLS]->(presence)
MERGE (page)-[:CALLS]->(resolve_actor)
MERGE (page)-[:CALLS]->(status_notice)
MERGE (add_editor_handler)-[:CALLS]->(update_editors)
MERGE (remove_editor_handler)-[:CALLS]->(update_editors)
MERGE (trail_loader)-[:CALLS]->(read_trail)
MERGE (load_profiles)-[:CALLS]->(nema)
MERGE (format_preview)-[:CALLS]->(nema)
@endcypher
*/
// =====================================================================
// /nema — 2b · 协同（文件即任务 · 多人多 Agent）
// per 2026-11-14 参考 mockup: Docs.dc.html
// 结构: 左 工作集(任务卡/文件列表) + 中 多 Agent 流水线(5 步) +
//      当前文件 .nema 规约 / 生成代码对照 + 协作组(同一文件/独立组) + 租约说明
// =====================================================================
"use client";

import { useEffect, useState } from "react";
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
import {
  listNemaInterpreterProfilesWasm,
  loadNemaWasm,
  nemaDocumentForSharedFile,
  NEMA_DEFAULT_NEW_DOCUMENT_REVISION,
  readNemaEditTrail,
  serializeNemaWithProfileWasm,
  type NemaEditTraceEntry,
  type NemaInterpreterProfile,
} from "@/lib/nema/nema";
import {
  nemaEditorPresence,
  resolveNemaActor,
  NEMA_REALTIME_COLLABORATION_AVAILABLE,
} from "@/lib/nema/nemaCollaboration";
import { useStore as useMainStore } from "@/lib/store";
import { StatusNotice } from "@/components/status/StatusNotice";
import { LocalFolderBrowser } from "@/components/nema/LocalFolderBrowser";
import MonacoEditor from "@/components/editor/MonacoEditor";
import { useIdeKernel } from "@/hooks/useIdeKernel";

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
  const [profiles, setProfiles] = useState<NemaInterpreterProfile[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<NemaInterpreterProfile | null>(null);
  const [profileError, setProfileError] = useState<string | null>(null);
  const [previewSource, setPreviewSource] = useState("");
  // per 2026-11-21: 接 ide1.0 网页版编辑核心 (MonacoEditor + useIdeKernel) —
  // 生成代码可编辑, .nema 规约可编辑; 内核未连接时诚实降级为本地预览。
  const [generatedCode, setGeneratedCode] = useState(() => CODE_LINES.map((l) => l.text).join("\n"));
  const ideKernel = useIdeKernel({ enabled: true });

  // ---- 共同编辑区块状态 (per 2026-11-17 用户发令) ----
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [newEditorName, setNewEditorName] = useState("");
  const [editTrail, setEditTrail] = useState<NemaEditTraceEntry[] | null>(null);
  const [collaborationError, setCollaborationError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void (async () => {
      if (!(await loadNemaWasm())) throw new Error("Rust NEMA interpreter could not be loaded");
      const installed = await listNemaInterpreterProfilesWasm();
      if (!active) return;
      setProfiles(installed);
      setSelectedProfile(
        installed.find((profile) => profile.grammar_revision === NEMA_DEFAULT_NEW_DOCUMENT_REVISION) ?? null,
      );
      if (!installed.some((profile) => profile.grammar_revision === NEMA_DEFAULT_NEW_DOCUMENT_REVISION)) {
        setProfileError(`未安装默认语法 profile: ${NEMA_DEFAULT_NEW_DOCUMENT_REVISION}`);
      }
    })().catch((error: unknown) => {
      if (active) setProfileError(error instanceof Error ? error.message : String(error));
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (!selectedProfile) {
      setPreviewSource("");
      return;
    }
    let active = true;
    void (async () => {
      const document = nemaDocumentForSharedFile(
        {
          filePath: "src/graph/bfs.rs",
          title: "BFS 环检测",
          editors: ["Agent A1", "Agent A2", "林"],
          purpose: "防止遍历死循环",
          method: "访问集 + 路径栈判回边",
          environment: "Worktree",
        },
        new Date("2026-11-15T08:00:00.000Z"),
      );
      const source = await serializeNemaWithProfileWasm(document, selectedProfile);
      if (active) setPreviewSource(source);
    })().catch((error: unknown) => {
      if (active) setProfileError(error instanceof Error ? error.message : String(error));
    });
    return () => { active = false; };
  }, [selectedProfile]);

  // ---- 真实功能接线 ----
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const registerSharedFile = useNemaStore((s) => s.registerSharedFile);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);
  const updateNemaEditors = useNemaStore((s) => s.updateNemaEditors);
  const nemaWatcher = useNemaWatcher([""], undefined, selectedProfile);
  // 复用既有身份机制 (store.identities) 解析当前编辑者, 不新造用户体系
  const identities = useMainStore((s) => s.identities);
  const actorName = resolveNemaActor(identities);

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

  const handleRegister = async () => {
    const editors = editorsText.split(/[,，\s]+/).map((s) => s.trim()).filter(Boolean);
    if (!filePath.trim() || editors.length === 0 || !selectedProfile) return;
    try {
      await registerSharedFile({ filePath: filePath.trim(), title: title.trim(), editors }, selectedProfile);
      setShowForm(false);
      setFilePath("");
      setTitle("");
      setEditorsText("");
      setProfileError(null);
    } catch (error) {
      setProfileError(error instanceof Error ? error.message : String(error));
    }
  };

  // ---- 共同编辑区块接线 ----
  const selectedTask = nemaTasks.find((t) => t.id === selectedTaskId) ?? nemaTasks[0] ?? null;

  // 编辑痕迹: 从 pin 的 strict profile 实时解析 .nema 源 (无 profile 的 legacy 条目 → null)
  useEffect(() => {
    if (!selectedTask) {
      setEditTrail(null);
      return;
    }
    let active = true;
    setEditTrail(null);
    void readNemaEditTrail(selectedTask.source, selectedTask.interpreterProfile)
      .then((trail) => {
        if (active) setEditTrail(trail);
      })
      .catch(() => {
        if (active) setEditTrail(null);
      });
    return () => {
      active = false;
    };
  }, [selectedTask?.id, selectedTask?.source]);

  const handleAddEditor = async () => {
    const name = newEditorName.trim();
    if (!selectedTask || !name) return;
    setCollaborationError(null);
    try {
      const result = await updateNemaEditors(selectedTask.id, {
        editors: [...selectedTask.editors.filter((e) => e !== name), name],
        actor: actorName,
      });
      if (result.action === "refused") setCollaborationError(result.reason);
      else setNewEditorName("");
    } catch (error) {
      setCollaborationError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleRemoveEditor = async (editor: string) => {
    if (!selectedTask) return;
    setCollaborationError(null);
    try {
      const result = await updateNemaEditors(selectedTask.id, {
        editors: selectedTask.editors.filter((e) => e !== editor),
        actor: actorName,
      });
      if (result.action === "refused") setCollaborationError(result.reason);
    } catch (error) {
      setCollaborationError(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="flex h-full bg-bg">
      {/* ============ 左: 工作集 ============ */}
      <aside data-testid="nema-workset" className="w-72 shrink-0 border-r border-line bg-bg-card overflow-y-auto">
        <div className="px-3 py-3 border-b border-line">
          <div className="text-[11px] text-ink-dim">隶属当前 Worktree · Branch</div>
          <div className="mt-0.5 text-xs font-mono font-semibold text-ink flex items-center gap-1.5">
            <GitBranch size={12} className="text-accent" /> wt-relation-bfs · feat/relation-bfs
          </div>
        </div>

        {/* 按任务卡 / 其他 */}
        <div className="px-3 py-2 flex gap-1">
          {(["task", "other"] as const).map((s) => (
            <button
              key={s}
              type="button"
              data-active={scope === s ? "true" : "false"}
              onClick={() => setScope(s)}
              className={clsx(
                "ui-3d-tab flex-1 text-xs px-2 py-1.5 rounded-lg border transition-colors",
                scope === s ? "border-accent text-accent bg-accent/10 font-semibold" : "border-line text-ink-dim",
              )}
            >
              {s === "task" ? "按任务卡" : "其他"}
            </button>
          ))}
        </div>

        <div className="px-3 space-y-3 pb-4">
          {/* 任务卡选择 */}
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5">任务卡</div>
            {visibleTasks.length === 0 ? (
              <p className="text-[11px] text-ink-mute px-1 py-2">暂无{scope === "task" ? "已开卡" : "未开卡"}的 .nema 文件</p>
            ) : (
              <ul className="space-y-1">
                {visibleTasks.map((task) => (
                  <li
                    key={task.id}
                    data-testid={`nema-task-${task.id}`}
                    className={clsx(
                      "flex items-center gap-2 px-2.5 py-2 rounded-lg border text-xs",
                      task.status ? "border-accent/50 bg-accent/10" : "border-line hover:border-line",
                    )}
                  >
                    <span className="font-mono font-semibold text-ink-dim truncate w-14">{task.id}</span>
                    <span className="flex-1 truncate text-ink">{task.title}</span>
                    {task.status && <span className="text-[10px] text-accent">{task.status}</span>}
                    {task.status && (
                      <button type="button" onClick={() => router.push(`/work-item/${task.id}`)} className="text-[10px] text-ink-mute hover:text-accent">
                        打开
                      </button>
                    )}
                  </li>
                ))}
              </ul>
            )}
            <div className="mt-1.5 text-[11px] font-mono text-accent flex items-center gap-1">
              <FileCode2 size={11} /> ipa-s05-002 ソースコード作成 · 文件即任务
            </div>
          </div>

          {/* 文件列表 */}
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5 flex items-center justify-between">
              <span>代码 <span className="text-ink-mute">4</span></span>
            </div>
            {codeFiles.length === 0 ? (
              <p className="text-[11px] text-ink-mute px-1 py-2">暂无代码文件 — 关联共享文件后自动生成 .nema 侧车</p>
            ) : (
              <ul className="space-y-1">
                {codeFiles.map((f) => (
                  <li key={f.name} data-testid={`nema-file-${f.name}`} className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-line text-xs hover:border-accent/50">
                    <FileCode2 size={12} className="text-ink-mute shrink-0" />
                    <span className="flex-1 truncate font-mono text-ink">{f.name}</span>
                    <span className="text-[10px] text-ink-dim flex items-center gap-0.5">
                      <Users size={10} /> {f.people} 人
                    </span>
                    {!f.task.workItemId && (
                      <button
                        type="button"
                        data-testid={`nema-open-task-${f.name}`}
                        onClick={async () => {
                          const wid = await openNemaAsStarTask(f.task.id);
                          if (wid) router.push(`/work-item/${wid}`);
                        }}
                        className="btn-primary cel-btn-3d !min-h-0 !px-2 !py-0.5 text-[10px]"
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
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5">文档与表格 <span className="text-ink-mute">2</span></div>
            {docFiles.length === 0 ? (
              <p className="text-[11px] text-ink-mute px-1 py-2">暂无文档/表格</p>
            ) : (
              <ul className="space-y-1">
                {docFiles.map((f) => (
                  <li key={f.name} data-testid={`nema-file-${f.name}`} className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-line text-xs hover:border-accent/50">
                    <FileText size={12} className="text-ink-mute shrink-0" />
                    <span className="flex-1 truncate font-mono text-ink">{f.name}</span>
                    <span className="text-[10px] text-ink-dim flex items-center gap-0.5">
                      <Users size={10} /> {f.people} 人
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <button type="button" onClick={() => setShowForm((v) => !v)} data-testid="nema-link-file" className="btn cel-btn-3d w-full flex items-center justify-center gap-1.5 text-xs !py-2">
            <Plus size={12} /> 关联共享文件（自动生成 .nema 侧车）
          </button>

          {showForm && (
            <div data-testid="nema-register-form" className="rounded-lg border border-accent/40 bg-accent/10/50 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-accent">注册共享文件（共同编辑 ≥2 人）</span>
                <button type="button" onClick={() => setShowForm(false)} className="text-ink-mute hover:text-ink-dim"><X size={12} /></button>
              </div>
              <label className="block space-y-1 text-[11px] text-ink-dim">
                <span>解释器语法版本（新导入文件也使用此版本；已登记文件保持原 profile）</span>
                <select
                  aria-label="NEMA interpreter profile"
                  value={selectedProfile?.grammar_revision ?? ""}
                  onChange={(event) => setSelectedProfile(profiles.find((profile) => profile.grammar_revision === event.target.value) ?? null)}
                  className="w-full rounded-lg border border-line bg-bg-card px-2.5 py-1.5 text-xs font-mono"
                >
                  <option value="" disabled>选择已安装的 Rust profile</option>
                  {profiles.map((profile) => (
                    <option key={`${profile.plugin_id}:${profile.plugin_version}:${profile.grammar_revision}`} value={profile.grammar_revision}>
                      {profile.grammar_revision} · {profile.plugin_id}@{profile.plugin_version}
                    </option>
                  ))}
                </select>
                {profileError && <span role="status" className="block text-err">{profileError}</span>}
              </label>
              <LocalFolderBrowser onPickFile={(p) => setFilePath(p)} />
              <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="文件路径 (e.g. src/graph/bfs.rs)" className="w-full rounded-lg border border-line bg-bg-card px-2.5 py-1.5 text-xs font-mono" />
              <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="任务标题 (可选)" className="w-full rounded-lg border border-line bg-bg-card px-2.5 py-1.5 text-xs" />
              <input value={editorsText} onChange={(e) => setEditorsText(e.target.value)} placeholder="编辑者 (逗号分隔, e.g. 林, Agent A1)" className="w-full rounded-lg border border-line bg-bg-card px-2.5 py-1.5 text-xs" />
              <button type="button" onClick={handleRegister} disabled={!selectedProfile} className="btn-primary cel-btn-3d w-full !min-h-[34px] text-xs !py-1.5 disabled:cursor-not-allowed">
                注册并生成 .nema
              </button>
            </div>
          )}

          <div data-testid="nema-watcher-status" className="text-[10px] text-ink-mute font-mono">
            {nemaWatcher.watching
              ? `● 文件监听中 · ${selectedProfile?.grammar_revision ?? "profile unavailable"}`
              : profileError ?? "○ 正在加载 Rust NEMA interpreter profile"}
          </div>
          {nemaWatcher.lastResult?.errors.slice(0, 3).map((error) => (
            <div key={error} role="status" className="text-[10px] text-err font-mono truncate">{error}</div>
          ))}

          <p className="text-xs text-ink-dim leading-relaxed bg-bg rounded-lg p-3 border border-line">
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
                  i === 1 ? "bg-accent text-white border-accent font-semibold" : "bg-bg-card border-line text-ink-dim",
                )}
              >
                {step}
              </span>
              {i < PIPELINE.length - 1 && <span className="text-ink-mute">→</span>}
            </span>
          ))}
        </div>

        {/* 当前文件: .nema 规约 + 生成代码 */}
        <div className="text-xs font-semibold text-ink-dim">当前文件：bfs.rs</div>
        <div className="grid grid-cols-2 gap-3">
          {/* .nema 规约 */}
          <section data-testid="nema-spec" className="bg-bg-card rounded-xl border border-line shadow-soft overflow-hidden">
            <header className="px-3 py-2.5 border-b border-line flex items-center justify-between">
              <span className="font-mono text-xs font-semibold text-ink">bfs.rs.nema</span>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-accent/15 text-accent font-semibold">
                Rust profile · {selectedProfile?.grammar_revision ?? "loading"}
              </span>
            </header>
            <MonacoEditor
              value={previewSource}
              language="markdown"
              path="bfs.rs.nema"
              height={320}
              testId="nema-spec-editor"
              onChange={(next) => setPreviewSource(next)}
            />
          </section>

          {/* 生成代码 */}
          <section data-testid="nema-generated" className="bg-bg-card rounded-xl border border-line shadow-soft overflow-hidden">
            <header className="px-3 py-2.5 border-b border-line flex items-center justify-between">
              <span className="font-mono text-xs font-semibold text-ink">bfs.rs</span>
              <span className="text-[10px] flex items-center gap-1.5">
                <span className="text-ink-dim">由 .nema 生成 · SourceMap 对齐</span>
                <span className="px-1.5 py-0.5 rounded bg-warn/10 text-warn font-semibold">与 .nema 有 1 处漂移</span>
              </span>
            </header>
            <MonacoEditor
              value={generatedCode}
              language="rust"
              path="bfs.rs"
              height={320}
              testId="nema-generated-editor"
              onChange={(next) => { setGeneratedCode(next); ideKernel.syncText(next); }}
            />
          </section>
        </div>

        {/* 租约说明 */}
        <p data-testid="nema-lease-note" className="text-[11px] text-ink-dim flex items-center gap-1.5">
          <Lock size={11} /> 写入需持有文件租约（带递增令牌）；租约过期自动释放，人可随时接管
        </p>
        <p data-testid="nema-ide-kernel-status" className="text-[10px] font-mono text-ink-mute">
          {ideKernel.mode === "connected"
            ? `● ide-kernel 已连接 · ${ideKernel.kernelVersion ?? ""} (编辑同步到内核)`
            : ideKernel.mode === "error"
              ? `○ ide-kernel 不可用 (${ideKernel.error ?? "连接失败"}) — 本地预览编辑, 未同步内核`
              : "○ ide-kernel 探测中… 未连接时为本地预览编辑"}
        </p>

        {/* 协作组 */}
        <div className="grid grid-cols-2 gap-3">
          <section data-testid="nema-group-a" className="bg-bg-card rounded-xl border border-line shadow-soft p-3">
            <header className="text-xs font-semibold text-ink mb-2">协作组 A · 同一文件 <span className="font-mono text-ink-dim">bfs.rs.nema</span>，按段和记录分区并行 <span className="text-[10px] text-warn font-normal">(示意, 实时协作未接入)</span></header>
            <ul className="space-y-1.5">
              {[
                { who: "A1", tag: "Agent A1", branch: "feat/A1", part: "§1 · 回边判定" },
                { who: "A2", tag: "Agent A2", branch: "feat/A2", part: "§2 · 环路径输出" },
                { who: "林", tag: "林", branch: "直接编辑", part: "深度上限" },
              ].map((m) => (
                <li key={m.who} className="flex items-center gap-2 text-xs">
                  <span className="size-6 rounded-full bg-accent/15 text-accent font-bold grid place-items-center text-[10px]">{m.who}</span>
                  <span className="text-ink-dim w-16">{m.tag}</span>
                  <span className="font-mono text-[10px] text-ink-mute">{m.branch}</span>
                  <span className="text-ink-dim flex-1 text-right">{m.part}</span>
                </li>
              ))}
            </ul>
          </section>

          <section data-testid="nema-group-b" className="bg-bg-card rounded-xl border border-line shadow-soft p-3">
            <header className="text-xs font-semibold text-ink mb-2">独立组 B · 不同文件 <span className="text-ink-dim font-normal">各自分支推进，跨文件契约已冻结</span> <span className="text-[10px] text-warn font-normal">(示意, 实时协作未接入)</span></header>
            <ul className="space-y-1.5">
              {[
                { who: "B1", tag: "Agent B1", branch: "feat/B1", part: "bfs_test.rs" },
                { who: "U", tag: "Ulysses", branch: "直接编辑", part: "relation-detail-design.md" },
              ].map((m) => (
                <li key={m.who} className="flex items-center gap-2 text-xs">
                  <span className="size-6 rounded-full bg-bg-soft text-ink-dim font-bold grid place-items-center text-[10px]">{m.who}</span>
                  <span className="text-ink-dim w-16">{m.tag}</span>
                  <span className="font-mono text-[10px] text-ink-mute">{m.branch}</span>
                  <span className="text-ink-dim flex-1 text-right font-mono">{m.part}</span>
                </li>
              ))}
            </ul>
            <p className="mt-2 text-[11px] text-ink-dim leading-relaxed">
              路由 Agent 按文件拆分：同一文件的协作组按 .nema 的段、记录分区并行编辑，独立组在各自分支推进，跨文件调用先冻结契约。
            </p>
          </section>
        </div>

        {/* ============ 共同编辑区块 (真实 store 数据, per 2026-11-17 用户发令) ============ */}
        <section data-testid="nema-collaboration" className="bg-bg-card rounded-xl border border-line shadow-soft p-3">
          <header className="flex items-center justify-between mb-2 gap-2">
            <span className="text-xs font-semibold text-ink">共同编辑 · 编辑者登记与痕迹</span>
            {!NEMA_REALTIME_COLLABORATION_AVAILABLE && (
              <StatusNotice compact state="disconnected" title="实时协作未接入" />
            )}
          </header>
          {!selectedTask ? (
            <p data-testid="nema-collaboration-empty" className="text-[11px] text-ink-mute">
              暂无共同编辑文件 — 先关联共享文件 (编辑者 ≥ 2) 生成 .nema 侧车
            </p>
          ) : (
            <div className="space-y-2">
              <div className="flex items-center gap-1.5 flex-wrap">
                <span className="text-[10px] text-ink-mute">任务</span>
                {nemaTasks.map((task) => (
                  <button
                    key={task.id}
                    type="button"
                    data-testid={`nema-collab-select-${task.filePath}`}
                    data-active={task.id === selectedTask.id ? "true" : "false"}
                    onClick={() => setSelectedTaskId(task.id)}
                    className={clsx(
                      "ui-3d-chip text-[10px] px-2 py-0.5 rounded-full border",
                      task.id === selectedTask.id
                        ? "border-accent bg-accent/10 text-accent"
                        : "border-line text-ink-dim hover:border-accent/50",
                    )}
                  >
                    {task.filePath.split("/").pop() ?? task.filePath}
                  </button>
                ))}
              </div>
              <ul data-testid="nema-editor-list" className="space-y-1">
                {selectedTask.editors.map((editor) => {
                  const presence = nemaEditorPresence(selectedTask.editors, editTrail ?? [], editor);
                  const identity = identities.find((i) => i.display_name === editor);
                  return (
                    <li
                      key={editor}
                      data-testid={`nema-editor-${editor}`}
                      className="flex items-center gap-2 text-xs border border-line rounded-lg px-2 py-1"
                    >
                      <span className="size-5 rounded-full bg-accent/15 text-accent font-bold grid place-items-center text-[10px] shrink-0">
                        {editor.slice(0, 1).toUpperCase()}
                      </span>
                      <span className="text-ink truncate">{editor}</span>
                      {identity && (
                        <span
                          className="text-[10px] font-mono text-ink-mute shrink-0"
                          title={`${identity.email} · ${identity.type ?? "human"}`}
                        >
                          本地身份 {identity.id}
                        </span>
                      )}
                      <span
                        data-testid={`nema-presence-${editor}`}
                        title={`来源: ${presence.source}`}
                        className={clsx(
                          "ml-auto rounded-full border px-2 py-0.5 text-[10px] shrink-0",
                          presence.kind === "editing" && "border-ok/40 bg-ok/10 text-ok",
                          presence.kind === "offline" && "border-warn/40 bg-warn/10 text-warn",
                          presence.kind === "not_started" && "border-line text-ink-mute",
                          presence.kind === "unavailable" && "border-warn/30 bg-warn/5 text-warn",
                        )}
                      >
                        {presence.label}
                      </span>
                      <button
                        type="button"
                        data-testid={`nema-remove-editor-${editor}`}
                        onClick={() => void handleRemoveEditor(editor)}
                        className="text-[10px] text-ink-mute hover:text-err shrink-0"
                        title="从共同编辑者中移除"
                      >
                        移除
                      </button>
                    </li>
                  );
                })}
              </ul>
              <div className="flex items-center gap-1.5">
                <input
                  data-testid="nema-add-editor-input"
                  value={newEditorName}
                  onChange={(e) => setNewEditorName(e.target.value)}
                  placeholder="添加编辑者 (姓名或身份显示名)"
                  className="flex-1 rounded-lg border border-line bg-bg px-2 py-1 text-[11px] text-ink placeholder:text-ink-mute outline-none focus:border-accent"
                />
                <button
                  type="button"
                  data-testid="nema-add-editor"
                  onClick={() => void handleAddEditor()}
                  disabled={!newEditorName.trim()}
                  className="btn-primary cel-btn-3d !min-h-[30px] text-[11px] !px-3 !py-1 disabled:cursor-not-allowed"
                >
                  添加
                </button>
              </div>
              {collaborationError && (
                <p data-testid="nema-collaboration-error" role="status" className="text-[10px] text-err">
                  {collaborationError}
                </p>
              )}
              <div data-testid="nema-edit-trail" className="rounded-lg border border-line bg-bg-soft/50 p-3">
                <div className="text-[10px] font-mono text-ink-mute mb-1">
                  编辑痕迹 (.nema 侧车 Field/Value 表, strict v2)
                </div>
                {selectedTask.legacyFormat || !selectedTask.interpreterProfile ? (
                  <p className="text-[10px] text-ink-mute">legacy 格式条目 — 无 strict 痕迹表 (不静默升级)</p>
                ) : editTrail === null ? (
                  <p className="text-[10px] text-ink-mute">解析中…</p>
                ) : editTrail.length === 0 ? (
                  <p className="text-[10px] text-ink-mute">暂无编辑痕迹</p>
                ) : (
                  <ul className="space-y-0.5">
                    {editTrail.map((entry, index) => (
                      <li key={index} className="text-[10px] font-mono text-ink-dim">
                        {entry.editor} @ {entry.edited_at} — {entry.change}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          )}
        </section>

        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          <button type="button" data-testid="nema-advance-merge" className="btn-primary cel-btn-3d !min-h-[36px] text-xs !px-3.5 !py-2 flex items-center gap-1.5">
            <Send size={12} /> 推进到 ③ 语义合并
          </button>
          <button type="button" className="btn cel-btn-3d !min-h-[36px] text-xs !px-3.5 !py-2 flex items-center gap-1.5">
            <UserPlus size={12} /> 邀请 Agent 加入
          </button>
          <button type="button" className="btn cel-btn-3d !min-h-[36px] text-xs !px-3.5 !py-2 flex items-center gap-1.5">
            <Terminal size={12} /> 在 CLI 中打开
          </button>
        </div>
      </main>
    </div>
  );
}
