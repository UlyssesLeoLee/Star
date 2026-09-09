"use client";

// =====================================================================
// EdgeTypeSelector — 10 类关系 Dropdown (per DD §3.2.2 + brief v0.50 §2.1 A)
// =====================================================================
// Per docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §1.1.4
// Per docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.2.2
//
// 4 核心 + 6 扩展 = 10 variants. 客户端枚举 (per doc 14 §1.1 EdgeTypeSelector
// "无 API 调用, 客户端枚举"). 提供 weight [0.0, 1.0] 数字输入, 默认 0.7.
// =====================================================================

import { useCallback, useState, useId } from "react";
import {
  ALL_RELATIONSHIP_TYPES,
  type RelationshipType,
} from "@/lib/arg/types";

export interface EdgeTypeSelectorProps {
  /** Default selected type. */
  defaultType?: RelationshipType;
  /** Default weight [0.0, 1.0]. */
  defaultWeight?: number;
  /** Fired when user confirms a type + weight. */
  onSelect: (type: RelationshipType, weight: number) => void;
  /** Fired when user cancels. */
  onCancel: () => void;
}

const CORE_TYPES: readonly RelationshipType[] = [
  "DELEGATES_TO",
  "CONSULTS",
  "COLLABORATES_WITH",
  "REPORTS_TO",
];

const EXTENSION_TYPES: readonly RelationshipType[] = [
  "MENTORS",
  "PEER_REVIEWS",
  "STAND_IN_FOR",
  "SHADOWS",
  "CHALLENGES",
  "TRUSTS",
];

const TYPE_DESCRIPTIONS: Record<RelationshipType, string> = {
  DELEGATES_TO: "A delegates a task to B",
  CONSULTS: "A consults B for an opinion",
  COLLABORATES_WITH: "A and B collaborate in parallel (undirected)",
  REPORTS_TO: "A reports to B",
  MENTORS: "A mentors B (B receives historical context)",
  PEER_REVIEWS: "A and B review each other's outputs (undirected)",
  STAND_IN_FOR: "A stands in for B when B is unavailable",
  SHADOWS: "A shadows B and observes B's events",
  CHALLENGES: "A challenges B's decision",
  TRUSTS: "A trusts B (used by TrustEngine skip-verify)",
};

const TYPE_BADGE: Record<RelationshipType, string> = {
  DELEGATES_TO: "border-info/40 text-info bg-info/10",
  CONSULTS: "border-accent/40 text-accent bg-accent/10",
  COLLABORATES_WITH: "border-ok/40 text-ok bg-ok/10",
  REPORTS_TO: "border-warn/40 text-warn bg-warn/10",
  MENTORS: "border-purple-400/40 text-purple-300 bg-purple-400/10",
  PEER_REVIEWS: "border-cyan-400/40 text-cyan-300 bg-cyan-400/10",
  STAND_IN_FOR: "border-pink-400/40 text-pink-300 bg-pink-400/10",
  SHADOWS: "border-slate-400/40 text-slate-300 bg-slate-400/10",
  CHALLENGES: "border-red-400/40 text-red-300 bg-red-400/10",
  TRUSTS: "border-emerald-400/40 text-emerald-300 bg-emerald-400/10",
};

export function EdgeTypeSelector({
  defaultType = "DELEGATES_TO",
  defaultWeight = 0.7,
  onSelect,
  onCancel,
}: EdgeTypeSelectorProps) {
  const [type, setType] = useState<RelationshipType>(defaultType);
  const [weight, setWeight] = useState<number>(defaultWeight);
  const selectId = useId();
  const weightId = useId();

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const clamped = Math.max(0, Math.min(1, weight));
      onSelect(type, clamped);
    },
    [type, weight, onSelect],
  );

  return (
    <div
      data-testid="edge-type-selector"
      className="border border-line bg-bg-soft rounded-md p-4 shadow-md w-[420px] max-w-full"
    >
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold">Select Relationship Type</h3>
        <button
          type="button"
          onClick={onCancel}
          className="text-xs text-ink-dim hover:text-ink"
          data-testid="edge-type-cancel"
        >
          ✕ Cancel
        </button>
      </div>
      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label
            htmlFor={selectId}
            className="block text-xs font-mono text-ink-dim mb-1"
          >
            Type ({ALL_RELATIONSHIP_TYPES.length} variants)
          </label>
          <select
            id={selectId}
            data-testid="edge-type-select"
            value={type}
            onChange={(e) => setType(e.target.value as RelationshipType)}
            className="w-full bg-bg border border-line rounded px-2 py-1.5 text-sm font-mono"
          >
            <optgroup label="4 核心">
              {CORE_TYPES.map((t) => (
                <option key={t} value={t} data-testid={`edge-type-${t}`}>
                  {t} — {TYPE_DESCRIPTIONS[t]}
                </option>
              ))}
            </optgroup>
            <optgroup label="6 扩展">
              {EXTENSION_TYPES.map((t) => (
                <option key={t} value={t} data-testid={`edge-type-${t}`}>
                  {t} — {TYPE_DESCRIPTIONS[t]}
                </option>
              ))}
            </optgroup>
          </select>
          <div className="mt-1.5 flex items-center gap-2">
            <span
              className={`pill font-mono text-[10px] ${TYPE_BADGE[type]}`}
              data-testid="edge-type-badge"
            >
              {type}
            </span>
            <span className="text-[10px] text-ink-mute">
              {type === "COLLABORATES_WITH" || type === "PEER_REVIEWS"
                ? "undirected"
                : "directed"}
            </span>
          </div>
        </div>
        <div>
          <label
            htmlFor={weightId}
            className="block text-xs font-mono text-ink-dim mb-1"
          >
            Weight ({weight.toFixed(2)})
          </label>
          <input
            id={weightId}
            data-testid="edge-weight-input"
            type="range"
            min={0}
            max={1}
            step={0.05}
            value={weight}
            onChange={(e) => setWeight(Number(e.target.value))}
            className="w-full"
          />
        </div>
        <div className="flex items-center gap-2 pt-2">
          <button
            type="submit"
            data-testid="edge-type-confirm"
            className="btn text-xs py-1.5 px-3 min-h-[34px] bg-info text-black"
          >
            Create edge
          </button>
          <button
            type="button"
            onClick={onCancel}
            className="btn text-xs py-1.5 px-3 min-h-[34px]"
            data-testid="edge-type-cancel-btn"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
