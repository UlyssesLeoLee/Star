/*
@cypher schema=1 source_sha256=8301e7ec37bd7f44f22333e526fef926bf81999711a984b61bb1761c5e2c444f
MERGE (self:File {path:"frontend/src/proxy.ts"})
MERGE (proxy:Symbol {id:"frontend/src/proxy.ts::proxy",kind:"function"})
MERGE (self)-[:DEFINES]->(proxy)
@endcypher
*/
// =====================================================================
// proxy.ts (Next 16 middleware) — 移动端 UA 检测 (per 2026-11-14 用户发令:
//   "手机访问时应该打开 flutter 版本的页面; flutter 版将来也用于小程序和 app")
//
//   移动 UA (Android/iPhone/iPad) → 301 到 Flutter Web 版
//   目标地址: 环境变量 STAR_FLUTTER_WEB_URL (未配置时放行, 继续 PWA 布局)
//   豁免: 显式 ?web=1 参数 (手机用户强制看 Web 版)
// =====================================================================

import { NextRequest, NextResponse } from "next/server";

const MOBILE_UA =
  /Android|iPhone|iPad|iPod|Mobile Safari|Windows Phone|BlackBerry|Opera Mini|Kindle/i;

export function proxy(req: NextRequest) {
  const ua = req.headers.get("user-agent") ?? "";
  const flutterUrl = process.env.STAR_FLUTTER_WEB_URL;
  const forceWeb = req.nextUrl.searchParams.get("web") === "1";

  if (flutterUrl && !forceWeb && MOBILE_UA.test(ua)) {
    // 保留当前路径 (Flutter 版按同路由渲染)
    const target = new URL(flutterUrl);
    const path = req.nextUrl.pathname;
    if (path !== "/") target.pathname = path;
    target.search = req.nextUrl.search;
    return NextResponse.redirect(target, 301);
  }
  return NextResponse.next();
}

export const config = {
  matcher: "/((?!api|_next|static|physis|favicon.ico|sidebar-icon.png|.*\\..*).*)",
};
