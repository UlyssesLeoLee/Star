#!/usr/bin/env bash
# port_forward_all.sh — 4 canvas service port-forward to Windows localhost
# Bypass k3s CNI iptables bug (envoy pod can't route to ClusterIP/pod IP)
# Direct port-forward to each service works reliably.

set -e
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

# Kill any existing port-forwards
pkill -9 -f "port-forward.*-n star-system" 2>/dev/null || true
sleep 2

# Wait for any in-flight restart
sleep 1

echo "Starting 4 port-forwards via python (detached)..."
python3 - <<'PYEOF'
import subprocess
services = [
    ('canvas-game', 12080, 8084),
    ('canvas-engine', 13080, 8080),
    ('domain-canvas', 14080, 8081),
    ('canvas-realtime', 15080, 8082),
    ('envoy-edge', 11080, 10080),
]
for name, port, dest in services:
    log = f"C:\\Users\\leo19\\AppData\\Local\\Temp\\pf-{name}.log"
    proc = subprocess.Popen(
        ['wsl', '-d', 'Ubuntu', '--', 'bash', '-c',
         f'export KUBECONFIG=/etc/rancher/k3s/k3s.yaml; '
         f'exec sudo /usr/local/bin/k3s kubectl port-forward --address=0.0.0.0 '
         f'-n star-system svc/{name} {port}:{dest}'],
        stdin=subprocess.DEVNULL,
        stdout=open(log, 'wb'),
        stderr=subprocess.STDOUT,
        creationflags=0x00000008 | 0x00000020,  # DETACHED_PROCESS + CREATE_NEW_PROCESS_GROUP
    )
    print(f'  {name:18s} Windows localhost:{port}  (k3s {name}:{dest}) pid={proc.pid}')
PYEOF

sleep 10
echo ""
echo "Verify:"
wsl -d Ubuntu -- bash -c 'ss -tlnp 2>/dev/null | grep -E ":(11|12|13|14|15)080\b" | awk "{print \$4, \$6}"'
echo ""
echo "Browser URLs (Windows):"
echo "  http://localhost:12080/  (canvas-game main, /api/v1/gameplay JSON)"
echo "  http://localhost:13080/  (canvas-engine)"
echo "  http://localhost:14080/  (domain-canvas)"
echo "  http://localhost:15080/  (canvas-realtime)"
echo "  http://localhost:11080/  (envoy, partial: 503 due to k3s CNI bug)"
