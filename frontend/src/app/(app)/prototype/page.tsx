/*
@cypher schema=1 source_sha256=0b63c321926967237e04ba296ab47eb8449268f4c4c83d0f66d61f53ef2b5d2f
MERGE (self:File {path:"frontend/src/app/(app)/prototype/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/prototype/page.tsx::PrototypePage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /prototype — 2d · 原型 (per 2026-11-14 参考 mockup: Prototype.dc.html)
// 结构: 左 生成类型(1D~5D/UI/动效/图片视频) + 引用上下文; 中 描述 prompt
//      + 流水线(建模→校验→推演→构建→验证→导出); 右 交互预览(UI/2D/3D/4D/5D/1D/动效);
//      底栏 缩放/适配/图层/设备/数据来源/版本 + 附加到任务卡
// =====================================================================
"use client";

import { useCallback, useEffect, useState } from "react";
import {
  Layout,
  Network,
  Box,
  Timer,
  FlaskConical,
  FileText,
  Wand2,
  Image as ImageIcon,
  Paperclip,
  Play,
  Share2,
  Download,
  Check,
  Minus,
  Plus,
  Smartphone,
  Monitor,
  Layers,
  Palette,
  Settings2,
  GitBranch,
  Loader2,
  XCircle,
} from "lucide-react";
import { clsx } from "clsx";

// =====================================================================
// GemArc MCP 集成 (per INTEGRATION-MAP-GITHUB-REPOS-001 §2.1)
// 流水线 建模→校验→推演→构建→验证→导出 = GemArc tools
//   gemarc_brief(建模) → gemarc_validate(校验) → gemarc_simulate(推演)
//   → gemarc_build(构建) → gemarc_verify(验证)
// 经 /api/gemarc 代理调用本地 GemArc Streamable HTTP (E:\\GemArc)
// =====================================================================
interface GemArcStep {
  id: string;
  label: string;
  tool: string;
}

const GEMARC_PIPELINE: GemArcStep[] = [
  { id: "brief", label: "建模", tool: "gemarc_brief" },
  { id: "validate", label: "校验", tool: "gemarc_validate" },
  { id: "simulate", label: "推演", tool: "gemarc_simulate" },
  { id: "build", label: "构建", tool: "gemarc_build" },
  { id: "verify", label: "验证", tool: "gemarc_verify" },
  { id: "export", label: "导出", tool: "gemarc_render" },
];

async function callGemArc(tool: string, args: Record<string, unknown>, source: "local" | "remote") {
  const res = await fetch("/api/gemarc", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: Date.now(), method: "tools/call", params: { name: tool, arguments: args, _meta: { source: source === "remote" ? "remote" : "local" } } }),
  });
  if (!res.ok) throw new Error(`gemarc ${tool}: HTTP ${res.status}`);
  const payload = await res.json();
  if (payload?.error) throw new Error(`gemarc ${tool}: ${payload.error.message ?? JSON.stringify(payload.error)}`);
  const content = payload?.result?.content;
  if (Array.isArray(content)) {
    return content.map((c: { text?: string }) => c.text ?? "").join("\n");
  }
  return JSON.stringify(payload?.result ?? payload);
}

const GEN_TYPES = [
  { id: "ui", icon: Layout, label: "UI 界面", sub: "可点击的多页面" },
  { id: "2d", icon: Network, label: "2D 关系", sub: "节点与连线" },
  { id: "3d", icon: Box, label: "3D 空间", sub: "建筑 · 楼层 · 房间" },
  { id: "4d", icon: Timer, label: "4D 时间", sub: "事件随时间传播" },
  { id: "5d", icon: FlaskConical, label: "5D 推演", sub: "屏蔽模块做反事实" },
  { id: "1d", icon: FileText, label: "1D 文本", sub: "结构化说明" },
  { id: "motion", icon: Wand2, label: "动效", sub: "过渡与微交互" },
  { id: "media", icon: ImageIcon, label: "图片与视频", sub: "生成配图与演示片" },
];

const PIPELINE = ["建模", "校验", "推演", "构建", "验证", "导出"] as const;

export default function PrototypePage() {
  const [genType, setGenType] = useState("ui");
  const [device, setDevice] = useState<"mobile" | "desktop">("mobile");
  const [dataSource, setDataSource] = useState<"sample" | "branch">("sample");
  const [prompt, setPrompt] = useState("做一个手机端的关系查询界面，能切换出边、入边、环路径");
  const [gemarcStatus, setGemarcStatus] = useState<"checking" | "up" | "down">("checking");
  const [runningStep, setRunningStep] = useState<string | null>(null);
  const [pipelineLog, setPipelineLog] = useState<string>("");

  // GemArc 健康检查 (本地 full + 远端 stateless 双通道)
  useEffect(() => {
    fetch("/api/gemarc", { method: "GET" })
      .then((r) => r.json())
      .then((d: { local?: { status?: string }; remote?: { status?: string } }) => {
        const localUp = d?.local?.status === "ok";
        const remoteUp = d?.remote?.status === "ok";
        if (localUp && dataSource === "branch") setGemarcStatus("up");
        else if (remoteUp && dataSource === "sample") setGemarcStatus("up");
        else if (localUp || remoteUp) setGemarcStatus("up");
        else setGemarcStatus("down");
      })
      .catch(() => setGemarcStatus("down"));
  }, [dataSource]);

  // 真实流水线执行 (建模→校验→推演→构建→验证)
  const runPipeline = useCallback(async () => {
    setPipelineLog("");
    for (const step of GEMARC_PIPELINE) {
      setRunningStep(step.id);
      try {
        const out = await callGemArc(
          step.tool,
          { project: "prototype-demo", query: prompt, note: `device=${device}` },
          dataSource === "sample" ? "remote" : "local",
        );
        setPipelineLog((prev) => `${prev}\n[${step.label}] ${out.slice(0, 300)}`);
      } catch (err) {
        setPipelineLog((prev) => `${prev}\n[${step.label}] 失败: ${String(err)}`);
        setRunningStep(null);
        return;
      }
    }
    setRunningStep(null);
  }, [prompt, device, dataSource]);

  return (
    <div className="flex flex-col h-full bg-bg">
      <div className="flex-1 flex min-h-0">
        {/* ============ 左: 生成类型 + 引用 ============ */}
        <aside data-testid="prototype-gen-panel" className="w-64 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-3">
          <h2 className="text-xs font-bold text-ink mb-2">要生成什么</h2>
          <ul className="grid grid-cols-2 gap-1.5">
            {GEN_TYPES.map((t) => (
              <li key={t.id}>
                <button
                  type="button"
                  onClick={() => setGenType(t.id)}
                  data-testid={`prototype-gen-${t.id}`}
                  className={clsx(
                    "w-full text-left px-2.5 py-2 rounded-lg border transition-colors",
                    genType === t.id ? "border-accent bg-accent/10" : "border-line hover:border-line",
                  )}
                >
                  <span className={clsx("flex items-center gap-1.5 text-xs font-semibold", genType === t.id ? "text-accent" : "text-ink-dim")}>
                    <t.icon size={12} /> {t.label}
                  </span>
                  <span className="block text-[10px] text-ink-mute mt-0.5">{t.sub}</span>
                </button>
              </li>
            ))}
          </ul>

          <div className="mt-4">
            <h3 className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1.5">引用上下文</h3>
            <ul className="space-y-1">
              <li className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-line text-xs">
                <Paperclip size={11} className="text-ink-mute" />
                <span className="font-mono text-ink-dim flex-1 truncate">T-142 relation-api.md</span>
              </li>
              <li className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-line text-xs">
                <Paperclip size={11} className="text-ink-mute" />
                <span className="font-mono text-ink-dim flex-1 truncate">ipa-s03-005 画面設計</span>
              </li>
            </ul>
            <button type="button" className="mt-1.5 w-full text-xs py-1.5 rounded-lg border border-dashed border-line text-ink-dim hover:border-accent/60 flex items-center justify-center gap-1">
              + 文件 / 截图 / 任务卡
            </button>
          </div>
        </aside>

        {/* ============ 中: prompt + 流水线 ============ */}
        <main className="flex-1 min-w-0 overflow-y-auto p-4">
          <div className="max-w-2xl space-y-4">
            <textarea
              data-testid="prototype-prompt"
              rows={4}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              className="w-full rounded-xl border border-line bg-bg-card p-3.5 text-sm text-ink focus:border-accent focus:outline-none shadow-soft"
            />
            <p className="text-xs text-ink-dim">
              已生成 3 个页签的可点击原型，数据来自{dataSource === "sample" ? "示例" : "当前 Branch 的接口"}。
              <span className="font-mono">建模→校验→推演→构建→验证→导出 = GemArc MCP tools</span>
            </p>
            {pipelineLog && (
              <pre data-testid="prototype-pipeline-log" className="mt-2 max-h-40 overflow-y-auto rounded-lg border border-line bg-bg-soft p-2.5 text-[10px] font-mono text-ink-dim whitespace-pre-wrap">
                {pipelineLog}
              </pre>
            )}
            <div className="flex items-center gap-2">
              <span className="text-xs text-ink-dim flex-1">描述你要的原型</span>
              <button
                type="button"
                onClick={() => void runPipeline()}
                disabled={runningStep !== null || gemarcStatus !== "up"}
                data-testid="prototype-generate"
                className="flex items-center gap-1.5 text-xs px-4 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft font-semibold disabled:opacity-50"
              >
                {runningStep !== null ? <Loader2 size={12} className="animate-spin" /> : <Play size={12} />}
                生成 (GemArc 流水线)
              </button>
              <button type="button" className="text-xs px-3 py-2 rounded-lg border border-line text-ink-dim hover:border-accent/60">
                从模板
              </button>
            </div>

            {/* 流水线 */}
            <div data-testid="prototype-pipeline" className="flex items-center gap-1.5 text-xs flex-wrap">
              {PIPELINE.map((step, i) => (
                <span key={step} className="flex items-center gap-1.5">
                  <span className={clsx("px-2.5 py-1.5 rounded-lg border", i <= 3 ? "bg-accent text-white border-accent font-semibold" : "bg-bg-card border-line text-ink-dim")}>
                    {step}
                  </span>
                  {i < PIPELINE.length - 1 && <span className="text-ink-mute">→</span>}
                </span>
              ))}
              <span className="ml-2 flex items-center gap-2">
                <button type="button" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg bg-accent text-white font-semibold">
                  <Play size={12} /> 交互预览
                </button>
                <button type="button" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim">
                  <Download size={12} /> 导出
                </button>
                <button type="button" className="flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim">
                  <Share2 size={12} /> 分享
                </button>
              </span>
            </div>
          </div>
        </main>

        {/* ============ 右: 交互预览 ============ */}
        <aside data-testid="prototype-preview" className="w-80 shrink-0 border-l border-line bg-bg-card overflow-y-auto p-3">
          <h2 className="text-xs font-bold text-ink mb-2">UI · 关系查询（手机）</h2>
          <div data-testid="prototype-preview-ui" className={clsx("mx-auto rounded-xl border-2 border-line bg-bg overflow-hidden", device === "mobile" ? "w-[200px]" : "w-full")}>
            <div className="bg-bg-card border-b border-line px-3 py-2 flex items-center justify-between">
              <span className="text-[11px] font-bold text-ink">关系查询</span>
              <span className="text-[9px] text-ink-mute">A → B</span>
            </div>
            <div className="p-2.5 space-y-1.5">
              {[
                { tab: "出边", rows: ["A → B 依赖", "A → D 引用"] },
                { tab: "入边", rows: ["C → A 依赖"] },
                { tab: "环路径", rows: ["查询环路径"] },
              ].map((g) => (
                <div key={g.tab} className="rounded-lg bg-bg-card border border-line p-2">
                  <div className="text-[10px] font-semibold text-accent">{g.tab}</div>
                  {g.rows.map((r) => (
                    <div key={r} className="text-[10px] font-mono text-ink-dim mt-0.5">{r}</div>
                  ))}
                </div>
              ))}
            </div>
          </div>

          <div className="mt-3 space-y-2 text-xs">
            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">2D · 节点关系</div>
              <div className="mt-1.5 flex items-center justify-center gap-3 font-mono text-[11px] text-ink-dim">
                <span className="size-7 rounded-full bg-accent/15 text-accent grid place-items-center">A</span>
                <span className="text-ink-mute">—</span>
                <span className="size-7 rounded-full bg-accent/15 text-accent grid place-items-center">B</span>
                <span className="text-ink-mute">—</span>
                <span className="size-7 rounded-full bg-accent/15 text-accent grid place-items-center">C</span>
              </div>
            </div>

            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">3D · 机房 → 机柜 → 服务</div>
              <div className="mt-1 text-[11px] text-ink-dim">机柜 A2 · canvas-realtime ×3</div>
            </div>

            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">4D · 事件随时间传播</div>
              <div className="mt-1.5 flex items-center gap-1 font-mono text-[10px]">
                {["postgres", "api", "realtime", "告警"].map((n, i) => (
                  <span key={n} className="flex items-center gap-1">
                    <span className="px-1.5 py-0.5 rounded bg-bg-soft text-ink-dim">{n}</span>
                    {i < 3 && <span className="text-ink-mute">→</span>}
                  </span>
                ))}
              </div>
              <div className="mt-1 text-[10px] text-ok">t = 0 · 正常</div>
            </div>

            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">5D · 反事实对照</div>
              <div className="mt-1 space-y-0.5 text-[11px] font-mono">
                <div className="text-ink-dim">屏蔽「队列」模块</div>
                <div className="text-ok">正常 P95 42 ms</div>
                <div className="text-err">反事实 P95 118 ms</div>
              </div>
              <div className="mt-1 text-[10px] text-ink-mute">按显式规则推演，不是一般意义的因果推断</div>
            </div>

            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">1D · 结构化说明</div>
              <ol className="mt-1 space-y-0.5 text-[11px] text-ink-dim list-decimal list-inside">
                <li>输入起点节点</li>
                <li>选择方向与深度</li>
                <li>查看环路径并跳到任务卡</li>
              </ol>
            </div>

            <div className="rounded-lg border border-line p-2.5">
              <div className="font-semibold text-ink">动效 · 合并成功提示</div>
              <div className="mt-1 flex items-center gap-1.5 text-[11px] text-ok">
                <Check size={12} /> 已合并到 dev
              </div>
            </div>
          </div>
        </aside>
      </div>

      {/* ============ 底栏 ============ */}
      <footer data-testid="prototype-footer" className="border-t border-line bg-bg-card px-3 py-2 flex items-center gap-2 text-xs flex-wrap">
        <span className="flex items-center gap-1 text-ink-dim">
          <Minus size={12} /> 80% <Plus size={12} />
        </span>
        <span className="flex items-center gap-1 text-ink-dim"><Settings2 size={12} /> 适配</span>
        <span className="flex items-center gap-1 text-ink-dim"><Layers size={12} /> 图层</span>
        <span className="flex items-center gap-1 text-ink-dim">评论</span>

        <span className="mx-1 w-px h-4 bg-bg-soft" />

        <span className="flex items-center gap-1 text-ink-dim"><Palette size={12} /> UI 界面 / 原型</span>
        <span className="flex items-center gap-1 text-ink-dim"><Palette size={12} /> 主色</span>

        <span className="mx-1 w-px h-4 bg-bg-soft" />

        <span className="flex items-center gap-1.5">
          <span className="text-ink-dim">设备</span>
          <button type="button" onClick={() => setDevice("mobile")} className={clsx("flex items-center gap-1 px-2 py-0.5 rounded border", device === "mobile" ? "border-accent text-accent" : "border-line text-ink-dim")}>
            <Smartphone size={11} /> 手机 390 × 844
          </button>
          <button type="button" onClick={() => setDevice("desktop")} className={clsx("flex items-center gap-1 px-2 py-0.5 rounded border", device === "desktop" ? "border-accent text-accent" : "border-line text-ink-dim")}>
            <Monitor size={11} /> 桌面 1440 × 900
          </button>
        </span>

        <span className="mx-1 w-px h-4 bg-bg-soft" />

        <span className="flex items-center gap-1.5">
          <span className="text-ink-dim">数据来源</span>
          <button
            type="button"
            onClick={() => setDataSource("sample")}
            className={clsx("px-2 py-0.5 rounded border", dataSource === "sample" ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim")}
          >
            示例数据（原型用）
          </button>
          <button
            type="button"
            onClick={() => setDataSource("branch")}
            className={clsx("px-2 py-0.5 rounded border flex items-center gap-1", dataSource === "branch" ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim")}
          >
            <GitBranch size={11} /> 当前 Branch 的接口
          </button>
          <span data-testid="prototype-gemarc-status" className={clsx("flex items-center gap-1 text-[10px] font-mono", gemarcStatus === "up" ? "text-ok" : gemarcStatus === "down" ? "text-err" : "text-ink-mute")}>
            {gemarcStatus === "up" ? "● GemArc MCP 在线" : gemarcStatus === "down" ? "○ GemArc MCP 离线" : "… GemArc 检测中"}
          </span>
        </span>

        <span className="mx-1 w-px h-4 bg-bg-soft" />

        <span className="flex items-center gap-1.5">
          <span className="text-ink-dim">版本</span>
          <span className="text-ink-dim font-semibold">刚刚 生成 v3</span>
          <span className="text-ink-mute">5 分钟前 改为手机尺寸</span>
          <span className="text-ink-mute">10 分钟前 首次生成</span>
        </span>

        <div className="flex-1" />

        <a href="/kanban?view=%E6%8C%89%E6%96%87%E4%BB%B6%E5%BC%80%E5%8D%A1" className="text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim hover:border-accent/60">
          附加到任务卡 T-142
        </a>
        <a href="/canvas/canvas-001" data-testid="prototype-to-canvas" className="text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim hover:border-accent/60">
          放进画布白板
        </a>
        <button type="button" className="text-xs px-2.5 py-1.5 rounded-lg border border-line text-ink-dim hover:border-accent/60">
          导出 HTML / 视频
        </button>
      </footer>
    </div>
  );
}
