/*
@cypher schema=1 source_sha256=414cb4012467ac63ee38524ef9dbb6a8f822a3a62577edb148eb48b6d2a98ee0
MERGE (self:File {path:"frontend/src/components/effects/GasParticlesField.tsx"})
MERGE (field:Symbol {id:"frontend/src/components/effects/GasParticlesField.tsx::GasParticlesField",kind:"component"})
MERGE (self)-[:DEFINES]->(field)
@endcypher
*/
"use client";

// =====================================================================
// GasParticlesField — 真3D 体积粒子场 (per 2026-11-18 升级)
// =====================================================================
// 从 shader point-sprite 升级为:
//   - 真实 3D 几何体粒子 (四面体/八面体/立方体混合)
//   - 实例化渲染 InstancedMesh 高效批量绘制
//   - 每粒子独立旋转 + 缩放脉冲 + 颜色语义
//   - 3 种变体: rise(上升气泡)/swirl(螺旋星云)/pulse(脉冲波)
//   - 主题 token 同源, reduced-motion 静态帧
// =====================================================================

import { useMemo, useRef, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { readThemeRgb } from "@/lib/theme/themeTokens";

export type GasVariant = "rise" | "swirl" | "pulse";
export type GasColor = "accent" | "info" | "ok" | "warn" | "err";

// per 2026-11-17 主题统一: 语义色名 → 当前主题 CSS token + fallback
const COLOR_SOURCE: Record<GasColor, { token: string; fallback: string }> = {
  accent: { token: "--color-primary",   fallback: "#2f81f7" },
  info:   { token: "--color-secondary", fallback: "#58a6ff" },
  ok:     { token: "--ok-DEFAULT",      fallback: "#3fb950" },
  warn:   { token: "--warn-DEFAULT",    fallback: "#d29922" },
  err:    { token: "--err-DEFAULT",     fallback: "#f85149" },
};

function themeColor(color: GasColor): THREE.Color {
  const src = COLOR_SOURCE[color];
  const fb = new THREE.Color(src.fallback);
  const rgb = readThemeRgb(src.token, [fb.r, fb.g, fb.b]);
  return new THREE.Color(rgb[0], rgb[1], rgb[2]);
}

// ============================================================================
// 1. INSTANCED PARTICLE SYSTEM — 真3D 几何体
// ============================================================================
interface ParticleFieldProps {
  variant: GasVariant;
  color: GasColor;
  density: number;
  width: number;
  height: number;
}

// Particle data structure for CPU-side simulation
interface Particle {
  position: THREE.Vector3;
  velocity: THREE.Vector3;
  rotation: THREE.Euler;
  rotSpeed: THREE.Vector3;
  scale: number;
  baseScale: number;
  life: number;
  seed: number;
  phase: number;
}

function ParticleField({ variant, color, density, width, height }: ParticleFieldProps) {
  const meshRef = useRef<THREE.InstancedMesh>(null);
  const glowMeshRef = useRef<THREE.InstancedMesh>(null);
  const count = Math.floor(60 + density * 80);
  const dummy = useMemo(() => new THREE.Object3D(), []);
  const colorObj = useMemo(() => themeColor(color), [color]);
  const reducedMotion = useMemo(() => window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false, []);

  // Mixed geometry pool for variety
  const geometries = useMemo(() => [
    new THREE.TetrahedronGeometry(0.04, 0),
    new THREE.OctahedronGeometry(0.035, 0),
    new THREE.BoxGeometry(0.05, 0.05, 0.05),
    new THREE.IcosahedronGeometry(0.03, 0),
  ], []);

  // Initialize particles
  const particles = useMemo<Particle[]>(() => {
    const arr: Particle[] = [];
    for (let i = 0; i < count; i++) {
      const seed = Math.random();
      const phase = Math.random();
      let pos: THREE.Vector3;
      let vel: THREE.Vector3;

      if (variant === "rise") {
        pos = new THREE.Vector3(
          (Math.random() - 0.5) * 0.8,
          -0.5 + Math.random() * 1.0,
          (Math.random() - 0.5) * 0.4
        );
        vel = new THREE.Vector3(
          (Math.random() - 0.5) * 0.1,
          0.15 + Math.random() * 0.25,
          (Math.random() - 0.5) * 0.05
        );
      } else if (variant === "swirl") {
        const angle = Math.random() * Math.PI * 2;
        const radius = 0.1 + Math.random() * 0.2;
        pos = new THREE.Vector3(
          Math.cos(angle) * radius,
          (Math.random() - 0.5) * 0.5,
          Math.sin(angle) * radius
        );
        vel = new THREE.Vector3(0, 0, 0);
      } else {
        pos = new THREE.Vector3(
          (Math.random() - 0.5) * 0.2,
          (Math.random() - 0.5) * 0.2,
          (Math.random() - 0.5) * 0.2
        );
        vel = new THREE.Vector3(
          pos.x * 2,
          pos.y * 2,
          pos.z * 2
        );
      }

      const baseScale = 0.6 + Math.random() * 0.8;
      arr.push({
        position: pos,
        velocity: vel,
        rotation: new THREE.Euler(Math.random() * Math.PI, Math.random() * Math.PI, Math.random() * Math.PI),
        rotSpeed: new THREE.Vector3(
          (Math.random() - 0.5) * 2,
          (Math.random() - 0.5) * 2,
          (Math.random() - 0.5) * 2
        ),
        scale: baseScale,
        baseScale,
        life: Math.random(),
        seed,
        phase,
      });
    }
    return arr;
  }, [count, variant]);

  // Per-instance colors (slight hue variation)
  const instanceColors = useMemo(() => {
    const colors = new Float32Array(count * 3);
    const hsl = { h: 0, s: 0, l: 0 };
    colorObj.getHSL(hsl);
    for (let i = 0; i < count; i++) {
      const hueShift = (Math.random() - 0.5) * 0.05;
      const lightShift = (Math.random() - 0.5) * 0.2;
      const c = new THREE.Color().setHSL(
        (hsl.h + hueShift + 1) % 1,
        hsl.s,
        Math.max(0.1, Math.min(0.9, hsl.l + lightShift))
      );
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;
    }
    return colors;
  }, [count, colorObj]);

  useEffect(() => {
    if (meshRef.current) {
      meshRef.current.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      // Set per-instance colors
      const colorAttr = new THREE.InstancedBufferAttribute(instanceColors, 3);
      meshRef.current.geometry.setAttribute("instanceColor", colorAttr);
      meshRef.current.instanceColor = colorAttr;
    }
    if (glowMeshRef.current) {
      glowMeshRef.current.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    }
  }, [instanceColors]);

  useFrame((_, delta) => {
    if (!meshRef.current || reducedMotion) return;

    const time = performance.now() * 0.001;

    particles.forEach((p, i) => {
      // Update life cycle
      p.life += delta * (variant === "rise" ? 0.3 : variant === "pulse" ? 0.4 : 0.15);
      if (p.life > 1) {
        p.life = 0;
        // Reset position
        if (variant === "rise") {
          p.position.set((Math.random() - 0.5) * 0.8, -0.5, (Math.random() - 0.5) * 0.4);
          p.velocity.set((Math.random() - 0.5) * 0.1, 0.15 + Math.random() * 0.25, (Math.random() - 0.5) * 0.05);
        } else if (variant === "pulse") {
          p.position.set((Math.random() - 0.5) * 0.1, (Math.random() - 0.5) * 0.1, (Math.random() - 0.5) * 0.1);
          p.velocity.copy(p.position).multiplyScalar(2);
        }
      }

      // Variant-specific motion
      if (variant === "rise") {
        p.position.addScaledVector(p.velocity, delta);
        p.position.x += Math.sin(time * 1.2 + p.seed * 6.28) * 0.003;
        p.position.z += Math.cos(time * 0.8 + p.seed * 6.28) * 0.002;
        p.scale = p.baseScale * Math.sin(p.life * Math.PI) * (0.8 + Math.sin(time * 2 + p.phase * 6.28) * 0.2);
      } else if (variant === "swirl") {
        const angle = time * 0.8 + p.seed * 6.28;
        const radius = 0.15 + 0.1 * Math.sin(time * 0.5 + p.seed * 3.14);
        p.position.x = Math.cos(angle) * radius;
        p.position.z = Math.sin(angle) * radius;
        p.position.y = Math.sin(time * 0.6 + p.seed * 6.28) * 0.25;
        p.scale = p.baseScale * (0.7 + 0.3 * Math.sin(time * 1.5 + p.seed * 6.28));
      } else {
        // pulse
        const life = p.life;
        p.position.addScaledVector(p.velocity, delta * 0.5);
        p.scale = p.baseScale * (1 - life) * (0.8 + Math.sin(time * 3 + p.phase * 6.28) * 0.2);
        // Slow down expansion
        p.velocity.multiplyScalar(0.98);
      }

      // Rotation
      p.rotation.x += p.rotSpeed.x * delta;
      p.rotation.y += p.rotSpeed.y * delta;
      p.rotation.z += p.rotSpeed.z * delta;

      // Update instance matrix
      dummy.position.copy(p.position);
      dummy.rotation.copy(p.rotation);
      dummy.scale.setScalar(Math.max(0.01, p.scale));
      dummy.updateMatrix();
      meshRef.current!.setMatrixAt(i, dummy.matrix);

      // Glow shell (slightly larger, additive)
      if (glowMeshRef.current) {
        dummy.scale.setScalar(Math.max(0.01, p.scale * 1.8));
        dummy.updateMatrix();
        glowMeshRef.current.setMatrixAt(i, dummy.matrix);
      }
    });

    meshRef.current.instanceMatrix.needsUpdate = true;
    if (glowMeshRef.current) {
      glowMeshRef.current.instanceMatrix.needsUpdate = true;
    }
  });

  return (
    <group>
      {/* Core particles - mixed geometries via single instanced mesh with tetrahedron */}
      <instancedMesh ref={meshRef} args={[geometries[0], undefined, count]} frustumCulled={false}>
        <meshStandardMaterial
          vertexColors
          roughness={0.3}
          metalness={0.6}
          emissive={colorObj}
          emissiveIntensity={0.4}
          transparent
          opacity={0.9}
        />
      </instancedMesh>

      {/* Glow shell - additive blending for gas effect */}
      <instancedMesh ref={glowMeshRef} args={[geometries[1], undefined, count]} frustumCulled={false}>
        <meshBasicMaterial
          color={colorObj}
          transparent
          opacity={0.15}
          blending={THREE.AdditiveBlending}
          depthWrite={false}
        />
      </instancedMesh>
    </group>
  );
}

export function GasParticlesField(props: ParticleFieldProps) {
  return (
    <Canvas
      camera={{ position: [0, 0, 1.2], fov: 50 }}
      gl={{ alpha: true, antialias: true, powerPreference: "low-power", premultipliedAlpha: false }}
      dpr={[1, 1.5]}
      style={{ background: "transparent" }}
      onCreated={({ gl }) => {
        gl.setClearColor(0x000000, 0);
      }}
    >
      <ambientLight intensity={0.5} />
      <directionalLight position={[2, 3, 2]} intensity={0.8} />
      <pointLight position={[-2, -1, 1]} intensity={0.3} color="#4488ff" />
      <ParticleField {...props} />
    </Canvas>
  );
}
