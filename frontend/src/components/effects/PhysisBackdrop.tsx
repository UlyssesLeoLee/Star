/*
@cypher schema=1 source_sha256=74006c3c2020d437c31ac8fbf26e95aa9b1f1a34e67234620478583fc8daf6db
MERGE (self:File {path:"frontend/src/components/effects/PhysisBackdrop.tsx"})
MERGE (backdrop:Symbol {id:"frontend/src/components/effects/PhysisBackdrop.tsx::PhysisBackdrop",kind:"component"})
MERGE (wasm:Symbol {id:"frontend/src/lib/physis/physis_wasm.js::PhysisWorld",kind:"wasm"})
MERGE (self)-[:DEFINES]->(backdrop)
MERGE (self)-[:IMPORTS]->(wasm)
@endcypher
*/
"use client";

// =====================================================================
// PhysisBackdrop — 全 3D 物理主题背景层 (游戏化动态版)
// per 2026-11-14: 全 3D 主题应有游戏化动态化整体设计
//   - 多形状粒子 (盒/二十面体/环面/八面体) + 自旋 + emissive 发光
//   - 真实物理: 重力/阻尼/底部反弹 (Physis GVPE WASM)
//   - 鼠标 = 光晕点光源 + 吸引子 (粒子追随光标)
//   - 点击任意处 → 冲击波 (斥力场推开粒子)
//   - 自定义事件 "physis-burst" (聊天发消息/任务完成 → 粒子爆发庆祝)
// =====================================================================

import { useEffect, useRef } from "react";
import * as THREE from "three";
import { useTheme } from "next-themes";

interface PhysisWorldWasm {
  new (gravity: number[]): {
    spawn(x: number, y: number, z: number, radius: number, mass: number): number;
    step(dt: number): void;
    position(idx: number): number[];
    attract(idx: number, tx: number, ty: number, tz: number, k: number): void;
    len(): number;
  };
}

const COUNT = 90;
// 主题粒子色板 (游戏化霓虹)
const PALETTE = [0x7c8cff, 0x9c6bff, 0x5ad2ff, 0x8b5cf6, 0x63e6be, 0xffb86b, 0xff5c7a];

