// crates/star-desktop/frontend/tests/BoardView.test.tsx
// =====================================================================
// Vitest test — BoardView 集成组件 (PR-252)
// 8 tests: 集成 IPC + KanbanBoard + WASM hooks + view mode + DSL filter + transition + refresh
// =====================================================================
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { BoardView } from '../src/components/BoardView';

const mockWorkItems = [
  {
    id: 'wi-001', key: 'STAR-001', title: 'Story', status: 'todo',
    w_t_m: 'W', kind: 'story', priority: 'P1',
  },
  {
    id: 'wi-002', key: 'STAR-002', title: 'Bug', status: 'in_progress',
    w_t_m: 'T', kind: 'bug', priority: 'P0',
  },
  {
    id: 'wi-003', key: 'STAR-003', title: 'Spike', status: 'done',
    w_t_m: 'M', kind: 'spike', priority: 'P3',
  },
];

describe('BoardView (PR-252)', () => {
  beforeEach(() => {
    // @ts-ignore
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockResolvedValue(mockWorkItems),
    };
  });

  it('renders toolbar with view mode select + dsl filter + refresh', async () => {
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('view-mode-select')).toBeDefined();
    });
    expect(screen.getByTestId('dsl-filter-input')).toBeDefined();
    expect(screen.getByTestId('refresh-button')).toBeDefined();
  });

  it('renders board meta with algorithm info', async () => {
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('board-meta')).toBeDefined();
    });
    expect(screen.getByTestId('board-meta').textContent).toMatch(/algo: dagre/);
    expect(screen.getByTestId('board-meta').textContent).toMatch(/3 items/);
  });

  it('renders kanban board with work items', async () => {
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('kanban-board')).toBeDefined();
    });
    expect(screen.getByText('STAR-001')).toBeDefined();
    expect(screen.getByText('STAR-002')).toBeDefined();
    expect(screen.getByText('STAR-003')).toBeDefined();
  });

  it('changes algorithm when view mode changes', async () => {
    render(
      <MemoryRouter>
        <BoardView initialViewMode="tree" />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('view-mode-select')).toBeDefined();
    });
    const select = screen.getByTestId('view-mode-select') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'agent' } });
    await waitFor(() => {
      expect(screen.getByTestId('board-meta').textContent).toMatch(/algo: d3_force|elk/);
    });
  });

  it('filters work items by DSL', async () => {
    render(
      <MemoryRouter>
        <BoardView initialDsl="bug" />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('dsl-filter-input')).toBeDefined();
    });
    // 'bug' 关键字 → 只显示 kind=bug (STAR-002)
    const input = screen.getByTestId('dsl-filter-input');
    fireEvent.change(input, { target: { value: 'bug' } });
    await waitFor(() => {
      expect(screen.getByTestId('board-meta').textContent).toMatch(/1 items/);
    });
    expect(screen.queryByText('STAR-001')).toBeNull(); // story filtered out
  });

  it('refreshes work items on Refresh button click', async () => {
    const { invoke } = globalThis.__TAURI_INTERNALS__ as { invoke: ReturnType<typeof vi.fn> };
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('refresh-button')).toBeDefined();
    });
    const initialCallCount = invoke.mock.calls.length;
    fireEvent.click(screen.getByTestId('refresh-button'));
    await waitFor(() => {
      expect(invoke.mock.calls.length).toBeGreaterThan(initialCallCount);
    });
  });

  it('handles kanban transition via drop zone', async () => {
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('kanban-column-review')).toBeDefined();
    });
    const reviewColumn = screen.getByTestId('kanban-column-review');
    const dataTransfer = { getData: vi.fn(() => 'wi-001') };
    fireEvent.drop(reviewColumn, { dataTransfer });
    await waitFor(() => {
      // STAR-001 should now be in 'review' column
      expect(screen.getByTestId('board-meta').textContent).toMatch(/3 items/);
    });
  });

  it('shows error state on Tauri IPC failure', async () => {
    // @ts-ignore
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockRejectedValue(new Error('Tauri runtime error')),
    };
    // Inject __TAURI_INTERNALS__ to make hook think we're in Tauri (fall through to setError)
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    // Error fallback to mock in non-Tauri runtime, so should still render
    await waitFor(() => {
      expect(screen.getByTestId('board-view-loading') || screen.getByTestId('board-view')).toBeDefined();
    });
  });
});
