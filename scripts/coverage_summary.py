#!/usr/bin/env python3
"""
ULYS-154 PR-1: cargo-tarpaulin cobertura.xml → per-crate baseline summary

tarpaulin 输出的 cobertura.xml 是标准 Cobertura coverage report 格式。
每个 <package name="..."> 对应一个 crate (tarpaulin 内部按 workspace member 拆)。
每个 <class> 对应一个源文件, line-rate 是该文件的行覆盖率 (0.0~1.0)。

输出格式 (stdout):
    crate-name-1         lines=120  covered=96   rate=80.00%
    crate-name-2         lines=340  covered=312  rate=91.76%
    ...
    ── workspace total ──
    lines=12345  covered=7890  rate=63.91%

stderr 打印解析警告 (例如 class 没 name 属性)。

用法:
    python3 scripts/coverage_summary.py coverage/cobertura.xml > coverage/baseline.txt

为什么用 Python 而不是 jq/yq:
- cobertura.xml 解析需要 namespace 处理 + 嵌套结构, jq 不适合
- 团队约定 (AGENTS.md §6.2 守门 #2): 测试/工具脚本优先 Python
- 不引外部依赖, 只用 stdlib (xml.etree)
"""
from __future__ import annotations
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path


def parse_cobertura(path: Path) -> dict[str, tuple[int, int]]:
    """
    返回 {crate_name: (lines, covered)} dict。
    line-rate 公式: rate = covered_lines / total_lines
    total_lines = ceil(class['line-rate'] * class['lines-valid']) —— 这里直接读 lines-valid 更准确。
    """
    try:
        tree = ET.parse(path)
    except ET.ParseError as e:
        print(f"::warning::cobertura.xml parse failed: {e}", file=sys.stderr)
        return {}

    root = tree.getroot()
    pkg_lines: dict[str, int] = defaultdict(int)
    pkg_covered: dict[str, int] = defaultdict(int)

    # Cobertura 1.x 没有 namespace, 但部分 tarpaulin 版本会加 xmlns
    # strip namespace 兼容
    for elem in root.iter():
        if "}" in elem.tag:
            elem.tag = elem.tag.split("}", 1)[1]

    for pkg in root.findall(".//package"):
        name = pkg.attrib.get("name", "").strip()
        if not name:
            print("::warning::<package> 无 name 属性, 跳过", file=sys.stderr)
            continue
        # tarpaulin 把 <package name="..."> 直接用 crate 名 (e.g. star-context)
        for cls in pkg.findall("classes/class"):
            lv_str = cls.attrib.get("lines-valid", "")
            lc_str = cls.attrib.get("lines-covered", "")
            if lv_str and lc_str:
                lv = int(lv_str)
                lc = int(lc_str)
            else:
                # 回退路径: 某些 tarpaulin 版本只给 line-rate + <lines> 子元素
                # 从 <line hits="0|1"/> 计数: 每行算 1, hits>0 算 covered
                lv = 0
                lc = 0
                for line in cls.findall("lines/line"):
                    lv += 1
                    if line.attrib.get("hits", "0") not in ("0", ""):
                        lc += 1
            pkg_lines[name] += lv
            pkg_covered[name] += lc

    return {k: (pkg_lines[k], pkg_covered[k]) for k in pkg_lines}


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <cobertura.xml>", file=sys.stderr)
        return 2
    src = Path(sys.argv[1])
    if not src.exists():
        print(f"::warning::{src} 不存在 (tarpaulin 可能没跑成功)", file=sys.stderr)
        # 输出空 baseline 占位
        print("(no coverage data — tarpaulin did not produce cobertura.xml)")
        print("── workspace total ──")
        print("lines=0  covered=0  rate=N/A")
        return 1

    data = parse_cobertura(src)
    if not data:
        print("(no <package> entries parsed — 检查 cobertura.xml 是否为有效 tarpaulin 输出)",
              file=sys.stderr)
        print("── workspace total ──")
        print("lines=0  covered=0  rate=N/A")
        return 1

    # 按 crate 名排序, 数字列右对齐
    total_lines = 0
    total_covered = 0
    rows = []
    name_w = max((len(k) for k in data), default=10)
    for name in sorted(data):
        lines, covered = data[name]
        rate = (covered / lines * 100) if lines else 0.0
        rows.append(f"{name:<{name_w}}  lines={lines:<6}  covered={covered:<6}  rate={rate:6.2f}%")
        total_lines += lines
        total_covered += covered

    for r in rows:
        print(r)

    overall = (total_covered / total_lines * 100) if total_lines else 0.0
    print("── workspace total ──")
    print(f"lines={total_lines:<6}  covered={total_covered:<6}  rate={overall:6.2f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main())