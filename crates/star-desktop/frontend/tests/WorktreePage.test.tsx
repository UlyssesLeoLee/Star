// crates/star-desktop/frontend/tests/WorktreePage.test.tsx
// =====================================================================
// Vitest test — WorktreePage renders 3 mock worktree groups
// PR-246 placeholder test (PR-247+ 替换为 Tauri IPC invoke)
// =====================================================================
/*
@cypher schema=1 source_sha256=e432d6a3446c40a16239dd7fa0b7626c7ce358a1911f97be6f81592e4b7421a6
MERGE (self:File {path:"crates/star-desktop/frontend/tests/WorktreePage.test.tsx"})
MERGE (page:File {path:"crates/star-desktop/frontend/src/pages/WorktreePage.tsx"})
MERGE (self)-[:TESTS]->(page)
@endcypher
*/

import { describe, it, expect } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import WorktreePage from '../src/pages/WorktreePage';

describe('WorktreePage', () => {
  it('renders the current board-backed worktree view', async () => {
    render(<WorktreePage />);
    expect(screen.getByText('Worktree Board (BoardView — PR #252)')).toBeDefined();
    await waitFor(() => {
      expect(screen.getByTestId('board-view')).toBeDefined();
    });
    expect(screen.getByTestId('kanban-board')).toBeDefined();
  });
});
