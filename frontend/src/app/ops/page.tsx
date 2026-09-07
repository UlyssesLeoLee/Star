"use client";

/**
 * /ops — Ops Console (MVP-骨架)
 * (per docs/requirements/SRS-STAR-OPS-001.md v0.1
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md v0.1)
 *
 * 4 tab 占位: 集群更新 (F-01) / Log AI (F-02) / 运维数据 (F-03) / 文档 (F-04)
 * 后端走 star-ops binary (port 8090), MVP 返 200 + stub meta
 *
 * Phase OPS-INTRY (2026-09-08):
 *   - Hero 头部 + 4 KPI 胶囊 (per automation-debug 视觉一致)
 *   - 4 Tab 切换 + 占位卡片 (per 拍板 Q1 范围: 仅入口 + 4 空壳页面)
 *   - 真实 API 契约由 curl 验证 (8 REST stub)
 */

import { useState } from "react";
import { Server, Brain, BarChart3, FileText, Wrench, Sparkles, Activity } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTranslation } from "@/lib/i18n";

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
          <PlaceholderCard
            title={t.opsConsole.clusterTitle}
            tag="F-01"
            items={[
              t.opsConsole.clusterReleases,
              t.opsConsole.clusterCanary,
              t.opsConsole.clusterRollback,
              t.opsConsole.clusterStatus,
            ]}
          />
        </TabsContent>

        <TabsContent value="logai" className="mt-[21px]">
          <PlaceholderCard
            title={t.opsConsole.logAITitle}
            tag="F-02"
            items={[
              t.opsConsole.logAIUpload,
              t.opsConsole.logAIAnalysis,
              t.opsConsole.logAIChannelMock,
              t.opsConsole.logAINeedsReview,
            ]}
            accent="violet"
          />
        </TabsContent>

        <TabsContent value="metrics" className="mt-[21px]">
          <PlaceholderCard
            title={t.opsConsole.metricsTitle}
            tag="F-03"
            items={[t.opsConsole.metricsKPI]}
          />
        </TabsContent>

        <TabsContent value="docs" className="mt-[21px]">
          <PlaceholderCard
            title={t.opsConsole.docsTitle}
            tag="F-04"
            items={[t.opsConsole.docsCategory]}
          />
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
