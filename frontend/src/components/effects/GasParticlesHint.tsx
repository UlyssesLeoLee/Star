"use client";

// =====================================================================
// GasParticlesHint — 气态粒子特效, 用于提示"下一步"操作 (SSR-safe wrapper)
// =====================================================================
// Per 2026-09-05 Ulysses 拍板: 通用组件 + 3-5 个场景全量接入.
//
// 设计原则 (per 守门 #1 + 缺标比错标):
//   - 本文件只做 props 转发 + 容器 div + a11y 守卫 (reducemotion/touch/active)
//   - 真正的 R3F <Canvas> + 自定义 shader 逻辑抽到子文件 GasParticlesField.tsx
//   - 子文件用 next/dynamic { ssr: false } 加载, 避免 Next.js prerender 报
//     "WebGL context not available" / "window is not defined" (per 2026-09-05 build
//     实证: 20+ 页面 prerender 失败, 根因 = R3F 在 SSR 阶段访问 browser-only API)
//
// 已知缺口 (per 缺标比错标):
//   - 仅支持 absolute 定位在父容器右上角, 父容器必须 `position: relative`
//   - 移动端 < 640px 自动隐藏 (touch 设备粒子干扰阅读)
//   - 不做色彩对比度调整 (跟 theme 配色), 由父容器控色
//   - prefers-reduced-motion 直接禁用 (a11y)
// =====================================================================

import dynamic from "next/dynamic";
import type { GasVariant, GasColor } from "./GasParticlesField";

// 真正用 R3F + 自定义 shader 的子组件, ssr: false 隔离 Next.js prerender
const GasParticlesField = dynamic(
  () => import("./GasParticlesField").then((m) => m.GasParticlesField),
  { ssr: false, loading: () => null },
);

export type { GasVariant, GasColor };

export interface GasParticlesHintProps {
  variant?: GasVariant;
  color?: GasColor;
  /** 0..1, 默认 0.6 (轻量, 不抢戏) */
  density?: number;
  /** 是否激活; false 时不渲染 Canvas (省 GPU) */
  active?: boolean;
  /** 覆盖默认 size (默认 120x120) */
  width?: number;
  /** 覆盖默认 size (默认 120) */
  height?: number;
  /** 右上角偏移, 默认 -8px / -8px (让粒子飘在元素外) */
  offsetX?: number;
  offsetY?: number;
  className?: string;
}

export function GasParticlesHint({
  variant = "rise",
  color = "accent",
  density = 0.6,
  active = true,
  width = 120,
  height = 120,
  offsetX = -8,
  offsetY = -8,
  className,
}: GasParticlesHintProps) {
  // SSR 阶段直接不渲染 div, 客户端 mount 后再判定 + 渲染
  // (per 守门 #1: SSR 安全第一, 避免 hydration mismatch)
  if (typeof window === "undefined") return null;

  // reduced-motion / touch 设备 / active=false 直接不渲染
  const prefersReduced = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  const isTouch = window.matchMedia?.("(hover: none) and (pointer: coarse)").matches;
  if (prefersReduced || isTouch || !active) return null;

  return (
    <div
      aria-hidden
      data-testid="gas-particles-hint"
      data-variant={variant}
      data-color={color}
      className={className}
      style={{
        position: "absolute",
        top: offsetY,
        right: offsetX,
        width: `${width}px`,
        height: `${height}px`,
        pointerEvents: "none",
        zIndex: 5,
      }}
    >
      <GasParticlesField
        variant={variant}
        color={color}
        density={density}
        width={width}
        height={height}
      />
    </div>
  );
}
