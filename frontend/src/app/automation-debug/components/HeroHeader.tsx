"use client";

/**
 * HeroHeader — 调试控制台的"杂志封面"头部 (per 9/5 14:41 JST 用户拍板 q4-first-impression_opt1)
 *
 * 设计要点 (色彩心理学 + 平面设计理论 4 律):
 *   1. 对比 Contrast: 72pt 巨型渐变标题 vs 11pt micro 标签, 4.5x+ 视觉权重差
 *   2. 重复 Repetition: 3 个 KPI 胶囊共享同一节奏 (icon + label + value)
 *   3. 亲密性 Proximity: 标题+副标题 8px 间距, KPI 胶囊间 13px (φ 比例)
 *   4. 留白 White space: 上下 89px (φ 黄金分割), 左右 34px
 *
 * 神作感来源:
 *   - 巨型日文标题 "調 試 制 御 盤" + 渐变文字 (墨色 → 主蓝 → 紫)
 *   - 3 颗金属感 KPI 胶囊 (主蓝 / 紫 / 翡翠) 替代单调 Card 标题
 *   - 背景渐变 + Three.js 漂浮 3D 核心 (AnimeCore3D 动态注入)
 *   - HUD 战术角标 (SYS / LIVE / 版本) 制造工业仪表盘语言
 */

import dynamic from "next/dynamic";
import { Sparkles, Zap, Cpu, Activity } from "lucide-react";

// Three.js 走 dynamic SSR-false (per 守门 #22: 控制台不污染 main 编译链 + SSR)
const AnimeCore3D = dynamic(() => import("./AnimeCore3D"), {
  ssr: false,
  loading: () => null,
});

interface KpiPillProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  tone: "primary" | "violet" | "ok";
  pulse?: boolean;
}

function KpiPill({ icon, label, value, tone, pulse }: KpiPillProps) {
  const toneMap: Record<KpiPillProps["tone"], string> = {
    primary: "var(--color-primary)",
    violet:  "var(--color-accent-violet)",
    ok:      "var(--ok-DEFAULT)",
  };
  const c = toneMap[tone];
  return (
    <div
      className={`anime-panel lift-on-hover px-5 py-3 flex items-center gap-3 min-w-[130px] ${pulse ? "pulse-ok" : ""}`}
      style={{
        borderColor: `color-mix(in srgb, ${c} 35%, transparent)`,
      }}
    >
      <div
        className="p-2 rounded-md shrink-0"
        style={{
          background: `color-mix(in srgb, ${c} 14%, transparent)`,
          color: c,
          boxShadow: `0 0 14px color-mix(in srgb, ${c} 28%, transparent)`,
        }}
      >
        {icon}
      </div>
      <div className="min-w-0">
        <div className="text-[10px] text-ink-mute uppercase tracking-[0.12em] font-semibold font-mono">
          {label}
        </div>
        <div
          className="text-[20px] leading-[26px] font-bold font-mono tracking-tight"
          style={{ color: c }}
        >
          {value}
        </div>
      </div>
    </div>
  );
}

interface HeroHeaderProps {
  scriptCount: number;
  testCount?: number;
  runningCount?: number;
}

