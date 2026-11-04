@echo off
REM @cypher schema=1 source_sha256=30478ed6b6d88766d6c73656edb4078eae2bd5365dde9508eae1234d6eb0e404
REM MERGE (self:File {path:"maintenance/update-backend-k3s.bat"})
REM MERGE (entry:Symbol {id:"maintenance/update-backend-k3s.bat::entry",kind:"script_entry"})
REM MERGE (self)-[:DEFINES]->(entry)
REM @endcypher

echo ERROR: This legacy backend updater is retired and makes no changes.
echo For the supported Canvas deployment path, review:
echo   docs\deployment\star-canvas-services-deploy.md
echo That path requires an already provisioned and reachable K3s context.
exit /b 2