export function PhysisBackdrop() {
  const { resolvedTheme } = useTheme();
  const mountRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (resolvedTheme !== "physis") return;
    const mount = mountRef.current;
    if (!mount) return;

    let raf = 0;
    let disposed = false;
    let world: InstanceType<PhysisWorldWasm> | null = null;
    const shocks: { x: number; y: number; age: number }[] = [];
    let mouseWorld = { x: 0, y: 0 };

    // --- three.js 场景 ---
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 100);
    camera.position.set(0, 0, 14);

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);
    mount.appendChild(renderer.domElement);
    renderer.domElement.style.width = "100%";
    renderer.domElement.style.height = "100%";

    // 光晕点光源 (跟随鼠标) + 基础光
    const cursorLight = new THREE.PointLight(0x8b5cf6, 0, 30);
    cursorLight.position.set(0, 0, 5);
    scene.add(cursorLight);
    const dir = new THREE.DirectionalLight(0xffffff, 1.1);
    dir.position.set(5, 8, 6);
    scene.add(dir);
    scene.add(new THREE.AmbientLight(0x8899ff, 0.45));

    // --- 多形状粒子 ---
    const geos = [
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.IcosahedronGeometry(0.62, 0),
      new THREE.TorusGeometry(0.42, 0.16, 10, 20),
      new THREE.OctahedronGeometry(0.66, 0),
    ];
    const bodies: { mesh: THREE.Mesh; spin: [number, number] }[] = [];

    for (let i = 0; i < COUNT; i++) {
      const color = PALETTE[i % PALETTE.length];
      const mat = new THREE.MeshStandardMaterial({
        color,
        emissive: color,
        emissiveIntensity: 0.25 + Math.random() * 0.3,
        roughness: 0.3,
        metalness: 0.25,
        transparent: true,
        opacity: 0.9,
      });
      const mesh = new THREE.Mesh(geos[i % geos.length], mat);
      const s = 0.25 + Math.random() * 0.8;
      mesh.scale.setScalar(s);
      mesh.position.set((Math.random() - 0.5) * 22, 5 + Math.random() * 10, (Math.random() - 0.5) * 9);
      scene.add(mesh);
      bodies.push({
        mesh,
        spin: [(Math.random() - 0.5) * 1.6, (Math.random() - 0.5) * 1.2],
      });
    }

    // --- Physis WASM ---
    (async () => {
      try {
        const mod = await import("@/lib/physis/physis_wasm.js");
        const World = mod.PhysisWorld as unknown as PhysisWorldWasm;
        const w = new World([0, -3.0, 0]);
        bodies.forEach((b) => {
          w.spawn(b.mesh.position.x, b.mesh.position.y, b.mesh.position.z, 0.5, 1.0);
        });
        world = w;

        let last = performance.now();
        const tick = () => {
          if (disposed) return;
          raf = requestAnimationFrame(tick);
          const now = performance.now();
          const dt = Math.min((now - last) / 1000, 0.05);
          last = now;

          // 物理 2 子步
          w.step(dt / 2);
          w.step(dt / 2);

          // 同步粒子 + 冲击波 + 底部反弹
          for (let i = 0; i < bodies.length; i++) {
            const p = w.position(i);
            const { mesh, spin } = bodies[i];
            mesh.position.set(p[0], p[1], p[2]);
            mesh.rotation.x += dt * spin[0];
            mesh.rotation.y += dt * spin[1];

            if (p[1] < -6.5) w.attract(i, p[0], 6.0, p[2], 1.2);

            // 冲击波斥力 (老化衰减)
            for (const s of shocks) {
              const dx = p[0] - s.x;
              const dy = p[1] - s.y;
              const d2 = dx * dx + dy * dy;
              if (d2 < 64) {
                const d = Math.sqrt(d2) + 0.1;
                const f = (1 - s.age / 0.8) * (28 / d);
                w.attract(i, p[0] + dx * f * 2, p[1] + dy * f * 2, p[2], 0.08);
              }
            }
          }
          // 冲击波老化
          for (const s of shocks) s.age += dt;
          while (shocks.length > 0 && shocks[0].age > 0.8) shocks.shift();

          // 光晕光源跟随鼠标 (发光强度脉动)
          cursorLight.position.set(mouseWorld.x, mouseWorld.y, 5);
          cursorLight.intensity = 1.4 + Math.sin(now * 0.004) * 0.5;

          renderer.render(scene, camera);
        };
        tick();
      } catch (e) {
        console.warn("PhysisBackdrop: wasm load failed, static render", e);
        renderer.render(scene, camera);
      }
    })();

    // 屏幕 → 世界坐标
    const screenToWorld = (e: PointerEvent) => {
      const w = window.innerWidth;
      const h = window.innerHeight;
      return {
        x: ((e.clientX / w) * 2 - 1) * 11,
        y: -((e.clientY / h) * 2 - 1) * 8,
      };
    };

    // 鼠标 = 吸引子 + 光晕
    const onPointerMove = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      mouseWorld = t;
      for (let i = 0; i < COUNT; i += 3) {
        world.attract(i, t.x, t.y, 0, 0.12);
      }
    };

    // 点击 → 冲击波
    const onPointerDown = (e: PointerEvent) => {
      if (!world) return;
      const t = screenToWorld(e);
      shocks.push({ x: t.x, y: t.y, age: 0 });
      if (shocks.length > 4) shocks.shift();
    };

    // 粒子爆发 (window.dispatchEvent(new CustomEvent("physis-burst")))
    const onBurst = (e: Event) => {
      if (!world) return;
      const d = (e as CustomEvent<{ x?: number; y?: number }>).detail ?? {};
      const t = d.x !== undefined && d.y !== undefined
        ? { x: d.x, y: d.y }
        : { x: (Math.random() - 0.5) * 12, y: 2 };
      shocks.push({ x: t.x, y: t.y, age: -0.5 });
      if (shocks.length > 4) shocks.shift();
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("physis-burst", onBurst);

    const onResize = () => {
      renderer.setSize(mount.clientWidth, mount.clientHeight);
      camera.aspect = mount.clientWidth / Math.max(1, mount.clientHeight);
      camera.updateProjectionMatrix();
    };
    onResize();
    window.addEventListener("resize", onResize);

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("physis-burst", onBurst);
      window.removeEventListener("resize", onResize);
      scene.traverse((o) => {
        const m = o as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material as THREE.Material | undefined;
        if (mat) mat.dispose();
      });
      geos.forEach((g) => g.dispose());
      renderer.dispose();
      if (renderer.domElement.parentNode === mount) mount.removeChild(renderer.domElement);
    };
  }, [resolvedTheme]);

  if (resolvedTheme !== "physis") return null;

  return (
    <div
      ref={mountRef}
      data-testid="physis-backdrop"
      aria-hidden="true"
      className="fixed inset-0 pointer-events-none"
      style={{ zIndex: 0 }}
    />
  );
}
