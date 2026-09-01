// =====================================================================
// cytoscape-cose-bilkent ambient declaration
// =====================================================================
// 该包没有官方 d.ts, 这里声明其默认导出为注册函数
// (per cytoscape-cose-bilkent@4.1.0 README: 注册 cose-bilkent 布局到 cytoscape)
//
// 用法 (per ArchGraphModal.tsx):
//   import coseBilkent from "cytoscape-cose-bilkent";
//   coseBilkent(cytoscape);  // 注册扩展
// =====================================================================

declare module "cytoscape-cose-bilkent" {
  import type { Ext, Core } from "cytoscape";
  const ext: (cy: typeof Ext.prototype.cy) => void;
  export default ext;
}
