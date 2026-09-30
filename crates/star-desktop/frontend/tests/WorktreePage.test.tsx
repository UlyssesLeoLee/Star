// crates/star-desktop/frontend/tests/WorktreePage.test.tsx
// =====================================================================
// Vitest test — WorktreePage renders 3 mock worktree groups
// PR-246 placeholder test (PR-247+ 替换为 Tauri IPC invoke)
// =====================================================================
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import WorktreePage from '../src/pages/WorktreePage';

describe('WorktreePage (PR-246)', () => {
  it('renders 3 mock worktree groups', () => {
    render(<WorktreePage />);
    expect(screen.getByText('core-canvas')).toBeDefined();
    expect(screen.getByText('frontend-ui')).toBeDefined();
    expect(screen.getByText('wasm-frontend')).toBeDefined();
  });

  it('renders table headers', () => {
    render(<WorktreePage />);
    expect(screen.getByText('ID')).toBeDefined();
    expect(screen.getByText('Name')).toBeDefined();
    expect(screen.getByText('Worktree Count')).toBeDefined();
    expect(screen.getByText('Active')).toBeDefined();
  });

  it('renders mock data rows', () => {
    render(<WorktreePage />);
    expect(screen.getByText('grp-001')).toBeDefined();
    expect(screen.getByText('grp-002')).toBeDefined();
    expect(screen.getByText('grp-003')).toBeDefined();
  });
});
