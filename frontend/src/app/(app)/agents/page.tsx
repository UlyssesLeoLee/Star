// =====================================================================
// frontend/src/app/(app)/agents/page.tsx — U5 placeholder
//
// Will be replaced by U4 (Agents / Analytics / Inbox / Settings) worker.
// Renders a minimal stub so the 307 from /agent, /validation,
// /automation, /development, /local-runtime lands on a real 200 page
// instead of a 404 during the U5 deliverable window.
// =====================================================================

export default function AgentsPage() {
  return (
    <div className="p-6">
      <h1 className="text-title font-semibold text-ink">Agents</h1>
      <p className="text-body text-ink-dim mt-2">
        U5 placeholder. U4 will replace with: agent list (left), state
        machine diagram (center), lease/heartbeat panel (right).
      </p>
    </div>
  );
}
