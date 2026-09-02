"use client";

/**
 * RunPanel — 跑脚本 + 显示 output (头 500 字符, 避免长 output 占 token)
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Loader2, Play } from "lucide-react";
import { ScriptMeta, RunResult } from "../hooks/useDebugConsole";

interface Props {
  scripts: Record<string, ScriptMeta>;
  runResult: RunResult | null;
  loading: boolean;
  onRun: (scriptId: string) => void;
}

export function RunPanel({ scripts, runResult, loading, onRun }: Props) {
  const enabled = Object.values(scripts).filter((s) => s.status === "enabled");

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>运行脚本</CardTitle>
          <CardDescription>
            仅显示启用的脚本 ({enabled.length} / {Object.keys(scripts).length}) —
            关闭的脚本 dispatcher 跳过 (per §12.6 close-behavior=1)
          </CardDescription>
        </CardHeader>
        <CardContent className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
          {enabled.map((s) => (
            <Button
              key={s.id}
              variant="outline"
              disabled={loading}
              onClick={() => onRun(s.id)}
              className="justify-start"
            >
              {loading ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Play className="mr-2 h-4 w-4" />
              )}
              <span className="truncate font-mono text-xs">{s.id}</span>
            </Button>
          ))}
        </CardContent>
      </Card>

      {runResult && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <span>运行结果 — {runResult.script_id}</span>
              {runResult.ok ? (
                <Badge className="bg-green-100 text-green-800">成功</Badge>
              ) : (
                <Badge variant="destructive">失败</Badge>
              )}
            </CardTitle>
            <CardDescription>
              exit_code={runResult.exit_code}, duration={runResult.duration_ms.toFixed(0)}ms
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div>
                <div className="text-sm font-semibold mb-1">stdout (头 500 字符):</div>
                <pre className="bg-muted p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap max-h-96 overflow-y-auto">
                  {runResult.output_preview || "(empty)"}
                </pre>
              </div>
              {runResult.stderr_preview && (
                <div>
                  <div className="text-sm font-semibold mb-1 text-red-600">stderr:</div>
                  <pre className="bg-red-50 p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap max-h-32 overflow-y-auto">
                    {runResult.stderr_preview}
                  </pre>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
