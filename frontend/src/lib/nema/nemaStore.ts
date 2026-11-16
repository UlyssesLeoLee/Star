/*
@cypher schema=1 source_sha256=7f601b45913d63d89c8c867453b12ab1ca37bf91173474e83987ac0844137c47
MERGE (self:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::useNemaStore",kind:"hook"})
MERGE (register:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::registerSharedFile",kind:"function"})
MERGE (open:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::openNemaAsStarTask",kind:"function"})
MERGE (import:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::importNemaSource",kind:"function"})
MERGE (sync:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::syncNemaSource",kind:"function"})
MERGE (parse_smart:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::parseNemaSmart",kind:"function"})
MERGE (parse_entry:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::parseNemaEntryView",kind:"function"})
MERGE (parse_wasm:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNemaWasm",kind:"function"})
MERGE (parse_legacy:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNema",kind:"function"})
MERGE (parse_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNemaWithProfileWasm",kind:"function"})
MERGE (format_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::serializeNemaWithProfileWasm",kind:"function"})
MERGE (task_view:Symbol {id:"frontend/src/lib/nema/nema.ts::taskViewFromStrictDocument",kind:"function"})
MERGE (operation_lock:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::withNemaOperationLock",kind:"function"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(open)
MERGE (self)-[:DEFINES]->(import)
MERGE (self)-[:DEFINES]->(sync)
MERGE (self)-[:DEFINES]->(parse_smart)
MERGE (self)-[:DEFINES]->(parse_entry)
MERGE (self)-[:DEFINES]->(operation_lock)
MERGE (register)-[:CALLS]->(operation_lock)
MERGE (open)-[:CALLS]->(operation_lock)
MERGE (parse_smart)-[:CALLS]->(parse_wasm)
MERGE (parse_smart)-[:CALLS]->(parse_legacy)
MERGE (parse_entry)-[:CALLS]->(parse_smart)
MERGE (parse_entry)-[:CALLS]->(parse_profile)
MERGE (parse_entry)-[:CALLS]->(task_view)
MERGE (register)-[:CALLS]->(format_profile)
MERGE (open)-[:CALLS]->(parse_entry)
MERGE (import)-[:CALLS]->(parse_entry)
MERGE (sync)-[:CALLS]->(parse_entry)
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
  taskViewFromDocument,
  taskViewFromStrictDocument,
  parseNema,
  parseNemaWasm,
  parseNemaWithProfileWasm,
  loadNemaWasm,
  serializeNemaWithProfileWasm,
  type NemaDocument,
  type NemaInterpreterProfile,
  type NemaTaskView,
} from "./nema";

// wasm codec 预加载 (legacy codec validation; profile-aware APIs are explicit).
void loadNemaWasm();

/** Legacy task projection: validate with the Rust AST when available, then build the UI view model. */
async function parseNemaSmart(source: string): Promise<NemaDocument> {
  if (await loadNemaWasm()) {
    try {
      await parseNemaWasm(source);
    } catch {
      /* Preserve the legacy TypeScript projection for files outside the old codec subset. */
    }
  }
  return parseNema(source);
}

/** Pinned entries always use their exact Rust interpreter; no profile fallback is allowed. */
async function parseNemaEntryView(source: string, profile?: NemaInterpreterProfile): Promise<NemaTaskView> {
  if (!profile) return taskViewFromDocument(await parseNemaSmart(source));
  if (!(await loadNemaWasm())) throw new Error("Rust NEMA interpreter is unavailable");
  const strict = await parseNemaWithProfileWasm(source, profile);
  return taskViewFromStrictDocument(strict);
}
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
  /** Grammar and interpreter identity pinned at import/create time; absent only on legacy entries. */
  interpreterProfile?: NemaInterpreterProfile;
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
  }, profile: NemaInterpreterProfile) => Promise<NemaTaskEntry | null>;
  /** 打开 .nema 为 Star 任务: 创建 WorkItem 并回填绑定。 */
  openNemaAsStarTask: (nemaId: string) => Promise<string | null>;
  /** 移除 .nema 任务记录。 */
  removeNemaTask: (nemaId: string) => void;
  /** 由 .nema 源码导入任务。 */
  importNemaSource: (nemaPath: string, source: string, profile: NemaInterpreterProfile) => Promise<NemaTaskEntry>;
  /** 文件系统监听同步 (per WBS 8.4.4): 磁盘 .nema 内容变化 → 更新记录 + 回写已绑定任务。 */
  syncNemaSource: (nemaPath: string, source: string, importProfile?: NemaInterpreterProfile) => Promise<{ action: "imported" | "updated" | "unchanged"; entry: NemaTaskEntry }>;
}

