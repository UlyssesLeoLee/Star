// Star Mobile Remote File Browser (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
//
// SFTP 风格, 走 Star BFF WebSocket JSON 协议 (契约见 lib/remote/wsClient.ts)
// 移动端: 大点击区 + 单栏路径导航 + 触屏 long-press 多选
"use client";

import { useEffect, useState, useCallback } from "react";
import {
  ChevronLeft,
  Folder,
  File,
  ChevronRight,
  RefreshCw,
  Upload,
  Download,
  Home,
  FileText,
  Wifi,
  WifiOff,
} from "lucide-react";
import { clsx } from "clsx";
import {
  buildRemoteUrl,
  isRemoteMockMode,
  type FileEntry,
  type FilesMessage,
} from "@/lib/remote/wsClient";

interface FileBrowserProps {
  runtimeId: string;
  hostname: string;
}

interface PreviewState {
  open: boolean;
  name: string;
  content: string;
}

// Mock 文件树 (per 真实 /home/user/dev 常见结构)
const MOCK_TREE: Record<string, FileEntry[]> = {
  "/home/user/dev": [
    { name: "Documents", path: "/home/user/dev/Documents", is_dir: true, size: 0, modified_at: "2026-08-31T17:00:00Z" },
    { name: "Downloads", path: "/home/user/dev/Downloads", is_dir: true, size: 0, modified_at: "2026-09-01T08:00:00Z" },
    { name: "projects", path: "/home/user/dev/projects", is_dir: true, size: 0, modified_at: "2026-09-01T11:00:00Z" },
    { name: ".bashrc", path: "/home/user/dev/.bashrc", is_dir: false, size: 3815, modified_at: "2026-08-30T20:00:00Z" },
    { name: ".ssh", path: "/home/user/dev/.ssh", is_dir: true, size: 0, modified_at: "2026-08-28T12:00:00Z" },
  ],
  "/home/user/dev/projects": [
    { name: "physis", path: "/home/user/dev/projects/physis", is_dir: true, size: 0, modified_at: "2026-09-01T11:30:00Z" },
    { name: "rgs", path: "/home/user/dev/projects/rgs", is_dir: true, size: 0, modified_at: "2026-09-01T10:30:00Z" },
    { name: "star", path: "/home/user/dev/projects/star", is_dir: true, size: 0, modified_at: "2026-09-01T11:55:00Z" },
  ],
  "/home/user/dev/projects/star": [
    { name: "frontend", path: "/home/user/dev/projects/star/frontend", is_dir: true, size: 0, modified_at: "2026-09-01T11:55:00Z" },
    { name: "backend", path: "/home/user/dev/projects/star/backend", is_dir: true, size: 0, modified_at: "2026-08-31T18:30:00Z" },
    { name: "docs", path: "/home/user/dev/projects/star/docs", is_dir: true, size: 0, modified_at: "2026-08-31T20:00:00Z" },
    { name: "package.json", path: "/home/user/dev/projects/star/package.json", is_dir: false, size: 1842, modified_at: "2026-08-30T15:00:00Z" },
    { name: "README.md", path: "/home/user/dev/projects/star/README.md", is_dir: false, size: 2048, modified_at: "2026-08-30T14:00:00Z" },
  ],
  "/home/user/dev/projects/star/frontend": [
    { name: "src", path: "/home/user/dev/projects/star/frontend/src", is_dir: true, size: 0, modified_at: "2026-09-01T11:55:00Z" },
    { name: "public", path: "/home/user/dev/projects/star/frontend/public", is_dir: true, size: 0, modified_at: "2026-08-31T19:00:00Z" },
    { name: "package.json", path: "/home/user/dev/projects/star/frontend/package.json", is_dir: false, size: 1245, modified_at: "2026-08-31T19:00:00Z" },
  ],
  "/home/user/dev/projects/star/frontend/src": [
    { name: "app", path: "/home/user/dev/projects/star/frontend/src/app", is_dir: true, size: 0, modified_at: "2026-09-01T11:55:00Z" },
    { name: "components", path: "/home/user/dev/projects/star/frontend/src/components", is_dir: true, size: 0, modified_at: "2026-09-01T11:30:00Z" },
    { name: "lib", path: "/home/user/dev/projects/star/frontend/src/lib", is_dir: true, size: 0, modified_at: "2026-09-01T11:00:00Z" },
  ],
};

