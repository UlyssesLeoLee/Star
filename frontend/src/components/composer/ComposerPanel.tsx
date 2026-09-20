// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/composer/ComposerPanel.tsx -- ULYS-98-W3.5

"use client";

import { useCallback, useState } from "react";

const BFF_BASE_URL =
  process.env.NEXT_PUBLIC_BFF_URL ?? "http://localhost:8080";

export interface FileEdit {
  path: string;
  diff: string;
  new_content: string;
}

export interface ComposerPanelProps {
  files: { path: string; content: string }[];
  instruction: string;
  model?: string;
  userId: string;
}

export function ComposerPanel(props: ComposerPanelProps) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [edits, setEdits] = useState<FileEdit[]>([]);
  const [accepted, setAccepted] = useState<Set<string>>(new Set());

  const submit = useCallback(async () => {
    setBusy(true);
    setError(null);
    setEdits([]);
    setAccepted(new Set());
    try {
      const res = await fetch(`${BFF_BASE_URL}/v1/composer/edit`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          files: props.files,
          instruction: props.instruction,
          user_id: props.userId,
          model: props.model ?? null,
        }),
        signal: AbortSignal.timeout(15000),
      });
      if (!res.ok) throw new Error(`POST composer/edit -> HTTP ${res.status}`);
      const body = (await res.json()) as { edits: FileEdit[] };
      setEdits(body.edits ?? []);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setBusy(false);
    }
  }, [props.files, props.instruction, props.userId, props.model]);

  const apply = useCallback(async () => {
    const chosen = edits.filter((e) => accepted.has(e.path));
    if (chosen.length === 0) return;
    setBusy(true);
    try {
      const res = await fetch(`${BFF_BASE_URL}/v1/composer/apply`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ edits: chosen }),
        signal: AbortSignal.timeout(20000),
      });
      if (!res.ok) throw new Error(`apply -> HTTP ${res.status}`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setBusy(false);
    }
  }, [accepted, edits]);

  const reject = useCallback(() => {
    setEdits([]);
    setAccepted(new Set());
  }, []);

  const toggleAccept = useCallback((path: string) => {
    setAccepted((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  return { edits, busy, error, submit, apply, reject, accepted, toggleAccept };
}

export default ComposerPanel;
