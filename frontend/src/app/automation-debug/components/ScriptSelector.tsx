"use client";

/**
 * ScriptSelector — 14 份脚本 + 5 套 unittest 列表 (per §12.2 清单表)
 * 用户勾选/关闭脚本 (per close-behavior=1 跳过关闭的)
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScriptMeta } from "../hooks/useDebugConsole";

interface Props {
  scripts: Record<string, ScriptMeta>;
  selectedScriptId: string | null;
  onSelect: (id: string) => void;
  onToggle: (id: string, status: "enabled" | "disabled") => void;
}

const CATEGORY_LABEL = {
  base: "基类",
  p_card: "[P] 任务卡",
  unittest: "unittest",
};

const CATEGORY_COLOR = {
  base: "bg-blue-100 text-blue-800",
  p_card: "bg-green-100 text-green-800",
  unittest: "bg-purple-100 text-purple-800",
};

export function ScriptSelector({ scripts, selectedScriptId, onSelect, onToggle }: Props) {
  const sorted = Object.values(scripts).sort((a, b) => {
    if (a.category !== b.category) {
      const order = { base: 0, p_card: 1, unittest: 2 };
      return order[a.category] - order[b.category];
    }
    return a.id.localeCompare(b.id);
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>脚本清单 ({sorted.length})</CardTitle>
        <CardDescription>
          按类别分组: 基类 / [P] 任务卡 / unittest
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2 max-h-[600px] overflow-y-auto">
        {sorted.map((s) => (
          <div
            key={s.id}
            className={`p-3 border rounded-md cursor-pointer transition-colors ${
              selectedScriptId === s.id ? "border-primary bg-primary/5" : "hover:bg-muted/50"
            } ${s.status === "disabled" ? "opacity-50" : ""}`}
            onClick={() => onSelect(s.id)}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <code className="text-sm font-mono font-semibold">{s.id}</code>
                  <Badge className={CATEGORY_COLOR[s.category]} variant="secondary">
                    {CATEGORY_LABEL[s.category]}
                  </Badge>
                  {s.status === "disabled" && (
                    <Badge variant="destructive" className="text-xs">
                      已关闭
                    </Badge>
                  )}
                  {s.run_count > 0 && (
                    <Badge variant="outline" className="text-xs">
                      跑 {s.run_count} 次
                    </Badge>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                  {s.description}
                </p>
                <p className="text-xs text-muted-foreground/70 mt-1 font-mono">
                  {s.path}
                </p>
              </div>
              <Button
                size="sm"
                variant={s.status === "enabled" ? "outline" : "default"}
                onClick={(e) => {
                  e.stopPropagation();
                  onToggle(s.id, s.status === "enabled" ? "disabled" : "enabled");
                }}
              >
                {s.status === "enabled" ? "关闭" : "启用"}
              </Button>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
