/*
@cypher schema=1 source_sha256=6dc2c377573e0552835dc68eaa8532dba67926be2ab240d2cbd25a895eb597cc
MERGE (self:File {path:"frontend/src/app/worktree/GitBindingPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/app/worktree/GitBindingPanel.tsx::GitBindingPanel",kind:"component"})
MERGE (store:File {path:"frontend/src/lib/group/gitBindingStore.ts"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:IMPORTS]->(store)
MERGE (panel)-[:CALLS]->(store)
@endcypher
*/
"use client";

// =====================================================================
// GitBindingPanel — Git 仓库绑定 (per WBS §23.34, 2026-11-13 用户发令)
// 未绑定: 3 张 provider 卡 (GitHub/GitLab/自托管) + 绑定表单;
// 已绑定: 绑定行 (provider 徽章 + 远端 + 本地目录) + 解绑。
// 自动识别结果 (Worktree→本地目录→远端 branch) 由 worktree 页渲染。
// =====================================================================

import { useState } from "react";
import { GitBranch, GitFork, Server, Plus, Trash2, Folder } from "lucide-react";
import { clsx } from "clsx";
import {
  GIT_PROVIDERS,
  gitBindingApi,
  useGitBindings,
  type GitProviderId,
} from "@/lib/group/gitBindingStore";
import { useTranslation } from "@/lib/i18n";

const PROVIDER_ICONS: Record<GitProviderId, typeof GitBranch> = {
  github: GitFork,
  gitlab: GitBranch,
  "self-host": Server,
};

