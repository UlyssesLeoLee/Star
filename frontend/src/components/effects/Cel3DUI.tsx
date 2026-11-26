/*
@cypher schema=1 source_sha256=7b72a0e5041490c4d9b2d8e4929a4c5ecf9c9f2904f5883806b00fb1c142e399
MERGE (self:File {path:"frontend/src/components/effects/Cel3DUI.tsx"})
MERGE (btn:Symbol {id:"frontend/src/components/effects/Cel3DUI.tsx::CelButton3D",kind:"component"})
MERGE (toggle:Symbol {id:"frontend/src/components/effects/Cel3DUI.tsx::CelToggle3D",kind:"component"})
MERGE (beacon:Symbol {id:"frontend/src/components/effects/Cel3DUI.tsx::CelBeacon3D",kind:"component"})
MERGE (dial:Symbol {id:"frontend/src/components/effects/Cel3DUI.tsx::CelDial3D",kind:"component"})
MERGE (card:Symbol {id:"frontend/src/components/effects/Cel3DUI.tsx::CelCard3D",kind:"component"})
MERGE (self)-[:DEFINES]->(btn)
MERGE (self)-[:DEFINES]->(toggle)
MERGE (self)-[:DEFINES]->(beacon)
MERGE (self)-[:DEFINES]->(dial)
MERGE (self)-[:DEFINES]->(card)
@endcypher
*/
"use client";

import React, { useMemo, useRef, useState, useEffect, useCallback } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { resolveCelTokenPalette, readThemeRgb, type CelTokenPalette } from "@/lib/theme/themeTokens";

// ============================================================================
// 0. SHARED THEME / VISIBILITY / REDUCED-MOTION HOOKS
// ============================================================================
function useThemeRevision(): number {
  const [rev, setRev] = useState(0);
  useEffect(() => {
    if (typeof MutationObserver === "undefined") return;
    const obs = new MutationObserver(() => setRev((r) => r + 1));
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    return () => obs.disconnect();
  }, []);
  return rev;
}

function usePageVisibility(): boolean {
  const [visible, setVisible] = useState(true);
  useEffect(() => {
    const onVis = () => setVisible(!document.hidden);
    setVisible(!document.hidden);
    document.addEventListener("visibilitychange", onVis);
    return () => document.removeEventListener("visibilitychange", onVis);
  }, []);
  return visible;
}

function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(false);
  useEffect(() => {
    const mq = window.matchMedia?.("(prefers-reduced-motion: reduce)");
    if (!mq) return;
    setReduced(mq.matches);
    const onChange = (e: MediaQueryListEvent) => setReduced(e.matches);
    mq.addEventListener?.("change", onChange);
    return () => mq.removeEventListener?.("change", onChange);
  }, []);
  return reduced;
}

// ============================================================================
// 1. SHARED SHADERS FOR 3D CEL UI PRIMITIVES
// ============================================================================

const CEL_UI_VERTEX = /* glsl */ `
  uniform float uOutlineOffset;
  varying vec3 vNormal;
  varying vec3 vWorldPos;
  varying vec3 vViewPos;

  void main() {
    vec3 norm = normalize(normalMatrix * normal);
    vNormal = norm;
    vec3 displaced = position + norm * uOutlineOffset;
    vec4 mvPos = modelViewMatrix * vec4(displaced, 1.0);
    vViewPos = -mvPos.xyz;
    vWorldPos = (modelMatrix * vec4(displaced, 1.0)).xyz;
    gl_Position = projectionMatrix * mvPos;
  }
`;

const CEL_UI_FRAGMENT = /* glsl */ `
  uniform vec3 uBaseColor;
  uniform vec3 uShadowColor;
  uniform vec3 uHighlightColor;
  uniform vec3 uRimColor;
  uniform vec3 uLightPos;
  uniform float uIsOutline;
  uniform float uBands;

  varying vec3 vNormal;
  varying vec3 vWorldPos;
  varying vec3 vViewPos;

  void main() {
    if (uIsOutline > 0.5) {
      gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
      return;
    }

    vec3 N = normalize(vNormal);
    vec3 L = normalize(uLightPos - vWorldPos);
    vec3 V = normalize(vViewPos);
    vec3 H = normalize(L + V);

    // Half-Lambert Anime Lighting
    float NdotL = dot(N, L) * 0.5 + 0.5;

    // Stepped Cel Ramp (Sharp 2-band or 3-band)
    float celFactor = 0.2;
    if (uBands < 2.5) {
      celFactor = step(0.48, NdotL) * 0.8 + 0.2;
    } else {
      float s1 = smoothstep(0.33, 0.36, NdotL);
      float s2 = smoothstep(0.66, 0.69, NdotL);
      celFactor = 0.2 + 0.4 * s1 + 0.4 * s2;
    }

    vec3 color = mix(uShadowColor, uBaseColor, celFactor);

    // Hard Specular Highlight Glint
    float NdotH = max(0.0, dot(N, H));
    float spec = smoothstep(0.92, 0.95, pow(NdotH, 24.0));
    color += spec * uHighlightColor * 0.8;

    // Anime Fresnel Edge Rim
    float fresnel = 1.0 - max(0.0, dot(N, V));
    float rim = smoothstep(0.68, 0.74, fresnel) * smoothstep(0.05, 0.3, NdotL);
    color += rim * uRimColor * 1.1;

    gl_FragColor = vec4(color, 1.0);
  }
`;

// ============================================================================
// 2. CEL BUTTON 3D — 机械物理按键
// ============================================================================
export interface CelButton3DProps {
  label: string;
  sublabel?: string;
  variant?: "crimson" | "cyan" | "gold" | "stealth";
  onClick?: () => void;
  className?: string;
  active?: boolean;
  disabled?: boolean;
}

