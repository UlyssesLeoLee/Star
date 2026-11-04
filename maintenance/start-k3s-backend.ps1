# @cypher schema=1 source_sha256=d933b3c99e9d2d377e5050741b2ec33b8a98e1378568a718fed0a83e4364ea46
# MERGE (self:File {path:"maintenance/start-k3s-backend.ps1"})
# MERGE (entry:Symbol {id:"maintenance/start-k3s-backend.ps1::entry",kind:"script_entry"})
# MERGE (self)-[:DEFINES]->(entry)
# MERGE (write_error:Symbol {id:"maintenance/start-k3s-backend.ps1::System.Console.Error.WriteLine",kind:"external_method"})
# MERGE (entry)-[:CALLS]->(write_error)
# @endcypher

[Console]::Error.WriteLine('ERROR: This legacy K3s start entrypoint is retired and makes no changes.')
[Console]::Error.WriteLine('It does not start, create, stop, or modify a virtual machine or cluster.')
[Console]::Error.WriteLine('The Multipass provisioning flow is not a start-existing or recovery operation; do not use it as a substitute.')
[Console]::Error.WriteLine('If an already provisioned cluster is reachable, review docs/deployment/star-canvas-services-deploy.md for the Canvas deployment path.')
[Console]::Error.WriteLine('A stopped or unreachable guest requires a Star-managed infrastructure lifecycle operation that is not available from this legacy entrypoint.')
exit 2
