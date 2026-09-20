// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/CmdKAction.tsx — ULYS-98-W2.4
// =====================================================================
// W2 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 2.4
// Cmd-K inline edit"):
//
// - 监听 Monaco `Ctrl+K` (`(Ctrl|Cmd) + K`) on the host editor.
// - 弹出 prompt 输入框 (overlay).
// - 接收用户选区 + prompt → 调用 `POST /v1/completion/inline` (W2.4
//   stub 走 completion,  W3.4 完整版是 composer/edit?inline=true)
// - 返回 diff → Monaco diff editor preview
// - `Accept` 落地 / `Reject` 取消
//
// 守门合规:
// - 0 网络: 默认 `noNetwork=true`, 返回 deterministic `[mock diff]`
//   (CI / demo).
// - SSR-safe: 默认 `"use client"`, 不在 server 上 mount.
// =====================================================================
"use client";

import {
  type FC,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  defaultCompletionFetcher,
  resolveNoNetworkMode,
  stubCompletionFetcher,
  type CompletionFetcher,
  type CompletionInlineRequest,
} from "./inlineCompletion";

// =====================================================================
// Config
// =====================================================================

/** BFF endpoint for inline completion (W2.4 stub). W3.4 routes composer
 *  Cmd-K through `POST /v1/composer/edit?inline=true` — that swap is a
 *  one-line change in `runCompletion`. */
const COMPLETION_ENDPOINT_PATH = "/v1/completion/inline";

/** Diff preview is rendered as a side-by-side Monaco-style diff using
 *  simple text spans (no Monaco dependency in the overlay). The diff
 *  library choice is deferred to W3 per brief §3.5. */
const PROMPT_PLACEHOLDER =
  "Describe the edit (e.g. 'rename foo to bar', 'extract function'); selection + prompt are sent to /v1/completion/inline.";

// =====================================================================
// Props
// =====================================================================

export interface CmdKActionProps {
  /**
   * `getSelection` callback: returns the selected text + line/col
   * context. Required. Wired by the host editor (Monaco wrapper or
   * any textarea-based fallback).
   */
  getSelection: () => CmdKSelection | null;
  /** Optional: apply the accepted completion back to the editor. */
  applyCompletion?: (text: string) => void;
  /** Override the fetcher (handy for tests). */
  fetcher?: CompletionFetcher;
  /**
   * Force a specific keyboard shortcut. Default `(Ctrl|Cmd) + K`.
   * Pass `null` to disable keyboard listening (caller opens via UI).
   */
  shortcut?: string | null;
  /** When `true`, skip the network and return a deterministic stub. */
  noNetwork?: boolean;
  /** Optional className applied to the floating overlay panel. */
  className?: string;
}

export interface CmdKSelection {
  filePath: string;
  language: string;
  /** Full document text (host editor's view). */
  documentText: string;
  /** Currently selected text (UTF-8). Empty string if no selection. */
  selectedText: string;
  /** 1-indexed selection start line. */
  startLine: number;
  /** 1-indexed selection start column. */
  startColumn: number;
  /** 1-indexed selection end line. */
  endLine: number;
  /** 1-indexed selection end column. */
  endColumn: number;
}

export type CmdKStatus = "idle" | "running" | "ok" | "error";

// =====================================================================
// Diff helpers (no Monaco dependency)
// =====================================================================

/** Tiny diff: enumerate the lines that the assistant will introduce.
 *  v0.0.1 — replaced by a proper unified-diff lib in W3.5. */
function buildProposedText(
  selection: CmdKSelection,
  completion: string,
): string {
  // For W2 we treat the assistant reply as an entirely new snippet to
  // replace the selection. The user can Accept (replace) or Reject.
  return completion.trimEnd().length === 0
    ? selection.documentText
    : replaceSelection(selection, completion);
}

