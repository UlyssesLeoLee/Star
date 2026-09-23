/**
 * DiffAnnotator — ULYS-165 FR-ORCA-035 行内 diff 注释组件.
 *
 * Per docs/ecosystem-survey/orca-design-survey.md §11 line 483-485:
 *   在 Diff 上写注释 → 回喂 Agent
 *
 * **本文件 (ULYS-165 v2)** — 适配 PR #84 (ULYS-201) 的 DiffAnnotation DTO:
 *   - 移除 status 字段 (per MVP in-memory registry)
 *   - 用 author: 'user' | 'agent' 替代
 *   - line_range 用 line_start + line_end 分离字段
 *
 * ## 交互
 * - 点击 diff 行号 → 行号旁弹出 textarea
 * - 提交 → POST /v1/diff-annotations
 * - DELETE → DELETE /v1/diff-annotations/:comment_id
 *
 * ## 设计要点
 * - 受控组件, 通过 props 接 annotations + onChange
 * - 不在内部 fetch (留 P1 接入 react-query / SWR)
 */

import { useState } from "react";
import { MessageSquarePlus, X } from "lucide-react";

export type DiffAnnotationAuthor = "user" | "agent";

export interface DiffAnnotationView {
  comment_id: string;
  file_path: string;
  line_start: number;
  line_end: number;
  body: string;
  agent_run_id: string;
  created_at: string;
  author: DiffAnnotationAuthor;
}

interface DiffAnnotatorProps {
  /** worktree-relative POSIX 路径 */
  filePath: string;
  /** 当前渲染的 diff 行号集合 (1-indexed) */
  visibleLines: number[];
  /** 该文件已有的 annotations */
  annotations: DiffAnnotationView[];
  /** 当前 agent_run_id (必填, 关联到某次 AI 输出) */
  agentRunId: string;
  /** tenant + workspace id (从 session 取) */
  tenantId: string;
  workspaceId: string;
  /** 创建 callback */
  onCreate?: (payload: {
    comment_id: string;
    file_path: string;
    line_start: number;
    line_end: number;
    body: string;
    agent_run_id: string;
    author: DiffAnnotationAuthor;
  }) => Promise<DiffAnnotationView>;
  /** 删除 callback */
  onDelete?: (comment_id: string) => Promise<void>;
}

export function DiffAnnotator({
  filePath,
  visibleLines,
  annotations,
  agentRunId,
  tenantId: _tenantId,
  workspaceId: _workspaceId,
  onCreate,
  onDelete,
}: DiffAnnotatorProps) {
  const [activeLine, setActiveLine] = useState<number | null>(null);
  const [draftBody, setDraftBody] = useState<string>("");
  const [submitting, setSubmitting] = useState(false);

  const onLineClick = (line: number) => {
    setActiveLine(line);
    setDraftBody("");
  };

  const onCancel = () => {
    setActiveLine(null);
    setDraftBody("");
  };

  const onSubmit = async () => {
    if (!activeLine || !draftBody.trim() || !onCreate) return;
    setSubmitting(true);
    try {
      // comment_id 由后端生成 (Uuid::new_v4), 但前端可预生成保证幂等
      const comment_id = crypto.randomUUID();
      await onCreate({
        comment_id,
        file_path: filePath,
        line_start: activeLine,
        line_end: activeLine + 1,
        body: draftBody.trim(),
        agent_run_id: agentRunId,
        author: "user",
      });
      onCancel();
    } finally {
      setSubmitting(false);
    }
  };

  const annotationsByLine = new Map<number, DiffAnnotationView[]>();
  for (const a of annotations) {
    if (a.file_path !== filePath) continue;
    for (let line = a.line_start; line < a.line_end; line++) {
      const list = annotationsByLine.get(line) ?? [];
      list.push(a);
      annotationsByLine.set(line, list);
    }
  }

  return (
    <div className="diff-annotator" data-testid="diff-annotator">
      <ul className="diff-lines">
        {visibleLines.map((line) => {
          const lineAnnotations = annotationsByLine.get(line) ?? [];
          return (
            <li
              key={line}
              className="diff-line"
              data-line-number={line}
              data-testid={`diff-line-${line}`}
            >
              <button
                className="diff-gutter"
                onClick={() => onLineClick(line)}
                aria-label={`Annotate line ${line}`}
                data-testid={`diff-line-${line}-button`}
              >
                {line}
              </button>
              {lineAnnotations.length > 0 && (
                <span
                  className="diff-annotation-count"
                  aria-label={`${lineAnnotations.length} annotation(s)`}
                  data-testid={`diff-line-${line}-count`}
                >
                  {lineAnnotations.length}
                </span>
              )}
              {lineAnnotations.map((a) => (
                <div
                  key={a.comment_id}
                  className="diff-annotation-chip"
                  data-testid={`annotation-${a.comment_id}`}
                >
                  <span className="annotation-body">{a.body}</span>
                  <span className="annotation-author">[{a.author}]</span>
                  {onDelete && (
                    <button
                      className="annotation-delete"
                      onClick={() => onDelete(a.comment_id)}
                      aria-label="Delete annotation"
                    >
                      <X size={12} />
                    </button>
                  )}
                </div>
              ))}
            </li>
          );
        })}
      </ul>
      {activeLine !== null && (
        <div
          className="diff-annotation-input"
          data-testid="diff-annotation-input"
        >
          <textarea
            value={draftBody}
            onChange={(e) => setDraftBody(e.target.value)}
            placeholder={`Annotate line ${activeLine}...`}
            disabled={submitting}
            data-testid="annotation-body-input"
          />
          <div className="annotation-actions">
            <button
              onClick={onCancel}
              disabled={submitting}
              data-testid="annotation-cancel"
            >
              Cancel
            </button>
            <button
              onClick={onSubmit}
              disabled={!draftBody.trim() || submitting}
              data-testid="annotation-submit"
            >
              <MessageSquarePlus size={14} /> Submit
            </button>
          </div>
        </div>
      )}
    </div>
  );
}