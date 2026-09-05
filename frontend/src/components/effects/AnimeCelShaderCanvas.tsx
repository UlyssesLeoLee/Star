"use client";

import React, { useMemo, useRef, useState, useEffect } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import * as THREE from "three";

export type CelPalette =
  | "crimson"
  | "cyan"
  | "gold"
  | "stealth"
  | "manga-vermilion"
  | "manga-cobalt"
  | "manga-gold"
  | "manga-sumi";

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

    // 5. Hard-Edged Anime Specular Highlight
    float NdotH = max(0.0, dot(N, H));
    float spec = smoothstep(0.92, 0.95, pow(NdotH, 32.0));
    color += spec * uHighlightColor * 0.85;

    // 6. Anime Fresnel Rim Light (Edge Sheen)
    if (uEnableRim > 0.5) {
      float fresnel = 1.0 - max(0.0, dot(N, V));
      float rim = smoothstep(0.68, 0.74, fresnel) * smoothstep(0.05, 0.3, halfLambert);
      color += rim * uRimColor * 1.2;
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
    outline: string;
  }
> = {
  // ── Dark Masterpiece Palettes (Neo-Tokyo Dark) ──
  crimson: {
    base: "#ff184c",
    shadow: "#3d0513",
    highlight: "#ffffff",
    rim: "#00f0ff",
    outline: "#000000",
  },
  cyan: {
    base: "#00f0ff",
    shadow: "#032838",
    highlight: "#ffffff",
    rim: "#ff184c",
    outline: "#000000",
  },
  gold: {
    base: "#ffc400",
    shadow: "#382902",
    highlight: "#ffffff",
    rim: "#00f0ff",
    outline: "#000000",
  },
  stealth: {
    base: "#2a344a",
    shadow: "#090d16",
    highlight: "#8ca8db",
    rim: "#00f0ff",
    outline: "#000000",
  },

  // ── Light Masterpiece Palettes (Shōnen Studio Archival Paper) ──
  "manga-vermilion": {
    base: "#e60033",
    shadow: "#2a040b",
    highlight: "#ffffff",
    rim: "#0055ff",
    outline: "#0a0d14",
  },
  "manga-cobalt": {
    base: "#0055ff",
    shadow: "#05102e",
    highlight: "#ffffff",
    rim: "#e60033",
    outline: "#0a0d14",
  },
  "manga-gold": {
    base: "#d48800",
    shadow: "#2b1b00",
    highlight: "#ffffff",
    rim: "#0055ff",
    outline: "#0a0d14",
  },
  "manga-sumi": {
    base: "#222836",
    shadow: "#06080d",
    highlight: "#ffffff",
    rim: "#e60033",
    outline: "#0a0d14",
  },
};

// ============================================================================
// 3. 3D CEL-SHADED MESH RIG WITH INVERTED HULL OUTLINE
// ============================================================================
interface CelMeshProps {
  geometry: THREE.BufferGeometry;
  palette: CelPalette;
  outlineThickness: number;
  bands: number;
  enableHalftone: boolean;
  enableRim: boolean;
  lightPos: THREE.Vector3;
}

