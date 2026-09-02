"use client";

/**
 * FeatureToggles — 脚本内功能点勾选 (per §12.2 功能点列)
 * 例如 integration_e2e.py → provider=openclaw / provider=hermes / dry_run
 */

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { ScriptMeta } from "../hooks/useDebugConsole";

interface Props {
  script: ScriptMeta;
  onToggleFeature: (scriptId: string, featureId: string, enabled: boolean) => void;
}

export function FeatureToggles({ script, onToggleFeature }: Props) {
  // 简化: 所有 feature 默认 enabled, 用户可关
  const [enabled, setEnabled] = useState<Record<string, boolean>>(
    Object.fromEntries(script.features.map((f) => [f, true]))
  );

  const handleToggle = (featureId: string, checked: boolean) => {
    setEnabled({ ...enabled, [featureId]: checked });
    onToggleFeature(script.id, featureId, checked);
  };

  // 按 = 分割, 跟脚本 CLI 拍板对照
  const grouped = script.features.reduce<Record<string, string[]>>((acc, f) => {
    const group = f.includes("=") ? f.split("=")[0] : "other";
    if (!acc[group]) acc[group] = [];
    acc[group].push(f);
    return acc;
  }, {});

  return (
    <Card>
      <CardHeader>
        <CardTitle>功能点 — {script.id}</CardTitle>
        <CardDescription>
          {script.features.length} 个功能点 (per §12.2 清单表) — 关闭=跳过 (per §12.6)
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {Object.entries(grouped).map(([group, features]) => (
          <div key={group}>
            <div className="flex items-center gap-2 mb-2">
              <Badge variant="outline">{group}</Badge>
              <span className="text-xs text-muted-foreground">
                ({features.length} 个)
              </span>
            </div>
            <div className="grid grid-cols-2 gap-2 ml-2">
              {features.map((f) => (
                <div key={f} className="flex items-center space-x-2">
                  <Checkbox
                    id={`${script.id}-${f}`}
                    checked={enabled[f] ?? true}
                    onCheckedChange={(checked) => handleToggle(f, !!checked)}
                  />
                  <Label
                    htmlFor={`${script.id}-${f}`}
                    className="text-sm font-mono cursor-pointer"
                  >
                    {f}
                  </Label>
                </div>
              ))}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
