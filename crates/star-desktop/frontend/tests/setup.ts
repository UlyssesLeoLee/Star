// crates/star-desktop/frontend/tests/setup.ts
// =====================================================================
// Vitest setup — Tauri IPC mock + jsdom (per PR #246 + PR #244 §3.5)
// =====================================================================

/*
@cypher schema=1 source_sha256=459e103f194035263640204b8860f2e1494d3e9ab7ebd68806554ad0c2898dd1
MERGE (self:File {path:"crates/star-desktop/frontend/tests/setup.ts"})
@endcypher
*/

import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(cleanup);

type TauriInternalsMock = {
  invoke: (...args: [command: string, args?: unknown]) => unknown;
};

declare global {
  var __TAURI_INTERNALS__: TauriInternalsMock;
}

// Tauri IPC mock (replace @tauri-apps/api invoke)
const mockInvoke = vi.fn((cmd: string) => {
  switch (cmd) {
    case 'list_work_items':
      return Promise.resolve([]);
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
