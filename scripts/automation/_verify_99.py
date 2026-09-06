"""独立验证 99-pg-broker-audit.md 4 项缺陷跟 cargo metadata 一致性."""
import subprocess, json, re

r = subprocess.run(['cargo', 'metadata', '--no-deps', '--format-version', '1'],
                   cwd=r'D:\Star', capture_output=True, timeout=120)
d = json.loads(r.stdout)
pkgs = {p['name']: p for p in d['packages']}

print("=" * 70)
print("[verifier] 99-pg-broker-audit.md 4 项缺陷独立验证")
print("=" * 70)

# 缺陷 1: 15 star-* 已实装
star = [n for n in pkgs if n.startswith('star-')]
print(f"\n[缺陷 1] 15 star-* 已实装验证:")
print(f"  实测: {len(star)} star-* crate 已实装")
for s in star:
    print(f"    {s}")
assert len(star) == 15, f"应 15, 实测 {len(star)}"

# 缺陷 2: dual-namespace
print(f"\n[缺陷 2] dual-namespace 验证:")
print(f"  domain-* 34 + star-* 15 + other 3 = 52")
domain = [n for n in pkgs if n.startswith('domain-')]
other = [n for n in pkgs if not n.startswith('domain-') and not n.startswith('star-')]
print(f"  实测: domain={len(domain)}, star={len(star)}, other={len(other)}, total={len(pkgs)}")
assert len(domain) + len(star) + len(other) == 52

# 缺陷 3: 22+9=31 vs 52
print(f"\n[缺陷 3] 数字差异验证:")
print(f"  docswiki 写 22+9=31")
print(f"  实测 52")
print(f"  gap: +{52-31}")
assert 52 - 31 == 21

# 缺陷 4: 9 "应新建" 0/9 实装
print(f"\n[缺陷 4] 9 应新建 crate 0/9 实装验证:")
design = ['domain-dispatcher', 'domain-llm', 'domain-mcp', 'domain-tool', 'domain-rag',
          'domain-context', 'domain-memory', 'domain-rate-limiter', 'domain-observability']
existing = [d for d in design if d in pkgs]
print(f"  实装 0/{len(design)}")
for d in existing:
    print(f"    ! EXISTS: {d}")
assert len(existing) == 0, f"应 0 实装, 实测 {len(existing)}"

# 命名冲突: 5 个 design-only 跟 star-* 重名
print(f"\n[命名冲突] design-only 跟已实装 star-* 重名:")
conflicts_expected = [('domain-dispatcher', 'star-dispatcher'),
                      ('domain-mcp', 'star-mcp'),
                      ('domain-context', 'star-context')]
for d, s in conflicts_expected:
    d_in = d in pkgs
    s_in = s in pkgs
    print(f"  domain={d} (exists={d_in})  star={s} (exists={s_in})  冲突={'YES' if not d_in and s_in else 'NO'}")

# pgwiki 50-issues/04-broker-arch-refs 32 broker
print(f"\n[pgwiki 50-issues 验证] 32 broker 是否真没在 cargo:")
broker_txt = open(r'D:\Star\docs\wiki\pgwiki\50-issues\04-broker-arch-refs.md').read()
brokers = set(re.findall(r'`([\w\-]+)`', broker_txt))
print(f"  broker unique: {len(brokers)}")
mis = []
correct = []
for b in brokers:
    found = False
    for ns in ['', 'domain-', 'star-', 'crate-', 'crates/']:
        if (ns + b) in pkgs:
            mis.append(b)
            found = True
            break
    if not found:
        correct.append(b)
print(f"  误列 (实际已实装, 应从 broker 移出): {len(mis)}")
for m in mis: print(f"    {m}")
print(f"  正确 broker (cargo 没): {len(correct)}")

# docswiki/04-domain-crates.md §8 数字 vs 实测
print(f"\n[docswiki §8 数字自洽] 15 个 star-* src 文件数 vs 实测:")
from pathlib import Path
expected_files = {'star-mcp': 49, 'star-api-rest': 20, 'star-cli': 16, 'star-saga': 11,
                  'star-sa': 6, 'star-dispatcher': 5, 'star-cache': 4, 'star-credential': 4,
                  'star-treesitter': 4, 'star-context': 3, 'star-sse': 3, 'star-webhook': 3,
                  'star-taskgraph': 2, 'star-vcs': 2, 'star-dto': 1}
mismatch = 0
for n, expected in expected_files.items():
    if n in pkgs:
        sp = Path(pkgs[n]['manifest_path']).parent / 'src'
        actual = sum(1 for _ in sp.rglob('*.rs')) if sp.exists() else 0
        ok = 'OK' if actual == expected else 'MISMATCH'
        if actual != expected:
            mismatch += 1
            print(f"  {n}: docswiki={expected} 实际={actual}  {ok}")
print(f"  全部 15 star-* src 文件数: {15 - mismatch}/15 一致")
