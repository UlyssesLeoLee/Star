import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConfirmationModal } from "../ConfirmationModal";

describe("ConfirmationModal", () => {
  it("does not render for non-Destructive action", () => {
    const { container } = render(
      <ConfirmationModal action="Lock" worktreeId="wt-1" onClose={vi.fn()} onConfirm={vi.fn()} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders modal for Destructive action", () => {
    render(
      <ConfirmationModal action="Merge" worktreeId="wt-merge" onClose={vi.fn()} onConfirm={vi.fn()} />
    );
    expect(screen.getByTestId("confirmation-modal")).toBeInTheDocument();
    expect(screen.getByTestId("confirmation-modal").getAttribute("data-action")).toBe("Merge");
  });

  it("calls onConfirm when confirm button clicked", () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmationModal action="Delete" worktreeId="wt-1" onClose={vi.fn()} onConfirm={onConfirm} />
    );
    fireEvent.click(screen.getByTestId("confirm-button"));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("calls onClose when backdrop clicked", () => {
    const onClose = vi.fn();
    render(
      <ConfirmationModal action="Merge" worktreeId="wt-1" onClose={onClose} onConfirm={vi.fn()} />
    );
    fireEvent.click(screen.getByTestId("confirmation-modal"));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
