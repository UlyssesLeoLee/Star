// crates/star-desktop/frontend/tests/HomePage.test.tsx
// =====================================================================
// Vitest test — HomePage renders 8 IPC commands + react-router link
// PR-246 React 19 + Tauri IPC mock (per setup.ts)
// =====================================================================
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import HomePage from '../src/pages/HomePage';

describe('HomePage (PR-246)', () => {
  it('renders the title', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );
    expect(screen.getByText(/Tauri 2.0 \+ React 19/)).toBeDefined();
  });

  it('renders 8 IPC commands', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );
    expect(screen.getByText(/list_work_items/)).toBeDefined();
    expect(screen.getByText(/list_worktree_groups/)).toBeDefined();
    expect(screen.getByText(/list_canvas_entities/)).toBeDefined();
    expect(screen.getByText(/get_app_version/)).toBeDefined();
    expect(screen.getByText(/get_keyboard_layout/)).toBeDefined();
    expect(screen.getByText(/get_board_info/)).toBeDefined();
    expect(screen.getByText(/get_worktree_info/)).toBeDefined();
    expect(screen.getByText(/get_canvas_info/)).toBeDefined();
  });

  it('renders link to worktree page', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );
    const link = screen.getByText(/Go to Worktree page/);
    expect(link).toBeDefined();
  });
});
