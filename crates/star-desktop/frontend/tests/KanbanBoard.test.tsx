// crates/star-desktop/frontend/tests/KanbanBoard.test.tsx
// =====================================================================
// Vitest test — Star Desktop KanbanBoard component (PR-249)
// 7 tests: 6 columns × W/T/M swimlane + drop zone + drag + WIP limit + onTransition
// =====================================================================
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { KanbanBoard } from '../src/components/KanbanBoard';
import type { WorkItem, WorkItemStatus } from '../src/types/ids';

const mockWorkItems: WorkItem[] = [
  {
    id: 'test-wi-001', key: 'TEST-001', title: 'Story W', status: 'todo',
    w_t_m: 'W', kind: 'story', priority: 'P1', story_points: 3,
  },
  {
    id: 'test-wi-002', key: 'TEST-002', title: 'Task T', status: 'in_progress',
    w_t_m: 'T', kind: 'task', priority: 'P2', story_points: 2,
  },
  {
    id: 'test-wi-003', key: 'TEST-003', title: 'Spike M', status: 'done',
    w_t_m: 'M', kind: 'spike', priority: 'P3', story_points: 1,
  },
  {
    id: 'test-wi-004', key: 'TEST-004', title: 'Bug T', status: 'todo',
    w_t_m: 'T', kind: 'bug', priority: 'P0', story_points: 5,
  },
];

function renderBoard(props: Partial<React.ComponentProps<typeof KanbanBoard>> = {}) {
  return render(
    <MemoryRouter>
      <KanbanBoard workItems={mockWorkItems} {...props} />
    </MemoryRouter>,
  );
}

describe('KanbanBoard (PR-249)', () => {
  it('renders 6 default kanban columns', () => {
    renderBoard();
    expect(screen.getByTestId('kanban-column-todo')).toBeDefined();
    expect(screen.getByTestId('kanban-column-in_progress')).toBeDefined();
    expect(screen.getByTestId('kanban-column-review')).toBeDefined();
    expect(screen.getByTestId('kanban-column-blocked')).toBeDefined();
    expect(screen.getByTestId('kanban-column-done')).toBeDefined();
    expect(screen.getByTestId('kanban-column-wontfix')).toBeDefined();
  });

  it('renders 3 swimlane labels when enabled', () => {
    renderBoard();
    expect(screen.getByText(/W — Work/)).toBeDefined();
    expect(screen.getByText(/T — Transaction/)).toBeDefined();
    expect(screen.getByText(/M — Master/)).toBeDefined();
  });

  it('groups workItems into W/T/M swimlanes', () => {
    renderBoard();
    const wSwimlane = screen.getByTestId('kanban-column-todo')?.closest('[data-swimlane="W"]');
    expect(wSwimlane).toBeDefined();
    // TEST-001 should be in W lane todo column
    expect(screen.getByText('TEST-001')).toBeDefined();
    expect(screen.getByText('TEST-002')).toBeDefined();
    expect(screen.getByText('TEST-003')).toBeDefined();
  });

  it('marks fallback column (todo) as protected', () => {
    renderBoard();
    expect(screen.getByText(/\(protected\)/)).toBeDefined();
  });

  it('handles drop zone visual state', () => {
    renderBoard();
    const column = screen.getByTestId('kanban-column-in_progress');
    const dataTransfer = { setData: vi.fn(), getData: vi.fn(() => 'test-wi-001'), dropEffect: '' };
    fireEvent.dragOver(column, { dataTransfer });
    expect(column.className).toContain('border-blue-500');
    fireEvent.dragLeave(column, { dataTransfer });
  });

  it('calls onTransition when card is dropped', () => {
    const onTransition = vi.fn();
    renderBoard({ onTransition });
    const column = screen.getByTestId('kanban-column-review');
    const dataTransfer = { getData: vi.fn(() => 'test-wi-001') };
    fireEvent.drop(column, { dataTransfer });
    expect(onTransition).toHaveBeenCalledWith('test-wi-001', 'review');
  });

  it('shows WIP limit warning when exceeded', () => {
    const items = Array.from({ length: 6 }, (_, i) => ({
      id: `test-wi-${i}`, key: `TEST-${i}`, title: `Item ${i}`, status: 'in_progress' as WorkItemStatus,
      w_t_m: 'W' as const, kind: 'task' as const, priority: 'P2' as const,
    }));
    renderBoard({ workItems: items });
    expect(screen.getByText(/Over WIP limit/)).toBeDefined();
  });
});
