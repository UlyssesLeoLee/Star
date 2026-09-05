"use client";

// =====================================================================
// AgentSettingsTab — 给每个 agent 配 API / 模型 / 提示 (per 9/5 23:00 JST 拍板)
// =====================================================================
// 3 个 tab 之一 (Canvas v1 / Roguelike v2 / Agent 设置):
//   - 左侧 12 agents 列表 (per store.agentSessions)
//   - 右侧 form (API key / model select / maxTokens / temperature /
//     systemPrompt / baseUrl / enabled toggle)
//   - 实时编辑 (updateAgentSetting) + 保存按钮 (validateSettings 校验)
//   - 默认值 per agent_kind (claude-sonnet / gpt-4o / codex / internal)
// =====================================================================

import { useState, useMemo, useEffect, useCallback } from "react";
import { useStore } from "@/lib/store";
import {
  validateSettings, modelOptionsFor,
  type AgentSettings,
} from "@/lib/agent-game/settings";
import { StatusPill } from "@/components/StatusPill";
import { Check, Key, Cpu, Zap, FileText, Globe, Power, Save, AlertCircle, Bot } from "lucide-react";

interface AgentSettingsTabProps {
  /** 初始选中 agent id (默认第一个) */
  initialAgentId?: string | null;
}

export function AgentSettingsTab({ initialAgentId = null }: AgentSettingsTabProps) {
  const agents = useStore((s) => s.agentSessions);
  const allSettings = useStore((s) => s.agentSettings);
  const initAgentSettings = useStore((s) => s.initAgentSettings);
  const updateAgentSetting = useStore((s) => s.updateAgentSetting);
  const replaceAgentSettings = useStore((s) => s.replaceAgentSettings);
  const toggleAgentEnabled = useStore((s) => s.toggleAgentEnabled);

  // 选中 agent (local state, 不进 store)
  const [selectedId, setSelectedId] = useState<string | null>(initialAgentId ?? agents[0]?.id ?? null);

  // 切换 agent 时, 确保 settings 已 init
  useEffect(() => {
    if (selectedId && !allSettings[selectedId]) {
      initAgentSettings(selectedId);
    }
  }, [selectedId, allSettings, initAgentSettings]);

  // 首次 mount: 给所有未 init 的 agent 批量 init
  useEffect(() => {
    for (const a of agents) {
      if (!allSettings[a.id]) {
        initAgentSettings(a.id);
      }
    }
  }, [agents, allSettings, initAgentSettings]);

  const selected = agents.find((a) => a.id === selectedId) ?? null;
  const settings = selectedId ? allSettings[selectedId] : null;

  // 校验结果 (实时)
  const validation = useMemo(() => {
    if (!settings) return { ok: true, errors: [] as string[] };
    return validateSettings(settings);
  }, [settings]);

  // 保存 (per 拍板, 必检)
  const [saved, setSaved] = useState(false);
  const handleSave = useCallback(() => {
    if (!settings || !validation.ok) return;
    replaceAgentSettings(settings.agentId, settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 1500);
  }, [settings, validation, replaceAgentSettings]);

  return (
    <div data-testid="agent-settings-tab" className="flex h-full min-h-0">
      {/* 左侧 agent 列表 */}
      <div className="w-64 border-r border-line bg-bg-soft/40 overflow-y-auto">
        <div className="px-3 py-2 text-[10px] uppercase tracking-wider text-ink-mute font-mono border-b border-line">
          Agents ({agents.length})
        </div>
        {agents.map((a) => {
          const aSettings = allSettings[a.id];
          const isSelected = a.id === selectedId;
          return (
            <button
              key={a.id}
              data-testid={`agent-settings-option-${a.id}`}
              data-selected={isSelected}
              onClick={() => setSelectedId(a.id)}
              className={`w-full text-left px-3 py-2 border-b border-line/40 hover:bg-bg-soft transition-colors ${isSelected ? "bg-info/10 border-l-2 border-l-info" : ""}`}
            >
              <div className="flex items-center gap-1.5">
                <Bot size={11} className={aSettings?.enabled ? "text-info" : "text-ink-mute"} />
                <span className="font-mono text-xs text-info truncate flex-1">{a.id}</span>
                {aSettings && !aSettings.enabled && (
                  <span className="text-[9px] text-ink-mute">off</span>
                )}
              </div>
              <div className="text-[10px] text-ink-mute truncate">{a.agent_kind} · {a.current_step}</div>
              <div className="mt-1">
                <StatusPill value={a.status} size="xs" />
              </div>
            </button>
          );
        })}
      </div>

      {/* 右侧 form */}
      <div className="flex-1 overflow-y-auto p-4">
        {!selected || !settings ? (
          <div className="text-center text-ink-mute text-xs py-12">选择左侧 agent 配置 API</div>
        ) : (
          <div className="max-w-2xl mx-auto" data-testid="agent-settings-form">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-base font-semibold flex items-center gap-2">
                  <Bot size={16} className="text-info" />
                  {selected.id}
                </h2>
                <div className="text-[10px] text-ink-mute font-mono mt-0.5">
                  {selected.agent_kind} · worktree {selected.worktree_id} · started {new Date(selected.started_at).toLocaleString()}
                </div>
              </div>
              <button
                data-testid="agent-settings-toggle-enabled"
                onClick={() => toggleAgentEnabled(selected.id)}
                className={`btn text-xs py-1 px-2 ${settings.enabled ? "border-ok/50 text-ok" : "border-ink-mute/40 text-ink-mute"}`}
              >
                <Power size={11} /> {settings.enabled ? "Enabled" : "Disabled"}
              </button>
            </div>

            <div className="space-y-3">
              {/* API Key */}
              <FieldRow
                icon={<Key size={12} />}
                label="API Key"
                hint="Mock 模式, 实际不调用"
                testId="agent-settings-field-apikey"
              >
                <input
                  data-testid="agent-settings-input-apikey"
                  type="password"
                  value={settings.apiKey}
                  onChange={(e) => updateAgentSetting(selected.id, "apiKey", e.target.value)}
                  className="input text-xs font-mono w-full"
                />
              </FieldRow>

              {/* Model + Base URL (2 列) */}
              <div className="grid grid-cols-2 gap-3">
                <FieldRow
                  icon={<Cpu size={12} />}
                  label="Model"
                  hint={`per ${selected.agent_kind}`}
                  testId="agent-settings-field-model"
                >
                  <select
                    data-testid="agent-settings-input-model"
                    value={settings.model}
                    onChange={(e) => updateAgentSetting(selected.id, "model", e.target.value)}
                    className="input text-xs font-mono w-full"
                  >
                    {modelOptionsFor(selected.agent_kind).map((m) => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                </FieldRow>
                <FieldRow
                  icon={<Globe size={12} />}
                  label="Base URL"
                  hint="https://..."
                  testId="agent-settings-field-baseurl"
                >
                  <input
                    data-testid="agent-settings-input-baseurl"
                    type="url"
                    value={settings.baseUrl}
                    onChange={(e) => updateAgentSetting(selected.id, "baseUrl", e.target.value)}
                    className="input text-xs font-mono w-full"
                  />
                </FieldRow>
              </div>

              {/* Max Tokens + Temperature (2 列) */}
              <div className="grid grid-cols-2 gap-3">
                <FieldRow
                  icon={<Zap size={12} />}
                  label="Max Tokens"
                  hint="1..200000"
                  testId="agent-settings-field-maxtokens"
                >
                  <input
                    data-testid="agent-settings-input-maxtokens"
                    type="number"
                    min={1}
                    max={200000}
                    value={settings.maxTokens}
                    onChange={(e) => updateAgentSetting(selected.id, "maxTokens", Number(e.target.value))}
                    className="input text-xs font-mono w-full"
                  />
                </FieldRow>
                <FieldRow
                  icon={<Zap size={12} />}
                  label="Temperature"
                  hint="0..2 (推荐 0.7)"
                  testId="agent-settings-field-temperature"
                >
                  <input
                    data-testid="agent-settings-input-temperature"
                    type="number"
                    min={0}
                    max={2}
                    step={0.1}
                    value={settings.temperature}
                    onChange={(e) => updateAgentSetting(selected.id, "temperature", Number(e.target.value))}
                    className="input text-xs font-mono w-full"
                  />
                </FieldRow>
              </div>

              {/* System Prompt */}
              <FieldRow
                icon={<FileText size={12} />}
                label="System Prompt"
                hint="per agent 角色定义"
                testId="agent-settings-field-systemprompt"
              >
                <textarea
                  data-testid="agent-settings-input-systemprompt"
                  value={settings.systemPrompt}
                  onChange={(e) => updateAgentSetting(selected.id, "systemPrompt", e.target.value)}
                  rows={4}
                  className="input text-xs w-full resize-none"
                />
              </FieldRow>

              {/* 校验提示 */}
              {!validation.ok && (
                <div data-testid="agent-settings-validation-errors" className="border border-err/40 bg-err/10 rounded p-2">
                  <div className="text-[10px] text-err font-semibold flex items-center gap-1 mb-1">
                    <AlertCircle size={11} /> Validation Errors
                  </div>
                  <ul className="text-[10px] text-err/80 list-disc list-inside space-y-0.5">
                    {validation.errors.map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                </div>
              )}

              {/* Save Button */}
              <div className="flex items-center justify-between pt-2 border-t border-line">
                <div className="text-[10px] text-ink-mute font-mono">
                  last update: {new Date(settings.updatedAt).toLocaleTimeString()}
                </div>
                <button
                  data-testid="agent-settings-save-btn"
                  onClick={handleSave}
                  disabled={!validation.ok}
                  className={`btn text-xs py-1 px-3 ${validation.ok ? "border-accent text-accent hover:bg-accent/10" : "border-ink-mute/30 text-ink-mute opacity-50 cursor-not-allowed"}`}
                >
                  {saved ? (
                    <><Check size={11} /> 已保存</>
                  ) : (
                    <><Save size={11} /> 保存</>
                  )}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function FieldRow({
  icon, label, hint, testId, children,
}: {
  icon: React.ReactNode;
  label: string;
  hint?: string;
  testId: string;
  children: React.ReactNode;
}) {
  return (
    <div data-testid={testId} className="space-y-1">
      <div className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider text-ink-mute font-mono">
        {icon}
        <span>{label}</span>
        {hint && <span className="text-ink-mute/60 normal-case tracking-normal">· {hint}</span>}
      </div>
      {children}
    </div>
  );
}
