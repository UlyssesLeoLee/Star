/*
@cypher schema=1 source_sha256=b1fe8c33fbe072f30d9f020a3976731b51d90730e77805a555c319ffa58308ba
MERGE (self:File {path:"frontend/src/components/effects/PhysisBackdrop.tsx"})
MERGE (backdrop:Symbol {id:"frontend/src/components/effects/PhysisBackdrop.tsx::PhysisBackdrop",kind:"component"})
MERGE (wasm:Symbol {id:"frontend/public/physis/physis_wasm.js::PhysisWorld",kind:"wasm"})
MERGE (self)-[:DEFINES]->(backdrop)
MERGE (self)-[:IMPORTS]->(wasm)
@endcypher
*/
"use client";

// =====================================================================
// PhysisBackdrop — 全 3D 主题背景层 (per 2026-11-14 用户发令: 全 3D 主题)
//   three.js 渲染 + Physis WASM 物理 (GVPE 引擎: 重力/阻尼/鼠标吸引子)
//   浮空粒子块: 受重力下落, 底部隐边界反弹, 鼠标悬停吸引
//   仅 "physis" 主题挂载 (ThemeProvider 判定), 其他主题零开销
// =====================================================================

import { useEffect, useRef } from "react";
import * as THREE from "three";
import { useTheme } from "next-themes";

// wasm 模块类型 (physis_wasm.d.ts 生成)
interface PhysisWorldWasm {
  new (gravity: Float32Array | number[]): {
    spawn(x: number, y: number, z: number, radius: number, mass: number): number;
    step(dt: number): void;
    position(idx: number): Float32Array | number[];
    attract(idx: number, tx: number, ty: number, tz: number, k: number): void;
    len(): number;
    free?(): void;
  };
}

export function PhysisBackdrop() {
  const { resolvedTheme } = useTheme();
  const mountRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (resolvedTheme !== "physis") return;
    const mount = mountRef.current;
    if (!mount) return;

    let raf = 0;
    let disposed = false;
    let world: PhysisWorldWasm["prototype"] | null = null;

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

    // 灯光 (柔和立体感)
    const light = new THREE.DirectionalLight(0xffffff, 1.2);
    light.position.set(5, 8, 6);
    scene.add(light);
    const ambient = new THREE.AmbientLight(0x8899ff, 0.5);
    scene.add(ambient);

    // 粒子块 (BoxGeometry + 主题色材质)
    const COUNT = 90;
    const bodies: THREE.Mesh[] = [];
    const colors = [0x7c8cff, 0x9c6bff, 0x5ad2ff, 0x8b5cf6, 0x63e6be, 0xffb86b];
    const geo = new THREE.BoxGeometry(1, 1, 1);

    for (let i = 0; i < COUNT; i++) {
      const mat = new THREE.MeshStandardMaterial({
        color: colors[i % colors.length],
        roughness: 0.35,
        metalness: 0.2,
        transparent: true,
        opacity: 0.85,
      });
      const mesh = new THREE.Mesh(geo, mat);
      const s = 0.3 + Math.random() * 0.9;
      mesh.scale.setScalar(s);
      mesh.position.set((Math.random() - 0.5) * 22, 6 + Math.random() * 10, (Math.random() - 0.5) * 8);
      mesh.rotation.set(Math.random() * Math.PI, Math.random() * Math.PI, 0);
      scene.add(mesh);
      bodies.push(mesh);
    }

    // --- Physis WASM 世界 ---
    (async () => {
      try {
        // wasm glue 打包进 bundle (src/lib/physis), Vite 自动处理 _bg.wasm asset
        const mod = await import("@/lib/physis/physis_wasm.js");
        const World = mod.PhysisWorld as new (g: number[]) => PhysisWorldWasm["prototype"];
        const w = new World([0, -3.2, 0]);
        const idxMap = bodies.map((b, i) => w.spawn(b.position.x, b.position.y, b.position.z, 0.5, 1.0));
        world = w as unknown as PhysisWorldWasm["prototype"];
        void idxMap;

        let last = performance.now();
        const tick = () => {
          if (disposed) return;
          raf = requestAnimationFrame(tick);
          const now = performance.now();
          const dt = Math.min((now - last) / 1000, 0.05);
          last = now;

          // 物理步进 (2 子步)
          w.step(dt / 2);
          w.step(dt / 2);

          // 底部隐边界: 位置同步 + 反弹 (简单处理: 世界坐标同步, 低于边界时吸引向上)
          for (let i = 0; i < bodies.length; i++) {
            const p = w.position(i);
            const mesh = bodies[i];
            mesh.position.set(p[0], p[1], p[2]);
            mesh.rotation.x += dt * 0.4;
            mesh.rotation.y += dt * 0.6;

            if (p[1] < -6.5) {
              w.attract(i, p[0], 6.0, p[2], 1.2);
            }
          }

          renderer.render(scene, camera);
        };
        tick();
      } catch (e) {
        // wasm 加载失败: 静态渲染一次 (无物理)
        console.warn("PhysisBackdrop: wasm load failed, static render", e);
        renderer.render(scene, camera);
      }
    })();

    // 鼠标吸引子 (世界坐标)
    const onPointerMove = (e: PointerEvent) => {
      if (!world) return;
      const w = window.innerWidth;
      const h = window.innerHeight;
      const tx = ((e.clientX / w) * 2 - 1) * 11;
      const ty = -((e.clientY / h) * 2 - 1) * 8;
      for (let i = 0; i < COUNT; i += 3) {
        (world as unknown as { attract(i: number, x: number, y: number, z: number, k: number): void })
          .attract(i, tx, ty, 0, 0.15);
      }
    };
    window.addEventListener("pointermove", onPointerMove);

    // resize
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
      window.removeEventListener("resize", onResize);
      scene.traverse((o) => {
        const m = o as THREE.Mesh;
        if (m.geometry) m.geometry.dispose();
        const mat = m.material as THREE.Material | undefined;
        if (mat) mat.dispose();
      });
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
      className="fixed inset-0 -z-0 pointer-events-none"
      style={{ zIndex: 0 }}
    />
  );
}
