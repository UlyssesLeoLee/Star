/*
@cypher schema=1 source_sha256=d0c1d80b517b10fc1573ff2bfeaea65ab138aab82e299fa2db10c48fb77d1c3b
MERGE (self:File {path:"frontend/src/components/settings/HooksSettingsPanel.tsx"})
MERGE (react:ExternalService {name:"react"})
MERGE (lucide:ExternalService {name:"lucide-react"})
MERGE (localStorage:ExternalService {name:"localStorage"})
MERGE (hooksSettingsPanel:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HooksSettingsPanel",kind:"component"})
MERGE (hookFormModal:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HookFormModal",kind:"component"})
MERGE (hookConfigModal:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HookConfigModal",kind:"component"})
MERGE (readHooks:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::readHooks",kind:"function"})
MERGE (writeHooks:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::writeHooks",kind:"function"})
MERGE (hookType:Type {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HookType"})
MERGE (hookConfig:Type {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HookConfig"})
MERGE (hookFormState:Type {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HookFormState"})
MERGE (self)-[:DEFINES]->(hooksSettingsPanel)
MERGE (self)-[:DEFINES]->(hookFormModal)
MERGE (self)-[:DEFINES]->(hookConfigModal)
MERGE (self)-[:DEFINES]->(readHooks)
MERGE (self)-[:DEFINES]->(writeHooks)
MERGE (self)-[:DEFINES]->(hookType)
MERGE (self)-[:DEFINES]->(hookConfig)
MERGE (self)-[:DEFINES]->(hookFormState)
MERGE (self)-[:IMPORTS]->(react)
MERGE (self)-[:IMPORTS]->(lucide)
MERGE (readHooks)-[:READS]->(localStorage)
MERGE (writeHooks)-[:WRITES]->(localStorage)
MERGE (hooksSettingsPanel)-[:CALLS]->(readHooks)
MERGE (hooksSettingsPanel)-[:CALLS]->(writeHooks)
MERGE (hooksSettingsPanel)-[:CALLS]->(hookFormModal)
MERGE (hooksSettingsPanel)-[:CALLS]->(hookConfigModal)
@endcypher
*/
"use client";

// =====================================================================
// HooksSettingsPanel — 设置 › 扩展 › Hooks 管理面板
// 诚实原则: 后端 hooks 持久化/执行引擎未接入 → 配置本地持久化到
//   localStorage (key star-hooks) 并显式标注。
// ① 按 type 分组列表 (pre_commit / pre_push / post_merge / custom),
//    每行: 名称/类型徽标/命令预览/启用 toggle/编辑/配置/删除; 空态提示;
// ② 新增/编辑弹窗 (表单: 名称/type select/命令/args/环境变量/匹配模式/描述);
// ③ 配置弹窗 (只读详情 + 快捷编辑);
// ④ 删除二次确认;
// ⑤ 启用/禁用与所有变更即时写 localStorage (star-hooks),
//    storage 事件监听跨标签页同步;
// ⑥ 底部诚实徽标 '本地持久化 — Hooks 执行引擎接入后同步';
// ⑦ 导出 readHooks/writeHooks 供测试。
// =====================================================================
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Plus, Pencil, Trash2, Settings2, X } from "lucide-react";

export const HOOKS_STORAGE_KEY = "star-hooks";

export type HookType = "pre_commit" | "pre_push" | "post_merge" | "custom";

export const HOOK_TYPES: HookType[] = ["pre_commit", "pre_push", "post_merge", "custom"];

export const HOOK_TYPE_LABELS: Record<HookType, string> = {
  pre_commit: "提交前",
  pre_push: "推送前",
  post_merge: "合并后",
  custom: "自定义",
};

export interface HookConfig {
  id: string;
  name: string;
  type: HookType;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  enabled: boolean;
  description: string;
  matchPattern?: string;
}

interface HookFormState {
  name: string;
  type: HookType;
  command: string;
  /** 每行一个参数 */
  argsText: string;
  /** 每行一个 KEY=VALUE */
  envText: string;
  matchPattern: string;
  description: string;
  enabled: boolean;
}

