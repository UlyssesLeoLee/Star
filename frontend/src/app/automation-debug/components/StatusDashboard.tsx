"use client";

/**
 * StatusDashboard — 14 份脚本 + 5 套 unittest 状态总览 (跑 / 关闭 / AI mock 等)
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { RefreshCw, CheckCircle2, XCircle, Activity } from "lucide-react";
import { StatusResult } from "../hooks/useDebugConsole";

interface Props {
  status: StatusResult | null;
  onRefresh: () => void;
}

export function StatusDashboard({ status, onRefresh }: Props) {
  if (!status) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>状态</CardTitle>
          <CardDescription>Loading...</CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>状态总览</CardTitle>
          <CardDescription>13 份脚本 + 5 套 unittest 状态 (per §12.2 清单表)</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4 mb-4">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-5 w-5 text-green-600" />
              <div>
                <div className="text-2xl font-bold">{status.enabled}</div>
                <div className="text-xs text-muted-foreground">已启用</div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <XCircle className="h-5 w-5 text-red-600" />
              <div>
                <div className="text-2xl font-bold">{status.disabled}</div>
                <div className="text-xs text-muted-foreground">已关闭</div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Activity className="h-5 w-5 text-blue-600" />
              <div>
                <div className="text-2xl font-bold">{status.total_runs}</div>
                <div className="text-xs text-muted-foreground">总运行次数</div>
              </div>
            </div>
          </div>
          <Button onClick={onRefresh} variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            刷新
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>脚本状态详情</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-1 max-h-96 overflow-y-auto">
            {Object.entries(status.scripts).map(([id, s]) => (
              <div
                key={id}
                className="flex items-center justify-between p-2 border rounded text-sm"
              >
                <code className="font-mono">{id}</code>
                <div className="flex items-center gap-2">
                  {s.status === "enabled" ? (
                    <Badge className="bg-green-100 text-green-800">enabled</Badge>
                  ) : (
                    <Badge variant="destructive">disabled</Badge>
                  )}
                  <Badge variant="outline" className="text-xs">
                    {s.run_count} runs
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
