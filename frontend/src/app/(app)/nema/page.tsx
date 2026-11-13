/*
@cypher schema=1 source_sha256=9ebd61c9cb4f71ce835a901a69f3bfdb8f3cd856fb802ef895f72181f9f07363
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
import { useNemaWatcher } from "@/lib/nema/nemaWatcher";
import { parseNema, taskViewFromDocument } from "@/lib/nema/nema";
import { LocalFolderBrowser } from "@/components/nema/LocalFolderBrowser";
import { useTranslation } from "@/lib/i18n";

export default function NemaPage() {
  const { t, tx } = useTranslation();
  const router = useRouter();
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const registerSharedFile = useNemaStore((s) => s.registerSharedFile);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);
  const removeNemaTask = useNemaStore((s) => s.removeNemaTask);
  // 文件系统监听 (per WBS 8.4.4): 本地模式轮询 workspace-files 检测 .nema 变化
  const nemaWatcher = useNemaWatcher([""]);

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
      setNotice(t.nema.formNotice);
      return;
    }
    const entry = registerSharedFile({ filePath: filePath.trim(), title: title.trim(), editors, purpose });
    if (entry === null) {
      setNotice(tx(t.nema.coeditRule, { filePath }));
    } else {
      setNotice(tx(t.nema.sidecarCreated, { nemaPath: entry.nemaPath, count: editors.length }));
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
        title={t.nema.pageTitle}
        subtitle={t.nema.pageSubtitle}
        icon={<FileText className="text-accent" size={20} />}
        track="D"
        count={`${nemaTasks.length} .nema`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label={t.nema.nemaTasks} value={nemaTasks.length} tone="info" />
        <Stat label={t.nema.boundTasks} value={boundTasks.length} tone="ok" />
        <Stat label={t.nema.autoRule} value={t.nema.autoRuleValue} tone="default" hint={t.nema.autoRuleHint} />
        <Stat label={t.nema.format} value=".NEMA v1" tone="default" hint="nema-sigma" />
      </div>

      {/* 本地文件夹桥接 (per 2026-11-11: 网页版关联本地文件夹) */}
      <LocalFolderBrowser onPickFile={(p) => setFilePath(p)} />

      {/* .nema 文件系统监听状态 (per WBS 8.4.4: 轮询检测磁盘 .nema 变化 → 任务状态回写) */}
      <div className="card mb-5" data-testid="nema-watcher-status">
        <SectionTitle>{t.nema.watcherTitle}</SectionTitle>
        <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-ink-dim">
          {nemaWatcher.watching ? (
            <span className="flex items-center gap-1.5 text-ok" data-testid="nema-watcher-active">
              <span className="size-2 rounded-full bg-ok animate-pulse" />
              {t.nema.watcherActive}
            </span>
          ) : (
            <span data-testid="nema-watcher-inactive">{t.nema.watcherInactive}</span>
          )}
          {nemaWatcher.lastResult && (
            <span data-testid="nema-watcher-result" className="font-mono">
              {t.nema.watcherScanned}: {nemaWatcher.lastResult.scanned} · {t.nema.watcherImported}: {nemaWatcher.lastResult.imported} · {t.nema.watcherUpdated}: {nemaWatcher.lastResult.updated}
            </span>
          )}
          {nemaWatcher.watching && (
            <button type="button" onClick={() => void nemaWatcher.scanNow()} className="btn-ghost text-xs" data-testid="nema-watcher-scan-now">
              {t.nema.watcherScanNow}
            </button>
          )}
        </div>
      </div>

      {/* 注册共同编辑文件 */}
      <div className="card mb-5" data-testid="nema-register-form">
        <SectionTitle>{t.nema.registerSectionTitle}</SectionTitle>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mt-3">
          <input
            data-testid="nema-input-path"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            placeholder={t.nema.pathPlaceholder}
            className="input text-xs"
          />
          <input
            data-testid="nema-input-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t.nema.titlePlaceholder}
            className="input text-xs"
          />
          <input
            data-testid="nema-input-editors"
            value={editorsText}
            onChange={(e) => setEditorsText(e.target.value)}
            placeholder={t.nema.editorsPlaceholder}
            className="input text-xs"
          />
          <input
            data-testid="nema-input-purpose"
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            placeholder={t.nema.purposePlaceholder}
            className="input text-xs"
          />
        </div>
        <div className="flex items-center gap-3 mt-3">
          <button data-testid="nema-register-btn" className="btn text-xs" onClick={handleRegister}>
            <FilePlus2 size={12} /> {t.nema.registerButton}
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
          {t.nema.coeditRuleFootnote}
        </div>
      </div>

      {/* .nema 任务列表 */}
      <div className="card" data-testid="nema-task-list">
        <SectionTitle>{tx(t.nema.nemaTaskListTitle, { count: nemaTasks.length })}</SectionTitle>
        {nemaTasks.length === 0 ? (
          <div className="text-xs text-ink-mute py-6 text-center">
            {t.nema.emptyList}
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
                    <FileText size={13} className="text-accent shrink-0" />
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
                          {t.nema.openBoundTask}
                        </button>
                      ) : (
                        <button
                          className="btn text-[10px] px-2 py-1"
                          onClick={() => handleOpenAsTask(entry.id)}
                          data-testid={`nema-open-task-${entry.id}`}
                        >
                          <Play size={10} /> {t.nema.openAsStarTask}
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
                    {t.nema.fiveW1H}: {view.what} · {view.why} · {view.how} · {view.who.join(",")} · {view.when} · {view.where}
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
