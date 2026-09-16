"""v0.71 envoy configmap 校验: 5 业务路由 + 3 集群 + 2 listener"""
import re
from pathlib import Path

CM = Path("deploy/k3s-local/envoy/configmap.yaml")

def main():
    cm = CM.read_text(encoding="utf-8")
    # Count routes (prefix: "/X/")
    routes = re.findall(r'prefix: "([^"]+)"', cm)
    # Count clusters (- name: name_cluster)
    clusters = re.findall(r'- name: (\w+_cluster)', cm)
    # Count listeners (- name: name_listener)
    listeners = re.findall(r'- name: (\w+_listener)', cm)
    print(f"业务路由 ({len(routes)}):")
    for r in routes:
        print(f"  {r}")
    print(f"集群 ({len(clusters)}):")
    for c in clusters:
        print(f"  {c}")
    print(f"listener ({len(listeners)}):")
    for l in listeners:
        print(f"  {l}")
    # Validation
    expected_routes = ["/api/v1/", "/oauth/", "/mcp/", "/ws/", "/actuator/"]
    expected_clusters = ["sds_cluster", "star_api_rest_cluster", "star_mcp_cluster", "star_ops_cluster"]
    expected_listeners = ["readiness_listener", "star_https_listener"]
    # Filter only the 业务路由 (5 expected: /api/v1/, /oauth/, /mcp/, /ws/, /actuator/)
    actual_route_prefixes = sorted(r for r in routes if r.startswith("/") and r != "/" and r not in ["/ready", "/healthz"])
    print(f"\\n实际 业务路由: {actual_route_prefixes}")
    if sorted(expected_routes) == actual_route_prefixes:
        print("OK: 5 业务路由匹配")
    else:
        print(f"FAIL: 期望 {sorted(expected_routes)} vs 实际 {actual_route_prefixes}")
    if sorted(expected_clusters) == sorted(clusters):
        print("OK: 3 集群匹配")
    else:
        print(f"FAIL: 期望 {sorted(expected_clusters)} vs 实际 {sorted(clusters)}")
    if sorted(expected_listeners) == sorted(listeners):
        print("OK: 2 listener 匹配")
    else:
        print(f"FAIL: 期望 {sorted(expected_listeners)} vs 实际 {sorted(listeners)}")

if __name__ == "__main__":
    main()
