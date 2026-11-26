/*
@cypher schema=1 source_sha256=5727c6bd2f648712bebccf1ef0f511e2ca5a341778c04d4012a547b58a72c17b
MERGE (self:File {path:"frontend/src/components/effects/TacticalCore3D.tsx"})
MERGE (fx:Symbol {id:"frontend/src/components/effects/TacticalCore3D.tsx::TacticalCore3D",kind:"component"})
MERGE (self)-[:DEFINES]->(fx)
@endcypher
*/
"use client";

import { useMemo, useRef, useState, useEffect, useCallback } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { readThemeRgb } from "@/lib/theme/themeTokens";

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
// 2. ENERGY RING SHADER (additive glow)
// ============================================================================
const ENERGY_RING_VERTEX = /* glsl */ `
  varying vec3 vNormal;
  varying vec3 vViewPos;
  varying vec3 vWorldPos;
  varying vec2 vUv;

  void main() {
    vUv = uv;
    vNormal = normalize(normalMatrix * normal);
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    vViewPos = -mvPos.xyz;
    vWorldPos = (modelMatrix * vec4(position, 1.0)).xyz;
    gl_Position = projectionMatrix * mvPos;
  }
`;

const ENERGY_RING_FRAGMENT = /* glsl */ `
  uniform vec3 uGlowColor;
  uniform vec3 uCoreColor;
  uniform float uTime;
  uniform float uIntensity;

  varying vec3 vNormal;
  varying vec3 vViewPos;
  varying vec3 vWorldPos;
  varying vec2 vUv;

  void main() {
    vec3 N = normalize(vNormal);
    vec3 V = normalize(vViewPos);

    // Fresnel glow
    float fresnel = 1.0 - max(0.0, dot(N, V));
    float glow = smoothstep(0.3, 0.9, fresnel);

    // Pulsing energy wave along ring
    float wave = sin(vUv.x * 8.0 - uTime * 4.0) * 0.5 + 0.5;
    float pulse = smoothstep(0.4, 0.6, wave);

    vec3 color = mix(uCoreColor, uGlowColor, glow * pulse * uIntensity);

    // Additive blending effect
    float alpha = glow * pulse * uIntensity * 0.8;

    gl_FragColor = vec4(color, alpha);
  }
`;

// ============================================================================
// 3. PROCEDURAL 3D MESH RIG — TRUE GIMBAL SYSTEM
// ============================================================================
export type TacticalCoreTheme = "light" | "dark" | "christmas" | "charisma" | "shanghai" | "cel" | "physis";

