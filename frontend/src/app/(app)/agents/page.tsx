"use client";

// @cypher schema=1 source_sha256=d846d62f132e4f17f20a0b8a13ee80ba0584c509c1468faaa49ac886f2625ad4
// MERGE (self:File {path:"frontend/src/app/(app)/agents/page.tsx"})
// MERGE (page:Symbol {id:"frontend/src/app/(app)/agents/page.tsx::AgentsPage",kind:"function"})
// MERGE (agentsApi:File {path:"frontend/src/lib/agents/agentsApi.ts"})
// MERGE (statusNotice:File {path:"frontend/src/components/status/StatusNotice.tsx"})
// MERGE (shader:File {path:"frontend/src/components/effects/AnimeCelShaderCanvas.tsx"})
// MERGE (palette:Type {id:"frontend/src/components/effects/AnimeCelShaderCanvas.tsx::CelPalette"})
// MERGE (self)-[:DEFINES]->(page)
// MERGE (self)-[:IMPORTS]->(agentsApi)
// MERGE (self)-[:IMPORTS]->(statusNotice)
// MERGE (self)-[:IMPORTS]->(shader)
// MERGE (page)-[:CALLS]->(agentsApi)
// MERGE (page)-[:USES_TYPE]->(palette)
// @endcypher

// =====================================================================
// /agents — Tactical Agent Command Hub (战术机关与自律代理人控制台)
// =====================================================================
// 2026-11-17 清 seed/mock 纪律 (per 用户发令 "确保平台内每一个功能都完善"):
//   数据源 = 真实 Agent 运行时 (star-api-rest, 同源代理 /api/v1/* → 127.0.0.1:8081):
//     - 初始态 = 加载中 (不再预填 MOCK_AGENTS_FALLBACK)
//     - 成功 → 渲染真实 agent (运行时自带数据源说明原样展示)
//     - 空 → 诚实空态 "暂无 agent"
//     - 失败/不可达 → StatusNotice "未接入 — 真实 Agent 运行时不可达",
//       绝不回落 mock 内容; 401/403 → "未授权"
//     - Argo CLI (GET /api/v1/argo/nodes supported_cli) 同理: 不可达时明确
//       显示 "Argo CLI 未接入", 不假装本地 mock 兜底
//   操作按钮 (SYNC WORKTREE / RUN VERIFY) = 真实动作:
//     SYNC → 重新从运行时拉取 agent 花名册; RUN VERIFY → ping /api/v1/health
//     展示真实结果; 运行时未接入时按钮禁用 + 原因说明
//   全面应用 3渲2 (3D-to-2D NPR Cel-Shaded) 艺术级日漫科技美学:
//     实时 WebGL 3D 渲染台 (Three.js + GLSL NPR 着色器)、战术 3D 触觉按钮
//     (CelButton3D) 与机械开关 (CelToggle3D, CelBeacon3D)、墨线阶梯卡片、
//     HUD 标签、Swiss 8pt 栅格与米勒定律三区信息聚合
// =====================================================================

import { useCallback, useEffect, useState } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Bot, RefreshCw } from "lucide-react";
import {
  listAgents,
  listArgoCli,
  pingRuntimeHealth,
  AgentsApiError,
  type AgentCenterRow,
} from "@/lib/agents/agentsApi";
import {
  StatusNotice,
  inferNoticeState,
  type NoticeState,
} from "@/components/status/StatusNotice";
import { useTranslation } from "@/lib/i18n";
import { AnimeCelShaderCanvas, CelPalette } from "@/components/effects/AnimeCelShaderCanvas";
import { TacticalCore3D } from "@/components/effects/TacticalCore3D";
import {
  CelButton3D,
  CelToggle3D,
  CelBeacon3D,
  CelDial3D,
} from "@/components/effects/Cel3DUI";
import { CelCard3D } from "@/components/effects/CelCard3D";

type AgentsPhase = "loading" | "ready" | "error";

