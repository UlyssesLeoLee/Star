"use client";

/**
 * ClusterTab — F-01 集群更新 端到端 UI 组件
 * (per docs/requirements/SRS-STAR-OPS-001.md v0.1 §4 F-01
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md v0.1 §3.1)
 *
 * 4 卡片 (跟 brief §2.1 wt7 一致):
 *   1. 列出 release (cluster_list GET)
 *   2. 触发灰度 (cluster_canary POST, canary_weight 滑块 0-100)
 *   3. 回滚 (cluster_rollback POST, target_revision 输入)
 *   4. 状态 (cluster_status GET, 实时)
 *
 * 守門 #5 v2: cluster_* 1MB body 限制 (ops_api.rs 整体应用 RequestBodyLimitLayer)
 * 守門 #6 v2: 1 pre-existing TS err (agent-view/page.tsx) 不在 F-01 scope
 * 守門 #1 R-05: 真实 K8s 切换 owner 拍板, MVP 永远 mock 路径
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "@/lib/i18n";
import { Server, GitBranch, RotateCcw, Activity, AlertCircle, CheckCircle2, Loader2 } from "lucide-react";

interface HelmRelease {
  name: string;
  namespace: string;
  chart: string;
  revision: number;
  status: string;
  last_deployed_at: string;
  canary_weight: number;
}

interface ClusterStatus {
  release_name: string;
  phase: string;
  revision: number;
  canary_weight: number;
  mock: boolean;
}

interface HelmActionAck {
  action_id: string;
  status: string;
  detail: string;
}

const OPS_API_URL = process.env.NEXT_PUBLIC_OPS_URL || "http://localhost:8090";

export function ClusterTab() {
  const { t } = useTranslation();
  const [releases, setReleases] = useState<HelmRelease[]>([]);
  const [status, setStatus] = useState<ClusterStatus | null>(null);
  const [canaryWeight, setCanaryWeight] = useState(10);
  const [rollbackRevision, setRollbackRevision] = useState(2);
  const [loading, setLoading] = useState<{ list: boolean; canary: boolean; rollback: boolean; status: boolean }>({
    list: false, canary: false, rollback: false, status: false,
  });
  const [error, setError] = useState<string | null>(null);
  const [lastAction, setLastAction] = useState<HelmActionAck | null>(null);

  const fetchReleases = useCallback(async () => {
    setLoading((l) => ({ ...l, list: true }));
    setError(null);
    try {
      const res = await fetch(`${OPS_API_URL}/api/ops/cluster/releases`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setReleases(data.data || []);
    } catch (e) {
      setError(`List failed: ${(e as Error).message}`);
    } finally {
      setLoading((l) => ({ ...l, list: false }));
    }
  }, []);

  const fetchStatus = useCallback(async () => {
    setLoading((l) => ({ ...l, status: true }));
    try {
      const res = await fetch(`${OPS_API_URL}/api/ops/cluster/status`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setStatus(data.data || null);
    } catch {
      // 静默失败 (status 卡片降级)
    } finally {
      setLoading((l) => ({ ...l, status: false }));
    }
  }, []);

  const triggerCanary = useCallback(async () => {
    setLoading((l) => ({ ...l, canary: true }));
    setError(null);
    try {
      const res = await fetch(`${OPS_API_URL}/api/ops/cluster/canary`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          release_name: "star-mcp",
          canary_weight: canaryWeight,
          target_revision: null,
        }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setLastAction(data.data);
    } catch (e) {
      setError(`Canary failed: ${(e as Error).message}`);
    } finally {
      setLoading((l) => ({ ...l, canary: false }));
    }
  }, [canaryWeight]);

  const triggerRollback = useCallback(async () => {
    setLoading((l) => ({ ...l, rollback: true }));
    setError(null);
    try {
      const res = await fetch(`${OPS_API_URL}/api/ops/cluster/rollback`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          release_name: "star-mcp",
          target_revision: rollbackRevision,
        }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setLastAction(data.data);
    } catch (e) {
      setError(`Rollback failed: ${(e as Error).message}`);
    } finally {
      setLoading((l) => ({ ...l, rollback: false }));
    }
  }, [rollbackRevision]);

  useEffect(() => {
    fetchReleases();
    fetchStatus();
  }, [fetchReleases, fetchStatus]);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      {/* 1. 列出 release 卡片 */}
      <Card
        title={t.opsConsole.clusterReleases}
        icon={<GitBranch className="w-4 h-4" />}
        loading={loading.list}
      >
        {releases.length === 0 ? (
          <p className="text-xs text-ink-mute">No releases</p>
        ) : (
          <ul className="text-xs space-y-1">
            {releases.map((r) => (
              <li key={r.name} className="flex justify-between">
                <span className="text-ink">{r.name}</span>
                <span className="text-ink-mute font-mono">rev {r.revision} · {r.status}</span>
              </li>
            ))}
          </ul>
        )}
        <button onClick={fetchReleases} className="mt-2 text-[10px] text-accent hover:underline">
          Refresh
        </button>
      </Card>

      {/* 2. 触发灰度 卡片 */}
      <Card
        title={t.opsConsole.clusterCanary}
        icon={<Server className="w-4 h-4" />}
        loading={loading.canary}
      >
        <label className="text-xs text-ink-mute">Canary weight: {canaryWeight}%</label>
        <input
          type="range"
          min={0}
          max={100}
          value={canaryWeight}
          onChange={(e) => setCanaryWeight(Number(e.target.value))}
          className="w-full mt-2"
          data-testid="cluster-canary-slider"
        />
        <button
          onClick={triggerCanary}
          disabled={loading.canary}
          className="mt-2 px-3 py-1 text-xs rounded border border-line bg-bg-soft hover:bg-accent/10"
        >
          {loading.canary ? "Triggering..." : "Trigger Canary"}
        </button>
      </Card>

      {/* 3. 回滚 卡片 */}
      <Card
        title={t.opsConsole.clusterRollback}
        icon={<RotateCcw className="w-4 h-4" />}
        loading={loading.rollback}
      >
        <label className="text-xs text-ink-mute">Target revision</label>
        <input
          type="number"
          min={1}
          value={rollbackRevision}
          onChange={(e) => setRollbackRevision(Number(e.target.value))}
          className="w-full mt-1 px-2 py-1 text-xs rounded border border-line bg-bg"
          data-testid="cluster-rollback-input"
        />
        <button
          onClick={triggerRollback}
          disabled={loading.rollback}
          className="mt-2 px-3 py-1 text-xs rounded border border-line bg-bg-soft hover:bg-accent/10"
        >
          {loading.rollback ? "Rolling back..." : "Rollback"}
        </button>
      </Card>

      {/* 4. 状态 卡片 */}
      <Card
        title={t.opsConsole.clusterStatus}
        icon={<Activity className="w-4 h-4" />}
        loading={loading.status}
      >
        {status ? (
          <div className="text-xs space-y-1">
            <div className="flex justify-between">
              <span className="text-ink">{status.release_name}</span>
              <span className={`font-mono ${status.phase === "healthy" ? "text-ok" : "text-warn"}`}>
                {status.phase}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-ink-mute">Revision</span>
              <span className="font-mono">{status.revision}</span>
            </div>
            {status.canary_weight > 0 && (
              <div className="flex justify-between">
                <span className="text-ink-mute">Canary</span>
                <span className="font-mono">{status.canary_weight}%</span>
              </div>
            )}
            {status.mock && (
              <div className="text-[10px] text-ink-mute italic">mock 路径 (守門 #1 R-05)</div>
            )}
          </div>
        ) : (
          <p className="text-xs text-ink-mute">No status</p>
        )}
        <button onClick={fetchStatus} className="mt-2 text-[10px] text-accent hover:underline">
          Refresh
        </button>
      </Card>

      {/* Error + Last Action 显示 */}
      {error && (
        <div className="md:col-span-2 flex items-center gap-2 px-3 py-2 rounded border border-err/40 bg-err/5 text-xs text-err">
          <AlertCircle className="w-3 h-3" />
          {error}
        </div>
      )}
      {lastAction && (
        <div className="md:col-span-2 flex items-center gap-2 px-3 py-2 rounded border border-ok/40 bg-ok/5 text-xs text-ok">
          <CheckCircle2 className="w-3 h-3" />
          Last action: {lastAction.action_id} ({lastAction.status})
        </div>
      )}
    </div>
  );
}

function Card({
  title,
  icon,
  loading,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  loading: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className="anime-panel anime-chamfer p-4">
      <div className="flex items-center gap-2 mb-2">
        <span className="text-accent">{icon}</span>
        <h3 className="text-sm font-semibold text-ink">{title}</h3>
        {loading && <Loader2 className="w-3 h-3 animate-spin text-ink-mute ml-auto" />}
      </div>
      {children}
    </div>
  );
}
