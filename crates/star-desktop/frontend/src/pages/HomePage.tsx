// crates/star-desktop/frontend/src/pages/HomePage.tsx
// =====================================================================
// Star Desktop — HomePage (PR-246)
// 简单 home page, 展示 Tauri IPC 8 commands 列表 + react-router 链接
// =====================================================================
import { Link } from 'react-router-dom';

const IPC_COMMANDS = [
  { id: 1, name: 'list_work_items', description: '4 mock work items (W/T/M)' },
  { id: 2, name: 'list_worktree_groups', description: '4 mock worktree groups' },
  { id: 3, name: 'list_canvas_entities', description: '4 mock canvas entities' },
  { id: 4, name: 'get_app_version', description: 'semver string from CARGO_PKG_VERSION' },
  { id: 5, name: 'get_keyboard_layout', description: 'W/T/M swimlane + 6 statuses' },
  { id: 6, name: 'get_board_info', description: 'BoardKind × 3 + SwimlaneGroupBy × 4 + 6 columns' },
  { id: 7, name: 'get_worktree_info', description: 'WorktreeStatus × 6 + 4 health dimensions' },
  { id: 8, name: 'get_canvas_info', description: 'route prefix /api/v1/canvas + 5 phases' },
];

export default function HomePage() {
  return (
    <div>
      <h2>Tauri 2.0 + React 19 (PR-246)</h2>
      <p>Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5</p>

      <h3>Available IPC Commands (8 total)</h3>
      <ul>
        {IPC_COMMANDS.map((cmd) => (
          <li key={cmd.id}>
            <code>#{cmd.id} {cmd.name}</code> — {cmd.description}
          </li>
        ))}
      </ul>

      <h3>Worktree Group (per PR-244 P5 plan)</h3>
      <ul>
        <li><Link to="/worktree">Go to Worktree page →</Link></li>
      </ul>
    </div>
  );
}
