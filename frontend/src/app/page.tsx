/*
@cypher schema=1 source_sha256=48dcca1c5b1dcb950d0489a0454d650c0df0d55feedf3a4e6d0541ec3d869c89
MERGE (self:File {path:"frontend/src/app/page.tsx"})
@endcypher
*/
// =====================================================================
// frontend/src/app/page.tsx — U5 (root redirect)
//
// The frontend redesign consolidates 22 domain routes into 6
// panel routes (per frontend layout design specification
// §2). The root path "/" now serves as a default landing redirect to
// the new primary panel "/inbox" (notifications / @-mentions / audit
// feed).
//
// Why a page-level redirect (and not a next.config.js entry)?
//   - The 26 legacy routes are in next.config.js as 307s.
//   - But "/" is special: it can also serve a real page in the future
//     (e.g. a marketing landing for unauthenticated users). Keeping it
//     in the page layer (rather than a 308 from next.config.js) means
//     we can swap the destination without re-deploying the routing
//     config. U1 may also move this to a "default dashboard" view that
//     varies per user role — the page layer is the right home for that.
//
// The redirect uses next/navigation's `redirect()` from a Server
// Component (no "use client") so Next.js emits a 307 at the server
// before any HTML is generated.
// =====================================================================

import { redirect } from "next/navigation";

export default function RootPage(): never {
  redirect("/inbox");
}