export function HeroHeader({ scriptCount, testCount = 5, runningCount = 0 }: HeroHeaderProps) {
  return (
    <section
      className="relative overflow-hidden anime-panel anime-chamfer mb-[21px] animate-[hero-fade-in_700ms_cubic-bezier(0.16,1,0.3,1)_both]"
      style={{
        paddingTop: "var(--space-9)",      // 55px 顶部 (φ)
        paddingBottom: "var(--space-9)",   // 55px 底部 (φ)
        paddingLeft: "var(--space-7)",     // 34px 左 (φ)
        paddingRight: "var(--space-7)",    // 34px 右 (φ)
      }}
    >
      {/* 3D 抽象核心 — 背景层, 主题自适应颜色 */}
      <AnimeCore3D />

      {/* 极光渐变 (4 个色块叠加, 透明度极低, 不抢戏) */}
      <div
        aria-hidden
        className="absolute inset-0 -z-20 pointer-events-none"
        style={{
          background:
            "radial-gradient(120% 80% at 0% 0%, color-mix(in srgb, var(--color-primary) 18%, transparent), transparent 60%)," +
            "radial-gradient(80% 60% at 100% 100%, color-mix(in srgb, var(--color-accent-violet) 14%, transparent), transparent 55%)," +
            "radial-gradient(60% 50% at 50% 50%, color-mix(in srgb, var(--color-secondary) 6%, transparent), transparent 70%)",
        }}
      />

      {/* HUD 角标 (右上) */}
      <div aria-hidden className="absolute top-4 right-6 flex gap-2 flex-wrap justify-end">
        <span className="anime-hud-tag">
          <span className="inline-block w-1.5 h-1.5 rounded-full mr-1 animate-[pulse-dot_1.6s_ease-in-out_infinite]"
            style={{ background: "var(--ok-DEFAULT)", boxShadow: "0 0 6px var(--ok-DEFAULT)" }} />
          SYS / LIVE
        </span>
        <span className="anime-hud-tag">v0.2.0</span>
        <span className="anime-hud-tag">P3-B-F</span>
      </div>

      <div className="relative flex items-end justify-between gap-[34px] flex-wrap">
        {/* 主标题区 */}
        <div className="min-w-0 flex-1">
          {/* 副标 eyebrow — 11pt micro, 紫罗兰 */}
          <div className="flex items-center gap-2 mb-3">
            <span className="anime-badge-neon">
              <Sparkles className="w-3 h-3" />
              AI-ASSISTED
            </span>
            <span className="text-[11px] text-ink-mute font-mono uppercase tracking-[0.14em]">
              Automation Debug Console
            </span>
          </div>

          {/* 巨型渐变标题 — 调 试 制 御 盤 */}
          <h1
            className="font-anime text-[clamp(48px,7.2vw,96px)] font-black leading-[0.95] tracking-[-0.03em] select-none"
            style={{
              background:
                "linear-gradient(135deg, var(--color-text) 0%, var(--color-primary) 55%, var(--color-accent-violet) 100%)",
              WebkitBackgroundClip: "text",
              WebkitTextFillColor: "transparent",
              backgroundClip: "text",
              filter: "drop-shadow(0 2px 12px color-mix(in srgb, var(--color-primary) 18%, transparent))",
            }}
          >
            調 試 制 御 盤
          </h1>

          {/* 副标题 — 重要数据点 (大字号 + 等宽数字, 强烈对比) */}
          <p className="mt-3 text-[15px] leading-[22px] text-ink-dim">
            <span className="font-mono font-bold text-ink-DEFAULT text-[18px]">{scriptCount}</span> 份脚本
            <span className="mx-2 text-ink-mute">·</span>
            <span className="font-mono font-bold text-ink-DEFAULT text-[18px]">{testCount}</span> 套 unittest
            <span className="mx-2 text-ink-mute">·</span>
            <span className="font-mono text-ink-mute">docs/automation-design.md §12</span>
          </p>
        </div>

        {/* KPI 胶囊组 — 工业仪表盘语言 */}
        <div className="flex gap-[13px] flex-wrap">
          <KpiPill
            icon={<Zap className="w-4 h-4" />}
            label="运行中"
            value={String(runningCount)}
            tone="primary"
          />
          <KpiPill
            icon={<Cpu className="w-4 h-4" />}
            label="脚本池"
            value={`${scriptCount}/${scriptCount}`}
            tone="violet"
          />
          <KpiPill
            icon={<Activity className="w-4 h-4" />}
            label="健康度"
            value="A+"
            tone="ok"
            pulse
          />
        </div>
      </div>

      {/* 装饰底边线 (漫画分镜感) */}
      <div
        aria-hidden
        className="absolute bottom-0 left-0 right-0 h-[1px]"
        style={{
          background:
            "linear-gradient(90deg, transparent 0%, var(--color-primary) 30%, var(--color-accent-violet) 70%, transparent 100%)",
          opacity: 0.5,
        }}
      />
    </section>
  );
}
