// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/DiffAnnotator.tsx
//
// Diff 行级标注 UI (per FR-ORCA-035 §11 v1.0, ULYS-211 / 父 ULYS-201)。
//
// spec: "用户对 AI 生成的 diff 可在任意行 drop comment; comment 作为
//        structured feedback 直接回到 agent session 的下一轮 prompt."
//
// 本组件 = line gutter + 行内 composer + annotation 列表 + feed_to_agent。
// 只吃 props 里的 diff 行, 不自己拉 diff —— diff 内容的来源 (worktree-canvas
// Files tab / PR 视图) 各自注入, 本组件只负责标注那一层。
//
// 行号语义 (per api/annotations.ts): 1-based 半开区间。gutter 上点第 N 行
// 发出的是 [N, N+1); shift+click 拉到第 M 行发出 [min, max+1)。删除行
// (removed) 在新文件侧没有行号, 按 annotate.rs 的 1-based 约束不可标注。

"use client";

import { useCallback, useMemo, useState } from "react";
import { MessageSquarePlus, Send, Trash2, X } from "lucide-react";

import {
  AUTHOR_USER,
  formatLineRange,
  lineSpanRange,
  rangeCoversLine,
  type AnnotationAuthor,
  type DiffAnnotation,
} from "@/lib/api/annotations";
import {
  useAnnotationStore,
  useAnnotationsLoading,
  useFeedToAgent,
  useFileAnnotations,
} from "@/lib/hooks/use-annotations";

// =====================================================================
// diff 行模型
// =====================================================================

export type DiffLineKind = "context" | "added" | "removed" | "hunk";

export interface DiffLine {
  kind: DiffLineKind;
  /** 新文件侧行号 (1-based); removed / hunk 头为 null → 不可标注 */
  newLine: number | null;
  /** 旧文件侧行号 (仅展示) */
  oldLine?: number | null;
  /** 行内容 (不含 +/- 前缀) */
  content: string;
}

export interface DiffAnnotatorProps {
  /** 相对 repo 根的文件路径 */
  filePath: string;
  /** 关联的 agent run (feed_to_agent 的聚合维度) */
  agentRunId: string;
  lines: DiffLine[];
  /** 默认以 user 身份标注 (agent 身份留给 refinement loop) */
  author?: AnnotationAuthor;
  className?: string;
}

const KIND_STYLE: Record<DiffLineKind, string> = {
  added: "bg-emerald-500/10",
  removed: "bg-rose-500/10",
  context: "",
  hunk: "bg-slate-500/10 text-slate-500",
};

const KIND_SIGN: Record<DiffLineKind, string> = {
  added: "+",
  removed: "-",
  context: " ",
  hunk: "@",
};

// =====================================================================
// 组件
// =====================================================================

