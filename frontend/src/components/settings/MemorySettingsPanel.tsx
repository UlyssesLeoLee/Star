/*
@cypher schema=1 source_sha256=1a7f847ac5da055328862774e0b939d85b4270fb78f3a6a0db5be4e7fdd74e26
MERGE (self:File {path:"frontend/src/components/settings/MemorySettingsPanel.tsx"})
MERGE (react:ExternalService {name:"react"})
MERGE (localStorage:ExternalService {name:"localStorage"})
MERGE (memorySettingsPanel:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::MemorySettingsPanel",kind:"component"})
MERGE (readMemory:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::readMemory",kind:"function"})
MERGE (writeMemory:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::writeMemory",kind:"function"})
MERGE (memoryItem:Type {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::MemoryItem"})
MERGE (memoryProject:Type {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::MemoryProject"})
MERGE (self)-[:DEFINES]->(memorySettingsPanel)
MERGE (self)-[:DEFINES]->(readMemory)
MERGE (self)-[:DEFINES]->(writeMemory)
MERGE (self)-[:DEFINES]->(memoryItem)
MERGE (self)-[:DEFINES]->(memoryProject)
MERGE (self)-[:IMPORTS]->(react)
MERGE (readMemory)-[:READS]->(localStorage)
MERGE (writeMemory)-[:WRITES]->(localStorage)
MERGE (memorySettingsPanel)-[:CALLS]->(readMemory)
MERGE (memorySettingsPanel)-[:CALLS]->(writeMemory)
@endcypher
*/
"use client";

// =====================================================================
// MemorySettingsPanel — 设置 › 总记忆面板 (项目记忆 = AGENTS.md 式项目级指令文档)
// 诚实原则: 后端记忆 API 未接入 → 内容本地持久化到 localStorage 并显式标注。
// ① 顶部说明条; ② 左列表 (全局 GLOBAL.md 固定 + 项目记忆按项目列) + 右 Markdown
//    编辑器 (textarea + 字数统计 + 脏标记); ③ 保存写 localStorage key
//    star-memory-<key> (全局 star-memory-global), 清脏 + 成功提示;
//    ④ 未保存草稿离开警告 (beforeunload + 面板内提示);
//    ⑤ 底部诚实徽标 '本地持久化 — 记忆 API 接入后同步服务端';
//    ⑥ 新建项目记忆按钮未接入禁用 + 原因; ⑦ 导出 readMemory/writeMemory 供测试。
// =====================================================================
import { useEffect, useMemo, useState } from "react";

export const GLOBAL_MEMORY_KEY = "global";
export const MEMORY_STORAGE_PREFIX = "star-memory-";

export interface MemoryProject {
  id: string;
  name: string;
}

export interface MemoryItem {
  /** localStorage 后缀 key; 全局固定为 "global" */
  key: string;
  label: string;
  /** AGENTS.md 式文件名 (全局 = GLOBAL.md, 项目 = AGENTS.md) */
  fileName: string;
  projectId?: string;
}

function storageKey(key: string): string {
  return `${MEMORY_STORAGE_PREFIX}${key}`;
}

/** 读取记忆内容; 未保存 / localStorage 不可用返回 null (供测试直接调用) */
export function readMemory(key: string): string | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage.getItem(storageKey(key));
  } catch {
    return null;
  }
}

/** 写入记忆内容; 成功返回 true (供测试直接调用) */
export function writeMemory(key: string, content: string): boolean {
  if (typeof window === "undefined") return false;
  try {
    window.localStorage.setItem(storageKey(key), content);
    return true;
  } catch {
    return false;
  }
}

