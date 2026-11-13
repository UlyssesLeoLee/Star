@echo off
REM @cypher schema=1 source_sha256=40fe5842288e31659973f3d5aca90c3a5b3e449d0e9b0cf57cec352987570d46
REM MERGE (self:File {path:"maintenance/start-k3s-backend.bat"})
REM MERGE (entry:Symbol {id:"maintenance/start-k3s-backend.bat::entry",kind:"script_entry"})
REM MERGE (self)-[:DEFINES]->(entry)
REM @endcypher

echo ERROR: This legacy K3s start entrypoint is retired and makes no changes.
echo It does not start or create a virtual machine or cluster.
echo If an already provisioned cluster is reachable, review:
echo   docs\deployment\star-canvas-services-deploy.md
echo No supported Star-managed resume operation for a stopped guest is provided here.
exit /b 2
