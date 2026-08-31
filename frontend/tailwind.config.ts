import type { Config } from "tailwindcss";

// =====================================================================
// tailwind.config.ts — U5 (multica-style design tokens per §7 of
//   docs/frontend/design/ui-redesign-multica-style.md)
//
// Rationale:
//   - Extend the existing color palette (bg/border/ink/accent/ok/warn/err/info)
//     to the exact hex values from the design spec so U1 (AppShell) and the
//     other 4 workers (U2/U3/U4) can compose classes consistently.
//   - Add a 8-step fontSize scale (micro 11px → display-lg 48px) so that
//     `text-micro` / `text-label` / `text-body` etc. map to a single source
//     of truth rather than ad-hoc `text-[11px]` literals scattered through
//     components.
//   - Add Inter (sans) + JetBrains Mono (mono) font stacks so the layout
//     shell can rely on font-sans / font-mono only.
//   - Add soft/ring-accent boxShadow tokens used by Card / focus ring.
//
// Note: this is the FOUNDATION layer; U1 will compose these into
// AppShell, U2 into SubNav/Issues, U3 into Projects, U4 into the 4 panels.
// =====================================================================

const config: Config = {
  content: ["./src/**/*.{ts,tsx,mdx}", "./e2e/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // ── Background layers ── driven by CSS vars (theme-adaptive)
        bg: {
          DEFAULT: "var(--bg-DEFAULT)",
          soft:    "var(--bg-soft)",
          lighter: "var(--bg-lighter)",
          card:    "var(--bg-card)",
        },
        // ── Border tokens ── driven by CSS vars
        border: {
          DEFAULT: "var(--border-DEFAULT)",
          line:    "var(--border-line)",
        },
        // Legacy alias (back-compat for old components)
        line: "var(--line-DEFAULT)",
        // ── Foreground text ── driven by CSS vars
        ink: {
          DEFAULT: "var(--ink-DEFAULT)",
          dim:     "var(--ink-dim)",
          mute:    "var(--ink-mute)",
        },
        // ── Brand / accent ── driven by CSS vars
        accent: {
          DEFAULT: "var(--color-accent, var(--color-primary))",
          50:      "var(--color-accent-50, var(--color-primary))",
          soft:    "var(--color-accent-soft, var(--color-primary))",
        },
        // ── Semantic status ── driven by CSS vars
        ok:   "var(--ok-DEFAULT)",
        warn: "var(--warn-DEFAULT)",
        err:  "var(--err-DEFAULT)",
        info: "var(--info-DEFAULT)",
      },
      fontSize: {
        micro: ["11px", { lineHeight: "16px", letterSpacing: "0.04em" }],
        label: ["12px", { lineHeight: "16px" }],
        body: ["13px", { lineHeight: "20px" }],
        "body-lg": ["15px", { lineHeight: "22px" }],
        title: ["16px", { lineHeight: "24px" }],
        "title-lg": ["20px", { lineHeight: "28px" }],
        display: ["32px", { lineHeight: "40px" }],
        "display-lg": ["48px", { lineHeight: "56px", letterSpacing: "-0.02em" }],
      },
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Roboto",
          "Helvetica Neue",
          "Arial",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
      boxShadow: {
        // Per design §3+§5 — soft elevation for cards and the focus ring
        soft: "0 1px 2px 0 rgba(0, 0, 0, 0.3), 0 1px 3px 0 rgba(0, 0, 0, 0.15)",
        "ring-accent":
          "0 0 0 2px rgba(47, 129, 247, 0.4), 0 0 0 4px rgba(47, 129, 247, 0.15)",
      },
      borderRadius: {
        // multica-style tighter corners
        sm: "3px",
        DEFAULT: "5px",
        md: "6px",
        lg: "8px",
      },
    },
  },
  plugins: [],
};
export default config;