export function DiffAnnotator({
  filePath,
  agentRunId,
  lines,
  author = AUTHOR_USER,
  className = "",
}: DiffAnnotatorProps) {
  const annotations = useFileAnnotations(filePath, agentRunId);
  const loading = useAnnotationsLoading(filePath);
  const addAnnotation = useAnnotationStore((s) => s.addAnnotation);
  const deleteAnnotation = useAnnotationStore((s) => s.deleteAnnotation);
  const error = useAnnotationStore((s) => s.error);
  const { feed, promptFragment, feeding } = useFeedToAgent(agentRunId);

  /** 选区锚点 (点第一下的行); null = 没有打开的 composer */
  const [anchor, setAnchor] = useState<number | null>(null);
  /** 选区末端 (shift+click 拉到的行); 未拉伸时等于 anchor */
  const [head, setHead] = useState<number | null>(null);
  const [body, setBody] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const selection = useMemo(
    () => (anchor === null ? null : lineSpanRange(anchor, head ?? anchor)),
    [anchor, head],
  );

  /**
   * 该行落在哪些 annotation 里 (gutter 角标用)。
   * 按**渲染出来的行**去问每条 annotation, 而不是展开 line_range —— agent
   * 侧写入的区间可能远大于当前 hunk, 展开会白跑几万次循环。
   */
  const annotationsByLine = useMemo(() => {
    const map = new Map<number, DiffAnnotation[]>();
    for (const line of lines) {
      if (line.newLine === null) continue;
      const hits = annotations.filter((a) => rangeCoversLine(a.line_range, line.newLine as number));
      if (hits.length > 0) map.set(line.newLine, hits);
    }
    return map;
  }, [annotations, lines]);

  const onGutterClick = useCallback(
    (line: number, shiftKey: boolean) => {
      if (shiftKey && anchor !== null) {
        setHead(line);
        return;
      }
      // 再点同一行 = 收起 composer
      if (anchor === line && head === line) {
        setAnchor(null);
        setHead(null);
        setBody("");
        return;
      }
      setAnchor(line);
      setHead(line);
    },
    [anchor, head],
  );

  const closeComposer = useCallback(() => {
    setAnchor(null);
    setHead(null);
    setBody("");
  }, []);

  const onSubmit = useCallback(async () => {
    if (!selection || !body.trim()) return; // 空 body 会被后端判 InvalidField, 前端先挡
    setSubmitting(true);
    const created = await addAnnotation({
      file_path: filePath,
      line_range: selection,
      body: body.trim(),
      agent_run_id: agentRunId,
      author,
    });
    setSubmitting(false);
    if (created) closeComposer();
  }, [selection, body, addAnnotation, filePath, agentRunId, author, closeComposer]);

  /** composer 挂在选区末行之后, 与 GitHub 行内评论一致 */
  const composerAfterLine = selection ? selection.end - 1 : null;

  return (
    <section
      data-testid="diff-annotator"
      data-file-path={filePath}
      data-agent-run-id={agentRunId}
      className={`flex flex-col gap-3 ${className}`}
    >
      <header className="flex items-center justify-between gap-2 text-xs">
        <span className="font-mono truncate" title={filePath}>
          {filePath}
        </span>
        <span data-testid="annotation-count" className="shrink-0 text-slate-500">
          {loading ? "载入中…" : `${annotations.length} 条标注`}
        </span>
      </header>

      {error && (
        <p data-testid="annotation-error" role="alert" className="text-xs text-rose-500">
          {error}
        </p>
      )}

      {/* ── diff + line gutter ── */}
      <div
        data-testid="diff-lines"
        className="overflow-x-auto rounded border border-slate-200 font-mono text-[11px] leading-5 dark:border-slate-700"
      >
        {lines.map((line, idx) => {
          const annotatable = line.newLine !== null;
          const marks = line.newLine !== null ? annotationsByLine.get(line.newLine) : undefined;
          const selected =
            selection !== null &&
            line.newLine !== null &&
            rangeCoversLine(selection, line.newLine);

          return (
            <div key={`${idx}-${line.newLine ?? "x"}`}>
              <div
                data-testid={line.newLine !== null ? `diff-line-${line.newLine}` : undefined}
                data-selected={selected || undefined}
                className={`group flex items-start gap-2 px-1 ${KIND_STYLE[line.kind]} ${
                  selected ? "outline outline-1 outline-blue-400" : ""
                }`}
              >
                <span className="w-10 shrink-0 select-none text-right text-slate-400">
                  {line.newLine ?? line.oldLine ?? ""}
                </span>

                {annotatable ? (
                  <button
                    type="button"
                    data-testid={`annotation-gutter-${line.newLine}`}
                    aria-label={`在第 ${line.newLine} 行添加标注`}
                    onClick={(e) => onGutterClick(line.newLine as number, e.shiftKey)}
                    className="mt-0.5 shrink-0 rounded p-0.5 text-slate-400 opacity-40 transition hover:bg-blue-500 hover:text-white group-hover:opacity-100 focus:opacity-100"
                  >
                    <MessageSquarePlus size={11} />
                  </button>
                ) : (
                  <span className="w-[17px] shrink-0" aria-hidden />
                )}

                <span className="w-2 shrink-0 select-none text-slate-400">
                  {KIND_SIGN[line.kind]}
                </span>
                <span className="whitespace-pre">{line.content}</span>

                {marks && marks.length > 0 && (
                  <span
                    data-testid={`annotation-marker-${line.newLine}`}
                    title={marks.map((a) => a.body).join("\n")}
                    className="ml-auto shrink-0 rounded-full bg-blue-500 px-1.5 text-[10px] text-white"
                  >
                    {marks.length}
                  </span>
                )}
              </div>

              {/* ── 行内 composer ── */}
              {composerAfterLine !== null && line.newLine === composerAfterLine && (
                <div
                  data-testid="annotation-composer"
                  className="border-y border-blue-300 bg-blue-500/5 p-2 dark:border-blue-800"
                >
                  <div className="mb-1 flex items-center justify-between text-[10px] text-slate-500">
                    <span data-testid="composer-range">
                      行 {selection ? formatLineRange(selection) : ""}
                      {anchor !== null && " · Shift+点击其他行可扩选"}
                    </span>
                    <button
                      type="button"
                      data-testid="annotation-cancel"
                      aria-label="取消标注"
                      onClick={closeComposer}
                      className="rounded p-0.5 hover:bg-slate-200 dark:hover:bg-slate-700"
                    >
                      <X size={11} />
                    </button>
                  </div>
                  <textarea
                    data-testid="annotation-body-input"
                    aria-label="标注内容"
                    value={body}
                    onChange={(e) => setBody(e.target.value)}
                    rows={3}
                    placeholder="写下要回给 agent 的修改意见…"
                    className="w-full rounded border border-slate-200 bg-transparent p-1 font-sans text-xs dark:border-slate-700"
                  />
                  <div className="mt-1 flex justify-end">
                    <button
                      type="button"
                      data-testid="annotation-submit"
                      disabled={!body.trim() || submitting}
                      onClick={() => void onSubmit()}
                      className="rounded bg-blue-500 px-2 py-0.5 text-[11px] text-white disabled:opacity-40"
                    >
                      {submitting ? "提交中…" : "添加标注"}
                    </button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* ── annotation 列表 ── */}
      <div data-testid="annotation-list" className="flex flex-col gap-1">
        {annotations.length === 0 ? (
          <p data-testid="annotation-empty" className="text-xs text-slate-500">
            还没有标注 — 点击行号旁的图标即可在该行 drop comment。
          </p>
        ) : (
          annotations.map((a) => (
            <div
              key={a.id}
              data-testid="annotation-item"
              data-annotation-id={a.id}
              className="flex items-start gap-2 rounded border border-slate-200 p-2 text-xs dark:border-slate-700"
            >
              <span className="shrink-0 font-mono text-[10px] text-slate-500">
                L{formatLineRange(a.line_range)} · {a.author.kind}
              </span>
              <span data-testid="annotation-body" className="flex-1 break-words">
                {a.body}
              </span>
              <button
                type="button"
                data-testid="annotation-delete"
                aria-label="删除标注"
                onClick={() => void deleteAnnotation(a.id, filePath)}
                className="shrink-0 rounded p-0.5 text-slate-400 hover:bg-rose-500 hover:text-white"
              >
                <Trash2 size={11} />
              </button>
            </div>
          ))
        )}
      </div>

      {/* ── feed_to_agent ── */}
      <div className="flex items-center gap-2">
        <button
          type="button"
          data-testid="feed-to-agent-button"
          disabled={feeding || annotations.length === 0}
          onClick={() => void feed()}
          className="inline-flex items-center gap-1 rounded bg-slate-800 px-2 py-1 text-[11px] text-white disabled:opacity-40 dark:bg-slate-200 dark:text-slate-900"
        >
          <Send size={11} />
          {feeding ? "发送中…" : "回传 agent (feed_to_agent)"}
        </button>
        <span className="text-[10px] text-slate-500">
          标注将作为 structured feedback 进入下一轮 prompt
        </span>
      </div>

      {promptFragment !== null && (
        <pre
          data-testid="prompt-fragment"
          className="max-h-48 overflow-auto rounded border border-slate-200 p-2 font-mono text-[10px] whitespace-pre-wrap dark:border-slate-700"
        >
          {promptFragment}
        </pre>
      )}
    </section>
  );
}

export default DiffAnnotator;
