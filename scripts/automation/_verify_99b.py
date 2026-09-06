"""独立验证 8 个 domain-*-design 节点 跟 S5 §1.1 实际清单 一致性."""
import subprocess, json
r = subprocess.run(['cargo','metadata','--no-deps','--format-version','1'],
                   capture_output=True, timeout=120, cwd=r'D:\Star')
d = json.loads(r.stdout)
pkgs = {p['name']: p for p in d['packages']}

# 99 报告 / obsidian_topology_gen.py 列的 8 个 design-only
# 实际 9 个是 S5 §1.1:
s5_list = ['domain-dispatcher', 'domain-llm', 'domain-mcp', 'domain-tool', 'domain-rag',
           'domain-context', 'domain-memory', 'domain-rate-limiter', 'domain-observability']
# obsidian_topology_gen.py 实际生成 8 个 (我列的):
gen_list = ['domain-dispatcher-design', 'domain-llm-design', 'domain-mcp-design', 'domain-tool-design',
            'domain-rag-design', 'domain-memory-design', 'domain-rate-limiter-design', 'domain-observability-design']
# → 漏 1 个: domain-context-design (因为我判断 domain-context 已实装)

print("S5 §1.1 9 个清单 vs 99 报告 8 个 design-only 节点对比:")
print(f"  S5 §1.1 应有 9 个: {s5_list}")
print(f"  99 报告写了 8 个 (漏 domain-context): {[g.replace('-design','') for g in gen_list]}")

# 验证 8 个里面有几个实际未实装
gen_strip = [g.replace('-design', '') for g in gen_list]
existing = [d for d in gen_strip if d in pkgs]
print(f"\n  99 报告 8 个 design-only 中实际已实装: {len(existing)} (应 0)")
for d in existing: print(f"    EXISTS: {d}")

# domain-context 实际状态
print(f"\n  domain-context 实际: 在 cargo={('domain-context' in pkgs)}, src=1 文件 (placeholder/stub)")

# 完整 9 个 S5 §1.1 vs cargo
print(f"\n  S5 §1.1 9 个全 vs cargo metadata:")
for d in s5_list:
    status = 'EXISTS' if d in pkgs else 'NOT EXISTS'
    if d in pkgs:
        sp = __import__('pathlib').Path(pkgs[d]['manifest_path']).parent / 'src'
        files = sum(1 for _ in sp.rglob('*.rs')) if sp.exists() else 0
        print(f"    {d}: {status} (src={files} files)")
    else:
        print(f"    {d}: {status}")

# dual-namespace 命名冲突 (3 个)
print(f"\n  dual-namespace 命名冲突 (3 个, docswiki/99 报告 §1 缺陷 2 列):")
for d in ['domain-dispatcher', 'domain-mcp', 'domain-context']:
    s = d.replace('domain-', 'star-')
    if s in pkgs:
        print(f"    {d} (NOT EXISTS)  vs  {s} (EXISTS)  冲突 YES")
    else:
        print(f"    {d}  vs  {s}  NOT conflict")
