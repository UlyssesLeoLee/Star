/*
@cypher schema=1 source_sha256=8632b2607f64327b6fd41a634fd8f19ed5e5d2be95a695fc4f584cc78301da20
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx"})
MERGE (skills:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx::SkillsSettingsPanel",kind:"component"})
MERGE (plugins:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx::PluginsSettingsPanel",kind:"component"})
MERGE (registry:File {path:"frontend/src/lib/settings/capabilityRegistry.ts"})
MERGE (self)-[:DEFINES]->(skills)
MERGE (self)-[:DEFINES]->(plugins)
MERGE (self)-[:IMPORTS]->(registry)
MERGE (skills)-[:CALLS]->(registry)
MERGE (plugins)-[:CALLS]->(registry)
@endcypher
*/
"use client";

// =====================================================================
// SkillsSettingsPanel + PluginsSettingsPanel (per WBS §23.33-B, 2026-11-13)
// Skills: 包名/版本/作用域/启停; Plugins: 权限清单/启停。
// =====================================================================

import { useState } from "react";
import { Sparkles, Blocks, Plus, Trash2, Power, ShieldCheck } from "lucide-react";
import { clsx } from "clsx";
import {
  capabilityRegistryApi,
  useCapabilityRegistry,
  type CapabilityScope,
} from "@/lib/settings/capabilityRegistry";
import { useTranslation } from "@/lib/i18n";