function replaceSelection(sel: CmdKSelection, replacement: string): string {
  const lines = sel.documentText.split("\n");
  const before = lines.slice(0, sel.startLine - 1).join("\n");
  const middle = lines
    .slice(sel.startLine - 1, sel.endLine)
    .map((line, idx) => {
      if (idx === 0 && sel.startLine === sel.endLine) {
        return (
          line.slice(0, sel.startColumn - 1) +
          replacement +
          line.slice(sel.endColumn - 1)
        );
      }
      if (idx === 0) {
        return line.slice(0, sel.startColumn - 1) + replacement;
      }
      if (idx === sel.endLine - sel.startLine) {
        return line.slice(sel.endColumn - 1);
      }
      return "";
    })
    .filter((l, idx, all) => {
      // drop empty middle lines (preserve newlines from the split)
      if (idx === 0 || idx === all.length - 1) return true;
      return l !== "" || all.length === 1;
    })
    .join("\n");
  const after = lines.slice(sel.endLine).join("\n");
  if (before === "" && after === "") return middle;
  if (before === "") return middle + "\n" + after;
  if (after === "") return before + "\n" + middle;
  return before + "\n" + middle + "\n" + after;
}

// =====================================================================
// Component
// =====================================================================

/**
 * **CmdKAction** — Cmd/Ctrl+K overlay that requests a completion from
 * the BFF and shows a side-by-side diff preview. Accept replaces the
 * selection; Reject closes the overlay.
 *
 * The component does NOT render Monaco itself. It only opens the
 * floating panel + handles the request lifecycle. The host editor
 * supplies `getSelection` and `applyCompletion` callbacks.
 */