function CelButtonMesh({
  variant,
  pressed,
  hovered,
  focused,
  themeRev,
}: {
  variant: "crimson" | "cyan" | "gold" | "stealth";
  pressed: boolean;
  hovered: boolean;
  focused: boolean;
  themeRev: number;
}) {
  const meshRef = useRef<THREE.Group>(null);
  const keycapRef = useRef<THREE.Mesh>(null);
  const baseRef = useRef<THREE.Mesh>(null);
  const springRef = useRef<THREE.Mesh>(null);
  const rippleRef = useRef<THREE.Mesh>(null);

  // per 2026-11-17 主题统一: 从当前主题 --cel-* token 取色 (dark fallback = 原 hex)
  const colors = useMemo(() => {
    const p = (v: CelTokenPalette) => ({
      base: new THREE.Color(...v.base),
      shadow: new THREE.Color(...v.shadow),
      highlight: new THREE.Color(...v.highlight),
      rim: new THREE.Color(...v.rim),
    });
    switch (variant) {
      case "cyan":
        return p(resolveCelTokenPalette({
          baseToken: "--cel-cyan", rimToken: "--cel-crimson",
          fallback: { base: [0, 0.941, 1], shadow: [0.008, 0.133, 0.188], highlight: [1, 1, 1], rim: [1, 0.094, 0.298] },
        }));
      case "gold":
        return p(resolveCelTokenPalette({
          baseToken: "--cel-gold", rimToken: "--cel-cyan",
          fallback: { base: [1, 0.769, 0], shadow: [0.2, 0.141, 0.008], highlight: [1, 1, 1], rim: [0, 0.941, 1] },
        }));
      case "stealth":
        return p(resolveCelTokenPalette({
          baseToken: "--cel-surface-sub", rimToken: "--cel-cyan", highlightToken: "--cel-text-secondary",
          fallback: { base: [0.165, 0.208, 0.278], shadow: [0.047, 0.067, 0.11], highlight: [0.561, 0.647, 0.8], rim: [0, 0.941, 1] },
        }));
      case "crimson":
      default:
        return p(resolveCelTokenPalette({
          baseToken: "--cel-crimson", rimToken: "--cel-cyan",
          fallback: { base: [1, 0.094, 0.298], shadow: [0.22, 0.016, 0.063], highlight: [1, 1, 1], rim: [0, 0.941, 1] },
        }));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [variant, themeRev]);

  // 几何：键帽(圆角盒) + 底座(圆柱) + 弹簧(螺旋线) + 波纹环
  const keycapGeom = useMemo(() => new THREE.BoxGeometry(2.4, 0.5, 0.35, 8, 4, 4), []);
  const baseGeom = useMemo(() => new THREE.CylinderGeometry(1.3, 1.3, 0.25, 16), []);
  const springGeom = useMemo(() => {
    const g = new THREE.BufferGeometry();
    const pts = [];
    const turns = 4;
    const h = 0.35;
    for (let i = 0; i <= 64; i++) {
      const t = i / 64;
      const angle = t * turns * Math.PI * 2;
      const r = 0.25 * (1 - t * 0.15);
      pts.push(new THREE.Vector3(
        Math.cos(angle) * r,
        -0.25 + t * h,
        Math.sin(angle) * r
      ));
    }
    g.setFromPoints(pts);
    return g;
  }, []);
  const rippleGeom = useMemo(() => new THREE.RingGeometry(1.2, 1.5, 32), []);

  const keycapUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: focused ? new THREE.Color(...readThemeRgb("--cel-cyan", [0, 0.941, 1])) : colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors, focused]
  );

  const baseUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.shadow },
      uShadowColor: { value: new THREE.Color(colors.shadow.r * 0.5, colors.shadow.g * 0.5, colors.shadow.b * 0.5) },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const springUniforms = useMemo(
    () => ({
      uBaseColor: { value: new THREE.Color(0.6, 0.65, 0.75) },
      uShadowColor: { value: new THREE.Color(0.2, 0.25, 0.35) },
      uHighlightColor: { value: new THREE.Color(0.9, 0.95, 1.0) },
      uRimColor: { value: new THREE.Color(0.4, 0.5, 0.7) },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    []
  );

  const rippleUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const rippleTimeRef = useRef(0);

  useFrame((_, delta) => {
    if (meshRef.current) {
      // Physical depression in 3D space with spring return
      const targetZ = pressed ? -0.18 : hovered ? 0.08 : 0.0;
      meshRef.current.position.z += (targetZ - meshRef.current.position.z) * 0.35;
      const targetRotX = pressed ? -0.08 : hovered ? 0.05 : 0.0;
      meshRef.current.rotation.x += (targetRotX - meshRef.current.rotation.x) * 0.35;
    }

    // Spring compression visualization
    if (springRef.current) {
      const targetScaleY = pressed ? 0.45 : 1.0;
      springRef.current.scale.y += (targetScaleY - springRef.current.scale.y) * 0.3;
    }

    // Ripple on press
    if (rippleRef.current && pressed && rippleTimeRef.current === 0) {
      rippleTimeRef.current = 0.01;
    }
    if (rippleTimeRef.current > 0 && rippleRef.current) {
      rippleTimeRef.current += delta * 4;
      if (rippleTimeRef.current > 1) {
        rippleTimeRef.current = 0;
      } else {
        const mesh = rippleRef.current;
        mesh.scale.setScalar(1 + rippleTimeRef.current * 2.5);
        if (mesh.material) {
          (mesh.material as THREE.ShaderMaterial).uniforms.uRimColor.value.setScalar(1 - rippleTimeRef.current);
        }
      }
    }
  });

  return (
    <group ref={meshRef}>
      {/* Base housing */}
      <mesh ref={baseRef} geometry={baseGeom} position={[0, -0.3, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={baseUniforms}
        />
      </mesh>

      {/* Spring coil (visible through cutout) - using thin cylinder */}
      <mesh ref={springRef} geometry={springGeom} position={[0, -0.12, 0]} scale={[1, 1, 1]}>
        <meshBasicMaterial color={new THREE.Color(0.6, 0.65, 0.75)} transparent opacity={0.7} />
      </mesh>

      {/* Keycap - main interactive surface */}
      <mesh ref={keycapRef} geometry={keycapGeom} position={[0, 0.0, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={keycapUniforms}
        />
      </mesh>

      {/* Outline for keycap */}
      <mesh geometry={keycapGeom} position={[0, 0.0, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={{
            ...keycapUniforms,
            uIsOutline: { value: 1.0 },
            uOutlineOffset: { value: focused ? 0.06 : 0.04 },
          }}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Press ripple ring */}
      {rippleTimeRef.current > 0 && rippleRef.current && (
        <mesh ref={rippleRef} geometry={rippleGeom} position={[0, -0.15, 0]} rotation={[-Math.PI / 2, 0, 0]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={rippleUniforms}
            transparent
            side={THREE.DoubleSide}
          />
        </mesh>
      )}
    </group>
  );
}

export function CelButton3D({
  label,
  sublabel,
  variant = "crimson",
  onClick,
  className = "",
  disabled = false,
}: CelButton3DProps) {
  const [pressed, setPressed] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const [mounted, setMounted] = useState(false);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();

  useEffect(() => setMounted(true), []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (disabled) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      setPressed(true);
      onClick?.();
      setTimeout(() => setPressed(false), 120);
    }
  }, [disabled, onClick]);

  if (!mounted) {
    return (
      <button
        disabled={disabled}
        className={`px-4 py-2 bg-[var(--cel-crimson)] text-[var(--cel-ink)] font-black text-xs ${className}`}
      >
        {label}
      </button>
    );
  }

  return (
    <div
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-label={`${label} ${sublabel || ""}`}
      className={`relative inline-block cursor-pointer select-none focus:outline-none ${
        disabled ? "opacity-40 cursor-not-allowed" : ""
      } ${className}`}
      onMouseDown={() => !disabled && setPressed(true)}
      onMouseUp={() => {
        if (!disabled) {
          setPressed(false);
          onClick?.();
        }
      }}
      onMouseEnter={() => !disabled && setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onKeyDown={handleKeyDown}
      style={{ width: "100%" }}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 4.5], fov: 30 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          className="w-full h-[48px]"
        >
          <CelButtonMesh
            variant={variant}
            pressed={pressed}
            hovered={hovered && !disabled}
            focused={focused && !disabled}
            themeRev={themeRev}
          />
        </Canvas>
      )}
      <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
        <span className="font-black text-xs text-[var(--cel-ink)] leading-tight">{label}</span>
        {sublabel && (
          <span className="font-mono text-[9px] text-[var(--cel-text-secondary)] leading-none mt-0.5">
            {sublabel}
          </span>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// 3. CEL TOGGLE 3D — 实体杠杆开关
// ============================================================================
export interface CelToggle3DProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  sublabel?: string;
  className?: string;
}

function CelToggleMesh({
  checked,
  focused,
  themeRev,
}: { checked: boolean; focused: boolean; themeRev: number }) {
  const baseRef = useRef<THREE.Mesh>(null);
  const leverRef = useRef<THREE.Group>(null);
  const knobRef = useRef<THREE.Mesh>(null);
  const gearRef = useRef<THREE.Group>(null);
  const indicatorRef = useRef<THREE.Mesh>(null);

  // per 2026-11-17 主题统一: ON = 主题 crimson, OFF = 主题次级面
  const colorsOn = useMemo(
    () => {
      const v = resolveCelTokenPalette({
        baseToken: "--cel-crimson", rimToken: "--cel-cyan",
        fallback: { base: [1, 0.094, 0.298], shadow: [0.22, 0.016, 0.063], highlight: [1, 1, 1], rim: [0, 0.941, 1] },
      });
      return {
        base: new THREE.Color(...v.base),
        shadow: new THREE.Color(...v.shadow),
        highlight: new THREE.Color(...v.highlight),
        rim: new THREE.Color(...v.rim),
      };
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [themeRev]
  );

  const colorsOff = useMemo(
    () => {
      const v = resolveCelTokenPalette({
        baseToken: "--cel-surface-sub", rimToken: "--cel-cyan", highlightToken: "--cel-text-secondary",
        fallback: { base: [0.102, 0.133, 0.2], shadow: [0.031, 0.043, 0.071], highlight: [0.451, 0.518, 0.651], rim: [0, 0.941, 1] },
      });
      return {
        base: new THREE.Color(...v.base),
        shadow: new THREE.Color(...v.shadow),
        highlight: new THREE.Color(...v.highlight),
        rim: new THREE.Color(...v.rim),
      };
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [themeRev]
  );

  const baseGeom = useMemo(() => new THREE.BoxGeometry(2.2, 0.9, 0.28), []);
  const leverGeom = useMemo(() => new THREE.CylinderGeometry(0.18, 0.22, 1.0, 12), []);
  const knobGeom = useMemo(() => new THREE.SphereGeometry(0.22, 12, 12), []);
  const gearGeom = useMemo(() => {
    const g = new THREE.BufferGeometry();
    const pts = [];
    const teeth = 12;
    for (let i = 0; i <= teeth; i++) {
      const angle = (i / teeth) * Math.PI * 2;
      const r1 = 0.18;
      const r2 = 0.24;
      pts.push(new THREE.Vector3(Math.cos(angle) * r1, 0, Math.sin(angle) * r1));
      pts.push(new THREE.Vector3(Math.cos(angle) * r2, 0, Math.sin(angle) * r2));
    }
    g.setFromPoints(pts);
    return g;
  }, []);
  const indicatorGeom = useMemo(() => new THREE.BoxGeometry(0.5, 0.25, 0.12), []);

  const baseUniforms = useMemo(
    () => ({
      uBaseColor: { value: colorsOff.base },
      uShadowColor: { value: colorsOff.shadow },
      uHighlightColor: { value: colorsOff.highlight },
      uRimColor: { value: colorsOff.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colorsOff]
  );

  const leverUniforms = useMemo(
    () => ({
      uBaseColor: { value: checked ? colorsOn.base : colorsOff.base },
      uShadowColor: { value: checked ? colorsOn.shadow : colorsOff.shadow },
      uHighlightColor: { value: checked ? colorsOn.highlight : colorsOff.highlight },
      uRimColor: { value: focused ? new THREE.Color(...readThemeRgb("--cel-cyan", [0, 0.941, 1])) : colorsOn.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [checked, colorsOn, colorsOff, focused]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colorsOn.base },
      uShadowColor: { value: colorsOn.shadow },
      uHighlightColor: { value: colorsOn.highlight },
      uRimColor: { value: colorsOn.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: focused ? 0.06 : 0.04 },
      uBands: { value: 2.0 },
    }),
    [colorsOn, focused]
  );

  const indicatorUniforms = useMemo(
    () => ({
      uBaseColor: { value: checked ? colorsOn.base : colorsOff.base },
      uShadowColor: { value: checked ? colorsOn.shadow : colorsOff.shadow },
      uHighlightColor: { value: checked ? colorsOn.highlight : colorsOff.highlight },
      uRimColor: { value: checked ? colorsOn.rim : colorsOff.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [checked, colorsOn, colorsOff]
  );

  const gearUniforms = useMemo(
    () => ({
      uBaseColor: { value: new THREE.Color(0.4, 0.45, 0.55) },
      uShadowColor: { value: new THREE.Color(0.15, 0.18, 0.25) },
      uHighlightColor: { value: new THREE.Color(0.8, 0.85, 0.95) },
      uRimColor: { value: new THREE.Color(0.5, 0.55, 0.7) },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    []
  );

  useFrame(() => {
    if (leverRef.current) {
      const targetAngle = checked ? 0.55 : -0.55;
      leverRef.current.rotation.z += (targetAngle - leverRef.current.rotation.z) * 0.28;
      const targetX = checked ? 0.55 : -0.55;
      leverRef.current.position.x += (targetX - leverRef.current.position.x) * 0.28;
    }

    // Gear rotation sync
    if (gearRef.current) {
      gearRef.current.rotation.z = leverRef.current?.rotation.z ?? 0;
    }

    // Indicator light pulse
    if (indicatorRef.current && checked) {
      indicatorRef.current.material = indicatorRef.current.material;
      const mat = indicatorRef.current.material as THREE.ShaderMaterial;
      if (mat.uniforms?.uRimColor) {
        mat.uniforms.uRimColor.value.setScalar(0.6 + Math.sin(Date.now() * 0.006) * 0.4);
      }
    }
  });

  return (
    <group>
      {/* Base track */}
      <mesh ref={baseRef} geometry={baseGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={baseUniforms}
        />
      </mesh>
      <mesh geometry={baseGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={outlineUniforms}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Gear mechanism (visible through slot) */}
      <group ref={gearRef} position={[-0.65, 0, 0.16]}>
        <mesh geometry={gearGeom} rotation={[-Math.PI / 2, 0, 0]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={gearUniforms}
            side={THREE.DoubleSide}
          />
        </mesh>
      </group>

      {/* Lever arm with knob */}
      <group ref={leverRef} position={[checked ? 0.55 : -0.55, 0, 0.22]}>
        <mesh geometry={leverGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={leverUniforms}
          />
        </mesh>
        <mesh geometry={leverGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>

        {/* Knob at end of lever */}
        <mesh ref={knobRef} geometry={knobGeom} position={[checked ? 0.55 : -0.55, 0, 0.45]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={leverUniforms}
          />
        </mesh>
        <mesh geometry={knobGeom} position={[checked ? 0.55 : -0.55, 0, 0.45]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      {/* Status indicator LED */}
      <mesh ref={indicatorRef} geometry={indicatorGeom} position={[0, 0.5, 0.25]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={indicatorUniforms}
        />
      </mesh>
      <mesh geometry={indicatorGeom} position={[0, 0.5, 0.25]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={outlineUniforms}
          side={THREE.BackSide}
        />
      </mesh>
    </group>
  );
}

export function CelToggle3D({
  checked,
  onChange,
  label,
  sublabel,
  className = "",
}: CelToggle3DProps) {
  const [focused, setFocused] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [animating, setAnimating] = useState(false);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();

  useEffect(() => setMounted(true), []);

  useEffect(() => {
    setAnimating(true);
    const timer = setTimeout(() => setAnimating(false), 450);
    return () => clearTimeout(timer);
  }, [checked]);

  const handleClick = useCallback(() => {
    onChange(!checked);
  }, [checked, onChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onChange(!checked);
    }
  }, [checked, onChange]);

  if (!mounted) {
    return (
      <button
        role="switch"
        aria-checked={checked}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        className={`relative inline-flex items-center h-6 w-11 rounded-full transition-colors ${className}`}
      >
        <span className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform ${checked ? "translate-x-6" : "translate-x-1"}`} />
      </button>
    );
  }

  return (
    <div
      role="switch"
      aria-checked={checked}
      aria-label={label}
      tabIndex={0}
      className={`relative inline-block cursor-pointer select-none focus:outline-none ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{ width: "140px", height: "64px" }}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 4.2], fov: 35 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          frameloop={animating ? "always" : "demand"}
          className="w-full h-full"
        >
          <CelToggleMesh checked={checked} focused={focused} themeRev={themeRev} />
        </Canvas>
      )}
      <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none px-2">
        {label && <span className="font-black text-xs text-[var(--cel-text-primary)] leading-tight">{label}</span>}
        {sublabel && <span className="font-mono text-[8px] text-[var(--cel-text-secondary)] leading-none mt-0.5">{sublabel}</span>}
      </div>
    </div>
  );
}

// ============================================================================
// 4. CEL BEACON 3D — 脉冲塔
// ============================================================================
export interface CelBeacon3DProps {
  status: "idle" | "active" | "warning" | "error";
  size?: number;
  title?: string;
  className?: string;
  "data-testid"?: string;
}

function CelBeaconMesh({
  status,
  focused,
  themeRev,
}: { status: "idle" | "active" | "warning" | "error"; focused: boolean; themeRev: number }) {
  const towerRef = useRef<THREE.Group>(null);
  const antennaRef = useRef<THREE.Group>(null);
  const dishRef = useRef<THREE.Mesh>(null);
  const pulseRef = useRef<THREE.Mesh>(null);
  const scanConeRef = useRef<THREE.Mesh>(null);
  const beaconLightRef = useRef<THREE.Mesh>(null);

  const statusColors = useMemo(() => {
    const tokenMap: Record<string, string> = {
      active: "--cel-cyan",
      warning: "--cel-gold",
      error: "--cel-crimson",
      idle: "--cel-surface-sub",
    };
    const token = tokenMap[status] ?? "--cel-surface-sub";
    const v = resolveCelTokenPalette({
      baseToken: token, rimToken: "--cel-cyan",
      fallback: { base: [0.5, 0.5, 0.5], shadow: [0.1, 0.1, 0.1], highlight: [1, 1, 1], rim: [0, 0.941, 1] },
    });
    return {
      base: new THREE.Color(...v.base),
      shadow: new THREE.Color(...v.shadow),
      highlight: new THREE.Color(...v.highlight),
      rim: new THREE.Color(...v.rim),
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, themeRev]);

  const towerGeom = useMemo(() => new THREE.CylinderGeometry(0.15, 0.22, 1.8, 8), []);
  const antennaGeom = useMemo(() => new THREE.CylinderGeometry(0.02, 0.02, 1.2, 6), []);
  const dishGeom = useMemo(() => new THREE.ConeGeometry(0.35, 0.4, 16), []);
  const pulseGeom = useMemo(() => new THREE.RingGeometry(0.4, 0.45, 32), []);
  const scanConeGeom = useMemo(() => new THREE.ConeGeometry(0.8, 2.5, 16, 1, true), []);
  const beaconLightGeom = useMemo(() => new THREE.SphereGeometry(0.12, 12, 12), []);

  const towerUniforms = useMemo(
    () => ({
      uBaseColor: { value: statusColors.base },
      uShadowColor: { value: statusColors.shadow },
      uHighlightColor: { value: statusColors.highlight },
      uRimColor: { value: focused ? new THREE.Color(...readThemeRgb("--cel-cyan", [0, 0.941, 1])) : statusColors.rim },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [statusColors, focused]
  );

  const antennaUniforms = useMemo(
    () => ({
      uBaseColor: { value: new THREE.Color(0.3, 0.35, 0.45) },
      uShadowColor: { value: new THREE.Color(0.1, 0.12, 0.18) },
      uHighlightColor: { value: new THREE.Color(0.7, 0.75, 0.85) },
      uRimColor: { value: new THREE.Color(0.4, 0.5, 0.7) },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    []
  );

  const pulseUniforms = useMemo(
    () => ({
      uBaseColor: { value: statusColors.rim },
      uShadowColor: { value: statusColors.shadow },
      uHighlightColor: { value: statusColors.highlight },
      uRimColor: { value: statusColors.rim },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [statusColors]
  );

  const scanUniforms = useMemo(
    () => ({
      uBaseColor: { value: statusColors.base },
      uShadowColor: { value: new THREE.Color(0, 0, 0) },
      uHighlightColor: { value: statusColors.highlight },
      uRimColor: { value: statusColors.rim },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [statusColors]
  );

  const beaconLightUniforms = useMemo(
    () => ({
      uBaseColor: { value: statusColors.base },
      uShadowColor: { value: statusColors.shadow },
      uHighlightColor: { value: statusColors.highlight },
      uRimColor: { value: statusColors.rim },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [statusColors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: statusColors.base },
      uShadowColor: { value: statusColors.shadow },
      uHighlightColor: { value: statusColors.highlight },
      uRimColor: { value: statusColors.rim },
      uLightPos: { value: new THREE.Vector3(3, 5, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: focused ? 0.05 : 0.03 },
      uBands: { value: 2.0 },
    }),
    [statusColors, focused]
  );

  const pulsePhaseRef = useRef(0);
  const scanAngleRef = useRef(0);

  useFrame((_, delta) => {
    // Antenna rotation
    if (antennaRef.current) {
      antennaRef.current.rotation.y += delta * (status === "active" ? 3.0 : 0.5);
    }

    // Dish counter-rotation
    if (dishRef.current) {
      dishRef.current.rotation.y -= delta * (status === "active" ? 2.5 : 0.3);
    }

    // Pulse rings
    if (pulseRef.current && status === "active") {
      pulsePhaseRef.current += delta * 1.5;
      const p = (Math.sin(pulsePhaseRef.current) + 1) / 2;
      pulseRef.current.scale.setScalar(0.8 + p * 3.0);
      if (pulseRef.current.material) {
        (pulseRef.current.material as THREE.ShaderMaterial).uniforms.uRimColor.value.setScalar(1 - p);
      }
    } else if (pulseRef.current) {
      pulseRef.current.scale.setScalar(0.1);
    }

    // Scan cone
    if (scanConeRef.current && status === "active") {
      scanAngleRef.current += delta * 2.0;
      scanConeRef.current.rotation.y = scanAngleRef.current;
      scanConeRef.current.material = scanConeRef.current.material;
      const mat = scanConeRef.current.material as THREE.ShaderMaterial;
      if (mat.uniforms?.uRimColor) {
        mat.uniforms.uRimColor.value.setScalar(0.3 + Math.abs(Math.sin(scanAngleRef.current)) * 0.5);
      }
    } else if (scanConeRef.current) {
      scanConeRef.current.scale.setScalar(0.01);
    }

    // Beacon light pulse
    if (beaconLightRef.current) {
      const pulse = 0.5 + Math.sin(Date.now() * 0.005 + (status === "error" ? Math.PI : 0)) * 0.5;
      beaconLightRef.current.scale.setScalar(0.8 + pulse * 0.4);
      if (beaconLightRef.current.material) {
        (beaconLightRef.current.material as THREE.ShaderMaterial).uniforms.uRimColor.value.setScalar(pulse);
      }
    }
  });

  return (
    <group ref={towerRef}>
      {/* Tower body */}
      <mesh ref={towerRef} geometry={towerGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={towerUniforms}
        />
      </mesh>
      <mesh geometry={towerGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={outlineUniforms}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Antenna array */}
      <group ref={antennaRef} position={[0, 1.1, 0]}>
        <mesh geometry={antennaGeom} position={[0, 0.6, 0]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={antennaUniforms}
          />
        </mesh>

        {/* Dish at top */}
        <group ref={dishRef} position={[0, 1.2, 0]} rotation={[0, 0, Math.PI]}>
          <mesh geometry={dishGeom}>
            <shaderMaterial
              vertexShader={CEL_UI_VERTEX}
              fragmentShader={CEL_UI_FRAGMENT}
              uniforms={antennaUniforms}
            />
          </mesh>
        </group>
      </group>

      {/* Pulse rings at base */}
      <mesh ref={pulseRef} geometry={pulseGeom} position={[0, -0.9, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={pulseUniforms}
          transparent
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Scan cone */}
      <mesh ref={scanConeRef} geometry={scanConeGeom} position={[0, 0, 0]} rotation={[Math.PI, 0, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={scanUniforms}
          transparent
          opacity={0.15}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Beacon light at top */}
      <mesh ref={beaconLightRef} geometry={beaconLightGeom} position={[0, 1.3, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={beaconLightUniforms}
        />
      </mesh>
    </group>
  );
}

export function CelBeacon3D({
  status = "idle",
  size = 48,
  title,
  className = "",
  "data-testid": testId,
}: CelBeacon3DProps) {
  const [focused, setFocused] = useState(false);
  const [mounted, setMounted] = useState(false);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => setMounted(true), []);

  if (!mounted) {
    const colorMap = {
      active: "var(--cel-cyan)",
      warning: "var(--cel-gold)",
      error: "var(--cel-crimson)",
      idle: "var(--cel-surface-sub)",
    };
    return (
      <div data-testid={testId} className={`relative inline-block ${className}`} style={{ width: size, height: size }}>
        <div className="absolute inset-0 flex items-center justify-center">
          <div
            className="w-2 h-2 rounded-full animate-pulse"
            style={{ backgroundColor: colorMap[status] }}
          />
        </div>
        {title && <div className="text-[10px] font-mono text-center mt-2">{title}</div>}
      </div>
    );
  }

  return (
    <div
      role="status"
      aria-label={title ?? `3D Beacon: ${status}`}
      title={title}
      data-testid={testId}
      tabIndex={0}
      className={`relative inline-block cursor-pointer select-none focus:outline-none ${className}`}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      style={{ width: size, height: size }}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 5.5], fov: 30 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          frameloop={reducedMotion ? "demand" : "always"}
          className="w-full h-full"
        >
          <CelBeaconMesh status={status} focused={focused} themeRev={themeRev} />
        </Canvas>
      )}
    </div>
  );
}

// ============================================================================
// 5. CEL DIAL 3D — 物理旋钮编码器
// ============================================================================
export interface CelDial3DProps {
  value: number;
  onChange: (value: number) => void;
  options: string[];
  className?: string;
}

function CelDialMesh({
  value,
  options,
  focused,
  themeRev,
}: { value: number; options: string[]; focused: boolean; themeRev: number }) {
  const dialRef = useRef<THREE.Group>(null);
  const knobRef = useRef<THREE.Group>(null);
  const markerRef = useRef<THREE.Mesh>(null);
  const detentRef = useRef<THREE.Group>(null);
  const ringRef = useRef<THREE.Mesh>(null);

  const colors = useMemo(
    () => {
      const v = resolveCelTokenPalette({
        baseToken: "--cel-cyan", rimToken: "--cel-crimson",
        fallback: { base: [0, 0.941, 1], shadow: [0.008, 0.133, 0.188], highlight: [1, 1, 1], rim: [1, 0.094, 0.298] },
      });
      return {
        base: new THREE.Color(...v.base),
        shadow: new THREE.Color(...v.shadow),
        highlight: new THREE.Color(...v.highlight),
        rim: new THREE.Color(...v.rim),
      };
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [themeRev]
  );

  const bgColors = useMemo(
    () => {
      const v = resolveCelTokenPalette({
        baseToken: "--cel-surface-sub", rimToken: "--cel-cyan", highlightToken: "--cel-text-secondary",
        fallback: { base: [0.102, 0.133, 0.2], shadow: [0.031, 0.043, 0.071], highlight: [0.451, 0.518, 0.651], rim: [0, 0.941, 1] },
      });
      return {
        base: new THREE.Color(...v.base),
        shadow: new THREE.Color(...v.shadow),
        highlight: new THREE.Color(...v.highlight),
        rim: new THREE.Color(...v.rim),
      };
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [themeRev]
  );

  const dialGeom = useMemo(() => new THREE.CylinderGeometry(1.1, 1.1, 0.15, 32), []);
  const knobGeom = useMemo(() => new THREE.CylinderGeometry(0.95, 0.95, 0.3, 24), []);
  const markerGeom = useMemo(() => new THREE.BoxGeometry(0.08, 0.28, 0.12), []);
  const ringGeom = useMemo(() => new THREE.RingGeometry(1.0, 1.1, 64), []);
  const detentGeom = useMemo(() => new THREE.BoxGeometry(0.04, 0.06, 0.18), []);

  const dialUniforms = useMemo(
    () => ({
      uBaseColor: { value: bgColors.base },
      uShadowColor: { value: bgColors.shadow },
      uHighlightColor: { value: bgColors.highlight },
      uRimColor: { value: bgColors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [bgColors]
  );

  const knobUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: focused ? new THREE.Color(...readThemeRgb("--cel-cyan", [0, 0.941, 1])) : colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors, focused]
  );

  const markerUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const ringUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const detentUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: focused ? 0.05 : 0.03 },
      uBands: { value: 3.0 },
    }),
    [colors, focused]
  );

  const targetRotation = useMemo(
    () => (value / Math.max(1, options.length - 1)) * Math.PI * 2 - Math.PI,
    [value, options.length]
  );

  useFrame(() => {
    if (knobRef.current) {
      knobRef.current.rotation.z += (targetRotation - knobRef.current.rotation.z) * 0.25;
    }
    if (detentRef.current) {
      detentRef.current.rotation.z = knobRef.current?.rotation.z ?? 0;
    }
  });

  // Build detent notches
  const detents = useMemo(() => {
    const arr = [];
    const count = options.length;
    for (let i = 0; i < count; i++) {
      const angle = (i / Math.max(1, count - 1)) * Math.PI * 2 - Math.PI;
      arr.push(angle);
    }
    return arr;
  }, [options.length]);

  return (
    <group ref={dialRef}>
      {/* Base dial */}
      <mesh geometry={dialGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={dialUniforms}
        />
      </mesh>

      {/* Outer ring with tick marks */}
      <mesh ref={ringRef} geometry={ringGeom} position={[0, 0, 0.08]} rotation={[-Math.PI / 2, 0, 0]}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={ringUniforms}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Detent notches (internal, rotate with knob) */}
      <group ref={detentRef}>
        {detents.map((angle, i) => (
          <mesh key={i} geometry={detentGeom} position={[Math.cos(angle) * 0.9, Math.sin(angle) * 0.9, 0.2]} rotation={[0, 0, angle + Math.PI / 2]}>
            <shaderMaterial
              vertexShader={CEL_UI_VERTEX}
              fragmentShader={CEL_UI_FRAGMENT}
              uniforms={detentUniforms}
            />
          </mesh>
        ))}
      </group>

      {/* Knob with marker */}
      <group ref={knobRef}>
        <mesh geometry={knobGeom} position={[0, 0, 0.15]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={knobUniforms}
          />
        </mesh>
        <mesh geometry={knobGeom} position={[0, 0, 0.15]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>

        {/* Marker on knob top */}
        <mesh ref={markerRef} geometry={markerGeom} position={[0, 0.9, 0.3]} rotation={[0, 0, Math.PI / 2]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={markerUniforms}
          />
        </mesh>
        <mesh geometry={markerGeom} position={[0, 0.9, 0.3]} rotation={[0, 0, Math.PI / 2]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>

        {/* Grip texture (radial grooves) */}
        {[0, 1, 2, 3].map((i) => (
          <mesh key={i} geometry={markerGeom} position={[0, 0, 0.15]} rotation={[0, 0, (i * Math.PI) / 2]} scale={[0.5, 2.5, 1]}>
            <shaderMaterial
              vertexShader={CEL_UI_VERTEX}
              fragmentShader={CEL_UI_FRAGMENT}
              uniforms={{
                ...knobUniforms,
                uBaseColor: { value: new THREE.Color(colors.base.r * 0.7, colors.base.g * 0.7, colors.base.b * 0.7) },
              }}
            />
          </mesh>
        ))}
      </group>
    </group>
  );
}

export function CelDial3D({
  value,
  onChange,
  options,
  className = "",
}: CelDial3DProps) {
  const [focused, setFocused] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [dragging, setDragging] = useState(false);
  const startRef = useRef({ x: 0, y: 0, value: 0 });
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();

  useEffect(() => setMounted(true), []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    setDragging(true);
    startRef.current = { x: e.clientX, y: e.clientY, value };
    e.preventDefault();
  }, [value]);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!dragging) return;
    const deltaY = startRef.current.y - e.clientY;
    const deltaX = e.clientX - startRef.current.x;
    const delta = deltaY * 0.01 + deltaX * 0.005;
    const newValue = Math.round(Math.max(0, Math.min(options.length - 1, startRef.current.value + delta)));
    onChange(newValue);
  }, [dragging, onChange, options.length]);

  const handleMouseUp = useCallback(() => {
    setDragging(false);
  }, []);

  useEffect(() => {
    if (dragging) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
      return () => {
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [dragging, handleMouseMove, handleMouseUp]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "ArrowRight" || e.key === "ArrowUp") {
      e.preventDefault();
      onChange(Math.min(options.length - 1, value + 1));
    } else if (e.key === "ArrowLeft" || e.key === "ArrowDown") {
      e.preventDefault();
      onChange(Math.max(0, value - 1));
    }
  }, [value, onChange, options.length]);

  if (!mounted) {
    return (
      <select value={value} onChange={(e) => onChange(Number(e.target.value))} className={className}>
        {options.map((o, i) => <option key={o} value={i}>{o}</option>)}
      </select>
    );
  }

  return (
    <div
      role="slider"
      aria-valuemin={0}
      aria-valuemax={options.length - 1}
      aria-valuenow={value}
      tabIndex={0}
      className={`relative inline-block cursor-pointer select-none focus:outline-none ${className}`}
      onMouseDown={handleMouseDown}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onKeyDown={handleKeyDown}
      style={{ width: "80px", height: "80px" }}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 4.5], fov: 30 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          className="w-full h-full"
        >
          <CelDialMesh value={value} options={options} focused={focused} themeRev={themeRev} />
        </Canvas>
      )}
      <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
        <span className="font-mono text-[10px] text-[var(--cel-text-secondary)]">{options[value]}</span>
      </div>
    </div>
  );
}

// ============================================================================
// 6. CEL CARD 3D — 悬浮全息面板
// ============================================================================
export interface CelCard3DProps {
  children: React.ReactNode;
  className?: string;
  variant?: "default" | "elevated" | "outlined";
}

function CelCardMesh({ variant, themeRev }: { variant: "default" | "elevated" | "outlined"; themeRev: number }) {
  const panelRef = useRef<THREE.Mesh>(null);
  const edgeRef = useRef<THREE.Group>(null);
  const cornerRefs = useRef<THREE.Mesh[]>([]);

  const colors = useMemo(
    () => {
      const v = resolveCelTokenPalette({
        baseToken: "--cel-surface-card", rimToken: "--cel-cyan", highlightToken: "--cel-text-primary",
        fallback: { base: [0.08, 0.1, 0.14], shadow: [0.02, 0.03, 0.04], highlight: [0.6, 0.7, 0.85], rim: [0, 0.941, 1] },
      });
      return {
        base: new THREE.Color(...v.base),
        shadow: new THREE.Color(...v.shadow),
        highlight: new THREE.Color(...v.highlight),
        rim: new THREE.Color(...v.rim),
      };
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [themeRev]
  );

  const panelGeom = useMemo(() => new THREE.BoxGeometry(4.0, 3.0, 0.06, 2, 2, 1), []);
  const edgeGeom = useMemo(() => new THREE.BoxGeometry(4.06, 3.06, 0.02, 2, 2, 1), []);
  const cornerGeom = useMemo(() => new THREE.BoxGeometry(0.18, 0.18, 0.12), []);

  const panelUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const edgeUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const cornerUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: 0.03 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const breathPhaseRef = useRef(0);

  useFrame((_, delta) => {
    // Subtle breathing glow on edges
    breathPhaseRef.current += delta * 0.6;
    const breath = Math.sin(breathPhaseRef.current) * 0.15 + 0.85;
    if (edgeRef.current) {
      edgeRef.current.children.forEach((child) => {
        if (child instanceof THREE.Mesh && child.material) {
          (child.material as THREE.ShaderMaterial).uniforms.uRimColor.value.setScalar(breath);
        }
      });
    }

    // Corner pulse
    if (cornerRefs.current.length) {
      cornerRefs.current.forEach((c, i) => {
        if (c.material) {
          const phase = breathPhaseRef.current + i * 1.5;
          const pulse = Math.sin(phase) * 0.2 + 0.8;
          c.scale.setScalar(pulse);
          (c.material as THREE.ShaderMaterial).uniforms.uRimColor.value.setScalar(pulse);
        }
      });
    }
  });

  const corners: readonly [number, number, number][] = [
    [-1.94, 1.44, 0.03], [1.94, 1.44, 0.03],
    [-1.94, -1.44, 0.03], [1.94, -1.44, 0.03],
  ] as const;

  return (
    <group>
      {/* Main panel */}
      <mesh ref={panelRef} geometry={panelGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={panelUniforms}
        />
      </mesh>
      <mesh geometry={panelGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={outlineUniforms}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Edge glow frame */}
      <group ref={edgeRef}>
        <mesh geometry={edgeGeom} position={[0, 0, 0.04]}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={edgeUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      {/* Corner accents */}
      {corners.map((pos, i) => (
        <mesh key={i} ref={(el) => { if (el) cornerRefs.current[i] = el; }} geometry={cornerGeom} position={pos}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={cornerUniforms}
          />
        </mesh>
      ))}
    </group>
  );
}

export function CelCard3D({ children, className = "", variant = "default" }: CelCard3DProps) {
  const [mounted, setMounted] = useState(false);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();

  useEffect(() => setMounted(true), []);

  if (!mounted) {
    return (
      <div className={`relative bg-[var(--cel-surface-card, var(--color-surface-2))] border border-[var(--cel-ink, var(--color-border))] p-4 ${className}`}>
        {children}
      </div>
    );
  }

  return (
    <div className={`relative ${className}`} style={{ width: "100%", height: "100%" }}>
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 4.5], fov: 25 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          className="absolute inset-0 -z-10 w-full h-full pointer-events-none"
        >
          <CelCardMesh variant={variant} themeRev={themeRev} />
        </Canvas>
      )}
      <div className="relative z-10 p-4 h-full w-full">{children}</div>
    </div>
  );
}

// ============================================================================
// 7. CEL CRYSTAL 3D — 暗夜神格八面体棱镜晶核
// ============================================================================
export type CelTone = "cyan" | "crimson" | "gold" | "ok";

function resolveToneColors(tone: CelTone) {
  const tokenMap: Record<CelTone, { base: string; rim: string; fbBase: [number, number, number]; fbRim: [number, number, number] }> = {
    cyan: { base: "--cel-cyan", rim: "--cel-crimson", fbBase: [0, 0.941, 1], fbRim: [1, 0.094, 0.298] },
    crimson: { base: "--cel-crimson", rim: "--cel-cyan", fbBase: [1, 0.094, 0.298], fbRim: [0, 0.941, 1] },
    gold: { base: "--cel-gold", rim: "--cel-cyan", fbBase: [1, 0.769, 0], fbRim: [0, 0.941, 1] },
    ok: { base: "--ok-DEFAULT", rim: "--cel-cyan", fbBase: [0.063, 0.725, 0.506], fbRim: [0, 0.941, 1] },
  };
  const cfg = tokenMap[tone] ?? tokenMap.cyan;
  const v = resolveCelTokenPalette({
    baseToken: cfg.base,
    rimToken: cfg.rim,
    fallback: {
      base: cfg.fbBase,
      shadow: [cfg.fbBase[0] * 0.16, cfg.fbBase[1] * 0.16, cfg.fbBase[2] * 0.16],
      highlight: [1, 1, 1],
      rim: cfg.fbRim,
    },
  });
  return {
    base: new THREE.Color(...v.base),
    shadow: new THREE.Color(...v.shadow),
    highlight: new THREE.Color(...v.highlight),
    rim: new THREE.Color(...v.rim),
  };
}

export interface CelCrystal3DProps {
  tone?: CelTone;
  size?: number;
  label?: string;
  value?: string | number;
  title?: string;
  className?: string;
  "data-testid"?: string;
}

function CelCrystalMesh({
  tone,
  hovered,
  burstTime,
  themeRev,
  reducedMotion,
}: {
  tone: CelTone;
  hovered: boolean;
  burstTime: number;
  themeRev: number;
  reducedMotion: boolean;
}) {
  const crystalRef = useRef<THREE.Group>(null);
  const ringARef = useRef<THREE.Mesh>(null);
  const ringBRef = useRef<THREE.Mesh>(null);
  const shardsRef = useRef<THREE.Mesh[]>([]);

  const colors = useMemo(
    () => resolveToneColors(tone),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [tone, themeRev]
  );

  const crystalGeom = useMemo(() => new THREE.OctahedronGeometry(0.74, 0), []);
  const ringGeom = useMemo(() => new THREE.TorusGeometry(1.12, 0.032, 8, 32), []);
  const shardGeom = useMemo(() => new THREE.TetrahedronGeometry(0.11, 0), []);

  const crystalUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors]
  );

  const ringUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.rim },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.base },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: 0.045 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  useFrame((_, delta) => {
    if (reducedMotion) return;
    const isBursting = Date.now() - burstTime < 600;
    const speed = isBursting ? 5.5 : hovered ? 2.4 : 1.0;

    if (crystalRef.current) {
      crystalRef.current.rotation.y += delta * 0.95 * speed;
      crystalRef.current.rotation.x = Math.sin(Date.now() * 0.0015) * 0.2;
      const scale = isBursting ? 1.15 : hovered ? 1.06 : 1.0;
      crystalRef.current.scale.lerp(new THREE.Vector3(scale, scale, scale), 0.15);
    }
    if (ringARef.current) {
      ringARef.current.rotation.z += delta * 1.1 * speed;
      ringARef.current.rotation.x += delta * 0.4 * speed;
    }
    if (ringBRef.current) {
      ringBRef.current.rotation.z -= delta * 0.9 * speed;
      ringBRef.current.rotation.y += delta * 0.6 * speed;
    }
    shardsRef.current.forEach((shard, idx) => {
      if (!shard) return;
      const t = Date.now() * 0.0018 * (isBursting ? 2.5 : 1) + idx * (Math.PI / 2);
      const radius = (hovered || isBursting ? 1.48 : 1.28) + Math.sin(t * 1.5) * 0.08;
      shard.position.set(
        Math.cos(t) * radius,
        Math.sin(t * 1.4 + idx) * 0.45,
        Math.sin(t) * radius
      );
      shard.rotation.x += delta * 2.5 * speed;
      shard.rotation.y += delta * 1.8 * speed;
    });
  });

  return (
    <group>
      <group ref={crystalRef}>
        <mesh geometry={crystalGeom}>
          <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={crystalUniforms} />
        </mesh>
        <mesh geometry={crystalGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      <mesh ref={ringARef} geometry={ringGeom} rotation={[Math.PI / 3, 0.25, 0]}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={ringUniforms} />
      </mesh>
      <mesh ref={ringBRef} geometry={ringGeom} rotation={[-Math.PI / 3, -0.25, Math.PI / 4]}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={crystalUniforms} />
      </mesh>

      {[0, 1, 2, 3].map((idx) => (
        <mesh
          key={idx}
          ref={(el) => { if (el) shardsRef.current[idx] = el; }}
          geometry={shardGeom}
          position={[Math.cos((idx * Math.PI) / 2) * 1.28, 0, Math.sin((idx * Math.PI) / 2) * 1.28]}
        >
          <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={ringUniforms} />
        </mesh>
      ))}
    </group>
  );
}

export function CelCrystal3D({
  tone = "cyan",
  size = 44,
  label,
  value,
  title = "Dark Divinity Crystal Core [真3D 晶核]",
  className = "",
  "data-testid": testId,
}: CelCrystal3DProps) {
  const [mounted, setMounted] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [burstTime, setBurstTime] = useState(0);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => setMounted(true), []);

  const handleTrigger = useCallback(() => setBurstTime(Date.now()), []);

  if (!mounted) {
    return (
      <div
        data-testid={testId}
        style={{ width: size, height: size }}
        className={`divinity-3d-frame grid place-items-center shrink-0 ${className}`}
      >
        <span className="size-2.5 rotate-45 bg-[var(--cel-cyan)] border border-[var(--cel-ink)] animate-pulse" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={label ? `${label} ${value ?? ""}`.trim() : title}
      title={title}
      data-testid={testId}
      onClick={handleTrigger}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleTrigger();
        }
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ width: size, height: size }}
      className={`divinity-3d-frame relative cursor-pointer select-none overflow-hidden shrink-0 transition-transform duration-100 hover:scale-105 active:scale-95 ${className}`}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 3.3], fov: 42 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          frameloop={reducedMotion ? "demand" : "always"}
          className="w-full h-full pointer-events-none"
        >
          <CelCrystalMesh
            tone={tone}
            hovered={hovered}
            burstTime={burstTime}
            themeRev={themeRev}
            reducedMotion={reducedMotion}
          />
        </Canvas>
      )}
      {value !== undefined && (
        <span className="absolute bottom-0.5 right-1 font-mono text-[9px] font-black text-[var(--cel-text-primary)] pointer-events-none">
          {value}
        </span>
      )}
    </div>
  );
}

// ============================================================================
// 8. CEL HOLO CUBE 3D — 战术全息魔方 / 容器拓扑核
// ============================================================================
export interface CelHoloCube3DProps {
  status?: "optimal" | "syncing" | "warning";
  size?: number;
  title?: string;
  className?: string;
  "data-testid"?: string;
}

function CelHoloCubeMesh({
  status,
  hovered,
  burstTime,
  themeRev,
  reducedMotion,
}: {
  status: "optimal" | "syncing" | "warning";
  hovered: boolean;
  burstTime: number;
  themeRev: number;
  reducedMotion: boolean;
}) {
  const outerRef = useRef<THREE.Mesh>(null);
  const coreRef = useRef<THREE.Group>(null);
  const scanRef = useRef<THREE.Mesh>(null);

  const colors = useMemo(() => {
    const tone: CelTone = status === "warning" ? "gold" : status === "syncing" ? "crimson" : "cyan";
    return resolveToneColors(tone);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, themeRev]);

  const outerGeom = useMemo(() => new THREE.BoxGeometry(1.42, 1.42, 1.42), []);
  const coreGeom = useMemo(() => new THREE.DodecahedronGeometry(0.54, 0), []);
  const scanGeom = useMemo(() => new THREE.RingGeometry(0.75, 0.88, 4), []);

  const coreUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: 0.04 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  useFrame((_, delta) => {
    if (reducedMotion) return;
    const isBursting = Date.now() - burstTime < 600;
    const speed = isBursting ? 5.0 : hovered ? 2.2 : status === "syncing" ? 1.6 : 1.0;

    if (outerRef.current) {
      outerRef.current.rotation.x += delta * 0.45 * speed;
      outerRef.current.rotation.y += delta * 0.65 * speed;
    }
    if (coreRef.current) {
      coreRef.current.rotation.x -= delta * 0.9 * speed;
      coreRef.current.rotation.y += delta * 1.1 * speed;
    }
    if (scanRef.current) {
      const wave = Math.sin(Date.now() * 0.003 * speed);
      scanRef.current.position.y = wave * 0.62;
      scanRef.current.rotation.z += delta * 1.2 * speed;
    }
  });

  return (
    <group>
      <mesh ref={outerRef} geometry={outerGeom} rotation={[0.45, 0.6, 0]}>
        <meshBasicMaterial color={colors.base} wireframe transparent opacity={0.75} />
      </mesh>

      <group ref={coreRef}>
        <mesh geometry={coreGeom}>
          <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={coreUniforms} />
        </mesh>
        <mesh geometry={coreGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      <mesh ref={scanRef} geometry={scanGeom} rotation={[-Math.PI / 2, 0, Math.PI / 4]}>
        <meshBasicMaterial color={colors.rim} transparent opacity={0.65} side={THREE.DoubleSide} />
      </mesh>
    </group>
  );
}

export function CelHoloCube3D({
  status = "optimal",
  size = 44,
  title = "Tactical HoloCube 3D [真3D 拓扑核]",
  className = "",
  "data-testid": testId,
}: CelHoloCube3DProps) {
  const [mounted, setMounted] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [burstTime, setBurstTime] = useState(0);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => setMounted(true), []);

  const handleTrigger = useCallback(() => setBurstTime(Date.now()), []);

  if (!mounted) {
    return (
      <div
        data-testid={testId}
        style={{ width: size, height: size }}
        className={`divinity-3d-frame grid place-items-center shrink-0 ${className}`}
      >
        <span className="size-3 border-2 border-[var(--cel-cyan)] animate-pulse" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={title}
      title={title}
      data-testid={testId}
      onClick={handleTrigger}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleTrigger();
        }
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ width: size, height: size }}
      className={`divinity-3d-frame relative cursor-pointer select-none overflow-hidden shrink-0 transition-transform duration-100 hover:scale-105 active:scale-95 ${className}`}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 3.4], fov: 42 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          frameloop={reducedMotion ? "demand" : "always"}
          className="w-full h-full pointer-events-none"
        >
          <CelHoloCubeMesh
            status={status}
            hovered={hovered}
            burstTime={burstTime}
            themeRev={themeRev}
            reducedMotion={reducedMotion}
          />
        </Canvas>
      )}
    </div>
  );
}

// ============================================================================
// 9. CEL ORBITAL GAUGE 3D — 真3D三渲二轨道进度仪
// ============================================================================
export interface CelOrbitalGauge3DProps {
  progress: number;
  tone?: CelTone;
  size?: number;
  title?: string;
  className?: string;
  "data-testid"?: string;
}

function CelOrbitalGaugeMesh({
  progress,
  tone,
  hovered,
  burstTime,
  themeRev,
  reducedMotion,
}: {
  progress: number;
  tone: CelTone;
  hovered: boolean;
  burstTime: number;
  themeRev: number;
  reducedMotion: boolean;
}) {
  const gaugeRef = useRef<THREE.Group>(null);
  const coreRef = useRef<THREE.Mesh>(null);
  const satRef = useRef<THREE.Mesh>(null);

  const clamped = Math.max(0, Math.min(100, Number.isFinite(progress) ? progress : 0));
  const arcAngle = Math.max(0.25, (clamped / 100) * Math.PI * 2);

  const colors = useMemo(
    () => resolveToneColors(tone),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [tone, themeRev]
  );

  const trackGeom = useMemo(() => new THREE.TorusGeometry(1.06, 0.038, 8, 40), []);
  const arcGeom = useMemo(() => new THREE.TorusGeometry(1.06, 0.085, 10, 40, arcAngle), [arcAngle]);
  const coreGeom = useMemo(() => new THREE.IcosahedronGeometry(0.46, 0), []);
  const satGeom = useMemo(() => new THREE.OctahedronGeometry(0.11, 0), []);

  const arcUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors]
  );

  const trackUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.shadow },
      uShadowColor: { value: new THREE.Color(0.02, 0.03, 0.06) },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.base },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  useFrame((_, delta) => {
    if (reducedMotion) return;
    const isBursting = Date.now() - burstTime < 600;
    const speed = isBursting ? 4.5 : hovered ? 2.0 : 1.0;

    if (gaugeRef.current) {
      gaugeRef.current.rotation.z += delta * 0.45 * speed;
      gaugeRef.current.rotation.x = 0.42 + Math.sin(Date.now() * 0.0014) * 0.14;
    }
    if (coreRef.current) {
      coreRef.current.rotation.y += delta * 1.1 * speed;
      coreRef.current.rotation.x += delta * 0.6 * speed;
    }
    if (satRef.current) {
      satRef.current.position.set(
        Math.cos(arcAngle) * 1.06,
        Math.sin(arcAngle) * 1.06,
        0
      );
      satRef.current.rotation.z += delta * 2.5 * speed;
    }
  });

  return (
    <group ref={gaugeRef} rotation={[0.42, -0.25, 0]}>
      <mesh geometry={trackGeom}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={trackUniforms} />
      </mesh>
      <mesh geometry={arcGeom}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={arcUniforms} />
      </mesh>
      <mesh ref={coreRef} geometry={coreGeom}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={arcUniforms} />
      </mesh>
      <mesh ref={satRef} geometry={satGeom} position={[Math.cos(arcAngle) * 1.06, Math.sin(arcAngle) * 1.06, 0]}>
        <shaderMaterial vertexShader={CEL_UI_VERTEX} fragmentShader={CEL_UI_FRAGMENT} uniforms={trackUniforms} />
      </mesh>
    </group>
  );
}

export function CelOrbitalGauge3D({
  progress,
  tone = "cyan",
  size = 44,
  title = "Orbital Gauge 3D [真3D 轨道仪表]",
  className = "",
  "data-testid": testId,
}: CelOrbitalGauge3DProps) {
  const [mounted, setMounted] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [burstTime, setBurstTime] = useState(0);
  const themeRev = useThemeRevision();
  const isPageVisible = usePageVisibility();
  const reducedMotion = usePrefersReducedMotion();

  useEffect(() => setMounted(true), []);

  const clamped = Math.round(Math.max(0, Math.min(100, Number.isFinite(progress) ? progress : 0)));
  const handleTrigger = useCallback(() => setBurstTime(Date.now()), []);

  if (!mounted) {
    return (
      <div
        data-testid={testId}
        style={{ width: size, height: size }}
        className={`divinity-3d-frame grid place-items-center shrink-0 ${className}`}
      >
        <span className="font-mono text-[9px] font-black text-[var(--cel-cyan)]">{clamped}%</span>
      </div>
    );
  }

  return (
    <div
      role="img"
      aria-label={`${title}: ${clamped}%`}
      title={`${title} (${clamped}%)`}
      data-testid={testId}
      onClick={handleTrigger}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{ width: size, height: size }}
      className={`divinity-3d-frame relative cursor-pointer select-none overflow-hidden shrink-0 transition-transform duration-100 hover:scale-105 ${className}`}
    >
      {isPageVisible && (
        <Canvas
          camera={{ position: [0, 0, 3.2], fov: 44 }}
          gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
          frameloop={reducedMotion ? "demand" : "always"}
          className="w-full h-full pointer-events-none"
        >
          <CelOrbitalGaugeMesh
            progress={clamped}
            tone={tone}
            hovered={hovered}
            burstTime={burstTime}
            themeRev={themeRev}
            reducedMotion={reducedMotion}
          />
        </Canvas>
      )}
    </div>
  );
}
