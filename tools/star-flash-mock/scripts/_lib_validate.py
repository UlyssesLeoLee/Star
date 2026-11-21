#!/usr/bin/env python3
# _lib_validate.py — Star Mock Project 通用 fixture 验证 helper (per ULYS-140 + 守门 #19)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-140 (回归测试) 2026-09-20 JST — 替代慢 per-file grep loop (per 守门 #19 Python 化)
# 守门: #5 (no secret leak) + #11 (缺标比错标) + #19 (Python 化累积规)
#
# 用途: 给 regression-test-*.sh 调, 跑批量验证. 比 bash for-loop + grep 快 5x+ on Windows.
#
# 子命令:
#   validate-json  <dir>           — 所有 .json 解析 + 报 invalid
#   scan-secret    <dir>           — forbidden pattern 检测 (守门 #5)
#   require-field  <dir> <field>   — 每 .json 必须含 field
#   count-classes  <dir>           — 统计 W/T/M 分布 (per 守门 #13)
#   count-by-pattern <dir> <regex> — 按文件名 pattern 计数
#
# 输出: 每子命令 0 exit = OK, 1+ exit = FAIL; 行格式 `LABEL=value` 给 bash 解析.

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path
from typing import Iterable

FORBIDDEN_PATTERNS = (
    "password=",
    "api_key=",
    "secret=",
    "token=",
    "BEGIN PRIVATE KEY",
    "GHCR_PAT",
)


def iter_json_files(root: Path) -> Iterable[Path]:
    if not root.exists():
        return
    yield from root.rglob("*.json")


