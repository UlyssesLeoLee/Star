import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { RiskBadge } from "../RiskBadge";

describe("RiskBadge", () => {
  it("renders muted R 0 for zero count", () => {
    render(<RiskBadge count={0} />);
    expect(screen.getByTestId("risk-badge")).toHaveTextContent("R 0");
  });
  it("amber for low count", () => {
    render(<RiskBadge count={2} />);
    expect(screen.getByTestId("risk-badge").className).toMatch(/amber/);
  });
  it("red for high count", () => {
    render(<RiskBadge count={5} />);
    expect(screen.getByTestId("risk-badge").className).toMatch(/red/);
  });
});
