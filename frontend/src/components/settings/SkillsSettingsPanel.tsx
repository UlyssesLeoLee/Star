/*
@cypher schema=1 source_sha256=57f9a80b40d18e3f396a15e04371a4ec0f6bd0480cb69315fc07eb4f81bd2ee1
MERGE (self:File {path:"frontend/src/components/settings/SkillsSettingsPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::SkillsSettingsPanel",kind:"component"})
MERGE (addModal:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::AddSkillModal",kind:"component"})
MERGE (configModal:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::ConfigSkillModal",kind:"component"})
MERGE (uninstallModal:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::UninstallConfirmModal",kind:"component"})
MERGE (readSkills:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::readSkills",kind:"function"})
MERGE (writeSkills:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::writeSkills",kind:"function"})
MERGE (normalize:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::normalizeSkillInput",kind:"function"})
MERGE (fetchFromUrl:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::fetchSkillFromUrl",kind:"function"})
MERGE (starSkill:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::StarSkill",kind:"type"})
MERGE (store:Database {name:"localStorage:star-skills"})
MERGE (react:ExternalService {name:"react"})
MERGE (lucide:ExternalService {name:"lucide-react"})
MERGE (clsxPkg:ExternalService {name:"clsx"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:DEFINES]->(addModal)
MERGE (self)-[:DEFINES]->(configModal)
MERGE (self)-[:DEFINES]->(uninstallModal)
MERGE (self)-[:DEFINES]->(readSkills)
MERGE (self)-[:DEFINES]->(writeSkills)
MERGE (self)-[:DEFINES]->(normalize)
MERGE (self)-[:DEFINES]->(fetchFromUrl)
MERGE (self)-[:DEFINES]->(starSkill)
MERGE (self)-[:IMPORTS]->(react)
MERGE (self)-[:IMPORTS]->(lucide)
MERGE (self)-[:IMPORTS]->(clsxPkg)
MERGE (panel)-[:CALLS]->(addModal)
MERGE (panel)-[:CALLS]->(configModal)
MERGE (panel)-[:CALLS]->(uninstallModal)
MERGE (panel)-[:CALLS]->(readSkills)
MERGE (panel)-[:CALLS]->(writeSkills)
MERGE (panel)-[:USES_TYPE]->(starSkill)
MERGE (addModal)-[:CALLS]->(normalize)
MERGE (addModal)-[:CALLS]->(fetchFromUrl)
MERGE (readSkills)-[:READS]->(store)
MERGE (writeSkills)-[:WRITES]->(store)
@endcypher
*/
"use client";

// =====================================================================
// SkillsSettingsPanel — Skills 管理面板 (浏览/安装/启用/禁用/配置/卸载)
//
// 数据: 本地 localStorage key "star-skills"
//   字段: {id, name, description, version, author, capabilities,
//          configSchema?, config?, enabled, installedAt}
// 诚实徽标 (缺标比错标): 本地持久化 — Skills 注册表 API 接入后同步
// data-testid: skills-panel / skill-item-<id> / skill-action-<action>-<id>
// =====================================================================

import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import {
  Sparkles,
  Plus,
  Settings2,
  Trash2,
  Download,
  FileJson,
  Store,
  Info,
  X,
  AlertTriangle,
  Power,
} from "lucide-react";
import { clsx } from "clsx";

export const SKILLS_STORAGE_KEY = "star-skills";

export interface StarSkill {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  capabilities: string[];
  /** JSON-Schema 风格配置描述; 缺省时配置弹窗退化为原始 JSON 编辑 */
  configSchema?: Record<string, unknown>;
  /** 配置值 (配置弹窗编辑后持久化) */
  config?: Record<string, unknown>;
  enabled: boolean;
  /** ISO 8601 */
  installedAt: string;
}

// =====================================================================
// 持久化 helpers — localStorage "star-skills" (SSR 安全, 损坏数据自愈)
// =====================================================================

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

export function readSkills(): StarSkill[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(SKILLS_STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (s): s is StarSkill => isRecord(s) && typeof s.id === "string" && typeof s.name === "string",
    );
  } catch {
    return [];
  }
}

