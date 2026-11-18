/*
@cypher schema=1 source_sha256=4bf0312884d0898b4c0801bbdc0be67047bcfc1f28d84989056f843e9c87cc75
MERGE (self:File {path:"frontend/src/app/automation-debug/components/AnimeCore3D.tsx"})
MERGE (core:Symbol {id:"frontend/src/app/automation-debug/components/AnimeCore3D.tsx::AnimeCore3D",kind:"component"})
MERGE (self)-[:DEFINES]->(core)
@endcypher
*/
"use client";

/**
 * AnimeCore3D — "系统之心" 聚变反应堆 (per 2026-11-18 纯3D人格化升级)
 *
 * 从抽象环面+二十面体升级为个性鲜明的多体反应堆:
 *   - 中央聚变核心: 二十面体 + 内嵌发光球 + 脉冲缩放
 *   - 三层陀螺稳定环: 不同半径/倾角/转速, 独立万向节
 *   - 6 能量卫星: 椭圆轨道 + 拖尾光点 + 相位错开
 *   - 底部能量基座环: 呼吸脉冲
 *   - 鼠标 hover 倾斜响应 + 点击爆发加速
 *
 * 设计意图:
 *   - 100% 程序生成, 0 外部资源 / 0 贴图, 符合"无版权"约束
 *   - 颜色全部从主题 token 读取 (--color-primary / --color-accent-violet),
 *     跨主题自适应, html class 变化实时重读
 *
 * 性能守门 (per 守门 #22 + #23):
 *   - DPR 限到 1.5
 *   - useMemo 缓存几何 / 材质
 *   - prefers-reduced-motion 静态渲染
 */

import { useRef, useMemo, useState, useEffect } from "react";
import { Canvas, useFrame, ThreeEvent } from "@react-three/fiber";
import { Float, Edges } from "@react-three/drei";
import * as THREE from "three";
import { readThemeRgb } from "@/lib/theme/themeTokens";

interface CoreProps {
  hue?: number;
  accentHue?: number;
}

