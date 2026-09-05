"use client";

/**
 * useCountUp — 数字滚动 (缓动 0 → target) (per 9/5 14:41 JST 用户拍板 §4.4 微交互)
 *
 * 用途: KPI 胶囊进入视野时数字 0 → target 缓动, 给"工业仪表盘"那一下仪式感
 * 时长: 700ms, cubic-bezier(0.16, 1, 0.3, 1) 跟现有 theme transition 对齐
 * 性能: requestAnimationFrame, 卸载自动 cancel
 */

import { useEffect, useState } from "react";

export function useCountUp(target: number, durationMs = 700, enabled = true): number {
  const [value, setValue] = useState(0);

  useEffect(() => {
    if (!enabled) {
      setValue(target);
      return;
    }
    let raf = 0;
    const start = performance.now();
    const ease = (t: number) => {
      // cubic-bezier(0.16, 1, 0.3, 1) 的近似 — "out-quint" feel
      return 1 - Math.pow(1 - t, 5);
    };
    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / durationMs);
      setValue(Math.round(ease(t) * target));
      if (t < 1) raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [target, durationMs, enabled]);

  return value;
}