export function MemorySettingsPanel({ projects = [] }: { projects?: MemoryProject[] }) {
  const items = useMemo<MemoryItem[]>(
    () => [
      { key: GLOBAL_MEMORY_KEY, label: "全局记忆", fileName: "GLOBAL.md" },
      ...projects.map((p) => ({ key: p.id, label: p.name, fileName: "AGENTS.md", projectId: p.id })),
    ],
    [projects],
  );

  const [activeKey, setActiveKey] = useState<string>(GLOBAL_MEMORY_KEY);
  const [savedContents, setSavedContents] = useState<Record<string, string>>(() => {
    const map: Record<string, string> = {};
    for (const item of items) map[item.key] = readMemory(item.key) ?? "";
    return map;
  });
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [savedFlash, setSavedFlash] = useState(false);

  // projects prop 变化时补载新 key 的已保存内容
  useEffect(() => {
    setSavedContents((prev) => {
      const next = { ...prev };
      let changed = false;
      for (const item of items) {
        if (!(item.key in next)) {
          next[item.key] = readMemory(item.key) ?? "";
          changed = true;
        }
      }
      return changed ? next : prev;
    });
  }, [items]);

  const activeItem =
    items.find((i) => i.key === activeKey) ??
    items[0] ?? { key: GLOBAL_MEMORY_KEY, label: "全局记忆", fileName: "GLOBAL.md" };
  const content = drafts[activeKey] ?? savedContents[activeKey] ?? "";
  const dirty = drafts[activeKey] !== undefined && drafts[activeKey] !== savedContents[activeKey];

  const handleEdit = (value: string) => {
    setDrafts((prev) => ({ ...prev, [activeKey]: value }));
    setSavedFlash(false);
  };

  // ③ 保存: 写 localStorage, 清脏标记 + 成功提示
  const handleSave = () => {
    if (!writeMemory(activeKey, content)) return; // 隐私模式/配额失败 — 保持脏标记
    setSavedContents((prev) => ({ ...prev, [activeKey]: content }));
    setDrafts((prev) => {
      const next = { ...prev };
      delete next[activeKey];
      return next;
    });
    setSavedFlash(true);
  };

  // ④ 未保存草稿离开警告: dirty 期间注册 beforeunload
  useEffect(() => {
    if (!dirty) return;
    const handler = (e: BeforeUnloadEvent) => {
      e.preventDefault();
      e.returnValue = "";
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [dirty]);

  // 保存成功提示 2.5s 自动消失
  useEffect(() => {
    if (!savedFlash) return;
    const timer = window.setTimeout(() => setSavedFlash(false), 2500);
    return () => window.clearTimeout(timer);
  }, [savedFlash]);

  return (
    <div data-testid="memory-settings-panel" className="flex flex-col gap-3">
      {/* ① 顶部说明条 */}
      <div data-testid="memory-intro" className="rounded-xl border border-line bg-bg-card px-3.5 py-2.5">
        <p className="text-xs font-semibold text-ink">项目记忆 = AGENTS.md 式项目级指令文档</p>
        <p className="mt-0.5 text-[10px] text-ink-mute">
          全局记忆对所有项目生效; 项目记忆随项目加载, 是智能体每次会话都会读到的项目级指令。
        </p>
      </div>

      <div className="grid gap-3 md:grid-cols-[240px_minmax(0,1fr)]">
        {/* ② 左: 记忆条目列表 */}
        <div className="flex flex-col gap-2">
          <ul data-testid="memory-settings-list" className="flex flex-col gap-1.5">
            {items.map((item) => {
              const itemDirty =
                drafts[item.key] !== undefined && drafts[item.key] !== (savedContents[item.key] ?? "");
              const isActive = item.key === activeKey;
              return (
                <li key={item.key}>
                  <button
                    type="button"
                    data-testid={item.projectId ? `memory-item-${item.projectId}` : "memory-item-global"}
                    onClick={() => setActiveKey(item.key)}
                    className={`flex w-full items-center gap-2 rounded-lg border px-2.5 py-2 text-left transition-colors ${
                      isActive
                        ? "border-accent bg-accent/10"
                        : "border-line bg-bg-card hover:border-accent/50"
                    }`}
                  >
                    <span className="min-w-0 flex-1 truncate text-xs font-semibold text-ink">{item.label}</span>
                    {itemDirty && (
                      <span className="rounded bg-warn/10 px-1 py-0.5 text-[9px] font-semibold text-warn">
                        未保存
                      </span>
                    )}
                    <span className="shrink-0 font-mono text-[9px] text-ink-mute">{item.fileName}</span>
                  </button>
                </li>
              );
            })}
          </ul>

          {/* ⑥ 新建记忆按钮: 未接入禁用 + 原因 */}
          <div className="flex flex-col gap-1">
            <button
              type="button"
              data-testid="memory-item-new"
              disabled
              className="cursor-not-allowed rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-xs text-ink-mute"
            >
              ＋ 新建项目记忆
            </button>
            <p data-testid="memory-item-new-reason" className="text-[10px] text-ink-mute">
              未接入 — 记忆 API 接入后方可新建项目记忆
            </p>
          </div>
        </div>

        {/* ② 右: Markdown 编辑器 */}
        <div className="rounded-xl border border-line bg-bg-card p-3.5">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-xs font-bold text-ink">
              {activeItem.label} · <span className="font-mono">{activeItem.fileName}</span>
            </h3>
            <span className="font-mono text-[10px] text-ink-mute">{storageKey(activeItem.key)}</span>
            {dirty && (
              <span
                data-testid="memory-editor-dirty"
                className="rounded bg-warn/10 px-1.5 py-0.5 text-[10px] font-semibold text-warn"
              >
                未保存草稿
              </span>
            )}
            <span data-testid="memory-editor-count" className="ml-auto font-mono text-[10px] text-ink-mute">
              {content.length} 字
            </span>
          </div>

          <textarea
            data-testid="memory-editor"
            value={content}
            onChange={(e) => handleEdit(e.target.value)}
            placeholder="在此编写记忆内容 (Markdown)…"
            rows={14}
            className="mt-2 w-full resize-y rounded-lg border border-line bg-bg-soft/60 p-3 font-mono text-xs leading-5 text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />

          <div className="mt-2 flex flex-wrap items-center gap-2">
            <button
              type="button"
              data-testid="memory-editor-save"
              onClick={handleSave}
              disabled={!dirty}
              className={
                dirty
                  ? "rounded-lg bg-accent px-3 py-1.5 text-xs text-white hover:bg-accent-soft"
                  : "cursor-not-allowed rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-xs text-ink-mute"
              }
            >
              保存到本地
            </button>
            {savedFlash && (
              <span data-testid="memory-editor-saved" className="text-[11px] text-ok">
                已保存到本地 (localStorage)
              </span>
            )}
            {dirty && <span className="text-[10px] text-ink-mute">有未保存修改 — 离开页面前请保存</span>}
          </div>
        </div>
      </div>

      {/* ⑤ 底部诚实徽标 */}
      <div
        data-testid="memory-honest-badge"
        className="flex w-fit items-center rounded-full border border-warn/40 bg-warn/10 px-2.5 py-1 text-[10px] font-semibold text-warn"
      >
        本地持久化 — 记忆 API 接入后同步服务端
      </div>
    </div>
  );
}
