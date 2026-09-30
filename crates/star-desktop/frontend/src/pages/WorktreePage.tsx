// crates/star-desktop/frontend/src/pages/WorktreePage.tsx
// =====================================================================
// Star Desktop — WorktreePage (PR-246 placeholder)
// Per docs §3.2 P5: 后续 PR-247+ 替换为 KanbanBoard.tsx 复用
// =====================================================================
import { useState } from 'react';

interface WorktreeGroup {
  id: string;
  name: string;
  worktree_count: number;
  active: boolean;
}

const MOCK_GROUPS: WorktreeGroup[] = [
  { id: 'grp-001', name: 'core-canvas', worktree_count: 5, active: true },
  { id: 'grp-002', name: 'frontend-ui', worktree_count: 3, active: true },
  { id: 'grp-003', name: 'wasm-frontend', worktree_count: 4, active: false },
];

export default function WorktreePage() {
  const [groups] = useState<WorktreeGroup[]>(MOCK_GROUPS);

  return (
    <div>
      <h2>Worktree Groups (mock — PR-247+ 替换为 Tauri IPC)</h2>
      <p>Per PR #244 §3.1: React 19 + React Router 7 skeleton (本 PR)</p>

      <table style={{ borderCollapse: 'collapse', width: '100%' }}>
        <thead>
          <tr>
            <th style={{ border: '1px solid #ccc', padding: 8 }}>ID</th>
            <th style={{ border: '1px solid #ccc', padding: 8 }}>Name</th>
            <th style={{ border: '1px solid #ccc', padding: 8 }}>Worktree Count</th>
            <th style={{ border: '1px solid #ccc', padding: 8 }}>Active</th>
          </tr>
        </thead>
        <tbody>
          {groups.map((g) => (
            <tr key={g.id}>
              <td style={{ border: '1px solid #ccc', padding: 8 }}>{g.id}</td>
              <td style={{ border: '1px solid #ccc', padding: 8 }}>{g.name}</td>
              <td style={{ border: '1px solid #ccc', padding: 8 }}>{g.worktree_count}</td>
              <td style={{ border: '1px solid #ccc', padding: 8 }}>{g.active ? '✅' : '❌'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
