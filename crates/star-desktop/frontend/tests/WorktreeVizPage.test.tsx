// crates/star-desktop/frontend/tests/WorktreeVizPage.test.tsx
// =====================================================================
// Vitest test — WorktreeVizPage (PR-265)
// 验证 useLayoutEngine WASM hook 集成 100+ 节点 viz
// =====================================================================
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import WorktreeVizPage from '../src/pages/WorktreeVizPage';

describe('WorktreeVizPage (PR-265)', () => {
  it('renders page + svg + toolbar', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    expect(screen.getByTestId('worktree-viz-page')).toBeDefined();
    expect(screen.getByTestId('viz-svg')).toBeDefined();
    expect(screen.getByTestId('viz-toolbar')).toBeDefined();
    expect(screen.getByTestId('viz-node-count')).toBeDefined();
    expect(screen.getByTestId('viz-view-mode')).toBeDefined();
    expect(screen.getByTestId('viz-algorithm')).toBeDefined();
  });

  it('default 120 nodes → tree mode → elk (>= 100 threshold)', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    const algo = screen.getByTestId('viz-algorithm');
    expect(algo.textContent).toMatch(/algo:.*elk/);
  });

  it('renders 120 viz-node-{id} SVG groups', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    expect(screen.getByTestId('viz-node-wt-0000')).toBeDefined();
    expect(screen.getByTestId('viz-node-wt-0050')).toBeDefined();
    expect(screen.getByTestId('viz-node-wt-0119')).toBeDefined();
    const circles = document.querySelectorAll('[data-testid^="viz-node-"]');
    expect(circles.length).toBe(120);
  });

  it('slider change updates node count meta', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    const slider = screen.getByTestId('viz-node-count') as HTMLInputElement;
    fireEvent.change(slider, { target: { value: '50' } });
    // 50 < 100 → tree mode → dagre
    const algo = screen.getByTestId('viz-algorithm');
    expect(algo.textContent).toMatch(/algo:.*dagre/);
    const circles = document.querySelectorAll('[data-testid^="viz-node-"]');
    expect(circles.length).toBe(50);
  });

  it('view mode change to agent with 120 nodes → elk (>50 threshold)', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    const select = screen.getByTestId('viz-view-mode') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'agent' } });
    // 120 > 50 → agent → elk
    const algo = screen.getByTestId('viz-algorithm');
    expect(algo.textContent).toMatch(/algo:.*elk/);
  });

  it('view mode change to dependency/risk/history with any count → d3_force', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    const select = screen.getByTestId('viz-view-mode') as HTMLSelectElement;
    for (const mode of ['dependency', 'risk', 'history']) {
      fireEvent.change(select, { target: { value: mode } });
      const algo = screen.getByTestId('viz-algorithm');
      expect(algo.textContent).toMatch(/algo:.*d3_force/);
    }
  });

  it('hook ready/version shown (JS fallback default)', () => {
    render(
      <MemoryRouter>
        <WorktreeVizPage />
      </MemoryRouter>,
    );
    const algo = screen.getByTestId('viz-algorithm');
    // WASM binary not in jsdom env → ready=false, version=js-fallback
    expect(algo.textContent).toMatch(/WASM: layout=N/);
  });
});