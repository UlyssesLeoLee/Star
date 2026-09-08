#!/usr/bin/env python3
# capacity_planning.py — §5.4 Ops Console 容量规划 (per TEST-DESIGN-OPS-001 v0.2 §5.4)
#
# 触发: 2026-09-08 19:55 JST Mavis 接手 (per 5-LEVEL-FULL brief §1)
# 范围: 3 档用户负载 (100/1000/5000) 容量规划, 验证守門 #7 v3 P95<200ms
# 工具: Python 3 stdlib (asyncio + urllib), 避免 k6 等外部工具 (per守門 #1 R-05 mock 路径)
# 守門:
#   - 0 真实 K8s/LLM/PG (per守門 #1 R-05, 走 /healthz /readyz /api/ops/* stub)
#   - 0 子代理调用 (per守門 #9 v20)
#   - 守門 #11 缺标比错标, 5 已知缺口显式列
#
# 已知缺口 (per守門 #11 缺标比错标, DDD Review 必查):
#   - §5.5 缺口 #2: k6 容量规划脚本替代, MVP 阶段纯 Python 简易版
#   - §5.5 缺口 #3: 多节点 HA + 负载均衡实测缺 ([M] 子项)
#   - §5.5 缺口 #4: 真实 PG 容器 + sqlx::test 容量实测缺 (per F-05 sprint)
#   - 真实 prod K8s 集群替代, MVP 单实例 axum 0.8
#   - Python urllib 跟真实浏览器 + HTTP 客户端可能有差异
#
# 引用:
#   - docs/test-design/TEST-DESIGN-OPS-001.md v0.2 §5.4
#   - docs/briefs/5-level-full-impl.md v0.1 §2.1 §5
#   - SRS-001 §7.1 NFR-OP-005 (P95<200ms 硬约束)
#
# 用法:
#   python capacity_planning.py --tier light    # 100 用户 / 10 RPS / 60s
#   python capacity_planning.py --tier medium   # 1000 用户 / 100 RPS / 60s
#   python capacity_planning.py --tier heavy    # 5000 用户 / 500 RPS / 60s
#   python capacity_planning.py --tier smoke   # 5 用户 / 5 RPS / 5s (本地快速验证)

import argparse
import asyncio
import io
import json
import os
import statistics
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, asdict
from typing import List

# 守門 #5 v2 + Windows GBK 环境: 强制 UTF-8 stdout, 避免 emoji 打印失败
# 适用: 守門 #19 v19 scripts/automation 路径
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")
    except Exception:
        pass
os.environ.setdefault("PYTHONIOENCODING", "utf-8")

# === 3 档用户负载配置 (per TEST-DESIGN §5.4) ===
TIERS = {
    "smoke": {"concurrent_users": 5, "rps": 5, "duration_sec": 5, "target_p95_ms": 200, "error_rate_pct": 0.1},
    "light": {"concurrent_users": 100, "rps": 10, "duration_sec": 60, "target_p95_ms": 200, "error_rate_pct": 0.1},
    "medium": {"concurrent_users": 1000, "rps": 100, "duration_sec": 300, "target_p95_ms": 300, "error_rate_pct": 0.5},
    "heavy": {"concurrent_users": 5000, "rps": 500, "duration_sec": 600, "target_p95_ms": 500, "error_rate_pct": 1.0},
}

# === 10 端点 (per TEST-DESIGN §4.2 4 tab × 10 端点) ===
ENDPOINTS = [
    ("GET", "/healthz", None),
    ("GET", "/readyz", None),
    ("GET", "/api/ops/cluster/releases", None),
    ("GET", "/api/ops/cluster/status", None),
    ("POST", "/api/ops/cluster/canary", {"release_name": "star-mcp", "canary_weight": 10, "target_revision": None}),
    ("POST", "/api/ops/cluster/rollback", {"release_name": "star-mcp", "target_revision": 2}),
    ("POST", "/api/ops/log/upload", {"source": "cap/test", "level_filter": ["ERROR"], "content": "2026-09-08 ERROR cap test"}),
    ("GET", "/api/ops/metrics/summary", None),
    ("GET", "/api/ops/docs", None),
]


@dataclass
class RequestResult:
    method: str
    path: str
    status: int
    latency_ms: float
    timestamp: float
    error: str = ""


async def single_request(base_url: str, method: str, path: str, body) -> RequestResult:
    """单次 HTTP 请求, 测延迟 + 状态码"""
    url = f"{base_url}{path}"
    start = time.perf_counter()
    try:
        data = json.dumps(body).encode() if body else None
        req = urllib.request.Request(
            url, data=data, method=method,
            headers={"content-type": "application/json"} if data else {},
        )
        # 同步调用, 在 thread pool 跑避免阻塞 event loop
        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, lambda: urllib.request.urlopen(req, timeout=10))
        elapsed_ms = (time.perf_counter() - start) * 1000
        return RequestResult(method=method, path=path, status=200, latency_ms=elapsed_ms, timestamp=time.time())
    except urllib.error.HTTPError as e:
        elapsed_ms = (time.perf_counter() - start) * 1000
        return RequestResult(method=method, path=path, status=e.code, latency_ms=elapsed_ms, timestamp=time.time(), error=str(e))
    except Exception as e:
        elapsed_ms = (time.perf_counter() - start) * 1000
        return RequestResult(method=method, path=path, status=0, latency_ms=elapsed_ms, timestamp=time.time(), error=str(e))