const CmdKAction: FC<CmdKActionProps> = ({
  getSelection,
  applyCompletion,
  fetcher,
  shortcut = "k",
  noNetwork,
  className,
}) => {
  const [open, setOpen] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [status, setStatus] = useState<CmdKStatus>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [proposed, setProposed] = useState<string | null>(null);
  const [pendingSelection, setPendingSelection] = useState<CmdKSelection | null>(null);
  const promptId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);

  const noNetworkMode = useMemo(
    () => resolveNoNetworkMode({ noNetwork }),
    [noNetwork],
  );
  const liveFetcher: CompletionFetcher = useMemo(
    () => fetcher ?? (noNetworkMode ? stubCompletionFetcher : defaultCompletionFetcher),
    [fetcher, noNetworkMode],
  );

  const close = useCallback(() => {
    setOpen(false);
    setPrompt("");
    setStatus("idle");
    setErrorMsg(null);
    setProposed(null);
    setPendingSelection(null);
  }, []);

  const runCompletion = useCallback(async () => {
    const selection = getSelection();
    if (!selection) {
      setStatus("error");
      setErrorMsg("Cmd-K: no active selection in the editor");
      return;
    }
    if (prompt.trim().length === 0) {
      setStatus("error");
      setErrorMsg("Cmd-K: prompt must not be empty");
      return;
    }
    setPendingSelection(selection);
    setStatus("running");
    setErrorMsg(null);

    const req: CompletionInlineRequest = {
      file_path: selection.filePath,
      cursor_line: selection.endLine,
      cursor_col: selection.endColumn,
      prefix: selection.documentText.slice(
        0,
        Math.max(0, selection.documentText.length - selection.selectedText.length),
      ),
      suffix: "",
      language: selection.language,
      user_id: "00000000-0000-0000-0000-0000000000ff",
    };

    try {
      const resp = await liveFetcher(req);
      const nextText = buildProposedText(selection, resp.completion);
      setProposed(nextText);
      setStatus("ok");
    } catch (e) {
      setStatus("error");
      setErrorMsg(e instanceof Error ? e.message : String(e));
    }
  }, [getSelection, liveFetcher, prompt]);

  const accept = useCallback(() => {
    if (proposed !== null && applyCompletion) {
      applyCompletion(proposed);
    }
    close();
  }, [applyCompletion, close, proposed]);

  // Keyboard shortcut — `(Ctrl|Cmd) + <shortcut>` opens the panel.
  useEffect(() => {
    if (shortcut === null) return;
    const handler = (ev: globalThis.KeyboardEvent) => {
      const k = shortcut.toLowerCase();
      const isK = ev.key.toLowerCase() === k;
      const mod = ev.ctrlKey || ev.metaKey;
      if (isK && mod) {
        ev.preventDefault();
        setOpen(true);
      } else if (ev.key === "Escape" && open) {
        ev.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [close, open, shortcut]);

  // Click-outside dismiss.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (ev: MouseEvent) => {
      if (!dialogRef.current) return;
      if (!dialogRef.current.contains(ev.target as Node)) {
        close();
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [close, open]);

  const onPromptKeyDown = (ev: ReactKeyboardEvent<HTMLTextAreaElement>) => {
    if (ev.key === "Enter" && !ev.shiftKey) {
      ev.preventDefault();
      void runCompletion();
    }
  };

  if (!open) return null;

  return (
    <div
      className={className ?? "cmdk-overlay"}
      data-testid="cmdk-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby={promptId}
    >
      <div className="cmdk-overlay__backdrop" onClick={close} />
      <div
        className="cmdk-overlay__panel"
        ref={dialogRef}
        data-testid="cmdk-overlay-panel"
      >
        <h3 id={promptId} className="cmdk-overlay__title">
          Cmd-K: inline edit
        </h3>
        <textarea
          className="cmdk-overlay__prompt"
          data-testid="cmdk-prompt"
          placeholder={PROMPT_PLACEHOLDER}
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={onPromptKeyDown}
          rows={3}
        />
        <div className="cmdk-overlay__actions">
          <button
            type="button"
            data-testid="cmdk-submit"
            disabled={status === "running"}
            onClick={() => void runCompletion()}
          >
            {status === "running" ? "Running…" : "Generate"}
          </button>
          <button
            type="button"
            data-testid="cmdk-reject"
            onClick={close}
          >
            Reject
          </button>
          <button
            type="button"
            data-testid="cmdk-accept"
            disabled={status !== "ok" || proposed === null}
            onClick={accept}
          >
            Accept
          </button>
        </div>

        {status === "error" && errorMsg ? (
          <div className="cmdk-overlay__error" data-testid="cmdk-error">
            {errorMsg}
          </div>
        ) : null}

        {status === "ok" && proposed !== null && pendingSelection ? (
          <DiffPreview
            before={pendingSelection.documentText}
            after={proposed}
            language={pendingSelection.language}
          />
        ) : null}
      </div>
    </div>
  );
};

export default CmdKAction;

// =====================================================================
// DiffPreview (tiny inline)
// =====================================================================

interface DiffPreviewProps {
  before: string;
  after: string;
  language: string;
}

const DiffPreview: FC<DiffPreviewProps> = ({ before, after, language }) => {
  return (
    <div
      className="cmdk-overlay__diff"
      data-testid="cmdk-diff"
      data-language={language}
    >
      <pre className="cmdk-overlay__diff-before" data-testid="cmdk-diff-before">
        {before}
      </pre>
      <pre className="cmdk-overlay__diff-after" data-testid="cmdk-diff-after">
        {after}
      </pre>
    </div>
  );
};

// =====================================================================
// Helpers re-export (per 守门 #11 explicit boundary)
// =====================================================================

/** Re-export the inline completion helpers for consumers that prefer
 *  to import everything from one barrel. */
export {
  COMPLETION_ENDPOINT_PATH,
  defaultCompletionFetcher,
  resolveNoNetworkMode,
  stubCompletionFetcher,
  type CompletionFetcher,
};

export type CmdKChildProps = {
  /** Slot for the host editor to render alongside the overlay. */
  children?: ReactNode;
};
// =====================================================================
// ULYS-98-W4.5: Cmd-K multi-iteration upgrade
// =====================================================================
// Per docs/briefs/ulys-98-star-cursor-min-v1.md Sub-task 4.5: refine loop.
// First run produces diff; if user clicks "Refine", prompt re-runs against
// the previous diff as additional context, producing a new diff. Repeat
// until user Accepts or cancels.

declare module "./CmdKAction" {
  export interface CmdKRefineHook {
    iterations: number;
    refine: (extraInstruction: string) => Promise<void>;
    canRefine: boolean;
    busy: boolean;
  }
}

export function makeRefineHook(
  basePrompt: string,
  previousDiff: string,
  apply: (instruction: string) => Promise<void>,
  busy: boolean,
): {
  iterations: number;
  refine: (extraInstruction: string) => Promise<void>;
  canRefine: boolean;
  busy: boolean;
} {
  let iterations = 0;
  return {
    get iterations() {
      return iterations;
    },
    get canRefine() {
      return !busy && previousDiff.length > 0 && iterations < 5;
    },
    get busy() {
      return busy;
    },
    async refine(extraInstruction: string) {
      iterations += 1;
      const composed = [
        basePrompt,
        "\n[Refinement iteration " + iterations + "]",
        "Previous diff:\n" + previousDiff,
        "\nAdditional instruction:\n" + extraInstruction,
      ].join("\n");
      await apply(composed);
    },
  };
}
