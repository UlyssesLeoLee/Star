"use client";

/**
 * LogAITab — F-02 端到端实装
 * (per docs/briefs/ops-f02-log-ai-impl.md
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md §3.2 F-02)
 *
 * 范围:
 * - 上传区: textarea + 提交按钮, 调 /api/ops/log/upload
 * - AI 分析结果区: 调 /api/ops/log/analysis/{id}
 * - useQuery 实时轮询 (refetchInterval 5s, 守门 #6 v2 retriable 自动 retry)
 * - i18n 3 语言 (zh-CN / en / ja), 走 useTranslation
 * - 守门 #23: confidence < 0.5 必标 "needs review"
 */

import { useState, useCallback, useMemo } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Upload, Brain, AlertTriangle, CheckCircle2, RefreshCw } from "lucide-react";
import {
  uploadLog,
  getAnalysis,
  type LogUploadRequest,
  type LogAnalysis,
  type OpsResponse,
  OpsApiError,
  type LogUploadAck,
} from "@/lib/ops-api";
import { useTranslation } from "@/lib/i18n";

export function LogAITab() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [content, setContent] = useState("");
  const [source, setSource] = useState("k8s-pod/star-mcp");
  const [levelFilter, setLevelFilter] = useState<string[]>(["ERROR", "WARN"]);
  const [lastLogId, setLastLogId] = useState<string | null>(null);

  // 1. upload mutation
  const uploadMutation = useMutation({
    mutationFn: (req: LogUploadRequest) => uploadLog(req),
    onSuccess: (resp: OpsResponse<LogUploadAck>) => {
      setLastLogId(resp.data.log_id);
      // 立即 invalidate analysis query 触发刷新
      queryClient.invalidateQueries({ queryKey: ["ops-analysis", resp.data.log_id] });
    },
  });

  // 2. analysis query (per log_id 实时轮询)
  const analysisQuery = useQuery({
    queryKey: ["ops-analysis", lastLogId],
    queryFn: () => getAnalysis(lastLogId as string),
    enabled: Boolean(lastLogId),
    refetchInterval: 5_000, // 5s 轮询, 守门 #6 v2 retriable 由 react-query 处理
    retry: (failureCount, err) => {
      // 守门 #6 v2: 仅 retriable 错误 retry (max 3)
      if (err instanceof OpsApiError) {
        return err.retriable && failureCount < 3;
      }
      return failureCount < 3;
    },
  });

  const handleSubmit = useCallback(() => {
    if (!content.trim()) return;
    uploadMutation.mutate({
      source,
      level_filter: levelFilter as ("ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE")[],
      content,
    });
  }, [content, source, levelFilter, uploadMutation]);

  // 守门 #23: confidence < 0.5 必标 needs_review
  const needsReview = useMemo(() => {
    const a = analysisQuery.data?.data;
    return a ? a.confidence < 0.5 : false;
  }, [analysisQuery.data]);

  return (
    <div className="space-y-4">
      {/* === 上传区 === */}
      <div className="anime-panel anime-chamfer p-[21px]">
        <div className="flex items-center gap-2 mb-4">
          <Upload className="w-4 h-4 text-accent" />
          <h3 className="text-title font-bold">{t.opsConsole.logAIUploadTitle}</h3>
          <span className="anime-hud-tag ml-auto">F-02</span>
        </div>

        <div className="space-y-3">
          <div>
            <label className="text-xs text-ink-dim font-mono block mb-1">
              {t.opsConsole.logAISourceLabel}
            </label>
            <input
              type="text"
              value={source}
              onChange={(e) => setSource(e.target.value)}
              className="w-full px-3 py-2 rounded border border-line bg-bg-soft text-sm font-mono"
              placeholder="k8s-pod/star-mcp"
            />
          </div>

          <div>
            <label className="text-xs text-ink-dim font-mono block mb-1">
              {t.opsConsole.logAILevelFilterLabel}
            </label>
            <div className="flex gap-2 flex-wrap">
              {["TRACE", "DEBUG", "INFO", "WARN", "ERROR"].map((lv) => (
                <label key={lv} className="flex items-center gap-1 text-xs font-mono">
                  <input
                    type="checkbox"
                    checked={levelFilter.includes(lv)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setLevelFilter((prev) => [...prev, lv]);
                      } else {
                        setLevelFilter((prev) => prev.filter((x) => x !== lv));
                      }
                    }}
                    className="accent-accent"
                  />
                  <span>{lv}</span>
                </label>
              ))}
            </div>
          </div>

          <div>
            <label className="text-xs text-ink-dim font-mono block mb-1">
              {t.opsConsole.logAIContentLabel}
            </label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              rows={6}
              className="w-full px-3 py-2 rounded border border-line bg-bg-soft text-sm font-mono resize-y"
              placeholder="2026-09-08T07:30:00Z ERROR helm release 3 deploy failed: timeout"
            />
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleSubmit}
              disabled={uploadMutation.isPending || !content.trim()}
              className="px-4 py-2 rounded bg-accent text-bg-deepest font-mono text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {uploadMutation.isPending
                ? t.opsConsole.logAIUploading
                : t.opsConsole.logAIUploadBtn}
            </button>
            {uploadMutation.isError && (
              <span className="text-xs text-error font-mono">
                {t.opsConsole.logAIErrorPrefix}: {(uploadMutation.error as Error).message}
              </span>
            )}
            {uploadMutation.isSuccess && (
              <span className="text-xs text-success font-mono">
                {t.opsConsole.logAIUploadOk} (log_id: {uploadMutation.data?.data.log_id.slice(0, 8)}...)
              </span>
            )}
          </div>
        </div>
      </div>

      {/* === AI 分析结果区 === */}
      {lastLogId && (
        <div className="anime-panel anime-chamfer p-[21px]">
          <div className="flex items-center gap-2 mb-4">
            <Brain
              className="w-4 h-4"
              style={{ color: "var(--color-accent-violet)" }}
            />
            <h3 className="text-title font-bold">{t.opsConsole.logAIAnalysisTitle}</h3>
            <span
              className="anime-badge-neon ml-auto"
              style={{
                color: "var(--color-accent-violet)",
                borderColor: "color-mix(in srgb, var(--color-accent-violet) 40%, transparent)",
                background: "color-mix(in srgb, var(--color-accent-violet) 10%, transparent)",
              }}
            >
              {analysisQuery.data?.meta.ai_channel ?? "mock"}
            </span>
            {analysisQuery.isFetching && (
              <RefreshCw className="w-3 h-3 animate-spin text-ink-mute" />
            )}
          </div>

          {analysisQuery.isError && (
            <div className="flex items-center gap-2 text-error text-sm">
              <AlertTriangle className="w-4 h-4" />
              <span>{(analysisQuery.error as Error).message}</span>
            </div>
          )}

          {analysisQuery.data && (
            <LogAnalysisResultView
              analysis={analysisQuery.data.data}
              needsReview={needsReview}
              confidenceLabel={t.opsConsole.logAIConfidenceLabel}
              needsReviewLabel={t.opsConsole.logAINeedsReview}
              anomaliesLabel={t.opsConsole.logAIAnomaliesLabel}
              suggestionsLabel={t.opsConsole.logAISuggestionsLabel}
            />
          )}
        </div>
      )}
    </div>
  );
}

