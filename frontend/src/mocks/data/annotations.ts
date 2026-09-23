// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/mocks/data/annotations.ts
//
// Diff Annotation 演示数据 (per FR-ORCA-035 §11 v1.0, ULYS-211)。
//
// 缺标 (per 守门 #11): 后端目前没有"取某 worktree 变更 diff"的接口
// (agent-bridge 只落了 AnnotationRegistry, 见 lib/api/annotations.ts 文件头),
// 所以 diff 正文走本文件的 mock —— 与 src/mocks/data/* 其余模块同构。
// **annotation 本身不是 mock 数据**: 它走真实 REST client, 只是 MSW 在
// 后端路由补齐前拦截而已。

import type { DiffLine } from "@/components/DiffAnnotator";

/** MSW 预注册的 agent run (对应 annotate.rs 的 `register_agent_run`) */
export const DEMO_AGENT_RUN_ID = "3f1c8a02-5b7e-4c21-9d6a-8e0f12b4c7d5";

export const DEMO_FILE_PATH = "crates/agent-bridge/src/annotate.rs";

/**
 * 一段 AI 生成的 diff (新文件侧 1-based 行号)。
 * removed 行在新文件侧没有行号 → newLine = null → gutter 不可标注。
 */
export const DEMO_DIFF_LINES: DiffLine[] = [
  { kind: "hunk", newLine: null, oldLine: null, content: "@@ -12,7 +12,11 @@ impl AnnotationRegistry" },
  { kind: "context", newLine: 12, oldLine: 12, content: "    pub fn add(" },
  { kind: "context", newLine: 13, oldLine: 13, content: "        &mut self," },
  { kind: "context", newLine: 14, oldLine: 14, content: "        file_path: impl Into<String>," },
  { kind: "removed", newLine: null, oldLine: 15, content: "        let id = Uuid::new_v4();" },
  { kind: "added", newLine: 15, oldLine: null, content: "        let id = Uuid::now_v7();" },
  { kind: "added", newLine: 16, oldLine: null, content: "        // now_v7 保证 created_at ASC 与 id 单调一致" },
  { kind: "context", newLine: 17, oldLine: 16, content: "        let annotation = DiffAnnotation {" },
  { kind: "added", newLine: 18, oldLine: null, content: "            id," },
  { kind: "added", newLine: 19, oldLine: null, content: "            created_at: Utc::now()," },
  { kind: "context", newLine: 20, oldLine: 17, content: "        };" },
  { kind: "context", newLine: 21, oldLine: 18, content: "        Ok(id)" },
  { kind: "context", newLine: 22, oldLine: 19, content: "    }" },
];