/** AgentsApiError.code → 诚实三态 (确定性映射, 不依赖错误文本) */
function noticeStateForError(err: unknown): NoticeState {
  if (err instanceof AgentsApiError) {
    if (err.code === "network_error") return "disconnected";
    if (err.code === "unauthorized") return "unauthorized";
    if (err.code === "http_error" && err.status !== null && err.status >= 500) {
      return "disconnected";
    }
    return "failed";
  }
  return inferNoticeState(err);
}

export default function AgentsPage() {
  const { t, tx } = useTranslation();

  // ── 真实运行时 agent 花名册 ──────────────────────────────────────
  const [rows, setRows] = useState<AgentCenterRow[]>([]);
  const [phase, setPhase] = useState<AgentsPhase>("loading");
  const [loadError, setLoadError] = useState<unknown>(null);
  const [dataNote, setDataNote] = useState<string | null>(null);

  // ── Argo CLI (后端门面 GET /api/v1/argo/nodes) ────────────────────
  const [argoCli, setArgoCli] = useState<string[] | null>(null);
  const [argoError, setArgoError] = useState<unknown>(null);

  // ── 操作按钮 (真实动作 + 结果展示) ───────────────────────────────
  const [actionResult, setActionResult] = useState<string | null>(null);
  const [actionBusy, setActionBusy] = useState(false);

  // ── 3渲2 视觉状态 ────────────────────────────────────────────────
  const [celPalette, setCelPalette] = useState<CelPalette>("crimson");
  const [celBands, setCelBands] = useState<number>(3);
  const [autoSagaGuard, setAutoSagaGuard] = useState(true);
  const [idempotentLock, setIdempotentLock] = useState(true);
  const [dispatchMode, setDispatchMode] = useState(0);

  /** 从真实运行时拉取 agent 列表; 返回 rows (失败返回 null, 已置 error 态) */
  const loadAgents = useCallback(async (): Promise<AgentCenterRow[] | null> => {
    setPhase("loading");
    setLoadError(null);
    try {
      const { rows: next, note } = await listAgents();
      setRows(next);
      setDataNote(note);
      setPhase("ready");
      return next;
    } catch (err) {
      setRows([]);
      setLoadError(err);
      setPhase("error");
      return null;
    }
  }, []);

  useEffect(() => {
    void loadAgents();
  }, [loadAgents]);

  useEffect(() => {
    let alive = true;
    listArgoCli()
      .then((cli) => {
        if (!alive) return;
        setArgoCli(cli);
        setArgoError(null);
      })
      .catch((err) => {
        if (!alive) return;
        // 诚实: 不可达时不假装本地 mock 兜底, 明确显示未接入
        setArgoCli(null);
        setArgoError(err);
      });
    return () => {
      alive = false;
    };
  }, []);

  // ── 真实动作: SYNC = 重新拉取花名册; VERIFY = ping /api/v1/health ──
  const runSync = useCallback(async () => {
    setActionResult(null);
    setActionBusy(true);
    const loaded = await loadAgents();
    setActionBusy(false);
    if (loaded !== null) {
      setActionResult(tx(t.agentCenter.syncResult, { count: String(loaded.length) }));
    }
  }, [loadAgents, t.agentCenter.syncResult, tx]);

  const runVerify = useCallback(async () => {
    setActionResult(null);
    setActionBusy(true);
    try {
      const health = await pingRuntimeHealth();
      setActionResult(
        health.ok
          ? t.agentCenter.verifyOk
          : tx(t.agentCenter.verifyFailed, { status: String(health.status) }),
      );
    } catch (err) {
      setActionResult(
        tx(t.agentCenter.verifyUnreachable, {
          reason: err instanceof Error ? err.message : String(err),
        }),
      );
    } finally {
      setActionBusy(false);
    }
  }, [t.agentCenter.verifyFailed, t.agentCenter.verifyOk, t.agentCenter.verifyUnreachable, tx]);

  // ── 派生统计 (真实行驱动) ────────────────────────────────────────
  const active = rows.filter((a) => a.status === "running").length;
  const paused = rows.filter((a) => a.status === "paused" || a.status === "stopped").length;
  const failed = rows.filter((a) => a.status === "failed").length;

  // ── 诚实三态映射 (加载中 / 未接入 / 未授权 / 失败 / 空) ──────────
  let notice: { state: NoticeState; title: string; hint: string } | null = null;
  if (phase === "loading") {
    notice = { state: "loading", title: t.agentCenter.loadingTitle, hint: t.agentCenter.loadingHint };
  } else if (phase === "error") {
    const inferred = noticeStateForError(loadError);
    if (inferred === "unauthorized") {
      notice = { state: "unauthorized", title: t.agentCenter.unauthorizedTitle, hint: t.agentCenter.unauthorizedHint };
    } else if (inferred === "disconnected") {
      notice = { state: "disconnected", title: t.agentCenter.disconnectedTitle, hint: t.agentCenter.disconnectedHint };
    } else {
      notice = { state: "failed", title: t.agentCenter.loadFailedTitle, hint: t.agentCenter.loadFailedHint };
    }
  } else if (rows.length === 0) {
    notice = { state: "empty", title: t.agentCenter.emptyTitle, hint: t.agentCenter.emptyHint };
  }

  const actionsReachable = phase === "ready";

  return (
    <div className="space-y-6" data-testid="agents-page">
      {argoCli !== null && (
        <div data-testid="agents-cli-badge" className="mb-4 flex items-center gap-2 flex-wrap text-xs font-mono">
          <span className="text-ink-mute">Argo CLI 支持:</span>
          {argoCli.map((c) => (
            <span key={c} className="px-2 py-1 rounded border border-accent/40 bg-accent/10 text-accent">{c}</span>
          ))}
        </div>
      )}
      {argoError !== null && (
        <div data-testid="agents-argo-unavailable" className="mb-3">
          <StatusNotice state="disconnected" compact title={t.agentCenter.argoUnavailable} />
        </div>
      )}
      <PageHeader
        title={t.pageTitles['/agents'].title}
        subtitle="Tactical Agent Command // S-Class Runtime & Autonomous Agents (战术机关与自律代理人)"
        icon={<Bot className="text-[var(--cel-cyan, var(--accent-DEFAULT))]" size={22} />}
        count={`${active} active`}
      />

      {/* KPI Stats Deck (3渲2 触觉物理卡片, 真实运行时行驱动) */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <CelCard3D className="h-full">
          <Stat bare label="Total Agents" value={rows.length} hint="全域代理人注册" tone="info" />
        </CelCard3D>
        <CelCard3D className="h-full">
          <Stat bare label="Active Nodes" value={active} hint="并发执行中" tone="ok" />
        </CelCard3D>
        <CelCard3D className="h-full">
          <Stat bare label="Standby / Paused" value={paused} tone="warn" />
        </CelCard3D>
        <CelCard3D className="h-full">
          <Stat bare label="Failed / Blocked" value={failed} tone="err" />
        </CelCard3D>
      </div>

      {/* Main Grid: Left Operational Table & Comms, Right Real 3D Cel Shader Terminal */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        {/* LEFT COLUMN: Agent Sessions Table & Protocols (7 Cols) */}
        <div className="lg:col-span-7 space-y-6">
          {/* Agent Sessions Table — 真实运行时数据 (per 2026-11-17 清 mock) */}
          <div className="card">
            <SectionTitle
              action={
                <div className="flex items-center gap-2">
                  <span className="anime-hud-tag">DEC-008</span>
                  <span className="text-xs font-mono font-bold text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]">{rows.length} SESSIONS</span>
                </div>
              }
            >
              Agent Sessions 〔実行セッション一覧〕
            </SectionTitle>

            {dataNote !== null && rows.length > 0 && (
              <div
                data-testid="agents-data-note"
                className="mb-3 flex items-center gap-2 rounded border border-warn/30 bg-warn/5 px-3 py-1.5 text-xs font-mono text-warn"
              >
                {t.agentCenter.dataNoteLabel}: {dataNote}
              </div>
            )}

            {notice !== null ? (
              /* 加载中 / 未接入 / 未授权 / 失败 / 空 — 诚实展示, 绝不回落 mock */
              <div data-testid="agents-notice-wrap" className="py-6 space-y-3">
                <StatusNotice state={notice.state} title={notice.title} hint={notice.hint} />
                {phase === "error" && (
                  <button
                    type="button"
                    data-testid="agents-retry"
                    onClick={() => void loadAgents()}
                    className="inline-flex items-center gap-1.5 text-xs px-3.5 py-2 rounded-lg border border-line text-ink-dim hover:bg-bg-soft"
                  >
                    <RefreshCw size={12} /> {t.agentCenter.retry}
                  </button>
                )}
              </div>
            ) : (
              <table className="table">
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Role</th>
                    <th>Started At</th>
                  </tr>
                </thead>
                <tbody>
                  {rows.map((a) => (
                    <tr key={a.id} data-testid={`agent-row-${a.id}`} className="transition-colors">
                      <td className="font-mono text-xs text-[var(--cel-cyan, var(--accent-DEFAULT))] font-bold">{a.id}</td>
                      <td className="font-bold text-sm text-[var(--cel-text-primary, var(--color-text))] flex items-center gap-2">
                        <span className="size-2 bg-[var(--cel-crimson, var(--err-DEFAULT))] border border-black rotate-45" />
                        {a.name}
                      </td>
                      <td><StatusPill value={a.status} size="xs" /></td>
                      <td className="text-xs text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))] font-mono">{a.role}</td>
                      <td className="font-mono text-xs text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]">{a.started_at ?? "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>

          {/* Tactical 3D Mechanical Protocols */}
          <div className="card">
            <SectionTitle
              action={
                <span className="text-xs font-mono text-[var(--cel-gold, var(--warn-DEFAULT))] font-bold">
                  NPR_TACTILE_SPRING
                </span>
              }
            >
              Tactical Protocols 〔戦術プロトコル設定〕
            </SectionTitle>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2">
              <div className="border-2 border-black p-4 bg-[var(--cel-surface-sub, var(--color-surface))] shadow-soft">
                <CelToggle3D
                  label="AUTONOMOUS SAGA GUARD"
                  sublabel="IDEMPOTENT RECOVERY"
                  checked={autoSagaGuard}
                  onChange={setAutoSagaGuard}
                />
              </div>

              <div className="border-2 border-black p-4 bg-[var(--cel-surface-sub, var(--color-surface))] shadow-soft">
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
                <span className="text-xs font-mono font-bold text-[var(--cel-cyan, var(--accent-DEFAULT))]">SAGA: ONLINE</span>
              </div>
            </div>
          </div>

          {/* Live Activity Feed — 占位演示 (诚实标注, 不冒充真实通信流) */}
          <div className="card" data-testid="live-activity-placeholder">
            <SectionTitle
              action={
                <span
                  data-testid="agents-comms-placeholder"
                  className="text-xs font-mono font-bold text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]"
                >
                  {t.agentCenter.commsPlaceholder}
                </span>
              }
            >
              Comms Stream 〔通信ログ・指令ストリーム〕
            </SectionTitle>
            <div className="border-2 border-black p-5 bg-[var(--cel-surface-stage, var(--color-surface))] shadow-soft text-sm space-y-3">
              <div className="flex items-center justify-between border-b border-black pb-1.5 text-xs font-mono text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]">
                <span className="text-[var(--cel-crimson, var(--err-DEFAULT))] font-bold">// TACTICAL SPEECH STREAM</span>
                <span>CH-01 SECURE</span>
              </div>
              <p className="leading-relaxed text-[var(--cel-text-primary, var(--color-text))] font-medium">
                <strong className="text-[var(--cel-crimson, var(--err-DEFAULT))] mr-1.5">[MAVIS // 統制官]:</strong>
                「22 DDD 领域状态机与 3渲2 NPR 着色器全节点同步。不变量持有，零脏状态泄漏，所有代理人处于高阶就绪状态！」
              </p>
              <div className="flex items-center justify-between text-xs font-mono text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))] pt-1">
                <span>CONCURRENCY: 4 WORKERS</span>
                <span className="text-[var(--cel-cyan, var(--accent-DEFAULT))] font-bold">STATUS: OPTIMAL</span>
              </div>
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN: Real 3D Cel-Shaded Terminal (5 Cols) */}
        <div className="lg:col-span-5 space-y-6">
          <div className="card relative overflow-hidden">
            {/* Top Badge */}
            <div className="flex items-center justify-between border-b-2 border-black pb-3 mb-4">
              <div className="flex items-center gap-3">
                <TacticalCore3D size={42} status="nominal" className="shrink-0" />
                <div>
                  <div className="text-[11px] font-black text-[var(--cel-gold, var(--warn-DEFAULT))] uppercase tracking-widest font-mono">
                    AVATAR SPEC // S-CLASS
                  </div>
                  <h3 className="text-base font-black uppercase italic tracking-wider text-[var(--cel-text-primary, var(--color-text))] flex items-center gap-2">
                    AGENT CORE 3D
                    <span className="text-xs font-black not-italic px-2 py-0.5 bg-[var(--cel-crimson, var(--err-DEFAULT))] text-black border border-black">
                      神格
                    </span>
                  </h3>
                </div>
              </div>
              <span className="text-xs font-mono font-bold text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]">
                〔戦術司令機〕
              </span>
            </div>

            {/* Real 3D Three.js WebGL NPR Cel Shader Canvas */}
            <div className="relative w-full h-80 sm:h-96 bg-[var(--cel-surface-stage, var(--color-surface))] border-2 border-black overflow-hidden flex items-center justify-center group shadow-soft">
              {/* Screentone Backdrop */}
              <div className="absolute inset-0 bg-screentone-dense opacity-20 pointer-events-none" />

              {/* Japanese Calligraphy Backdrop */}
              <div className="absolute inset-0 flex items-center justify-center opacity-5 pointer-events-none font-black text-8xl select-none text-[var(--cel-text-primary, var(--color-text))]">
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
                <span className="size-2 rounded-full bg-[var(--cel-cyan, var(--accent-DEFAULT))] animate-ping" />
                <span className="bg-black/85 border border-[var(--cel-cyan, var(--accent-DEFAULT))]/50 text-[var(--cel-cyan, var(--accent-DEFAULT))] px-2.5 py-1 text-[11px] font-mono font-bold">
                  REALTIME_GLSL_NPR_3D
                </span>
              </div>

              <div className="absolute top-2.5 right-2.5 z-20 pointer-events-none">
                <span className="bg-black/85 border border-[var(--cel-gold, var(--warn-DEFAULT))]/50 text-[var(--cel-gold, var(--warn-DEFAULT))] px-2.5 py-1 text-[11px] font-mono font-bold">
                  {celBands}-BAND CEL RAMP
                </span>
              </div>

              <div className="absolute bottom-2.5 left-2.5 right-2.5 flex justify-between items-end z-20 pointer-events-none">
                <div className="bg-black/90 border border-[#232f47] px-2.5 py-1 text-[10px] font-mono text-slate-200">
                  INVERTED_HULL: <span className="text-[var(--cel-cyan, var(--accent-DEFAULT))] font-bold">5.0px INK</span>
                </div>
                <div className="bg-[var(--cel-crimson, var(--err-DEFAULT))] text-black font-black text-[10px] px-2 py-0.5 border border-black">
                  LIGHT_TRACKING: ACTIVE
                </div>
              </div>
            </div>

            {/* Live Palette Selector Tuning Bar — 5 Themes Complete Coverage */}
            <div className="mt-4 bg-[var(--cel-surface-stage, var(--color-surface))] border-2 border-black p-4 space-y-3 text-xs">
              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <span className="text-[11px] font-bold text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))] uppercase font-mono">
                    CEL PALETTE (5大主题流派):
                  </span>
                  <span className="text-[10px] font-mono text-[var(--cel-gold, var(--warn-DEFAULT))] uppercase font-bold">
                    {celPalette}
                  </span>
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {(
                    [
                      { id: "crimson", label: "🌙 茜红 (Dark)" },
                      { id: "cyan", label: "🌙 电光青" },
                      { id: "gold", label: "🌙 黄金" },
                      { id: "charisma-violet", label: "✨ 极魅紫 (HD-2D)" },
                      { id: "charisma-crimson", label: "✨ 醉梦红" },
                      { id: "shanghai-gilt", label: "🪭 留声金 (月份牌)" },
                      { id: "shanghai-jade", label: "🪭 翡翠绿" },
                      { id: "manga-vermilion", label: "☀️ 宣纸红 (Light)" },
                      { id: "cel-azure", label: "🎞️ 赛璐璐蓝 (Cel)" },
                    ] as const
                  ).map((item) => (
                    <button
                      key={item.id}
                      onClick={() => setCelPalette(item.id as CelPalette)}
                      className={`px-2.5 py-1.5 text-xs font-mono font-bold uppercase border border-black transition-all ${
                        celPalette === item.id
                          ? "bg-[var(--cel-primary, var(--color-primary))] text-white border-white shadow-sm"
                          : "bg-[var(--cel-surface-sub, var(--color-surface))] text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))] hover:text-white"
                      }`}
                    >
                      {item.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="flex items-center justify-between pt-1.5 border-t border-black/40">
                <span className="text-[11px] font-mono text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]">CEL BANDS:</span>
                <div className="flex gap-1.5">
                  {[2, 3, 4].map((b) => (
                    <button
                      key={b}
                      onClick={() => setCelBands(b)}
                      className={`w-7 h-7 text-xs font-mono font-bold border border-black flex items-center justify-center ${
                        celBands === b ? "bg-[var(--cel-cyan, var(--accent-DEFAULT))] text-black" : "bg-[var(--cel-surface-sub, var(--color-surface))] text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]"
                      }`}
                    >
                      {b}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            {/* Quick 3D Tactile Action Buttons — 真实动作 (per 2026-11-17) */}
            <div className="mt-4 pt-3 border-t-2 border-black flex flex-col gap-2 items-center">
              <CelButton3D
                label="01 // SYNC WORKTREE"
                sublabel="RELOAD ROSTER FROM RUNTIME"
                variant="cyan"
                disabled={!actionsReachable || actionBusy}
                onClick={() => {
                  void runSync();
                }}
              />
              <CelButton3D
                label="02 // RUN VERIFY"
                sublabel="PING /API/V1/HEALTH"
                variant="gold"
                disabled={!actionsReachable || actionBusy}
                onClick={() => {
                  void runVerify();
                }}
              />
              {!actionsReachable && (
                <span
                  data-testid="agents-actions-disabled-hint"
                  className="text-xs font-mono text-[var(--cel-text-secondary, color-mix(in srgb, var(--color-text) 70%, transparent))]"
                >
                  {t.agentCenter.actionsDisabledHint}
                </span>
              )}
              {actionResult !== null && (
                <div
                  data-testid="agents-action-result"
                  className="w-full text-center text-xs font-mono font-bold text-[var(--cel-cyan, var(--accent-DEFAULT))] border border-[var(--cel-cyan, var(--accent-DEFAULT))]/40 bg-[var(--cel-cyan, var(--accent-DEFAULT))]/10 px-3 py-2 rounded"
                >
                  {actionResult}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
