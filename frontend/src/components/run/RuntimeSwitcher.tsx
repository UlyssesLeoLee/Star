/*
@cypher schema=1 source_sha256=placeholder
@endcypher
*/
// RuntimeSwitcher — Lane 运行时切换器 (2026-11-27)
// 允许将 Lane 的运行时切换到另一台电脑
// 减少内存和硬盘占用，优化多机开发协作
"use client";

import { clsx } from "clsx";
import {
  Monitor,
  Server,
  Computer,
  Wifi,
  WifiOff,
  Check,
  Loader2,
  X,
  ArrowRight,
  AlertTriangle,
} from "lucide-react";
import { useState, useCallback, useEffect } from "react";

export interface RuntimeTarget {
  id: string;
  name: string;
  host: string;
  status: "online" | "offline" | "busy";
  cpu: string;
  memory: string;
  currentLanes: number;
  maxLanes: number;
}

export interface RuntimeSwitcherProps {
  projectId: string;
  laneId: string;
  laneName: string;
  currentRuntime?: RuntimeTarget;
  onSwitch: (targetRuntimeId: string) => void;
  onClose: () => void;
  className?: string;
}

export function RuntimeSwitcher({
  projectId,
  laneId,
  laneName,
  currentRuntime,
  onSwitch,
  onClose,
  className,
}: RuntimeSwitcherProps) {
  const [runtimes, setRuntimes] = useState<RuntimeTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTarget, setSelectedTarget] = useState<string | null>(null);
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmSwitch, setConfirmSwitch] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);

  // 获取可用运行时会话列表
  useEffect(() => {
    setLoading(true);
    setFetchError(null);
    fetch("/api/runtimes/available", {
      headers: { "Content-Type": "application/json" },
    })
      .then((r) => r.json())
      .then((data) => setRuntimes(data.runtimes || []))
      .catch((e) => setFetchError(e instanceof Error ? e.message : "获取运行时失败"))
      .finally(() => setLoading(false));
  }, []);

  const handleSwitch = useCallback(async () => {
    if (!selectedTarget) return;
    setSwitching(true);
    setError(null);
    try {
      const res = await fetch("/api/runtimes/switch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          project_id: projectId,
          lane_id: laneId,
          target_runtime_id: selectedTarget,
        }),
      });
      if (!res.ok) {
        const err = await res.json();
        throw new Error(err.message || "切换失败");
      }
      onSwitch(selectedTarget);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "切换失败");
    } finally {
      setSwitching(false);
    }
  }, [selectedTarget, projectId, laneId, onSwitch]);

  return (
    <div className={clsx("space-y-4", className)}>
      {/* 当前运行时信息 */}
      {currentRuntime && (
        <div className="rounded-lg border-2 border-line bg-bg p-3">
          <h4 className="flex items-center gap-1.5 text-[10px] font-mono font-bold text-ink-mute mb-2">
            <Monitor size={12} />
            当前运行时
          </h4>
          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm font-bold">{currentRuntime.name}</span>
              <span className="block text-[10px] font-mono text-ink-mute mt-0.5">
                {currentRuntime.host} · {currentRuntime.cpu} · {currentRuntime.memory}
              </span>
            </div>
            <span className={clsx(
              "flex items-center gap-1 rounded px-2 py-0.5 text-[10px] font-mono font-bold",
              currentRuntime.status === "online" ? "bg-ok/10 text-ok" :
              currentRuntime.status === "busy" ? "bg-warn/10 text-warn" : "bg-ink-mute/10 text-ink-mute",
            )}>
              {currentRuntime.status === "online" ? <Wifi size={10} /> : <WifiOff size={10} />}
              {currentRuntime.status === "online" ? "在线" : currentRuntime.status === "busy" ? "忙碌" : "离线"}
            </span>
          </div>
        </div>
      )}

      {/* 目标运行时选择 */}
      <div className="rounded-lg border-2 border-line bg-bg p-3">
        <h4 className="flex items-center gap-1.5 text-[10px] font-mono font-bold text-ink-mute mb-2">
          <Server size={12} />
          选择目标运行时
        </h4>

        {loading && (
          <div className="flex items-center justify-center py-4">
            <Loader2 size={20} className="animate-spin text-ink-mute" />
            <span className="ml-2 text-xs text-ink-mute">获取运行时会话列表...</span>
          </div>
        )}

        {fetchError && (
          <div className="rounded border border-err/30 bg-err/5 p-2.5 flex items-start gap-2">
            <AlertTriangle size={13} className="text-err shrink-0 mt-0.5" />
            <p className="text-[10px] text-err">{fetchError}</p>
          </div>
        )}

        {!loading && !fetchError && runtimes.length === 0 && (
          <p className="text-[10px] text-ink-mute py-2">
            暂无其他运行时会话。在另一台电脑上启动 Star 后，运行时会话列表会自动出现。
          </p>
        )}

        {!loading && runtimes.length > 0 && (
          <div className="space-y-1.5 max-h-48 overflow-y-auto">
            {runtimes.map((rt) => {
              const isCurrent = currentRuntime?.id === rt.id;
              return (
                <button
                  key={rt.id}
                  type="button"
                  disabled={isCurrent || rt.status === "offline"}
                  onClick={() => setSelectedTarget(rt.id)}
                  className={clsx(
                    "flex w-full items-center rounded border px-2.5 py-1.5 text-left transition-colors",
                    isCurrent && "border-ok/30 bg-ok/5 cursor-not-allowed opacity-60",
                    selectedTarget === rt.id && "border-accent/50 bg-accent/10",
                    rt.status === "offline" && "cursor-not-allowed opacity-50",
                    !isCurrent && rt.status !== "offline" && "border-line hover:border-line/80",
                  )}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <span className={clsx(
                        rt.status === "online" ? "text-ok" :
                        rt.status === "busy" ? "text-warn" : "text-ink-mute",
                      )}>
                        {rt.status === "online" ? <Wifi size={10} /> :
                         rt.status === "busy" ? <Wifi size={10} /> : <WifiOff size={10} />}
                      </span>
                      <span className="text-xs font-bold truncate">{rt.name}</span>
                    </div>
                    <span className="block text-[10px] font-mono text-ink-mute mt-0.5 truncate">
                      {rt.host} · {rt.cpu} · {rt.memory}
                    </span>
                    <span className="block text-[9px] font-mono text-ink-mute/70 mt-0.5">
                      {rt.currentLanes}/{rt.maxLanes} Lane
                    </span>
                  </div>
                  {isCurrent && <Check size={13} className="shrink-0 text-ok ml-1.5" />}
                  {selectedTarget === rt.id && !isCurrent && (
                    <Check size={13} className="shrink-0 text-accent ml-1.5" />
                  )}
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* 切换确认与错误 */}
      {error && (
        <div className="rounded border border-err/30 bg-err/5 p-2.5 flex items-start gap-2">
          <AlertTriangle size={13} className="text-err shrink-0 mt-0.5" />
          <p className="text-[10px] text-err">{error}</p>
        </div>
      )}

      {selectedTarget && !confirmSwitch && (
        <button
          type="button"
          onClick={() => setConfirmSwitch(true)}
          className="flex w-full items-center justify-center gap-2 rounded border-2 border-warn/50 bg-warn/10 px-3 py-2 text-sm font-bold text-warn hover:bg-warn/20 transition-colors"
        >
          <ArrowRight size={14} />
          将 Lane「{laneName}」的运行时切换到选中电脑
        </button>
      )}

      {confirmSwitch && (
        <div className="rounded-lg border-2 border-warn/50 bg-warn/5 p-3 space-y-2">
          <p className="text-xs text-warn font-bold flex items-center gap-1.5">
            <AlertTriangle size={14} />
            确认运行时切换
          </p>
          <p className="text-[10px] text-ink-dim leading-relaxed">
            这会将 Lane「{laneName}」的所有 Worktree 和运行时上下文转移到目标电脑。
            当前电脑的 Worktree 和 branch 状态会保留，但不再占用运行时资源。
            这个操作可以**减少内存和硬盘利用**，优化整体资源使用。
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => { setConfirmSwitch(false); setSelectedTarget(null); }}
              className="flex-1 rounded border border-line px-3 py-1.5 text-xs"
              disabled={switching}
            >
              取消
            </button>
            <button
              type="button"
              onClick={handleSwitch}
              disabled={switching}
              className="flex flex-1 items-center justify-center gap-1 rounded border border-err/50 bg-err/10 text-err px-3 py-1.5 text-xs font-bold hover:bg-err/20 disabled:opacity-50"
            >
              {switching ? <Loader2 size={12} className="animate-spin" /> : <Computer size={12} />}
              {switching ? "切换中..." : "确认切换运行时"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}