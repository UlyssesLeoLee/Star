// =====================================================================
// frontend/src/app/(app)/projects/page.tsx — U5 placeholder
//
// Will be replaced by U3 (Projects 多 panel) worker.
// Renders a minimal stub so the 307 from /workspace, /project, /board,
// /planning, /scm, /collaboration, /workflow, /relation, /canvas/:id
// lands on a real 200 page instead of a 404 during the U5 deliverable.
// =====================================================================

export default function ProjectsPage() {
  return (
    <div className="p-6">
      <h1 className="text-title font-semibold text-ink">Projects</h1>
      <p className="text-body text-ink-dim mt-2">
        U5 placeholder. U3 will replace with: List / Board / Gantt / Calendar
        / Workflow tabs, filter bar.
      </p>
    </div>
  );
}
