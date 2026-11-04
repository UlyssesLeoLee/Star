# @cypher schema=1 source_sha256=037383fe10b87c0a34e604c502eac1f98b87694fddd86660ada43015d0b9f0d1
# MERGE (self:File {path:"maintenance/update-backend-k3s.ps1"})
# MERGE (entry:Symbol {id:"maintenance/update-backend-k3s.ps1::entry",kind:"script_entry"})
# MERGE (self)-[:DEFINES]->(entry)
# MERGE (write_error:Symbol {id:"maintenance/update-backend-k3s.ps1::System.Console.Error.WriteLine",kind:"external_method"})
# MERGE (entry)-[:CALLS]->(write_error)
# @endcypher

[Console]::Error.WriteLine('ERROR: This legacy backend updater is retired and makes no changes.')
[Console]::Error.WriteLine('For the supported Canvas deployment path, review docs/deployment/star-canvas-services-deploy.md.')
[Console]::Error.WriteLine('That path applies only to an already provisioned and reachable K3s context; it does not create a Multipass instance.')
[Console]::Error.WriteLine('No general Star API image build or deployment command is supported by this entrypoint.')
exit 2
