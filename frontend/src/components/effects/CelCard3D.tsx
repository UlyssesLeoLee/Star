"use client";

import { useRef, useState, useCallback, type ReactNode, type HTMLAttributes } from "react";

export interface CelCard3DProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  className?: string;
  maxTilt?: number; // Maximum tilt angle in degrees, default 7
  glint?: boolean; // Stepped anime specular glint tracking cursor
  chamfer?: boolean; // Tactical corner cut
  elevation?: "sm" | "md" | "lg";
}

/**
 * CelCard3D — 零 WebGL 消耗的高性能 3渲2 真实物理 3D 触觉卡片
 *
 * 核心设计优势:
 * 1. 0 WebGL 上下文开销，完全基于 GPU 硬件合成层 CSS 3D (perspective / preserve-3d)
 * 2. 实时跟随光标计算微空间偏转 (rotateX / rotateY) 与动态阶梯光斑 (Stepped Anime Glint)
 * 3. 内部元素可利用 style={{ transform: "translateZ(14px)" }} 获得真实的物理空间视差
 * 4. 零掉帧：静止即零消耗，鼠标移出平滑弹回原位
 */
export function CelCard3D({
  children,
  className = "",
  maxTilt = 7,
  glint = true,
  chamfer = false,
  elevation = "md",
  ...rest
}: CelCard3DProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [tilt, setTilt] = useState({ rotateX: 0, rotateY: 0 });
  const [glintPos, setGlintPos] = useState({ x: 50, y: 50, opacity: 0 });
  const [isHovered, setIsHovered] = useState(false);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (!cardRef.current) return;
      const rect = cardRef.current.getBoundingClientRect();
      const clientX = e.clientX - rect.left;
      const clientY = e.clientY - rect.top;

      // Normalization from -1 to 1
      const normX = (clientX / rect.width) * 2 - 1;
      const normY = (clientY / rect.height) * 2 - 1;

      // Calculate tilt angles (X tilt from vertical position, Y tilt from horizontal position)
      const rotX = -normY * maxTilt;
      const rotY = normX * maxTilt;

      setTilt({ rotateX: rotX, rotateY: rotY });

      if (glint) {
        setGlintPos({
          x: (clientX / rect.width) * 100,
          y: (clientY / rect.height) * 100,
          opacity: 1,
        });
      }
    },
    [maxTilt, glint]
  );

  const handleMouseEnter = useCallback(() => {
    setIsHovered(true);
  }, []);

  const handleMouseLeave = useCallback(() => {
    setIsHovered(false);
    setTilt({ rotateX: 0, rotateY: 0 });
    setGlintPos((prev) => ({ ...prev, opacity: 0 }));
  }, []);

  const shadowClass =
    elevation === "lg"
      ? "shadow-[inset_1px_1px_0_0_var(--cel-bevel-hi),inset_-1px_-1px_0_0_var(--cel-bevel-lo),6px_6px_0px_0px_var(--cel-shadow-color,#000000)]"
      : elevation === "sm"
      ? "shadow-[inset_1px_1px_0_0_var(--cel-bevel-hi),inset_-1px_-1px_0_0_var(--cel-bevel-lo),2px_2px_0px_0px_var(--cel-shadow-color,#000000)]"
      : "shadow-[inset_1px_1px_0_0_var(--cel-bevel-hi),inset_-1px_-1px_0_0_var(--cel-bevel-lo),4px_4px_0px_0px_var(--cel-shadow-color,#000000)]";

  return (
    <div
      ref={cardRef}
      onMouseMove={handleMouseMove}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      style={{
        perspective: "1100px",
      }}
      className="inline-block w-full transition-transform duration-150 ease-out select-none"
      {...rest}
    >
      <div
        className={`relative border-2 border-[var(--cel-ink,#000000)] bg-[var(--cel-surface-card,#0f1422)] text-[var(--cel-text-primary,#ffffff)] ${shadowClass} ${
          chamfer ? "clip-hud-corner" : ""
        } ${className}`}
        style={{
          transformStyle: "preserve-3d",
          transform: isHovered
            ? `rotateX(${tilt.rotateX.toFixed(2)}deg) rotateY(${tilt.rotateY.toFixed(2)}deg) translateZ(8px)`
            : "rotateX(0deg) rotateY(0deg) translateZ(0px)",
          transition: isHovered
            ? "transform 80ms ease-out, box-shadow 150ms ease"
            : "transform 350ms cubic-bezier(0.16, 1, 0.3, 1), box-shadow 250ms ease",
        }}
      >
        {/* 3渲2 阶梯式光标动态高光 (Stepped Anime Specular Glint) */}
        {glint && (
          <div
            className="absolute inset-0 pointer-events-none transition-opacity duration-200 z-20 overflow-hidden"
            style={{
              opacity: glintPos.opacity,
              background: `radial-gradient(circle 120px at ${glintPos.x}% ${glintPos.y}%, var(--cel-glint-color, rgba(255,255,255,0.28)) 0%, var(--cel-glint-rim, rgba(0,240,255,0.16)) 40%, transparent 70%)`,
            }}
          />
        )}

        {/* 战术微点阵网底 (Screentone Overlay) */}
        <div className="absolute inset-0 bg-screentone-dense opacity-10 pointer-events-none z-0" />

        {/* 卡片内容区 (可通过 translateZ 自由挂载悬浮层) */}
        <div
          className="relative z-10 w-full h-full"
          style={{ transform: "translateZ(0px)" }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
