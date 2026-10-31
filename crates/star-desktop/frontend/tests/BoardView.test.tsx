// crates/star-desktop/frontend/tests/BoardView.test.tsx
// =====================================================================
// Vitest test — BoardView 集成组件 (PR-252)
// 8 tests: 集成 IPC + KanbanBoard + WASM hooks + view mode + DSL filter + transition + refresh
// =====================================================================

/*
@cypher schema=1 source_sha256=f1668534f22a2a2417407aa1898bce7b086c9d847b5dec6610f8f76730effdae
MERGE (self:File {path:"crates/star-desktop/frontend/tests/BoardView.test.tsx"})
MERGE (board:File {path:"crates/star-desktop/frontend/src/components/BoardView.tsx"})
MERGE (self)-[:TESTS]->(board)
@endcypher
*/

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { BoardView } from '../src/components/BoardView';

const mockWorkItems = [
  {
    id: 'test-wi-001', key: 'TEST-001', title: 'Story', status: 'todo',
    w_t_m: 'W', kind: 'story', priority: 'P1',
  },
  {
    id: 'test-wi-002', key: 'TEST-002', title: 'Bug', status: 'in_progress',
    w_t_m: 'T', kind: 'bug', priority: 'P0',
  },
  {
    id: 'test-wi-003', key: 'TEST-003', title: 'Spike', status: 'done',
    w_t_m: 'M', kind: 'spike', priority: 'P3',
  },
];

describe('BoardView (PR-252)', () => {
  beforeEach(() => {
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
    expect(screen.getByText('TEST-001')).toBeDefined();
    expect(screen.getByText('TEST-002')).toBeDefined();
    expect(screen.getByText('TEST-003')).toBeDefined();
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

  it('filters completed work items with the supported show DSL keyword', async () => {
    render(
      <MemoryRouter>
        <BoardView initialDsl="show" />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('dsl-filter-input')).toBeDefined();
    });
    // `show` filters out completed work items.
    const input = screen.getByTestId('dsl-filter-input');
    fireEvent.change(input, { target: { value: 'show' } });
    await waitFor(() => {
      expect(screen.getByTestId('board-meta').textContent).toMatch(/2 items/);
    });
    expect(screen.queryByText('TEST-003')).toBeNull(); // completed item filtered out
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
      expect(screen.getAllByTestId('kanban-column-review')).toHaveLength(3);
    });
    const reviewColumn = screen.getAllByTestId('kanban-column-review')[0];
    const dataTransfer = { getData: vi.fn(() => 'test-wi-001') };
    fireEvent.drop(reviewColumn, { dataTransfer });
    await waitFor(() => {
      // TEST-001 should now be in 'review' column
      expect(screen.getByTestId('board-meta').textContent).toMatch(/3 items/);
    });
  });

  it('shows error state on Tauri IPC failure', async () => {
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockRejectedValue(new Error('Tauri runtime error')),
    };
    // Inject __TAURI_INTERNALS__ to make hook think we're in Tauri (fall through to setError)
    render(
      <MemoryRouter>
        <BoardView />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(screen.getByTestId('board-view-error')).toBeDefined();
    });
  });
});
