"use client";

/**
 * /automation-debug — Automation Debug Console
 * (per docs/automation-design.md v0.2 §12.4 调试控制台)
 *
 * 用户在浏览器勾选 14 份 Python 脚本 + 5 套 unittest (per §12.2 清单表) + 关闭 + 跑 + AI 修改 mock
 * 后端走 FastAPI 8080 (per §12.3 7 个 API 端点)
 */

import { useState } from "react";
import { useDebugConsole } from "./hooks/useDebugConsole";
import { ScriptSelector } from "./components/ScriptSelector";
import { FeatureToggles } from "./components/FeatureToggles";
import { RunPanel } from "./components/RunPanel";
import { AIEditPanel } from "./components/AIEditPanel";
import { StatusDashboard } from "./components/StatusDashboard";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function AutomationDebugPage() {
  const console = useDebugConsole();
  const [selectedScriptId, setSelectedScriptId] = useState<string | null>(null);

  if (console.loading && Object.keys(console.scripts).length === 0) {
    return (
      <div className="container mx-auto p-6">
        <Card>
          <CardHeader>
            <CardTitle>Automation Debug Console</CardTitle>
            <CardDescription>Loading scripts from FastAPI 8080...</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  if (console.error) {
    return (
      <div className="container mx-auto p-6">
        <Card>
          <CardHeader>
            <CardTitle>Automation Debug Console</CardTitle>
            <CardDescription className="text-red-500">Error: {console.error}</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              请确保 console_server.py 正在运行:{" "}
              <code className="bg-muted px-2 py-1 rounded">
                python scripts/automation/console_server.py
              </code>
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const selectedScript = selectedScriptId ? console.scripts[selectedScriptId] : null;

  return (
    <div className="container mx-auto p-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Automation Debug Console</CardTitle>
          <CardDescription>
            {Object.keys(console.scripts).length} 份 Python 脚本 + 5 套 unittest 调试控制台
            (per docs/automation-design.md v0.2 §12)
          </CardDescription>
        </CardHeader>
      </Card>

      <Tabs defaultValue="scripts" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="scripts">📜 脚本清单</TabsTrigger>
          <TabsTrigger value="run">▶️ 运行</TabsTrigger>
          <TabsTrigger value="ai">🤖 AI 修改</TabsTrigger>
          <TabsTrigger value="status">📊 状态</TabsTrigger>
        </TabsList>

        <TabsContent value="scripts" className="space-y-4">
          <div className="grid grid-cols-12 gap-4">
            <div className="col-span-4">
              <ScriptSelector
                scripts={console.scripts}
                selectedScriptId={selectedScriptId}
                onSelect={setSelectedScriptId}
                onToggle={console.toggleScript}
              />
            </div>
            <div className="col-span-8">
              {selectedScript ? (
                <FeatureToggles
                  script={selectedScript}
                  onToggleFeature={console.toggleFeature}
                />
              ) : (
                <Card>
                  <CardContent className="pt-6">
                    <p className="text-sm text-muted-foreground">
                      ← 选择一份脚本查看功能点 (per §12.2 清单表)
                    </p>
                  </CardContent>
                </Card>
              )}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="run" className="space-y-4">
          <RunPanel
            scripts={console.scripts}
            runResult={console.runResult}
            loading={console.running}
            onRun={console.runScript}
          />
        </TabsContent>

        <TabsContent value="ai" className="space-y-4">
          <AIEditPanel
            scripts={console.scripts}
            aiEditResult={console.aiEditResult}
            loading={console.aiEditing}
            onAIEdit={console.aiEdit}
          />
        </TabsContent>

        <TabsContent value="status" className="space-y-4">
          <StatusDashboard
            status={console.status}
            onRefresh={console.refreshStatus}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