function ReactorCore({ hue = 220, accentHue = 280 }: CoreProps) {
  const groupRef = useRef<THREE.Group>(null);
  const coreRef = useRef<THREE.Mesh>(null);
  const innerGlowRef = useRef<THREE.Mesh>(null);
  const ring1Ref = useRef<THREE.Mesh>(null);
  const ring2Ref = useRef<THREE.Mesh>(null);
  const ring3Ref = useRef<THREE.Mesh>(null);
  const baseRingRef = useRef<THREE.Mesh>(null);
  const satelliteRefs = useRef<THREE.Mesh[]>([]);
  const trailRefs = useRef<THREE.Mesh[]>([]);
  const burstRef = useRef(0);

  const reducedMotion = useMemo(
    () => typeof window !== "undefined" && (window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false),
    []
  );

  // 读 CSS 变量, 跨主题自适应; 监听 html class 变化重读 token
  const [themeRev, setThemeRev] = useState(0);
  useEffect(() => {
    const obs = new MutationObserver(() => setThemeRev((r) => r + 1));
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    return () => obs.disconnect();
  }, []);

  const colors = useMemo(() => {
    const primary = readThemeRgb("--color-primary", [0.184, 0.506, 0.969]);
    const accent = readThemeRgb("--color-accent-violet", [0.486, 0.227, 0.929]);
    const ok = readThemeRgb("--ok-DEFAULT", [0.247, 0.725, 0.314]);
    return {
      primary: new THREE.Color(primary[0], primary[1], primary[2]),
      accent: new THREE.Color(accent[0], accent[1], accent[2]),
      ok: new THREE.Color(ok[0], ok[1], ok[2]),
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hue, accentHue, themeRev]);

  // Geometries
  const coreGeom = useMemo(() => new THREE.IcosahedronGeometry(0.55, 1), []);
  const glowGeom = useMemo(() => new THREE.SphereGeometry(0.3, 16, 16), []);
  const ring1Geom = useMemo(() => new THREE.TorusGeometry(1.0, 0.045, 12, 64), []);
  const ring2Geom = useMemo(() => new THREE.TorusGeometry(1.35, 0.035, 12, 72), []);
  const ring3Geom = useMemo(() => new THREE.TorusGeometry(1.7, 0.025, 10, 80), []);
  const baseGeom = useMemo(() => new THREE.TorusGeometry(2.0, 0.06, 10, 64), []);
  const satGeom = useMemo(() => new THREE.OctahedronGeometry(0.09, 0), []);
  const trailGeom = useMemo(() => new THREE.SphereGeometry(0.035, 6, 6), []);

  useFrame((state, delta) => {
    if (!groupRef.current || reducedMotion) return;
    const t = state.clock.getElapsedTime();

    // Burst decay
    if (burstRef.current > 0) burstRef.current = Math.max(0, burstRef.current - delta * 1.5);
    const speedMult = 1 + burstRef.current * 4;

    // Main group slow rotation + hover tilt handled in pointer move
    groupRef.current.rotation.y += delta * 0.12 * speedMult;
    groupRef.current.rotation.x = Math.sin(t * 0.18) * 0.12;

    // Core pulse
    if (coreRef.current) {
      coreRef.current.rotation.z = -t * 0.3 * speedMult;
      coreRef.current.rotation.x = t * 0.2 * speedMult;
      const pulse = 1 + Math.sin(t * 2.5) * 0.08 + burstRef.current * 0.25;
      coreRef.current.scale.setScalar(pulse);
    }
    if (innerGlowRef.current) {
      const glowPulse = 0.8 + Math.sin(t * 3.0) * 0.25 + burstRef.current * 0.5;
      innerGlowRef.current.scale.setScalar(glowPulse);
      const mat = innerGlowRef.current.material as THREE.MeshBasicMaterial;
      mat.opacity = 0.5 + Math.sin(t * 3.0) * 0.3 + burstRef.current * 0.3;
    }

    // Three gyroscopic stabilizer rings - independent axes/speeds
    if (ring1Ref.current) {
      ring1Ref.current.rotation.x = t * 0.5 * speedMult;
      ring1Ref.current.rotation.y = t * 0.2 * speedMult;
    }
    if (ring2Ref.current) {
      ring2Ref.current.rotation.y = -t * 0.35 * speedMult;
      ring2Ref.current.rotation.z = t * 0.25 * speedMult;
    }
    if (ring3Ref.current) {
      ring3Ref.current.rotation.z = -t * 0.22 * speedMult;
      ring3Ref.current.rotation.x = t * 0.15 * speedMult;
    }

    // Base ring breathing
    if (baseRingRef.current) {
      const breathe = 1 + Math.sin(t * 1.2) * 0.04;
      baseRingRef.current.scale.setScalar(breathe);
      const mat = baseRingRef.current.material as THREE.MeshBasicMaterial;
      mat.opacity = 0.25 + Math.sin(t * 1.2) * 0.12 + burstRef.current * 0.3;
    }

    // 6 energy satellites on elliptical orbits
    satelliteRefs.current.forEach((sat, i) => {
      if (!sat) return;
      const phase = (i / 6) * Math.PI * 2;
      const orbitT = t * (0.4 + i * 0.05) * speedMult + phase;
      const rx = 1.9 + (i % 3) * 0.15;
      const rz = 1.5 + (i % 2) * 0.2;
      sat.position.set(
        Math.cos(orbitT) * rx,
        Math.sin(orbitT * 0.7 + phase) * 0.5,
        Math.sin(orbitT) * rz
      );
      sat.rotation.x += delta * 2 * speedMult;
      sat.rotation.y += delta * 1.5 * speedMult;
      const satScale = 0.8 + Math.sin(t * 2 + phase) * 0.25 + burstRef.current * 0.4;
      sat.scale.setScalar(satScale);

      // Trail particle lags behind
      const trail = trailRefs.current[i];
      if (trail) {
        const lagT = orbitT - 0.25;
        trail.position.set(
          Math.cos(lagT) * rx,
          Math.sin(lagT * 0.7 + phase) * 0.5,
          Math.sin(lagT) * rz
        );
        const trailScale = 0.6 + Math.sin(t * 2.5 + phase) * 0.2;
        trail.scale.setScalar(trailScale);
      }
    });
  });

  const handleMove = (e: ThreeEvent<PointerEvent>) => {
    if (!groupRef.current) return;
    groupRef.current.rotation.y += (e.point.x * 0.3 - groupRef.current.rotation.y) * 0.05;
  };

  const handleClick = () => {
    burstRef.current = 1;
  };

  return (
    <group ref={groupRef} onPointerMove={handleMove} onClick={handleClick}>
      {/* Central fusion core */}
      <Float speed={1.6} rotationIntensity={0.2} floatIntensity={0.4}>
        <mesh ref={coreRef} geometry={coreGeom}>
          <meshPhysicalMaterial
            color={colors.accent}
            emissive={colors.accent}
            emissiveIntensity={0.5}
            roughness={0.1}
            metalness={0.3}
            clearcoat={1}
            clearcoatRoughness={0.1}
            transparent
            opacity={0.9}
          />
          <Edges color={colors.accent} threshold={1} />
        </mesh>
        {/* Inner energy glow */}
        <mesh ref={innerGlowRef} geometry={glowGeom}>
          <meshBasicMaterial
            color={colors.primary}
            transparent
            opacity={0.6}
            blending={THREE.AdditiveBlending}
            depthWrite={false}
          />
        </mesh>
      </Float>

      {/* Gyroscopic stabilizer ring 1 - primary color, tilted */}
      <mesh ref={ring1Ref} geometry={ring1Geom} rotation={[Math.PI / 3, 0, 0]}>
        <meshStandardMaterial
          color={colors.primary}
          emissive={colors.primary}
          emissiveIntensity={0.4}
          roughness={0.2}
          metalness={0.6}
        />
      </mesh>

      {/* Ring 2 - accent, counter-rotating */}
      <mesh ref={ring2Ref} geometry={ring2Geom} rotation={[0, Math.PI / 4, Math.PI / 6]}>
        <meshStandardMaterial
          color={colors.accent}
          emissive={colors.accent}
          emissiveIntensity={0.35}
          roughness={0.25}
          metalness={0.5}
        />
      </mesh>

      {/* Ring 3 - ok green, outermost */}
      <mesh ref={ring3Ref} geometry={ring3Geom} rotation={[Math.PI / 2.5, Math.PI / 5, 0]}>
        <meshStandardMaterial
          color={colors.ok}
          emissive={colors.ok}
          emissiveIntensity={0.3}
          roughness={0.3}
          metalness={0.4}
          transparent
          opacity={0.8}
        />
      </mesh>

      {/* Base energy pedestal ring */}
      <mesh ref={baseRingRef} geometry={baseGeom} rotation={[Math.PI / 2, 0, 0]} position={[0, -1.2, 0]}>
        <meshBasicMaterial
          color={colors.primary}
          transparent
          opacity={0.3}
          blending={THREE.AdditiveBlending}
          depthWrite={false}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* 6 energy satellites + trails */}
      {Array.from({ length: 6 }).map((_, i) => (
        <group key={i}>
          <mesh
            ref={(el) => { if (el) satelliteRefs.current[i] = el; }}
            geometry={satGeom}
          >
            <meshStandardMaterial
              color={i % 2 === 0 ? colors.primary : colors.accent}
              emissive={i % 2 === 0 ? colors.primary : colors.accent}
              emissiveIntensity={0.7}
              roughness={0.15}
              metalness={0.5}
            />
          </mesh>
          <mesh
            ref={(el) => { if (el) trailRefs.current[i] = el; }}
            geometry={trailGeom}
          >
            <meshBasicMaterial
              color={i % 2 === 0 ? colors.primary : colors.accent}
              transparent
              opacity={0.4}
              blending={THREE.AdditiveBlending}
              depthWrite={false}
            />
          </mesh>
        </group>
      ))}
    </group>
  );
}

/**
 * 主组件 — 用 dynamic({ ssr: false }) 在 page 引入, 避免污染 SSR
 */
export default function AnimeCore3D(props: CoreProps) {
  return (
    <div
      aria-hidden
      className="absolute inset-0 -z-10 pointer-events-auto"
      style={{ touchAction: "none" }}
    >
      <Canvas
        dpr={[1, 1.5]}
        camera={{ position: [0, 0, 4.6], fov: 45 }}
        gl={{ antialias: true, alpha: true, powerPreference: "high-performance" }}
        style={{ background: "transparent" }}
      >
        <ambientLight intensity={0.6} />
        <directionalLight position={[5, 5, 5]} intensity={0.8} />
        <pointLight
          position={[-5, -3, 3]}
          intensity={0.4}
          color={new THREE.Color(...readThemeRgb("--color-accent-violet", [0.486, 0.227, 0.929]))}
        />
        <ReactorCore {...props} />
      </Canvas>
    </div>
  );
}
