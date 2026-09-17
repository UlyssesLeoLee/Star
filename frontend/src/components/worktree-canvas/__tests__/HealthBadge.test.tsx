import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { HealthBadge } from "../HealthBadge";

describe("HealthBadge", () => {
  it("green for score >= 80", () => {
    render(<HealthBadge score={95} />);
    expect(screen.getByTestId("health-badge")).toHaveTextContent("H 95");
    expect(screen.getByTestId("health-badge").className).toMatch(/green/);
  });
  it("amber for 60-79", () => {
    render(<HealthBadge score={65} />);
    expect(screen.getByTestId("health-badge").className).toMatch(/amber/);
  });
  it("red for < 60", () => {
    render(<HealthBadge score={30} />);
    expect(screen.getByTestId("health-badge").className).toMatch(/red/);
  });
});
