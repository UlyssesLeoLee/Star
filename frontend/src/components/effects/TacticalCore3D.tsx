"use client";

import { useMemo, useRef, useState, useEffect, useCallback } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

// ============================================================================
// 1. ULTRA-LIGHTWEIGHT 3-RENDERED-2 (NPR CEL) SHADER
// ============================================================================
const CORE_VERTEX_SHADER = /* glsl */ `
  varying vec3 vNormal;
  varying vec3 vViewPos;
  varying vec3 vWorldPos;

  void main() {
    vNormal = normalize(normalMatrix * normal);
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    vViewPos = -mvPos.xyz;
    vWorldPos = (modelMatrix * vec4(position, 1.0)).xyz;
    gl_Position = projectionMatrix * mvPos;
  }
`;

const CORE_FRAGMENT_SHADER = /* glsl */ `
  uniform vec3 uBaseColor;
  uniform vec3 uShadowColor;
  uniform vec3 uKeyRimColor;
  uniform vec3 uCounterRimColor;
  uniform vec3 uLightPos;

  varying vec3 vNormal;
  varying vec3 vViewPos;
  varying vec3 vWorldPos;

  void main() {
    vec3 N = normalize(vNormal);
    vec3 L = normalize(uLightPos - vWorldPos);
    vec3 V = normalize(vViewPos);

    // 1. Half-Lambert Diffuse
    float NdotL = dot(N, L) * 0.5 + 0.5;

    // 2. Stepped Sharp Cel Ramp (2-stage anime quantization)
    float celFactor = step(0.46, NdotL) * 0.75 + 0.25;
    vec3 color = mix(uShadowColor, uBaseColor, celFactor);

    // 3. Dual-Rim Chroma Shift (Modern 3渲2 Anime Key/Counter Rim)
    float fresnel = 1.0 - max(0.0, dot(N, V));
    float keyRim = smoothstep(0.68, 0.74, fresnel) * smoothstep(0.1, 0.4, NdotL);
    float counterRim = smoothstep(0.78, 0.84, fresnel) * (1.0 - smoothstep(0.1, 0.4, NdotL));

    color += keyRim * uKeyRimColor * 1.3;
    color += counterRim * uCounterRimColor * 1.1;

    // 4. Hard Anime Specular
    vec3 H = normalize(L + V);
    float spec = step(0.94, pow(max(0.0, dot(N, H)), 18.0));
    color += spec * vec3(1.0, 1.0, 1.0) * 0.8;

    gl_FragColor = vec4(color, 1.0);
  }
`;

// Inverted Hull Comic Ink Outline
const OUTLINE_VERTEX = /* glsl */ `
  uniform float uThickness;
  void main() {
    vec4 mvPos = modelViewMatrix * vec4(position + normal * uThickness, 1.0);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const OUTLINE_FRAGMENT = /* glsl */ `
  uniform vec3 uOutlineColor;
  void main() {
    gl_FragColor = vec4(uOutlineColor, 1.0);
  }
