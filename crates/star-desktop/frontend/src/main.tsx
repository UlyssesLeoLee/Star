// crates/star-desktop/frontend/src/main.tsx
// =====================================================================
// Star Desktop — Tauri 2.0 + React 19 entry point (PR-246 follow-up of PR #239)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5
//
// Phase 1: React 19 + React Router 7 skeleton
//   - Root App 组件 + 2 routes (/ + /worktree)
//   - 后续 PR-247+ 替换 main.ts → /board route + KanbanBoard.tsx 复用
// =====================================================================
import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import App from './App';
import HomePage from './pages/HomePage';
import WorktreePage from './pages/WorktreePage';
import WorktreeVizPage from './pages/WorktreeVizPage';

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'worktree', element: <WorktreePage /> },
      { path: 'viz', element: <WorktreeVizPage /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
