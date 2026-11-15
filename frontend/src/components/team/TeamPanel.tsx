/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/team/TeamPanel.tsx"})
@endcypher
*/
"use client";

// =====================================================================
// TeamPanel — group 页右栏 Team tab 内容 (运行时与 Agent 小队管理)
// =====================================================================
// 职责:
//   1. Runtime / Daemon 管理区: 数据来自 store 内现有 daemon 相关集合
//      store.localRuntimes (seed 预览), 每行 = 名称 / 状态徽标 / 租约到期 /
//      操作按钮 启动·停止·重启
//   2. Worker / Router 小队区: store 无小队集合, 渲染本地 TEAM_SQUAD_PREVIEW
//      常量 (预览数据, 宿主 daemon 投影接入后替换)
//   3. 诚实标注 (per 缺标比错标): 控制通道未接入时全部动作按钮 disabled
//      + title 原因 + 行尾 "未接入" 徽标; 不生成演示执行状态冒充真实记录
// =====================================================================

import Link from "next/link";
import {
  ArrowUpRight,
  Bot,
  Cpu,
  Play,
  RotateCw,
  Square,
  UserMinus,
  UserPlus,
  type LucideIcon,
} from "lucide-react";
import { useStore } from "@/lib/store";
import type { LocalRuntime, RuntimeStatus } from "@/types/ids";

type RuntimeAction = "start" | "stop" | "restart";
type SquadAction = "assign" | "recall";

interface TeamSquadRow {
  id: string;
  role: "worker" | "router";
  name: string;
  currentTask: string;
}

// 预览数据, 宿主 daemon 投影接入后替换 — store 尚无 Worker/Router 小队集合;
// 宿主小队目录接入后改为从 store 读取并删除本常量。
const TEAM_SQUAD_PREVIEW: TeamSquadRow[] = [
  { id: "sq-001", role: "worker", name: "Worker · 编码执行小队", currentTask: "上下文编译 · wt-001" },
  { id: "sq-002", role: "worker", name: "Worker · 验证小队", currentTask: "空闲 (预览)" },
  { id: "sq-003", role: "router", name: "Router · 任务路由小队", currentTask: "路由规划 · prj-stargate" },
];

const RUNTIME_ACTIONS: Array<{ action: RuntimeAction; label: string; icon: LucideIcon }> = [
  { action: "start", label: "启动", icon: Play },
  { action: "stop", label: "停止", icon: Square },
  { action: "restart", label: "重启", icon: RotateCw },
];

const SQUAD_ACTIONS: Array<{ action: SquadAction; label: string; icon: LucideIcon }> = [
  { action: "assign", label: "分配", icon: UserPlus },
  { action: "recall", label: "回收", icon: UserMinus },
];

const RUNTIME_STATUS_META: Record<RuntimeStatus, { label: string; cls: string }> = {
  online: { label: "在线", cls: "border-ok/40 text-ok bg-ok/10" },
  offline: { label: "离线", cls: "border-line text-ink-dim bg-bg-soft" },
  registered: { label: "已注册", cls: "border-info/40 text-info bg-info/10" },
  compromised: { label: "已入侵", cls: "border-err/40 text-err bg-err/10" },
  revoked: { label: "已吊销", cls: "border-line text-ink-dim bg-bg-soft" },
};

// 动作按钮未接入原因 (宿主控制通道就绪后接线真实命令并解除禁用)
const RUNTIME_ACTION_REASON = "宿主 daemon 控制通道未接入";
const SQUAD_ACTION_REASON = "宿主小队调度通道未接入";

