// crates/star-desktop/frontend/tests/KanbanCard.test.tsx
// =====================================================================
// Vitest test — Star Desktop KanbanCard component (PR-248)
// 5 tests: 渲染、draggable、handleClick、handleArchClick、priority color
// =====================================================================
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { KanbanCard } from '../src/components/KanbanCard';
import type { WorkItem } from '../src/types/ids';

const mockWorkItem: WorkItem = {
  id: 'wi-001',
  key: 'STAR-001',
  title: 'feat(canvas): W/T/M swimlane',
  status: 'in_progress',
  w_t_m: 'W',
  kind: 'story',
  priority: 'P0',
  story_points: 3,
};

function renderCard(props: Partial<React.ComponentProps<typeof KanbanCard>> = {}) {
  return render(
    <MemoryRouter>
      <KanbanCard workItem={mockWorkItem} {...props} />
    </MemoryRouter>,
  );
}

describe('KanbanCard (PR-248)', () => {
  it('renders key, title, status, priority', () => {
    renderCard();
    expect(screen.getByText('STAR-001')).toBeDefined();
    expect(screen.getByText('feat(canvas): W/T/M swimlane')).toBeDefined();
    expect(screen.getByText('in_progress')).toBeDefined();
    expect(screen.getByText('P0')).toBeDefined();
  });

  it('renders story_points when provided', () => {
    renderCard();
    expect(screen.getByText('3 SP')).toBeDefined();
  });

  it('handles click via onClick prop', () => {
    const onClick = vi.fn();
    renderCard({ onClick });
    const card = screen.getByTestId('kanban-card-wi-001');
    fireEvent.click(card);
    expect(onClick).toHaveBeenCalledWith(mockWorkItem);
  });

  it('handles drag start with dataTransfer', () => {
    const onDragStart = vi.fn();
    renderCard({ onDragStart });
    const card = screen.getByTestId('kanban-card-wi-001');
    const dataTransfer = { setData: vi.fn(), effectAllowed: '' };
    fireEvent.dragStart(card, { dataTransfer });
    expect(dataTransfer.setData).toHaveBeenCalledWith('text/issue-id', 'wi-001');
    expect(onDragStart).toHaveBeenCalled();
  });

  it('shows arch icon only when onArchClick provided', () => {
    const { rerender } = renderCard();
    expect(screen.queryByTestId('kanban-card-arch-wi-001')).toBeNull();

    const onArchClick = vi.fn();
    rerender(
      <MemoryRouter>
        <KanbanCard workItem={mockWorkItem} onArchClick={onArchClick} />
      </MemoryRouter>,
    );
    expect(screen.getByTestId('kanban-card-arch-wi-001')).toBeDefined();

    const archBtn = screen.getByTestId('kanban-card-arch-wi-001');
    fireEvent.click(archBtn);
    expect(onArchClick).toHaveBeenCalledWith(mockWorkItem);
  });
});
