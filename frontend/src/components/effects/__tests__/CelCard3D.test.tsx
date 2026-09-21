import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CelCard3D } from "../CelCard3D";

describe("CelCard3D", () => {
  it("renders children content with 3D perspective wrapper", () => {
    const { unmount } = render(
      <CelCard3D data-testid="cel-card-1">
        <span data-testid="child-content">Tactical Card Info</span>
      </CelCard3D>
    );

    const card = screen.getByTestId("cel-card-1");
    const child = screen.getByTestId("child-content");

    expect(card).toBeInTheDocument();
    expect(child).toBeInTheDocument();
    expect(child.textContent).toBe("Tactical Card Info");
    expect(card.style.perspective).toBe("1100px");
    unmount();
  });

  it("applies tilt calculation and glint on mouse movement", () => {
    const { unmount } = render(
      <CelCard3D data-testid="cel-card-2" maxTilt={10} glint={true} chamfer={true}>
        <div>Card Body</div>
      </CelCard3D>
    );

    const card = screen.getByTestId("cel-card-2");

    fireEvent.mouseEnter(card);
    fireEvent.mouseMove(card, { clientX: 100, clientY: 50 });

    const inner = card.firstElementChild as HTMLElement;
    expect(inner).toBeInTheDocument();
    expect(inner.style.transformStyle).toBe("preserve-3d");

    fireEvent.mouseLeave(card);
    expect(inner.style.transform).toBe("rotateX(0deg) rotateY(0deg) translateZ(0px)");
    unmount();
  });

  it("applies custom elevation classes correctly", () => {
    const { rerender, unmount } = render(
      <CelCard3D data-testid="cel-card-3" elevation="lg">
        <div>High Elevation</div>
      </CelCard3D>
    );

    const innerLg = (screen.getByTestId("cel-card-3").firstElementChild as HTMLElement);
    expect(innerLg.className).toContain("6px_6px_0px_0px");

    rerender(
      <CelCard3D data-testid="cel-card-3" elevation="sm">
        <div>Low Elevation</div>
      </CelCard3D>
    );
    const innerSm = (screen.getByTestId("cel-card-3").firstElementChild as HTMLElement);
    expect(innerSm.className).toContain("2px_2px_0px_0px");
    unmount();
  });
});
