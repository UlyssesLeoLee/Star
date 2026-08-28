// =====================================================================
// frontend/src/app/(app)/issues/page.tsx — U5 placeholder
//
// Will be replaced by U2 (SubNav + Issues 主面板) worker.
// Renders a minimal stub so the 307 from /work-item and /worktree lands
// on a real 200 page instead of a 404 during the U5 deliverable window.
// =====================================================================

export default function IssuesPage() {
  return (
    <div className="p-6">
      <h1 className="text-title font-semibold text-ink">Issues</h1>
      <p className="text-body text-ink-dim mt-2">
        U5 placeholder. U2 will replace with: Kanban / List / Tree view,
        WIP limits, search, sprint selector.
      </p>
    </div>
  );
}
