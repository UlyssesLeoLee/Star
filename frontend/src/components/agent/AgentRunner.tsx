// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/agent/AgentRunner.tsx -- ULYS-98-W4.4
// (per docs/briefs/ulys-98-star-cursor-min-v1.md Sub-task 4.4 frontend Agent mode UI)

"use client";

import { useCallback, useState } from "react";

const BFF_BASE_URL =
  process.env.NEXT_PUBLIC_BFF_URL ?? "http://localhost:8080";

export interface AgentStep {
  step_id: string;
  thought: string;
  tool_call?: unknown;
  observation?: string | null;
}

export interface AgentRunnerResult {
  steps: AgentStep[];
  final_answer: string | null;
  busy: boolean;
  error: string | null;
  run: (prompt: string) => Promise<void>;
  reset: () => void;
}

export interface AgentRunnerProps {
  userId: string;
  sessionId: string;
  model?: string;
  maxSteps?: number;
}

export function useAgentRunner(props: AgentRunnerProps): AgentRunnerResult {
  const [steps, setSteps] = useState<AgentStep[]>([]);
  const [finalAnswer, setFinalAnswer] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reset = useCallback(() => {
    setSteps([]);
    setFinalAnswer(null);
    setError(null);
    setBusy(false);
  }, []);

  const run = useCallback(
    async (prompt: string) => {
      setBusy(true);
      setError(null);
      setSteps([]);
      setFinalAnswer(null);
      try {
        const res = await fetch(`${BFF_BASE_URL}/v1/agent/run`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            session_id: props.sessionId,
            user_id: props.userId,
            prompt,
            model: props.model ?? null,
            max_steps: props.maxSteps ?? 5,
          }),
          signal: AbortSignal.timeout(60000),
        });
        if (!res.ok) throw new Error(`POST /v1/agent/run -> HTTP ${res.status}`);
        const body = (await res.json()) as {
          steps: AgentStep[];
          final_answer: string | null;
        };
        setSteps(body.steps ?? []);
        setFinalAnswer(body.final_answer ?? null);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
      } finally {
        setBusy(false);
      }
    },
    [props.sessionId, props.userId, props.model, props.maxSteps],
  );

  return { steps, final_answer: finalAnswer, busy, error, run, reset };
}

export function AgentRunnerView(props: { runner: AgentRunnerResult }) {
  const { steps, final_answer, busy, error } = props.runner;
  return (
    <div data-testid="agent-runner" className="flex flex-col gap-2 p-3">
      {error && (
        <div className="text-sm text-red-600" role="alert">
          {error}
        </div>
      )}
      <ol data-testid="agent-steps" className="space-y-1">
        {steps.map((s) => (
          <li key={s.step_id} className="text-sm border rounded p-2 bg-white/50">
            <div className="font-mono text-xs opacity-60">
              step {s.step_id.slice(0, 8)}
            </div>
            <div className="mt-1 whitespace-pre-wrap">{s.thought}</div>
          </li>
        ))}
      </ol>
      {final_answer && (
        <div
          data-testid="agent-final-answer"
          className="mt-2 p-2 border-l-4 border-green-500 bg-green-50 text-sm"
        >
          {final_answer}
        </div>
      )}
      {busy && (
        <div className="text-xs opacity-60" aria-live="polite">
          agent running\u2026
        </div>
      )}
    </div>
  );
}

export default AgentRunnerView;
