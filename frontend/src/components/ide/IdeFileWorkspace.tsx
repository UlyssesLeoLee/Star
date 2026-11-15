/*
@cypher schema=1 source_sha256=32c863d694eb52c2b3f2bd8ccab3febdf69effa177083470cc9ff5ca7899fefc
MERGE (self:File {path:"frontend/src/components/ide/IdeFileWorkspace.tsx"})
MERGE (nemaStore:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (nemaTaskEntry:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::NemaTaskEntry",kind:"type"})
MERGE (ideFileWorkspace:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::IdeFileWorkspace",kind:"component"})
MERGE (deriveSemanticNodes:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::deriveSemanticNodes",kind:"function"})
MERGE (buildFileTree:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::buildFileTree",kind:"function"})
MERGE (self)-[:IMPORTS]->(nemaStore)
MERGE (self)-[:USES_TYPE]->(nemaTaskEntry)
MERGE (self)-[:DEFINES]->(ideFileWorkspace)
MERGE (self)-[:DEFINES]->(deriveSemanticNodes)
MERGE (self)-[:DEFINES]->(buildFileTree)
MERGE (ideFileWorkspace)-[:CALLS]->(deriveSemanticNodes)
MERGE (ideFileWorkspace)-[:CALLS]->(buildFileTree)
@endcypher
*/
"use client";

// =====================================================================
// IdeFileWorkspace — IDE1.0 文件编辑工作区 (file-as-task 编辑内核视图)
// 五个区块: 顶栏编辑内核状态条 / 左侧文件树 / 右侧语义节点面板 /
//           底部 Patch 提交记录 / IDE1.0 能力徽标行。
// 诚实标注: ide-kernel 服务未接入 → 内核徽标显示"未接入", 语义节点为
//           文件路径段本地推导 (标注"本地推导预览"), Patch 记录为空态。
// 数据源: 调用方从 nemaStore.nemaTasks 取 {id,filePath,editors,workItemId}
//         传入 files prop — 本组件只读 props, 不直接订阅 store。
// =====================================================================

import { useMemo, useState, type ReactElement } from "react";
import {
  Activity,
  ChevronDown,
  ChevronRight,
  Cpu,
  FileCode,
  Folder,
  FolderOpen,
  GitCommit,
  Layers,
  Route,
  ShieldCheck,
} from "lucide-react";
import type { NemaTaskEntry } from "@/lib/nema/nemaStore";

/** 工作区文件引用 — 从 nemaStore.nemaTasks 投影出的字段子集。 */
export type IdeFileRef = Pick<NemaTaskEntry, "id" | "filePath" | "editors" | "workItemId">;

/** Patch 提交记录条目 (编辑内核接入后由 CAS+Fencing 协调层同步)。 */
export interface IdePatchEntry {
  id: string;
  summary: string;
  created_at?: string;
}

export interface IdeFileWorkspaceProps {
  files: IdeFileRef[];
  /** ide-kernel 服务是否已连接 (默认 false → "未接入" 徽标)。 */
  kernelConnected?: boolean;
  /** 诊断计数 (编辑内核未接入时为本地会话值, 默认 0)。 */
  diagCount?: number;
  /** Patch 提交记录 (空 → 显示空态文案)。 */
  patches?: IdePatchEntry[];
}

/** 语义节点 — 路径段本地推导: 文件夹=模块节点, 文件名=函数节点。 */
export interface IdeSemNode {
  label: string;
  kind: "module" | "function";
}

/** 按文件路径段推导语义节点 (最后一个段=函数节点, 其余=模块节点)。 */
export function deriveSemanticNodes(filePath: string): IdeSemNode[] {
  const segments = filePath.split("/").filter(Boolean);
  return segments.map((segment, index) => ({
    label: segment,
    kind: index === segments.length - 1 ? "function" : "module",
  }));
}

interface TreeNodeDir {
  kind: "dir";
  name: string;
  path: string;
  children: TreeNode[];
}
interface TreeNodeFile {
  kind: "file";
  file: IdeFileRef;
  name: string;
}
type TreeNode = TreeNodeDir | TreeNodeFile;

