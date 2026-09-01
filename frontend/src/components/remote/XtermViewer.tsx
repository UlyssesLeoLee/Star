// Star Mobile xterm.js Terminal (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
//
// 客户端: xterm.js + addon-fit, 通过 Star BFF WebSocket shell relay
// 移动端: 自带软键盘 (字母/符号/控制键三栏切换)
"use client";

import { useEffect, useRef, useState } from "react";
import {
  Loader2,
  Wifi,
  WifiOff,
  Terminal as TerminalIcon,
} from "lucide-react";
import { buildRemoteUrl, isRemoteMockMode } from "@/lib/remote/wsClient";

interface XtermViewerProps {
  runtimeId: string;
  hostname: string;
}

const MOCK_BANNER = `\x1b[1;36m★ Star Mobile Shell (Mock) — ${"${hostname}"}\x1b[0m
\x1b[2mvia Star BFF WebSocket relay · real-mode 后端落地后切真\x1b[0m

`;

const MOCK_PROMPT = (host: string) => `\x1b[32muser@${host}\x1b[0m:\x1b[34m~\x1b[0m$ `;

export function XtermViewer({ runtimeId, hostname }: XtermViewerProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<unknown>(null);
  const fitAddonRef = useRef<unknown>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [status, setStatus] = useState<"idle" | "connecting" | "connected" | "error">("idle");
  const [mockMode, setMockMode] = useState(false);
  const [keyboardMode, setKeyboardMode] = useState<"alpha" | "symbol" | "ctrl">("alpha");
  const [history, setHistory] = useState<string[]>([]);
  const [inputBuf, setInputBuf] = useState("");
  const lineBufRef = useRef("");

  // 连 / 断
  const connect = async () => {
    setStatus("connecting");
    const isMock = isRemoteMockMode();
    setMockMode(isMock);

    if (!containerRef.current) return;

    // 动态 import (xterm 引用 DOM)
    // xterm.css 通过 globals.css 顶部 @import 注入 (见 src/app/globals.css)
    const { Terminal: XTerm } = await import("@xterm/xterm");
    const { FitAddon } = await import("@xterm/addon-fit");

    const term = new XTerm({
      fontFamily: 'ui-monospace, "JetBrains Mono", Menlo, monospace',
      fontSize: 13,
      cursorBlink: true,
      theme: {
        background: "#0b0d10",
        foreground: "#e6e9ef",
        cursor: "#00f0ff",
        black: "#0b0d10",
        green: "#10b981",
        blue: "#5b5bd6",
        cyan: "#00f0ff",
        yellow: "#c77b30",
        red: "#ff3366",
      },
      convertEol: true,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();

    termRef.current = term;
    fitAddonRef.current = fit;

    // Resize
    const onResize = () => { try { fit.fit(); } catch { /* ignore */ } };
    window.addEventListener("resize", onResize);

    if (isMock) {
      // 写入 banner
      term.write(MOCK_BANNER);
      term.write(MOCK_PROMPT(hostname));
      setStatus("connected");
      // Mock 模式下我们用 term.onData 接键盘, 写本地历史
      term.onData((data) => {
        for (const ch of data) {
          const code = ch.charCodeAt(0);
          if (code === 13) {
            // Enter
            term.write("\r\n");
            const cmd = lineBufRef.current.trim();
            if (cmd) {
              setHistory((h) => [...h, cmd]);
              mockExec(term, hostname, cmd);
            }
            lineBufRef.current = "";
            term.write(MOCK_PROMPT(hostname));
            setInputBuf("");
          } else if (code === 127 || code === 8) {
            // Backspace
            if (lineBufRef.current.length > 0) {
              lineBufRef.current = lineBufRef.current.slice(0, -1);
              term.write("\b \b");
              setInputBuf(lineBufRef.current);
            }
          } else if (code === 27) {
            // Escape sequence (arrow keys): 简化忽略
          } else if (code >= 32 && code < 127) {
            lineBufRef.current += ch;
            term.write(ch);
            setInputBuf(lineBufRef.current);
          }
        }
      });

      return () => {
        window.removeEventListener("resize", onResize);
        term.dispose();
      };
    }

    // Real mode
    try {
      const url = buildRemoteUrl("terminal", runtimeId);
      const ws = new WebSocket(url);
      ws.binaryType = "arraybuffer";

      ws.onopen = () => {
        setStatus("connected");
        term.write(`\x1b[2mconnected to ${hostname}\x1b[0m\r\n`);
        ws.send(JSON.stringify({ type: "resize", cols: term.cols, rows: term.rows }));
      };
      ws.onmessage = (e) => {
        const data = typeof e.data === "string" ? e.data : new TextDecoder().decode(e.data);
        term.write(data);
      };
      ws.onerror = () => {
        setStatus("error");
        term.write("\r\n\x1b[31mWS error\x1b[0m\r\n");
      };
      ws.onclose = () => {
        setStatus("error");
        term.write("\r\n\x1b[31mdisconnected\x1b[0m\r\n");
      };

      term.onData((data) => ws.send(JSON.stringify({ type: "stdin", data })));
      term.onResize(({ cols, rows }) => ws.send(JSON.stringify({ type: "resize", cols, rows })));

      wsRef.current = ws;

      return () => {
        window.removeEventListener("resize", onResize);
        term.dispose();
        try { ws.close(); } catch { /* ignore */ }
      };
    } catch {
      setStatus("error");
    }
  };

  return (
    <div className="flex flex-col h-full bg-[#0b0d10]">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-line bg-bg-soft/80 backdrop-blur text-xs">
        {status === "connected" ? (
          <Wifi size={13} className="text-ok" />
        ) : status === "error" ? (
          <WifiOff size={13} className="text-err" />
        ) : (
          <Loader2 size={13} className="text-accent animate-spin" />
        )}
        <TerminalIcon size={13} className="text-ink-dim" />
        <span className="font-mono text-ink-dim">
          {hostname} · shell {mockMode ? "· mock" : ""}
        </span>
        <span className="ml-auto text-ink-mute font-mono">{status}</span>
      </div>

      <div
        ref={containerRef}
        data-testid="xterm-container"
        className="flex-1 min-h-0"
        onClick={() => { if (status === "idle") void connect(); }}
      />

      {status === "idle" && (
        <button
          type="button"
          onClick={connect}
          data-testid="xterm-connect"
          className="absolute inset-0 mt-9 flex flex-col items-center justify-center gap-3 text-ink-dim hover:text-ink"
        >
          <div className="size-16 rounded-2xl border border-accent/40 grid place-items-center bg-accent/10">
            <TerminalIcon size={28} className="text-accent" />
          </div>
          <span className="text-sm font-semibold">点击开启 {hostname} shell</span>
        </button>
      )}

      {/* Mobile 软键盘 (alpha / symbol / ctrl 三模式) */}
      {status === "connected" && (
        <SoftKeyboard
          mode={keyboardMode}
          onModeChange={setKeyboardMode}
          onKey={(k) => {
            const term = termRef.current as { input: (s: string) => void } | null;
            term?.input(k);
          }}
        />
      )}
    </div>
  );
}

// =====================================================================
// Mobile 软键盘 (per docs/frontend/design/dynamic-interaction-design.md)
// =====================================================================
function SoftKeyboard({
  mode,
  onModeChange,
  onKey,
}: {
  mode: "alpha" | "symbol" | "ctrl";
  onModeChange: (m: "alpha" | "symbol" | "ctrl") => void;
  onKey: (k: string) => void;
}) {
  const alphaRows = [
    ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    ["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    ["z", "x", "c", "v", "b", "n", "m"],
  ];
  const symbolRows = [
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    ["-", "=", "[", "]", "\\", ";", "'", ",", ".", "/"],
    ["!", "@", "#", "$", "%", "^", "&", "*", "(", ")"],
  ];
  const ctrlKeys = [
    { label: "Tab", data: "\t" },
    { label: "Esc", data: "\x1b" },
    { label: "↑", data: "\x1b[A" },
    { label: "↓", data: "\x1b[B" },
    { label: "←", data: "\x1b[D" },
    { label: "→", data: "\x1b[C" },
    { label: "Home", data: "\x1b[H" },
    { label: "End", data: "\x1b[F" },
    { label: "PgUp", data: "\x1b[5~" },
    { label: "PgDn", data: "\x1b[6~" },
  ];

  const rows = mode === "alpha" ? alphaRows : mode === "symbol" ? symbolRows : [];

  return (
    <div className="border-t border-line bg-bg-soft/95 backdrop-blur p-1.5">
      {/* 模式切换 */}
      <div className="flex gap-1 mb-1.5">
        {(["alpha", "symbol", "ctrl"] as const).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => onModeChange(m)}
            data-testid={`kb-mode-${m}`}
            className={
              "flex-1 py-1.5 text-[10px] font-mono uppercase rounded-md " +
              (mode === m
                ? "bg-accent/20 text-accent border border-accent/40"
                : "bg-bg text-ink-mute border border-line")
            }
          >
            {m}
          </button>
        ))}
      </div>

      {/* 键位 */}
      {mode === "ctrl" ? (
        <div className="grid grid-cols-5 gap-1">
          {ctrlKeys.map((k) => (
            <button
              key={k.label}
              type="button"
              onClick={() => onKey(k.data)}
              data-testid={`kb-ctrl-${k.label}`}
              className="py-2.5 text-xs font-mono rounded-md border border-line bg-bg text-ink-dim active:bg-accent/20"
            >
              {k.label}
            </button>
          ))}
        </div>
      ) : (
        <div className="space-y-1">
          {rows.map((row, i) => (
            <div key={i} className="flex gap-0.5" style={{ paddingLeft: i === 1 ? 12 : 0 }}>
              {row.map((k) => (
                <button
                  key={k}
                  type="button"
                  onClick={() => onKey(k)}
                  data-testid={`kb-${k}`}
                  className="flex-1 py-2 text-sm font-mono rounded-md border border-line bg-bg text-ink active:bg-accent/20"
                >
                  {k}
                </button>
              ))}
            </div>
          ))}
          <div className="flex gap-0.5 pt-1">
            <button
              type="button"
              onClick={() => onKey(" ")}
              data-testid="kb-space"
              className="flex-1 py-2 text-xs font-mono rounded-md border border-line bg-bg text-ink-dim"
            >
              space
            </button>
            <button
              type="button"
              onClick={() => onKey("\b")}
              data-testid="kb-backspace"
              className="px-4 py-2 text-xs font-mono rounded-md border border-line bg-bg text-ink-dim"
            >
              ⌫
            </button>
            <button
              type="button"
              onClick={() => onKey("\r")}
              data-testid="kb-enter"
              className="px-4 py-2 text-xs font-mono rounded-md border border-accent/40 bg-accent/10 text-accent font-semibold"
            >
              ↵ Enter
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// =====================================================================
// Mock 模式下的命令执行 (UI 演示用, 仅 4-5 条命令)
// =====================================================================
function mockExec(
  term: { write: (s: string) => void },
  host: string,
  cmd: string,
) {
  const out = (s: string) => term.write(s);
  const [base, ...rest] = cmd.split(/\s+/);
  switch (base) {
    case "ls":
      out("\x1b[1;34mdrwx\x1b[0m  6 user user  4096 Sep  1 11:00 \x1b[1;34msrc\x1b[0m\r\n");
      out("\x1b[1;34mdrwx\x1b[0m  3 user user  4096 Sep  1 10:00 \x1b[1;34mnode_modules\x1b[0m\r\n");
      out("\x1b[-rw-r--r--\x1b[0m 1 user user  1234 Aug 31 18:00 package.json\r\n");
      out("\x1b[-rw-r--r--\x1b[0m 1 user user   256 Aug 31 17:00 README.md\r\n");
      break;
    case "pwd":
      out(`/home/user/dev/${host.toLowerCase()}\r\n`);
      break;
    case "whoami":
      out(`user\r\n`);
      break;
    case "date":
      out(new Date().toUTCString() + "\r\n");
      break;
    case "clear":
      out("\x1b[2J\x1b[H");
      break;
    case "echo":
      out(rest.join(" ") + "\r\n");
      break;
    case "help":
      out("Mock commands: ls, pwd, whoami, date, clear, echo, help, exit\r\n");
      break;
    case "exit":
      out("Mock shell: cannot exit (no real backend). Try 'help'.\r\n");
      break;
    default:
      out(`\x1b[31m${base}\x1b[0m: command not found (mock mode)\r\n`);
  }
}