`;

// ============================================================================
// 2. PROCEDURAL 3D MESH RIG (~180 POLYGONS)
// ============================================================================
function CoreMesh({
  status,
  isHovered,
  burstTime,
  isLight = false,
}: {
  status: "nominal" | "warning" | "overdrive";
  isHovered: boolean;
  burstTime: number;
  isLight?: boolean;
}) {
  const coreRef = useRef<THREE.Group>(null);
  const ringRef = useRef<THREE.Group>(null);

  const colors = useMemo(() => {
    if (isLight) {
      // ── Shōnen Studio Archival Paper Palette (Light Mode) ──
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#d48800"),
            shadow: new THREE.Color("#362100"),
            keyRim: new THREE.Color("#e60033"),
            counterRim: new THREE.Color("#0055ff"),
            outline: new THREE.Color("#0a0d14"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#e60033"),
            shadow: new THREE.Color("#3a040d"),
            keyRim: new THREE.Color("#0055ff"),
            counterRim: new THREE.Color("#d48800"),
            outline: new THREE.Color("#0a0d14"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#0055ff"),
            shadow: new THREE.Color("#061230"),
            keyRim: new THREE.Color("#e60033"),
            counterRim: new THREE.Color("#d48800"),
            outline: new THREE.Color("#0a0d14"),
          };
      }
    }

    // ── Neo-Tokyo Cyber Manga Void Palette (Dark Mode) ──
    switch (status) {
      case "warning":
        return {
          base: new THREE.Color("#ffc400"),
          shadow: new THREE.Color("#2e1c00"),
          keyRim: new THREE.Color("#ff184c"),
          counterRim: new THREE.Color("#00f0ff"),
          outline: new THREE.Color("#000000"),
        };
      case "overdrive":
        return {
          base: new THREE.Color("#ff184c"),
          shadow: new THREE.Color("#36030e"),
          keyRim: new THREE.Color("#00f0ff"),
          counterRim: new THREE.Color("#ffc400"),
          outline: new THREE.Color("#000000"),
        };
      case "nominal":
      default:
        return {
          base: new THREE.Color("#00f0ff"),
          shadow: new THREE.Color("#022230"),
          keyRim: new THREE.Color("#ff184c"),
          counterRim: new THREE.Color("#ffc400"),
          outline: new THREE.Color("#000000"),
        };
    }
  }, [status, isLight]);

  const coreGeom = useMemo(() => new THREE.IcosahedronGeometry(0.85, 0), []);
  const ringGeom = useMemo(() => new THREE.TorusGeometry(1.25, 0.05, 8, 24), []);

  const uniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uKeyRimColor: { value: colors.keyRim },
      uCounterRimColor: { value: colors.counterRim },
      uLightPos: { value: new THREE.Vector3(3, 4, 4) },
    }),
    [colors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uThickness: { value: 0.045 },
      uOutlineColor: { value: colors.outline },
    }),
    [colors.outline]
  );

  useFrame((_, delta) => {
    const isBursting = Date.now() - burstTime < 600;
    const speed = isBursting ? 7.5 : isHovered ? 2.5 : 1.0;

    if (coreRef.current) {
      coreRef.current.rotation.y += delta * 1.2 * speed;
      coreRef.current.rotation.x += delta * 0.6 * speed;
    }
    if (ringRef.current) {
      ringRef.current.rotation.x += delta * 1.6 * speed;
      ringRef.current.rotation.z -= delta * 0.9 * speed;
    }
  });

  return (
    <group>
      {/* 1. Central Icosahedron Facet Core */}
      <group ref={coreRef}>
        <mesh geometry={coreGeom}>
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={uniforms}
          />
        </mesh>
        <mesh geometry={coreGeom}>
          <shaderMaterial
            vertexShader={OUTLINE_VERTEX}
            fragmentShader={OUTLINE_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      {/* 2. Orbiting Gimbal Ring */}
      <group ref={ringRef}>
        <mesh geometry={ringGeom}>
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={uniforms}
          />
        </mesh>
        <mesh geometry={ringGeom}>
          <shaderMaterial
            vertexShader={OUTLINE_VERTEX}
            fragmentShader={OUTLINE_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>
    </group>
  );
}

// ============================================================================
// 3. EXPORTABLE TACTICAL CORE COMPONENT (40x40)
// ============================================================================
export interface TacticalCore3DProps {
  status?: "nominal" | "warning" | "overdrive";
  size?: number;
  className?: string;
}

export function TacticalCore3D({
  status = "nominal",
  size = 40,
  className = "",
}: TacticalCore3DProps) {
  const [mounted, setMounted] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [burstTime, setBurstTime] = useState(0);
  const [isPageVisible, setIsPageVisible] = useState(true);
  const [isLight, setIsLight] = useState(false);

  useEffect(() => {
    setMounted(true);

    const checkTheme = () => {
      setIsLight(document.documentElement.classList.contains("light"));
    };
    checkTheme();

    let observer: MutationObserver | null = null;
    if (typeof MutationObserver !== "undefined") {
      observer = new MutationObserver(checkTheme);
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["class"],
      });
    }

    const handleVisibilityChange = () => {
      setIsPageVisible(!document.hidden);
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      observer?.disconnect();
    };
  }, []);

  const handleClick = useCallback(() => {
    setBurstTime(Date.now());
  }, []);

  if (!mounted) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`bg-[var(--cel-surface-stage,#090d16)] border border-[var(--cel-ink,#000000)] grid place-items-center ${className}`}
      >
        <span className="size-2 bg-[var(--cel-cyan,#00f0ff)] rotate-45 border border-[var(--cel-ink,#000000)] animate-pulse" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      title="Tactical Neural Core [3渲2 真3D 陀螺仪]"
      aria-label="Tactical Neural Core"
      onClick={handleClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={{ width: size, height: size }}
      className={`relative cursor-pointer select-none group border-2 border-[var(--cel-ink,#000000)] bg-[var(--cel-surface-stage,#090d16)] cel-shadow overflow-hidden transition-transform duration-100 hover:scale-105 active:scale-95 ${className}`}
    >
      {/* 3渲2 背景微网格 */}
      <div className="absolute inset-0 bg-screentone-dense opacity-20 pointer-events-none z-0" />

      {/* 仅在页面可见时驱动 Canvas，切后台彻底休眠 */}
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 3.0], fov: 45 }}
          gl={{
            powerPreference: "low-power",
            antialias: false,
            alpha: true,
            depth: true,
            stencil: false,
          }}
          className="w-full h-full relative z-10 pointer-events-none"
        >
          <CoreMesh
            status={status}
            isHovered={isHovered}
            burstTime={burstTime}
            isLight={isLight}
          />
        </Canvas>
      )}

      {/* 触碰交互角标 */}
      <div className="absolute bottom-0 right-0 size-1.5 bg-[var(--cel-cyan,#00f0ff)] pointer-events-none z-20" />
    </div>
  );
}
