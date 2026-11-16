/*
@cypher schema=1 source_sha256=b26b0c7ca9d46cf05dfb15a006257e77d1d65a6d7328bf81a33b955511f46c09
MERGE (self:File {path:"frontend/src/types/imGateway.d.ts"})
@endcypher
*/
// =====================================================================
// imGateway 模块契约占位 (sibling 任务将落地 frontend/src/lib/chat/imGateway.ts)
//
// dmStore.tryLoadImGateway 按此签名动态 import 该模块:
//   - 模块存在且导出 getImGateway()/createImGateway() → 返回 ImGateway 实例
//   - 模块缺失/网关不可达 → null → dmStore 回退本地模式 (realtimeConnected=false)
//
// 真实文件落地后, TypeScript 自动优先解析真实文件, 本声明不再生效。
// =====================================================================
declare module "@/lib/chat/imGateway" {
  import type {
    ImGateway,
    ImGatewayInboundMessage,
    ImGatewayOutboundMessage,
  } from "@/lib/chat/dmStore";

  export type { ImGateway, ImGatewayInboundMessage, ImGatewayOutboundMessage };

  /** 取已创建的网关单例 (未初始化/不可达 → null) */
  export function getImGateway(): ImGateway | null;

  /** 创建网关实例 (dmStore.tryLoadImGateway 按此签名调用) */
  export function createImGateway(opts?: { url?: string }): ImGateway;
}
