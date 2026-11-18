/*
@cypher schema=1 source_sha256=90183ebca19dc4caff66e19f56d7000355418a0928ce5c9bdbedc2137563c23c
MERGE (self:File {path:"frontend/src/components/effects/AnimeCelShaderCanvas.tsx"})
MERGE (canvas:Symbol {id:"frontend/src/components/effects/AnimeCelShaderCanvas.tsx::AnimeCelShaderCanvas",kind:"component"})
MERGE (mecha:Symbol {id:"frontend/src/components/effects/AnimeCelShaderCanvas.tsx::MechaSilhouette",kind:"component"})
MERGE (self)-[:DEFINES]->(canvas)
MERGE (self)-[:DEFINES]->(mecha)
@endcypher
*/
"use client";

import React, { useMemo, useRef, useState, useEffect } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import * as THREE from "three";
import { readThemeRgb } from "@/lib/theme/themeTokens";

export type CelPalette =
  | "crimson"
  | "cyan"
  | "gold"
  | "stealth"
  | "manga-vermilion"
  | "manga-cobalt"
  | "manga-gold"
  | "manga-sumi"
  // ── 极魅像素 Charisma HD-2D Palettes ──
  | "charisma-violet"
  | "charisma-crimson"
  | "charisma-gold"
  // ── 圣夜霜雪 Christmas Glassmorphism Palettes ──
  | "christmas-noel"
  | "christmas-pine"
  // ── 老上海月份牌 Shanghai Yuefenpai Palettes ──
  | "shanghai-gilt"
  | "shanghai-jade"
  // ── 日式经典赛璐璐 Japanese Cel Palettes ──
  | "cel-azure"
  | "cel-vermilion"
  | "cel-yellow";

export type ActiveTheme = "dark" | "light" | "charisma" | "christmas" | "shanghai" | "cel" | "physis";

export interface AnimeCelShaderProps {
  palette?: CelPalette;
  outlineThickness?: number;
  bands?: number;
  enableHalftone?: boolean;
  enableRim?: boolean;
  speed?: number;
  className?: string;
  onHoverAction?: () => void;
}

// ============================================================================
// 1. GLSL SHADER DEFINITIONS: Authentic 3D-to-2D (NPR Cel Shading)
// ============================================================================

// ---- Main Surface Cel Shader (Vertex) ----
const CEL_VERTEX_SHADER = /* glsl */ `
  varying vec3 vNormal;
  varying vec3 vViewPosition;
  varying vec3 vWorldPosition;
  varying vec2 vUv;

  void main() {
    vUv = uv;
    vNormal = normalize(normalMatrix * normal);
    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
    vViewPosition = -mvPosition.xyz;
    vWorldPosition = (modelMatrix * vec4(position, 1.0)).xyz;
    gl_Position = projectionMatrix * mvPosition;
  }
`;

// ---- Main Surface Cel Shader (Fragment) ----
// Features: 2/3-Band Step Ramp, Hard Specular Glint, Fresnel Rim, Ben-Day Halftone Screentone
const CEL_FRAGMENT_SHADER = /* glsl */ `
  uniform vec3 uBaseColor;
  uniform vec3 uShadowColor;
  uniform vec3 uHighlightColor;
  uniform vec3 uRimColor;
  uniform vec3 uCounterRimColor;
  uniform vec3 uLightPos;
  uniform float uBands;
  uniform float uHalftoneScale;
  uniform float uHalftoneIntensity;
  uniform float uEnableHalftone;
  uniform float uEnableRim;
  uniform float uTime;

  varying vec3 vNormal;
  varying vec3 vViewPosition;
  varying vec3 vWorldPosition;
  varying vec2 vUv;

  void main() {
    vec3 N = normalize(vNormal);
    vec3 L = normalize(uLightPos - vWorldPosition);
    vec3 V = normalize(vViewPosition);
    vec3 H = normalize(L + V);

    // 1. Half-Lambert Diffuse Calculation (Anime Lighting Basis)
    float NdotL = dot(N, L);
    float halfLambert = NdotL * 0.5 + 0.5;

    // 2. Stepped Tone Quantization (Discrete Cel Bands, Zero Diffuse Blur)
    float celFactor = 0.2;
    if (uBands < 2.5) {
      celFactor = step(0.48, halfLambert) * 0.8 + 0.2;
    } else if (uBands < 3.5) {
      float step1 = smoothstep(0.33, 0.36, halfLambert);
      float step2 = smoothstep(0.66, 0.69, halfLambert);
      celFactor = 0.2 + 0.4 * step1 + 0.4 * step2;
    } else {
      float step1 = smoothstep(0.25, 0.27, halfLambert);
      float step2 = smoothstep(0.50, 0.52, halfLambert);
      float step3 = smoothstep(0.75, 0.77, halfLambert);
      celFactor = 0.15 + 0.28 * step1 + 0.28 * step2 + 0.29 * step3;
    }

    // 3. Base Color Stepped Interpolation
    vec3 color = mix(uShadowColor, uBaseColor, celFactor);

    // 4. Manga Halftone Screentone in Shadow Region
    if (uEnableHalftone > 0.5) {
      vec2 screenCoord = gl_FragCoord.xy * uHalftoneScale;
      float dotDist = length(fract(screenCoord) - vec2(0.5));
      float dotMask = step(0.32, dotDist);
      float shadowMask = 1.0 - smoothstep(0.35, 0.55, halfLambert);
      color = mix(color, uShadowColor * 0.5, (1.0 - dotMask) * shadowMask * uHalftoneIntensity);
    }

    // 5. Hard-Edged Anime Stepped Specular Glint
    float NdotH = max(0.0, dot(N, H));
    float spec = step(0.93, pow(NdotH, 36.0));
    color += spec * uHighlightColor * 0.95;

    // 6. Anime Dual-Rim Lighting (Key Rim + Counter Rim Chroma Shift)
    if (uEnableRim > 0.5) {
      float fresnel = 1.0 - max(0.0, dot(N, V));
      float keyRim = smoothstep(0.68, 0.74, fresnel) * smoothstep(0.05, 0.3, halfLambert);
      float counterRim = smoothstep(0.76, 0.82, fresnel) * (1.0 - smoothstep(0.05, 0.3, halfLambert));
      color += keyRim * uRimColor * 1.3;
      color += counterRim * uCounterRimColor * 1.05;
    }

    gl_FragColor = vec4(color, 1.0);
  }
`;

