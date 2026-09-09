"use client";

// =====================================================================
// AchievementWall — 成就墙 (per brief v0.50 §2.1 A)
// =====================================================================
// Per docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.2.4
// + docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §1.1
// + docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md §2.3
//
// 20 成就 (8 拓扑 + 7 行为 + 5 产出) + 4 稀有度筛选 (Common/Rare/Epic/Legendary)
// + 3 类别筛选 (Topology/Behavior/Output) + 解锁状态视觉差异化.
// =====================================================================

import { useEffect, useMemo, useState } from "react";
import { useARGStore } from "@/lib/arg/store";
import type {
  Achievement,
  AchievementCategory,
  Rarity,
} from "@/lib/arg/types";

const RARITIES: readonly Rarity[] = ["common", "rare", "epic", "legendary"];
const CATEGORIES: readonly AchievementCategory[] = [
  "topology",
  "behavior",
  "output",
];

const RARITY_BADGE: Record<Rarity, string> = {
  common: "border-ink-mute/40 text-ink-mute bg-bg-soft",
  rare: "border-info/40 text-info bg-info/10",
  epic: "border-purple-400/40 text-purple-300 bg-purple-400/10",
  legendary: "border-amber-400/40 text-amber-300 bg-amber-400/10",
};

const CATEGORY_LABEL: Record<AchievementCategory, string> = {
  topology: "Topology (8)",
  behavior: "Behavior (7)",
  output: "Output (5)",
};

export function AchievementWall() {
  const [rarity, setRarity] = useState<Rarity | "all">("all");
  const [category, setCategory] = useState<AchievementCategory | "all">("all");

  const achievements = useARGStore((s) => s.achievements);
  const achievementsLoading = useARGStore((s) => s.achievementsLoading);
  const achievementsError = useARGStore((s) => s.achievementsError);
  const unlocked = useARGStore((s) => s.unlockedAchievements);
  const loadAchievements = useARGStore((s) => s.loadAchievements);
  const loadMyUnlocks = useARGStore((s) => s.loadMyUnlocks);

  useEffect(() => {
    void loadAchievements();
    void loadMyUnlocks();
  }, [loadAchievements, loadMyUnlocks]);

  const filtered = useMemo<Achievement[]>(() => {
    return achievements.filter((a) => {
      const okRarity = rarity === "all" || a.rarity === rarity;
      const okCategory = category === "all" || a.category === category;
      return okRarity && okCategory;
    });
  }, [achievements, rarity, category]);

  const grouped = useMemo(() => {
    const out: Record<AchievementCategory, Achievement[]> = {
      topology: [],
      behavior: [],
      output: [],
    };
    for (const a of filtered) out[a.category].push(a);
    return out;
  }, [filtered]);

  const unlockedCount = useMemo(
    () => achievements.filter((a) => unlocked.has(a.code)).length,
    [achievements, unlocked],
  );

  return (
    <div className="space-y-3" data-testid="achievement-wall">
      <div className="flex flex-wrap items-center gap-2 text-xs font-mono">
        <div className="flex items-center gap-1" data-testid="rarity-filter">
          <span className="text-ink-dim">Rarity:</span>
          <button
            type="button"
            onClick={() => setRarity("all")}
            data-testid="rarity-all"
            className={`pill border text-[10px] px-2 py-0.5 ${
              rarity === "all"
                ? "border-accent text-accent bg-accent/10"
                : "border-line text-ink-dim"
            }`}
          >
            All
          </button>
          {RARITIES.map((r) => (
            <button
              key={r}
              type="button"
              onClick={() => setRarity(r)}
              data-testid={`rarity-${r}`}
              className={`pill border text-[10px] px-2 py-0.5 ${
                rarity === r
                  ? RARITY_BADGE[r]
                  : "border-line text-ink-dim"
              }`}
            >
              {r}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-1 ml-2" data-testid="category-filter">
          <span className="text-ink-dim">Category:</span>
          <button
            type="button"
            onClick={() => setCategory("all")}
            data-testid="category-all"
            className={`pill border text-[10px] px-2 py-0.5 ${
              category === "all"
                ? "border-accent text-accent bg-accent/10"
                : "border-line text-ink-dim"
            }`}
          >
            All
          </button>
          {CATEGORIES.map((c) => (
            <button
              key={c}
              type="button"
              onClick={() => setCategory(c)}
              data-testid={`category-${c}`}
              className={`pill border text-[10px] px-2 py-0.5 ${
                category === c
                  ? "border-accent text-accent bg-accent/10"
                  : "border-line text-ink-dim"
              }`}
            >
              {CATEGORY_LABEL[c]}
            </button>
          ))}
        </div>
        <div className="ml-auto text-ink-dim" data-testid="achievement-progress">
          {unlockedCount} / {achievements.length} unlocked
        </div>
      </div>
      {achievementsError && (
        <div
          data-testid="achievement-error"
          className="text-xs text-warn border border-warn/40 bg-warn/10 p-2 rounded"
        >
          Failed to load achievements: {achievementsError}
        </div>
      )}
      {achievementsLoading && achievements.length === 0 ? (
        <div className="text-xs text-ink-dim" data-testid="achievement-loading">
          Loading 20 achievements…
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
          {filtered.length === 0 ? (
            <div
              className="col-span-full text-xs text-ink-dim italic"
              data-testid="achievement-empty"
            >
              No achievements match the current filter.
            </div>
          ) : (
            filtered.map((a) => {
              const isUnlocked = unlocked.has(a.code);
              return (
                <div
                  key={a.code}
                  data-testid={`achievement-card-${a.code}`}
                  data-unlocked={isUnlocked}
                  className={`card p-3 space-y-1 ${
                    isUnlocked ? "" : "opacity-50 grayscale"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span
                      className={`pill border text-[10px] px-1.5 py-0.5 ${
                        RARITY_BADGE[a.rarity]
                      }`}
                    >
                      {a.rarity}
                    </span>
                    <span className="text-[10px] font-mono text-ink-dim">
                      {a.code}
                    </span>
                  </div>
                  <div className="text-sm font-semibold">
                    {a.name_en}
                    <span className="ml-1 text-xs text-ink-dim">
                      ({a.name_zh})
                    </span>
                  </div>
                  <div className="text-xs text-ink-dim">{a.description}</div>
                  <div className="text-[10px] uppercase text-ink-mute">
                    {a.category}
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}
      {/* category sections (only when "all") */}
      {category === "all" && (
        <div className="text-[10px] text-ink-mute font-mono">
          by category: topo {grouped.topology.length} / beh{" "}
          {grouped.behavior.length} / out {grouped.output.length}
        </div>
      )}
    </div>
  );
}
