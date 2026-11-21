#!/usr/bin/env python3
# _generate_math_fixtures.py - 数学测试 fixture generator (6 族 × 5 case = 30 fixture)
# 触发: 2026-11-21 user "完善自动测试脚本和mock项目，加入数学测试的内容"
# 守门: #5 无 secret + #11 缺标比错标 + #19 Python 化累积规
#
# 6 族 (与 Rust gvpe_math / 前端数学模块一一对应):
#   vec3      — 向量代数 (reflect 保长/对合, Lagrange 恒等式)  ↔ crates/physis-wasm/src/gvpe_math/vec3.rs
#   quat      — 四元数 (slerp 端点/单位性, 旋转保内积)        ↔ gvpe_math/quat.rs
#   mat3      — 3x3 矩阵 (逆定义式, 结合律, 转置对合)         ↔ gvpe_math/mat3.rs
#   transform — 刚体变换 (复合/逆往返)                        ↔ gvpe_math/transform.rs
#   aabb      — 轴对齐包围盒 (交集幂等/交换/overlaps 一致)    ↔ gvpe_math/aabb.rs
#   coords    — 画布坐标换算 (client↔world 往返, letterbox)   ↔ frontend/src/lib/canvas/coords.ts
#
# 每 fixture 含: input + expected + tolerance + invariant (数学不变量名) + source_ref。
# expected 由 Python 参考实现独立算出 (与 Rust/TS 实现互为 oracle, 不抄实现代码)。
# 验证: _lib_validate.py validate-math <dir> 重算不变量, 误差超 tolerance 即 FAIL。

from __future__ import annotations

import json
import math
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
MOCK_DATA = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "math"

# ---- 参考实现 (纯 Python, 独立于 Rust/TS) ----

def v_dot(a, b):
    return sum(x * y for x, y in zip(a, b))

def v_cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]

def v_norm(a):
    return math.sqrt(v_dot(a, a))

def v_normalize(a):
    n = v_norm(a)
    return [x / n for x in a]

def v_reflect(v, n):
    d = 2 * v_dot(v, n)
    return [v[i] - d * n[i] for i in range(3)]

def q_mul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return [
        aw*bx + ax*bw + ay*bz - az*by,
        aw*by - ax*bz + ay*bw + az*bx,
        aw*bz + ax*by - ay*bx + az*bw,
        aw*bw - ax*bx - ay*by - az*bz,
    ]

def q_rotate(q, v):
    # v' = q * (0,v) * q⁻¹ (单位四元数)
    p = q_mul(q, [v[0], v[1], v[2], 0.0])
    qi = [-q[0], -q[1], -q[2], q[3]]
    r = q_mul(p, qi)
    return r[:3]

def q_slerp(a, b, t):
    dot = sum(x*y for x, y in zip(a, b))
    if dot < 0:
        b = [-x for x in b]
        dot = -dot
    if dot > 0.9995:
        r = [a[i] + t*(b[i]-a[i]) for i in range(4)]
        n = math.sqrt(sum(x*x for x in r))
        return [x/n for x in r]
    theta = math.acos(dot)
    s = math.sin(theta)
    wa, wb = math.sin((1-t)*theta)/s, math.sin(t*theta)/s
    return [wa*a[i] + wb*b[i] for i in range(4)]