/** 由文件路径段构建树形结构 (目录在前, 字母序; 递归排序子节点)。 */
export function buildFileTree(files: IdeFileRef[]): TreeNode[] {
  const root: TreeNodeDir = { kind: "dir", name: "", path: "", children: [] };
  for (const file of files) {
    const segments = file.filePath.split("/").filter(Boolean);
    let dir = root;
    segments.forEach((segment, index) => {
      const isLast = index === segments.length - 1;
      if (isLast) {
        dir.children.push({ kind: "file", file, name: segment });
        return;
      }
      const childPath = dir.path ? `${dir.path}/${segment}` : segment;
      let child = dir.children.find(
        (n): n is TreeNodeDir => n.kind === "dir" && n.name === segment,
      );
      if (!child) {
        child = { kind: "dir", name: segment, path: childPath, children: [] };
        dir.children.push(child);
      }
      dir = child;
    });
  }
  const sortNodes = (nodes: TreeNode[]): TreeNode[] =>
    nodes
      .map((n) => (n.kind === "dir" ? { ...n, children: sortNodes(n.children) } : n))
      .sort((a, b) => {
        if (a.kind !== b.kind) return a.kind === "dir" ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
  return sortNodes(root.children);
}

/** testid 用安全串: 目录路径中只保留字母数字与 . _ -。 */
function sanitizeTestId(raw: string): string {
  return raw.replace(/[^a-zA-Z0-9._-]/g, "-");
}

/** IDE1.0 四项核心能力 (editshell 内核能力, 展示用途徽标)。 */
const CAPABILITIES = [
  { id: "file-routing", label: "文件级路由", icon: Route },
  { id: "semantic-parallel", label: "语义节点并行", icon: Layers },
  { id: "patch-commit", label: "Patch 级提交", icon: GitCommit },
  { id: "cas-fencing", label: "CAS+Fencing", icon: ShieldCheck },
] as const;

export function IdeFileWorkspace({
  files,
  kernelConnected = false,
  diagCount = 0,
  patches = [],
}: IdeFileWorkspaceProps) {
  const [collapsedDirs, setCollapsedDirs] = useState<Set<string>>(new Set());
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null);

  const tree = useMemo(() => buildFileTree(files), [files]);

  // 选中文件: 显式选择优先, 否则默认第一个 (files 为空时为 null)。
  const selectedFile = useMemo(() => {
    if (files.length === 0) return null;
    return files.find((f) => f.id === selectedFileId) ?? files[0];
  }, [files, selectedFileId]);

  const semNodes = useMemo(
    () => (selectedFile ? deriveSemanticNodes(selectedFile.filePath) : []),
    [selectedFile],
  );

  const patchCount = patches.length;

  const toggleDir = (path: string) => {
    setCollapsedDirs((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  const renderNode = (node: TreeNode, depth: number): ReactElement => {
    if (node.kind === "dir") {
      const isCollapsed = collapsedDirs.has(node.path);
      return (
        <div key={`dir:${node.path}`}>
          <button
            type="button"
            aria-expanded={!isCollapsed}
            data-testid={`ide-file-folder-${sanitizeTestId(node.path)}`}
            onClick={() => toggleDir(node.path)}
            className="flex w-full items-center gap-1.5 rounded px-2 py-1 text-left text-[13px] text-ink hover:bg-bg-soft"
          >
            {isCollapsed ? (
              <ChevronRight className="h-3.5 w-3.5 shrink-0 opacity-60" />
            ) : (
              <ChevronDown className="h-3.5 w-3.5 shrink-0 opacity-60" />
            )}
            {isCollapsed ? (
              <Folder className="h-4 w-4 shrink-0 text-sky-500" />
            ) : (
              <FolderOpen className="h-4 w-4 shrink-0 text-sky-500" />
            )}
            <span className="truncate font-medium">{node.name}</span>
            <span className="ml-auto text-[11px] opacity-50">{node.children.length}</span>
          </button>
          {!isCollapsed && (
            <div className="ml-[13px] border-l border-line pl-2" role="group">
              {node.children.map((child) => renderNode(child, depth + 1))}
            </div>
          )}
        </div>
      );
    }
    const isSelected = selectedFile?.id === node.file.id;
    return (
      <button
        type="button"
        key={`file:${node.file.id}`}
        aria-current={isSelected}
        data-testid={`ide-file-node-${node.file.id}`}
        onClick={() => setSelectedFileId(node.file.id)}
        className={`flex w-full items-center gap-1.5 rounded px-2 py-1 text-left text-[13px] text-ink hover:bg-bg-soft ${
          isSelected ? "bg-bg-soft" : ""
        }`}
      >
        <FileCode className="h-4 w-4 shrink-0 text-amber-500" />
        <span className="truncate">{node.name}</span>
        <span className="ml-auto flex shrink-0 items-center gap-1">
          {node.file.workItemId ? (
            <span className="rounded bg-emerald-500/10 px-1 text-[10px] text-emerald-600">任务</span>
          ) : null}
          <span className="text-[11px] opacity-50">{node.file.editors.length}人</span>
        </span>
      </button>
    );
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-bg text-ink">
      {/* ① 顶栏编辑内核状态条 */}
      <header className="flex flex-wrap items-center gap-2 border-b border-line bg-bg-card px-3 py-2">
        <Cpu className="h-4 w-4 opacity-60" />
        <span className="text-sm font-semibold">编辑内核</span>
        <span
          data-testid="ide-kernel-status"
          className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium ${
            kernelConnected
              ? "bg-emerald-500/10 text-emerald-600"
              : "bg-amber-500/10 text-amber-600"
          }`}
        >
          <span
            className={`h-1.5 w-1.5 rounded-full ${
              kernelConnected ? "bg-emerald-500" : "bg-amber-500"
            }`}
          />
          {kernelConnected ? "已连接" : "未接入"}
        </span>
        <span className="ml-auto flex items-center gap-3 text-xs opacity-80">
          <span className="inline-flex items-center gap-1">
            <Activity className="h-3.5 w-3.5" />
            诊断
            <b data-testid="ide-diag-count">{diagCount}</b>
          </span>
          <span className="inline-flex items-center gap-1">
            <GitCommit className="h-3.5 w-3.5" />
            Patch
            <b data-testid="ide-patch-count">{patchCount}</b>
          </span>
        </span>
      </header>

      {/* ⑤ IDE1.0 能力徽标行 */}
      <div className="flex flex-wrap items-center gap-2 border-b border-line px-3 py-1.5">
        {CAPABILITIES.map(({ id, label, icon: Icon }) => (
          <span
            key={id}
            data-testid={`ide-cap-${id}`}
            className="inline-flex items-center gap-1.5 rounded-full border border-line bg-bg-soft px-2 py-0.5 text-[11px] text-ink-dim"
          >
            <Icon className="h-3.5 w-3.5 opacity-70" />
            {label}
          </span>
        ))}
      </div>

      {/* ② 左文件树 + ③ 右语义节点面板 */}
      <div className="flex min-h-0 flex-1">
        <aside className="flex w-64 shrink-0 flex-col border-r border-line">
          <header className="border-b border-line px-3 py-2 text-xs font-semibold">文件树</header>
          <div data-testid="ide-file-tree" className="flex-1 overflow-auto p-1.5">
            {tree.length === 0 ? (
              <p className="p-2 text-xs opacity-60">暂无文件 — 注册共享文件后出现在这里</p>
            ) : (
              tree.map((node) => renderNode(node, 0))
            )}
          </div>
        </aside>

        <section data-testid="ide-semantic-panel" className="flex min-w-0 flex-1 flex-col">
          <header className="flex items-center gap-2 border-b border-line px-3 py-2">
            <h2 className="text-sm font-semibold">语义节点</h2>
            <span className="rounded bg-amber-500/10 px-1.5 py-0.5 text-[11px] text-amber-600">
              本地推导预览
            </span>
          </header>
          {selectedFile ? (
            <div
              data-testid={`ide-sem-node-${selectedFile.id}`}
              className="min-h-0 flex-1 overflow-auto p-3"
            >
              <p className="mb-2 truncate font-mono text-xs opacity-60">{selectedFile.filePath}</p>
              <ul className="space-y-1.5">
                {semNodes.map((node, index) => (
                  <li
                    key={`${node.kind}:${index}`}
                    data-testid={`ide-sem-node-${selectedFile.id}-${index}`}
                    className="flex items-center gap-2 rounded border border-line bg-bg-card px-2 py-1.5"
                  >
                    {node.kind === "module" ? (
                      <Layers className="h-4 w-4 shrink-0 text-sky-500" />
                    ) : (
                      <FileCode className="h-4 w-4 shrink-0 text-amber-500" />
                    )}
                    <span className="truncate font-mono text-xs">{node.label}</span>
                    <span className="ml-auto shrink-0 rounded bg-bg-soft px-1.5 py-0.5 text-[10px] opacity-70">
                      {node.kind === "module" ? "模块节点" : "函数节点"}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          ) : (
            <div className="flex-1 p-4 text-sm opacity-60">选中左侧文件后展示语义节点</div>
          )}
        </section>
      </div>

      {/* ④ 底部 Patch 提交记录区 */}
      <footer
        data-testid="ide-patch-log"
        className="flex max-h-44 min-h-0 flex-col border-t border-line"
      >
        <header className="flex items-center gap-2 px-3 py-1.5 text-xs font-semibold">
          Patch 提交记录
          <span className="opacity-50">({patchCount})</span>
        </header>
        {patches.length === 0 ? (
          <p className="px-3 pb-3 text-xs opacity-60">
            暂无 Patch 提交 — 编辑内核接入后同步
          </p>
        ) : (
          <ul className="min-h-0 overflow-auto px-3 pb-2">
            {patches.map((patch) => (
              <li key={patch.id} className="flex items-center gap-2 py-0.5 text-xs">
                <GitCommit className="h-3.5 w-3.5 shrink-0 opacity-60" />
                <span className="font-mono opacity-60">{patch.id}</span>
                <span className="truncate">{patch.summary}</span>
                {patch.created_at ? (
                  <span className="ml-auto shrink-0 opacity-50">{patch.created_at}</span>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </footer>
    </div>
  );
}
