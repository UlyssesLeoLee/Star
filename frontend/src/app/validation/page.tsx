"use client";

import { useStore } from "@/lib/store";
import { PageHeader, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { ShieldCheck } from "lucide-react";

export default function ValidationPage() {
  const cases = useStore((s) => s.validationCases);

  const pass = cases.filter((c) => c.result === "pass").length;
  const fail = cases.filter((c) => c.result === "fail").length;
  const fb = cases.filter((c) => c.result === "feedback_required").length;
  const skipped = cases.filter((c) => c.result === "skipped").length;
  const avgCoverage = cases.reduce((s, c) => s + c.coverage, 0) / cases.length;

  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Validation"
        subtitle="7 实体 + 5 状态机(pass/fail/skipped/feedback_required/pending) + AcceptanceCoveragePort。"
        icon={<ShieldCheck className="text-accent" size={20} />}
        track="B"
        count={cases.length}
      />

      <div className="grid grid-cols-2 md:grid-cols-5 gap-3 mb-5">
        <Stat label="Pass" value={pass} tone="ok" />
        <Stat label="Fail" value={fail} tone="err" />
        <Stat label="Feedback required" value={fb} tone="warn" hint="triggers feedback question" />
        <Stat label="Skipped" value={skipped} tone="default" />
        <Stat label="Avg coverage" value={`${(avgCoverage * 100).toFixed(0)}%`} tone="info" />
      </div>

      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th><th>Name</th><th>Kind</th><th>Result</th>
              <th>Coverage</th><th>Work-item</th><th>Changeset</th><th>When</th>
            </tr>
          </thead>
          <tbody>
            {cases.map((c) => (
              <tr key={c.id}>
                <td className="font-mono text-xs">{c.id}</td>
                <td className="font-medium">{c.name}</td>
                <td><StatusPill value={c.kind} size="xs" /></td>
                <td><StatusPill value={c.result} size="xs" /></td>
                <td>
                  <div className="flex items-center gap-2">
                    <div className="h-1.5 w-16 rounded bg-bg-soft overflow-hidden">
                      <div
                        className={`h-full ${c.coverage > 0.9 ? "bg-ok" : c.coverage > 0.7 ? "bg-warn" : "bg-err"}`}
                        style={{ width: `${c.coverage * 100}%` }}
                      />
                    </div>
                    <span className="font-mono text-xs text-ink-dim">{(c.coverage * 100).toFixed(0)}%</span>
                  </div>
                </td>
                <td className="font-mono text-xs text-info">{c.work_item_id ?? "—"}</td>
                <td className="font-mono text-xs">{c.changeset_id ?? "—"}</td>
                <td className="text-ink-dim text-xs">{new Date(c.executed_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