export function TeamPanel() {
  // store 内现有 daemon 相关集合 = localRuntimes (seed 预览数据)
  const runtimes = useStore((s) => s.localRuntimes);

  return (
    <div data-testid="team-panel" className="space-y-3">
      <div className="flex items-start justify-between gap-2">
        <div>
          <div className="mb-1 text-[9px] font-semibold uppercase tracking-[0.18em] text-ink-mute">TEAM</div>
          <h2 className="text-sm font-semibold">运行时与 Agent 小队</h2>
          <p className="mt-1 text-[9px] leading-relaxed text-ink-mute">
            Runtime / Daemon 与 Worker / Router 小队统一在团队管理；控制通道未接入前只读展示。
          </p>
        </div>
        <span
          className="shrink-0 rounded border border-warning/30 px-1.5 py-0.5 text-[8px] text-warning"
          title="当前展示本地 seed 预览数据, 非授权服务端运行时事实; 宿主 daemon 投影接入后替换"
        >
          预览数据
        </span>
      </div>

      <section aria-label="Runtime / Daemon 管理">
        <div className="mb-1.5 flex items-center gap-2 text-[10px] font-semibold">
          <Cpu size={13} className="text-accent" />
          Runtime / Daemon
          <span className="font-normal text-ink-mute">store.localRuntimes · seed 预览</span>
        </div>
        <div className="space-y-1.5">
          {runtimes.length === 0 && (
            <p className="rounded-lg border border-dashed border-line p-3 text-center text-[9px] text-ink-mute">
              暂无 Runtime 记录
            </p>
          )}
          {runtimes.map((runtime: LocalRuntime) => {
            const status = RUNTIME_STATUS_META[runtime.status];
            return (
              <div
                key={runtime.id}
                data-testid={`team-runtime-row-${runtime.id}`}
                className="rounded-lg border border-line bg-bg-soft/40 p-2"
              >
                <div className="flex items-center gap-2">
                  <span
                    className="min-w-0 flex-1 truncate text-[10px] font-medium"
                    title={`${runtime.hostname} · ${runtime.id}`}
                  >
                    {runtime.hostname}
                  </span>
                  <span className={`shrink-0 rounded border px-1.5 py-0.5 text-[8px] ${status.cls}`}>
                    {status.label}
                  </span>
                  <span className="shrink-0 rounded border border-warning/30 px-1.5 py-0.5 text-[8px] text-warning">
                    未接入
                  </span>
                </div>
                <div className="mt-1.5 flex items-center gap-2">
                  <span
                    className="min-w-0 truncate font-mono text-[8px] text-ink-mute"
                    title="租约到期信息未由宿主投影提供"
                  >
                    租约到期 <span className="text-ink-dim">—</span>
                  </span>
                  <span className="flex-1" />
                  <div className="flex shrink-0 items-center gap-1">
                    {RUNTIME_ACTIONS.map(({ action, label, icon: Icon }) => (
                      <button
                        key={action}
                        type="button"
                        disabled
                        aria-disabled="true"
                        title={`${label}未接入：${RUNTIME_ACTION_REASON}`}
                        data-testid={`team-action-${action}-${runtime.id}`}
                        className="inline-flex cursor-not-allowed items-center gap-1 rounded border border-line px-1.5 py-0.5 text-[9px] text-ink-dim opacity-60"
                      >
                        <Icon size={10} />
                        {label}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </section>

      <section aria-label="Worker / Router 小队">
        <div className="mb-1.5 flex items-center gap-2 text-[10px] font-semibold">
          <Bot size={13} className="text-accent" />
          Worker / Router 小队
          <span className="font-normal text-ink-mute">本地预览常量 · 宿主投影接入后替换</span>
        </div>
        <div className="space-y-1.5">
          {TEAM_SQUAD_PREVIEW.map((squad) => (
            <div
              key={squad.id}
              data-testid={`team-squad-row-${squad.id}`}
              className="rounded-lg border border-line bg-bg-soft/40 p-2"
            >
              <div className="flex items-center gap-2">
                <span
                  className={`shrink-0 rounded px-1 py-0.5 text-[8px] font-semibold ${
                    squad.role === "worker"
                      ? "border-info/40 text-info bg-info/10"
                      : "border-warn/40 text-warn bg-warn/10"
                  }`}
                >
                  {squad.role === "worker" ? "Worker" : "Router"}
                </span>
                <span className="min-w-0 flex-1 truncate text-[10px] font-medium">{squad.name}</span>
                <span className="shrink-0 rounded border border-warning/30 px-1.5 py-0.5 text-[8px] text-warning">
                  未接入
                </span>
              </div>
              <div className="mt-1.5 flex items-center gap-2">
                <span className="min-w-0 flex-1 truncate text-[9px] text-ink-mute">
                  当前任务 <span className="text-ink-dim">{squad.currentTask}</span>
                </span>
                <div className="flex shrink-0 items-center gap-1">
                  {SQUAD_ACTIONS.map(({ action, label, icon: Icon }) => (
                    <button
                      key={action}
                      type="button"
                      disabled
                      aria-disabled="true"
                      title={`${label}未接入：${SQUAD_ACTION_REASON}`}
                      data-testid={`team-action-${action}-${squad.id}`}
                      className="inline-flex cursor-not-allowed items-center gap-1 rounded border border-line px-1.5 py-0.5 text-[9px] text-ink-dim opacity-60"
                    >
                      <Icon size={10} />
                      {label}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      <p className="rounded border border-dashed border-line px-2 py-1.5 text-[8px] leading-relaxed text-ink-mute">
        控制通道未接入：启动 / 停止 / 重启 / 分配 / 回收按钮均不可用；接入前不生成演示执行状态，不冒充真实记录。
      </p>

      <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <Link href="/local-runtime" className="inline-flex items-center gap-1 text-[9px] text-accent">
          管理 Runtime <ArrowUpRight size={10} />
        </Link>
        <Link href="/agent-relationships" className="inline-flex items-center gap-1 text-[9px] text-accent">
          管理小队关系 <ArrowUpRight size={10} />
        </Link>
        <Link href="/agents" className="inline-flex items-center gap-1 text-[9px] text-accent">
          Agent 会话 <ArrowUpRight size={10} />
        </Link>
        <Link href="/remote" className="inline-flex items-center gap-1 text-[9px] text-accent">
          远程协作 <ArrowUpRight size={10} />
        </Link>
      </div>
    </div>
  );
}
