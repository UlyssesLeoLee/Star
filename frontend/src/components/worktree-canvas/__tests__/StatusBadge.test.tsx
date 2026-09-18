import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBadge } from "../StatusBadge";

describe("StatusBadge", () => {
  it("renders all 7 human states", () => {
    const states = ["RUNNING", "WAITING", "READY", "DIVERGED", "CONFLICT", "MERGED", "STALE"] as const;
    for (const s of states) {
      const { unmount } = render(<StatusBadge state={s} />);
      expect(screen.getByTestId(`status-badge-${s}`)).toBeInTheDocument();
      unmount();
    }
  });
});
