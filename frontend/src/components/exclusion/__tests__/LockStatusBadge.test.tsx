/**
 * UT-04: LockStatusBadge 角标状态
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { LockStatusBadge } from "../LockStatusBadge";

describe("LockStatusBadge", () => {
  it("renders with green color when no waiting or timeout", () => {
    const { container } = render(<LockStatusBadge stats={{ held: 3, waiting: 0, timeout: 0 }} />);
    const badge = container.querySelector(".lock-status-badge");
    expect(badge?.className).toContain("green");
    expect(screen.getByText(/3 持锁/)).toBeInTheDocument();
  });

  it("renders with yellow color when waiting > 0", () => {
    const { container } = render(<LockStatusBadge stats={{ held: 1, waiting: 2, timeout: 0 }} />);
    const badge = container.querySelector(".lock-status-badge");
    expect(badge?.className).toContain("yellow");
  });

  it("renders with red color when timeout > 0", () => {
    const { container } = render(<LockStatusBadge stats={{ held: 1, waiting: 0, timeout: 1 }} />);
    const badge = container.querySelector(".lock-status-badge");
    expect(badge?.className).toContain("red");
  });

  it("renders total count correctly", () => {
    render(<LockStatusBadge stats={{ held: 1, waiting: 2, timeout: 3 }} />);
    expect(screen.getByText(/6 总/)).toBeInTheDocument();
  });
});
