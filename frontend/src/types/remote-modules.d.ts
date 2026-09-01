// Ambient module declarations for ESM-only packages without types
// (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
//
// @novnc/novnc v1.7: package.json `exports: "./core/rfb.js"` 但无 .d.ts
// 动态 import 在 runtime 解析成功, 但 TS 静态检查需要 ambient 声明

declare module "@novnc/novnc/core/rfb.js" {
  // noVNC RFB class (per docs/API.md)
  // 简化类型: 只声明我用到的 surface
  export default class RFB {
    constructor(
      target: HTMLElement,
      url: string,
      options?: {
        credentials?: { password?: string; username?: string };
        shared?: boolean;
        repeaterID?: string;
        wsProtocols?: string[];
      },
    );
    scaleViewport: boolean;
    resizeSession: boolean;
    addEventListener(
      type: "connect" | "disconnect" | "securityfailure" | "credentialsrequired" | "clipboard" | "bell" | "desktopname" | "capabilities",
      listener: (e: CustomEvent) => void,
    ): void;
    disconnect(): void;
  }
}

declare module "@novnc/novnc" {
  // 主入口别名 (per package.json exports)
  export { default } from "@novnc/novnc/core/rfb.js";
}

declare module "@xterm/xterm/css/xterm.css" {
  // CSS 模块: 仅用于 side-effect import, 无运行时导出
  const _: void;
  export default _;
}
