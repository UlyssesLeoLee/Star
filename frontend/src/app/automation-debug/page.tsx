"use client";

/**
 * /automation-debug — Automation Debug Console
 * (per docs/automation-design.md v0.2 §12.4 调试控制台)
 *
 * 9/5 14:41 JST 用户拍板升档 (守门 #19 文档同步 §3):
 *   - Hero 头部: 巨型渐变标题 + 3 KPI 胶囊 + 3D 漂浮核心
 *   - 4 Tab: lucide 3D-style 图标 + 角标计数 + 选中辉光
 *   - 卡片: anime-chamfer 切角 + anime-panel 玻璃 + lift-on-hover
 *   - 字体: Zen Maru Gothic 圆体 + JetBrains Mono + Noto Serif JP
 *
 * 用户在浏览器勾选 14 份 Python 脚本 + 5 套 unittest (per §12.2 清单表) + 关闭 + 跑 + AI 修改 mock
 * 后端走 FastAPI 8080 (per §12.3 7 个 API 端点)
 */

import { useState } from "react";
import { useDebugConsole } from "./hooks/useDebugConsole";
import { useCountUp } from "./hooks/useCountUp";
import { ScriptSelector } from "./components/ScriptSelector";
import { FeatureToggles } from "./components/FeatureToggles";
import { RunPanel } from "./components/RunPanel";
import { AIEditPanel } from "./components/AIEditPanel";
import { StatusDashboard } from "./components/StatusDashboard";
import { HeroHeader } from "./components/HeroHeader";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollText, PlayCircle, Bot, GaugeCircle, Inbox } from "lucide-react";

