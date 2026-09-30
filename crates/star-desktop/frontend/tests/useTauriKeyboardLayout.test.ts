// crates/star-desktop/frontend/tests/useTauriKeyboardLayout.test.ts
// =====================================================================
// Vitest test — useTauriKeyboardLayout hook (PR-250)
// =====================================================================
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTauriKeyboardLayout } from '../src/hooks/useTauriKeyboardLayout';

const mockLayout = {
  swimlanes: ['W', 'T', 'M'],
  statuses: ['todo', 'in_progress', 'review', 'blocked', 'done', 'wontfix'],
};

describe('useTauriKeyboardLayout (PR-250)', () => {
  beforeEach(() => {
    // @ts-ignore
    globalThis.__TAURI_INTERNALS__ = {
      invoke: vi.fn().mockResolvedValue(mockLayout),
    };
  });

  it('fetches keyboard layout on mount', async () => {
    const { result } = renderHook(() => useTauriKeyboardLayout());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.layout).toEqual(mockLayout);
    expect(result.current.error).toBeNull();
  });

  it('falls back to mock in browser dev mode', async () => {
    // @ts-ignore
    delete globalThis.__TAURI_INTERNALS__;
    const { result } = renderHook(() => useTauriKeyboardLayout());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.layout?.swimlanes).toEqual(['W', 'T', 'M']);
    expect(result.current.layout?.statuses.length).toBe(6);
  });
});
