// crates/star-desktop/frontend/tests/setup.ts
// =====================================================================
// Vitest setup — Tauri IPC mock + jsdom (per PR #246 + PR #244 §3.5)
// =====================================================================
import '@testing-library/react';

// Tauri IPC mock (replace @tauri-apps/api invoke)
const mockInvoke = vi.fn((cmd: string) => {
  switch (cmd) {
    case 'list_work_items':
      return Promise.resolve([
        { id: 'wi-001', title: 'feat(canvas): W/T/M swimlane', status: 'in_progress', w_t_m: 'W' },
        { id: 'wi-002', title: 'fix(canvas): bug #71', status: 'review', w_t_m: 'T' },
        { id: 'wi-003', title: 'spike(wasm): layout PoC', status: 'done', w_t_m: 'M' },
        { id: 'wi-004', title: 'doc(arch): Tauri PoC', status: 'todo', w_t_m: 'M' },
      ]);
    case 'list_worktree_groups':
      return Promise.resolve([
        { id: 'grp-001', name: 'core-canvas', worktree_count: 5, active: true },
        { id: 'grp-002', name: 'frontend-ui', worktree_count: 3, active: true },
        { id: 'grp-003', name: 'wasm-frontend', worktree_count: 4, active: false },
      ]);
    case 'get_app_version':
      return Promise.resolve('0.1.0');
    case 'get_keyboard_layout':
      return Promise.resolve({
        swimlanes: ['W', 'T', 'M'],
        statuses: ['todo', 'in_progress', 'review', 'blocked', 'done', 'wontfix'],
      });
    default:
      return Promise.reject(new Error(`mock for ${cmd} not implemented`));
  }
});

globalThis.__TAURI_INTERNALS__ = {
  invoke: mockInvoke,
};
