import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TacticalCore3D } from "../TacticalCore3D";

describe("TacticalCore3D", () => {
  it("renders tactical core with role=button and aria-label", () => {
    const { unmount } = render(<TacticalCore3D status="nominal" size={32} />);

    const core = screen.getByRole("button", { name: "Tactical Neural Core" });
    expect(core).toBeInTheDocument();
    expect(core.getAttribute("title")).toBe("Tactical Neural Core [3渲2 真3D 陀螺仪]");
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