export function GitBindingPanel() {
  const { t, tx } = useTranslation();
  const bindings = useGitBindings();
  const [selectedProvider, setSelectedProvider] = useState<GitProviderId | null>(null);
  const [form, setForm] = useState({
    full_name: "",
    base_url: "",
    local_path: "",
    default_branch: "main",
  });
  const [formError, setFormError] = useState<string | null>(null);

  const provider = selectedProvider ? GIT_PROVIDERS.find((p) => p.id === selectedProvider) ?? null : null;

  function submitBind() {
    if (!provider) return;
    const fullName = form.full_name.trim();
    if (!fullName || !fullName.includes("/")) {
      setFormError(t.gitBinding.fullNameRequired);
      return;
    }
    if (provider.host === null && !form.base_url.trim()) {
      setFormError(t.gitBinding.baseUrlRequired);
      return;
    }
    if (!form.local_path.trim()) {
      setFormError(t.gitBinding.localPathRequired);
      return;
    }
    const baseUrl = provider.host
      ? `https://${provider.host}`
      : form.base_url.trim().replace(/\/+$/, "");
    gitBindingApi.addBinding({
      provider: provider.id,
      base_url: baseUrl,
      full_name: fullName,
      local_path: form.local_path.trim(),
      default_branch: form.default_branch.trim() || "main",
    });
    setSelectedProvider(null);
    setForm({ full_name: "", base_url: "", local_path: "", default_branch: "main" });
    setFormError(null);
  }

  return (
    <section className="card space-y-4" data-testid="git-binding-panel">
      <header>
        <h2 className="text-base font-semibold text-ink flex items-center gap-2">
          <GitBranch size={16} className="text-accent" /> {t.gitBinding.title}
        </h2>
        <p className="mt-1 text-sm text-ink-dim">{t.gitBinding.description}</p>
      </header>

      {bindings.length === 0 && !selectedProvider && (
        <div className="grid gap-3 sm:grid-cols-3" data-testid="git-provider-cards">
          {GIT_PROVIDERS.map((p) => {
            const Icon = PROVIDER_ICONS[p.id];
            return (
              <button
                key={p.id}
                type="button"
                data-testid={`git-provider-${p.id}`}
                onClick={() => setSelectedProvider(p.id)}
                className="card p-4 text-left transition-all hover:border-accent hover:translate-x-[-2px] hover:translate-y-[-2px]"
              >
                <span className="grid size-9 place-items-center rounded-lg border border-line bg-bg-soft text-accent">
                  <Icon size={18} />
                </span>
                <div className="mt-2 text-sm font-semibold text-ink">{p.label}</div>
                <div className="mt-1 text-[11px] text-ink-mute font-mono">
                  {p.host ?? t.gitBinding.selfHostHint}
                </div>
              </button>
            );
          })}
        </div>
      )}

      {selectedProvider && provider && (
        <form
          data-testid="git-binding-form"
          onSubmit={(e) => { e.preventDefault(); submitBind(); }}
          className="grid gap-3 md:grid-cols-2"
        >
          <div className="md:col-span-2 text-xs font-semibold text-accent">
            {t.gitBinding.bindingTo} {provider.label}
          </div>
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.gitBinding.fullNameLabel}
            <input
              data-testid="git-form-full-name"
              value={form.full_name}
              onChange={(e) => setForm((f) => ({ ...f, full_name: e.target.value }))}
              placeholder="owner/repo"
              className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
            />
          </label>
          {provider.host === null && (
            <label className="grid gap-1 text-xs text-ink-dim">
              {t.gitBinding.baseUrlLabel}
              <input
                data-testid="git-form-base-url"
                value={form.base_url}
                onChange={(e) => setForm((f) => ({ ...f, base_url: e.target.value }))}
                placeholder="https://git.example.com"
                className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
              />
            </label>
          )}
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.gitBinding.localPathLabel}
            <input
              data-testid="git-form-local-path"
              value={form.local_path}
              onChange={(e) => setForm((f) => ({ ...f, local_path: e.target.value }))}
              placeholder="D:/Star"
              className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
            />
          </label>
          <label className="grid gap-1 text-xs text-ink-dim">
            {t.gitBinding.defaultBranchLabel}
            <input
              data-testid="git-form-default-branch"
              value={form.default_branch}
              onChange={(e) => setForm((f) => ({ ...f, default_branch: e.target.value }))}
              className="rounded border border-line bg-bg px-3 py-2 text-sm font-mono"
            />
          </label>
          {formError && <p className="text-xs text-warn md:col-span-2" role="alert" data-testid="git-form-error">{formError}</p>}
          <div className="flex gap-2 md:col-span-2">
            <button type="submit" data-testid="git-form-submit" className="btn text-xs">
              <Plus size={12} /> {t.gitBinding.bindButton}
            </button>
            <button type="button" onClick={() => { setSelectedProvider(null); setFormError(null); }} className="btn-ghost text-xs">
              {t.gitBinding.cancel}
            </button>
          </div>
        </form>
      )}

      {bindings.length > 0 && (
        <ul className="space-y-2" data-testid="git-binding-list">
          {bindings.map((binding) => {
            const def = GIT_PROVIDERS.find((p) => p.id === binding.provider);
            const Icon = def ? PROVIDER_ICONS[binding.provider] : Server;
            return (
              <li
                key={binding.id}
                data-testid={`git-binding-${binding.id}`}
                className="flex flex-wrap items-center gap-3 rounded border border-line bg-bg-soft/40 p-3"
              >
                <span className="grid size-8 place-items-center rounded border border-line bg-bg-card text-accent">
                  <Icon size={14} />
                </span>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="text-sm font-semibold text-ink font-mono">{binding.full_name}</span>
                    <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-accent/40 bg-accent/10 text-accent">
                      {def?.label ?? binding.provider}
                    </span>
                  </div>
                  <div className="mt-0.5 flex items-center gap-1.5 text-[11px] font-mono text-ink-mute">
                    <Folder size={10} /> {binding.local_path}
                    <span>·</span>
                    <GitBranch size={10} /> {binding.default_branch}
                  </div>
                </div>
                {!selectedProvider && (
                  <button
                    type="button"
                    data-testid={`git-binding-add-more`}
                    onClick={() => setSelectedProvider(GIT_PROVIDERS[0].id)}
                    className="btn-ghost text-xs"
                  >
                    <Plus size={12} /> {t.gitBinding.addMore}
                  </button>
                )}
                <button
                  type="button"
                  data-testid={`git-binding-remove-${binding.id}`}
                  onClick={() => gitBindingApi.removeBinding(binding.id)}
                  aria-label={tx(t.gitBinding.removeAria, { name: binding.full_name })}
                  className="rounded border border-line p-1.5 text-ink-mute hover:text-danger hover:border-danger/50 transition-colors"
                >
                  <Trash2 size={13} />
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
