"use client";

/**
 * AIEditPanel — AI 修改 mock (per §12.4 ai-edit-mode=本地 mock 拍板)
 * 不开外部 API, 读脚本源码 + 产生模板建议
 */

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Loader2, Sparkles } from "lucide-react";
import { ScriptMeta, AIEditResult } from "../hooks/useDebugConsole";

interface Props {
  scripts: Record<string, ScriptMeta>;
  aiEditResult: AIEditResult | null;
  loading: boolean;
  onAIEdit: (scriptId: string, featuresContext: Record<string, string>) => void;
}

export function AIEditPanel({ scripts, aiEditResult, loading, onAIEdit }: Props) {
  const [selectedId, setSelectedId] = useState<string>("integration_e2e");
  const [featuresContextStr, setFeaturesContextStr] = useState("provider=hermes");

  const handleRun = () => {
    const ctx: Record<string, string> = {};
    for (const kv of featuresContextStr.split(",")) {
      if (kv.includes("=")) {
        const [k, v] = kv.split("=", 1);
        const value = kv.slice(kv.indexOf("=") + 1);
        ctx[k.trim()] = value.trim();
      }
    }
    onAIEdit(selectedId, ctx);
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            AI 修改 Mock
          </CardTitle>
          <CardDescription>
            <strong>本地 mock</strong>, 不开外部 API (per §12 ai-edit-mode=本地 mock 拍板)。
            读脚本源码 + 静态分析, 产生 3 条模板建议 (add_field / remove_method / rename_class)。
            用户需手动 apply 建议。
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div>
            <label className="text-sm font-semibold">脚本</label>
            <select
              className="w-full mt-1 p-2 border rounded-md bg-background"
              value={selectedId}
              onChange={(e) => setSelectedId(e.target.value)}
            >
              {Object.values(scripts).map((s) => (
                <option key={s.id} value={s.id}>
                  [{s.category}] {s.id} — {s.path}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-sm font-semibold">features_context (e.g. provider=hermes)</label>
            <input
              type="text"
              className="w-full mt-1 p-2 border rounded-md bg-background font-mono"
              value={featuresContextStr}
              onChange={(e) => setFeaturesContextStr(e.target.value)}
              placeholder="provider=hermes,dry_run=true"
            />
          </div>
          <Button onClick={handleRun} disabled={loading}>
            {loading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                生成建议中...
              </>
            ) : (
              <>
                <Sparkles className="mr-2 h-4 w-4" />
                跑 AI 修改 Mock
              </>
            )}
          </Button>
        </CardContent>
      </Card>

      {aiEditResult && (
        <Card>
          <CardHeader>
            <CardTitle>建议 — {aiEditResult.script_id}</CardTitle>
            <CardDescription>
              {aiEditResult.suggestions.length} 条建议 (duration {aiEditResult.duration_ms.toFixed(0)}ms) —
              confidence &lt; 0.5 表示 mock 永远是低置信度, 用户需手动 review
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {aiEditResult.suggestions.map((s, i) => (
              <div key={i} className="border rounded-md p-3 space-y-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <Badge variant="secondary">{s.type}</Badge>
                  <code className="text-xs font-mono">{s.target}</code>
                  <Badge variant="outline" className="text-xs">
                    confidence: {s.confidence}
                  </Badge>
                </div>
                <p className="text-sm">{s.rationale}</p>
                <pre className="bg-muted p-2 rounded text-xs overflow-x-auto whitespace-pre-wrap">
                  {s.diff_preview}
                </pre>
              </div>
            ))}
            {aiEditResult.suggestions.length === 0 && (
              <p className="text-sm text-muted-foreground">
                无建议 (脚本太短或已优化)
              </p>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