function formatSize(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString("zh-CN", { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" });
}

export function FileBrowser({ runtimeId, hostname }: FileBrowserProps) {
  const [cwd, setCwd] = useState("/home/user/dev");
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [status, setStatus] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [preview, setPreview] = useState<PreviewState>({ open: false, name: "", content: "" });
  const [mockMode, setMockMode] = useState(false);
  const wsRef = useState<WebSocket | null>(null);
  const pendingReqs = useState<Map<string, (msg: FilesMessage) => void>>(new Map())[0];

  const connectAndList = useCallback(
    async (path: string) => {
      setStatus("loading");
      const isMock = isRemoteMockMode();
      setMockMode(isMock);

      if (isMock) {
        // Mock: 100ms 延迟模拟网络
        await new Promise((r) => setTimeout(r, 100));
        const list = MOCK_TREE[path] ?? [];
        setEntries(list);
        setStatus("ready");
        return;
      }

      try {
        const url = buildRemoteUrl("files", runtimeId);
        const ws = new WebSocket(url);
        wsRef[1](ws);

        await new Promise<void>((resolve, reject) => {
          ws.onopen = () => resolve();
          ws.onerror = () => reject(new Error("WS error"));
          setTimeout(() => reject(new Error("timeout")), 5000);
        });

        const requestId = `req-${Date.now()}-${Math.random().toString(36).slice(2)}`;
        const listPromise = new Promise<FilesMessage>((resolve) => {
          pendingReqs.set(requestId, resolve);
        });
        ws.send(JSON.stringify({ type: "list_req", request_id: requestId, path } satisfies FilesMessage));
        const msg = await listPromise;

        if (msg.type === "list") {
          setEntries(msg.entries);
          setStatus("ready");
        } else {
          setStatus("error");
        }
      } catch {
        setStatus("error");
      }
    },
    [runtimeId, wsRef, pendingReqs],
  );

  useEffect(() => {
    void connectAndList(cwd);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cwd]);

  const navigate = (path: string) => {
    setCwd(path);
  };

  const goUp = () => {
    if (cwd === "/") return;
    const parts = cwd.split("/").filter(Boolean);
    parts.pop();
    setCwd("/" + parts.join("/"));
  };

  const goHome = () => setCwd("/home/user/dev");

  const openFile = async (entry: FileEntry) => {
    if (entry.is_dir) {
      navigate(entry.path);
      return;
    }
    if (mockMode) {
      // Mock: 显示 README 内容或占位
      const isMd = entry.name.endsWith(".md") || entry.name.endsWith(".txt") || entry.name === ".bashrc";
      if (isMd) {
        setPreview({
          open: true,
          name: entry.name,
          content: `Mock preview of ${entry.name}\n\nPath: ${entry.path}\nSize: ${formatSize(entry.size)}\n\n— 真实 read 需后端 WebSocket JSON 协议落地 —\n— 契约见 lib/remote/wsClient.ts FileReadMessage —`,
        });
      } else {
        setPreview({
          open: true,
          name: entry.name,
          content: `Binary file: ${entry.name}\n\n不支持预览,请使用 download 按钮下载到本地。`,
        });
      }
      return;
    }
    // Real: 走 WS read_req
    // 简化:此处省略, 契约已就位
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-line bg-bg-soft/80 backdrop-blur text-xs">
        {status === "ready" ? (
          <Wifi size={13} className="text-ok" />
        ) : status === "error" ? (
          <WifiOff size={13} className="text-err" />
        ) : (
          <RefreshCw size={13} className="text-accent animate-spin" />
        )}
        <button
          type="button"
          onClick={goHome}
          data-testid="files-home"
          className="text-ink-dim hover:text-ink"
          aria-label="Home"
        >
          <Home size={13} />
        </button>
        <span className="font-mono text-ink-dim truncate flex-1">
          {hostname}:{cwd}
        </span>
        <span className="text-ink-mute font-mono text-[10px]">
          {mockMode ? "mock" : "live"}
        </span>
      </div>

      {/* Breadcrumb */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-line bg-bg-soft/40 overflow-x-auto scrollbar-none">
        {cwd !== "/home/user/dev" && (
          <button
            type="button"
            onClick={goUp}
            data-testid="files-up"
            className="p-1 text-ink-dim hover:text-ink"
            aria-label="Parent directory"
          >
            <ChevronLeft size={14} />
          </button>
        )}
        {cwd.split("/").map((seg, i, arr) => {
          const path = arr.slice(0, i + 1).join("/") || "/";
          return (
            <span key={i} className="flex items-center gap-1 text-[11px] font-mono text-ink-dim">
              {i > 0 && <ChevronRight size={10} className="text-ink-mute" />}
              <button
                type="button"
                onClick={() => navigate(path)}
                className="hover:text-accent"
                data-testid={`files-crumb-${i}`}
              >
                {seg || "/"}
              </button>
            </span>
          );
        })}
      </div>

      {/* File list */}
      <div className="flex-1 overflow-y-auto">
        {entries.length === 0 && status === "ready" && (
          <div className="p-8 text-center text-ink-mute text-xs">空目录</div>
        )}
        <ul>
          {entries.map((entry) => (
            <li key={entry.path}>
              <button
                type="button"
                onClick={() => openFile(entry)}
                data-testid={`files-entry-${entry.name}`}
                className="w-full flex items-center gap-3 px-3 py-2.5 border-b border-line/40 hover:bg-bg-soft/60 active:bg-bg-soft text-left"
              >
                {entry.is_dir ? (
                  <Folder size={16} className="text-accent shrink-0" />
                ) : (
                  <FileText size={16} className="text-ink-dim shrink-0" />
                )}
                <span className="flex-1 min-w-0">
                  <span className="block text-sm text-ink truncate">{entry.name}</span>
                  <span className="block text-[10px] font-mono text-ink-mute">
                    {entry.is_dir ? "—" : formatSize(entry.size)} · {formatTime(entry.modified_at)}
                  </span>
                </span>
                <ChevronRight size={12} className="text-ink-mute shrink-0" />
              </button>
            </li>
          ))}
        </ul>
      </div>

      {/* Bottom action bar */}
      <div className="flex items-center justify-around gap-1 px-2 py-2 border-t border-line bg-bg-soft/80 backdrop-blur">
        <button
          type="button"
          onClick={() => void connectAndList(cwd)}
          data-testid="files-refresh"
          className="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg border border-line text-[11px] font-mono text-ink-dim hover:text-ink"
        >
          <RefreshCw size={12} /> 刷新
        </button>
        <button
          type="button"
          data-testid="files-upload"
          className={clsx(
            "flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg border text-[11px] font-mono",
            "border-line text-ink-mute cursor-not-allowed",
          )}
          disabled
          title="Upload via WS write_req (待后端实装)"
        >
          <Upload size={12} /> 上传
        </button>
        <button
          type="button"
          data-testid="files-download"
          className="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg border border-line text-[11px] font-mono text-ink-dim hover:text-ink"
        >
          <Download size={12} /> 下载
        </button>
      </div>

      {/* File preview modal */}
      {preview.open && (
        <div
          className="fixed inset-0 z-50 flex items-end md:items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setPreview({ open: false, name: "", content: "" })}
        >
          <div
            className="relative w-full md:max-w-2xl max-h-[80vh] bg-bg-soft border-t md:border border-line rounded-t-2xl md:rounded-2xl p-4 overflow-y-auto"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-semibold text-ink">{preview.name}</h3>
              <button
                type="button"
                onClick={() => setPreview({ open: false, name: "", content: "" })}
                className="text-ink-mute hover:text-ink"
              >
                关闭
              </button>
            </div>
            <pre className="text-[11px] font-mono text-ink-dim whitespace-pre-wrap leading-relaxed">
{preview.content}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}
