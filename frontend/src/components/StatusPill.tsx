"use client";

import { clsx } from "clsx";

const COLOR: Record<string, string> = {
  // 通用
  active: "border-ok/40 text-ok bg-ok/10",
  online: "border-ok/40 text-ok bg-ok/10",
  completed: "border-ok/40 text-ok bg-ok/10",
  resolved: "border-ok/40 text-ok bg-ok/10",
  merged: "border-ok/40 text-ok bg-ok/10",
  approved: "border-ok/40 text-ok bg-ok/10",
  passing: "border-ok/40 text-ok bg-ok/10",
  pass: "border-ok/40 text-ok bg-ok/10",
  delivered: "border-ok/40 text-ok bg-ok/10",
  read: "border-ok/40 text-ok bg-ok/10",
  enabled: "border-ok/40 text-ok bg-ok/10",
  // 工作中
  in_progress: "border-info/40 text-info bg-info/10",
  todo: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  initializing: "border-info/40 text-info bg-info/10",
  cloning: "border-info/40 text-info bg-info/10",
  syncing: "border-info/40 text-info bg-info/10",
  spawning: "border-info/40 text-info bg-info/10",
  compiling_context: "border-info/40 text-info bg-info/10",
  planning: "border-info/40 text-info bg-info/10",
  executing: "border-info/40 text-info bg-info/10",
  awaiting_feedback: "border-warn/40 text-warn bg-warn/10",
  awaiting_human: "border-warn/40 text-warn bg-warn/10",
  awaiting_tool: "border-warn/40 text-warn bg-warn/10",
  validating: "border-info/40 text-info bg-info/10",
  paused: "border-warn/40 text-warn bg-warn/10",
  dirty: "border-warn/40 text-warn bg-warn/10",
  behind: "border-warn/40 text-warn bg-warn/10",
  diverged: "border-warn/40 text-warn bg-warn/10",
  ci_running: "border-info/40 text-info bg-info/10",
  review_requested: "border-info/40 text-info bg-info/10",
  committing: "border-info/40 text-info bg-info/10",
  pushing: "border-info/40 text-info bg-info/10",
  open: "border-info/40 text-info bg-info/10",
  acknowledged: "border-info/40 text-info bg-info/10",
  draft: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  planned: "border-info/40 text-info bg-info/10",
  pending: "border-warn/40 text-warn bg-warn/10",
  // 阻塞 / 失败
  conflict: "border-err/40 text-err bg-err/10",
  blocked: "border-err/40 text-err bg-err/10",
  ci_failed: "border-err/40 text-err bg-err/10",
  failed: "border-err/40 text-err bg-err/10",
  feedback_required: "border-err/40 text-err bg-err/10",
  review_required: "border-warn/40 text-warn bg-warn/10",
  changes_requested: "border-warn/40 text-warn bg-warn/10",
  suspended: "border-warn/40 text-warn bg-warn/10",
  paused_rt: "border-warn/40 text-warn bg-warn/10",
  circuit_open: "border-err/40 text-err bg-err/10",
  error: "border-err/40 text-err bg-err/10",
  compromised: "border-err/40 text-err bg-err/10",
  // 终态
  closed: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  abandoned: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  archived: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  reverted: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  wontfix: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  cancelled: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  revoked: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  disabled: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  invited: "border-info/40 text-info bg-info/10",
  // Notification
  suppressed: "border-ink-mute/40 text-ink-mute bg-bg-soft",
  skip: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  // 布尔
  allow: "border-ok/40 text-ok bg-ok/10",
  deny:  "border-err/40 text-err bg-err/10",
  no: "border-ink-mute/40 text-ink-dim bg-bg-soft",
  none: "border-ink-mute/40 text-ink-dim bg-bg-soft",
};

export function StatusPill({ value, size = "sm" }: { value: string; size?: "sm" | "xs" }) {
  const k = value.toLowerCase();
  const cls = COLOR[k] ?? "border-line text-ink-dim bg-bg-soft";
  return (
    <span className={clsx(
      "pill font-mono",
      size === "xs" ? "text-[10px] px-1.5 py-0" : "text-xs",
      cls,
    )}>
      {value.replace(/_/g, " ")}
    </span>
  );
}
