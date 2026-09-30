// crates/star-desktop/frontend/tests/useTauriWorkItems.test.ts
// =====================================================================
// Vitest test — useTauriWorkItems hook (PR-250)
// 验证: invoke('list_work_items') → setWorkItems + loading/error state
// =====================================================================
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTauriWorkItems } from '../src/hooks/useTauriWorkItems';

const mockWorkItems = [
  { id: 'wi-001', title: 'Item 1', status: 'todo', w_t_m: 'W' },
  { id: 'wi-002', title: 'Item 2', status: 'done', w_t_m: 'T' },
];

describe('useTauriWorkItems (PR-250)', () => {
  beforeEach(() => {
    // @ts-ignore — reset global before each test
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockResolvedValue(mockWorkItems),
    };
  });

  it('fetches work items on mount', async () => {
    const { result } = renderHook(() => useTauriWorkItems());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.workItems).toEqual(mockWorkItems);
    expect(result.current.error).toBeNull();
  });

  it('exposes refetch function', async () => {
    const { result } = renderHook(() => useTauriWorkItems());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(typeof result.current.refetch).toBe('function');
  });

  it('handles invoke error gracefully (non-Tauri runtime)', async () => {
    // @ts-ignore — simulate non-Tauri runtime (no __TAURI_INTERNALS__)
    delete globalThis.__TAURI_INTERNALS__;
    const { result } = renderHook(() => useTauriWorkItems());
    await waitFor(() => expect(result.current.loading).toBe(false));
    // Should fall back to mock data
    expect(result.current.workItems.length).toBeGreaterThan(0);
    expect(result.current.error).toBeNull();
  });
});
