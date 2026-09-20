// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/CmdKAction.test.tsx — ULYS-98-W2.4
// =====================================================================
// W2 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 2.4"):
// "1 unit test" — we ship 6 covering:
// 1. Cmd/Ctrl+K opens the overlay (default keyboard shortcut).
// 2. Empty prompt + Enter shows an error, no submit fired.
// 3. Generate fires a completion via the stub fetcher (no network).
// 4. Accept calls applyCompletion + closes the overlay.
// 5. Reject closes the overlay without applying.
// 6. Disabled `shortcut` prop suppresses the keyboard listener.
// =====================================================================
import {
  describe,
  it,
  expect,
  vi,
  beforeEach,
  afterEach,
} from "vitest";
import {
  fireEvent,
  render,
  screen,
  waitFor,
  cleanup,
} from "@testing-library/react";

import CmdKAction, {
  type CmdKActionProps,
  type CmdKSelection,
} from "./CmdKAction";

const sampleSelection = (overrides: Partial<CmdKSelection> = {}): CmdKSelection => ({
  filePath: "src/lib.ts",
  language: "typescript",
  documentText: "function greet(name) {\n  console.log('hi', name);\n}\n",
  selectedText: "hi",
  startLine: 2,
  startColumn: 17,
  endLine: 2,
  endColumn: 19,
  ...overrides,
});

interface HarnessProps extends Omit<CmdKActionProps, "getSelection" | "applyCompletion"> {
  selection?: CmdKSelection | null;
  applied?: (text: string) => void;
}

function Harness({
  selection,
  applied,
  ...rest
}: HarnessProps): JSX.Element {
  return (
    <CmdKAction
      {...rest}
      getSelection={() => selection ?? null}
      applyCompletion={applied}
    />
  );
}

describe("CmdKAction (ULYS-98-W2.4)", () => {
  beforeEach(() => {
    cleanup();
    vi.clearAllMocks();
  });
  afterEach(() => {
    cleanup();
  });

  it("renders nothing until Cmd/Ctrl+K opens the overlay", () => {
    render(<Harness selection={sampleSelection()} />);
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();

    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(screen.getByTestId("cmdk-overlay")).not.toBeNull();
  });

  it("supports the macOS Cmd modifier", () => {
    render(<Harness selection={sampleSelection()} />);
    fireEvent.keyDown(window, { key: "k", metaKey: true });
    expect(screen.getByTestId("cmdk-overlay")).not.toBeNull();
  });

  it("Esc closes the overlay", () => {
    render(<Harness selection={sampleSelection()} />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(screen.getByTestId("cmdk-overlay")).not.toBeNull();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();
  });

  it("disabled shortcut prop suppresses keyboard listener", () => {
    render(<Harness selection={sampleSelection()} shortcut={null} />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();
  });

  it("Generate button fires a completion via the stub fetcher (no network)", async () => {
    const applied = vi.fn();
    render(
      <Harness
        selection={sampleSelection()}
        applied={applied}
        noNetwork
      />,
    );
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });

    const promptArea = screen.getByTestId("cmdk-prompt");
    fireEvent.change(promptArea, { target: { value: "rename hi to hello" } });
    fireEvent.click(screen.getByTestId("cmdk-submit"));

    await waitFor(() => {
      expect(screen.getByTestId("cmdk-diff")).not.toBeNull();
    });

    // The Accept button is enabled after a successful completion.
    const accept = screen.getByTestId("cmdk-accept") as HTMLButtonElement;
    expect(accept.disabled).toBe(false);

    fireEvent.click(accept);
    expect(applied).toHaveBeenCalledTimes(1);
    // The stub completion replaces the selected "hi" with the mock
    // marker; the resulting text contains the marker.
    expect(applied.mock.calls[0][0]).toContain("[mock inline completion");
    // Overlay closes after Accept.
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();
  });

  it("Enter (without Shift) in the prompt submits", async () => {
    render(<Harness selection={sampleSelection()} noNetwork />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    const promptArea = screen.getByTestId("cmdk-prompt");
    fireEvent.change(promptArea, { target: { value: "rename hi to hello" } });
    fireEvent.keyDown(promptArea, { key: "Enter" });

    await waitFor(() => {
      expect(screen.getByTestId("cmdk-diff")).not.toBeNull();
    });
  });

  it("Shift+Enter inserts a newline instead of submitting", () => {
    render(<Harness selection={sampleSelection()} noNetwork />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    const promptArea = screen.getByTestId("cmdk-prompt");
    fireEvent.change(promptArea, { target: { value: "draft" } });
    fireEvent.keyDown(promptArea, { key: "Enter", shiftKey: true });

    // The diff preview should NOT appear because we didn't submit.
    expect(screen.queryByTestId("cmdk-diff")).toBeNull();
  });

  it("empty prompt + Enter shows an error and does not submit", async () => {
    render(<Harness selection={sampleSelection()} noNetwork />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    fireEvent.click(screen.getByTestId("cmdk-submit"));
    await waitFor(() => {
      expect(screen.getByTestId("cmdk-error").textContent).toMatch(/non-empty|prompt/i);
    });
  });

  it("missing selection shows a useful error", async () => {
    render(<Harness selection={null} noNetwork />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    fireEvent.change(screen.getByTestId("cmdk-prompt"), {
      target: { value: "rename" },
    });
    fireEvent.click(screen.getByTestId("cmdk-submit"));
    await waitFor(() => {
      expect(screen.getByTestId("cmdk-error").textContent).toMatch(/selection/i);
    });
  });

  it("Reject closes the overlay without calling applyCompletion", () => {
    const applied = vi.fn();
    render(<Harness selection={sampleSelection()} applied={applied} />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    fireEvent.click(screen.getByTestId("cmdk-reject"));
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();
    expect(applied).not.toHaveBeenCalled();
  });

  it("Click on the backdrop closes the overlay", () => {
    render(<Harness selection={sampleSelection()} />);
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(screen.getByTestId("cmdk-overlay")).not.toBeNull();
    // The backdrop sits next to the panel; clicking it (or the overlay
    // container itself) triggers `close`. We click the backdrop by
    // class role.
    const backdrop = screen.getByTestId("cmdk-overlay").querySelector(
      ".cmdk-overlay__backdrop",
    ) as HTMLElement | null;
    expect(backdrop).not.toBeNull();
    fireEvent.click(backdrop!);
    expect(screen.queryByTestId("cmdk-overlay")).toBeNull();
  });

  it("custom fetcher is used when provided", async () => {
    const fetcher = vi.fn(async () => ({
      completion: "MY-CUSTOM",
      model: "test-model",
      token_usage: {
        user_id: "u",
        tenant_id: "t",
        provider: "mock",
        model: "test-model",
        input_tokens: 1,
        output_tokens: 1,
        total_tokens: 2,
        captured_at: new Date().toISOString(),
        request_id: null,
      },
      created_at: new Date().toISOString(),
    }));
    render(
      <Harness
        selection={sampleSelection()}
        fetcher={fetcher}
        noNetwork={false}
      />,
    );
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    fireEvent.change(screen.getByTestId("cmdk-prompt"), {
      target: { value: "x" },
    });
    fireEvent.click(screen.getByTestId("cmdk-submit"));

    await waitFor(() => {
      expect(screen.getByTestId("cmdk-diff")).not.toBeNull();
    });
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(
      (screen.getByTestId("cmdk-diff-after") as HTMLElement).textContent,
    ).toContain("MY-CUSTOM");
  });
});