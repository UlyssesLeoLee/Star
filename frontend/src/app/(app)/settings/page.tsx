// =====================================================================
// frontend/src/app/(app)/settings/page.tsx — U5 placeholder
//
// Will be replaced by U4 (Agents / Analytics / Inbox / Settings) worker.
// Renders a minimal stub so the 307 from /permission, /identity,
// /tenant, /integration lands on a real 200 page instead of a 404
// during the U5 deliverable window.
// =====================================================================

export default function SettingsPage() {
  return (
    <div className="p-6">
      <h1 className="text-title font-semibold text-ink">Settings</h1>
      <p className="text-body text-ink-dim mt-2">
        U5 placeholder. U4 will replace with: Profile / Workspace / Members
        / Permissions / Runtimes / Skills / Billing.
      </p>
    </div>
  );
}