export function SkillsSettingsPanel() {
  const { t, tx } = useTranslation();
  const registry = useCapabilityRegistry();
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState("");
  const [version, setVersion] = useState("1.0.0");
  const [description, setDescription] = useState("");
  const [scope, setScope] = useState<CapabilityScope>("user");
  const [formError, setFormError] = useState<string | null>(null);

  function submitAdd() {
    if (!name.trim()) {
      setFormError(t.settingsCapabilities.skills.nameRequired);
      return;
    }
    capabilityRegistryApi.addSkill({ name: name.trim(), version: version.trim() || "1.0.0", scope, enabled: true, description: description.trim() });
    setName(""); setVersion("1.0.0"); setDescription(""); setFormError(null); setFormOpen(false);
  }

  return (
    <div className="space-y-4" data-testid="advanced-settings-skills-panel">
      <div className="card p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold text-ink flex items-center gap-2">
              <Sparkles size={16} className="text-accent" /> {t.settingsCapabilities.skills.title}
            </h2>
            <p className="mt-1 text-sm text-ink-dim">{t.settingsCapabilities.skills.description}</p>
          </div>
          <button type="button" data-testid="skills-add" onClick={() => setFormOpen((v) => !v)} className="btn text-xs">
            <Plus size={12} /> {t.settingsCapabilities.skills.addSkill}
          </button>
        </div>
      </div>

      {formOpen && (
        <form data-testid="skills-add-form" onSubmit={(e) => { e.preventDefault(); submitAdd(); }} className="card p-4 grid gap-3 md:grid-cols-2">
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.skills.nameLabel}
            <input data-testid="skills-form-name" value={name} onChange={(e) => setName(e.target.value)} placeholder={t.settingsCapabilities.skills.namePlaceholder} className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.skills.versionLabel}
            <input data-testid="skills-form-version" value={version} onChange={(e) => setVersion(e.target.value)} className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.skills.descriptionLabel}
            <input data-testid="skills-form-description" value={description} onChange={(e) => setDescription(e.target.value)} className="rounded border border-line bg-bg px-3 py-2 text-sm" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.skills.scopeLabel}
            <select data-testid="skills-form-scope" value={scope} onChange={(e) => setScope(e.target.value as CapabilityScope)} className="rounded border border-line bg-bg px-3 py-2 text-sm">
              <option value="user">{t.settingsCapabilities.mcp.scopeUser}</option>
              <option value="project">{t.settingsCapabilities.mcp.scopeProject}</option>
            </select>
          </label>
          {formError && <p className="text-xs text-warn md:col-span-2" role="alert">{formError}</p>}
          <div className="flex gap-2 md:col-span-2">
            <button type="submit" data-testid="skills-form-submit" className="btn text-xs"><Plus size={12} /> {t.settingsCapabilities.skills.addButton}</button>
            <button type="button" onClick={() => setFormOpen(false)} className="btn-ghost text-xs">{t.settingsCapabilities.mcp.cancel}</button>
          </div>
        </form>
      )}

      {registry.skills.length === 0 ? (
        <p className="text-sm text-ink-mute">{t.settingsCapabilities.skills.empty}</p>
      ) : (
        <ul className="space-y-2" data-testid="skills-list">
          {registry.skills.map((skill) => (
            <li key={skill.id} data-testid={`skill-${skill.id}`} className={clsx("card p-3 flex flex-wrap items-center gap-3", !skill.enabled && "opacity-60")}>
              <span className="grid size-8 shrink-0 place-items-center rounded border border-line bg-bg-soft text-accent"><Sparkles size={14} /></span>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-semibold text-ink font-mono">{skill.name}</span>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-line text-ink-mute">v{skill.version}</span>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-accent/40 bg-accent/10 text-accent">
                    {skill.scope === "user" ? t.settingsCapabilities.mcp.scopeUserShort : t.settingsCapabilities.mcp.scopeProjectShort}
                  </span>
                </div>
                {skill.description && <p className="mt-0.5 truncate text-[11px] text-ink-mute">{skill.description}</p>}
              </div>
              <button type="button" data-testid={`skill-toggle-${skill.id}`} onClick={() => capabilityRegistryApi.toggleSkill(skill.id)} aria-pressed={skill.enabled}
                className={clsx("rounded border px-2 py-1 text-[10px] font-mono flex items-center gap-1", skill.enabled ? "border-ok/50 bg-ok/10 text-ok" : "border-line text-ink-mute hover:text-ink")}>
                <Power size={11} />{skill.enabled ? t.settingsCapabilities.mcp.on : t.settingsCapabilities.mcp.off}
              </button>
              <button type="button" data-testid={`skill-remove-${skill.id}`} onClick={() => capabilityRegistryApi.removeSkill(skill.id)}
                aria-label={tx(t.settingsCapabilities.skills.removeAria, { name: skill.name })} className="rounded border border-line p-1.5 text-ink-mute hover:text-danger hover:border-danger/50 transition-colors">
                <Trash2 size={13} />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function PluginsSettingsPanel() {
  const { t, tx } = useTranslation();
  const registry = useCapabilityRegistry();
  const [formOpen, setFormOpen] = useState(false);
  const [name, setName] = useState("");
  const [version, setVersion] = useState("1.0.0");
  const [permissions, setPermissions] = useState("");
  const [description, setDescription] = useState("");
  const [scope, setScope] = useState<CapabilityScope>("project");
  const [formError, setFormError] = useState<string | null>(null);

  function submitAdd() {
    if (!name.trim()) {
      setFormError(t.settingsCapabilities.plugins.nameRequired);
      return;
    }
    capabilityRegistryApi.addPlugin({
      name: name.trim(),
      version: version.trim() || "1.0.0",
      scope,
      enabled: true,
      permissions: permissions.split(/[,\s]+/).filter(Boolean),
      description: description.trim(),
    });
    setName(""); setVersion("1.0.0"); setPermissions(""); setDescription(""); setFormError(null); setFormOpen(false);
  }

  return (
    <div className="space-y-4" data-testid="advanced-settings-plugins-panel">
      <div className="card p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold text-ink flex items-center gap-2">
              <Blocks size={16} className="text-accent" /> {t.settingsCapabilities.plugins.title}
            </h2>
            <p className="mt-1 text-sm text-ink-dim">{t.settingsCapabilities.plugins.description}</p>
          </div>
          <button type="button" data-testid="plugins-add" onClick={() => setFormOpen((v) => !v)} className="btn text-xs">
            <Plus size={12} /> {t.settingsCapabilities.plugins.addPlugin}
          </button>
        </div>
      </div>

      {formOpen && (
        <form data-testid="plugins-add-form" onSubmit={(e) => { e.preventDefault(); submitAdd(); }} className="card p-4 grid gap-3 md:grid-cols-2">
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.plugins.nameLabel}
            <input data-testid="plugins-form-name" value={name} onChange={(e) => setName(e.target.value)} placeholder={t.settingsCapabilities.plugins.namePlaceholder} className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.plugins.versionLabel}
            <input data-testid="plugins-form-version" value={version} onChange={(e) => setVersion(e.target.value)} className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.plugins.permissionsLabel}
            <input data-testid="plugins-form-permissions" value={permissions} onChange={(e) => setPermissions(e.target.value)} placeholder={t.settingsCapabilities.plugins.permissionsPlaceholder} className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono" />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">{t.settingsCapabilities.plugins.scopeLabel}
            <select data-testid="plugins-form-scope" value={scope} onChange={(e) => setScope(e.target.value as CapabilityScope)} className="rounded border border-line bg-bg px-3 py-2 text-sm">
              <option value="project">{t.settingsCapabilities.mcp.scopeProject}</option>
              <option value="user">{t.settingsCapabilities.mcp.scopeUser}</option>
            </select>
          </label>
          <label className="grid gap-1 text-xs text-ink-dim md:col-span-2">{t.settingsCapabilities.plugins.descriptionLabel}
            <input data-testid="plugins-form-description" value={description} onChange={(e) => setDescription(e.target.value)} className="rounded border border-line bg-bg px-3 py-2 text-sm" />
          </label>
          {formError && <p className="text-xs text-warn md:col-span-2" role="alert">{formError}</p>}
          <div className="flex gap-2 md:col-span-2">
            <button type="submit" data-testid="plugins-form-submit" className="btn text-xs"><Plus size={12} /> {t.settingsCapabilities.plugins.addButton}</button>
            <button type="button" onClick={() => setFormOpen(false)} className="btn-ghost text-xs">{t.settingsCapabilities.mcp.cancel}</button>
          </div>
        </form>
      )}

      {registry.plugins.length === 0 ? (
        <p className="text-sm text-ink-mute">{t.settingsCapabilities.plugins.empty}</p>
      ) : (
        <ul className="space-y-2" data-testid="plugins-list">
          {registry.plugins.map((plugin) => (
            <li key={plugin.id} data-testid={`plugin-${plugin.id}`} className={clsx("card p-3 flex flex-wrap items-center gap-3", !plugin.enabled && "opacity-60")}>
              <span className="grid size-8 shrink-0 place-items-center rounded border border-line bg-bg-soft text-accent"><Blocks size={14} /></span>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-sm font-semibold text-ink font-mono">{plugin.name}</span>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-line text-ink-mute">v{plugin.version}</span>
                  <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-accent/40 bg-accent/10 text-accent">
                    {plugin.scope === "user" ? t.settingsCapabilities.mcp.scopeUserShort : t.settingsCapabilities.mcp.scopeProjectShort}
                  </span>
                </div>
                {plugin.description && <p className="mt-0.5 truncate text-[11px] text-ink-mute">{plugin.description}</p>}
                {plugin.permissions.length > 0 && (
                  <div className="mt-1 flex flex-wrap gap-1" aria-label={t.settingsCapabilities.plugins.permissionsLabel}>
                    {plugin.permissions.map((p) => (
                      <span key={p} className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-warn/40 bg-warn/10 text-warn flex items-center gap-1">
                        <ShieldCheck size={9} />{p}
                      </span>
                    ))}
                  </div>
                )}
              </div>
              <button type="button" data-testid={`plugin-toggle-${plugin.id}`} onClick={() => capabilityRegistryApi.togglePlugin(plugin.id)} aria-pressed={plugin.enabled}
                className={clsx("rounded border px-2 py-1 text-[10px] font-mono flex items-center gap-1", plugin.enabled ? "border-ok/50 bg-ok/10 text-ok" : "border-line text-ink-mute hover:text-ink")}>
                <Power size={11} />{plugin.enabled ? t.settingsCapabilities.mcp.on : t.settingsCapabilities.mcp.off}
              </button>
              <button type="button" data-testid={`plugin-remove-${plugin.id}`} onClick={() => capabilityRegistryApi.removePlugin(plugin.id)}
                aria-label={tx(t.settingsCapabilities.plugins.removeAria, { name: plugin.name })} className="rounded border border-line p-1.5 text-ink-mute hover:text-danger hover:border-danger/50 transition-colors">
                <Trash2 size={13} />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
