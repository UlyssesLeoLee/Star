"use client";

/**
 * /ops — Ops Console (F-02 端到端实装)
 * (per docs/requirements/SRS-STAR-OPS-001.md v0.1
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md v0.1
 *  + docs/briefs/ops-f02-log-ai-impl.md)
 *
 * 4 tab: 集群更新 (F-01) / Log AI (F-02, 端到端) / 运维数据 (F-03) / 文档 (F-04)
 * 后端走 star-ops binary (port 8090), F-02 已端到端实装 (mock subprocess)
 *   - /api/ops/log/upload (trace_id 传递 + level_filter + 1MB 限制)
 *   - /api/ops/log/analysis/{id} (Ladder 真实调 ai_log_mock.py)
 *
 * Phase OPS-INTRY → OPS-F02-E2E (2026-09-08):
 *   - Hero 头部 + 4 KPI 胶囊 (per automation-debug 视觉一致)
 *   - 4 Tab 切换
 *   - F-02 LogAITab 端到端实装 (useMutation + useQuery 5s 轮询 + i18n 3 语言)
 *   - 守门 #23: confidence < 0.5 必标 needs_review
 */

import { useState } from "react";
import { Server, Brain, BarChart3, FileText, Wrench, Sparkles, Activity } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTranslation } from "@/lib/i18n";
import { LogAITab } from "./components/LogAITab";
import { ClusterTab } from "./components/ClusterTab";
import { MetricsTab } from "./components/MetricsTab";
import { DocsTab } from "./components/DocsTab";

export default function OpsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("cluster");

  return (
    <div className="container mx-auto p-[21px] space-y-[21px]">
      {/* === HERO 头部 === */}
      <div className="anime-panel anime-chamfer p-[21px]">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center gap-3">
            <div className="size-12 rounded-lg bg-accent/15 border border-accent/40 grid place-items-center text-accent shadow-[0_0_12px_rgba(0,240,255,0.3)]">
              <Wrench size={22} />
            </div>
            <div>
              <h1 className="font-anime text-title-lg font-bold text-ink">
                {t.opsConsole.pageTitle}
              </h1>
              <p className="text-xs text-ink-dim font-mono mt-1">
                {t.opsConsole.pageSubtitle}
              </p>
            </div>
          </div>

          {/* 4 KPI 胶囊 */}
          <div className="flex items-center gap-2 flex-wrap">
            <KPIChip icon={<Server size={11} />} label="F-01" value="Cluster" />
            <KPIChip icon={<Brain size={11} />} label="F-02" value="Log AI" />
            <KPIChip icon={<BarChart3 size={11} />} label="F-03" value="Metrics" />
            <KPIChip icon={<FileText size={11} />} label="F-04" value="Docs" />
          </div>
        </div>
      </div>

      {/* === 4 Tab === */}
      <Tabs value={activeTab} defaultValue="cluster" onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-4 gap-2 p-1.5 anime-panel">
          <TabsTrigger
            value="cluster"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <Server className="w-4 h-4" />
            <span>{t.opsConsole.tabCluster}</span>
            <span className="anime-hud-tag ml-1">F-01</span>
          </TabsTrigger>
          <TabsTrigger
            value="logai"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <Brain className="w-4 h-4" />
            <span>{t.opsConsole.tabLogAI}</span>
            <span
              className="anime-badge-neon ml-1"
              style={{
                color: "var(--color-accent-violet)",
                borderColor: "color-mix(in srgb, var(--color-accent-violet) 40%, transparent)",
                background: "color-mix(in srgb, var(--color-accent-violet) 10%, transparent)",
              }}
            >
              AI
            </span>
          </TabsTrigger>
          <TabsTrigger
            value="metrics"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <BarChart3 className="w-4 h-4" />
            <span>{t.opsConsole.tabMetrics}</span>
            <span className="anime-hud-tag ml-1">F-03</span>
          </TabsTrigger>
          <TabsTrigger
            value="docs"
            className="gap-2 data-[state=active]:tab-glow data-[state=active]:bg-bg-soft data-[state=active]:border data-[state=active]:border-border-line"
          >
            <FileText className="w-4 h-4" />
            <span>{t.opsConsole.tabDocs}</span>
            <span className="anime-hud-tag ml-1">F-04</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="cluster" className="mt-[21px]">
          <ClusterTab />
        </TabsContent>

        <TabsContent value="logai" className="mt-[21px]">
          <LogAITab />
        </TabsContent>

        <TabsContent value="metrics" className="mt-[21px]">
          <MetricsTab />
        </TabsContent>

        <TabsContent value="docs" className="mt-[21px]">
          <DocsTab />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function KPIChip({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md border border-line bg-bg-soft/50">
      <span className="text-accent">{icon}</span>
      <span className="text-[10px] font-mono text-ink-mute">{label}</span>
      <span className="text-[10px] font-mono text-accent font-semibold">{value}</span>
    </div>
  );
}

function PlaceholderCard({
  title,
  tag,
  items,
  accent = "accent",
}: {
  title: string;
  tag: string;
  items: string[];
  accent?: "accent" | "violet";
}) {
  const { t } = useTranslation();
  const accentVar = accent === "violet" ? "var(--color-accent-violet)" : "var(--color-primary)";

  return (
    <div className="anime-panel anime-chamfer lift-on-hover p-[21px]">
      <div className="flex items-center gap-2 mb-4">
        <Activity className="w-4 h-4" style={{ color: accentVar }} />
        <h3 className="text-title font-bold">{title}</h3>
        <span className="anime-hud-tag ml-auto">{tag}</span>
        <span
          className="anime-badge-neon"
          style={{
            color: accentVar,
            borderColor: `color-mix(in srgb, ${accentVar} 40%, transparent)`,
            background: `color-mix(in srgb, ${accentVar} 10%, transparent)`,
          }}
        >
          {t.opsConsole.comingSoon}
        </span>
      </div>
      <ul className="space-y-2 text-sm text-ink-dim">
        {items.map((it, i) => (
          <li key={i} className="flex items-center gap-2">
            <Sparkles className="w-3 h-3 text-ink-mute" />
            <span>{it}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
