// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/index.ts — ULYS-98-W1.1 + W2.3 export barrel
export { default as MonacoEditor, inferLanguageFromPath } from "./MonacoEditor";
export type { MonacoEditorProps } from "./MonacoEditor";
export {
  registerInlineCompletion,
  resolveNoNetworkMode,
  debounce,
  defaultCompletionFetcher,
  stubCompletionFetcher,
  DEBOUNCE_MS,
  MAX_INLINE_BYTES,
  COMPLETION_ENDPOINT,
  NO_NETWORK_DEFAULT,
} from "./inlineCompletion";
export type {
  CompletionFetcher,
  CompletionInlineRequest,
  CompletionInlineResponse,
  RegisterInlineCompletionOptions,
  TokenUsageDto,
} from "./inlineCompletion";
export { default as CmdKAction } from "./CmdKAction";