"""v0.68 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 57 次新事件触发)"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V068_ROW = """| **v0.68** | **2026-09-10 14:35 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标 + 守门 #12 v15 docs 同步饱和第 57 次新事件触发 仍允许)** | **§4 self-review fix v2: 4 项 findings 显式列 (per 9/10 14:30 JST 用户发令"review" PROACTIVELY 触发)**：**(1) commit `ed3b05b` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses): 3 files changed, 70 insertions(+), 7 deletions(-): (a) `AGENTS.md` §2.2 审批者 行 + §2.3 修订人 行 (2 changes): 改 "Mavis 接手**审核**通过 / Mavis 接手**审核** (per 8/27 07:16 反转 + 9/10 12:45 v0.62 反转, 真人代签流程全部取消, 改为 mavis 审核)"; (b) `crates/application/src/lib.rs` ApplicationError doc (4 lines updated): 改 `source_module (caller 传入, e.g. "domain-feedback" / "infrastructure")` + `source_kind (TitleCase: Validation / Policy / External / Internal)`, 跟 v0.65 修复后实际行为对齐; (c) `scripts/registry_test_tokio_attr.py` 2.5KB 幂等 Python 脚本 (本次 revert 留作 v0.69+ follow-up, 9 callsites 替换后 .await 处理不当, 修复 cost > benefit); **(2) 4 项 findings 显式列** (per 守门 #11 缺标比错标): **Finding #1 (Minor, Critical 文档)** — AGENTS.md §2.2/§2.3 仍标 "Mavis 接手代签" (本次修复); **Finding #2 (Minor)** — ApplicationError doc 仍说 "固定 application" + "lowercase kind" (本次修复); **Finding #3 (Minor, revert)** — InMemoryAdapterRegistry 测试用 Runtime::new().unwrap().block_on, regex 替换 .await 失败, 留 v0.69+ follow-up (当前 9/9 测试 pass); **Finding #4 (Minor, 不修)** — 5ffc9ad "fix(infrastructure): add pub mod registry" 重复 commit (守门 #1 禁回溯叙事, 提交历史不动); **(3) 跨 session 续做项**: (a) v0.69 = InMemoryAdapterRegistry 测试 #[tokio::test] refactor (Finding #3 follow-up); (b) v0.70 = v32 拍板激活 + 修订历史; (c) v0.71 = envoy 业务路由 + TLS 自动签发; **(4) 守门 #1 禁回溯叙事** — v0.1-v0.67 row 不动, v0.68 row 显式标 self-review fix v2; **(5) 触发**: 9/10 14:30 JST 用户发令"review" PROACTIVELY 触发 self-review; cargo check --workspace --lib 0 err; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.68-self-review-fix" in content:
        print("OK: v0.68 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.67\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.67 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V068_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.68 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