export function writeSkills(skills: StarSkill[]): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(SKILLS_STORAGE_KEY, JSON.stringify(skills));
  } catch {
    // localStorage 不可用 (隐私模式/配额) → 仅内存态, 面板仍可用
  }
}

function newSkillId(): string {
  if (typeof globalThis.crypto !== "undefined" && typeof globalThis.crypto.randomUUID === "function") {
    return `skill-${globalThis.crypto.randomUUID()}`;
  }
  return `skill-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

// =====================================================================
// 输入规范化 — URL/手动 JSON 安装共用
// =====================================================================

/** 将外部输入的技能 JSON 规范化成 StarSkill (id/installedAt 由本地生成) */
export function normalizeSkillInput(input: unknown): StarSkill {
  const raw = isRecord(input) ? input : {};
  const name = typeof raw.name === "string" ? raw.name.trim() : "";
  if (!name) throw new Error("技能 JSON 缺少有效的 name 字段");
  return {
    id: newSkillId(),
    name,
    description: typeof raw.description === "string" ? raw.description : "",
    version: typeof raw.version === "string" && raw.version.trim() ? raw.version.trim() : "1.0.0",
    author: typeof raw.author === "string" && raw.author.trim() ? raw.author.trim() : "unknown",
    capabilities: Array.isArray(raw.capabilities)
      ? raw.capabilities.filter((c): c is string => typeof c === "string")
      : [],
    configSchema: isRecord(raw.configSchema) ? raw.configSchema : undefined,
    config: isRecord(raw.config) ? raw.config : {},
    enabled: typeof raw.enabled === "boolean" ? raw.enabled : true,
    installedAt: new Date().toISOString(),
  };
}

/** 解包常见外层: {skill:{...}} / {skills:[...]} */
function unwrapSkillPayload(data: unknown): unknown {
  if (isRecord(data) && isRecord(data.skill)) return data.skill;
  if (isRecord(data) && Array.isArray(data.skills) && data.skills.length > 0) return data.skills[0];
  return data;
}

/** GET 技能 JSON URL 并规范化; 网络/HTTP/字段错误均抛 Error */
export async function fetchSkillFromUrl(url: string): Promise<StarSkill> {
  const res = await fetch(url, { headers: { accept: "application/json" } });
  if (!res.ok) throw new Error(`下载失败 (HTTP ${res.status})`);
  const data: unknown = await res.json();
  return normalizeSkillInput(unwrapSkillPayload(data));
}

// =====================================================================
// configSchema 表单
// =====================================================================

export interface ConfigPropertySchema {
  type?: string;
  title?: string;
  description?: string;
  default?: unknown;
  enum?: unknown[];
}

export function getConfigProperties(
  schema: Record<string, unknown>,
): Record<string, ConfigPropertySchema> | null {
  const props = schema.properties;
  if (!isRecord(props)) return null;
  return props as Record<string, ConfigPropertySchema>;
}

function toInputString(v: unknown): string {
  if (v === undefined || v === null) return "";
  if (typeof v === "string") return v;
  if (typeof v === "boolean") return v ? "true" : "false";
  return String(v);
}

// =====================================================================
// 通用弹窗壳
// =====================================================================

function Modal({
  testId,
  title,
  onClose,
  children,
}: {
  testId: string;
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <div
      data-testid={testId}
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center bg-black/50 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg space-y-3 rounded-xl border border-line bg-bg-card p-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-ink">{title}</h3>
          <button
            type="button"
            aria-label="关闭"
            onClick={onClose}
            className="rounded p-1 text-ink-mute hover:text-ink"
          >
            <X size={14} />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

// =====================================================================
// 新增技能弹窗 — URL 安装 / 手动 JSON / 官方目录占位
// =====================================================================

type AddTab = "url" | "json" | "catalog";

const ADD_TABS: { id: AddTab; label: string; icon: ReactNode }[] = [
  { id: "url", label: "从 URL 安装", icon: <Download size={12} /> },
  { id: "json", label: "手动 JSON", icon: <FileJson size={12} /> },
  { id: "catalog", label: "官方目录", icon: <Store size={12} /> },
];

function AddSkillModal({
  onClose,
  onInstall,
}: {
  onClose: () => void;
  onInstall: (skill: StarSkill) => void;
}) {
  const [tab, setTab] = useState<AddTab>("url");
  const [url, setUrl] = useState("");
  const [jsonText, setJsonText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit() {
    setError(null);
    if (tab === "catalog") {
      setError("官方目录尚未接入 — 请使用 URL 或 JSON 方式安装");
      return;
    }
    setBusy(true);
    try {
      if (tab === "url") {
        const trimmed = url.trim();
        if (!trimmed) {
          setError("请输入技能 JSON 的 URL");
          return;
        }
        try {
          new URL(trimmed);
        } catch {
          setError("URL 格式无效");
          return;
        }
        onInstall(await fetchSkillFromUrl(trimmed));
      } else {
        let parsed: unknown;
        try {
          parsed = JSON.parse(jsonText);
        } catch {
          setError("JSON 解析失败 — 请检查输入");
          return;
        }
        onInstall(normalizeSkillInput(parsed));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "安装失败");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal testId="skills-add-modal" title="新增技能" onClose={onClose}>
      <div className="flex items-center gap-1 border-b border-line pb-2">
        {ADD_TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            data-testid={`skills-add-tab-${t.id}`}
            aria-pressed={tab === t.id}
            onClick={() => setTab(t.id)}
            className={clsx(
              "flex items-center gap-1 rounded px-2 py-1 text-[11px]",
              tab === t.id ? "bg-accent/10 font-semibold text-accent" : "text-ink-mute hover:text-ink",
            )}
          >
            {t.icon}
            {t.label}
          </button>
        ))}
      </div>

      {tab === "url" && (
        <label className="grid gap-1 text-xs text-ink-dim">
          技能 JSON URL
          <input
            data-testid="skills-add-url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://…/skill.json"
            className="rounded border border-line bg-bg px-3 py-2 font-mono text-sm"
          />
          <span className="text-[10px] text-ink-mute">将 GET 该 URL 并解析技能 JSON（支持 {`{skill:{…}}`} / {`{skills:[…]}`} 包装）。</span>
        </label>
      )}

      {tab === "json" && (
        <label className="grid gap-1 text-xs text-ink-dim">
          技能 JSON
          <textarea
            data-testid="skills-add-json"
            value={jsonText}
            onChange={(e) => setJsonText(e.target.value)}
            rows={8}
            placeholder={'{"name":"tdd","version":"1.0.0","author":"star","capabilities":["test"],"configSchema":{"properties":{}}}'}
            className="rounded border border-line bg-bg px-3 py-2 font-mono text-xs"
          />
        </label>
      )}

      {tab === "catalog" && (
        <div
          data-testid="skills-add-catalog-placeholder"
          className="flex items-start gap-2 rounded border border-dashed border-line p-3 text-xs text-ink-mute"
        >
          <Store size={14} className="mt-0.5 shrink-0 text-accent" />
          <span>官方技能目录尚未接入 — 接入后可浏览并一键安装官方 Skills。当前请使用 URL 或 JSON 方式安装。</span>
        </div>
      )}

      {error && (
        <p data-testid="skills-add-error" role="alert" className="text-xs text-warn">
          {error}
        </p>
      )}

      <div className="flex justify-end gap-2">
        <button type="button" data-testid="skills-add-cancel" onClick={onClose} className="btn-ghost text-xs">
          取消
        </button>
        <button
          type="button"
          data-testid="skills-add-submit"
          onClick={submit}
          disabled={busy || tab === "catalog"}
          className="btn text-xs disabled:cursor-not-allowed disabled:opacity-50"
        >
          {busy ? "安装中…" : "安装"}
        </button>
      </div>
    </Modal>
  );
}

// =====================================================================
// 配置弹窗 — 有 configSchema 渲染表单; 无 schema 显示原始 JSON
// =====================================================================

function ConfigSkillModal({
  skill,
  onClose,
  onSave,
}: {
  skill: StarSkill;
  onClose: () => void;
  onSave: (next: StarSkill) => void;
}) {
  const properties = skill.configSchema ? getConfigProperties(skill.configSchema) : null;
  const [values, setValues] = useState<Record<string, string | boolean>>(() => {
    if (!properties) return {};
    const init: Record<string, string | boolean> = {};
    for (const [key, prop] of Object.entries(properties)) {
      const existing = skill.config?.[key] ?? prop.default;
      init[key] = typeof existing === "boolean" ? existing : toInputString(existing);
    }
    return init;
  });
  const [rawJson, setRawJson] = useState(() => JSON.stringify(skill.config ?? {}, null, 2));
  const [error, setError] = useState<string | null>(null);

  function save() {
    setError(null);
    if (!properties) {
      let parsed: unknown;
      try {
        parsed = JSON.parse(rawJson);
      } catch {
        setError("JSON 解析失败 — 配置未保存");
        return;
      }
      if (!isRecord(parsed)) {
        setError("配置必须是 JSON 对象");
        return;
      }
      onSave({ ...skill, config: parsed });
      return;
    }
    const config: Record<string, unknown> = {};
    for (const [key, prop] of Object.entries(properties)) {
      const rawVal = values[key];
      if (prop.type === "boolean") {
        config[key] = rawVal === true || rawVal === "true";
      } else if (prop.type === "number" || prop.type === "integer") {
        const text = toInputString(rawVal);
        if (text === "") {
          config[key] = null;
        } else {
          const num = Number(text);
          config[key] = Number.isNaN(num) ? text : num;
        }
      } else {
        config[key] = typeof rawVal === "string" ? rawVal : toInputString(rawVal);
      }
    }
    onSave({ ...skill, config });
  }

  return (
    <Modal testId="skills-config-modal" title={`配置 — ${skill.name}`} onClose={onClose}>
      {properties ? (
        <div className="grid gap-3">
          {Object.entries(properties).map(([key, prop]) => {
            const id = `skills-config-field-${key}`;
            const label = prop.title ?? key;
            const val = values[key];
            return (
              <label key={key} className="grid gap-1 text-xs text-ink-dim">
                <span>
                  {label}
                  {prop.description && (
                    <span className="ml-1.5 text-[10px] text-ink-mute">{prop.description}</span>
                  )}
                </span>
                {prop.type === "boolean" ? (
                  <input
                    data-testid={id}
                    type="checkbox"
                    checked={val === true || val === "true"}
                    onChange={(e) => setValues((v) => ({ ...v, [key]: e.target.checked }))}
                    className="justify-self-start size-4"
                  />
                ) : Array.isArray(prop.enum) ? (
                  <select
                    data-testid={id}
                    value={typeof val === "string" ? val : toInputString(val)}
                    onChange={(e) => setValues((v) => ({ ...v, [key]: e.target.value }))}
                    className="rounded border border-line bg-bg px-3 py-2 text-sm"
                  >
                    <option value="">— 未设置 —</option>
                    {prop.enum.map((opt) => (
                      <option key={String(opt)} value={String(opt)}>
                        {String(opt)}
                      </option>
                    ))}
                  </select>
                ) : prop.type === "number" || prop.type === "integer" ? (
                  <input
                    data-testid={id}
                    type="number"
                    value={typeof val === "string" ? val : toInputString(val)}
                    onChange={(e) => setValues((v) => ({ ...v, [key]: e.target.value }))}
                    className="rounded border border-line bg-bg px-3 py-2 font-mono text-sm"
                  />
                ) : (
                  <input
                    data-testid={id}
                    type="text"
                    value={typeof val === "string" ? val : toInputString(val)}
                    onChange={(e) => setValues((v) => ({ ...v, [key]: e.target.value }))}
                    className="rounded border border-line bg-bg px-3 py-2 font-mono text-sm"
                  />
                )}
              </label>
            );
          })}
        </div>
      ) : (
        <label className="grid gap-1 text-xs text-ink-dim">
          原始 JSON 配置
          <textarea
            data-testid="skills-config-json"
            value={rawJson}
            onChange={(e) => setRawJson(e.target.value)}
            rows={8}
            className="rounded border border-line bg-bg px-3 py-2 font-mono text-xs"
          />
          <span className="text-[10px] text-ink-mute">该技能未提供 configSchema — 直接编辑原始 JSON 配置。</span>
        </label>
      )}

      {error && (
        <p data-testid="skills-config-error" role="alert" className="text-xs text-warn">
          {error}
        </p>
      )}

      <div className="flex justify-end gap-2">
        <button type="button" data-testid="skills-config-cancel" onClick={onClose} className="btn-ghost text-xs">
          取消
        </button>
        <button type="button" data-testid="skills-config-save" onClick={save} className="btn text-xs">
          保存配置
        </button>
      </div>
    </Modal>
  );
}

// =====================================================================
// 卸载确认弹窗
// =====================================================================

function UninstallConfirmModal({
  skill,
  onClose,
  onConfirm,
}: {
  skill: StarSkill;
  onClose: () => void;
  onConfirm: () => void;
}) {
  return (
    <Modal testId="skills-uninstall-modal" title="卸载技能" onClose={onClose}>
      <div className="flex items-start gap-2 rounded border border-warn/40 bg-warn/10 p-2.5 text-xs text-warn">
        <AlertTriangle size={14} className="mt-0.5 shrink-0" />
        <span>
          确认卸载 <span className="font-mono font-semibold">{skill.name}</span> (v{skill.version})？
          本地配置将一并删除。
        </span>
      </div>
      <div className="flex justify-end gap-2">
        <button type="button" data-testid="skills-uninstall-cancel" onClick={onClose} className="btn-ghost text-xs">
          取消
        </button>
        <button
          type="button"
          data-testid="skills-uninstall-confirm"
          onClick={onConfirm}
          className="rounded-lg border border-danger/50 bg-danger/10 px-3 py-1.5 text-xs text-danger hover:bg-danger/20"
        >
          确认卸载
        </button>
      </div>
    </Modal>
  );
}

// =====================================================================
// SkillsSettingsPanel — 面板主体
// =====================================================================

export function SkillsSettingsPanel() {
  const [skills, setSkills] = useState<StarSkill[]>([]);
  const [hydrated, setHydrated] = useState(false);
  const [addOpen, setAddOpen] = useState(false);
  const [configId, setConfigId] = useState<string | null>(null);
  const [uninstallId, setUninstallId] = useState<string | null>(null);

  // 挂载后从 localStorage 装载 (SSR 首帧渲染 null → 无 hydration 失配)
  useEffect(() => {
    setSkills(readSkills());
    setHydrated(true);
  }, []);

  /** 统一提交: 更新内存态 + 立即写回 localStorage (toggle 即时生效) */
  function commit(next: StarSkill[]) {
    setSkills(next);
    writeSkills(next);
  }

  const configTarget = configId ? skills.find((s) => s.id === configId) ?? null : null;
  const uninstallTarget = uninstallId ? skills.find((s) => s.id === uninstallId) ?? null : null;

  return (
    <div className="space-y-4" data-testid="skills-panel">
      <div className="card p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="flex items-center gap-2 text-base font-semibold text-ink">
              <Sparkles size={16} className="text-accent" /> Skills
            </h2>
            <p className="mt-1 text-sm text-ink-dim">浏览 / 安装 / 启用 / 禁用 / 配置本地 Skills。</p>
          </div>
          <button type="button" data-testid="skills-add" onClick={() => setAddOpen(true)} className="btn text-xs">
            <Plus size={12} /> 新增技能
          </button>
        </div>
      </div>

      {hydrated &&
        (skills.length === 0 ? (
          <p data-testid="skills-empty" className="text-sm text-ink-mute">
            暂无已安装技能 — 点击「新增技能」从 URL 或 JSON 安装。
          </p>
        ) : (
          <ul className="space-y-2" data-testid="skills-list">
            {skills.map((skill) => (
              <li
                key={skill.id}
                data-testid={`skill-item-${skill.id}`}
                className={clsx(
                  "card flex flex-wrap items-center gap-3 p-3",
                  !skill.enabled && "opacity-60",
                )}
              >
                <span className="grid size-8 shrink-0 place-items-center rounded border border-line bg-bg-soft text-accent">
                  <Sparkles size={14} />
                </span>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-ink">{skill.name}</span>
                    <span className="rounded border border-line px-1.5 py-0.5 font-mono text-[9px] text-ink-mute">
                      v{skill.version}
                    </span>
                    <span className="text-[10px] text-ink-mute">by {skill.author}</span>
                  </div>
                  {skill.description && (
                    <p className="mt-0.5 truncate text-[11px] text-ink-mute">{skill.description}</p>
                  )}
                  {skill.capabilities.length > 0 && (
                    <div className="mt-1 flex flex-wrap gap-1" aria-label="能力">
                      {skill.capabilities.map((cap) => (
                        <span
                          key={cap}
                          className="rounded border border-accent/40 bg-accent/10 px-1.5 py-0.5 font-mono text-[9px] text-accent"
                        >
                          {cap}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={skill.enabled}
                  aria-label={`启用/禁用 ${skill.name}`}
                  data-testid={`skill-action-toggle-${skill.id}`}
                  onClick={() =>
                    commit(skills.map((s) => (s.id === skill.id ? { ...s, enabled: !s.enabled } : s)))
                  }
                  className={clsx(
                    "flex items-center gap-1 rounded border px-2 py-1 font-mono text-[10px]",
                    skill.enabled
                      ? "border-ok/50 bg-ok/10 text-ok"
                      : "border-line text-ink-mute hover:text-ink",
                  )}
                >
                  <Power size={11} />
                  {skill.enabled ? "已启用" : "已禁用"}
                </button>
                <button
                  type="button"
                  data-testid={`skill-action-config-${skill.id}`}
                  aria-label={`配置 ${skill.name}`}
                  onClick={() => setConfigId(skill.id)}
                  className="rounded border border-line p-1.5 text-ink-mute transition-colors hover:border-accent/50 hover:text-accent"
                >
                  <Settings2 size={13} />
                </button>
                <button
                  type="button"
                  data-testid={`skill-action-uninstall-${skill.id}`}
                  aria-label={`卸载 ${skill.name}`}
                  onClick={() => setUninstallId(skill.id)}
                  className="rounded border border-line p-1.5 text-ink-mute transition-colors hover:border-danger/50 hover:text-danger"
                >
                  <Trash2 size={13} />
                </button>
              </li>
            ))}
          </ul>
        ))}

      {/* 诚实徽标 (缺标比错标): 本地数据不冒充后端注册表 */}
      <div
        data-testid="skills-honest-badge"
        className="flex items-center gap-1.5 text-[10px] text-ink-mute"
      >
        <Info size={11} className="shrink-0" />
        本地持久化 — Skills 注册表 API 接入后同步
      </div>

      {addOpen && (
        <AddSkillModal
          onClose={() => setAddOpen(false)}
          onInstall={(skill) => {
            commit([...skills, skill]);
            setAddOpen(false);
          }}
        />
      )}
      {configTarget && (
        <ConfigSkillModal
          skill={configTarget}
          onClose={() => setConfigId(null)}
          onSave={(next) => {
            commit(skills.map((s) => (s.id === next.id ? next : s)));
            setConfigId(null);
          }}
        />
      )}
      {uninstallTarget && (
        <UninstallConfirmModal
          skill={uninstallTarget}
          onClose={() => setUninstallId(null)}
          onConfirm={() => {
            commit(skills.filter((s) => s.id !== uninstallTarget.id));
            setUninstallId(null);
          }}
        />
      )}
    </div>
  );
}
