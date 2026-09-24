"use client";

// =====================================================================
// XtermPane.tsx — Per-pane xterm.js Terminal mount (per ULYS-232 P1-E)
// (per ULYS-200 P1-D + P1-C 集成, PR #98.5 followup)
// =====================================================================
// **目的**: 真实 mount @xterm/xterm container for each pane; 接 wsClient onOutput
// writes to xterm terminal; xterm onData feeds back to wsClient.sendStdin.
// **MVP v0**: 只支持 single xterm container per pane, split tree 多 pane 各自 mount.
// **不在本 MVP 范围**: scrollback 持久化 (per P1-A), resize 调整 (per AC-3).
//
// 守门:
// - Per-pane xterm instance (1 × iframe)
//// - 8 6 单测覆盖 mount/unmount lifecycle
// - 接 wsClient onOutput + onData -> sendStdin
// - WS reconnect 时重 mount
// =====================================================================

import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import type { Terminal } from "@xterm/xterm";
import type { FitAddon } from "@xterm/addon-fit";

export interface XtermPaneProps {
  /** Pane UUID (per SplitTree) */
  paneId: string;
  /** Optional xterm.js options override */
  options?: ConstructorParameters<typeof Terminal>[0];
  /** Handler: xterm receives data from user (per onData) */
  onData?: (data: string) => void;
  /** Handler: xterm receives binary (per onBinary) */
  onBinary?: (data: string) => void;
  /** Handler: pane resized (per onResize) */
  onResize?: (cols: number, rows: number) => void;
}

export interface XtermPaneRef {
  /** Write data to xterm (per wsClient onOutput) */
  write: (data: string) => void;
  /** Write raw bytes (base64-encoded) to xterm (per onBinary) */
  writeBinary: (data: string) => void;
  /** Clear xterm */
  clear: () => void;
  /** Fit to container */
  fit: () => void;
  /** Get current cols/rows */
  cols: () => number;
  rows: () => number;
  /** Check if mounted */
  isReady: () => boolean;
}

/**
 * Per-pane xterm.js mount component (per ULYS-223 P1-D + ULYS-222 P1-C 集成).
 *
 * Usage:
 *   const ref = useRef<XtermPaneRef>(null);
 *   <XtermPane ref={ref} paneId={pane.id} onData={(d) => wsClient.sendStdin(d)} />
 *   // in onOutput handler:
 *   ref.current?.write(data);
 */
export const XtermPane = forwardRef<XtermPaneRef, XtermPaneProps>(
  function XtermPane(props, ref) {
    const { paneId, options, onData, onBinary, onResize } = props;
    const containerRef = useRef<HTMLDivElement | null>(null);
    const terminalRef = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const [isReady, setIsReady] = useState(false);

    // Mount xterm + handle terminal instance + fit addon (per AC-1 AC-2 AC-3)
    useEffect(() => {
      if (!containerRef.current) return;

      let mounted = true;
      let cleanup: (() => void) | null = null;

      // Dynamic import to avoid SSR issues (per P1-D server-safe rendering)
      Promise.all([
        import("@xterm/xterm"),
        import("@xterm/addon-fit"),
      ])
        .then(([termMod, fitMod]) => {
          if (!mounted || !containerRef.current) return;

          const TerminalCtor = termMod.Terminal;
          const FitAddonCtor = fitMod.FitAddon;

          const term = new TerminalCtor({
            fontFamily: "Menlo, Monaco, 'Courier New', monospace",
            fontSize: 13,
            cursorBlink: true,
            theme: {
              background: "#1e1e1e",
              foreground: "#d4d4d4",
              cursor: "#d4d4d4",
            },
            cols: 80,
            rows: 24,
            ...options,
          });
          const fit = new FitAddonCtor();
          term.loadAddon(fit);
          term.open(containerRef.current);
          fit.fit();

          terminalRef.current = term;
          fitAddonRef.current = fit;
          setIsReady(true);

          // Wire xterm event handlers
          const dataSub = term.onData((data) => onData?.(data));
          const binarySub = term.onBinary((data) => onBinary?.(data));
          const resizeSub = term.onResize(({ cols, rows }) =>
            onResize?.(cols, rows),
          );

          cleanup = () => {
            dataSub.dispose();
            binarySub.dispose();
            resizeSub.dispose();
            term.dispose();
            terminalRef.current = null;
            fitAddonRef.current = null;
          };
        })
        .catch((err: unknown) => {
          // eslint-disable-next-line no-console
          console.error(`[XtermPane paneId=${paneId}] failed to load xterm:`, err);
        });

      return () => {
        mounted = false;
        if (cleanup) cleanup();
        setIsReady(false);
      };
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [paneId]);

    // Imperative API for parent to write to xterm (per wsClient.onOutput)
    useImperativeHandle(
      ref,
      () => ({
        write(data: string) {
          terminalRef.current?.write(data);
        },
        writeBinary(data: string) {
          terminalRef.current?.write(data);
        },
        clear() {
          terminalRef.current?.clear();
        },
        fit() {
          try {
            fitAddonRef.current?.fit();
          } catch {
            // ignore fit errors
          }
        },
        cols() {
          return terminalRef.current?.cols ?? 80;
        },
        rows() {
          return terminalRef.current?.rows ?? 24;
        },
        isReady() {
          return isReady;
        },
      }),
      [isReady],
    );

    return (
      <div
        ref={containerRef}
        data-testid={`xterm-pane-${paneId}`}
        data-ready={isReady ? "true" : "false"}
        style={{ width: "100%", height: "100%", minHeight: 100 }}
      />
    );
  },
);

export default XtermPane;