function makeId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

const nemaOperationTails = new Map<string, Promise<void>>();

/** Serialize asynchronous mutations for the same NEMA path or task identity. */
async function withNemaOperationLock<T>(key: string, operation: () => Promise<T>): Promise<T> {
  const predecessor = nemaOperationTails.get(key) ?? Promise.resolve();
  let release!: () => void;
  const current = new Promise<void>((resolve) => {
    release = resolve;
  });
  nemaOperationTails.set(key, current);
  await predecessor;
  try {
    return await operation();
  } finally {
    release();
    if (nemaOperationTails.get(key) === current) nemaOperationTails.delete(key);
  }
}

export const useNemaStore = create<NemaState>()(
  persist(
    (set, get) => ({
      nemaTasks: [],

      registerSharedFile: async (input, profile) => withNemaOperationLock(`path:${input.filePath}`, async () => {
        if (input.editors.length < 2) {
          // 单人编辑不产生 .nema (共同编辑才触发, per 用户发令)
          return null;
        }
        const existing = get().nemaTasks.find((t) => t.filePath === input.filePath);
        const pinnedProfile = existing?.interpreterProfile ?? profile;
        const now = new Date().toISOString();
        const doc = nemaDocumentForSharedFile({ ...input }, new Date(now));
        const source = await serializeNemaWithProfileWasm(doc, pinnedProfile);
        if (existing) {
          const updated: NemaTaskEntry = {
            ...existing,
            title: input.title,
            editors: [...input.editors],
            source,
            interpreterProfile: pinnedProfile,
            updated_at: now,
          };
          set((s) => ({
            nemaTasks: s.nemaTasks.map((t) => (t.id === existing.id ? updated : t)),
          }));
          return updated;
        }
        const id = makeId("nema");
        const entry: NemaTaskEntry = {
          id,
          filePath: input.filePath,
          nemaPath: `${input.filePath}.nema`,
          title: input.title,
          editors: [...input.editors],
          source,
          interpreterProfile: pinnedProfile,
          created_at: now,
          updated_at: now,
        };
        set((s) => ({ nemaTasks: [...s.nemaTasks, entry] }));
        return entry;
      }),

      openNemaAsStarTask: async (nemaId) => withNemaOperationLock(`task:${nemaId}`, async () => {
        const entry = get().nemaTasks.find((t) => t.id === nemaId);
        if (!entry) return null;
        if (entry.workItemId) return entry.workItemId;

        const view = await parseNemaEntryView(entry.source, entry.interpreterProfile);
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
      }),

      removeNemaTask: (nemaId) => {
        set((s) => ({ nemaTasks: s.nemaTasks.filter((t) => t.id !== nemaId) }));
      },

      importNemaSource: async (nemaPath, source, profile) => {
        const view = await parseNemaEntryView(source, profile);
        const id = makeId("nema");
        const entry: NemaTaskEntry = {
          id,
          filePath: view.filePath || nemaPath.replace(/\.nema$/, ""),
          nemaPath,
          title: view.title,
          editors: view.who,
          source,
          interpreterProfile: profile,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        set((s) => ({ nemaTasks: [...s.nemaTasks, entry] }));
        return entry;
      },

      syncNemaSource: async (nemaPath, source, importProfile) => {
        const existing = get().nemaTasks.find((t) => t.nemaPath === nemaPath);
        if (!existing) {
          if (!importProfile) throw new Error("Select an exact NEMA interpreter profile before importing an unpinned file");
          const entry = await get().importNemaSource(nemaPath, source, importProfile);
          return { action: "imported" as const, entry };
        }
        if (existing.source === source) {
          return { action: "unchanged" as const, entry: existing };
        }
        const view = await parseNemaEntryView(source, existing.interpreterProfile);
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
