"use client";

// =====================================================================
// GasParticlesField — R3F 渲染实现 (client-only, ssr: false 隔离)
// =====================================================================
// 拆分原因 (per 2026-09-05 守门实证):
//   - R3F <Canvas> + useFrame 等 hook 在 Next.js prerender 阶段访问
//     browser-only API, 必须 ssr: false
//   - dynamic 包装必须作用于组件 (不能用 dynamic 包 hook), 所以把含
//     <Canvas> + useFrame 的组件抽到本文件
//   - 父文件 GasParticlesHint.tsx 只做 props 转发 + a11y 守卫 + 容器 div
//
// 已知缺口 (per 缺标比错标):
//   - Shader uniform uSize 在 width/height 变化时不重新创建, 走 useMemo
//     依赖重建; 但 uniform.value 引用复用, 算"半优化" (不影响视觉)
// =====================================================================

import { useMemo, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

export type GasVariant = "rise" | "swirl" | "pulse";
export type GasColor = "accent" | "info" | "ok" | "warn" | "err";

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
    vec3 pos = position;

    float t = uTime * 0.6 + aPhase;
    float seed = aSeed;

    if (uVariant < 0.5) {
      // rise: 粒子从底部上升, 带横向噪声漂移
      float life = mod(t * 0.4 + seed, 1.0);
      pos.x += sin(t * 1.2 + seed * 6.28) * 0.15 * life;
      pos.z += cos(t * 0.8 + seed * 6.28) * 0.08 * life;
      pos.y = -0.4 + life * 0.9;
      vAlpha = sin(life * 3.14159) * uDensity;
    } else if (uVariant < 1.5) {
      // swirl: 围绕中心螺旋
      float angle = t * 0.8 + seed * 6.28;
      float radius = 0.15 + 0.12 * sin(t * 0.5 + seed * 3.14);
      pos.x = cos(angle) * radius;
      pos.z = sin(angle) * radius;
      pos.y = sin(t * 0.6 + seed * 6.28) * 0.25;
      vAlpha = uDensity * (0.5 + 0.5 * sin(t * 1.5 + seed * 6.28));
    } else {
      // pulse: 中心扩散波纹
      float life = mod(t * 0.3 + seed * 0.3, 1.0);
      pos.x *= life * 1.5;
      pos.z *= life * 1.5;
      pos.y = (seed - 0.5) * 0.3 * (1.0 - life);
      vAlpha = (1.0 - life) * uDensity;
    }

    vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
    gl_Position = projectionMatrix * mvPosition;
    gl_PointSize = uSize * (300.0 / -mvPosition.z) * (0.6 + 0.4 * sin(t * 2.0 + seed * 6.28));
  }
`;

// ---- shader 片元: 软圆形 + 加法混合出气态感 ----
const FRAGMENT_SHADER = /* glsl */ `
  uniform vec3 uColor;
  varying float vAlpha;

  void main() {
    vec2 c = gl_PointCoord - 0.5;
    float d = length(c);
    if (d > 0.5) discard;
    float a = pow(1.0 - d * 2.0, 2.0) * vAlpha;
    gl_FragColor = vec4(uColor, a);
  }
`;

interface ParticleFieldProps {
  variant: GasVariant;
  color: GasColor;
  density: number;
  width: number;
  height: number;
}

function ParticleField({ variant, color, density, width, height }: ParticleFieldProps) {
  const pointsRef = useRef<THREE.Points>(null);
  const count = Math.floor(80 + density * 70);

  const { positions, seeds, phases } = useMemo(() => {
    const pos = new Float32Array(count * 3);
    const sd = new Float32Array(count);
    const ph = new Float32Array(count);
    for (let i = 0; i < count; i++) {
      pos[i * 3 + 0] = (Math.random() - 0.5);
      pos[i * 3 + 1] = (Math.random() - 0.5);
      pos[i * 3 + 2] = (Math.random() - 0.5);
      sd[i] = Math.random();
      ph[i] = Math.random();
    }
    return { positions: pos, seeds: sd, phases: ph };
  }, [count]);

  const uniforms = useMemo(() => ({
    uTime:    { value: 0 },
    uVariant: { value: variant === "rise" ? 0 : variant === "swirl" ? 1 : 2 },
    uDensity: { value: density },
    uSize:    { value: Math.min(width, height) * 0.4 },
    uColor:   { value: new THREE.Color(COLOR_HEX[color]) },
  }), [variant, density, width, height, color]);

  useFrame((_, delta) => {
    uniforms.uTime.value += delta;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" args={[positions, 3]} />
        <bufferAttribute attach="attributes-aSeed"    args={[seeds, 1]} />
        <bufferAttribute attach="attributes-aPhase"   args={[phases, 1]} />
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

export function GasParticlesField(props: ParticleFieldProps) {
  return (
    <Canvas
      camera={{ position: [0, 0, 1], fov: 50 }}
      gl={{ alpha: true, antialias: true, powerPreference: "low-power" }}
      dpr={[1, 1.5]}
      style={{ background: "transparent" }}
    >
      <ParticleField {...props} />
    </Canvas>
  );
}
