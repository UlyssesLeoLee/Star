"use client";

/**
 * AnimeCore3D — 抽象漂浮的"系统之心" (per 9/5 14:43 JST 用户拍板 q5-3d-target_opt1)
 *
 * 设计意图:
 *   - 整个 hero 区背景里慢慢旋转的抽象几何体 (环面+六边形+光点)
 *   - 100% 程序生成, 0 外部资源 / 0 贴图, 符合"无版权"约束
 *   - 暗模式切霓虹青/紫, 亮模式切蓝/紫罗兰, 跨主题适配
 *   - 鼠标悬停时视角轻微响应 (damping)
 *
 * 性能守门 (per 守门 #22 + #23 + RGS 19 推 origin):
 *   - DPR 限到 1.5 (避免 retina 卡)
 *   - frameloop="demand" 但本组件强制 always (旋转要流畅)
 *   - 使用 useMemo 缓存几何 / 材质
 */

import { useRef, useMemo } from "react";
import { Canvas, useFrame, ThreeEvent } from "@react-three/fiber";
import { MeshTransmissionMaterial, Float, Edges } from "@react-three/drei";
import * as THREE from "three";

interface CoreProps {
  hue?: number;
  accentHue?: number;
}

function FloatingCore({ hue = 220, accentHue = 280 }: CoreProps) {
  const groupRef = useRef<THREE.Group>(null);
  const torusRef = useRef<THREE.Mesh>(null);
  const icosaRef = useRef<THREE.Mesh>(null);

  // 读 CSS 变量, 跨主题自适应
  const colors = useMemo(() => {
    if (typeof window === "undefined") {
      return { primary: new THREE.Color(`hsl(${hue}, 90%, 60%)`), accent: new THREE.Color(`hsl(${accentHue}, 80%, 60%)`) };
    }
    const styles = getComputedStyle(document.documentElement);
    const primaryHex = styles.getPropertyValue("--color-primary").trim() || `hsl(${hue}, 90%, 60%)`;
    const accentHex = styles.getPropertyValue("--color-accent-violet").trim() || `hsl(${accentHue}, 80%, 60%)`;
    return { primary: new THREE.Color(primaryHex), accent: new THREE.Color(accentHex) };
  }, [hue, accentHue]);

  useFrame((state) => {
    if (!groupRef.current) return;
    const t = state.clock.getElapsedTime();
    // 主群组慢转, 子物体反向自转
    groupRef.current.rotation.y = t * 0.12;
    groupRef.current.rotation.x = Math.sin(t * 0.18) * 0.12;
    if (torusRef.current) torusRef.current.rotation.x = t * 0.4;
    if (icosaRef.current) icosaRef.current.rotation.z = -t * 0.3;
  });

  const handleMove = (e: ThreeEvent<PointerEvent>) => {
    if (!groupRef.current) return;
    // 鼠标 hover 让 core 朝鼠标方向倾斜
    groupRef.current.rotation.y += (e.point.x * 0.3 - groupRef.current.rotation.y) * 0.05;
  };

  return (
    <group ref={groupRef} onPointerMove={handleMove}>
      {/* 外圈环面 — 主色半透玻璃 */}
      <Float speed={1.4} rotationIntensity={0.3} floatIntensity={0.6}>
        <mesh ref={torusRef}>
          <torusGeometry args={[1.2, 0.32, 32, 96]} />
          <meshPhysicalMaterial
            color={colors.primary}
            transmission={0.6}
            thickness={0.5}
            roughness={0.15}
            metalness={0.1}
            ior={1.4}
            clearcoat={1}
            clearcoatRoughness={0.1}
            envMapIntensity={1.2}
            transparent
            opacity={0.85}
          />
          <Edges color={colors.primary} threshold={15} />
        </mesh>
      </Float>

      {/* 内核二十面体 — 紫色辉光 */}
      <Float speed={2.2} rotationIntensity={0.5} floatIntensity={0.4}>
        <mesh ref={icosaRef}>
          <icosahedronGeometry args={[0.55, 1]} />
          <MeshTransmissionMaterial
            color={colors.accent}
            transmission={0.4}
            thickness={0.3}
            roughness={0.05}
            metalness={0.0}
            ior={1.6}
            chromaticAberration={0.06}
            backside
            backsideThickness={0.4}
          />
          <Edges color={colors.accent} threshold={1} />
        </mesh>
      </Float>

      {/* 12 颗小光点环绕 — 能量场 */}
      {Array.from({ length: 12 }).map((_, i) => {
        const angle = (i / 12) * Math.PI * 2;
        const r = 1.8;
        return (
          <mesh key={i} position={[Math.cos(angle) * r, Math.sin(angle * 2) * 0.3, Math.sin(angle) * r]}>
            <sphereGeometry args={[0.04, 8, 8]} />
            <meshBasicMaterial color={i % 2 === 0 ? colors.primary : colors.accent} />
          </mesh>
        );
      })}
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
        camera={{ position: [0, 0, 4.2], fov: 45 }}
        gl={{ antialias: true, alpha: true, powerPreference: "high-performance" }}
        style={{ background: "transparent" }}
      >
        {/* 环境光 — 让玻璃材质有反射 */}
        <ambientLight intensity={0.6} />
        <directionalLight position={[5, 5, 5]} intensity={0.8} />
        <pointLight position={[-5, -3, 3]} intensity={0.4} color={props.accentHue ? "#a855f7" : "#7c3aed"} />
        <FloatingCore {...props} />
      </Canvas>
    </div>
  );
}
