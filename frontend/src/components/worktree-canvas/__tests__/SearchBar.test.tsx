import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SearchBar } from "../SearchBar";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

describe("SearchBar", () => {
  beforeEach(() => {
    useWorktreeCanvasStore.setState({ searchQuery: "" });
  });
  it("renders input and placeholder", () => {
    render(<SearchBar />);
    expect(screen.getByTestId("search-bar")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/show:conflict/)).toBeInTheDocument();
  });
  it("updates store on input change", () => {
    render(<SearchBar />);
    fireEvent.change(screen.getByTestId("search-bar"), { target: { value: "show:conflict" } });
    expect(useWorktreeCanvasStore.getState().searchQuery).toBe("show:conflict");
  });
  it("clear button appears when query non-empty", () => {
    useWorktreeCanvasStore.setState({ searchQuery: "x" });
    render(<SearchBar />);
    expect(screen.getByLabelText("清除")).toBeInTheDocument();
  });
});