function CoreMesh({
  status,
  isHovered,
  burstTime,
  theme = "dark",
  isLight = false,
}: {
  status: "nominal" | "warning" | "overdrive";
  isHovered: boolean;
  burstTime: number;
  theme?: TacticalCoreTheme;
  isLight?: boolean;
}) {
  // Three independent gimbal groups
  const innerRef = useRef<THREE.Group>(null);   // Core icosahedron
  const middleRef = useRef<THREE.Group>(null);  // Middle gimbal ring
  const outerRef = useRef<THREE.Group>(null);   // Outer gimbal ring
  const energyRing1Ref = useRef<THREE.Mesh>(null);
  const energyRing2Ref = useRef<THREE.Mesh>(null);
  const energyRing3Ref = useRef<THREE.Mesh>(null);
  const particleRefs = useRef<THREE.Mesh[]>([]);

  const effectiveTheme = isLight ? "light" : theme;

  // per 2026-11-17 主题统一: 监听 html class 变化, 切主题时重算色板
  const [themeRev, setThemeRev] = useState(0);
  useEffect(() => {
    const obs = new MutationObserver(() => setThemeRev((r) => r + 1));
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    return () => obs.disconnect();
  }, []);

  const colors = useMemo(() => {
    // ── 0. 日式赛璐璐 (80-90s 经典日漫手绘平涂色与机械墨线) ──
    if (effectiveTheme === "cel") {
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#f59e0b"),
            shadow: new THREE.Color("#291500"),
            keyRim: new THREE.Color("#ea580c"),
            counterRim: new THREE.Color("#2563eb"),
            outline: new THREE.Color("#111827"),
            energy: new THREE.Color("#f59e0b"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#ea580c"),
            shadow: new THREE.Color("#270802"),
            keyRim: new THREE.Color("#f59e0b"),
            counterRim: new THREE.Color("#2563eb"),
            outline: new THREE.Color("#111827"),
            energy: new THREE.Color("#ea580c"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#2563eb"),
            shadow: new THREE.Color("#0f172a"),
            keyRim: new THREE.Color("#ea580c"),
            counterRim: new THREE.Color("#f59e0b"),
            outline: new THREE.Color("#111827"),
            energy: new THREE.Color("#00f0ff"),
          };
      }
    }

    // ── 1. 老上海月份牌 (民国海派擦笔水彩与留声机鎏金) ──
    if (effectiveTheme === "shanghai") {
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#c89228"),
            shadow: new THREE.Color("#201208"),
            keyRim: new THREE.Color("#b8282b"),
            counterRim: new THREE.Color("#1b5e48"),
            outline: new THREE.Color("#7a1518"),
            energy: new THREE.Color("#d49e35"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#b8282b"),
            shadow: new THREE.Color("#330408"),
            keyRim: new THREE.Color("#d49e35"),
            counterRim: new THREE.Color("#1b5e48"),
            outline: new THREE.Color("#221c16"),
            energy: new THREE.Color("#b8282b"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#d49e35"),
            shadow: new THREE.Color("#2b170c"),
            keyRim: new THREE.Color("#b8282b"),
            counterRim: new THREE.Color("#1b5e48"),
            outline: new THREE.Color("#c89228"),
            energy: new THREE.Color("#d49e35"),
          };
      }
    }

    // ── 2. 极魅像素 (歧路旅人 HD-2D 神恩紫罗兰与醉梦丝绒红) ──
    if (effectiveTheme === "charisma") {
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#f59e0b"),
            shadow: new THREE.Color("#241602"),
            keyRim: new THREE.Color("#8b5cf6"),
            counterRim: new THREE.Color("#e11d48"),
            outline: new THREE.Color("#050409"),
            energy: new THREE.Color("#f59e0b"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#e11d48"),
            shadow: new THREE.Color("#28040b"),
            keyRim: new THREE.Color("#8b5cf6"),
            counterRim: new THREE.Color("#f59e0b"),
            outline: new THREE.Color("#050409"),
            energy: new THREE.Color("#e11d48"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#8b5cf6"),
            shadow: new THREE.Color("#140d24"),
            keyRim: new THREE.Color("#e11d48"),
            counterRim: new THREE.Color("#f59e0b"),
            outline: new THREE.Color("#050409"),
            energy: new THREE.Color("#8b5cf6"),
          };
      }
    }

    // ── 3. 圣夜霜雪 (常规毛玻璃冬日绯红与常青松绿) ──
    if (effectiveTheme === "christmas") {
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#f8b229"),
            shadow: new THREE.Color("#241803"),
            keyRim: new THREE.Color("#d42426"),
            counterRim: new THREE.Color("#22c55e"),
            outline: new THREE.Color("#060b08"),
            energy: new THREE.Color("#f8b229"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#165b33"),
            shadow: new THREE.Color("#05170b"),
            keyRim: new THREE.Color("#f8b229"),
            counterRim: new THREE.Color("#d42426"),
            outline: new THREE.Color("#060b08"),
            energy: new THREE.Color("#165b33"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#d42426"),
            shadow: new THREE.Color("#140405"),
            keyRim: new THREE.Color("#f8b229"),
            counterRim: new THREE.Color("#165b33"),
            outline: new THREE.Color("#060b08"),
            energy: new THREE.Color("#d42426"),
          };
      }
    }

    // ── 4. 少年原画 (Shōnen Studio Archival Paper Palette) ──
    if (effectiveTheme === "light") {
      switch (status) {
        case "warning":
          return {
            base: new THREE.Color("#d48800"),
            shadow: new THREE.Color("#362100"),
            keyRim: new THREE.Color("#e60033"),
            counterRim: new THREE.Color("#0055ff"),
            outline: new THREE.Color("#0a0d14"),
            energy: new THREE.Color("#d48800"),
          };
        case "overdrive":
          return {
            base: new THREE.Color("#e60033"),
            shadow: new THREE.Color("#3a040d"),
            keyRim: new THREE.Color("#0055ff"),
            counterRim: new THREE.Color("#d48800"),
            outline: new THREE.Color("#0a0d14"),
            energy: new THREE.Color("#e60033"),
          };
        case "nominal":
        default:
          return {
            base: new THREE.Color("#0055ff"),
            shadow: new THREE.Color("#061230"),
            keyRim: new THREE.Color("#e60033"),
            counterRim: new THREE.Color("#d48800"),
            outline: new THREE.Color("#0a0d14"),
            energy: new THREE.Color("#0055ff"),
          };
      }
    }

    // ── 5. 主题 token 覆盖 (per 2026-11-17 主题统一: physis 及扩展主题从 CSS token 取色) ──
    if (effectiveTheme !== "dark") {
      const baseToken = status === "overdrive" ? "--cel-crimson"
        : status === "warning" ? "--cel-gold" : "--cel-cyan";
      const base = readThemeRgb(baseToken, [0, 0.941, 1]);
      const keyRim = readThemeRgb(status === "nominal" ? "--cel-crimson" : "--cel-cyan", [1, 0.094, 0.298]);
      const counterRim = readThemeRgb("--cel-gold", [1, 0.769, 0]);
      const outline = readThemeRgb("--cel-ink", [0, 0, 0]);
      const c = (v: [number, number, number]) => new THREE.Color(v[0], v[1], v[2]);
      return {
        base: c(base),
        shadow: c([base[0] * 0.16, base[1] * 0.16, base[2] * 0.16]),
        keyRim: c(keyRim),
        counterRim: c(counterRim),
        outline: c(outline),
        energy: c(base),
      };
    }

    // ── 6. 暗夜神格 (Neo-Tokyo Cyber Manga Void Palette - 默认) ──
    switch (status) {
      case "warning":
        return {
          base: new THREE.Color("#ffc400"),
          shadow: new THREE.Color("#2e1c00"),
          keyRim: new THREE.Color("#ff184c"),
          counterRim: new THREE.Color("#00f0ff"),
          outline: new THREE.Color("#000000"),
          energy: new THREE.Color("#ffc400"),
        };
      case "overdrive":
        return {
          base: new THREE.Color("#ff184c"),
          shadow: new THREE.Color("#36030e"),
          keyRim: new THREE.Color("#00f0ff"),
          counterRim: new THREE.Color("#ffc400"),
          outline: new THREE.Color("#000000"),
          energy: new THREE.Color("#ff184c"),
        };
      case "nominal":
      default:
        return {
          base: new THREE.Color("#00f0ff"),
          shadow: new THREE.Color("#022230"),
          keyRim: new THREE.Color("#ff184c"),
          counterRim: new THREE.Color("#ffc400"),
          outline: new THREE.Color("#000000"),
          energy: new THREE.Color("#00f0ff"),
        };
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, effectiveTheme, themeRev]);

  // Geometries
  const coreGeom = useMemo(() => new THREE.IcosahedronGeometry(0.75, 0), []);
  const middleRingGeom = useMemo(() => new THREE.TorusGeometry(1.15, 0.04, 8, 32), []);
  const outerRingGeom = useMemo(() => new THREE.TorusGeometry(1.45, 0.035, 8, 32), []);
  const energyRingGeom = useMemo(() => new THREE.RingGeometry(1.0, 1.05, 64), []);
  const particleGeom = useMemo(() => new THREE.OctahedronGeometry(0.08, 0), []);

  // Uniforms
  const coreUniforms = useMemo(
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

  const energyUniforms1 = useMemo(
    () => ({
      uGlowColor: { value: colors.energy },
      uCoreColor: { value: colors.base },
      uTime: { value: 0 },
      uIntensity: { value: 1.0 },
    }),
    [colors]
  );

  const energyUniforms2 = useMemo(
    () => ({
      uGlowColor: { value: colors.keyRim },
      uCoreColor: { value: colors.base },
      uTime: { value: 0 },
      uIntensity: { value: 0.7 },
    }),
    [colors]
  );

  const energyUniforms3 = useMemo(
    () => ({
      uGlowColor: { value: colors.counterRim },
      uCoreColor: { value: colors.base },
      uTime: { value: 0 },
      uIntensity: { value: 0.5 },
    }),
    [colors]
  );

  const particleUniforms = useMemo(
    () => ({
      uBaseColor: { value: colors.energy },
      uShadowColor: { value: colors.shadow },
      uHighlightColor: { value: colors.keyRim },
      uRimColor: { value: colors.energy },
      uLightPos: { value: new THREE.Vector3(3, 4, 4) },
      uIsOutline: { value: 0.0 },
      uOutlineOffset: { value: 0.0 },
      uBands: { value: 2.0 },
    }),
    [colors]
  );

  // Animation state
  const [burstPhase, setBurstPhase] = useState(0);

  useFrame((_, delta) => {
    const isBursting = Date.now() - burstTime < 600;
    const baseSpeed = isBursting ? 5.0 : isHovered ? 2.0 : 1.0;

    // Update energy ring time uniforms
    energyUniforms1.uTime.value += delta;
    energyUniforms2.uTime.value += delta * 1.3;
    energyUniforms3.uTime.value += delta * 0.7;

    // Three-axis independent gimbal rotation
    // Inner core: X + Y rotation
    if (innerRef.current) {
      innerRef.current.rotation.x += delta * 0.8 * baseSpeed;
      innerRef.current.rotation.y += delta * 1.2 * baseSpeed;
      innerRef.current.rotation.z += delta * 0.4 * baseSpeed;
    }

    // Middle gimbal: Y + Z rotation (counter-rotating)
    if (middleRef.current) {
      middleRef.current.rotation.y -= delta * 1.0 * baseSpeed;
      middleRef.current.rotation.z += delta * 1.5 * baseSpeed;
      middleRef.current.rotation.x += delta * 0.3 * baseSpeed;
    }

    // Outer gimbal: Z + X rotation (counter-rotating to middle)
    if (outerRef.current) {
      outerRef.current.rotation.z -= delta * 0.7 * baseSpeed;
      outerRef.current.rotation.x -= delta * 1.1 * baseSpeed;
      outerRef.current.rotation.y += delta * 0.5 * baseSpeed;
    }

    // Energy rings pulse and rotate
    if (energyRing1Ref.current) {
      energyRing1Ref.current.rotation.z += delta * 0.8 * baseSpeed;
      const pulse = 0.6 + Math.sin(Date.now() * 0.003) * 0.4;
      energyUniforms1.uIntensity.value = pulse * (isBursting ? 2.0 : 1.0);
    }
    if (energyRing2Ref.current) {
      energyRing2Ref.current.rotation.z -= delta * 1.2 * baseSpeed;
      energyRing2Ref.current.rotation.x += delta * 0.5 * baseSpeed;
      const pulse = 0.4 + Math.sin(Date.now() * 0.0025 + 1) * 0.3;
      energyUniforms2.uIntensity.value = pulse * (isBursting ? 1.5 : 1.0);
    }
    if (energyRing3Ref.current) {
      energyRing3Ref.current.rotation.y += delta * 0.6 * baseSpeed;
      energyRing3Ref.current.rotation.x -= delta * 0.9 * baseSpeed;
      const pulse = 0.3 + Math.sin(Date.now() * 0.002 + 2) * 0.25;
      energyUniforms3.uIntensity.value = pulse * (isBursting ? 1.2 : 1.0);
    }

    // Orbiting particles
    if (particleRefs.current.length) {
      particleRefs.current.forEach((p, i) => {
        if (!p) return;
        const t = Date.now() * 0.001 + i * 2;
        const r = 1.8 + Math.sin(t * 0.7) * 0.3;
        p.position.x = Math.cos(t * (isBursting ? 3 : 1)) * r;
        p.position.y = Math.sin(t * (isBursting ? 2.5 : 0.8) + i) * r * 0.6;
        p.position.z = Math.sin(t * (isBursting ? 2 : 1.2) + i * 1.5) * r;
        p.rotation.x += delta * (isBursting ? 8 : 3);
        p.rotation.y += delta * (isBursting ? 5 : 2);
        // Scale pulse
        const scale = 0.8 + Math.sin(t * 2) * 0.2;
        p.scale.setScalar(scale);
      });
    }

    // Burst flash
    if (isBursting && burstPhase < 1) {
      setBurstPhase((p) => p + delta * 3);
    } else if (!isBursting && burstPhase > 0) {
      setBurstPhase((p) => Math.max(0, p - delta * 2));
    }
  });

  // Build orbiting particles
  const particles = useMemo(() => {
    const arr = [];
    for (let i = 0; i < 6; i++) {
      arr.push(i);
    }
    return arr;
  }, []);

  return (
    <group>
      {/* Outer Gimbal Ring */}
      <group ref={outerRef}>
        <mesh geometry={outerRingGeom}>
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={coreUniforms}
          />
        </mesh>
        <mesh geometry={outerRingGeom}>
          <shaderMaterial
            vertexShader={OUTLINE_VERTEX}
            fragmentShader={OUTLINE_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      {/* Middle Gimbal Ring */}
      <group ref={middleRef}>
        <mesh geometry={middleRingGeom}>
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={coreUniforms}
          />
        </mesh>
        <mesh geometry={middleRingGeom}>
          <shaderMaterial
            vertexShader={OUTLINE_VERTEX}
            fragmentShader={OUTLINE_FRAGMENT}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      </group>

      {/* Inner Core */}
      <group ref={innerRef}>
        <mesh geometry={coreGeom}>
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={coreUniforms}
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

      {/* Energy Rings - additive glow layers */}
      <mesh ref={energyRing1Ref} geometry={energyRingGeom} rotation={[-Math.PI / 2, 0, 0]}>
        <shaderMaterial
          vertexShader={ENERGY_RING_VERTEX}
          fragmentShader={ENERGY_RING_FRAGMENT}
          uniforms={energyUniforms1}
          transparent
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>
      <mesh ref={energyRing2Ref} geometry={energyRingGeom} rotation={[-Math.PI / 2, 0, 0]} scale={[1.3, 1.3, 1]}>
        <shaderMaterial
          vertexShader={ENERGY_RING_VERTEX}
          fragmentShader={ENERGY_RING_FRAGMENT}
          uniforms={energyUniforms2}
          transparent
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>
      <mesh ref={energyRing3Ref} geometry={energyRingGeom} rotation={[-Math.PI / 2, 0, 0]} scale={[1.6, 1.6, 1]}>
        <shaderMaterial
          vertexShader={ENERGY_RING_VERTEX}
          fragmentShader={ENERGY_RING_FRAGMENT}
          uniforms={energyUniforms3}
          transparent
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>

      {/* Orbiting Tactical Particles */}
      {particles.map((i) => (
        <mesh
          key={i}
          ref={(el) => { if (el) particleRefs.current[i] = el; }}
          geometry={particleGeom}
          position={[
            Math.cos(i * 1.05) * 1.8,
            Math.sin(i * 1.05) * 0.5,
            Math.sin(i * 1.05) * 1.8,
          ]}
        >
          <shaderMaterial
            vertexShader={CORE_VERTEX_SHADER}
            fragmentShader={CORE_FRAGMENT_SHADER}
            uniforms={particleUniforms}
          />
        </mesh>
      ))}
    </group>
  );
}

// ============================================================================
// 4. EXPORTABLE TACTICAL CORE COMPONENT (40x40)
// ============================================================================
export interface TacticalCore3DProps {
  status?: "nominal" | "warning" | "overdrive";
  theme?: TacticalCoreTheme;
  size?: number;
  className?: string;
}

export function TacticalCore3D({
  status = "nominal",
  theme,
  size = 40,
  className = "",
}: TacticalCore3DProps) {
  const [mounted, setMounted] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [burstTime, setBurstTime] = useState(0);
  const [isPageVisible, setIsPageVisible] = useState(true);
  const [activeTheme, setActiveTheme] = useState<TacticalCoreTheme>(theme ?? "dark");

  useEffect(() => {
    setMounted(true);

    const checkTheme = () => {
      if (theme) {
        setActiveTheme(theme);
        return;
      }
      const cl = document.documentElement.classList;
      if (cl.contains("physis")) setActiveTheme("physis");
      else if (cl.contains("cel")) setActiveTheme("cel");
      else if (cl.contains("shanghai")) setActiveTheme("shanghai");
      else if (cl.contains("charisma")) setActiveTheme("charisma");
      else if (cl.contains("christmas")) setActiveTheme("christmas");
      else if (cl.contains("light")) setActiveTheme("light");
      else setActiveTheme("dark");
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
  }, [theme]);

  const handleClick = useCallback(() => {
    setBurstTime(Date.now());
  }, []);

  if (!mounted) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`bg-[var(--cel-surface-stage, var(--color-surface))] border border-[var(--cel-ink, var(--color-border))] grid place-items-center ${className}`}
      >
        <span className="size-2 bg-[var(--cel-cyan, var(--accent-DEFAULT))] rotate-45 border border-[var(--cel-ink, var(--color-border))] animate-pulse" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      title="Tactical Neural Core [3渲2 真3D 三轴陀螺仪]"
      aria-label="Tactical Neural Core"
      onClick={handleClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={{ width: size, height: size }}
      className={`relative cursor-pointer select-none group border-2 border-[var(--cel-ink, var(--color-border))] bg-[var(--cel-surface-stage, var(--color-surface))] cel-shadow overflow-hidden transition-transform duration-100 hover:scale-105 active:scale-95 ${className}`}
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
            theme={activeTheme}
          />
        </Canvas>
      )}

      {/* 触碰交互角标 */}
      <div className="absolute bottom-0 right-0 size-1.5 bg-[var(--cel-cyan, var(--accent-DEFAULT))] pointer-events-none z-20" />
    </div>
  );
}
