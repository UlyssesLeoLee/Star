// crates/star-desktop/frontend/tests/useLayoutEngine.test.ts
// =====================================================================
// Vitest test — useLayoutEngine hook (PR-251)
// 验证 JS fallback: pickAlgorithm + visualDistance + ready/version state
// =====================================================================
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useLayoutEngine, visualDistance } from '../src/hooks/useLayoutEngine';

describe('useLayoutEngine (PR-251)', () => {
  it('provides pickAlgorithm function', () => {
    const { result } = renderHook(() => useLayoutEngine());
    expect(typeof result.current.pickAlgorithm).toBe('function');
  });

  it('pickAlgorithm returns dagre for tree < 100 nodes', () => {
    const { result } = renderHook(() => useLayoutEngine());
    expect(result.current.pickAlgorithm('tree', 50)).toBe('dagre');
  });

  it('pickAlgorithm returns elk for tree >= 100 nodes', () => {
    const { result } = renderHook(() => useLayoutEngine());
    expect(result.current.pickAlgorithm('tree', 100)).toBe('elk');
    expect(result.current.pickAlgorithm('tree', 200)).toBe('elk');
  });

  it('pickAlgorithm returns d3_force for dependency/risk/history', () => {
    const { result } = renderHook(() => useLayoutEngine());
    expect(result.current.pickAlgorithm('dependency', 10)).toBe('d3_force');
    expect(result.current.pickAlgorithm('risk', 100)).toBe('d3_force');
    expect(result.current.pickAlgorithm('history', 1000)).toBe('d3_force');
  });

  it('pickAlgorithm returns elk for agent > 50 nodes', () => {
    const { result } = renderHook(() => useLayoutEngine());
    expect(result.current.pickAlgorithm('agent', 51)).toBe('elk');
    expect(result.current.pickAlgorithm('agent', 50)).toBe('d3_force');
  });

  it('visualDistance follows log(commitDistance + 1) * 30', () => {
    expect(visualDistance(0)).toBe(0);
    expect(visualDistance(1)).toBeCloseTo(Math.log(2) * 30, 5);
    expect(visualDistance(100)).toBeCloseTo(Math.log(101) * 30, 5);
  });

  it('falls back to JS (ready=false, version=js-fallback)', async () => {
    const { result } = renderHook(() => useLayoutEngine());
    await waitFor(() => {
      expect(result.current.version).toBe('0.1.0-js-fallback');
    });
    expect(result.current.ready).toBe(false);
  });
});