/** AI 分析结果展示 (守门 #23: needs_review 标徽) */
function LogAnalysisResultView({
  analysis,
  needsReview,
  confidenceLabel,
  needsReviewLabel,
  anomaliesLabel,
  suggestionsLabel,
}: {
  analysis: LogAnalysis;
  needsReview: boolean;
  confidenceLabel: string;
  needsReviewLabel: string;
  anomaliesLabel: string;
  suggestionsLabel: string;
}) {
  return (
    <div className="space-y-3 text-sm">
      <p className="text-ink-dim">{analysis.summary}</p>

      <div className="flex items-center gap-3 flex-wrap text-xs font-mono">
        <span className="text-ink-mute">
          {confidenceLabel}: <span className="text-accent">{analysis.confidence.toFixed(2)}</span>
        </span>
        <span className="text-ink-mute">channel: {analysis.generated_by}</span>
        {needsReview && (
          <span className="flex items-center gap-1 px-2 py-0.5 rounded border border-warning/40 bg-warning/10 text-warning">
            <AlertTriangle className="w-3 h-3" />
            {needsReviewLabel}
          </span>
        )}
        {!needsReview && (
          <span className="flex items-center gap-1 px-2 py-0.5 rounded border border-success/40 bg-success/10 text-success">
            <CheckCircle2 className="w-3 h-3" />
            confidence ≥ 0.5
          </span>
        )}
      </div>

      {analysis.anomalies.length > 0 && (
        <div>
          <h4 className="text-xs font-mono text-ink-mute mb-1">
            {anomaliesLabel} ({analysis.anomalies.length})
          </h4>
          <ul className="space-y-1">
            {analysis.anomalies.map((a, i) => (
              <li key={i} className="flex items-start gap-2 text-xs">
                <span className="px-1.5 py-0.5 rounded bg-error/15 text-error font-mono">
                  {a.level}
                </span>
                <span className="font-mono text-ink-mute">{a.timestamp}</span>
                <span className="text-ink-dim">{a.message_excerpt}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {analysis.suggestions.length > 0 && (
        <div>
          <h4 className="text-xs font-mono text-ink-mute mb-1">
            {suggestionsLabel} ({analysis.suggestions.length})
          </h4>
          <ul className="space-y-1">
            {analysis.suggestions.map((s, i) => (
              <li key={i} className="flex items-start gap-2 text-xs">
                <span className="font-mono text-ink-mute">[{s.confidence.toFixed(2)}]</span>
                <span className="text-ink-dim">{s.text}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
