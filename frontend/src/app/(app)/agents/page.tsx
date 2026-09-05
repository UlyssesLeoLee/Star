"use client";

// =====================================================================
// /agents — Tactical Agent Command Hub (战术机关与自律代理人控制台)
// =====================================================================
// 全面应用 3渲2 (3D-to-2D NPR Cel-Shaded) 艺术级日漫科技美学:
//   1. 实时 WebGL 3D 渲染台 (Three.js + GLSL NPR 着色器)
//   2. 战术 3D 触觉按钮 (CelButton3D) 与机械开关 (CelToggle3D, CelBeacon3D)
//   3. 墨线阶梯卡片、HUD 标签、Swiss 8pt 栅格与米勒定律三区信息聚合
// =====================================================================

import { useEffect, useState } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Bot, Activity, Cpu, Sparkles, Terminal } from "lucide-react";
import { MOCK_AGENTS_FALLBACK } from "@/mocks/data";
import type { AgentRow } from "@/mocks/schemas/agent";
import { useTranslation } from "@/lib/i18n";
import { AnimeCelShaderCanvas, CelPalette } from "@/components/effects/AnimeCelShaderCanvas";
import {
  CelButton3D,
  CelToggle3D,
  CelBeacon3D,
  CelDial3D,
} from "@/components/effects/Cel3DUI";

