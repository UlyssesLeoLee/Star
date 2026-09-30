// crates/star-desktop/frontend/src/App.tsx
// =====================================================================
// Star Desktop — App 根组件 (PR-246)
// React Router Outlet 渲染当前路由子页面
// 头部 + 底栏聊天可选 + WorktreeIndex 导航 (per PR-224 AGENTS.md 硬约束)
// =====================================================================
import { Outlet } from 'react-router-dom';
import styles from './App.module.css';

export default function App() {
  return (
    <div className={styles.app}>
      <header className={styles.header}>
        <h1 className={styles.title}>🪐 Star Desktop (Tauri 2.0 + React 19)</h1>
        <nav className={styles.nav}>
          <a href="/">Home</a>
          <a href="/worktree">Worktree</a>
        </nav>
      </header>
      <main className={styles.main}>
        <Outlet />
      </main>
      <footer className={styles.footer}>
        <span>Scope: Global</span>
        <span>Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5</span>
      </footer>
    </div>
  );
}
