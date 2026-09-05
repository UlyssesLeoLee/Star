// =====================================================================
// work-item/page.tsx — U2 改造: 全部 work-item 列表 → 重定向到 /issues
// =====================================================================
// 旧页面是 work-item 列表 (per docs/frontend/design/ui-redesign-multica-style.md §2),
// 现在 issues 主面板 (Kanban/List/Tree/Sprint) 已实装, 列表功能 100% 覆盖。
// 为了符合"6 路由吸收 22 路由"原则, 这里 redirect 到 /issues (per §2 6 路由表)。
//
// 注意:
//   - 详情页 /work-item/[id] 仍保留 (向后兼容; Issues 详情侧栏有"打开在旧 work-item"链接)
//   - 这个 redirect 是 server component, 不需要 "use client"
//   - 旧 test 文件路径为 work-item/page.test.tsx (新建)
//
// 已知缺口 (per 缺标比错标):
//   - redirect 是硬跳, 不保留 ?view= 状态 (per §10 redirect 设计)
//   - 22 路由 → 6 路由 redirect 完整列表由 U5 在 next.config.js 统一配置 (per §8.2)
// =====================================================================

import { redirect } from "next/navigation";

export default function WorkItemListPage() {
  // 全部 work-item 列表 → /issues (主面板, Kanban default)
  redirect("/sprint");
}
