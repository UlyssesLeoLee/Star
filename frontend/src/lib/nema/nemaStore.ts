/*
@cypher schema=1 source_sha256=c21320ea8b122a865330eb53d20b31d028e582a9b90c2a5506e51e813e2961d6
MERGE (self:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::useNemaStore",kind:"hook"})
MERGE (register:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::registerSharedFile",kind:"function"})
MERGE (open:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::openNemaAsStarTask",kind:"function"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(open)
@endcypher
*/
// =====================================================================
// .NEMA 文件即任务 store (per 2026-11-11 用户发令)
// 规则: 共同编辑的文件 (editors ≥ 2) 自动产生一个 .nema 文件;
//       .nema 文件可以作为 Star 任务 (WorkItem) 打开。
// =====================================================================

import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  nemaDocumentForSharedFile,
  serializeNema,
  taskViewFromDocument,
  parseNema,
} from "./nema";
import { useStore as useMainStore } from "@/lib/store";
import type { WorkItemDocument } from "@/types/ids";

export interface NemaTaskEntry {
  id: string;
  /** 共同编辑的文件路径 */
  filePath: string;
  /** 侧车文件路径 (.nema) */
  nemaPath: string;
  title: string;
  editors: string[];
  /** .nema 源码 (序列化全文) */
  source: string;
  /** 打开为 Star 任务后绑定的 work item id */
  workItemId?: string;
  created_at: string;
  updated_at: string;
}

interface NemaState {
  nemaTasks: NemaTaskEntry[];
  /** 注册一个共享文件: editors ≥ 2 时自动生成 .nema (幂等, 同 path 复用)。 */
  registerSharedFile: (input: {
    filePath: string;
    title: string;
    editors: string[];
    purpose?: string;
    method?: string;
    environment?: string;
  }) => NemaTaskEntry | null;
  /** 打开 .nema 为 Star 任务: 创建 WorkItem 并回填绑定。 */
  openNemaAsStarTask: (nemaId: string) => string | null;
  /** 移除 .nema 任务记录。 */
  removeNemaTask: (nemaId: string) => void;
  /** 由 .nema 源码导入任务。 */
  importNemaSource: (nemaPath: string, source: string) => NemaTaskEntry;
  /** 文件系统监听同步 (per WBS 8.4.4): 磁盘 .nema 内容变化 → 更新记录 + 回写已绑定任务。 */
  syncNemaSource: (nemaPath: string, source: string) => { action: "imported" | "updated" | "unchanged"; entry: NemaTaskEntry };
}

function makeId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export const useNemaStore = create<NemaState>()(
  persist(
    (set, get) => ({
      nemaTasks: [],

      registerSharedFile: (input) => {
        if (input.editors.length < 2) {
          // 单人编辑不产生 .nema (共同编辑才触发, per 用户发令)
          return null;
        }
        const existing = get().nemaTasks.find((t) => t.filePath === input.filePath);
        const now = new Date().toISOString();
        if (existing) {
          const doc = nemaDocumentForSharedFile({ ...input }, new Date(now));
          const updated: NemaTaskEntry = {
            ...existing,
            title: input.title,
            editors: [...input.editors],
            source: serializeNema(doc),
            updated_at: now,
          };
          set((s) => ({
            nemaTasks: s.nemaTasks.map((t) => (t.id === existing.id ? updated : t)),
          }));
          return updated;
        }
        const id = makeId("nema");
        const doc = nemaDocumentForSharedFile({ ...input }, new Date(now));
        const entry: NemaTaskEntry = {
          id,
          filePath: input.filePath,
          nemaPath: `${input.filePath}.nema`,
          title: input.title,
          editors: [...input.editors],
          source: serializeNema(doc),
          created_at: now,
          updated_at: now,
        };
        set((s) => ({ nemaTasks: [...s.nemaTasks, entry] }));
        return entry;
      },

      openNemaAsStarTask: (nemaId) => {
        const entry = get().nemaTasks.find((t) => t.id === nemaId);
        if (!entry) return null;
        if (entry.workItemId) return entry.workItemId;

        const view = taskViewFromDocument(parseNema(entry.source));
        // 跨 store 创建 WorkItem (main store), 文档关联 .nema 侧车 + 源文件
        const main = useMainStore.getState();
        const documents: WorkItemDocument[] = [
          {
            id: makeId("doc"),
            title: `${view.title} (.nema)`,
            path: entry.nemaPath,
            kind: "document",
            added_at: new Date().toISOString(),
          },
        ];
        if (view.filePath) {
          documents.push({
            id: makeId("doc"),
            title: `源文件 ${view.filePath.split("/").pop()}`,
            path: view.filePath,
            kind: "document",
            added_at: new Date().toISOString(),
          });
        }
        const workItemId = main.addWorkItem({
          tenant_id: "00000000-0000-0000-0000-000000000001",
          project_id: "00000000-0000-0000-0000-000000000001",
          title: view.title,
          kind: "task",
          status: "todo",
          priority: "p2",
          reporter_id: "usr-001",
          labels: ["nema", "file-as-task"],
          documents,
        });
        set((s) => ({
          nemaTasks: s.nemaTasks.map((t) => (t.id === nemaId ? { ...t, workItemId } : t)),
        }));
        return workItemId;
      },

      removeNemaTask: (nemaId) => {
        set((s) => ({ nemaTasks: s.nemaTasks.filter((t) => t.id !== nemaId) }));
      },

      importNemaSource: (nemaPath, source) => {
        const doc = parseNema(source);
        const view = taskViewFromDocument(doc);
        const id = makeId("nema");
        const entry: NemaTaskEntry = {
          id,
          filePath: view.filePath || nemaPath.replace(/\.nema$/, ""),
          nemaPath,
          title: view.title,
          editors: view.who,
          source,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        set((s) => ({ nemaTasks: [...s.nemaTasks, entry] }));
        return entry;
      },

      syncNemaSource: (nemaPath, source) => {
        const existing = get().nemaTasks.find((t) => t.nemaPath === nemaPath);
        if (!existing) {
          const entry = get().importNemaSource(nemaPath, source);
          return { action: "imported" as const, entry };
        }
        if (existing.source === source) {
          return { action: "unchanged" as const, entry: existing };
        }
        const doc = parseNema(source);
        const view = taskViewFromDocument(doc);
        const now = new Date().toISOString();
        const updated: NemaTaskEntry = {
          ...existing,
          title: view.title,
          editors: view.who,
          source,
          updated_at: now,
        };
        set((s) => ({
          nemaTasks: s.nemaTasks.map((t) => (t.id === existing.id ? updated : t)),
        }));
        // 任务状态回写 (per WBS 8.4.4): 已绑定 WorkItem 的 .nema 变化 → 同步标题/描述
        if (existing.workItemId) {
          const main = useMainStore.getState();
          const target = main.workItems.find((w) => w.id === existing.workItemId);
          if (target) {
            main.updateWorkItemField(existing.workItemId, "title", view.title);
            main.updateWorkItemField(
              existing.workItemId,
              "description",
              [view.what, view.why, view.how].filter(Boolean).join("\n\n"),
            );
          }
        }
        return { action: "updated" as const, entry: updated };
      },
    }),
    { name: "star-nema-files" },
  ),
);
