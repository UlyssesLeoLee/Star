import json, subprocess
r = subprocess.run(['cargo', 'metadata', '--no-deps', '--format-version', '1'],
                   cwd=r'D:\Star', capture_output=True, timeout=120)
d = json.loads(r.stdout)
pkgs = d['packages']
g = {'domain': 0, 'star': 0, 'other': 0}
star_files = {}
for p in pkgs:
    n = p['name']
    if n.startswith('domain-'):
        g['domain'] += 1
    elif n.startswith('star-'):
        g['star'] += 1
        # count src
        sp = __import__('pathlib').Path(p['manifest_path']).parent / 'src'
        star_files[n] = sum(1 for _ in sp.rglob('*.rs')) if sp.exists() else 0
    else:
        g['other'] += 1
print(f'total={len(pkgs)} domain={g["domain"]} star={g["star"]} other={g["other"]}')
print('star src files:')
for k, v in sorted(star_files.items(), key=lambda x: -x[1]):
    print(f'  {k}: {v}')
