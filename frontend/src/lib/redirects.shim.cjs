// =====================================================================
// frontend/src/lib/redirects.shim.cjs
//
// CommonJS bridge that next.config.js (which must be .js, not .ts) can
// require() to obtain the canonical redirect list without TypeScript
// compile-time overhead at Next.js config load.
//
// The 27 entries below MUST stay in sync with src/lib/redirects.ts.
// The vitest spec at e2e/redirects.spec.ts imports the .ts version
// and asserts count + specific entries are identical, so any drift
// between the two surfaces will fail the test suite.
//
// IMPORTANT: Do NOT Object.freeze() the entries. Next.js's
// load-custom-routes processRoutes() normalises each entry in-place
// (e.g. lower-casing the source path on Windows). Frozen objects throw
// "Cannot assign to read only property" during the build.
// =====================================================================

"use strict";

/** @type {Array<{source: string, destination: string, permanent: boolean}>} */
const LEGACY_REDIRECTS = [
  // ── /projects sink ─────────────────────────────────────────────────
  { source: "/workspace", destination: "/projects", permanent: false },
  { source: "/workspace/:id", destination: "/projects/:id", permanent: false },
  { source: "/project", destination: "/projects", permanent: false },
  { source: "/board", destination: "/projects?tab=board", permanent: false },
  { source: "/planning", destination: "/projects?tab=gantt", permanent: false },
  { source: "/scm", destination: "/projects?tab=workflow", permanent: false },
  { source: "/collaboration", destination: "/projects?tab=workflow", permanent: false },
  { source: "/workflow", destination: "/projects?tab=workflow", permanent: false },
  { source: "/relation", destination: "/projects?tab=relations", permanent: false },
  { source: "/canvas/:id", destination: "/projects?canvas=:id", permanent: false },

  // ── /issues sink ────────────────────────────────────────────────────
  { source: "/work-item", destination: "/issues?view=kanban", permanent: false },
  { source: "/worktree", destination: "/issues?view=tree", permanent: false },

  // ── /agents sink ────────────────────────────────────────────────────
  { source: "/agent", destination: "/agents", permanent: false },
  { source: "/validation", destination: "/agents?tab=validation", permanent: false },
  { source: "/automation", destination: "/agents?tab=automation", permanent: false },
  { source: "/development", destination: "/agents?tab=development", permanent: false },
  { source: "/local-runtime", destination: "/agents?tab=runtime", permanent: false },

  // ── /inbox sink ─────────────────────────────────────────────────────
  { source: "/notification", destination: "/inbox", permanent: false },
  { source: "/comment", destination: "/inbox?type=comment", permanent: false },
  { source: "/audit", destination: "/inbox?type=audit", permanent: false },
  { source: "/feedback", destination: "/inbox?type=feedback", permanent: false },
  { source: "/search", destination: "/inbox?type=search", permanent: false },
  { source: "/context", destination: "/inbox?type=context", permanent: false },

  // ── /settings sink ──────────────────────────────────────────────────
  { source: "/permission", destination: "/settings?tab=permissions", permanent: false },
  { source: "/identity", destination: "/settings?tab=members", permanent: false },
  { source: "/tenant", destination: "/settings?tab=workspace", permanent: false },
  { source: "/integration", destination: "/settings?tab=integrations", permanent: false },
];

module.exports = { LEGACY_REDIRECTS };
