"use client";

/**
 * useDebugConsole — 调 FastAPI 8080 的 React hook
 * (per docs/automation-design.md v0.2 §12.3 API 端点)
 */

import { useState, useCallback, useEffect } from "react";

export interface ScriptMeta {
  id: string;
  path: string;
  category: "base" | "p_card" | "unittest";
  features: string[];
  description: string;
  status: "enabled" | "disabled";
  last_run: string | null;
  last_run_output: string;
  run_count: number;
}

export interface RunResult {
  script_id: string;
  exit_code: number;
  duration_ms: number;
  output_preview: string;
  stderr_preview: string;
  ok: boolean;
}

export interface EditSuggestion {
  type: string;
  target: string;
  rationale: string;
  diff_preview: string;
  confidence: number;
}

export interface AIEditResult {
  script_id: string;
  script_path: string;
  suggestions: EditSuggestion[];
  duration_ms: number;
}

export interface StatusResult {
  total_scripts: number;
  enabled: number;
  disabled: number;
  total_runs: number;
  scripts: Record<string, { status: string; run_count: number; last_run: string | null }>;
}

const API_BASE = "http://localhost:8080";

export function useDebugConsole() {
  const [scripts, setScripts] = useState<Record<string, ScriptMeta>>({});
  const [status, setStatus] = useState<StatusResult | null>(null);
  const [runResult, setRunResult] = useState<RunResult | null>(null);
  const [aiEditResult, setAiEditResult] = useState<AIEditResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [running, setRunning] = useState(false);
  const [aiEditing, setAiEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchScripts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/api/scripts`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setScripts(data.scripts);
    } catch (e: any) {
      setError(`Failed to fetch scripts: ${e.message}. Is console_server.py running on :8080?`);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/status`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setStatus(data);
    } catch (e: any) {
      console.error("status fetch failed", e);
    }
  }, []);

  useEffect(() => {
    fetchScripts();
    fetchStatus();
  }, [fetchScripts, fetchStatus]);

  const refreshStatus = fetchStatus;

  const toggleScript = useCallback(async (scriptId: string, newStatus: "enabled" | "disabled") => {
    try {
      const res = await fetch(`${API_BASE}/api/scripts/${scriptId}/toggle?status=${newStatus}`, {
        method: "POST",
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      await fetchScripts();
      await fetchStatus();
    } catch (e: any) {
      setError(`Failed to toggle script: ${e.message}`);
    }
  }, [fetchScripts, fetchStatus]);

  const toggleFeature = useCallback(async (scriptId: string, featureId: string, enabled: boolean) => {
    try {
      const res = await fetch(
        `${API_BASE}/api/features/${scriptId}/${featureId}/toggle?enabled=${enabled}`,
        { method: "POST" }
      );
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
    } catch (e: any) {
      setError(`Failed to toggle feature: ${e.message}`);
    }
  }, []);

  const runScript = useCallback(async (scriptId: string) => {
    setRunning(true);
    setError(null);
    setRunResult(null);
    try {
      const res = await fetch(`${API_BASE}/api/scripts/${scriptId}/run`, { method: "POST" });
      if (!res.ok) {
        const errData = await res.json().catch(() => ({}));
        throw new Error(errData.detail || `HTTP ${res.status}`);
      }
      const data: RunResult = await res.json();
      setRunResult(data);
      await fetchStatus();
    } catch (e: any) {
      setError(`Failed to run script: ${e.message}`);
    } finally {
      setRunning(false);
    }
  }, [fetchStatus]);

  const aiEdit = useCallback(async (scriptId: string, featuresContext: Record<string, string> = {}) => {
    setAiEditing(true);
    setError(null);
    setAiEditResult(null);
    try {
      const res = await fetch(`${API_BASE}/api/ai_edit?script_id=${scriptId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ script_id: scriptId, features_context: featuresContext }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data: AIEditResult = await res.json();
      setAiEditResult(data);
    } catch (e: any) {
      setError(`Failed to run AI edit mock: ${e.message}`);
    } finally {
      setAiEditing(false);
    }
  }, []);

  return {
    scripts,
    status,
    runResult,
    aiEditResult,
    loading,
    running,
    aiEditing,
    error,
    toggleScript,
    toggleFeature,
    runScript,
    aiEdit,
    refreshStatus,
  };
}
