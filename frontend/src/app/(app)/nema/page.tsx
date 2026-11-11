/*
@cypher schema=1 source_sha256=60a91845bb4b886fe72d20f98848e8747ea4abcfedf0e9be9562c44f9b51d8c7
MERGE (self:File {path:"frontend/src/app/(app)/nema/page.tsx"})
MERGE (page:Function {name:"NemaPage", language:"typescript", visibility:"public"})
MERGE (register:Function {name:"SharedFileForm", language:"typescript", visibility:"private"})
MERGE (list:Function {name:"NemaTaskList", language:"typescript", visibility:"private"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(list)
@endcypher
*/
"use client";

// =====================================================================
// /nema — 文件即任务 (per 2026-11-11 用户发令)
// 共同编辑的文件 (editors ≥ 2) 自动产生 .nema 侧车文件;
// .nema 可作为 Star 任务打开 (创建 WorkItem + 关联文档)。
// =====================================================================

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { FileText, FilePlus2, Play, Trash2, Users, ChevronDown, ChevronUp } from "lucide-react";
import { useNemaStore } from "@/lib/nema/nemaStore";
import { parseNema, taskViewFromDocument } from "@/lib/nema/nema";
import { useTranslation } from "@/lib/i18n";

export default function NemaPage() {
  const { t } = useTranslation();
  const router = useRouter();
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const registerSharedFile = useNemaStore((s) => s.registerSharedFile);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);
  const removeNemaTask = useNemaStore((s) => s.removeNemaTask);

  const [filePath, setFilePath] = useState("");
  const [title, setTitle] = useState("");
  const [editorsText, setEditorsText] = useState("");
  const [purpose, setPurpose] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const boundTasks = useMemo(() => nemaTasks.filter((x) => x.workItemId), [nemaTasks]);

  const handleRegister = () => {
    const editors = editorsText
      .split(/[,，\s]+/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (!filePath.trim() || !title.trim() || editors.length === 0) {
      setNotice("filePath / title / editors 至少填一项");
      return;
    }
    const entry = registerSharedFile({ filePath: filePath.trim(), title: title.trim(), editors, purpose });
    if (entry === null) {
      setNotice(`「${filePath}」当前编辑者 < 2 — 共同编辑才自动产生 .nema (文件即任务规则)`);
    } else {
      setNotice(`✓ 已自动产生 .nema 侧车: ${entry.nemaPath} (编辑者 ${editors.length} 人)`);
      setFilePath("");
      setTitle("");
      setEditorsText("");
      setPurpose("");
    }
  };

  const handleOpenAsTask = (nemaId: string) => {
    const workItemId = openNemaAsStarTask(nemaId);
    if (workItemId) {
      router.push(`/work-item/${workItemId}`);
    }
  };

  return (
    <div className="max-w-6xl mx-auto">
      <PageHeader
        title="NEMA · 文件即任务"
        subtitle="共同编辑的文件自动产生 .nema 侧车 — .nema 可以作为 Star 任务打开"
        icon={<FileText className="text-accent" size={20} />}
        track="D"
        count={`${nemaTasks.length} .nema`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label=".nema 任务" value={nemaTasks.length} tone="info" />
        <Stat label="已绑定 Star 任务" value={boundTasks.length} tone="ok" />
        <Stat label="自动生成规则" value="≥2 编辑者" tone="default" hint="共同编辑触发" />
        <Stat label="格式" value=".NEMA v1" tone="default" hint="nema-sigma 规范子集" />
      </div>

      {/* 注册共同编辑文件 */}
      <div className="card mb-5" data-testid="nema-register-form">
        <SectionTitle>共同编辑文件登记 (自动 .nema 生成)</SectionTitle>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mt-3">
          <input
            data-testid="nema-input-path"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            placeholder="文件路径 docs/design/auth.md"
            className="input text-xs"
          />
          <input
            data-testid="nema-input-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="任务标题 (What)"
            className="input text-xs"
          />
          <input
            data-testid="nema-input-editors"
            value={editorsText}
            onChange={(e) => setEditorsText(e.target.value)}
            placeholder="共同编辑者 (逗号/空格分隔, ≥2 触发)"
            className="input text-xs"
          />
          <input
            data-testid="nema-input-purpose"
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            placeholder="目的 (Why, 可选)"
            className="input text-xs"
          />
        </div>
        <div className="flex items-center gap-3 mt-3">
          <button data-testid="nema-register-btn" className="btn text-xs" onClick={handleRegister}>
            <FilePlus2 size={12} /> 登记文件 (自动生成 .nema)
          </button>
          {notice && (
            <span
              data-testid="nema-notice"
              className={`text-xs font-mono ${notice.startsWith("✓") ? "text-ok" : "text-warn"}`}
            >
              {notice}
            </span>
          )}
        </div>
        <div className="text-[10px] text-ink-mute font-mono mt-2">
          规则: editors ≥ 2 才自动产生 .nema; 单人编辑不产生 (per 2026-11-11 用户发令)。
        </div>
      </div>

      {/* .nema 任务列表 */}
      <div className="card" data-testid="nema-task-list">
        <SectionTitle>.nema 任务 ({nemaTasks.length})</SectionTitle>
        {nemaTasks.length === 0 ? (
          <div className="text-xs text-ink-mute py-6 text-center">
            暂无共同编辑文件。登记一个 ≥2 编辑者的文件后, 自动产生 .nema 侧车。
          </div>
        ) : (
          <ul className="divide-y divide-line/50 mt-2">
            {nemaTasks.map((entry) => {
              const view = taskViewFromDocument(parseNema(entry.source));
              const expanded = expandedId === entry.id;
              return (
                <li key={entry.id} className="py-2" data-testid={`nema-task-${entry.id}`}>
                  <div className="flex items-center gap-2 flex-wrap">
                    <button
                      className="btn-ghost p-1 rounded"
                      onClick={() => setExpandedId(expanded ? null : entry.id)}
                      aria-label="toggle nema source"
                    >
                      {expanded ? <ChevronUp size={13} /> : <ChevronDown size={13} />}
                    </button>
                    <FileText size={13} className="text-cyan-400 shrink-0" />
                    <span className="text-sm font-semibold text-ink">{entry.title}</span>
                    <span className="text-[10px] font-mono text-ink-dim">{entry.nemaPath}</span>
                    <span className="flex items-center gap-1 text-[10px] font-mono text-violet-300">
                      <Users size={10} /> {entry.editors.join(", ")}
                    </span>
                    <div className="ml-auto flex items-center gap-1.5">
                      {entry.workItemId ? (
                        <button
                          className="btn-secondary text-[10px] px-2 py-1"
                          onClick={() => router.push(`/work-item/${entry.workItemId}`)}
                          data-testid={`nema-open-bound-${entry.id}`}
                        >
                          已绑定 Star 任务 →
                        </button>
                      ) : (
                        <button
                          className="btn text-[10px] px-2 py-1"
                          onClick={() => handleOpenAsTask(entry.id)}
                          data-testid={`nema-open-task-${entry.id}`}
                        >
                          <Play size={10} /> 打开为 Star 任务
                        </button>
                      )}
                      <button
                        className="btn-ghost p-1 rounded hover:text-red-400"
                        onClick={() => removeNemaTask(entry.id)}
                        aria-label="remove nema task"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                  <div className="text-[10px] font-mono text-ink-mute mt-1 pl-7">
                    5W1H: {view.what} · {view.why} · {view.how} · {view.who.join(",")} · {view.when} · {view.where}
                  </div>
                  {expanded && (
                    <pre
                      data-testid={`nema-source-${entry.id}`}
                      className="mt-2 ml-7 p-3 rounded border border-line bg-bg-soft/60 text-[10px] font-mono text-ink-dim overflow-x-auto"
                    >
                      {entry.source}
                    </pre>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