export default function AutomationDebugPage() {
  const console = useDebugConsole();
  const [selectedScriptId, setSelectedScriptId] = useState<string | null>(null);

  // 数字滚动 — KPI 胶囊进场仪式感 (per 9/5 §4.4)
  const scriptsAnimated = useCountUp(Object.keys(console.scripts).length, 700, !console.loading);
  const runningAnimated = useCountUp(0, 600, !console.loading);

  if (console.loading && Object.keys(console.scripts).length === 0) {
    return (
      <div className="container mx-auto p-[21px]">
        <div className="anime-panel anime-chamfer p-13 text-center">
          <div className="flex items-center justify-center gap-3 mb-3">
            <div className="w-2 h-2 rounded-full pulse-ok" style={{ background: "var(--color-primary)" }} />
            <span className="anime-hud-tag">LOADING</span>
          </div>
          <h2 className="font-anime text-title-lg font-bold">讀 取 中…</h2>
          <p className="text-sm text-ink-dim mt-2 font-mono">scripts from FastAPI 8080</p>
        </div>
      </div>
    );
  }

  if (console.error) {
    return (
      <div className="container mx-auto p-[21px]">
        <div
          className="anime-panel anime-chamfer p-13 border-2"
          style={{ borderColor: "var(--err-DEFAULT)" }}
        >
          <div className="flex items-center gap-2 mb-3">
            <span className="anime-hud-tag" style={{
              color: "var(--err-DEFAULT)",
              borderColor: "var(--err-DEFAULT)",
              background: "color-mix(in srgb, var(--err-DEFAULT) 12%, transparent)",
            }}>ERROR</span>
            <span className="text-[11px] text-ink-mute font-mono uppercase tracking-wider">FastAPI 8080 連線失敗</span>
          </div>
          <h2 className="font-anime text-title-lg font-bold text-ink-DEFAULT mb-2">連 線 中 斷</h2>
          <p className="text-sm text-ink-dim mb-4">請確認 console_server.py 已啟動:</p>
          <code className="block bg-ink-DEFAULT/5 px-4 py-3 rounded-md font-mono text-[13px] text-ink-DEFAULT">
            python scripts/automation/console_server.py
          </code>
          <p className="text-xs text-err mt-3 font-mono">[{console.error}]</p>
        </div>
      </div>
    );
  }

  const selectedScript = selectedScriptId ? console.scripts[selectedScriptId] : null;
  const scriptCount = Object.keys(console.scripts).length;

  return (
    <div className="container mx-auto p-[21px] space-y-[21px]">
      {/* === HERO 头部 — 杂志封面级 === */}
      <HeroHeader scriptCount={scriptsAnimated} testCount={5} runningCount={runningAnimated} />

      {/* === 4 个 Tab — 3D-style lucide 图标 + 角标计数 + 选中辉光 === */}
      <Tabs defaultValue="scripts" className="w-full">
        <TabsList className="grid w-full grid-cols-4 gap-2 p-1.5 anime-panel">
          <TabsTrigger
            value="scripts"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <ScrollText className="w-4 h-4" />
            <span>腳本</span>
            <span className="anime-hud-tag ml-1">{scriptCount}</span>
          </TabsTrigger>
          <TabsTrigger
            value="run"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <PlayCircle className="w-4 h-4" />
            <span>運行</span>
            {runningAnimated > 0 && <span className="anime-badge-neon ml-1">LIVE</span>}
          </TabsTrigger>
          <TabsTrigger
            value="ai"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <Bot className="w-4 h-4" />
            <span style={{ fontFamily: "var(--font-serif-jp)" }}>AI 修 改</span>
            <span className="anime-badge-neon ml-1" style={{ color: "var(--color-accent-violet)", borderColor: "color-mix(in srgb, var(--color-accent-violet) 40%, transparent)", background: "color-mix(in srgb, var(--color-accent-violet) 10%, transparent)" }}>NEW</span>
          </TabsTrigger>
          <TabsTrigger
            value="status"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <GaugeCircle className="w-4 h-4" />
            <span>狀態</span>
          </TabsTrigger>
        </TabsList>

        {/* 脚本网格 — 切角 + 玻璃面板 + lift-on-hover */}
        <TabsContent value="scripts" className="space-y-[21px] mt-[21px]">
          <div className="grid grid-cols-12 gap-[21px]">
            <div className="col-span-12 lg:col-span-4">
              <div className="anime-panel anime-chamfer lift-on-hover p-[21px]">
                <div className="flex items-center gap-2 mb-3">
                  <Inbox className="w-4 h-4 text-ink-dim" />
                  <h3 className="text-title font-bold">腳本清單</h3>
                  <span className="anime-hud-tag ml-auto">{scriptCount}</span>
                </div>
                <p className="text-xs text-ink-mute mb-4 font-mono">按類別分組: 基類 / [P] / unittest</p>
                <ScriptSelector
                  scripts={console.scripts}
                  selectedScriptId={selectedScriptId}
                  onSelect={setSelectedScriptId}
                  onToggle={console.toggleScript}
                />
              </div>
            </div>
            <div className="col-span-12 lg:col-span-8">
              {selectedScript ? (
                <FeatureToggles
                  script={selectedScript}
                  onToggleFeature={console.toggleFeature}
                />
              ) : (
                <div className="anime-panel anime-chamfer p-[34px] text-center h-full flex flex-col items-center justify-center min-h-[300px]">
                  <div className="w-12 h-12 rounded-full mx-auto mb-4 flex items-center justify-center pulse-ok"
                    style={{ background: "color-mix(in srgb, var(--color-primary) 10%, transparent)", border: "1px solid color-mix(in srgb, var(--color-primary) 30%, transparent)" }}>
                    <ScrollText className="w-5 h-5" style={{ color: "var(--color-primary)" }} />
                  </div>
                  <p className="text-[15px] text-ink-dim">
                    ← 選擇一份腳本查看功能點 <span className="font-mono text-ink-mute">(per §12.2 清單表)</span>
                  </p>
                </div>
              )}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="run" className="mt-[21px]">
          <RunPanel
            scripts={console.scripts}
            runResult={console.runResult}
            loading={console.running}
            onRun={console.runScript}
          />
        </TabsContent>

        <TabsContent value="ai" className="mt-[21px]">
          <AIEditPanel
            scripts={console.scripts}
            aiEditResult={console.aiEditResult}
            loading={console.aiEditing}
            onAIEdit={console.aiEdit}
          />
        </TabsContent>

        <TabsContent value="status" className="mt-[21px]">
          <StatusDashboard
            status={console.status}
            onRefresh={console.refreshStatus}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