type ModalState =
  | { kind: "none" }
  | { kind: "form"; mode: "create" }
  | { kind: "form"; mode: "edit"; hook: HookConfig }
  | { kind: "config"; hook: HookConfig }
  | { kind: "delete"; hook: HookConfig };

// =====================================================================
// localStorage 持久化 helpers (导出供测试直接调用)
// =====================================================================

function isHookConfig(v: unknown): v is HookConfig {
  if (typeof v !== "object" || v === null) return false;
  const h = v as Record<string, unknown>;
  return (
    typeof h.id === "string" &&
    typeof h.name === "string" &&
    typeof h.command === "string" &&
    typeof h.type === "string" &&
    (HOOK_TYPES as string[]).includes(h.type) &&
    (h.args === undefined ||
      (Array.isArray(h.args) && h.args.every((a) => typeof a === "string"))) &&
    (h.env === undefined ||
      (typeof h.env === "object" &&
        h.env !== null &&
        Object.values(h.env).every((x) => typeof x === "string"))) &&
    (h.matchPattern === undefined || typeof h.matchPattern === "string") &&
    (h.description === undefined || typeof h.description === "string") &&
    (h.enabled === undefined || typeof h.enabled === "boolean")
  );
}

function normalizeHook(h: HookConfig): HookConfig {
  return {
    id: h.id,
    name: h.name,
    type: h.type,
    command: h.command,
    args: h.args ?? [],
    env: h.env ?? {},
    enabled: h.enabled ?? true,
    description: h.description ?? "",
    ...(h.matchPattern !== undefined ? { matchPattern: h.matchPattern } : {}),
  };
}

/** 读取本地 hooks; localStorage 不可用/损坏/非法条目时安全降级 (供测试直接调用) */
export function readHooks(): HookConfig[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(HOOKS_STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isHookConfig).map(normalizeHook);
  } catch {
    return [];
  }
}

/** 写入本地 hooks; 成功返回 true (供测试直接调用) */
export function writeHooks(hooks: HookConfig[]): boolean {
  if (typeof window === "undefined") return false;
  try {
    window.localStorage.setItem(HOOKS_STORAGE_KEY, JSON.stringify(hooks));
    return true;
  } catch {
    return false;
  }
}

// =====================================================================
// 表单 ↔ HookConfig 转换
// =====================================================================

function parseArgsText(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((s) => s.trim())
    .filter(Boolean);
}

function parseEnvText(text: string): Record<string, string> {
  const env: Record<string, string> = {};
  for (const line of text.split(/\r?\n/)) {
    const t = line.trim();
    if (!t) continue;
    const eq = t.indexOf("=");
    if (eq <= 0) continue; // 无 "=" 或空 key 的行忽略
    env[t.slice(0, eq).trim()] = t.slice(eq + 1).trim();
  }
  return env;
}

function argsToText(args?: string[]): string {
  return (args ?? []).join("\n");
}

function envToText(env?: Record<string, string>): string {
  return Object.entries(env ?? {})
    .map(([k, v]) => `${k}=${v}`)
    .join("\n");
}

function formToHook(f: HookFormState, id: string): HookConfig {
  return {
    id,
    name: f.name.trim(),
    type: f.type,
    command: f.command.trim(),
    args: parseArgsText(f.argsText),
    env: parseEnvText(f.envText),
    enabled: f.enabled,
    description: f.description.trim(),
    ...(f.matchPattern.trim() !== "" ? { matchPattern: f.matchPattern.trim() } : {}),
  };
}

function hookToForm(h: HookConfig): HookFormState {
  return {
    name: h.name,
    type: h.type,
    command: h.command,
    argsText: argsToText(h.args),
    envText: envToText(h.env),
    matchPattern: h.matchPattern ?? "",
    description: h.description ?? "",
    enabled: h.enabled,
  };
}

