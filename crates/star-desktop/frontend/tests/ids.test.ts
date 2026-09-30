// crates/star-desktop/frontend/tests/ids.test.ts
// =====================================================================
// Vitest test — Star Desktop types/ids.ts (PR-247)
// 验证 W/T/M swimlane + 6 statuses + 兜底列 + DEFAULT 字段
// =====================================================================
import { describe, it, expect } from 'vitest';
import {
  DEFAULT_KANBAN_COLUMNS,
  DEFAULT_SWIMLANES,
  TODO_FALLBACK_STATUS,
  isFallbackStatus,
} from '../src/types/ids';

describe('types/ids.ts (PR-247)', () => {
  it('has 6 default kanban columns', () => {
    expect(DEFAULT_KANBAN_COLUMNS.length).toBe(6);
    expect(DEFAULT_KANBAN_COLUMNS[0].id).toBe('todo');
    expect(DEFAULT_KANBAN_COLUMNS[5].id).toBe('wontfix');
  });

  it('has 3 default swimlanes (W/T/M)', () => {
    expect(DEFAULT_SWIMLANES).toEqual(['W', 'T', 'M']);
  });

  it('fallback status is todo', () => {
    expect(TODO_FALLBACK_STATUS).toBe('todo');
    expect(isFallbackStatus('todo')).toBe(true);
    expect(isFallbackStatus('in_progress')).toBe(false);
    expect(isFallbackStatus('done')).toBe(false);
  });

  it('kanban columns have unique orders', () => {
    const orders = DEFAULT_KANBAN_COLUMNS.map((c) => c.order);
    const unique = new Set(orders);
    expect(unique.size).toBe(orders.length);
  });

  it('kanban columns have correct wip_limit', () => {
    const inProgress = DEFAULT_KANBAN_COLUMNS.find((c) => c.id === 'in_progress');
    const review = DEFAULT_KANBAN_COLUMNS.find((c) => c.id === 'review');
    expect(inProgress?.wip_limit).toBe(5);
    expect(review?.wip_limit).toBe(3);
  });
});
