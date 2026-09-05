"use client";

import React, { useMemo, useRef, useState, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

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
// 2. CEL BUTTON 3D (三渲二真3D物理按压按钮 - 宗师级工效学)
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
}: {
  variant: "crimson" | "cyan" | "gold" | "stealth";
  pressed: boolean;
  hovered: boolean;
  focused: boolean;
}) {
  const meshRef = useRef<THREE.Group>(null);

  const colors = useMemo(() => {
    switch (variant) {
      case "cyan":
        return {
          base: new THREE.Color("#00f0ff"),
          shadow: new THREE.Color("#022230"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#ff184c"),
        };
      case "gold":
        return {
          base: new THREE.Color("#ffc400"),
          shadow: new THREE.Color("#332402"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#00f0ff"),
        };
      case "stealth":
        return {
          base: new THREE.Color("#2a3547"),
          shadow: new THREE.Color("#0c111c"),
          highlight: new THREE.Color("#8fa5cc"),
          rim: new THREE.Color("#00f0ff"),
        };
      case "crimson":
      default:
        return {
          base: new THREE.Color("#ff184c"),
          shadow: new THREE.Color("#380410"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#00f0ff"),
        };
    }
  }, [variant]);

  const geom = useMemo(() => new THREE.BoxGeometry(2.4, 0.75, 0.4), []);

  const uniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: focused ? new THREE.Color("#00f0ff") : colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 3.0 },
    }),
    [colors, focused]
  );

  const outlineUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(3, 4, 5) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: focused ? 0.07 : 0.045 },
      uBands: { value: 3.0 },
    }),
    [colors, focused]
  );

  useFrame(() => {
    if (meshRef.current) {
      // Physical depression in 3D space with spring return
      const targetZ = pressed ? -0.16 : hovered ? 0.06 : 0.0;
      meshRef.current.position.z += (targetZ - meshRef.current.position.z) * 0.4;
      const targetRotX = pressed ? -0.06 : hovered ? 0.04 : 0.0;
      meshRef.current.rotation.x += (targetRotX - meshRef.current.rotation.x) * 0.4;
    }
  });

  return (
    <group ref={meshRef}>
      <mesh geometry={geom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={uniforms}
        />
      </mesh>
      <mesh geometry={geom}>
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

  useEffect(() => setMounted(true), []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      setPressed(true);
      onClick?.();
      setTimeout(() => setPressed(false), 120);
    }
  };

  if (!mounted) {
    return (
      <button
        disabled={disabled}
        className={`px-4 py-2 bg-[#ff184c] text-black font-black text-xs ${className}`}
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
      onMouseLeave={() => {
        setHovered(false);
        setPressed(false);
      }}
      onFocus={() => setFocused(true)}
      onBlur={() => {
        setFocused(false);
        setPressed(false);
      }}
      onKeyDown={handleKeyDown}
      style={{ width: "160px", height: "54px" }}
    >
      <div className="absolute inset-0 pointer-events-none">
        <Canvas camera={{ position: [0, 0, 3.2], fov: 45 }} gl={{ alpha: true }}>
          <CelButtonMesh
            variant={variant}
            pressed={pressed}
            hovered={hovered}
            focused={focused}
          />
        </Canvas>
      </div>

      {/* Focus Ring Indicator (Anime Chamfered Border) */}
      {focused && (
        <div className="absolute -inset-1 border-2 border-[#00f0ff] pointer-events-none animate-pulse" />
      )}

      {/* Label layer overlaid on the 3D surface */}
      <div
        className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none transition-transform duration-75"
        style={{
          transform: pressed ? "translateY(2px)" : hovered ? "translateY(-1px)" : "none",
        }}
      >
        <span className="text-xs font-black tracking-wider uppercase text-black italic drop-shadow-[0_1px_1px_rgba(255,255,255,0.8)]">
          {label}
        </span>
        {sublabel && (
          <span className="text-[8px] font-bold text-black/70 font-mono tracking-tighter uppercase -mt-0.5">
            {sublabel}
          </span>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// 3. CEL TOGGLE 3D (三渲二机械翻转拨钮 - 零歧义状态感知)
// ============================================================================
export interface CelToggle3DProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  sublabel?: string;
  className?: string;
}

function CelToggleMesh({ checked, focused }: { checked: boolean; focused: boolean }) {
  const leverRef = useRef<THREE.Group>(null);

  const colorsOn = useMemo(
    () => ({
      base: new THREE.Color("#ff184c"),
      shadow: new THREE.Color("#380410"),
      highlight: new THREE.Color("#ffffff"),
      rim: new THREE.Color("#00f0ff"),
    }),
    []
  );

  const colorsOff = useMemo(
    () => ({
      base: new THREE.Color("#1a2233"),
      shadow: new THREE.Color("#080b12"),
      highlight: new THREE.Color("#7384a6"),
      rim: new THREE.Color("#00f0ff"),
    }),
    []
  );

  const baseGeom = useMemo(() => new THREE.BoxGeometry(2.0, 0.8, 0.25), []);
  const leverGeom = useMemo(() => new THREE.CylinderGeometry(0.2, 0.25, 0.9, 12), []);

  const uniforms = useMemo(
    () => ({
      uBaseColor: { value: checked ? colorsOn.base : colorsOff.base },
      uShadowColor: { value: checked ? colorsOn.shadow : colorsOff.shadow },
      uHighlightColor: { value: checked ? colorsOn.highlight : colorsOff.highlight },
      uRimColor: { value: focused ? new THREE.Color("#00f0ff") : colorsOn.rim },
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

  useFrame(() => {
    if (leverRef.current) {
      const targetAngle = checked ? 0.45 : -0.45;
      leverRef.current.rotation.z += (targetAngle - leverRef.current.rotation.z) * 0.32;
      const targetX = checked ? 0.45 : -0.45;
      leverRef.current.position.x += (targetX - leverRef.current.position.x) * 0.32;
    }
  });

  return (
    <group>
      <mesh geometry={baseGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={uniforms}
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

      <group ref={leverRef} position={[checked ? 0.45 : -0.45, 0, 0.2]}>
        <mesh geometry={leverGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={uniforms}
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
      </group>
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
  useEffect(() => setMounted(true), []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onChange(!checked);
    }
  };

  return (
    <div
      role="switch"
      tabIndex={0}
      aria-checked={checked}
      aria-label={label}
      className={`flex items-center gap-2.5 cursor-pointer select-none focus:outline-none p-1 rounded-sm ${className}`}
      onClick={() => onChange(!checked)}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onKeyDown={handleKeyDown}
    >
      <div style={{ width: "84px", height: "38px" }} className="relative">
        {mounted && (
          <Canvas camera={{ position: [0, 0, 2.8], fov: 45 }} gl={{ alpha: true }}>
            <CelToggleMesh checked={checked} focused={focused} />
          </Canvas>
        )}
      </div>

      {(label || sublabel) && (
        <div className="flex flex-col">
          {label && (
            <span
              className={`text-xs font-black uppercase tracking-wider transition-colors ${
                checked ? "text-[#ff184c]" : "text-slate-400"
              }`}
            >
              {label}
            </span>
          )}
          {sublabel && (
            <span className="text-[9px] font-mono text-slate-500 tracking-tighter">
              {sublabel}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

// ============================================================================
// 4. CEL BEACON 3D (三渲二真3D状态菱形浮标 - 周边视觉即时唤醒)
// ============================================================================
export interface CelBeacon3DProps {
  status: "idle" | "active" | "alert" | "success";
  size?: number;
  className?: string;
  title?: string;
}

function CelBeaconMesh({ status }: { status: "idle" | "active" | "alert" | "success" }) {
  const beaconRef = useRef<THREE.Group>(null);
  const ringRef = useRef<THREE.Group>(null);

  const colors = useMemo(() => {
    switch (status) {
      case "alert":
        return {
          base: new THREE.Color("#ff184c"),
          shadow: new THREE.Color("#380410"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#ffc400"),
        };
      case "active":
        return {
          base: new THREE.Color("#ffc400"),
          shadow: new THREE.Color("#382802"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#00f0ff"),
        };
      case "success":
        return {
          base: new THREE.Color("#00f0ff"),
          shadow: new THREE.Color("#022230"),
          highlight: new THREE.Color("#ffffff"),
          rim: new THREE.Color("#ff184c"),
        };
      case "idle":
      default:
        return {
          base: new THREE.Color("#3d495c"),
          shadow: new THREE.Color("#0e131c"),
          highlight: new THREE.Color("#a2b4d1"),
          rim: new THREE.Color("#00f0ff"),
        };
    }
  }, [status]);

  const gemGeom = useMemo(() => new THREE.OctahedronGeometry(0.8, 0), []);
  const ringGeom = useMemo(() => new THREE.TorusGeometry(1.2, 0.05, 8, 32), []);

  const uniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
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
      uLightPos: { value: new THREE.Vector3(2, 3, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: 0.05 },
      uBands: { value: 3.0 },
    }),
    [colors]
  );

  useFrame((_, delta) => {
    const rotSpeed = status === "alert" ? 3.5 : status === "active" ? 2.0 : 0.8;
    if (beaconRef.current) {
      beaconRef.current.rotation.y += delta * rotSpeed;
      beaconRef.current.rotation.x = Math.sin(Date.now() * 0.002) * 0.2;
    }
    if (ringRef.current) {
      ringRef.current.rotation.x += delta * rotSpeed * 0.6;
      ringRef.current.rotation.y -= delta * rotSpeed * 0.4;
    }
  });

  return (
    <group>
      <group ref={beaconRef}>
        <mesh geometry={gemGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={uniforms}
          />
        </mesh>
        <mesh geometry={gemGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      <group ref={ringRef}>
        <mesh geometry={ringGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={uniforms}
          />
        </mesh>
      </group>
    </group>
  );
}

export function CelBeacon3D({
  status,
  size = 42,
  className = "",
  title,
}: CelBeacon3DProps) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  if (!mounted) {
    return <div style={{ width: size, height: size }} className="bg-black/50" />;
  }

  return (
    <div
      title={title || `Status: ${status}`}
      style={{ width: size, height: size }}
      className={`relative pointer-events-none ${className}`}
    >
      <Canvas camera={{ position: [0, 0, 3.2], fov: 45 }} gl={{ alpha: true }}>
        <CelBeaconMesh status={status} />
      </Canvas>
    </div>
  );
}

// ============================================================================
// 5. CEL DIAL 3D (三渲二真3D分段旋钮 - 离散状态快速索引)
// ============================================================================
export interface CelDial3DProps {
  value: number; // 0, 1, 2
  onChange: (value: number) => void;
  options: string[];
  className?: string;
}

function CelDialMesh({ value }: { value: number }) {
  const dialRef = useRef<THREE.Group>(null);

  const colors = useMemo(
    () => ({
      base: new THREE.Color("#182133"),
      shadow: new THREE.Color("#070a10"),
      highlight: new THREE.Color("#00f0ff"),
      rim: new THREE.Color("#ff184c"),
    }),
    []
  );

  const cylinderGeom = useMemo(() => new THREE.CylinderGeometry(1.0, 1.0, 0.35, 16), []);
  const notchGeom = useMemo(() => new THREE.BoxGeometry(0.18, 0.4, 0.3), []);

  const uniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.base },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.highlight },
      uRimColor: { value: colors.rim },
      uLightPos: { value: new THREE.Vector3(2, 4, 4) },
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
      uLightPos: { value: new THREE.Vector3(2, 4, 4) },
      uIsOutline: { value: 1.0 },
      uOutlineOffset: { value: 0.045 },
      uBands: { value: 3.0 },
    }),
    [colors]
  );

  useFrame(() => {
    if (dialRef.current) {
      // 3 discrete positions: 0deg, 60deg, 120deg
      const targetAngle = (value * Math.PI) / 3;
      dialRef.current.rotation.y += (targetAngle - dialRef.current.rotation.y) * 0.35;
    }
  });

  return (
    <group ref={dialRef} rotation={[0.4, 0, 0]}>
      <mesh geometry={cylinderGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={uniforms}
        />
      </mesh>
      <mesh geometry={cylinderGeom}>
        <shaderMaterial
          vertexShader={CEL_UI_VERTEX}
          fragmentShader={CEL_UI_FRAGMENT}
          uniforms={outlineUniforms}
          side={THREE.BackSide}
        />
      </mesh>

      {/* Indicator Notch */}
      <group position={[0, 0, 0.9]}>
        <mesh geometry={notchGeom}>
          <shaderMaterial
            vertexShader={CEL_UI_VERTEX}
            fragmentShader={CEL_UI_FRAGMENT}
            uniforms={{
              ...uniforms,
              uBaseColor: { value: new THREE.Color("#00f0ff") },
            }}
          />
        </mesh>
      </group>
    </group>
  );
}

export function CelDial3D({ value, onChange, options, className = "" }: CelDial3DProps) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  const handleClick = () => {
    const next = (value + 1) % options.length;
    onChange(next);
  };

  return (
    <div
      role="slider"
      aria-valuemin={0}
      aria-valuemax={options.length - 1}
      aria-valuenow={value}
      aria-label="Tactical Rotary Dial"
      tabIndex={0}
      onClick={handleClick}
      className={`flex items-center gap-3 cursor-pointer select-none focus:outline-none p-1.5 ${className}`}
    >
      <div style={{ width: "64px", height: "54px" }} className="relative">
        {mounted && (
          <Canvas camera={{ position: [0, 0, 2.6], fov: 45 }} gl={{ alpha: true }}>
            <CelDialMesh value={value} />
          </Canvas>
        )}
      </div>

      <div className="flex flex-col">
        <span className="text-[9px] font-mono text-slate-500 uppercase tracking-widest">
          ROTARY INDEX [{value + 1}/{options.length}]
        </span>
        <span className="text-xs font-black text-[#00f0ff] uppercase tracking-wider italic">
          {options[value]}
        </span>
      </div>
    </div>
  );
}