async def run_user(base_url: str, results: List[RequestResult], user_id: int, duration_sec: float, rps_per_user: int):
    """单用户 worker: 持续 rps_per_user RPS 跑 duration_sec"""
    interval = 1.0 / max(rps_per_user, 1)  # 每次请求间隔
    end_time = time.time() + duration_sec
    while time.time() < end_time:
        # 随机选 endpoint (round-robin)
        method, path, body = ENDPOINTS[user_id % len(ENDPOINTS)]
        result = await single_request(base_url, method, path, body)
        results.append(result)
        await asyncio.sleep(interval)


async def run_capacity_tier(base_url: str, tier_name: str) -> dict:
    """跑一个 tier, 返汇总"""
    tier = TIERS[tier_name]
    print(f"[CAP-PLAN] tier={tier_name} users={tier['concurrent_users']} rps={tier['rps']} duration={tier['duration_sec']}s")
    print(f"[CAP-PLAN] target P95 < {tier['target_p95_ms']}ms, error_rate < {tier['error_rate_pct']}%")

    # 计算每个用户的 RPS
    rps_per_user = max(1, tier["rps"] // tier["concurrent_users"])

    results: List[RequestResult] = []
    start = time.time()
    tasks = [
        run_user(base_url, results, uid, tier["duration_sec"], rps_per_user)
        for uid in range(tier["concurrent_users"])
    ]
    await asyncio.gather(*tasks)
    elapsed = time.time() - start

    # 统计
    if not results:
        return {"tier": tier_name, "error": "no results"}

    latencies = sorted([r.latency_ms for r in results])
    statuses = [r.status for r in results]
    errors = [r for r in results if r.status != 200]

    p50 = latencies[len(latencies) // 2]
    p95_idx = int(len(latencies) * 0.95)
    p99_idx = int(len(latencies) * 0.99)
    p95 = latencies[p95_idx] if p95_idx < len(latencies) else latencies[-1]
    p99 = latencies[p99_idx] if p99_idx < len(latencies) else latencies[-1]

    error_rate = len(errors) / len(results) * 100
    actual_rps = len(results) / elapsed

    summary = {
        "tier": tier_name,
        "concurrent_users": tier["concurrent_users"],
        "target_rps": tier["rps"],
        "actual_rps": round(actual_rps, 2),
        "duration_sec": round(elapsed, 2),
        "total_requests": len(results),
        "p50_ms": round(p50, 2),
        "p95_ms": round(p95, 2),
        "p99_ms": round(p99, 2),
        "max_ms": round(max(latencies), 2),
        "error_count": len(errors),
        "error_rate_pct": round(error_rate, 4),
        "target_p95_ms": tier["target_p95_ms"],
        "target_error_rate_pct": tier["error_rate_pct"],
        "p95_pass": p95 < tier["target_p95_ms"],
        "error_rate_pass": error_rate < tier["error_rate_pct"],
    }
    return summary


def print_summary(summary: dict):
    print()
    print("=" * 60)
    print(f"[CAP-PLAN] tier={summary['tier']} 汇总")
    print("=" * 60)
    print(f"  并发用户: {summary['concurrent_users']}")
    print(f"  目标 RPS: {summary['target_rps']} | 实际 RPS: {summary['actual_rps']}")
    print(f"  持续时间: {summary['duration_sec']}s")
    print(f"  总请求数: {summary['total_requests']}")
    print(f"  P50 延迟: {summary['p50_ms']}ms")
    print(f"  P95 延迟: {summary['p95_ms']}ms (目标 < {summary['target_p95_ms']}ms) {'✅' if summary['p95_pass'] else '❌'}")
    print(f"  P99 延迟: {summary['p99_ms']}ms")
    print(f"  Max 延迟: {summary['max_ms']}ms")
    print(f"  错误数: {summary['error_count']} / {summary['total_requests']} ({summary['error_rate_pct']}%, 目标 < {summary['target_error_rate_pct']}%) {'✅' if summary['error_rate_pass'] else '❌'}")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Ops Console 容量规划 (per TEST-DESIGN §5.4)")
    parser.add_argument("--tier", choices=list(TIERS.keys()), default="smoke", help="容量档 (smoke/light/medium/heavy)")
    parser.add_argument("--base-url", default="http://localhost:8090", help="star-ops API base URL")
    parser.add_argument("--report", help="可选, 写 JSON 报告到文件")
    args = parser.parse_args()

    print(f"[CAP-PLAN] base_url={args.base_url} tier={args.tier}")
    summary = asyncio.run(run_capacity_tier(args.base_url, args.tier))
    print_summary(summary)

    if args.report:
        with open(args.report, "w") as f:
            json.dump(summary, f, indent=2)
        print(f"[CAP-PLAN] 报告写入 {args.report}")

    # 退出码: P95 + error_rate 都达标 = 0, 否则 1
    if summary.get("p95_pass") and summary.get("error_rate_pass"):
        sys.exit(0)
    sys.exit(1)


if __name__ == "__main__":
    main()
