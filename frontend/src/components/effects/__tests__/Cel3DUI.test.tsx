import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import {
  CelButton3D,
  CelToggle3D,
  CelBeacon3D,
  CelDial3D,
  CelCrystal3D,
  CelHoloCube3D,
  CelOrbitalGauge3D,
} from "../Cel3DUI";

describe("Cel3DUI real 3D components", () => {
  afterEach(() => {
    cleanup();
    document.documentElement.classList.remove("dark", "light", "physis");
  });

  it("renders CelButton3D and fires onClick", () => {
    const onClick = vi.fn();
    render(<CelButton3D label="EXECUTE" sublabel="RUN" variant="cyan" onClick={onClick} />);
    const btn = screen.getByRole("button", { name: /EXECUTE/i });
    fireEvent.mouseDown(btn);
    fireEvent.mouseUp(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("renders CelToggle3D and toggles checked state", () => {
    const onChange = vi.fn();
    render(<CelToggle3D checked={false} onChange={onChange} label="SAGA GUARD" />);
    const toggle = screen.getByRole("switch", { name: "SAGA GUARD" });
    expect(toggle).toHaveAttribute("aria-checked", "false");
    fireEvent.click(toggle);
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("renders CelBeacon3D with status role and testid", () => {
    render(<CelBeacon3D status="active" size={32} title="Live Link" data-testid="beacon-1" />);
    const beacon = screen.getByTestId("beacon-1");
    expect(beacon).toHaveAttribute("role", "status");
    expect(beacon).toHaveAttribute("aria-label", "Live Link");
  });

  it("renders CelDial3D and responds to keyboard navigation", () => {
    const onChange = vi.fn();
    render(<CelDial3D value={0} onChange={onChange} options={["BALANCED", "OVERCLOCK", "SURVIVAL"]} />);
    const slider = screen.getByRole("slider");
    fireEvent.keyDown(slider, { key: "ArrowRight" });
    expect(onChange).toHaveBeenCalledWith(1);
  });

  it("renders CelCrystal3D and handles click / keyboard burst", () => {
    document.documentElement.classList.add("dark");
    render(<CelCrystal3D tone="cyan" size={44} label="Tasks" value={12} data-testid="crystal-1" />);
    const crystal = screen.getByTestId("crystal-1");
    expect(crystal).toHaveAttribute("role", "button");
    expect(crystal).toHaveTextContent("12");
    fireEvent.mouseEnter(crystal);
    fireEvent.click(crystal);
    fireEvent.keyDown(crystal, { key: "Enter" });
    fireEvent.mouseLeave(crystal);
    expect(crystal).toBeInTheDocument();
  });

  it("renders CelHoloCube3D and handles theme switch mutation", () => {
    render(<CelHoloCube3D status="optimal" size={40} title="Worktree Cube" data-testid="holocube-1" />);
    const cube = screen.getByTestId("holocube-1");
    expect(cube).toHaveAttribute("aria-label", "Worktree Cube");
    document.documentElement.classList.add("dark");
    fireEvent.click(cube);
    expect(cube).toBeInTheDocument();
  });

  it("renders CelOrbitalGauge3D with clamped percentage label", () => {
    render(<CelOrbitalGauge3D progress={76} tone="ok" size={44} title="Completion" data-testid="gauge-1" />);
    const gauge = screen.getByTestId("gauge-1");
    expect(gauge).toHaveAttribute("role", "img");
    expect(gauge).toHaveAttribute("aria-label", "Completion: 76%");
  });
});