const EMPTY_FORM: HookFormState = {
  name: "",
  type: "pre_commit",
  command: "",
  argsText: "",
  envText: "",
  matchPattern: "",
  description: "",
  enabled: true,
};

function newHookId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `hook-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

// =====================================================================
// 弹窗外壳 (overlay + dialog)
// =====================================================================

function ModalShell({ children, onClose }: { children: ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/40 p-4">
      <div
        role="dialog"
        aria-modal="true"
        className="max-h-[90vh] w-full max-w-md overflow-y-auto rounded-xl border border-line bg-bg-card p-4 shadow-xl"
      >
        {children}
      </div>
    </div>
  );
}

// =====================================================================
// 新增/编辑弹窗
// =====================================================================

function HookFormModal({
  mode,
  hook,
  onSave,
  onClose,
}: {
  mode: "create" | "edit";
  hook?: HookConfig;
  onSave: (draft: HookFormState) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState<HookFormState>(() =>
    mode === "edit" && hook ? hookToForm(hook) : { ...EMPTY_FORM },
  );
  const valid = form.name.trim() !== "" && form.command.trim() !== "";

  const set = (patch: Partial<HookFormState>) => setForm((prev) => ({ ...prev, ...patch }));

  return (
    <ModalShell onClose={onClose}>
      <div className="flex items-center gap-2">
        <h3 data-testid="hook-form-title" className="text-sm font-bold text-ink">
          {mode === "create" ? "新增 Hook" : "编辑 Hook"}
        </h3>
        <button
          type="button"
          onClick={onClose}
          aria-label="关闭弹窗"
          className="ml-auto rounded-lg p-1 text-ink-mute hover:text-ink"
        >
          <X size={14} />
        </button>
      </div>

      <form
        data-testid="hook-form-modal"
        className="mt-3 flex flex-col gap-2.5"
        onSubmit={(e) => {
          e.preventDefault();
          if (!valid) return;
          onSave(form);
        }}
      >
        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">名称 *</span>
          <input
            data-testid="hook-form-name"
            value={form.name}
            onChange={(e) => set({ name: e.target.value })}
            placeholder="例如: Lint 检查"
            className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">类型</span>
          <select
            data-testid="hook-form-type"
            value={form.type}
            onChange={(e) => set({ type: e.target.value as HookType })}
            className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-xs text-ink outline-none focus:border-accent"
          >
            {HOOK_TYPES.map((t) => (
              <option key={t} value={t}>
                {t} — {HOOK_TYPE_LABELS[t]}
              </option>
            ))}
          </select>
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">命令 *</span>
          <input
            data-testid="hook-form-command"
            value={form.command}
            onChange={(e) => set({ command: e.target.value })}
            placeholder="例如: npm run lint"
            className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 font-mono text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">参数 args (每行一个)</span>
          <textarea
            data-testid="hook-form-args"
            value={form.argsText}
            onChange={(e) => set({ argsText: e.target.value })}
            rows={2}
            placeholder={"--fix\n--cache"}
            className="resize-y rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 font-mono text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">环境变量 env (每行 KEY=VALUE)</span>
          <textarea
            data-testid="hook-form-env"
            value={form.envText}
            onChange={(e) => set({ envText: e.target.value })}
            rows={2}
            placeholder={"CI=1\nNODE_ENV=test"}
            className="resize-y rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 font-mono text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">匹配模式 matchPattern</span>
          <input
            data-testid="hook-form-pattern"
            value={form.matchPattern}
            onChange={(e) => set({ matchPattern: e.target.value })}
            placeholder="例如: **/*.{ts,tsx}"
            className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 font-mono text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-[10px] font-semibold text-ink-dim">描述</span>
          <input
            data-testid="hook-form-description"
            value={form.description}
            onChange={(e) => set({ description: e.target.value })}
            placeholder="这个 hook 做什么"
            className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-xs text-ink outline-none placeholder:text-ink-mute focus:border-accent"
          />
        </label>

        <label className="flex items-center gap-2 text-xs text-ink-dim">
          <input
            type="checkbox"
            data-testid="hook-form-enabled"
            checked={form.enabled}
            onChange={(e) => set({ enabled: e.target.checked })}
            className="size-3.5 accent-accent"
          />
          保存后立即启用
        </label>

        <div className="mt-1 flex items-center justify-end gap-2">
          <button
            type="button"
            data-testid="hook-form-cancel"
            onClick={onClose}
            className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-dim hover:text-ink"
          >
            取消
          </button>
          <button
            type="submit"
            data-testid="hook-form-save"
            disabled={!valid}
            className={
              valid
                ? "rounded-lg bg-accent px-3 py-1.5 text-xs text-white hover:bg-accent-soft"
                : "cursor-not-allowed rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-xs text-ink-mute"
            }
          >
            {mode === "create" ? "创建" : "保存"}
          </button>
        </div>
      </form>
    </ModalShell>
  );
}

// =====================================================================
// 配置弹窗 (只读详情 + 快捷编辑)
// =====================================================================

function HookConfigModal({ hook, onEdit, onClose }: { hook: HookConfig; onEdit: () => void; onClose: () => void }) {
  return (
    <ModalShell onClose={onClose}>
      <div className="flex items-center gap-2">
        <h3 data-testid="hook-config-title" className="text-sm font-bold text-ink">
          配置 Hook
        </h3>
        <button
          type="button"
          onClick={onClose}
          aria-label="关闭弹窗"
          className="ml-auto rounded-lg p-1 text-ink-mute hover:text-ink"
        >
          <X size={14} />
        </button>
      </div>

      <div data-testid="hook-config-modal" className="mt-3 flex flex-col gap-2 text-xs">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-ink">{hook.name}</span>
          <span
            data-testid={`hook-type-badge-${hook.type}`}
            className="rounded border border-line bg-bg-soft/60 px-1.5 py-0.5 font-mono text-[9px] text-ink-dim"
          >
            {hook.type}
          </span>
          {hook.enabled ? (
            <span className="text-[10px] font-semibold text-ok">已启用</span>
          ) : (
            <span className="text-[10px] text-ink-mute">已停用</span>
          )}
        </div>

        <div className="flex flex-col gap-1 rounded-lg border border-line bg-bg-soft/40 p-2.5">
          <span className="text-[10px] font-semibold text-ink-mute">命令</span>
          <code className="font-mono text-[11px] text-ink">{hook.command}</code>
          <span className="text-[10px] font-semibold text-ink-mute">参数 args</span>
          <code className="whitespace-pre-wrap font-mono text-[11px] text-ink">
            {(hook.args ?? []).join(" ") || "—"}
          </code>
          <span className="text-[10px] font-semibold text-ink-mute">环境变量 env</span>
          <code className="whitespace-pre-wrap font-mono text-[11px] text-ink">
            {Object.entries(hook.env ?? {})
              .map(([k, v]) => `${k}=${v}`)
              .join("\n") || "—"}
          </code>
          <span className="text-[10px] font-semibold text-ink-mute">匹配模式 matchPattern</span>
          <code className="font-mono text-[11px] text-ink">{hook.matchPattern ?? "—"}</code>
          {hook.description && (
            <>
              <span className="text-[10px] font-semibold text-ink-mute">描述</span>
              <span className="text-[11px] text-ink-dim">{hook.description}</span>
            </>
          )}
        </div>

        <div className="flex items-center justify-end gap-2">
          <button
            type="button"
            data-testid="hook-config-close"
            onClick={onClose}
            className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-dim hover:text-ink"
          >
            关闭
          </button>
          <button
            type="button"
            data-testid="hook-config-edit"
            onClick={onEdit}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs text-white hover:bg-accent-soft"
          >
            编辑
          </button>
        </div>
      </div>
    </ModalShell>
  );
}

// =====================================================================
// HooksSettingsPanel
// =====================================================================

export function HooksSettingsPanel() {
  const [hooks, setHooks] = useState<HookConfig[]>(() => readHooks());
  const [modal, setModal] = useState<ModalState>({ kind: "none" });

  // 跨标签页同步: 其它标签修改 localStorage 时刷新列表
  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === undefined || e.key === null || e.key === HOOKS_STORAGE_KEY) {
        setHooks(readHooks());
      }
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  /** 单一写入口: 先持久化再更新 UI (启用/禁用即时生效) */
  const persist = (next: HookConfig[]) => {
    writeHooks(next);
    setHooks(next);
  };

  const handleToggle = (id: string) => {
    persist(hooks.map((h) => (h.id === id ? { ...h, enabled: !h.enabled } : h)));
  };

  const handleSaveForm = (draft: HookFormState) => {
    if (modal.kind === "form" && modal.mode === "edit") {
      persist(hooks.map((h) => (h.id === modal.hook.id ? formToHook(draft, modal.hook.id) : h)));
    } else {
      persist([...hooks, formToHook(draft, newHookId())]);
    }
    setModal({ kind: "none" });
  };

  const handleDelete = (id: string) => {
    persist(hooks.filter((h) => h.id !== id));
    setModal({ kind: "none" });
  };

  const grouped = HOOK_TYPES.map((t) => ({
    type: t,
    items: hooks.filter((h) => h.type === t),
  })).filter((g) => g.items.length > 0);

  return (
    <div data-testid="hooks-panel" className="flex flex-col gap-3">
      {/* 顶部说明 + 新增按钮 */}
      <div className="flex items-start gap-2">
        <div>
          <h2 className="text-sm font-bold text-ink">Hooks</h2>
          <p className="mt-0.5 text-[11px] text-ink-dim">
            Git 生命周期钩子 (pre_commit / pre_push / post_merge / custom)。
            执行引擎未接入 — 配置本地持久化到本浏览器。
          </p>
        </div>
        <button
          type="button"
          data-testid="hook-action-add"
          onClick={() => setModal({ kind: "form", mode: "create" })}
          className="ml-auto flex shrink-0 items-center gap-1 rounded-lg bg-accent px-2.5 py-1.5 text-xs text-white hover:bg-accent-soft"
        >
          <Plus size={12} /> 新增 Hook
        </button>
      </div>

      {/* ① 空态 */}
      {hooks.length === 0 ? (
        <div
          data-testid="hooks-empty"
          className="rounded-xl border border-dashed border-line bg-bg-card px-4 py-10 text-center"
        >
          <p className="text-xs font-semibold text-ink-dim">暂无 Hooks</p>
          <p className="mt-1 text-[11px] text-ink-mute">
            点击右上角「新增 Hook」创建第一个钩子，例如提交前的 lint 检查。
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {grouped.map((g) => (
            <section key={g.type} data-testid={`hook-group-${g.type}`}>
              <div className="flex items-center gap-1.5 px-1 pb-1.5 text-[10px] text-ink-mute">
                <span className="font-semibold text-ink-dim">{HOOK_TYPE_LABELS[g.type]}</span>
                <span className="font-mono">{g.type}</span>
                <span className="rounded-full bg-bg-soft/60 px-1.5 py-0.5 text-[9px]">{g.items.length}</span>
              </div>
              <ul className="divide-y divide-line overflow-hidden rounded-xl border border-line">
                {g.items.map((h) => (
                  <li key={h.id} data-testid={`hook-item-${h.id}`} className="flex items-center gap-2.5 bg-bg-card px-3 py-2.5">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-1.5">
                        <span className="truncate text-xs font-semibold text-ink">{h.name}</span>
                        <span
                          data-testid={`hook-type-badge-${h.type}`}
                          className="shrink-0 rounded border border-line bg-bg-soft/60 px-1.5 py-0.5 font-mono text-[9px] text-ink-dim"
                        >
                          {h.type}
                        </span>
                        {h.enabled ? (
                          <span className="shrink-0 text-[10px] font-semibold text-ok">已启用</span>
                        ) : (
                          <span className="shrink-0 text-[10px] text-ink-mute">已停用</span>
                        )}
                      </div>
                      <div className="mt-0.5 truncate font-mono text-[10px] text-ink-mute">{h.command}</div>
                      {h.description && <div className="truncate text-[10px] text-ink-dim">{h.description}</div>}
                    </div>

                    {/* 启用 toggle (即时写 localStorage) */}
                    <button
                      type="button"
                      role="switch"
                      aria-checked={h.enabled}
                      aria-label={`启用/禁用 ${h.name}`}
                      data-testid={`hook-action-toggle-${h.id}`}
                      onClick={() => handleToggle(h.id)}
                      className={`relative h-4 w-8 shrink-0 rounded-full transition-colors ${
                        h.enabled ? "bg-accent" : "bg-line"
                      }`}
                    >
                      <span
                        className={`absolute top-0.5 size-3 rounded-full bg-white transition-all ${
                          h.enabled ? "left-4" : "left-0.5"
                        }`}
                      />
                    </button>

                    <button
                      type="button"
                      aria-label={`编辑 ${h.name}`}
                      data-testid={`hook-action-edit-${h.id}`}
                      onClick={() => setModal({ kind: "form", mode: "edit", hook: h })}
                      className="shrink-0 rounded-lg border border-line p-1.5 text-ink-mute hover:text-accent"
                    >
                      <Pencil size={12} />
                    </button>
                    <button
                      type="button"
                      aria-label={`配置 ${h.name}`}
                      data-testid={`hook-action-config-${h.id}`}
                      onClick={() => setModal({ kind: "config", hook: h })}
                      className="shrink-0 rounded-lg border border-line p-1.5 text-ink-mute hover:text-accent"
                    >
                      <Settings2 size={12} />
                    </button>
                    <button
                      type="button"
                      aria-label={`删除 ${h.name}`}
                      data-testid={`hook-action-delete-${h.id}`}
                      onClick={() => setModal({ kind: "delete", hook: h })}
                      className="shrink-0 rounded-lg border border-line p-1.5 text-ink-mute hover:text-err"
                    >
                      <Trash2 size={12} />
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}

      {/* ⑥ 底部诚实徽标 */}
      <div
        data-testid="hooks-honest-badge"
        className="flex w-fit items-center rounded-full border border-warn/40 bg-warn/10 px-2.5 py-1 text-[10px] font-semibold text-warn"
      >
        本地持久化 — Hooks 执行引擎接入后同步
      </div>

      {/* 弹窗 */}
      {modal.kind === "form" && (
        <HookFormModal
          mode={modal.mode}
          hook={modal.mode === "edit" ? modal.hook : undefined}
          onSave={handleSaveForm}
          onClose={() => setModal({ kind: "none" })}
        />
      )}
      {modal.kind === "config" && (
        <HookConfigModal
          hook={modal.hook}
          onEdit={() => setModal({ kind: "form", mode: "edit", hook: modal.hook })}
          onClose={() => setModal({ kind: "none" })}
        />
      )}
      {/* ④ 删除二次确认 */}
      {modal.kind === "delete" && (
        <ModalShell onClose={() => setModal({ kind: "none" })}>
          <div data-testid="hook-delete-confirm" className="flex flex-col gap-2">
            <h3 className="text-sm font-bold text-ink">删除 Hook?</h3>
            <p className="text-xs text-ink-dim">
              确定删除「<span className="font-semibold text-ink">{modal.hook.name}</span>
              」? 该操作不可撤销，仅影响本地配置。
            </p>
            <div className="mt-1 flex items-center justify-end gap-2">
              <button
                type="button"
                data-testid="hook-delete-confirm-cancel"
                onClick={() => setModal({ kind: "none" })}
                className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-dim hover:text-ink"
              >
                取消
              </button>
              <button
                type="button"
                data-testid="hook-delete-confirm-yes"
                onClick={() => handleDelete(modal.hook.id)}
                className="rounded-lg bg-err px-3 py-1.5 text-xs text-white hover:bg-err/80"
              >
                删除
              </button>
            </div>
          </div>
        </ModalShell>
      )}
    </div>
  );
}
