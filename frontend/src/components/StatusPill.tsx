"use client";

import { clsx } from "clsx";
import { useStatusLabel, type StatusKind } from "@/lib/i18n";

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

const DOT_COLOR: Record<string, string> = {
  active: "bg-ok shadow-[0_0_6px_rgba(16,185,129,0.8)]",
  online: "bg-ok shadow-[0_0_6px_rgba(16,185,129,0.8)]",
  completed: "bg-ok",
  resolved: "bg-ok",
  merged: "bg-ok",
  approved: "bg-ok",
  passing: "bg-ok",
  pass: "bg-ok",
  delivered: "bg-ok",
  in_progress: "bg-info animate-pulse shadow-[0_0_6px_rgba(0,240,255,0.8)]",
  initializing: "bg-info animate-pulse",
  running: "bg-info animate-pulse",
  awaiting_feedback: "bg-warn shadow-[0_0_6px_rgba(245,158,11,0.8)]",
  awaiting_human: "bg-warn shadow-[0_0_6px_rgba(245,158,11,0.8)]",
  paused: "bg-warn",
  conflict: "bg-err shadow-[0_0_6px_rgba(255,51,102,0.8)]",
  blocked: "bg-err shadow-[0_0_6px_rgba(255,51,102,0.8)]",
  failed: "bg-err shadow-[0_0_6px_rgba(255,51,102,0.8)]",
  error: "bg-err shadow-[0_0_6px_rgba(255,51,102,0.8)]",
  circuit_open: "bg-err",
  deny: "bg-err",
  allow: "bg-ok",
};

export interface StatusPillProps {
  value: string;
  size?: "sm" | "xs";
  /**
   * i18n 翻译类别 (per 2026-08-31 v0.3). 不传走原 value 美化 (snake -> "Snake Case"),
   * 传了之后走字典查表 (e.g. workItem -> 待办/进行中, sprint -> Active/Planned).
   */
  translateAs?: StatusKind;
}

export function StatusPill({ value, size = "sm", translateAs }: StatusPillProps) {
  const k = value.toLowerCase();
  const cls = COLOR[k] ?? "border-line text-ink-dim bg-bg-soft";
  const dot = DOT_COLOR[k];
  // 走 i18n 翻译 (useStatusLabel 内部处理 hook 规则, 即使未传 translateAs 也安全)
  const label = useStatusLabel(translateAs ?? ("workItem" as StatusKind), k);
  // 当 translateAs 未传, useStatusLabel 仍会兜底成 prettify, 与原行为一致
  return (
    <span className={clsx(
      "pill font-mono items-center",
      size === "xs" ? "text-[10px] px-1.5 py-0" : "text-xs px-2 py-0.5",
      cls,
    )}>
      {dot && <span className={clsx("size-1.5 rounded-full inline-block mr-1 shrink-0", dot)} />}
      <span>{label}</span>
    </span>
  );
}
