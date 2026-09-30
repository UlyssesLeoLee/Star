// crates/star-desktop/frontend/src/pages/WorktreePage.tsx
// =====================================================================
// Star Desktop — WorktreePage (PR-252 follow-up of PR #246)
// 集成 BoardView 替换 mock 表格 (per PR #244 §3.5)
// =====================================================================
import { BoardView } from "../components/BoardView";

export default function WorktreePage() {
  return (
    <div>
      <h2>Worktree Board (BoardView — PR #252)</h2>
      <p>Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5 阶段 5</p>
      <BoardView initialViewMode="tree" initialDsl="" />
    </div>
  );
}
