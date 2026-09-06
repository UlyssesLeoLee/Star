"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Server, AlertTriangle, Heart, MapPin } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import { useEffect, useState } from "react";

export default function LocalRuntimePage() {
  const { t } = useTranslation();
  const runtimes = useStore((s) => s.localRuntimes);
  // Mount-gate for heartbeat Date.now() 实时比较 (per 9/6 19:42 JST hydration 修复):
  //   server render 跟 client first render 之间 Date.now() 差几秒,
  //   跟 last_heartbeat_at 差值可能跨 60_000 边界, 触发 text-warn/text-ok mismatch
  //   mount 后才显示真实状态, 配合 1s ticker 强制 re-render
  const [now, setNow] = useState<number>(0);
  useEffect(() => {
    setNow(Date.now());
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/local-runtime'].title}
        subtitle="开发机上的执行环境。device/tenant/user 三重绑定(§23.2)。policy_violations 触发审计事件。"
        icon={<Server className="text-accent" size={20} />}
        track="E"
        count={runtimes.length}
      />

      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th><th>Hostname</th><th>Status</th><th>Bound user</th>
              <th><MapPin size={10} className="inline" /> Mount root</th>
              <th><Heart size={10} className="inline" /> Heartbeat</th>
              <th><AlertTriangle size={10} className="inline" /> Violations</th>
            </tr>
          </thead>
          <tbody>
            {runtimes.map((r) => (
              <tr key={r.id}>
                <td className="font-mono text-xs">{r.id}</td>
                <td className="font-medium">{r.hostname}</td>
                <td><StatusPill value={r.status} /></td>
                <td className="font-mono text-xs">{r.bound_user_id}</td>
                <td className="font-mono text-xs text-ink-dim">{r.mount_root}</td>
                <td className="text-xs">
                  <span className={now > 0 && now - new Date(r.last_heartbeat_at).getTime() > 60_000 ? "text-warn" : "text-ok"}>
                    {new Date(r.last_heartbeat_at).toLocaleTimeString()}
                  </span>
                </td>
                <td>
                  {r.policy_violations > 0 ? (
                    <span className="text-err font-mono text-xs">{r.policy_violations}</span>
                  ) : (
                    <span className="text-ok font-mono text-xs">0</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="card mt-3 bg-bg-soft/40">
        <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2">三重绑定检查 (INV-LR-01~05)</div>
        <ul className="text-xs space-y-1 text-ink-dim">
          <li>✓ <span className="font-mono">device_id</span> 与设备指纹 (TPM/Secure Enclave) 绑定</li>
          <li>✓ <span className="font-mono">tenant_id</span> 与登录用户 tenant 一致</li>
          <li>✓ <span className="font-mono">user_id</span> 与设备登录态一致</li>
          <li>✓ <span className="font-mono">mount_root</span> 在 policy.allowlist 内</li>
          <li>⚠ 任何 mismatch 触发 status=compromised + audit.policy_violation</li>
        </ul>
      </div>
    </div>
  );
}