// ---- Inverted Hull Outline Shader (Vertex) ----
const OUTLINE_VERTEX_SHADER = /* glsl */ `
  uniform float uOutlineThickness;

  void main() {
    vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
    vec3 viewNormal = normalize(normalMatrix * normal);
    mvPosition.xyz += viewNormal * uOutlineThickness;
    gl_Position = projectionMatrix * mvPosition;
  }
`;

// ---- Inverted Hull Outline Shader (Fragment) ----
const OUTLINE_FRAGMENT_SHADER = /* glsl */ `
  uniform vec3 uOutlineColor;

  void main() {
    gl_FragColor = vec4(uOutlineColor, 1.0);
  }
`;

// ============================================================================
// 2. COLOR PALETTES FOR CEL SHADER (Dark & Light Independent Masterpieces)
// ============================================================================
const PALETTE_CONFIGS: Record<
  CelPalette,
  {
    base: string;
    shadow: string;
    highlight: string;
    rim: string;
    counterRim: string;
    outline: string;
  }
> = {
  // ── Dark Masterpiece Palettes (Neo-Tokyo Dark) ──
  crimson: {
    base: "#ff184c",
    shadow: "#3d0513",
    highlight: "#ffffff",
    rim: "#00f0ff",
    counterRim: "#ffc400",
    outline: "#000000",
  },
  cyan: {
    base: "#00f0ff",
    shadow: "#032838",
    highlight: "#ffffff",
    rim: "#ff184c",
    counterRim: "#ffc400",
    outline: "#000000",
  },
  gold: {
    base: "#ffc400",
    shadow: "#382902",
    highlight: "#ffffff",
    rim: "#00f0ff",
    counterRim: "#ff184c",
    outline: "#000000",
  },
  stealth: {
    base: "#2a344a",
    shadow: "#090d16",
    highlight: "#8ca8db",
    rim: "#00f0ff",
    counterRim: "#ff184c",
    outline: "#000000",
  },

  // ── Light Masterpiece Palettes (Shōnen Studio Archival Paper) ──
  "manga-vermilion": {
    base: "#e60033",
    shadow: "#2a040b",
    highlight: "#ffffff",
    rim: "#0055ff",
    counterRim: "#d48800",
    outline: "#0a0d14",
  },
  "manga-cobalt": {
    base: "#0055ff",
    shadow: "#05102e",
    highlight: "#ffffff",
    rim: "#e60033",
    counterRim: "#0055ff",
    outline: "#0a0d14",
  },
  "manga-gold": {
    base: "#d48800",
    shadow: "#2b1b00",
    highlight: "#ffffff",
    rim: "#0055ff",
    counterRim: "#e60033",
    outline: "#0a0d14",
  },
  "manga-sumi": {
    base: "#222836",
    shadow: "#06080d",
    highlight: "#ffffff",
    rim: "#e60033",
    counterRim: "#0055ff",
    outline: "#0a0d14",
  },

  // ── 极魅像素 Charisma HD-2D (歧路旅人 体素斜面挤压) ──
  "charisma-violet": {
    base: "#8b5cf6",
    shadow: "#140d24",
    highlight: "#f5f3ff",
    rim: "#e11d48",
    counterRim: "#f59e0b",
    outline: "#050409",
  },
  "charisma-crimson": {
    base: "#e11d48",
    shadow: "#28040b",
    highlight: "#fff1f2",
    rim: "#8b5cf6",
    counterRim: "#f59e0b",
    outline: "#050409",
  },
  "charisma-gold": {
    base: "#f59e0b",
    shadow: "#241602",
    highlight: "#fffbeb",
    rim: "#8b5cf6",
    counterRim: "#e11d48",
    outline: "#050409",
  },

  // ── 圣夜霜雪 Christmas Glassmorphism (伯利恒烛金 + 常青松针) ──
  "christmas-noel": {
    base: "#d42426",
    shadow: "#140405",
    highlight: "#fff5f5",
    rim: "#f8b229",
    counterRim: "#165b33",
    outline: "#060b08",
  },
  "christmas-pine": {
    base: "#165b33",
    shadow: "#05170b",
    highlight: "#f0fdf4",
    rim: "#f8b229",
    counterRim: "#d42426",
    outline: "#060b08",
  },

  // ── 老上海月份牌 Shanghai Yuefenpai (擦笔水彩鎏金 + Art Deco) ──
  "shanghai-gilt": {
    base: "#d49e35",
    shadow: "#2b170c",
    highlight: "#fffbf0",
    rim: "#b8282b",
    counterRim: "#1b5e48",
    outline: "#221c16",
  },
  "shanghai-jade": {
    base: "#1b5e48",
    shadow: "#05150f",
    highlight: "#f0fff9",
    rim: "#d49e35",
    counterRim: "#b8282b",
    outline: "#221c16",
  },

  // ── 日式经典赛璐璐 (80-90s 经典日漫手绘平涂与胶片高光) ──
  "cel-azure": {
    base: "#2563eb",
    shadow: "#0f172a",
    highlight: "#ffffff",
    rim: "#ea580c",
    counterRim: "#f59e0b",
    outline: "#111827",
  },
  "cel-vermilion": {
    base: "#ea580c",
    shadow: "#270802",
    highlight: "#ffffff",
    rim: "#f59e0b",
    counterRim: "#2563eb",
    outline: "#111827",
  },
  "cel-yellow": {
    base: "#f59e0b",
    shadow: "#291500",
    highlight: "#ffffff",
    rim: "#ea580c",
    counterRim: "#2563eb",
    outline: "#111827",
  },
};

