// frontend/src/mocks/seed.ts
// mulberry32 — deterministic PRNG (fixed seed 1).
// Per docs/frontend/design/mock-data-isolation.md §2.4 — deterministic, CI-stable.
//
// Usage:
//   import { mulberry32 } from "@/mocks/seed";
//   const rand = mulberry32(1);
//   const n = rand(); // 0..1
//
// Note: hard-coded mock data in this project doesn't need seededRandom at runtime
// (it would be referenced by data-generation code if/when MSW handlers grow).
// For now, seed.ts only provides the primitive.

export function mulberry32(seed: number): () => number {
  let s = seed;
  return function () {
    s = (s + 0x6d2b79f5) | 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
