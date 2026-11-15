/*
@cypher schema=1 source_sha256=284b9d15b6baff703f134aa110e79cca63a39b7822be8ea784da23326304cd27
MERGE (self:File {path:"frontend/src/components/team/TeamPanel.test.tsx"})
@endcypher
*/

// =====================================================================
// TeamPanel unit tests
//
// 验收:
//   - panel root + store seed runtime rows (lr-001..lr-005) 渲染
//   - squad 预览行 (sq-001..sq-003) 渲染
//   - 全部动作按钮 disabled + title 原因 (未接入)
//   - 每行行尾 "未接入" 徽标 (不假装可用)
// =====================================================================

import { describe, expect, it } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { TeamPanel } from "./TeamPanel";

describe("TeamPanel", () => {
  it("renders the panel root and runtime rows from the store", () => {
    render(<TeamPanel />);

    expect(screen.getByTestId("team-panel")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-001")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-002")).toBeTruthy();
    expect(screen.getByTestId("team-runtime-row-lr-005")).toBeTruthy();
  });

  it("renders squad preview rows (Worker and Router)", () => {
    render(<TeamPanel />);

    expect(screen.getByTestId("team-squad-row-sq-001")).toBeTruthy();
    expect(screen.getByTestId("team-squad-row-sq-002")).toBeTruthy();
    expect(screen.getByTestId("team-squad-row-sq-003")).toBeTruthy();
  });

  it("disables every runtime action button and exposes the reason via title", () => {
    render(<TeamPanel />);

    for (const action of ["start", "stop", "restart"] as const) {
      const button = screen.getByTestId(`team-action-${action}-lr-001`) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      expect(button.title).toContain("未接入");
    }
  });

  it("disables every squad action button and exposes the reason via title", () => {
    render(<TeamPanel />);

    for (const action of ["assign", "recall"] as const) {
      const button = screen.getByTestId(`team-action-${action}-sq-001`) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      expect(button.title).toContain("未接入");
    }
  });

  it("marks each row with an explicit 未接入 badge", () => {
    render(<TeamPanel />);

    for (const rowId of [
      "team-runtime-row-lr-001",
      "team-runtime-row-lr-005",
      "team-squad-row-sq-001",
      "team-squad-row-sq-003",
    ]) {
      const row = screen.getByTestId(rowId);
      expect(within(row).getByText("未接入")).toBeTruthy();
    }
  });
});
