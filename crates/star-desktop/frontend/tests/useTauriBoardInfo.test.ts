// crates/star-desktop/frontend/tests/useTauriBoardInfo.test.ts
// =====================================================================
// Vitest test — useTauriBoardInfo hook (PR-250)
// =====================================================================
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTauriBoardInfo } from '../src/hooks/useTauriBoardInfo';

const mockBoardInfo = {
  board_kind_count: 3,
  swimlane_group_by_count: 4,
  default_column_count: 6,
};

describe('useTauriBoardInfo (PR-250)', () => {
  beforeEach(() => {
    // @ts-ignore
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockResolvedValue(mockBoardInfo),
    };
  });

  it('fetches board info on mount', async () => {
    const { result } = renderHook(() => useTauriBoardInfo());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.boardInfo).toEqual(mockBoardInfo);
    expect(result.current.error).toBeNull();
  });

  it('falls back to mock in browser dev mode', async () => {
    // @ts-ignore
    delete globalThis.__TAURI_INTERNALS__;
    const { result } = renderHook(() => useTauriBoardInfo());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.boardInfo).toEqual({
      board_kind_count: 3,
      swimlane_group_by_count: 4,
      default_column_count: 6,
    });
  });
});
