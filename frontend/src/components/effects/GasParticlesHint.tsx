"use client";

// =====================================================================
// GasParticlesHint — 气态粒子特效, 用于提示"下一步"操作
// =====================================================================
// Per 2026-09-05 Ulysses 拍板: 通用组件 + 3-5 个场景全量接入.
// 设计目标:
//   - 轻量 GPU 粒子 (Points + 自定义 shader, 80-150 粒子/实例)
//   - 3 种 variant 覆盖 4 个场景:
//     · rise  — 上升气流 (按钮上方, 提示"按这个"), 用于场景 1+4
//     · swirl — 螺旋环绕 (列/项周围), 用于场景 2
//     · pulse — 脉冲扩散 (高亮一次性提示), 用于场景 3
//   - color token 走 Tailwind design tokens (accent/info/ok/warn/err)
//   - `active` 切换立即启停, 不依赖 useFrame 重启
//   - dynamic import 包裹避免 SSR 报错
//
// 已知缺口 (per 缺标比错标):
//   - 仅支持 absolute 定位在父容器右上角, 尺寸固定 120x120 / 200x120
//     父容器必须 `position: relative` + 留出右上角空间
//   - 移动端 < 640px 自动隐藏 (touch 设备粒子干扰阅读)
//   - 不做色彩对比度调整 (跟 theme 配色), 由父容器控色
//   - prefers-reduced-motion 直接禁用 (a11y)
// =====================================================================

import { useMemo, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

export type GasVariant = "rise" | "swirl" | "pulse";
export type GasColor = "accent" | "info" | "ok" | "warn" | "err";

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

const COLOR_HEX: Record<GasColor, string> = {
  accent: "#2f81f7",
  info:    "#58a6ff",
  ok:      "#3fb950",
  warn:    "#d29922",
  err:     "#f85149",
};

// ---- shader 顶点: 粒子位置 + 大小 ----
const VERTEX_SHADER = /* glsl */ `
  uniform float uTime;
  uniform float uVariant;  // 0=rise 1=swirl 2=pulse
  uniform float uDensity;
  uniform float uSize;
  attribute float aSeed;   // 0..1 每粒子固定
  attribute float aPhase;  // 0..1 phase offset

  varying float vAlpha;

  void main() {
    // base 位置 (-0.5..0.5 cube -> 居中 1x1 区域)
    vec3 pos = position;

    float t = uTime * 0.6 + aPhase;
    float seed = aSeed;

    if (uVariant < 0.5) {
      // ---- rise: 粒子从底部上升, 带横向噪声漂移 ----
      float life = mod(t * 0.4 + seed, 1.0);
      pos.x += sin(t * 1.2 + seed * 6.28) * 0.15 * life;
      pos.z += cos(t * 0.8 + seed * 6.28) * 0.08 * life;
      pos.y = -0.4 + life * 0.9;     // -0.4..0.5
      // 上升时透明度淡入淡出
      vAlpha = sin(life * 3.14159) * uDensity;
    } else if (uVariant < 1.5) {
      // ---- swirl: 围绕中心螺旋, 半径 + 高度周期变化 ----
      float angle = t * 0.8 + seed * 6.28;
      float radius = 0.15 + 0.12 * sin(t * 0.5 + seed * 3.14);
      pos.x = cos(angle) * radius;
      pos.z = sin(angle) * radius;
      pos.y = sin(t * 0.6 + seed * 6.28) * 0.25;
      vAlpha = uDensity * (0.5 + 0.5 * sin(t * 1.5 + seed * 6.28));
    } else {
      // ---- pulse: 中心扩散, 一次性波纹 (循环) ----
      float life = mod(t * 0.3 + seed * 0.3, 1.0);
      pos.x *= life * 1.5;
      pos.z *= life * 1.5;
      pos.y = (seed - 0.5) * 0.3 * (1.0 - life);
      vAlpha = (1.0 - life) * uDensity;
    }

    vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
    gl_Position = projectionMatrix * mvPosition;
    // 远小近大 (perspective points size)
    gl_PointSize = uSize * (300.0 / -mvPosition.z) * (0.6 + 0.4 * sin(t * 2.0 + seed * 6.28));
  }
`;

// ---- shader 片元: 软圆形 + 加法混合出气态感 ----
const FRAGMENT_SHADER = /* glsl */ `
  uniform vec3 uColor;
  varying float vAlpha;

  void main() {
    // 中心 (0.5, 0.5), 边缘 = 0
    vec2 c = gl_PointCoord - 0.5;
    float d = length(c);
    if (d > 0.5) discard;
    // 软衰减 (平方)
    float a = pow(1.0 - d * 2.0, 2.0) * vAlpha;
    gl_FragColor = vec4(uColor, a);
  }
`;

interface ParticleFieldProps {
  variant: GasVariant;
  colorHex: string;
  density: number;
  width: number;
  height: number;
}

function ParticleField({ variant, colorHex, density, width, height }: ParticleFieldProps) {
  const pointsRef = useRef<THREE.Points>(null);

  // 粒子数 80..150 按 density 缩放
  const count = Math.floor(80 + density * 70);

  // attributes (useMemo 保证稳定)
  const { positions, seeds, phases } = useMemo(() => {
    const pos = new Float32Array(count * 3);
    const sd = new Float32Array(count);
    const ph = new Float32Array(count);
    for (let i = 0; i < count; i++) {
      // 起始位置在 -0.5..0.5 立方体内, 由 vertex shader 改写
      pos[i * 3 + 0] = (Math.random() - 0.5);
      pos[i * 3 + 1] = (Math.random() - 0.5);
      pos[i * 3 + 2] = (Math.random() - 0.5);
      sd[i] = Math.random();
      ph[i] = Math.random();
    }
    return { positions: pos, seeds: sd, phases: ph };
  }, [count]);

  // uniforms (useMemo 避免每帧重建)
  const uniforms = useMemo(() => ({
    uTime:    { value: 0 },
    uVariant: { value: variant === "rise" ? 0 : variant === "swirl" ? 1 : 2 },
    uDensity: { value: density },
    uSize:    { value: Math.min(width, height) * 0.4 },
    uColor:   { value: new THREE.Color(colorHex) },
  }), [variant, density, width, height, colorHex]);

  useFrame((_, delta) => {
    uniforms.uTime.value += delta;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          args={[positions, 3]}
        />
        <bufferAttribute
          attach="attributes-aSeed"
          args={[seeds, 1]}
        />
        <bufferAttribute
          attach="attributes-aPhase"
          args={[phases, 1]}
        />
      </bufferGeometry>
      <shaderMaterial
        uniforms={uniforms}
        vertexShader={VERTEX_SHADER}
        fragmentShader={FRAGMENT_SHADER}
        transparent
        depthWrite={false}
        blending={THREE.AdditiveBlending}
      />
    </points>
  );
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
  // SSR / reduced-motion / touch 设备直接不渲染 (省 GPU + a11y)
  if (typeof window !== "undefined") {
    const prefersReduced = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    const isTouch = window.matchMedia?.("(hover: none) and (pointer: coarse)").matches;
    if (prefersReduced || isTouch || !active) return null;
  }

  const colorHex = COLOR_HEX[color];

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
      <Canvas
        camera={{ position: [0, 0, 1], fov: 50 }}
        gl={{ alpha: true, antialias: true, powerPreference: "low-power" }}
        dpr={[1, 1.5]}
        style={{ background: "transparent" }}
      >
        <ParticleField
          variant={variant}
          colorHex={colorHex}
          density={density}
          width={width}
          height={height}
        />
      </Canvas>
    </div>
  );
}
