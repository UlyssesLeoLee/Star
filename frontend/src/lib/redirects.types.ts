// =====================================================================
// frontend/src/lib/redirects.types.ts
//
// Minimal type contract for Next.js redirect() entries. Mirrors
// next/dist/lib/load-custom-routes types but narrowed to the fields
// we use, so vitest can validate without pulling the next runtime.
//
// Reference: https://nextjs.org/docs/app/api-reference/next-config-js/redirects
// =====================================================================

export type NextRedirect = {
  /** Incoming URL path (supports :param placeholders). */
  source: string;
  /** Outgoing URL path or full URL (supports :param reuse). */
  destination: string;
  /**
   * permanent=false → 307 (preserves HTTP method)
   * permanent=true  → 308 (permanent)
   */
  permanent: boolean;
  /**
   * Optional: limit to specific HTTP methods / locales / headers / query /
   * cookies. We don't need any of these for the legacy-route redirect
   * pass, so all 26 entries are unconditional.
   */
  has?: Array<{
    type: "header" | "query" | "cookie";
    key: string;
    value?: string;
  }>;
  locale?: { redirect?: string | false };
  statusCode?: 307 | 308;
};