def cmd_validate_json(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    invalid: list[str] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            json.loads(fp.read_text(encoding="utf-8"))
        except Exception as e:
            invalid.append(f"{fp}: {e}")
    print(f"TOTAL={total}")
    print(f"INVALID={len(invalid)}")
    for line in invalid[:20]:  # cap to 20 to avoid flood
        print(f"INVALID_FILE={line}", file=sys.stderr)
    return 0 if not invalid else 1


def cmd_scan_secret(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    leaks: list[tuple[Path, str]] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            content = fp.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        lower = content.lower()
        for pat in FORBIDDEN_PATTERNS:
            if pat.lower() in lower:
                leaks.append((fp, pat))
                break
    print(f"TOTAL={total}")
    print(f"LEAKS={len(leaks)}")
    for fp, pat in leaks[:20]:
        print(f"LEAK={fp.name}: {pat}", file=sys.stderr)
    return 0 if not leaks else 1


def cmd_require_field(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    field = args.field
    missing: list[Path] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            content = fp.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            missing.append(fp)
            continue
        if field not in content:
            missing.append(fp)
    print(f"TOTAL={total}")
    print(f"MISSING={len(missing)}")
    for fp in missing[:20]:
        print(f"MISSING_FIELD={fp}", file=sys.stderr)
    return 0 if not missing else 1


def cmd_count_classes(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    counts: dict[str, int] = {"Work": 0, "Transaction": 0, "Master": 0, "?": 0}
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            data = json.loads(fp.read_text(encoding="utf-8"))
        except Exception:
            counts["?"] += 1
            continue
        cls = data.get("class") or data.get("wtm_class") or "?"
        if cls in counts:
            counts[cls] += 1
        else:
            counts["?"] += 1
    print(f"TOTAL={total}")
    for k, v in counts.items():
        print(f"CLASS_{k.upper()}={v}")
    return 0


def cmd_count_by_pattern(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    pattern = re.compile(args.regex)
    matched = 0
    total = 0
    for fp in iter_json_files(root):
        total += 1
        if pattern.search(fp.name):
            matched += 1
    print(f"TOTAL={total}")
    print(f"MATCHED={matched}")
    return 0


# ---- 数学 fixture 验证 (2026-11-21, per "完善自动测试脚本和mock项目，加入数学测试的内容") ----
# 与 _generate_math_fixtures.py 的参考实现互为 oracle: 生成器算 expected,
# 验证器独立重算不变量。任何一侧公式漂移都会被捕获。

def _m_dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def _m_norm(a):
    return math.sqrt(_m_dot(a, a))


def _m_close(a, b, tol):
    """递归比较数值/列表/字典, 误差 ≤ tol。None 只与 None 相等。"""
    if a is None or b is None:
        return a is None and b is None
    if isinstance(a, dict) and isinstance(b, dict):
        return set(a) == set(b) and all(_m_close(a[k], b[k], tol) for k in a)
    if isinstance(a, (list, tuple)) and isinstance(b, (list, tuple)):
        return len(a) == len(b) and all(_m_close(x, y, tol) for x, y in zip(a, b))
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return abs(a - b) <= tol
    return a == b


def _m_reflect(v, n):
    d = 2 * _m_dot(v, n)
    return [v[i] - d * n[i] for i in range(3)]


def _m_qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return [
        aw*bx + ax*bw + ay*bz - az*by,
        aw*by - ax*bz + ay*bw + az*bx,
        aw*bz + ax*by - ay*bx + az*bw,
        aw*bw - ax*bx - ay*by - az*bz,
    ]


def _m_qrotate(q, v):
    p = _m_qmul(q, [v[0], v[1], v[2], 0.0])
    qi = [-q[0], -q[1], -q[2], q[3]]
    return _m_qmul(p, qi)[:3]


def _m_qslerp(a, b, t):
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


def _m_mmul(a, b):
    return [[sum(a[i][k]*b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def _m_mtranspose(a):
    return [[a[j][i] for j in range(3)] for i in range(3)]


def _m_minverse(a):
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


def _m_intersect(a, b):
    lo = [max(a["min"][i], b["min"][i]) for i in range(3)]
    hi = [min(a["max"][i], b["max"][i]) for i in range(3)]
    if any(lo[i] > hi[i] for i in range(3)):
        return None
    return {"min": lo, "max": hi}


def _m_letterbox(rect, vb):
    rw = rect["width"] if rect["width"] > 0 else 1
    rh = rect["height"] if rect["height"] > 0 else 1
    s = max(vb["width"]/rw, vb["height"]/rh)
    return {"s": s, "ox": (rw - vb["width"]/s)/2, "oy": (rh - vb["height"]/s)/2}


def _m_client_to_world(client, rect, vb, vp):
    lb = _m_letterbox(rect, vb)
    sx = (client["clientX"] - rect["left"] - lb["ox"]) * lb["s"] + vb["x"]
    sy = (client["clientY"] - rect["top"] - lb["oy"]) * lb["s"] + vb["y"]
    zoom = vp["zoom"] if vp["zoom"] > 0 else 1
    return {"x": sx/zoom + vp["x"], "y": sy/zoom + vp["y"]}


def _m_world_to_client(world, rect, vb, vp):
    lb = _m_letterbox(rect, vb)
    sx = (world["x"] - vp["x"]) * vp["zoom"]
    sy = (world["y"] - vp["y"]) * vp["zoom"]
    return {"clientX": (sx - vb["x"])/lb["s"] + rect["left"] + lb["ox"],
            "clientY": (sy - vb["y"])/lb["s"] + rect["top"] + lb["oy"]}


def _m_recompute(family: str, inp: dict) -> dict:
    """按 family 独立重算 expected (验证器 oracle, 不读 fixture 的 expected)。"""
    if family == "vec3":
        v, n = inp["v"], inp["n"]
        r = _m_reflect(v, n)
        rr = _m_reflect(r, n)
        return {"reflect": r, "length_v": _m_norm(v), "length_reflect": _m_norm(r),
                "double_reflect": rr,
                "involution_error": _m_norm([rr[k]-v[k] for k in range(3)])}
    if family == "quat":
        if "t_values" in inp:  # slerp
            exp = {}
            for t in inp["t_values"]:
                q = _m_qslerp(inp["a"], inp["b"], t)
                exp[f"slerp_t{t}"] = q
                exp[f"norm_t{t}"] = math.sqrt(sum(x*x for x in q))
            return exp
        if "u" in inp:  # 保内积
            ru = _m_qrotate(inp["q"], inp["u"])
            rw = _m_qrotate(inp["q"], inp["w"])
            return {"dot_before": _m_dot(inp["u"], inp["w"]), "dot_after": _m_dot(ru, rw),
                    "rotated_u": ru, "rotated_w": rw}
        if "v" in inp:  # 保长度
            rv = _m_qrotate(inp["q"], inp["v"])
            return {"length_before": _m_norm(inp["v"]), "length_after": _m_norm(rv), "rotated": rv}
        q = inp["q"]  # 逆定义式
        qi = [-q[0], -q[1], -q[2], q[3]]
        return {"q_inverse": qi, "q_times_qinv": _m_qmul(q, qi)}
    if family == "mat3":
        if "b" in inp:  # 结合律
            a, b, c = inp["a"], inp["b"], inp["c"]
            return {"ab_times_c": _m_mmul(_m_mmul(a, b), c), "a_times_bc": _m_mmul(a, _m_mmul(b, c))}
        m = inp["m"]
        inv = _m_minverse(m)
        if inv is None:
            return {"inverse": None, "determinant": 0.0}
        # 超集重算: 验证器只比对 fixture.expected 中出现的键 (不同 case 键集不同)
        return {"inverse": inv, "inv_times_m": _m_mmul(inv, m), "m_times_inv": _m_mmul(m, inv),
                "transpose": _m_mtranspose(m), "double_transpose": _m_mtranspose(_m_mtranspose(m))}
    if family == "transform":
        def apply(q, t, v):
            r = _m_qrotate(q, v)
            return [r[i]+t[i] for i in range(3)]
        def compose(qa, ta_, qb, tb_):
            return _m_qmul(qa, qb), [x+y for x, y in zip(_m_qrotate(qa, tb_), ta_)]
        def invert(q, t):
            qi = [-q[0], -q[1], -q[2], q[3]]
            return qi, [-x for x in _m_qrotate(qi, t)]
        def apply_t(t, v):
            return apply(t["rotation"], t["translation"], v)
        def compose_t(a, b):
            q, t = compose(a["rotation"], a["translation"], b["rotation"], b["translation"])
            return {"rotation": q, "translation": t}
        if "c" in inp:  # 结合律 (3 变换)
            a, b, c, v = inp["a"], inp["b"], inp["c"], inp["v"]
            return {"lhs": apply_t(compose_t(compose_t(a, b), c), v),
                    "rhs": apply_t(compose_t(a, compose_t(b, c)), v)}
        if "a" in inp:  # 复合
            a, b, v = inp["a"], inp["b"], inp["v"]
            qc, tc = compose(a["rotation"], a["translation"], b["rotation"], b["translation"])
            return {"composed": {"rotation": qc, "translation": tc},
                    "sequential": apply(a["rotation"], a["translation"], apply(b["rotation"], b["translation"], v)),
                    "composed_apply": apply(qc, tc, v)}
        t = inp["t"]
        q, tr = t["rotation"], t["translation"]
        if "v" in inp:
            qinv, tinv = invert(q, tr)
            v = inp["v"]
            return {"inverse": {"rotation": qinv, "translation": tinv},
                    "roundtrip": apply(qinv, tinv, apply(q, tr, v)), "original": v,
                    "identity_apply": apply([0.0,0.0,0.0,1.0], [0.0,0.0,0.0], v),
                    "compose_identity": apply(*compose(q, tr, [0.0,0.0,0.0,1.0], [0.0,0.0,0.0]), v),
                    "original_apply": apply(q, tr, v)}
        qinv, tinv = invert(q, tr)
        qii, tii = invert(qinv, tinv)
        return {"double_inverse": {"rotation": qii, "translation": tii},
                "original": {"rotation": q, "translation": tr}}
    if family == "aabb":
        a = inp["a"]
        b = inp.get("b") or inp.get("d") or inp.get("e")
        iab = _m_intersect(a, b)
        exp = {"intersection": iab, "intersection_ab": iab, "a_intersect_b": iab,
               "b_intersect_a": _m_intersect(b, a),
               "overlaps_ab": iab is not None, "overlaps": iab is not None}
        if iab is not None:
            exp["intersect_again"] = _m_intersect(iab, iab)
        if "c" in inp:
            exp["overlaps_ac"] = _m_intersect(a, inp["c"]) is not None
            exp["intersection_ac"] = _m_intersect(a, inp["c"])
        if "d" in inp:
            exp["contained"] = inp["d"]
        if "e" in inp:
            exp["intersection"] = _m_intersect(a, inp["e"])
            exp["overlaps"] = _m_intersect(a, inp["e"]) is not None
        return exp
    if family == "coords":
        rect, vb, vp, client = inp["rect"], inp["viewBox"], inp["viewport"], inp["client"]
        world = _m_client_to_world(client, rect, vb, vp)
        back = _m_world_to_client(world, rect, vb, vp)
        return {"world": world, "roundtrip_client": back, "letterbox": _m_letterbox(rect, vb),
                "roundtrip_error": math.hypot(back["clientX"]-client["clientX"], back["clientY"]-client["clientY"])}
    raise ValueError(f"unknown math family: {family}")


def cmd_validate_math(args: argparse.Namespace) -> int:
    """验证 math fixture: 结构完整 + expected 与独立重算一致 (容差内)。

    mat3/transform 族按 case 名只比对 expected 中出现的键 (不同 case 的
    expected 键集不同), 避免超集误报。
    """
    root = Path(args.dir)
    total = 0
    failures: list[str] = []
    required = ("fixture_version", "module", "family", "case", "invariant", "tolerance", "input", "expected")
    for fp in iter_json_files(root):
        total += 1
        try:
            data = json.loads(fp.read_text(encoding="utf-8"))
        except Exception as e:
            failures.append(f"{fp.name}: JSON parse error: {e}")
            continue
        missing = [k for k in required if k not in data]
        if missing:
            failures.append(f"{fp.name}: missing fields {missing}")
            continue
        if data.get("module") != "math":
            failures.append(f"{fp.name}: module != math")
            continue
        family = data["family"]
        tol = float(data["tolerance"])
        try:
            recomputed = _m_recompute(family, data["input"])
        except Exception as e:
            failures.append(f"{fp.name}: recompute error: {e}")
            continue
        expected = data["expected"]
        for key, want in expected.items():
            if key not in recomputed:
                failures.append(f"{fp.name}: expected key '{key}' not recomputable for family={family} case={data['case']}")
                continue
            if not _m_close(want, recomputed[key], tol):
                failures.append(f"{fp.name}: key '{key}' mismatch (tol={tol}): fixture={want} recomputed={recomputed[key]}")
    print(f"TOTAL={total}")
    print(f"FAILURES={len(failures)}")
    for line in failures[:20]:
        print(f"MATH_FAIL={line}", file=sys.stderr)
    return 0 if not failures else 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Star Mock Project fixture validator")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p1 = sub.add_parser("validate-json", help="validate JSON syntax of all fixtures")
    p1.add_argument("dir")
    p1.set_defaults(func=cmd_validate_json)

    p2 = sub.add_parser("scan-secret", help="scan forbidden secret patterns (守门 #5)")
    p2.add_argument("dir")
    p2.set_defaults(func=cmd_scan_secret)

    p3 = sub.add_parser("require-field", help="each fixture must contain a literal field")
    p3.add_argument("dir")
    p3.add_argument("field")
    p3.set_defaults(func=cmd_require_field)

    p4 = sub.add_parser("count-classes", help="count W/T/M distribution (守门 #13)")
    p4.add_argument("dir")
    p4.set_defaults(func=cmd_count_classes)

    p5 = sub.add_parser("count-by-pattern", help="count files matching regex")
    p5.add_argument("dir")
    p5.add_argument("regex")
    p5.set_defaults(func=cmd_count_by_pattern)

    p6 = sub.add_parser("validate-math", help="recompute math invariants and compare expected (2026-11-21)")
    p6.add_argument("dir")
    p6.set_defaults(func=cmd_validate_math)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
