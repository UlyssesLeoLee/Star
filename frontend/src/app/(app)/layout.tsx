// =====================================================================
// frontend/src/app/(app)/layout.tsx — U5 placeholder
//
// (app) is a Next.js route group. Its URL prefix is empty — so files
// under (app)/ route at the same paths as without the group:
//   (app)/inbox/page.tsx   →  /inbox
//   (app)/issues/page.tsx  →  /issues
//   (app)/projects/page.tsx → /projects
//   (app)/agents/page.tsx  →  /agents
//   (app)/analytics/page.tsx → /analytics
//   (app)/settings/page.tsx → /settings
//
// This is the LAYOUT shell that the 6 new panel pages will share. U1
// (AppShell) will REWRITE this file to render AppHeader (top bar) +
// SubNav (left sidebar) wrapping {children}.
//
// For U5's deliverable, the layout is intentionally minimal: it just
// passes children through. This lets U2/U3/U4 add their page.tsx files
// under (app)/inbox, (app)/issues etc. without depending on U1's
// AppShell landing first.
//
// Important: this layout does NOT include Sidebar/Topbar from the
// legacy shell. Those remain in app/layout.tsx (RootLayout) so that
// the 22 legacy pages that are mid-redirect still render the shell
// if a request bypasses the redirect (e.g. dev-mode HMR / direct file
// import). Once U1 lands AppShell, RootLayout will be slimmed down to
// html/body + Providers only.
// =====================================================================

export default function AppLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Intentionally a no-op pass-through. U1 will replace the body of
  // this component with the AppShell scaffold (AppHeader + SubNav +
  // <main>{children}</main>).
  return <>{children}</>;
}