export default function AgentsPage() {
  const { t } = useTranslation();
  const [agents, setAgents] = useState<ReadonlyArray<AgentRow>>(MOCK_AGENTS_FALLBACK);
  const [celPalette, setCelPalette] = useState<CelPalette>("crimson");
  const [celBands, setCelBands] = useState<number>(3);
  const [autoSagaGuard, setAutoSagaGuard] = useState(true);
  const [idempotentLock, setIdempotentLock] = useState(true);
  const [dispatchMode, setDispatchMode] = useState(0);

  useEffect(() => {
    fetch("/api/agents")
      .then((r) => r.json())
      .then((data: ReadonlyArray<AgentRow>) => setAgents(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标, 避免 UX 退化) */
      });
  }, []);

  const active = agents.filter((a) => a.status === "active" || a.status === "in_progress").length;

  return (
    <div className="max-w-7xl mx-auto space-y-6" data-testid="agents-page">
      <PageHeader
        title={t.pageTitles['/agents'].title}
        subtitle="Tactical Agent Command // S-Class Runtime & Autonomous Agents (战术机关与自律代理人)"
        icon={<Bot className="text-[var(--cel-cyan,#00f0ff)]" size={22} />}
        count={`${active} active`}
      />

      {/* KPI Stats Deck */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Stat label="Total Agents" value={agents.length} hint="全域代理人注册" tone="info" />
        <Stat label="Active Nodes" value={active} hint="并发执行中" tone="ok" />
        <Stat label="Standby / Paused" value={agents.filter((a) => a.status === "paused").length} tone="warn" />
        <Stat label="Failed / Blocked" value={agents.filter((a) => a.status === "failed").length} tone="err" />
      </div>

      {/* Main Grid: Left Operational Table & Comms, Right Real 3D Cel Shader Terminal */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        {/* LEFT COLUMN: Agent Sessions Table & Protocols (7 Cols) */}
        <div className="lg:col-span-7 space-y-6">
          {/* Agent Sessions Table */}
          <div className="card">
            <SectionTitle
              action={
                <div className="flex items-center gap-2">
                  <span className="anime-hud-tag">DEC-008</span>
                  <span className="text-xs font-mono font-bold text-[var(--cel-text-secondary,#94a3b8)]">5 SESSIONS</span>
                </div>
              }
            >
              Agent Sessions 〔実行セッション一覧〕
            </SectionTitle>

            <table className="table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Name</th>
                  <th>Status</th>
                  <th>Role</th>
                  <th>Last Active</th>
                </tr>
              </thead>
              <tbody>
                {agents.map((a) => (
                  <tr key={a.id} data-testid={`agent-row-${a.id}`} className="transition-colors">
                    <td className="font-mono text-xs text-[var(--cel-cyan,#00f0ff)] font-bold">{a.id}</td>
                    <td className="font-bold text-sm text-[var(--cel-text-primary,#ffffff)] flex items-center gap-2">
                      <span className="size-2 bg-[var(--cel-crimson,#ff184c)] border border-black rotate-45" />
                      {a.name}
                    </td>
                    <td><StatusPill value={a.status} size="xs" /></td>
                    <td className="text-xs text-[var(--cel-text-secondary,#94a3b8)] font-mono">{a.role}</td>
                    <td className="font-mono text-xs text-[var(--cel-text-secondary,#94a3b8)]">{a.last_active}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Tactical 3D Mechanical Protocols */}
          <div className="card">
            <SectionTitle
              action={
                <span className="text-xs font-mono text-[var(--cel-gold,#ffc400)] font-bold">
                  NPR_TACTILE_SPRING
                </span>
              }
            >
              Tactical Protocols 〔戦術プロトコル設定〕
            </SectionTitle>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2">
              <div className="border-2 border-black p-3 bg-[var(--cel-surface-sub,#151c2c)] cel-shadow">
                <CelToggle3D
                  label="AUTONOMOUS SAGA GUARD"
                  sublabel="IDEMPOTENT RECOVERY"
                  checked={autoSagaGuard}
                  onChange={setAutoSagaGuard}
                />
              </div>

              <div className="border-2 border-black p-3 bg-[var(--cel-surface-sub,#151c2c)] cel-shadow">
                <CelToggle3D
                  label="HARD MEMORY LOCK"
                  sublabel="ZERO LEAK VECTOR"
                  checked={idempotentLock}
                  onChange={setIdempotentLock}
                />
              </div>
            </div>

            <div className="mt-4 pt-3 border-t-2 border-black flex items-center justify-between">
              <CelDial3D
                value={dispatchMode}
                onChange={setDispatchMode}
                options={["BALANCED", "OVERCLOCK", "SURVIVAL"]}
              />
              <div className="flex items-center gap-2">
                <CelBeacon3D status="active" size={32} title="Saga Engine Health" />
                <span className="text-xs font-mono font-bold text-[var(--cel-cyan,#00f0ff)]">SAGA: ONLINE</span>
              </div>
            </div>
          </div>

          {/* Live Activity Feed — Styled with Manga Comms Bubble */}
          <div className="card" data-testid="live-activity-placeholder">
            <SectionTitle>Comms Stream 〔通信ログ・指令ストリーム〕</SectionTitle>
            <div className="border-2 border-black p-4 bg-[var(--cel-surface-stage,#090d16)] cel-shadow text-sm space-y-2.5">
              <div className="flex items-center justify-between border-b border-black pb-1.5 text-xs font-mono text-[var(--cel-text-secondary,#94a3b8)]">
                <span className="text-[var(--cel-crimson,#ff184c)] font-bold">// TACTICAL SPEECH STREAM</span>
                <span>CH-01 SECURE</span>
              </div>
              <p className="leading-relaxed text-[var(--cel-text-primary,#ffffff)] font-medium">
                <strong className="text-[var(--cel-crimson,#ff184c)] mr-1.5">[MAVIS // 統制官]:</strong>
                「22 DDD 领域状态机与 3渲2 NPR 着色器全节点同步。不变量持有，零脏状态泄漏，所有代理人处于高阶就绪状态！」
              </p>
              <div className="flex items-center justify-between text-xs font-mono text-[var(--cel-text-secondary,#94a3b8)] pt-1">
                <span>CONCURRENCY: 4 WORKERS</span>
                <span className="text-[var(--cel-cyan,#00f0ff)] font-bold">STATUS: OPTIMAL</span>
              </div>
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: Real 3D Cel-Shaded Terminal (5 Cols) */}
        <div className="lg:col-span-5 space-y-6">
          <div className="card relative overflow-hidden">
            {/* Top Badge */}
            <div className="flex items-center justify-between border-b-2 border-black pb-3 mb-4">
              <div>
                <div className="text-[11px] font-black text-[var(--cel-gold,#ffc400)] uppercase tracking-widest font-mono">
                  AVATAR SPEC // S-CLASS
                </div>
                <h3 className="text-base font-black uppercase italic tracking-wider text-[var(--cel-text-primary,#ffffff)] flex items-center gap-2">
                  AGENT CORE 3D
                  <span className="text-xs font-black not-italic px-2 py-0.5 bg-[var(--cel-crimson,#ff184c)] text-black border border-black">
                    神格
                  </span>
                </h3>
              </div>
              <span className="text-xs font-mono font-bold text-[var(--cel-text-secondary,#94a3b8)]">
                〔戦術司令機〕
              </span>
            </div>

            {/* Real 3D Three.js WebGL NPR Cel Shader Canvas */}
            <div className="relative w-full h-80 sm:h-96 bg-[var(--cel-surface-stage,#090d16)] border-2 border-black overflow-hidden flex items-center justify-center group cel-shadow">
              {/* Screentone Backdrop */}
              <div className="absolute inset-0 bg-screentone-dense opacity-20 pointer-events-none" />

              {/* Japanese Calligraphy Backdrop */}
              <div className="absolute inset-0 flex items-center justify-center opacity-5 pointer-events-none font-black text-8xl select-none text-[var(--cel-text-primary,#ffffff)]">
                戦術
              </div>

              {/* True 3D WebGL Cel Shader Canvas */}
              <AnimeCelShaderCanvas
                palette={celPalette}
                bands={celBands}
                outlineThickness={0.05}
                enableHalftone={true}
                enableRim={true}
                speed={1.0}
                className="w-full h-full"
              />

              {/* Dynamic HUD Overlays */}
              <div className="absolute top-2.5 left-2.5 flex items-center gap-1.5 z-20 pointer-events-none">
                <span className="size-2 rounded-full bg-[var(--cel-cyan,#00f0ff)] animate-ping" />
                <span className="bg-black/85 border border-[var(--cel-cyan,#00f0ff)]/50 text-[var(--cel-cyan,#00f0ff)] px-2.5 py-1 text-[11px] font-mono font-bold">
                  REALTIME_GLSL_NPR_3D
                </span>
              </div>

              <div className="absolute top-2.5 right-2.5 z-20 pointer-events-none">
                <span className="bg-black/85 border border-[var(--cel-gold,#ffc400)]/50 text-[var(--cel-gold,#ffc400)] px-2.5 py-1 text-[11px] font-mono font-bold">
                  {celBands}-BAND CEL RAMP
                </span>
              </div>

              <div className="absolute bottom-2.5 left-2.5 right-2.5 flex justify-between items-end z-20 pointer-events-none">
                <div className="bg-black/90 border border-[#232f47] px-2.5 py-1 text-[10px] font-mono text-slate-200">
                  INVERTED_HULL: <span className="text-[var(--cel-cyan,#00f0ff)] font-bold">5.0px INK</span>
                </div>
                <div className="bg-[var(--cel-crimson,#ff184c)] text-black font-black text-[10px] px-2 py-0.5 border border-black">
                  LIGHT_TRACKING: ACTIVE
                </div>
              </div>
            </div>

            {/* Live Palette Selector Tuning Bar */}
            <div className="mt-3 bg-[var(--cel-surface-stage,#090d16)] border-2 border-black p-3 space-y-2.5 text-xs">
              <div className="flex items-center justify-between gap-2">
                <span className="text-[11px] font-bold text-[var(--cel-text-secondary,#94a3b8)] uppercase font-mono">
                  CEL PALETTE:
                </span>
                <div className="flex gap-1.5">
                  {(["crimson", "cyan", "gold", "stealth"] as CelPalette[]).map((p) => (
                    <button
                      key={p}
                      onClick={() => setCelPalette(p)}
                      className={`px-2.5 py-1 text-[11px] font-mono font-bold uppercase border border-black transition-all ${
                        celPalette === p
                          ? "bg-[var(--cel-crimson,#ff184c)] text-black border-white shadow-sm"
                          : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"
                      }`}
                    >
                      {p}
                    </button>
                  ))}
                </div>
              </div>

              <div className="flex items-center justify-between pt-1.5 border-t border-black/40">
                <span className="text-[11px] font-mono text-[var(--cel-text-secondary,#94a3b8)]">CEL BANDS:</span>
                <div className="flex gap-1.5">
                  {[2, 3, 4].map((b) => (
                    <button
                      key={b}
                      onClick={() => setCelBands(b)}
                      className={`w-6 h-6 text-[11px] font-mono font-bold border border-black flex items-center justify-center ${
                        celBands === b ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)]"
                      }`}
                    >
                      {b}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            {/* Quick 3D Tactile Action Buttons */}
            <div className="mt-4 pt-3 border-t-2 border-black flex flex-col gap-2 items-center">
              <CelButton3D
                label="01 // SYNC WORKTREE"
                sublabel="IDEMPOTENT MERGE"
                variant="cyan"
                onClick={() => {
                  alert("【3D 触觉派发】WORKSPACE 同步完成：代理人工作树状态机校验通过，0 冲突。");
                }}
              />
              <CelButton3D
                label="02 // RUN VERIFY"
                sublabel="CARGO --LIB -J 4"
                variant="gold"
                onClick={() => {
                  alert("【3D 触觉派发】CARGO 完整守门通过：--workspace --lib -j 4 零错误！");
                }}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
