/* tslint:disable */
/* eslint-disable */

/**
 * 浏览器侧物理世界: 浮空粒子块 (重力 + 球体碰撞 + 鼠标吸引子)
 */
export class PhysisWorld {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 施加外力 (鼠标吸引子): 向 target 施加力
     */
    attract(idx: number, tx: number, ty: number, tz: number, k: number): void;
    /**
     * 是否为空
     */
    is_empty(): boolean;
    /**
     * 刚体数量
     */
    len(): number;
    /**
     * 创建世界, gravity=[gx,gy,gz]
     */
    constructor(gravity: Float32Array | number[]);
    /**
     * 读取第 idx 个刚体位置 [x,y,z] (不存在返回 [0,0,0])
     */
    position(idx: number): Float32Array;
    /**
     * 生成动态球体 (radius 用于 profile)
     */
    spawn(x: number, y: number, z: number, radius: number, mass: number): number;
    /**
     * 步进 dt 秒
     */
    step(dt: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_physisworld_free: (a: number, b: number) => void;
    readonly physisworld_attract: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly physisworld_is_empty: (a: number) => number;
    readonly physisworld_len: (a: number) => number;
    readonly physisworld_new: (a: number, b: number) => number;
    readonly physisworld_position: (a: number, b: number) => [number, number];
    readonly physisworld_spawn: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly physisworld_step: (a: number, b: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
