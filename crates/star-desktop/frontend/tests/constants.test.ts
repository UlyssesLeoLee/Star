// crates/star-desktop/frontend/tests/constants.test.ts
// =====================================================================
// Vitest test — Star Desktop constants.ts (PR-247)
// =====================================================================
import { describe, it, expect } from 'vitest';
import {
  KANBAN_COLUMN_COUNT,
  ALL_WORK_ITEM_STATUSES,
  SWIMLANE_ORDER,
  getColumnByStatus,
  getNextColumn,
  isFallbackStatus,
  TODO_FALLBACK_STATUS,
} from '../src/constants';

describe('constants.ts (PR-247)', () => {
  it('has 6 kanban columns', () => {
    expect(KANBAN_COLUMN_COUNT).toBe(6);
  });

  it('has 6 work item statuses', () => {
    expect(ALL_WORK_ITEM_STATUSES.length).toBe(6);
    expect(ALL_WORK_ITEM_STATUSES[0]).toBe('todo');
    expect(ALL_WORK_ITEM_STATUSES[5]).toBe('wontfix');
  });

  it('has 3 swimlanes in W/T/M order', () => {
    expect(SWIMLANE_ORDER).toEqual(['W', 'T', 'M']);
  });

  it('getColumnByStatus returns correct column', () => {
    expect(getColumnByStatus('todo')?.id).toBe('todo');
    expect(getColumnByStatus('done')?.id).toBe('done');
    expect(getColumnByStatus('blocked')).toBeDefined();
  });

  it('getNextColumn transitions correctly', () => {
    expect(getNextColumn('todo')).toBe('in_progress');
    expect(getNextColumn('in_progress')).toBe('review');
    expect(getNextColumn('review')).toBe('blocked');
    expect(getNextColumn('wontfix')).toBe(null);
  });

  it('fallback status protection', () => {
    expect(TODO_FALLBACK_STATUS).toBe('todo');
    expect(isFallbackStatus('todo')).toBe(true);
    expect(isFallbackStatus('in_progress')).toBe(false);
  });
});
