/*
@cypher schema=1 source_sha256=96bcf597cb86dfe7d6745cae96b41708cb66d7e444a5ab87f2a167aaa5e684a1
MERGE (self:File {path:"frontend/src/components/effects/__tests__/TacticalCore3D.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/effects/__tests__/TacticalCore3D.test.tsx::TacticalCore3D",kind:"test_suite"})
MERGE (core:Symbol {id:"frontend/src/components/effects/TacticalCore3D.tsx::TacticalCore3D",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(core)
@endcypher
*/
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TacticalCore3D } from "../TacticalCore3D";

describe("TacticalCore3D", () => {
  it("renders tactical core with role=button and aria-label", () => {
    const { unmount } = render(<TacticalCore3D status="nominal" size={32} />);

    const core = screen.getByRole("button", { name: "Tactical Neural Core" });
    expect(core).toBeInTheDocument();
    expect(core.getAttribute("title")).toBe("Tactical Neural Core [3渲2 真3D 三轴陀螺仪]");
    unmount();
  });

  it("handles mouse hover and click interactions", () => {
    const { unmount } = render(<TacticalCore3D status="warning" size={40} />);

    const core = screen.getByRole("button", { name: "Tactical Neural Core" });
    fireEvent.mouseEnter(core);
    fireEvent.click(core);
    fireEvent.mouseLeave(core);

    expect(core).toBeInTheDocument();
    unmount();
  });
});