function CelShadedMesh({
  geometry,
  palette,
  outlineThickness,
  bands,
  enableHalftone,
  enableRim,
  lightPos,
}: CelMeshProps) {
  const p = PALETTE_CONFIGS[palette];

  // Surface Uniforms
  const surfaceUniforms = useMemo(
    () => ({
      uBaseColor: { value: new THREE.Color(p.base) },
      uShadowColor: { value: new THREE.Color(p.shadow) },
      uHighlightColor: { value: new THREE.Color(p.highlight) },
      uRimColor: { value: new THREE.Color(p.rim) },
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
// 4. THE CHRONO CORE SCENE COMPOSITION
// ============================================================================
interface ChronoSceneProps {
  palette: CelPalette;
  outlineThickness: number;
  bands: number;
  enableHalftone: boolean;
  enableRim: boolean;
  speed: number;
}

function ChronoScene({
  palette,
  outlineThickness,
  bands,
  enableHalftone,
  enableRim,
  speed,
}: ChronoSceneProps) {
  const groupRef = useRef<THREE.Group>(null);
  const ring1Ref = useRef<THREE.Group>(null);
  const ring2Ref = useRef<THREE.Group>(null);
  const shardsRef = useRef<THREE.Group>(null);

  const { pointer } = useThree();
  const lightPos = useMemo(() => new THREE.Vector3(5, 5, 5), []);

  // Procedural Geometries
  const coreGeom = useMemo(() => new THREE.IcosahedronGeometry(1.2, 0), []);
  const ringGeom = useMemo(() => new THREE.TorusGeometry(2.0, 0.08, 12, 48), []);
  const outerRingGeom = useMemo(() => new THREE.TorusGeometry(2.5, 0.06, 12, 48), []);
  const shardGeom = useMemo(() => new THREE.OctahedronGeometry(0.28, 0), []);

  // Update light position tracking pointer & animate rotations
  useFrame((_, delta) => {
    // Dynamic light reacts to mouse pointer
    lightPos.set(pointer.x * 6 + 2, pointer.y * 6 + 3, 5);

    const s = delta * speed;
    if (groupRef.current) {
      groupRef.current.rotation.y += s * 0.4;
      groupRef.current.rotation.x = Math.sin(Date.now() * 0.001) * 0.2;
    }
    if (ring1Ref.current) {
      ring1Ref.current.rotation.x += s * 0.8;
      ring1Ref.current.rotation.y += s * 0.5;
    }
    if (ring2Ref.current) {
      ring2Ref.current.rotation.y -= s * 0.7;
      ring2Ref.current.rotation.z += s * 0.6;
    }
    if (shardsRef.current) {
      shardsRef.current.rotation.y += s * 0.6;
    }
  });

  return (
    <group ref={groupRef}>
      {/* Central Tactical Core */}
      <CelShadedMesh
        geometry={coreGeom}
        palette={palette}
        outlineThickness={outlineThickness}
        bands={bands}
        enableHalftone={enableHalftone}
        enableRim={enableRim}
        lightPos={lightPos}
      />

      {/* Orbiting Gimbal Ring 1 */}
      <group ref={ring1Ref}>
        <CelShadedMesh
          geometry={ringGeom}
          palette={palette === "crimson" ? "gold" : "crimson"}
          outlineThickness={outlineThickness * 0.75}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
      </group>

      {/* Orbiting Gimbal Ring 2 */}
      <group ref={ring2Ref}>
        <CelShadedMesh
          geometry={outerRingGeom}
          palette={palette === "cyan" ? "crimson" : "cyan"}
          outlineThickness={outlineThickness * 0.75}
          bands={bands}
          enableHalftone={enableHalftone}
          enableRim={enableRim}
          lightPos={lightPos}
        />
      </group>

      {/* Floating Tactical Shards */}
      <group ref={shardsRef}>
        {[0, 1, 2, 3].map((i) => {
          const angle = (i * Math.PI) / 2;
          const r = 2.9;
          const pos: [number, number, number] = [
            Math.cos(angle) * r,
            (i % 2 === 0 ? 0.6 : -0.6),
            Math.sin(angle) * r,
          ];
          return (
            <group key={i} position={pos}>
              <CelShadedMesh
                geometry={shardGeom}
                palette={i % 2 === 0 ? "gold" : "cyan"}
                outlineThickness={outlineThickness * 0.7}
                bands={bands}
                enableHalftone={false}
                enableRim={true}
                lightPos={lightPos}
              />
            </group>
          );
        })}
      </group>
    </group>
  );
}

// ============================================================================
// 5. EXPORTABLE CONTROLLER CANVAS
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
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  if (!isClient) {
    return (
      <div className={`w-full h-full bg-[#080c14] flex items-center justify-center ${className}`}>
        <span className="text-xs font-mono font-bold text-[#00f0ff] animate-pulse">
          INITIALIZING_3D_CEL_PIPELINE...
        </span>
      </div>
    );
  }

  return (
    <div className={`relative w-full h-full overflow-hidden ${className}`}>
      <Canvas
        camera={{ position: [0, 0, 6.2], fov: 45 }}
        gl={{ antialias: true, alpha: true }}
        className="w-full h-full"
      >
        <ChronoScene
          palette={palette}
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