// ============================================================================
// 2b. 主题 token 覆盖层 (per 2026-11-17 主题统一)
// ============================================================================
// PALETTE_CONFIGS 保留各主题的设计原色 (dark 系完全不变);
// 非 dark 主题的 palette 在运行时用当前 html.<theme> 的 --cel-* token
// 覆盖 base/rim/counterRim/outline, 使 3D 内容与 CSS 主题严格同源。
// shadow 按 base×0.16 重算 (赛璐璐暗阶), highlight 取 --cel-text-primary。

const TOKEN_OVERRIDDEN_THEMES: ReadonlySet<ActiveTheme> = new Set([
  "christmas", "charisma", "shanghai", "cel", "physis",
]);

function hexFromRgb(rgb: [number, number, number]): string {
  const c = (v: number) => Math.round(Math.min(1, Math.max(0, v)) * 255).toString(16).padStart(2, "0");
  return `#${c(rgb[0])}${c(rgb[1])}${c(rgb[2])}`;
}

/** 读取当前主题 token 并覆盖 palette 的 base/rim/counterRim/outline/shadow/highlight */
function resolvePaletteWithTokens(palette: CelPalette, theme: ActiveTheme): typeof PALETTE_CONFIGS[CelPalette] {
  const base0 = PALETTE_CONFIGS[palette];
  if (!TOKEN_OVERRIDDEN_THEMES.has(theme)) return base0;
  const baseRgb = readThemeRgb("--cel-cyan", [0, 0.941, 1]);
  // base 语义: crimson 系 palette 用 --cel-crimson, gold 系用 --cel-gold, 其余 --cel-cyan
  const baseToken = /crimson|vermilion|noel/.test(palette) ? "--cel-crimson"
    : /gold|gilt|yellow/.test(palette) ? "--cel-gold"
    : "--cel-cyan";
  const base = readThemeRgb(baseToken, baseRgb);
  const rim = readThemeRgb("--cel-crimson", [1, 0.094, 0.298]);
  const counterRim = readThemeRgb("--cel-gold", [1, 0.769, 0]);
  const outline = readThemeRgb("--cel-ink", [0.04, 0.05, 0.08]);
  const highlight = readThemeRgb("--cel-text-primary", [1, 1, 1]);
  return {
    base: hexFromRgb(base),
    shadow: hexFromRgb([base[0] * 0.16, base[1] * 0.16, base[2] * 0.16]),
    highlight: hexFromRgb(highlight),
    rim: hexFromRgb(rim),
    counterRim: hexFromRgb(counterRim),
    outline: hexFromRgb(outline),
  };
}

