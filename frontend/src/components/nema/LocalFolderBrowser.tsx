/*
@cypher schema=1 source_sha256=e9033113c5ae33dbf44bc48c7a9f103b9600063288e9ea4b70d5b64276914f94
MERGE (self:File {path:"frontend/src/components/nema/LocalFolderBrowser.tsx"})
MERGE (browser:Function {name:"LocalFolderBrowser", language:"typescript", visibility:"public"})
MERGE (self)-[:DEFINES]->(browser)
@endcypher
*/
"use client";

// =====================================================================
// LocalFolderBrowser — 本地文件夹桥接浏览器 (per 2026-11-11 用户发令:
// 网页版关联本地文件夹)。仅本地模式渲染; 走 /api/v1/workspace-files
// (后端白名单只读 API, 绑定 127.0.0.1)。线上无后端 → 显示降级提示。
// =====================================================================

import { useCallback, useEffect, useState } from "react";
import { Folder, FileText, ChevronRight, RefreshCw, Eye } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import {
  isLocalMode,
  listWorkspaceFiles,
  readWorkspaceFile,
  type WorkspaceFileList,
} from "@/lib/localFiles";

export interface LocalFolderBrowserProps {
  onPickFile?: (relativePath: string) => void;
}

export function LocalFolderBrowser({ onPickFile }: LocalFolderBrowserProps) {
  const { t, tx } = useTranslation();
  const [currentPath, setCurrentPath] = useState("");
  const [listing, setListing] = useState<WorkspaceFileList | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<{ path: string; text: string } | null>(null);

  const localMode = isLocalMode();

  const load = useCallback(async (path: string) => {
    setError(null);
    setPreview(null);
    const result = await listWorkspaceFiles(path);
    if (result.status === 200 && result.body) {
      setListing(result.body);
      setCurrentPath(result.body.path);
    } else {
      setListing(null);
      setError(result.error ?? "LIST_FAILED");
    }
  }, []);

  useEffect(() => {
    if (localMode) void load("");
  }, [localMode, load]);

  if (!localMode) {
    return null; // 线上无本地后端 → 不渲染 (文件关联降级为 GitHub blob)
  }

  const openFile = async (name: string) => {
    const full = currentPath ? `${currentPath}/${name}` : name;
    const result = await readWorkspaceFile(full);
    if (result.status === 200 && result.body) {
      setPreview({ path: result.body.path, text: result.body.text });
    } else {
      setError(result.error ?? "READ_FAILED");
    }
  };

  return (
    <div className="card mb-5" data-testid="local-folder-browser">
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-xs font-semibold text-ink">{t.nema.localFolderBridge}</span>
        <span className="text-[10px] font-mono text-ink-mute truncate max-w-[320px]">
          {listing ? `${listing.root}/${listing.path}` : t.nema.connecting}
        </span>
        <button className="btn-ghost p-1 rounded" onClick={() => void load(currentPath)} aria-label="refresh">
          <RefreshCw size={11} />
        </button>
      </div>

      {error && (
        <div data-testid="local-folder-error" className="text-[11px] text-warn font-mono mt-1">
          {tx(t.nema.bridgeUnavailable, { error })}
        </div>
      )}

      {listing && (
        <div className="mt-2 max-h-52 overflow-y-auto rounded border border-line/60 divide-y divide-line/40">
          {currentPath && (
            <button
              className="w-full text-left px-2 py-1 text-[11px] font-mono text-ink-dim hover:bg-bg-soft"
              onClick={() => {
                const parent = currentPath.includes("/")
                  ? currentPath.slice(0, currentPath.lastIndexOf("/"))
                  : "";
                void load(parent);
              }}
            >
              <ChevronRight size={10} className="inline -rotate-90 mr-1" /> {t.nema.parentDir}
            </button>
          )}
          {listing.entries.map((entry) => (
            <button
              key={entry.name}
              data-testid={`local-folder-entry-${entry.name}`}
              className="w-full text-left px-2 py-1 text-[11px] font-mono hover:bg-bg-soft flex items-center gap-1.5"
              onClick={() => (entry.is_dir ? void load(currentPath ? `${currentPath}/${entry.name}` : entry.name) : void openFile(entry.name))}
            >
              {entry.is_dir ? (
                <Folder size={10} className="text-amber-400 shrink-0" />
              ) : (
                <FileText size={10} className="text-cyan-400 shrink-0" />
              )}
              <span className="text-ink truncate">{entry.name}</span>
              {!entry.is_dir && entry.size_bytes !== null && (
                <span className="text-[9px] text-ink-mute ml-auto shrink-0">{entry.size_bytes} B</span>
              )}
            </button>
          ))}
          {listing.entries.length === 0 && (
            <div className="px-2 py-2 text-[10px] text-ink-mute">{t.nema.emptyDir}</div>
          )}
        </div>
      )}

      {preview && (
        <div className="mt-2" data-testid="local-folder-preview">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-[10px] font-mono text-cyan-300">{preview.path}</span>
            <button
              className="btn text-[10px] py-0.5"
              onClick={() => onPickFile?.(preview.path)}
              aria-label={t.nema.registerSharedFileAria}
            >
              <Eye size={10} /> {t.nema.registerSharedFile}
            </button>
          </div>
          <pre className="mt-1 max-h-40 overflow-auto rounded border border-line/60 bg-bg-soft p-2 text-[10px] font-mono text-ink-dim whitespace-pre-wrap">
            {preview.text.slice(0, 2000)}
          </pre>
        </div>
      )}
    </div>
  );
}
