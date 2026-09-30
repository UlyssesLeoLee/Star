// crates/star-desktop/frontend/tests/useQueryEngine.test.ts
// =====================================================================
// Vitest test — useQueryEngine hook (PR-251)
// 验证 JS fallback: parseKeywords + allKeywords + version
// =====================================================================
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useQueryEngine, parseDslKeywords } from '../src/hooks/useQueryEngine';

describe('useQueryEngine (PR-251)', () => {
  it('parseDslKeywords returns matching keywords', () => {
    expect(parseDslKeywords('show ready')).toEqual(['show']);
    expect(parseDslKeywords('behind >5')).toEqual(['behind']);
    expect(parseDslKeywords('agent codex modified file.txt')).toEqual(['agent', 'modified']);
  });

  it('parseDslKeywords ignores non-keywords', () => {
    expect(parseDslKeywords('show ready invalid')).toEqual(['show']);
    expect(parseDslKeywords('foo bar baz')).toEqual([]);
  });

  it('parseDslKeywords is case-insensitive', () => {
    expect(parseDslKeywords('SHOW ready')).toEqual(['show']);
    expect(parseDslKeywords('Behind >5')).toEqual(['behind']);
  });

  it('provides parseKeywords + getKeywordName', () => {
    const { result } = renderHook(() => useQueryEngine());
    expect(typeof result.current.parseKeywords).toBe('function');
    expect(result.current.getKeywordName('show')).toBe('Show');
    expect(result.current.getKeywordName('agent')).toBe('Agent');
    expect(result.current.getKeywordName('behind')).toBe('Behind');
  });

  it('allKeywords returns 6 keywords', () => {
    const { result } = renderHook(() => useQueryEngine());
    expect(result.current.allKeywords.length).toBe(6);
    expect(result.current.allKeywords).toContain('show');
    expect(result.current.allKeywords).toContain('agent');
    expect(result.current.allKeywords).toContain('modified');
  });

  it('falls back to JS (ready=false, version=js-fallback)', async () => {
    const { result } = renderHook(() => useQueryEngine());
    await waitFor(() => {
      expect(result.current.version).toBe('0.1.0-js-fallback');
    });
    expect(result.current.ready).toBe(false);
  });
});