/** 监听 html class 变化 (next-themes 切主题), 返回递增版本号触发重渲染 */
function useThemeRevision(): number {
  const [rev, setRev] = useState(0);
  useEffect(() => {
    const el = document.documentElement;
    const obs = new MutationObserver(() => setRev((r) => r + 1));
    obs.observe(el, { attributes: true, attributeFilter: ["class"] });
    return () => obs.disconnect();
  }, []);
  return rev;
}

// ============================================================================
// 3. CEL-SHADED MESH WITH INVERTED HULL OUTLINE
// ============================================================================
interface CelMeshProps {
  geometry: THREE.BufferGeometry;
  palette: CelPalette;
  /** 已按当前主题 token 解析的色板 (缺省回退 PALETTE_CONFIGS) */
  resolvedPalette?: typeof PALETTE_CONFIGS[CelPalette];
  outlineThickness: number;
  bands: number;
  enableHalftone: boolean;
  enableRim: boolean;
  lightPos: THREE.Vector3;
}

function CelShadedMesh({
  geometry,
  palette,
  resolvedPalette,
  outlineThickness,
  bands,
  enableHalftone,
  enableRim,
  lightPos,
}: CelMeshProps) {
  const p = resolvedPalette ?? PALETTE_CONFIGS[palette];

  // Surface Uniforms
  const surfaceUniforms = useMemo(
    () => ({
      uBaseColor: { value: new THREE.Color(p.base) },
      uShadowColor: { value: new THREE.Color(p.shadow) },
      uHighlightColor: { value: new THREE.Color(p.highlight) },
      uRimColor: { value: new THREE.Color(p.rim) },
      uCounterRimColor: { value: new THREE.Color(p.counterRim) },
      uLightPos: { value: lightPos },
      uBands: { value: bands },
      uHalftoneScale: { value: 0.15 },
      uHalftoneIntensity: { value: 0.6 },
      uEnableHalftone: { value: enableHalftone ? 1.0 : 0.0 },
      uEnableRim: { value: enableRim ? 1.0 : 0.0 },
      uTime: { value: 0 },
    }),
    [p, bands, enableHalftone, enableRim, lightPos]
  );

  // Outline Uniforms
  const outlineUniforms = useMemo(
    () => ({
      uOutlineThickness: { value: outlineThickness },
      uOutlineColor: { value: new THREE.Color(p.outline) },
    }),
    [outlineThickness, p.outline]
  );

  useFrame((_, delta) => {
    surfaceUniforms.uTime.value += delta;
    surfaceUniforms.uLightPos.value.copy(lightPos);
  });

  return (
    <group>
      {/* 1. Main Cel-Shaded Surface (Front Facing) */}
      <mesh geometry={geometry}>
        <shaderMaterial
          vertexShader={CEL_VERTEX_SHADER}
          fragmentShader={CEL_FRAGMENT_SHADER}
          uniforms={surfaceUniforms}
        />
      </mesh>

      {/* 2. Inverted Hull Comic Ink Outline (Back Facing, Extruded along Normal) */}
      {outlineThickness > 0 && (
        <mesh geometry={geometry}>
          <shaderMaterial
            vertexShader={OUTLINE_VERTEX_SHADER}
            fragmentShader={OUTLINE_FRAGMENT_SHADER}
            uniforms={outlineUniforms}
            side={THREE.BackSide}
          />
        </mesh>
      )}
    </group>
  );
}

// ============================================================================
// 4. MECHA SILHOUETTE — 真3D 角色/机甲剪影
// ============================================================================
interface MechaSilhouetteProps {
  palette: CelPalette;
  activeTheme: ActiveTheme;
  outlineThickness: number;
  bands: number;
  enableHalftone: boolean;
  enableRim: boolean;
  speed: number;
}