def m_mul(a, b):
    # 行优先 3x3, a[i][j] = 第 i 行第 j 列
    return [[sum(a[i][k]*b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]

def m_transpose(a):
    return [[a[j][i] for j in range(3)] for i in range(3)]

def m_inverse(a):
    (a00,a01,a02),(a10,a11,a12),(a20,a21,a22) = a
    c00 = a11*a22 - a12*a21
    c01 = -(a10*a22 - a12*a20)
    c02 = a10*a21 - a11*a20
    det = a00*c00 + a01*c01 + a02*c02
    if abs(det) < 1e-12:
        return None
    c10 = -(a01*a22 - a02*a21)
    c11 = a00*a22 - a02*a20
    c12 = -(a00*a21 - a01*a20)
    c20 = a01*a12 - a02*a11
    c21 = -(a00*a12 - a02*a10)
    c22 = a00*a11 - a01*a10
    inv = 1.0/det
    return [[c00*inv,c10*inv,c20*inv],[c01*inv,c11*inv,c21*inv],[c02*inv,c12*inv,c22*inv]]

def m_apply(a, v):
    return [v_dot(row, v) for row in a]

IDENTITY3 = [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]]

# ---- fixture 构造 ----

def fixture(family: str, case: str, invariant: str, source_ref: str,
            input_: dict, expected: dict, tolerance: float = 1e-6,
            description: str = "") -> dict:
    return {
        "fixture_version": "v1",
        "module": "math",
        "family": family,
        "case": case,
        "method": "COMPUTE",
        "scenario": f"math-{family}-{case}",
        "description": description or f"数学不变量 {invariant} ({family} 族, per 2026-11-21 数学测试扩展)",
        "schema_ref": source_ref,
        "invariant": invariant,
        "tolerance": tolerance,
        "input": input_,
        "expected": expected,
        "守门": ["#5 无 secret", "#11 缺标比错标", "#19 Python 化累积规"],
    }


def gen_vec3() -> list[dict]:
    out = []
    cases = [
        ([1.0,-2.0,3.0],[0.0,1.0,0.0]),
        ([3.0,4.0,0.0],[1.0,0.0,0.0]),
        ([1.0,1.0,1.0],v_normalize([1.0,1.0,1.0])),
        ([0.5,-0.25,2.0],v_normalize([0.0,0.0,1.0])),
        ([-3.0,0.0,4.0],v_normalize([1.0,1.0,0.0])),
    ]
    for i,(v,n) in enumerate(cases,1):
        r = v_reflect(v,n)
        rr = v_reflect(r,n)
        out.append(fixture("vec3", f"{i:02d}-reflect", "reflect_preserves_length_and_involution",
            "crates/physis-wasm/src/gvpe_math/vec3.rs::reflect",
            {"v": v, "n": n},
            {"reflect": r, "length_v": v_norm(v), "length_reflect": v_norm(r),
             "double_reflect": rr, "involution_error": v_norm([rr[k]-v[k] for k in range(3)])}))
    return out


def gen_quat() -> list[dict]:
    out = []
    # 绕 z 轴 90° 与绕 x 轴 90°
    qz = [0.0,0.0,math.sin(math.pi/4),math.cos(math.pi/4)]
    qx = [math.sin(math.pi/4),0.0,0.0,math.cos(math.pi/4)]
    cases = [
        ("slerp-endpoints", qz, qx, [0.0,1.0], {"q0": qz, "q1": qx}),
        ("slerp-midpoint-unit", qz, qx, [0.5], None),
        ("rotate-preserves-dot", qz, None, None, None),
    ]
    # case 01/02: slerp 端点 + 中点单位性
    for i,(name,a,b,ts,_) in enumerate(cases[:2],1):
        expected = {}
        for t in ts:
            q = q_slerp(a,b,t)
            expected[f"slerp_t{t}"] = q
            expected[f"norm_t{t}"] = math.sqrt(sum(x*x for x in q))
        out.append(fixture("quat", f"{i:02d}-{name}", "slerp_endpoints_and_unit_norm",
            "crates/physis-wasm/src/gvpe_math/quat.rs::slerp",
            {"a": a, "b": b, "t_values": ts}, expected))
    # case 03: 旋转保内积
    u, w = [1.0,2.0,3.0], [-2.0,0.5,1.0]
    ru, rw = q_rotate(qz,u), q_rotate(qz,w)
    out.append(fixture("quat", "03-rotate-preserves-dot", "rotation_preserves_inner_product",
        "crates/physis-wasm/src/gvpe_math/quat.rs::rotate",
        {"q": qz, "u": u, "w": w},
        {"dot_before": v_dot(u,w), "dot_after": v_dot(ru,rw),
         "rotated_u": ru, "rotated_w": rw}))
    # case 04: q·q⁻¹ = identity
    qi = [-qz[0],-qz[1],-qz[2],qz[3]]
    prod = q_mul(qz,qi)
    out.append(fixture("quat", "04-inverse-identity", "quaternion_inverse_identity",
        "crates/physis-wasm/src/gvpe_math/quat.rs::inverse",
        {"q": qz}, {"q_inverse": qi, "q_times_qinv": prod}))
    # case 05: 旋转保长度
    v = [3.0,-4.0,12.0]
    rv = q_rotate(qx,v)
    out.append(fixture("quat", "05-rotate-preserves-length", "rotation_preserves_length",
        "crates/physis-wasm/src/gvpe_math/quat.rs::rotate",
        {"q": qx, "v": v}, {"length_before": v_norm(v), "length_after": v_norm(rv), "rotated": rv}))
    return out


def gen_mat3() -> list[dict]:
    out = []
    # 非对称可逆矩阵 (对角矩阵会掩盖余子式符号错误 — 2026-11-21 真实 bug 教训)
    A = [[2.0,1.0,0.5],[0.0,3.0,1.0],[1.0,0.0,4.0]]
    B = [[1.0,2.0,0.0],[0.5,1.0,3.0],[2.0,0.0,1.0]]
    C = [[0.0,1.0,2.0],[1.0,0.0,1.0],[3.0,1.0,0.0]]
    Ainv = m_inverse(A)
    out.append(fixture("mat3", "01-inverse-definition", "inverse_times_original_is_identity",
        "crates/physis-wasm/src/gvpe_math/mat3.rs::inverse",
        {"m": A}, {"inverse": Ainv, "inv_times_m": m_mul(Ainv,A), "m_times_inv": m_mul(A,Ainv)}))
    out.append(fixture("mat3", "02-mul-associative", "matrix_multiplication_associativity",
        "crates/physis-wasm/src/gvpe_math/mat3.rs::mul_mat3",
        {"a": A, "b": B, "c": C},
        {"ab_times_c": m_mul(m_mul(A,B),C), "a_times_bc": m_mul(A,m_mul(B,C))}))
    out.append(fixture("mat3", "03-transpose-involution", "transpose_is_involution",
        "crates/physis-wasm/src/gvpe_math/mat3.rs::transpose",
        {"m": A}, {"transpose": m_transpose(A), "double_transpose": m_transpose(m_transpose(A))}))
    # 旋转矩阵 (绕 z 30°): 逆 = 转置
    th = math.pi/6
    R = [[math.cos(th),-math.sin(th),0.0],[math.sin(th),math.cos(th),0.0],[0.0,0.0,1.0]]
    out.append(fixture("mat3", "04-rotation-inverse-transpose", "rotation_inverse_equals_transpose",
        "crates/physis-wasm/src/gvpe_math/mat3.rs::inverse",
        {"m": R}, {"inverse": m_inverse(R), "transpose": m_transpose(R)}))
    # 奇异矩阵 → inverse = None
    S = [[1.0,2.0,3.0],[2.0,4.0,6.0],[0.0,0.0,1.0]]
    out.append(fixture("mat3", "05-singular-no-inverse", "singular_matrix_has_no_inverse",
        "crates/physis-wasm/src/gvpe_math/mat3.rs::inverse",
        {"m": S}, {"inverse": None, "determinant": 0.0}))
    return out


def gen_transform() -> list[dict]:
    out = []
    # Transform = (rotation quat, translation vec3); 复合 = 先 b 后 a
    qz = [0.0,0.0,math.sin(math.pi/4),math.cos(math.pi/4)]
    qx = [math.sin(math.pi/6),0.0,0.0,math.cos(math.pi/6)]
    ta, tb = [1.0,2.0,3.0], [-2.0,0.5,1.0]
    def apply(q,t,v):
        r = q_rotate(q,v)
        return [r[i]+t[i] for i in range(3)]
    def compose(qa,ta_,qb,tb_):
        # (qa,ta)∘(qb,tb): v → qa*(qb*v+tb)+ta
        return q_mul(qa,qb), [x+y for x,y in zip(q_rotate(qa,tb_),ta_)]
    def invert(q,t):
        qi = [-q[0],-q[1],-q[2],q[3]]
        return qi, [-x for x in q_rotate(qi,t)]
    v = [0.5,-1.0,2.0]
    qc,tc = compose(qz,ta,qx,tb)
    out.append(fixture("transform", "01-compose-apply", "composition_equals_sequential_application",
        "crates/physis-wasm/src/gvpe_math/transform.rs::compose",
        {"a": {"rotation": qz, "translation": ta}, "b": {"rotation": qx, "translation": tb}, "v": v},
        {"composed": {"rotation": qc, "translation": tc},
         "sequential": apply(qz,ta,apply(qx,tb,v)), "composed_apply": apply(qc,tc,v)}))
    qinv,tinv = invert(qz,ta)
    out.append(fixture("transform", "02-inverse-roundtrip", "inverse_roundtrip_is_identity",
        "crates/physis-wasm/src/gvpe_math/transform.rs::inverse",
        {"t": {"rotation": qz, "translation": ta}, "v": v},
        {"inverse": {"rotation": qinv, "translation": tinv},
         "roundtrip": apply(qinv,tinv,apply(qz,ta,v)), "original": v}))
    # 单位元
    qid = [0.0,0.0,0.0,1.0]
    out.append(fixture("transform", "03-identity-element", "identity_transform_is_neutral",
        "crates/physis-wasm/src/gvpe_math/transform.rs::identity",
        {"t": {"rotation": qz, "translation": ta}, "v": v},
        {"identity_apply": apply(qid,[0.0,0.0,0.0],v),
         "compose_identity": apply(*compose(qz,ta,qid,[0.0,0.0,0.0]),v),
         "original_apply": apply(qz,ta,v)}))
    # 结合律 (3 变换)
    qy = [0.0,math.sin(math.pi/8),0.0,math.cos(math.pi/8)]
    tc2 = [0.0,1.0,-1.0]
    A = {"rotation": qz, "translation": ta}
    B = {"rotation": qx, "translation": tb}
    C = {"rotation": qy, "translation": tc2}
    def apply_t(t, v):
        return apply(t["rotation"], t["translation"], v)
    def compose_t(a, b):
        q, t = compose(a["rotation"], a["translation"], b["rotation"], b["translation"])
        return {"rotation": q, "translation": t}
    lhs = apply_t(compose_t(compose_t(A, B), C), v)
    rhs = apply_t(compose_t(A, compose_t(B, C)), v)
    out.append(fixture("transform", "04-compose-associative", "composition_associativity",
        "crates/physis-wasm/src/gvpe_math/transform.rs::compose",
        {"a": A, "b": B, "c": C, "v": v}, {"lhs": lhs, "rhs": rhs}))
    # 逆的逆 = 原
    qii,tii = invert(qinv,tinv)
    out.append(fixture("transform", "05-double-inverse", "double_inverse_is_original",
        "crates/physis-wasm/src/gvpe_math/transform.rs::inverse",
        {"t": {"rotation": qz, "translation": ta}},
        {"double_inverse": {"rotation": qii, "translation": tii},
         "original": {"rotation": qz, "translation": ta}}))
    return out


def gen_aabb() -> list[dict]:
    out = []
    def intersect(a,b):
        lo = [max(a["min"][i],b["min"][i]) for i in range(3)]
        hi = [min(a["max"][i],b["max"][i]) for i in range(3)]
        if any(lo[i] > hi[i] for i in range(3)):
            return None
        return {"min": lo, "max": hi}
    def overlaps(a,b):
        return intersect(a,b) is not None
    A = {"min":[0.0,0.0,0.0],"max":[2.0,2.0,2.0]}
    B = {"min":[1.0,1.0,1.0],"max":[3.0,3.0,3.0]}
    C = {"min":[5.0,5.0,5.0],"max":[6.0,6.0,6.0]}
    iab = intersect(A,B)
    out.append(fixture("aabb", "01-intersection-idempotent", "intersection_idempotent",
        "crates/physis-wasm/src/gvpe_math/aabb.rs::intersection",
        {"a": A, "b": B},
        {"intersection": iab, "intersect_again": intersect(iab,iab)}))
    out.append(fixture("aabb", "02-intersection-commutative", "intersection_commutative",
        "crates/physis-wasm/src/gvpe_math/aabb.rs::intersection",
        {"a": A, "b": B}, {"a_intersect_b": iab, "b_intersect_a": intersect(B,A)}))
    out.append(fixture("aabb", "03-overlaps-consistent", "overlaps_iff_intersection_nonempty",
        "crates/physis-wasm/src/gvpe_math/aabb.rs::overlaps",
        {"a": A, "b": B, "c": C},
        {"overlaps_ab": overlaps(A,B), "overlaps_ac": overlaps(A,C),
         "intersection_ab": iab, "intersection_ac": intersect(A,C)}))
    # 包含关系: A ⊇ D ⇒ intersect(A,D) = D
    D = {"min":[0.5,0.5,0.5],"max":[1.5,1.5,1.5]}
    out.append(fixture("aabb", "04-containment-absorption", "contained_box_absorbed_by_intersection",
        "crates/physis-wasm/src/gvpe_math/aabb.rs::intersection",
        {"a": A, "d": D}, {"intersection": intersect(A,D), "contained": D}))
    # 面接触 (边界相切) 仍算 overlap
    E = {"min":[2.0,0.0,0.0],"max":[4.0,2.0,2.0]}
    out.append(fixture("aabb", "05-face-touch-overlaps", "face_touching_boxes_overlap",
        "crates/physis-wasm/src/gvpe_math/aabb.rs::overlaps",
        {"a": A, "e": E}, {"overlaps": overlaps(A,E), "intersection": intersect(A,E)}))
    return out


def gen_coords() -> list[dict]:
    out = []
    # letterbox meet: s = max(vb.w/rect.w, vb.h/rect.h); ox/oy 居中留白
    def letterbox(rect, vb):
        rw = rect["width"] if rect["width"] > 0 else 1
        rh = rect["height"] if rect["height"] > 0 else 1
        s = max(vb["width"]/rw, vb["height"]/rh)
        return {"s": s, "ox": (rw - vb["width"]/s)/2, "oy": (rh - vb["height"]/s)/2}
    def client_to_world(client, rect, vb, vp):
        lb = letterbox(rect, vb)
        sx = (client["clientX"] - rect["left"] - lb["ox"]) * lb["s"] + vb["x"]
        sy = (client["clientY"] - rect["top"] - lb["oy"]) * lb["s"] + vb["y"]
        zoom = vp["zoom"] if vp["zoom"] > 0 else 1
        return {"x": sx/zoom + vp["x"], "y": sy/zoom + vp["y"]}
    def world_to_client(world, rect, vb, vp):
        lb = letterbox(rect, vb)
        sx = (world["x"] - vp["x"]) * vp["zoom"]
        sy = (world["y"] - vp["y"]) * vp["zoom"]
        return {"clientX": (sx - vb["x"])/lb["s"] + rect["left"] + lb["ox"],
                "clientY": (sy - vb["y"])/lb["s"] + rect["top"] + lb["oy"]}
    cases = [
        ({"left":0,"top":0,"width":1200,"height":800}, {"x":0,"y":0,"width":1200,"height":800}, {"x":0,"y":0,"zoom":1.0}, {"clientX":600,"clientY":400}),
        ({"left":100,"top":50,"width":1600,"height":800}, {"x":0,"y":0,"width":1200,"height":800}, {"x":10,"y":-20,"zoom":1.5}, {"clientX":900,"clientY":450}),
        ({"left":0,"top":0,"width":1200,"height":1000}, {"x":-50,"y":30,"width":1200,"height":800}, {"x":0,"y":0,"zoom":0.5}, {"clientX":300,"clientY":700}),
        ({"left":-20,"top":10,"width":800,"height":600}, {"x":0,"y":0,"width":1200,"height":800}, {"x":100,"y":100,"zoom":2.0}, {"clientX":400,"clientY":300}),
        ({"left":0,"top":0,"width":2400,"height":1600}, {"x":0,"y":0,"width":1200,"height":800}, {"x":-5,"y":7,"zoom":0.8}, {"clientX":1200,"clientY":800}),
    ]
    for i,(rect,vb,vp,client) in enumerate(cases,1):
        world = client_to_world(client,rect,vb,vp)
        back = world_to_client(world,rect,vb,vp)
        out.append(fixture("coords", f"{i:02d}-roundtrip", "client_world_roundtrip_identity",
            "frontend/src/lib/canvas/coords.ts::clientToWorld",
            {"rect": rect, "viewBox": vb, "viewport": vp, "client": client},
            {"world": world, "roundtrip_client": back,
             "letterbox": letterbox(rect,vb),
             "roundtrip_error": math.hypot(back["clientX"]-client["clientX"], back["clientY"]-client["clientY"])},
            tolerance=1e-6))
    return out


GENERATORS = {
    "vec3": gen_vec3, "quat": gen_quat, "mat3": gen_mat3,
    "transform": gen_transform, "aabb": gen_aabb, "coords": gen_coords,
}


def main() -> int:
    total = 0
    for family, gen in GENERATORS.items():
        out_dir = MOCK_DATA / family
        out_dir.mkdir(parents=True, exist_ok=True)
        fixtures = gen()
        for fx in fixtures:
            name = f"v1--math--{family}--{fx['case']}--COMPUTE.json"
            (out_dir / name).write_text(json.dumps(fx, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            total += 1
        print(f"FAMILY={family} FIXTURES={len(fixtures)}")
    print(f"TOTAL={total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())