function MechaSilhouette({
  palette,
  activeTheme,
  outlineThickness,
  bands,
  enableHalftone,
  enableRim,
  speed,
}: MechaSilhouetteProps) {
  const groupRef = useRef<THREE.Group>(null);
  const torsoRef = useRef<THREE.Group>(null);
  const headRef = useRef<THREE.Group>(null);
  const lArmRef = useRef<THREE.Group>(null);
  const rArmRef = useRef<THREE.Group>(null);
  const lLegRef = useRef<THREE.Group>(null);
  const rLegRef = useRef<THREE.Group>(null);
  const wing1Ref = useRef<THREE.Group>(null);
  const wing2Ref = useRef<THREE.Group>(null);
  const coreRef = useRef<THREE.Mesh>(null);
  const particleRefs = useRef<THREE.Mesh[]>([]);

  const { pointer } = useThree();
  const lightPos = useMemo(() => new THREE.Vector3(5, 5, 5), []);

  // per 2026-11-17 主题统一: 按当前主题 token 解析全部子 mesh 色板;
  // html class 变化 (切主题) → rev+1 → 重新解析 → uniforms 重建
  const themeRev = useThemeRevision();
  const ringPalette: CelPalette = palette === "crimson" ? "gold" : "crimson";
  const ring2Palette: CelPalette = palette === "cyan" ? "crimson" : "cyan";
  const resolved = useMemo(
    () => ({
      main: resolvePaletteWithTokens(palette, activeTheme),
      accent: resolvePaletteWithTokens(ringPalette, activeTheme),
      trim: resolvePaletteWithTokens(ring2Palette, activeTheme),
      gold: resolvePaletteWithTokens("gold", activeTheme),
      cyan: resolvePaletteWithTokens("cyan", activeTheme),
    }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [palette, ringPalette, ring2Palette, activeTheme, themeRev]
  );

  // Mecha geometries - humanoid silhouette
  const torsoGeom = useMemo(() => new THREE.BoxGeometry(1.2, 1.6, 0.6, 2, 2, 1), []);
  const headGeom = useMemo(() => new THREE.BoxGeometry(0.7, 0.7, 0.5, 2, 2, 1), []);
  const armGeom = useMemo(() => new THREE.BoxGeometry(0.35, 1.2, 0.35, 1, 2, 1), []);
  const forearmGeom = useMemo(() => new THREE.BoxGeometry(0.4, 0.9, 0.4, 1, 2, 1), []);
  const legGeom = useMemo(() => new THREE.BoxGeometry(0.4, 1.0, 0.4, 1, 2, 1), []);
  const shinGeom = useMemo(() => new THREE.BoxGeometry(0.35, 1.1, 0.35, 1, 2, 1), []);
  const footGeom = useMemo(() => new THREE.BoxGeometry(0.5, 0.2, 0.6, 1, 1, 1), []);
  const shoulderGeom = useMemo(() => new THREE.BoxGeometry(0.5, 0.35, 0.4, 1, 1, 1), []);
  const wingGeom = useMemo(() => new THREE.BoxGeometry(1.8, 1.2, 0.08, 2, 2, 1), []);
  const coreGeom = useMemo(() => new THREE.OctahedronGeometry(0.35, 0), []);
  const particleGeom = useMemo(() => new THREE.TetrahedronGeometry(0.08, 0), []);

  // Joint positions for articulation
  const shoulderOffset = 0.85;
  const hipOffset = 0.55;

  useFrame((_, delta) => {
    // Dynamic light reacts to mouse pointer
    lightPos.set(pointer.x * 8 + 2, pointer.y * 8 + 3, 5);

    const s = delta * speed;
    const t = Date.now() * 0.001;

    // Breathing idle animation
    const breathe = Math.sin(t * 1.2) * 0.02;
    const sway = Math.sin(t * 0.7) * 0.05;

    if (groupRef.current) {
      groupRef.current.rotation.y += s * 0.15;
      groupRef.current.position.y = breathe;
      groupRef.current.rotation.z = sway;
    }

    // Torso subtle twist
    if (torsoRef.current) {
      torsoRef.current.rotation.y = Math.sin(t * 0.8) * 0.1;
      torsoRef.current.rotation.x = Math.cos(t * 0.6) * 0.05;
    }

    // Head tracking (subtle)
    if (headRef.current) {
      headRef.current.rotation.y = pointer.x * 0.3;
      headRef.current.rotation.x = -pointer.y * 0.2;
    }

    // Arm swing (counter-motion)
    if (lArmRef.current) {
      lArmRef.current.rotation.x = Math.sin(t * 1.5) * 0.4 - 0.3;
      lArmRef.current.rotation.z = Math.cos(t * 1.2) * 0.15;
    }
    if (rArmRef.current) {
      rArmRef.current.rotation.x = -Math.sin(t * 1.5) * 0.4 + 0.3;
      rArmRef.current.rotation.z = -Math.cos(t * 1.2) * 0.15;
    }

    // Leg stance shift
    if (lLegRef.current) {
      lLegRef.current.rotation.x = Math.sin(t * 1.5 + Math.PI) * 0.15;
    }
    if (rLegRef.current) {
      rLegRef.current.rotation.x = -Math.sin(t * 1.5 + Math.PI) * 0.15;
    }

    // Wings flutter
    if (wing1Ref.current) {
      wing1Ref.current.rotation.x = Math.sin(t * 2.0) * 0.2;
      wing1Ref.current.rotation.z = Math.cos(t * 1.3) * 0.1;
      wing1Ref.current.position.y = 0.8 + Math.sin(t * 1.8) * 0.05;
    }
    if (wing2Ref.current) {
      wing2Ref.current.rotation.x = -Math.sin(t * 2.0) * 0.2;
      wing2Ref.current.rotation.z = -Math.cos(t * 1.3) * 0.1;
      wing2Ref.current.position.y = 0.8 + Math.sin(t * 1.8 + Math.PI) * 0.05;
    }

    // Core energy pulse
    if (coreRef.current) {
      const pulse = 0.7 + Math.sin(t * 3.0) * 0.3;
      coreRef.current.scale.setScalar(pulse);
      coreRef.current.rotation.x += delta * 2.0;
      coreRef.current.rotation.y += delta * 1.5;
      coreRef.current.rotation.z += delta * 1.0;
    }

    // Orbiting particles
    if (particleRefs.current.length) {
      particleRefs.current.forEach((p, i) => {
        if (!p) return;
        const pt = t * 0.8 + i * 1.2;
        const r = 2.2 + Math.sin(pt * 0.9) * 0.4;
        p.position.x = Math.cos(pt) * r;
        p.position.y = Math.sin(pt * 0.7) * 1.5 + Math.cos(pt * 0.5) * 0.5;
        p.position.z = Math.sin(pt) * r;
        p.rotation.x += delta * 3.0;
        p.rotation.y += delta * 2.0;
        const scale = 0.7 + Math.sin(pt * 2.5) * 0.3;
        p.scale.setScalar(scale);
      });
    }
  });

  return (
    <group ref={groupRef} position={[0, -0.5, 0]}>
      {/* Backpack / Wings */}
      <group ref={wing1Ref} position={[-1.0, 0.8, 0.2]} rotation={[0, -0.3, 0]}>
        <CelShadedMesh
          geometry={wingGeom}
          palette={palette}
          resolvedPalette={resolved.trim}
          outlineThickness={outlineThickness * 0.6}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
      </group>
      <group ref={wing2Ref} position={[1.0, 0.8, 0.2]} rotation={[0, 0.3, 0]}>
        <CelShadedMesh
          geometry={wingGeom}
          palette={palette}
          resolvedPalette={resolved.trim}
          outlineThickness={outlineThickness * 0.6}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
      </group>

      {/* Left Leg */}
      <group ref={lLegRef} position={[-hipOffset, -2.0, 0]}>
        <CelShadedMesh
          geometry={legGeom}
          palette={palette}
          resolvedPalette={resolved.main}
          outlineThickness={outlineThickness}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
        <group position={[0, -1.05, 0]}>
          <CelShadedMesh
            geometry={shinGeom}
            palette={palette}
            resolvedPalette={resolved.accent}
            outlineThickness={outlineThickness * 0.9}
            bands={bands}
            enableHalftone={enableHalftone}
            enableRim={enableRim}
            lightPos={lightPos}
          />
          <group position={[0, -0.65, 0.15]}>
            <CelShadedMesh
              geometry={footGeom}
              palette={palette}
              resolvedPalette={resolved.trim}
              outlineThickness={outlineThickness * 0.8}
              bands={bands}
              enableHalftone={enableHalftone}
              enableRim={enableRim}
              lightPos={lightPos}
            />
          </group>
        </group>
      </group>

      {/* Right Leg */}
      <group ref={rLegRef} position={[hipOffset, -2.0, 0]}>
        <CelShadedMesh
          geometry={legGeom}
          palette={palette}
          resolvedPalette={resolved.main}
          outlineThickness={outlineThickness}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
        <group position={[0, -1.05, 0]}>
          <CelShadedMesh
            geometry={shinGeom}
            palette={palette}
            resolvedPalette={resolved.accent}
            outlineThickness={outlineThickness * 0.9}
            bands={bands}
            enableHalftone={enableHalftone}
            enableRim={enableRim}
            lightPos={lightPos}
          />
          <group position={[0, -0.65, 0.15]}>
            <CelShadedMesh
              geometry={footGeom}
              palette={palette}
              resolvedPalette={resolved.trim}
              outlineThickness={outlineThickness * 0.8}
              bands={bands}
              enableHalftone={enableHalftone}
              enableRim={enableRim}
              lightPos={lightPos}
            />
          </group>
        </group>
      </group>

      {/* Torso */}
      <group ref={torsoRef} position={[0, 0.2, 0]}>
        <CelShadedMesh
          geometry={torsoGeom}
          palette={palette}
          resolvedPalette={resolved.main}
          outlineThickness={outlineThickness}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />

        {/* Core reactor in chest */}
        <mesh ref={coreRef} geometry={coreGeom} position={[0, 0.2, 0.5]}>
          <shaderMaterial
            vertexShader={CEL_VERTEX_SHADER}
            fragmentShader={CEL_FRAGMENT_SHADER}
            uniforms={{
              uBaseColor: { value: new THREE.Color(resolved.main.base) },
              uShadowColor: { value: new THREE.Color(resolved.main.shadow) },
              uHighlightColor: { value: new THREE.Color(resolved.main.highlight) },
              uRimColor: { value: new THREE.Color(resolved.main.rim) },
              uCounterRimColor: { value: new THREE.Color(resolved.main.counterRim) },
              uLightPos: { value: lightPos },
              uBands: { value: bands },
              uHalftoneScale: { value: 0.15 },
              uHalftoneIntensity: { value: 0.6 },
              uEnableHalftone: { value: enableHalftone ? 1.0 : 0.0 },
              uEnableRim: { value: enableRim ? 1.0 : 0.0 },
              uTime: { value: 0 },
            }}
          />
        </mesh>

        {/* Left Shoulder */}
        <group ref={lArmRef} position={[-shoulderOffset, 0.6, 0]}>
          <CelShadedMesh
            geometry={shoulderGeom}
            palette={palette}
            resolvedPalette={resolved.trim}
            outlineThickness={outlineThickness * 0.8}
            bands={bands}
            enableHalftone={enableHalftone}
            enableRim={enableRim}
            lightPos={lightPos}
          />
          <group position={[0, -0.7, 0]} rotation={[0.2, 0, 0]}>
            <CelShadedMesh
              geometry={armGeom}
              palette={palette}
              resolvedPalette={resolved.main}
              outlineThickness={outlineThickness}
              bands={bands}
              enableHalftone={enableHalftone}
              enableRim={enableRim}
              lightPos={lightPos}
            />
            <group position={[0, -1.05, 0]}>
              <CelShadedMesh
                geometry={forearmGeom}
                palette={palette}
                resolvedPalette={resolved.accent}
                outlineThickness={outlineThickness * 0.9}
                bands={bands}
                enableHalftone={enableHalftone}
                enableRim={enableRim}
                lightPos={lightPos}
              />
            </group>
          </group>
        </group>

        {/* Right Shoulder */}
        <group ref={rArmRef} position={[shoulderOffset, 0.6, 0]}>
          <CelShadedMesh
            geometry={shoulderGeom}
            palette={palette}
            resolvedPalette={resolved.trim}
            outlineThickness={outlineThickness * 0.8}
            bands={bands}
            enableHalftone={enableHalftone}
            enableRim={enableRim}
            lightPos={lightPos}
          />
          <group position={[0, -0.7, 0]} rotation={[-0.2, 0, 0]}>
            <CelShadedMesh
              geometry={armGeom}
              palette={palette}
              resolvedPalette={resolved.main}
              outlineThickness={outlineThickness}
              bands={bands}
              enableHalftone={enableHalftone}
              enableRim={enableRim}
              lightPos={lightPos}
            />
            <group position={[0, -1.05, 0]}>
              <CelShadedMesh
                geometry={forearmGeom}
                palette={palette}
                resolvedPalette={resolved.accent}
                outlineThickness={outlineThickness * 0.9}
                bands={bands}
                enableHalftone={enableHalftone}
                enableRim={enableRim}
                lightPos={lightPos}
              />
            </group>
          </group>
        </group>

        {/* Head */}
        <group ref={headRef} position={[0, 1.35, 0]}>
          <CelShadedMesh
            geometry={headGeom}
            palette={palette}
            resolvedPalette={resolved.main}
            outlineThickness={outlineThickness}
            bands={bands}
            enableHalftone={enableHalftone}
            enableRim={enableRim}
            lightPos={lightPos}
          />
          {/* Visor */}
          <mesh position={[0, 0, 0.3]} scale={[0.5, 0.25, 0.1]}>
            <meshBasicMaterial
              color={new THREE.Color(resolved.accent.base)}
              transparent
              opacity={0.9}
            />
          </mesh>
        </group>
      </group>

      {/* Orbiting Tactical Particles */}
      {[0, 1, 2, 3, 4, 5].map((i) => (
        <mesh
          key={i}
          ref={(el) => { if (el) particleRefs.current[i] = el; }}
          geometry={particleGeom}
          position={[
            Math.cos(i * 1.05) * 2.2,
            Math.sin(i * 1.05) * 0.5,
            Math.sin(i * 1.05) * 2.2,
          ]}
        >
          <CelShadedMesh
            geometry={particleGeom}
            palette={i % 2 === 0 ? "gold" : "cyan"}
            resolvedPalette={i % 2 === 0 ? resolved.gold : resolved.cyan}
            outlineThickness={outlineThickness * 0.7}
            bands={bands}
            enableHalftone={false}
            enableRim={true}
            lightPos={lightPos}
          />
        </mesh>
      ))}
    </group>
  );
}

// ============================================================================
// 5. EXPORTABLE CONTROLLER CANVAS (DUAL-THEME ADAPTIVE & VIEWPORT GUARDED)
// ============================================================================
export function AnimeCelShaderCanvas({
  palette = "crimson",
  outlineThickness = 0.045,
  bands = 3,
  enableHalftone = true,
  enableRim = true,
  speed = 1.0,
  className = "",
}: AnimeCelShaderProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isClient, setIsClient] = useState(false);
  const [inView, setInView] = useState(true);
  const [tabVisible, setTabVisible] = useState(true);
  const [activeTheme, setActiveTheme] = useState<ActiveTheme>("dark");

  useEffect(() => {
    setIsClient(true);

    const checkTheme = () => {
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

    let themeObserver: MutationObserver | null = null;
    if (typeof MutationObserver !== "undefined") {
      themeObserver = new MutationObserver(checkTheme);
      themeObserver.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["class"],
      });
    }

    const handleVisibilityChange = () => {
      setTabVisible(!document.hidden);
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    let observer: IntersectionObserver | null = null;
    if (containerRef.current && typeof IntersectionObserver !== "undefined") {
      observer = new IntersectionObserver(
        ([entry]) => {
          setInView(entry.isIntersecting);
        },
        { threshold: 0.05 }
      );
      observer.observe(containerRef.current);
    }

    return () => {
      themeObserver?.disconnect();
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      observer?.disconnect();
    };
  }, []);

  // ── Automatic 6-Theme Masterpiece Palette Mapping ──
  const activePalette = useMemo<CelPalette>(() => {
    switch (activeTheme) {
      case "cel":
        // 日式经典手绘赛璐璐: 纯正平涂双阶色块阴影与机械墨线 (EVA蓝 / 活力朱红 / 向日葵黄)
        switch (palette) {
          case "crimson": return "cel-vermilion";
          case "gold":    return "cel-yellow";
          default:        return "cel-azure";
        }
      case "light":
        // Shōnen 少年原画: 宣纸白线条美学 (manga-* series)
        switch (palette) {
          case "crimson": return "manga-vermilion";
          case "cyan":    return "manga-cobalt";
          case "gold":    return "manga-gold";
          case "stealth": return "manga-sumi";
          default:        return palette;
        }
      case "charisma":
        // 极魅像素 HD-2D: 神恩紫罗兰 × 醉梦丝绒红 × 帝国流金
        switch (palette) {
          case "crimson": return "charisma-crimson";
          case "gold":    return "charisma-gold";
          default:        return "charisma-violet";
        }
      case "christmas":
        // 圣夜霜雪 Glassmorphism: 圣夜绯红 × 常青松针
        switch (palette) {
          case "stealth":
          case "gold":    return "christmas-pine";
          default:        return "christmas-noel";
        }
      case "shanghai":
        // 老上海月份牌: 留声机鎏金 × 翡翠碧玉墨绿
        switch (palette) {
          case "stealth":
          case "cyan":    return "shanghai-jade";
          default:        return "shanghai-gilt";
        }
      case "physis":
        // 3D 物理主题: 深空紫蓝 — 走 dark 系 palette 再由 --cel-* token 覆盖
        return palette;
      case "dark":
      default:
        // 暗夜神格 Neo-Tokyo Cyber Manga: 原始赛博 palette
        return palette;
    }
  }, [palette, activeTheme]);

  const shouldRender = isClient && inView && tabVisible;

  if (!isClient) {
    return (
      <div className={`w-full h-full bg-[var(--cel-surface-stage, var(--color-surface))] flex items-center justify-center ${className}`}>
        <span className="text-xs font-mono font-bold text-[var(--cel-cyan, var(--accent-DEFAULT))] animate-pulse">
          INITIALIZING_3D_CEL_PIPELINE...
        </span>
      </div>
    );
  }

  return (
    <div ref={containerRef} className={`relative w-full h-full overflow-hidden ${className}`}>
      <Canvas
        camera={{ position: [0, 0, 6.2], fov: 45 }}
        gl={{ antialias: false, alpha: true, powerPreference: "low-power" }}
        frameloop={shouldRender ? "always" : "demand"}
        className="w-full h-full"
      >
        <MechaSilhouette
          palette={activePalette}
          activeTheme={activeTheme}
          outlineThickness={outlineThickness}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          speed={speed}
        />
      </Canvas>
    </div>
  );
}
