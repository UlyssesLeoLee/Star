/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx", type:"file", language:"tsx"}),
  (page:Function {name:"GroupWorkspacePage", type:"function", signature:"GroupWorkspacePage(props: PageProps): JSX.Element", visibility:"public", complexity:"complex"}),
  (apps:Variable {name:"GROUP_APPS", type:"variable", language:"typescript"}),
  (pluginPreview:Variable {name:"PLUGIN_PREVIEW", type:"variable", language:"typescript"}),
  (visibleGroupApps:Variable {name:"visibleGroupApps", type:"variable", language:"typescript"}),
  (selectedPlugin:Variable {name:"selectedPlugin", type:"variable", language:"typescript"}),
  (statusOptions:Variable {name:"STATUS_OPTIONS", type:"variable", language:"typescript"}),
  (appType:Variable {name:"AppId", type:"variable", language:"typescript"}),
  (scopeType:Variable {name:"ChatScope", type:"variable", language:"typescript"}),
  (pendingCanvasTask:Variable {name:"PendingCanvasTaskCommand", type:"variable", language:"typescript"}),
  (pendingCanvasBootstrap:Class {name:"PendingCanvasBootstrapCommand", type:"class", language:"typescript", visibility:"private"}),
  (pendingCanvasElement:Class {name:"PendingCanvasElementCommand", type:"class", language:"typescript", visibility:"private"}),
  (pendingCanvasElementMove:Class {name:"PendingCanvasElementMoveCommand", type:"class", language:"typescript", visibility:"private"}),
  (pendingCanvasElementDelete:Class {name:"PendingCanvasElementDeleteCommand", type:"class", language:"typescript", visibility:"private"}),
  (isAppId:Function {name:"isAppId", type:"function", signature:"isAppId(value: string | null): value is AppId", visibility:"private", complexity:"simple"}),
  (enabledPlugins:Variable {name:"enabledPlugins", type:"variable", language:"typescript"}),
  (groupCanvas:Variable {name:"groupCanvas", type:"variable", language:"typescript"}),
  (groupCanvases:Variable {name:"groupCanvases", type:"variable", language:"typescript"}),
  (requestedCanvasId:Variable {name:"requestedCanvasId", type:"variable", language:"typescript"}),
  (canvasLinkError:Variable {name:"canvasLinkError", type:"variable", language:"typescript"}),
  (setCanvasLinkError:Variable {name:"setCanvasLinkError", type:"variable", language:"typescript"}),
  (canvasTaskTitle:Variable {name:"canvasTaskTitle", type:"variable", language:"typescript"}),
  (canvasTaskDescription:Variable {name:"canvasTaskDescription", type:"variable", language:"typescript"}),
  (canvasTaskError:Variable {name:"canvasTaskError", type:"variable", language:"typescript"}),
  (createdCanvasTaskId:Variable {name:"createdCanvasTaskId", type:"variable", language:"typescript"}),
  (canvasTaskPending:Variable {name:"canvasTaskPending", type:"variable", language:"typescript"}),
  (canvasBootstrapPending:Variable {name:"canvasBootstrapPending", type:"variable", language:"typescript"}),
  (canvasBootstrapError:Variable {name:"canvasBootstrapError", type:"variable", language:"typescript"}),
  (canvasBootstrapCommands:Variable {name:"canvasBootstrapCommands", type:"variable", language:"typescript"}),
  (canvasTaskCommands:Variable {name:"canvasTaskCommands", type:"variable", language:"typescript"}),
  (pendingCanvasDocumentDraft:Variable {name:"pendingCanvasDocumentDraft", type:"variable", language:"typescript"}),
  (selectedCanvasElementIds:Variable {name:"selectedCanvasElementIds", type:"variable", language:"typescript"}),
  (selectedCanvasFrameId:Variable {name:"selectedCanvasFrameId", type:"variable", language:"typescript"}),
  (selectedCanvasConnectorId:Variable {name:"selectedCanvasConnectorId", type:"variable", language:"typescript"}),
  (canvasDocumentSavePending:Variable {name:"canvasDocumentSavePending", type:"variable", language:"typescript"}),
  (canvasDocumentError:Variable {name:"canvasDocumentError", type:"variable", language:"typescript"}),
  (canvasDocumentSaved:Variable {name:"canvasDocumentSaved", type:"variable", language:"typescript"}),
  (canvasDocumentCommand:Variable {name:"canvasDocumentCommand", type:"variable", language:"typescript"}),
  (pendingCanvasDocument:Class {name:"PendingCanvasDocumentCommand", type:"class", language:"typescript", visibility:"private"}),
  (canvasDocumentDraft:Class {name:"CanvasDocumentDraft", type:"class", language:"typescript", visibility:"private"}),
  (pendingLifecycleCommand:Class {name:"PendingLifecycleCommand", type:"class", language:"typescript", visibility:"private"}),
  (groupApiGeneration:Variable {name:"groupApiGeneration", type:"variable", language:"typescript"}),
  (projectionRefreshKey:Variable {name:"projectionRefreshKey", type:"variable", language:"typescript"}),
  (showCanvasTaskForm:Variable {name:"showCanvasTaskForm", type:"variable", language:"typescript"}),
  (selectedExistingWorkItemId:Variable {name:"selectedExistingWorkItemId", type:"variable", language:"typescript"}),
  (canvasElementLinkPending:Variable {name:"canvasElementLinkPending", type:"variable", language:"typescript"}),
  (canvasElementLinkError:Variable {name:"canvasElementLinkError", type:"variable", language:"typescript"}),
  (linkedCanvasWorkItemId:Variable {name:"linkedCanvasWorkItemId", type:"variable", language:"typescript"}),
  (canvasElementLinkCommands:Variable {name:"canvasElementLinkCommands", type:"variable", language:"typescript"}),
  (canvasElementMoveCommands:Variable {name:"canvasElementMoveCommands", type:"variable", language:"typescript"}),
  (lifecycleCommands:Variable {name:"lifecycleCommands", type:"variable", language:"typescript"}),
  (pendingLifecycleWorkItemId:Variable {name:"pendingLifecycleWorkItemId", type:"variable", language:"typescript"}),
  (lifecycleError:Variable {name:"lifecycleError", type:"variable", language:"typescript"}),
  (canvasElementMovePending:Variable {name:"canvasElementMovePending", type:"variable", language:"typescript"}),
  (canvasElementMovePendingRef:Variable {name:"canvasElementMovePendingRef", type:"variable", language:"typescript"}),
  (canvasElementMoveError:Variable {name:"canvasElementMoveError", type:"variable", language:"typescript"}),
  (canvasElementMoveSaved:Variable {name:"canvasElementMoveSaved", type:"variable", language:"typescript"}),
  (canvasElementDeleteCommands:Variable {name:"canvasElementDeleteCommands", type:"variable", language:"typescript"}),
  (canvasElementDeletePending:Variable {name:"canvasElementDeletePending", type:"variable", language:"typescript"}),
  (canvasElementDeletePendingRef:Variable {name:"canvasElementDeletePendingRef", type:"variable", language:"typescript"}),
  (canvasElementDeleteError:Variable {name:"canvasElementDeleteError", type:"variable", language:"typescript"}),
  (canvasElementDeleteSaved:Variable {name:"canvasElementDeleteSaved", type:"variable", language:"typescript"}),
  (canvasElementContentCommands:Variable {name:"canvasElementContentCommands", type:"variable", language:"typescript"}),
  (canvasElementContentPending:Variable {name:"canvasElementContentPending", type:"variable", language:"typescript"}),
  (canvasElementContentPendingRef:Variable {name:"canvasElementContentPendingRef", type:"variable", language:"typescript"}),
  (canvasElementContentError:Variable {name:"canvasElementContentError", type:"variable", language:"typescript"}),
  (canvasElementContentSaved:Variable {name:"canvasElementContentSaved", type:"variable", language:"typescript"}),
  (canvasElementTextDraft:Variable {name:"canvasElementTextDraft", type:"variable", language:"typescript"}),
  (showCanvasLinkForm:Variable {name:"showCanvasLinkForm", type:"variable", language:"typescript"}),
  (unlinkedCanvasWorkItems:Variable {name:"unlinkedCanvasWorkItems", type:"variable", language:"typescript"}),
  (projects:Variable {name:"projects", type:"variable", language:"typescript"}),
  (worktrees:Variable {name:"worktrees", type:"variable", language:"typescript"}),
  (worktree:Variable {name:"worktree", type:"variable", language:"typescript"}),
  (projectWorkItems:Variable {name:"projectWorkItems", type:"variable", language:"typescript"}),
  (currentApp:Variable {name:"currentApp", type:"variable", language:"typescript"}),
  (scope:Variable {name:"scope", type:"variable", language:"typescript"}),
  (project:Variable {name:"project", type:"variable", language:"typescript"}),
  (taskHref:Function {name:"taskHref", type:"function", signature:"taskHref(workItemId: string, cli?: boolean): string", visibility:"private", complexity:"simple"}),
  (appHref:Function {name:"appHref", type:"function", signature:"appHref(appId: AppId, params?: Record<string, string>): string", visibility:"private", complexity:"simple"}),
  (changeScope:Function {name:"changeScope", type:"function", signature:"changeScope(nextScope: ChatScope): void", visibility:"private", complexity:"simple"}),
  (openCanvasTask:Function {name:"openCanvasTask", type:"function", signature:"openCanvasTask(workItemId: string, refWorktreeId?: string): void", visibility:"private", complexity:"moderate"}),
  (clearCanvasBootstrapCommand:Function {name:"clearCanvasBootstrapCommand", type:"function", signature:"clearCanvasBootstrapCommand(): void", visibility:"private", complexity:"simple"}),
  (clearCanvasDocumentDraft:Function {name:"clearCanvasDocumentDraft", type:"function", signature:"clearCanvasDocumentDraft(): void", visibility:"private", complexity:"simple"}),
  (selectCanvas:Function {name:"selectCanvas", type:"function", signature:"selectCanvas(canvasId: string): void", visibility:"private", complexity:"simple"}),
  (getCanvasDocumentDraft:Function {name:"getCanvasDocumentDraft", type:"function", signature:"getCanvasDocumentDraft(): CanvasDocumentDraft | null", visibility:"private", complexity:"simple"}),
  (resetPendingGroupCommands:Function {name:"resetPendingGroupCommands", type:"function", signature:"resetPendingGroupCommands(): void", visibility:"private", complexity:"moderate"}),
  (createCanvasTask:Function {name:"createCanvasTask", type:"function", signature:"createCanvasTask(event: FormEvent<HTMLFormElement>): Promise<void>", visibility:"private", complexity:"moderate"}),
  (createWorktreeCanvas:Function {name:"createWorktreeCanvas", type:"function", signature:"createWorktreeCanvas(title?: string): Promise<void>", visibility:"private", complexity:"moderate"}),
  (linkExistingCanvasWorkItem:Function {name:"linkExistingCanvasWorkItem", type:"function", signature:"linkExistingCanvasWorkItem(event: FormEvent<HTMLFormElement>): Promise<void>", visibility:"private", complexity:"moderate"}),
  (stageCanvasViewport:Function {name:"stageCanvasViewport", type:"function", signature:"stageCanvasViewport(viewport: CanvasViewport): void", visibility:"private", complexity:"simple"}),
  (saveCanvasDocument:Function {name:"saveCanvasDocument", type:"function", signature:"saveCanvasDocument(): Promise<void>", visibility:"private", complexity:"moderate"}),
  (stageCanvasFrame:Function {name:"stageCanvasFrame", type:"function", signature:"stageCanvasFrame(event: FormEvent<HTMLFormElement>): void", visibility:"private", complexity:"simple"}),
  (removeCanvasFrame:Function {name:"removeCanvasFrame", type:"function", signature:"removeCanvasFrame(frameId: string): void", visibility:"private", complexity:"simple"}),
  (addSelectedElementsToCanvasFrame:Function {name:"addSelectedElementsToCanvasFrame", type:"function", signature:"addSelectedElementsToCanvasFrame(): void", visibility:"private", complexity:"simple"}),
  (removeSelectedElementsFromCanvasFrame:Function {name:"removeSelectedElementsFromCanvasFrame", type:"function", signature:"removeSelectedElementsFromCanvasFrame(): void", visibility:"private", complexity:"simple"}),
  (addCanvasVisualConnector:Function {name:"addCanvasVisualConnector", type:"function", signature:"addCanvasVisualConnector(): void", visibility:"private", complexity:"simple"}),
  (removeCanvasVisualConnector:Function {name:"removeCanvasVisualConnector", type:"function", signature:"removeCanvasVisualConnector(connectorId: string): void", visibility:"private", complexity:"simple"}),
  (saveCanvasElementPosition:Function {name:"saveCanvasElementPosition", type:"function", signature:"saveCanvasElementPosition(position): Promise<void>", visibility:"private", complexity:"moderate"}),
  (deleteCanvasElements:Function {name:"deleteCanvasElements", type:"function", signature:"deleteCanvasElements(elementIds: string[]): Promise<void>", visibility:"private", complexity:"moderate"}),
  (saveCanvasElementContent:Function {name:"saveCanvasElementContent", type:"function", signature:"saveCanvasElementContent(): Promise<void>", visibility:"private", complexity:"moderate"}),
  (updateCanvasFrame:Function {name:"updateCanvasFrame", type:"function", signature:"updateCanvasFrame(frameId: string, patch: Partial<CanvasFrame>): void", visibility:"private", complexity:"simple"}),
  (updateCanvasConnector:Function {name:"updateCanvasConnector", type:"function", signature:"updateCanvasConnector(connectorId: string, patch: Partial<CanvasConnector>): void", visibility:"private", complexity:"simple"}),
  (transitionLiveWorkItem:Function {name:"transitionLiveWorkItem", type:"function", signature:"transitionLiveWorkItem(item, targetStatus): Promise<void>", visibility:"private", complexity:"moderate"}),
  (getTaskDisplayStatus:Function {name:"getTaskDisplayStatus", type:"function", signature:"getTaskDisplayStatus(item): WorkItemStatus", visibility:"private", complexity:"simple"}),
  (renderLifecycleActions:Function {name:"renderLifecycleActions", type:"function", signature:"renderLifecycleActions(item): JSX.Element | null", visibility:"private", complexity:"moderate"}),
  (updateCanvasElement:Function {name:"WorktreeGroupApiClient.updateCanvasElement", type:"function", signature:"updateCanvasElement(worktreeId, canvasId, elementId, body, idempotencyKey): Promise<T>", visibility:"public", complexity:"simple"}),
  (deleteCanvasElement:Function {name:"WorktreeGroupApiClient.deleteCanvasElement", type:"function", signature:"deleteCanvasElement(worktreeId, canvasId, elementId, body, idempotencyKey): Promise<T>", visibility:"public", complexity:"simple"}),
  (updateCanvasDocument:Function {name:"WorktreeGroupApiClient.updateCanvasDocument", type:"function", signature:"updateCanvasDocument(worktreeId, canvasId, body, idempotencyKey): Promise<T>", visibility:"public", complexity:"simple"}),
  (createCanvasElement:Function {name:"WorktreeGroupApiClient.createCanvasElement", type:"function", signature:"createCanvasElement(worktreeId, canvasId, body, idempotencyKey): Promise<T>", visibility:"public", complexity:"simple"}),
  (taskMap:Function {name:"taskMap", type:"function", signature:"projectWorkItems.map(workItem => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (appMap:Function {name:"appMap", type:"function", signature:"GROUP_APPS.map(app => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (jiraColumnMap:Function {name:"jiraColumnMap", type:"function", signature:"STATUS_OPTIONS.map(status => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (jiraTaskMap:Function {name:"jiraTaskMap", type:"function", signature:"projectWorkItems.map(workItem => JSX.Element)", visibility:"private", complexity:"moderate"}),
  (statusMap:Function {name:"statusMap", type:"function", signature:"STATUS_OPTIONS.map(status => JSX.Element)", visibility:"private", complexity:"simple"}),
  (statusClick:Function {name:"statusClick", type:"function", signature:"onClick(): void", visibility:"private", complexity:"simple"}),
  (pluginToggle:Function {name:"pluginToggle", type:"function", signature:"onChange(event): void", visibility:"private", complexity:"simple"}),
  (pluginStateUpdate:Function {name:"pluginStateUpdate", type:"function", signature:"setEnabledPlugins(current => nextState)", visibility:"private", complexity:"simple"}),
  (paneRenderer:Function {name:"paneRenderer", type:"function", signature:"renderPane(paneId: string): JSX.Element", visibility:"private", complexity:"simple"}),
  (file)-[:CONTAINS]->(page),
  (file)-[:CONTAINS]->(apps),
  (file)-[:CONTAINS]->(pluginPreview),
  (file)-[:CONTAINS]->(statusOptions),
  (file)-[:CONTAINS]->(appType),
  (file)-[:CONTAINS]->(scopeType),
  (file)-[:CONTAINS]->(pendingCanvasTask),
  (file)-[:CONTAINS]->(pendingCanvasBootstrap),
  (file)-[:CONTAINS]->(pendingCanvasElement),
  (file)-[:CONTAINS]->(pendingCanvasElementMove),
  (file)-[:CONTAINS]->(pendingCanvasElementDelete),
  (file)-[:CONTAINS]->(pendingCanvasDocument),
  (file)-[:CONTAINS]->(pendingLifecycleCommand),
  (file)-[:CONTAINS]->(updateCanvasDocument),
  (file)-[:CONTAINS]->(createCanvasElement),
  (file)-[:CONTAINS]->(updateCanvasElement),
  (file)-[:CONTAINS]->(deleteCanvasElement),
  (file)-[:CONTAINS]->(isAppId),
  (page)-[:USES]->(apps),
  (page)-[:USES]->(pluginPreview),
  (page)-[:USES]->(visibleGroupApps),
  (page)-[:USES]->(selectedPlugin),
  (page)-[:USES]->(statusOptions),
  (page)-[:USES]->(scope),
  (page)-[:USES]->(enabledPlugins),
  (page)-[:USES]->(groupCanvas),
  (page)-[:USES]->(groupCanvases),
  (page)-[:USES]->(requestedCanvasId),
  (page)-[:USES]->(canvasLinkError),
  (page)-[:USES]->(setCanvasLinkError),
  (page)-[:USES]->(canvasTaskTitle),
  (page)-[:USES]->(canvasTaskDescription),
  (page)-[:USES]->(canvasTaskError),
  (page)-[:USES]->(createdCanvasTaskId),
  (page)-[:USES]->(canvasTaskPending),
  (page)-[:USES]->(canvasBootstrapPending),
  (page)-[:USES]->(canvasBootstrapError),
  (page)-[:USES]->(canvasBootstrapCommands),
  (page)-[:USES]->(canvasTaskCommands),
  (page)-[:USES]->(pendingCanvasDocumentDraft),
  (page)-[:USES]->(selectedCanvasElementIds),
  (page)-[:USES]->(selectedCanvasFrameId),
  (page)-[:USES]->(selectedCanvasConnectorId),
  (page)-[:USES]->(canvasDocumentSavePending),
  (page)-[:USES]->(canvasDocumentError),
  (page)-[:USES]->(canvasDocumentSaved),
  (page)-[:USES]->(canvasDocumentCommand),
  (page)-[:USES]->(groupApiGeneration),
  (page)-[:USES]->(projectionRefreshKey),
  (page)-[:USES]->(showCanvasTaskForm),
  (page)-[:USES]->(selectedExistingWorkItemId),
  (page)-[:USES]->(canvasElementLinkPending),
  (page)-[:USES]->(canvasElementLinkError),
  (page)-[:USES]->(linkedCanvasWorkItemId),
  (page)-[:USES]->(canvasElementLinkCommands),
  (page)-[:USES]->(canvasElementMoveCommands),
  (page)-[:USES]->(canvasElementMovePending),
  (page)-[:USES]->(canvasElementMovePendingRef),
  (page)-[:USES]->(canvasElementMoveError),
  (page)-[:USES]->(canvasElementMoveSaved),
  (page)-[:USES]->(canvasElementDeleteCommands),
  (page)-[:USES]->(canvasElementDeletePending),
  (page)-[:USES]->(canvasElementDeletePendingRef),
  (page)-[:USES]->(canvasElementDeleteError),
  (page)-[:USES]->(canvasElementDeleteSaved),
  (page)-[:USES]->(canvasElementContentCommands),
  (page)-[:USES]->(canvasElementContentPending),
  (page)-[:USES]->(canvasElementContentPendingRef),
  (page)-[:USES]->(canvasElementContentError),
  (page)-[:USES]->(canvasElementContentSaved),
  (page)-[:USES]->(canvasElementTextDraft),
  (page)-[:USES]->(showCanvasLinkForm),
  (page)-[:USES]->(unlinkedCanvasWorkItems),
  (page)-[:USES]->(projectWorkItems),
  (page)-[:USES]->(currentApp),
  (page)-[:USES]->(projects),
  (page)-[:USES]->(worktrees),
  (page)-[:USES]->(worktree),
  (page)-[:USES]->(project),
  (page)-[:CALLS]->(isAppId),
  (page)-[:CONTAINS]->(taskHref),
  (page)-[:CONTAINS]->(appHref),
  (page)-[:CONTAINS]->(changeScope),
  (page)-[:CONTAINS]->(openCanvasTask),
  (page)-[:CONTAINS]->(clearCanvasBootstrapCommand),
  (page)-[:CONTAINS]->(clearCanvasDocumentDraft),
  (page)-[:CONTAINS]->(selectCanvas),
  (page)-[:CONTAINS]->(getCanvasDocumentDraft),
  (page)-[:CONTAINS]->(resetPendingGroupCommands),
  (page)-[:CONTAINS]->(createCanvasTask),
  (page)-[:CONTAINS]->(createWorktreeCanvas),
  (page)-[:CONTAINS]->(linkExistingCanvasWorkItem),
  (page)-[:CONTAINS]->(stageCanvasViewport),
  (page)-[:CONTAINS]->(saveCanvasDocument),
  (page)-[:CONTAINS]->(stageCanvasFrame),
  (page)-[:CONTAINS]->(removeCanvasFrame),
  (page)-[:CONTAINS]->(addSelectedElementsToCanvasFrame),
  (page)-[:CONTAINS]->(removeSelectedElementsFromCanvasFrame),
  (page)-[:CONTAINS]->(addCanvasVisualConnector),
  (page)-[:CONTAINS]->(removeCanvasVisualConnector),
  (page)-[:CONTAINS]->(saveCanvasElementPosition),
  (page)-[:CONTAINS]->(deleteCanvasElements),
  (page)-[:CONTAINS]->(saveCanvasElementContent),
  (page)-[:CONTAINS]->(updateCanvasFrame),
  (page)-[:CONTAINS]->(updateCanvasConnector),
  (page)-[:CONTAINS]->(taskMap),
  (page)-[:CONTAINS]->(appMap),
  (page)-[:CONTAINS]->(jiraColumnMap),
  (page)-[:CONTAINS]->(jiraTaskMap),
  (page)-[:CONTAINS]->(statusMap),
  (page)-[:CONTAINS]->(statusClick),
  (page)-[:CONTAINS]->(pluginToggle),
  (page)-[:CONTAINS]->(pluginStateUpdate),
  (page)-[:CONTAINS]->(paneRenderer),
  (page)-[:CALLS]->(taskHref),
  (page)-[:CALLS]->(appHref),
  (page)-[:CALLS]->(changeScope),
  (page)-[:CALLS]->(openCanvasTask),
  (page)-[:CALLS]->(clearCanvasBootstrapCommand),
  (page)-[:CALLS]->(clearCanvasDocumentDraft),
  (page)-[:CALLS]->(selectCanvas),
  (page)-[:CALLS]->(getCanvasDocumentDraft),
  (page)-[:CALLS]->(resetPendingGroupCommands),
  (page)-[:CALLS]->(createCanvasTask),
  (page)-[:CALLS]->(createWorktreeCanvas),
  (page)-[:CALLS]->(linkExistingCanvasWorkItem),
  (page)-[:CALLS]->(stageCanvasViewport),
  (page)-[:CALLS]->(saveCanvasDocument),
  (page)-[:CALLS]->(stageCanvasFrame),
  (page)-[:CALLS]->(removeCanvasFrame),
  (page)-[:CALLS]->(addSelectedElementsToCanvasFrame),
  (page)-[:CALLS]->(removeSelectedElementsFromCanvasFrame),
  (page)-[:CALLS]->(addCanvasVisualConnector),
  (page)-[:CALLS]->(removeCanvasVisualConnector),
  (page)-[:CALLS]->(saveCanvasElementPosition),
  (page)-[:CALLS]->(deleteCanvasElements),
  (page)-[:CALLS]->(saveCanvasElementContent),
  (page)-[:CALLS]->(updateCanvasFrame),
  (page)-[:CALLS]->(updateCanvasConnector),
  (openCanvasTask)-[:CALLS]->(setCanvasLinkError),
  (page)-[:CALLS]->(taskMap),
  (page)-[:CALLS]->(appMap),
  (visibleGroupApps)-[:DERIVES_FROM]->(apps),
  (visibleGroupApps)-[:DERIVES_FROM]->(pluginPreview),
  (visibleGroupApps)-[:FILTERS_BY]->(enabledPlugins),
  (selectedPlugin)-[:DERIVES_FROM]->(currentApp),
  (selectedPlugin)-[:LOOKS_UP]->(pluginPreview),
  (page)-[:CALLS]->(jiraColumnMap),
  (page)-[:CALLS]->(jiraTaskMap),
  (page)-[:CALLS]->(statusMap),
  (page)-[:CALLS]->(statusClick),
  (page)-[:CALLS]->(pluginToggle),
  (page)-[:CALLS]->(pluginStateUpdate),
  (page)-[:CALLS]->(paneRenderer),(stageCanvasViewport)-[:CALLS]->(getCanvasDocumentDraft),(stageCanvasViewport)-[:USES]->(pendingCanvasDocumentDraft),(getCanvasDocumentDraft)-[:USES]->(pendingCanvasDocumentDraft),(saveCanvasDocument)-[:USES]->(pendingCanvasDocumentDraft),(saveCanvasDocument)-[:USES]->(canvasDocumentCommand),(saveCanvasDocument)-[:CALLS]->(updateCanvasDocument),(stageCanvasFrame)-[:CALLS]->(getCanvasDocumentDraft),(stageCanvasFrame)-[:USES]->(pendingCanvasDocumentDraft),(removeCanvasFrame)-[:CALLS]->(getCanvasDocumentDraft),(removeCanvasFrame)-[:USES]->(pendingCanvasDocumentDraft),(addSelectedElementsToCanvasFrame)-[:CALLS]->(getCanvasDocumentDraft),(addSelectedElementsToCanvasFrame)-[:USES]->(selectedCanvasElementIds),(addCanvasVisualConnector)-[:CALLS]->(getCanvasDocumentDraft),(addCanvasVisualConnector)-[:USES]->(selectedCanvasElementIds),(removeCanvasVisualConnector)-[:CALLS]->(getCanvasDocumentDraft),(removeCanvasVisualConnector)-[:USES]->(pendingCanvasDocumentDraft),(linkExistingCanvasWorkItem)-[:CALLS]->(createCanvasElement),(saveCanvasElementPosition)-[:USES]->(canvasElementMoveCommands),(saveCanvasElementPosition)-[:USES]->(canvasElementMovePendingRef),(saveCanvasElementPosition)-[:CALLS]->(updateCanvasElement),(page)-[:USES]->(lifecycleCommands),(page)-[:USES]->(pendingLifecycleWorkItemId),(page)-[:USES]->(lifecycleError),(page)-[:CONTAINS]->(transitionLiveWorkItem),(page)-[:CONTAINS]->(getTaskDisplayStatus),(page)-[:CONTAINS]->(renderLifecycleActions),(page)-[:CALLS]->(transitionLiveWorkItem),(page)-[:CALLS]->(getTaskDisplayStatus),(page)-[:CALLS]->(renderLifecycleActions),(transitionLiveWorkItem)-[:USES]->(lifecycleCommands),(transitionLiveWorkItem)-[:USES]->(pendingLifecycleWorkItemId),(renderLifecycleActions)-[:CALLS]->(transitionLiveWorkItem),(transitionLiveWorkItem)-[:CALLS]->(transitionApi:Function {name:"WorktreeGroupApiClient.transitionWorkItem",type:"function"});
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (deleteElements:Function {name:"deleteCanvasElements"}),
      (deleteApi:Function {name:"WorktreeGroupApiClient.deleteCanvasElement"}),
      (deletePending:Variable {name:"canvasElementDeletePending"}),
      (contentSave:Function {name:"saveCanvasElementContent"}),
      (contentPending:Variable {name:"canvasElementContentPending"}),
      (selectedIds:Variable {name:"selectedCanvasElementIds"}),
      (frameUpdate:Function {name:"updateCanvasFrame"}),
      (connectorUpdate:Function {name:"updateCanvasConnector"}),
      (draft:Function {name:"getCanvasDocumentDraft"});
CREATE (deleteElements)-[:CALLS]->(deleteApi),
       (deleteElements)-[:USES]->(deletePending),
       (deleteElements)-[:USES]->(selectedIds),
       (contentSave)-[:USES]->(contentPending),
       (frameUpdate)-[:CALLS]->(draft),
       (connectorUpdate)-[:CALLS]->(draft);
MATCH (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx"}),
      (page:Function {name:"GroupWorkspacePage"}),
      (contentSave:Function {name:"saveCanvasElementContent"}),
      (updateElement:Function {name:"WorktreeGroupApiClient.updateCanvasElement"}),
      (contentCommands:Variable {name:"canvasElementContentCommands"});
CREATE (selectedCanvasElement:Variable {name:"selectedCanvasElement",type:"variable",language:"typescript"}),
       (file)-[:CONTAINS]->(selectedCanvasElement),
       (page)-[:USES]->(selectedCanvasElement),
       (contentSave)-[:CALLS]->(updateElement),
       (contentSave)-[:USES]->(contentCommands);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx"}),
      (page:Function {name:"GroupWorkspacePage"}),
      (contentSave:Function {name:"saveCanvasElementContent"}),
      (updateElement:Function {name:"WorktreeGroupApiClient.updateCanvasElement"});
CREATE (geometrySave:Function {name:"saveCanvasElementGeometry",type:"function",language:"typescript"}),
       (geometryCommands:Variable {name:"canvasElementGeometryCommands",type:"variable",language:"typescript"}),
       (geometryPending:Variable {name:"canvasElementGeometryPending",type:"variable",language:"typescript"}),
       (geometryError:Variable {name:"canvasElementGeometryError",type:"variable",language:"typescript"}),
       (geometrySaved:Variable {name:"canvasElementGeometrySaved",type:"variable",language:"typescript"}),
       (geometryDraft:Variable {name:"canvasElementGeometryDraft",type:"variable",language:"typescript"}),
       (geometryType:Class {name:"CanvasElementGeometryDraft",type:"class",language:"typescript"}),
       (file)-[:CONTAINS]->(geometrySave),
       (file)-[:CONTAINS]->(geometryCommands),
       (file)-[:CONTAINS]->(geometryPending),
       (file)-[:CONTAINS]->(geometryError),
       (file)-[:CONTAINS]->(geometrySaved),
       (file)-[:CONTAINS]->(geometryDraft),
       (file)-[:CONTAINS]->(geometryType),
       (page)-[:CONTAINS]->(geometrySave),
       (geometrySave)-[:CALLS]->(updateElement),
       (geometrySave)-[:USES]->(geometryCommands),
       (geometrySave)-[:USES]->(geometryPending),
       (geometrySave)-[:USES]->(geometryError),
       (geometrySave)-[:USES]->(geometryDraft),
       (geometrySave)-[:USES]->(geometrySaved),
       (contentSave)-[:CALLS]->(updateElement);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (page:Function {name:"GroupWorkspacePage"}),
      (api:Function {name:"WorktreeGroupApiClient.startTaskCliSession"});
CREATE (startCli:Function {name:"startSelectedTaskCliSession",type:"function",language:"typescript"}),
       (ticket:Function {name:"getCliAttachmentTicket",type:"function",language:"typescript"}),
       (session:Variable {name:"cliSession",type:"variable",language:"typescript"}),
       (profile:Variable {name:"cliProfileId",type:"variable",language:"typescript"}),
       (commands:Variable {name:"cliSessionCommands",type:"variable",language:"typescript"});
CREATE (page)-[:CONTAINS]->(startCli),
       (page)-[:CONTAINS]->(ticket),
       (page)-[:USES]->(session),
       (page)-[:USES]->(profile),
       (page)-[:USES]->(commands),
       (startCli)-[:CALLS]->(api),
       (startCli)-[:USES]->(commands),
       (ticket)-[:USES]->(session);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (page:Function {name:"GroupWorkspacePage"}),
      (statusApi:Function {name:"WorktreeGroupApiClient.getTaskCliSessionStatus"}),
      (cancelApi:Function {name:"WorktreeGroupApiClient.cancelTaskCliSession"}),
      (reattachApi:Function {name:"WorktreeGroupApiClient.reattachTaskCliSession"}),
      (listApi:Function {name:"WorktreeGroupApiClient.listTaskCliSessions"}),
      (ticket:Function {name:"getCliAttachmentTicket"});
CREATE (refreshCliStatus:Function {name:"refreshSelectedTaskCliSessionStatus",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (reattachCli:Function {name:"reattachSelectedTaskCliSession",type:"function",language:"typescript",visibility:"private",complexity:"complex"}),
       (cancelCli:Function {name:"cancelSelectedTaskCliSession",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (connectionChange:Function {name:"handleTaskCliConnectionChange",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
       (cliStatus:Variable {name:"cliSessionStatus",type:"variable",language:"typescript"}),
       (cliHistory:Variable {name:"cliSessionHistory",type:"variable",language:"typescript"}),
       (cliHistoryPending:Variable {name:"cliSessionHistoryPending",type:"variable",language:"typescript"}),
       (cliHistoryError:Variable {name:"cliSessionHistoryError",type:"variable",language:"typescript"}),
       (cliHistoryRefresh:Variable {name:"cliSessionHistoryRefresh",type:"variable",language:"typescript"}),
       (cliPending:Variable {name:"cliSessionControlPending",type:"variable",language:"typescript"}),
       (cliControlRequest:Variable {name:"cliSessionControlRequest",type:"variable",language:"typescript"}),
       (cliReconnect:Variable {name:"cliNeedsReattach",type:"variable",language:"typescript"}),
       (cliConnected:Variable {name:"cliWsConnected",type:"variable",language:"typescript"}),
       (cliGeneration:Variable {name:"cliTerminalGeneration",type:"variable",language:"typescript"});
CREATE (page)-[:CONTAINS]->(refreshCliStatus),(page)-[:CONTAINS]->(reattachCli),(page)-[:CONTAINS]->(cancelCli),(page)-[:CONTAINS]->(connectionChange),
       (page)-[:USES]->(cliStatus),(page)-[:USES]->(cliPending),(page)-[:USES]->(cliControlRequest),(page)-[:USES]->(cliReconnect),(page)-[:USES]->(cliConnected),(page)-[:USES]->(cliGeneration),
       (page)-[:USES]->(cliHistory),(page)-[:USES]->(cliHistoryPending),(page)-[:USES]->(cliHistoryError),
       (page)-[:USES]->(cliHistoryRefresh),
       (page)-[:CALLS]->(listApi),
       (refreshCliStatus)-[:CALLS]->(statusApi),(reattachCli)-[:CALLS]->(statusApi),(reattachCli)-[:CALLS]->(reattachApi),
       (cancelCli)-[:CALLS]->(cancelApi),(connectionChange)-[:USES]->(cliConnected),(connectionChange)-[:USES]->(cliReconnect),
       (reattachCli)-[:USES]->(cliGeneration),(ticket)-[:USES]->(cliReconnect);
*/

"use client";

import { use as ReactUse, useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import {
  ArrowUpRight,
  Blocks,
  CheckSquare,
  ClipboardList,
  GitBranch,
  LayoutGrid,
  MessageSquareText,
  Plug,
  Plus,
  SquareKanban,
  Workflow,
  type LucideIcon,
} from "lucide-react";

import { CanvasView } from "@/components/CanvasView";
import { StatusPill } from "@/components/StatusPill";
import { TerminalStackContainer } from "@/components/terminal/TerminalStackContainer";
import {
  useWorktreeGroupApi,
  useWorktreeGroupProjection,
  type GroupWorkItemView,
} from "@/lib/group/groupProjection";
import { projectGroupAppsForNavigation } from "@/lib/group/groupAppsRegistry";
import type {
  GroupAppNavigationEntry,
  ScopedChatTarget,
  TaskCliSessionReceipt,
  TaskCliSessionStatus,
  WorktreeGroupApiClient,
} from "@/lib/group/worktreeGroupApi";
import { useStore } from "@/lib/store";
import type { CanvasConnector, CanvasFrame, CanvasViewport, WorkItemStatus } from "@/types/ids";

interface PageProps {
  params: Promise<{ id: string }>;
}

type AppId = "multica" | "jira" | "task-card" | "canvas" | "workflow" | "plugins" | `plugin:${string}`;
type ChatScope = "WORKTREE" | "GLOBAL";
/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (review:Function {name:"reviewLiveWorkItem"}),
      (reviewCommands:Variable {name:"reviewCommands"}),
      (reviewReasons:Variable {name:"reviewReasons"}),
      (reviewApi:Function {name:"WorktreeGroupApiClient.reviewWorkItem"}),
      (render:Function {name:"renderLifecycleActions"});
CREATE (review)-[:USES]->(reviewCommands),
       (review)-[:USES]->(reviewReasons),
       (review)-[:CALLS]->(reviewApi),
       (render)-[:CALLS]->(review);
*/

type LifecycleTargetStatus = "pending" | "claimed" | "in_progress" | "completed" | "failed" | "cancelled";
type ReviewAction = "submit" | "accept" | "reject";
type PendingCanvasTaskCommand = {
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type PendingCanvasBootstrapCommand = {
  correlationId: string;
  idempotencyKey: string;
  canvasId?: string;
};
type PendingCanvasElementCommand = {
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type PendingCanvasElementMoveCommand = {
  fingerprint: string;
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type CanvasElementGeometryDraft = { width: number; height: number; rotation: number };
type PendingCanvasElementDeleteCommand = {
  fingerprint: string;
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type PendingCanvasDocumentCommand = {
  fingerprint: string;
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type CanvasDocumentDraft = {
  expectedVersion: number;
  viewport: CanvasViewport;
  frames: CanvasFrame[];
  connectors: CanvasConnector[];
};
type PendingLifecycleCommand = {
  correlationId: string;
  idempotencyKey: string;
  body: Record<string, unknown>;
};
type PendingTaskCliSessionCommand = {
  correlationId: string;
  idempotencyKey: string;
  body: {
    expected_lifecycle_version: number;
    approved_launch_profile_id: string;
    correlation_id: string;
  };
};

function isAppId(value: string | null, pluginIds: readonly string[] = []): value is AppId {
  return GROUP_APPS.some((app) => app.id === value) || pluginIds.some((pluginId) => `plugin:${pluginId}` === value);
}

type GroupAppRegistryState =
  | { mode: "preview"; worktreeId: string; client: null; requestKey: number }
  | { mode: "loading"; worktreeId: string; client: WorktreeGroupApiClient; requestKey: number }
  | { mode: "error"; worktreeId: string; client: WorktreeGroupApiClient; requestKey: number; message: string }
  | {
      mode: "live";
      worktreeId: string;
      client: WorktreeGroupApiClient;
      requestKey: number;
      registryVersion: number;
      apps: GroupAppNavigationEntry[];
    };

const GROUP_APPS: Array<{
  id: AppId;
  label: string;
  subtitle: string;
  icon: LucideIcon;
}> = [
  { id: "multica", label: "Multica", subtitle: "任务生命周期", icon: Blocks },
  { id: "jira", label: "Jira 视图", subtitle: "Board / Backlog / Sprint", icon: SquareKanban },
  { id: "task-card", label: "Task Cards", subtitle: "统一任务入口", icon: ClipboardList },
  { id: "canvas", label: "Infinite Canvas", subtitle: "Miro 式协作画布", icon: LayoutGrid },
  { id: "workflow", label: "Workflow", subtitle: "LangGraph / L0", icon: Workflow },
  { id: "plugins", label: "Plugins", subtitle: "群组应用与能力", icon: Plug },
];

const PLUGIN_PREVIEW = [
  { id: "canvas-insights", name: "Canvas Insights", capability: "canvas.read / relation.query" },
  { id: "release-helper", name: "Release Helper", capability: "work_item.read / workflow.run" },
  { id: "scm-bridge", name: "SCM Bridge", capability: "pull_request.read / status.subscribe" },
];

const STATUS_OPTIONS: Array<{ value: WorkItemStatus; label: string }> = [
  { value: "todo", label: "待办" },
  { value: "in_progress", label: "进行中" },
  { value: "review", label: "评审中" },
  { value: "blocked", label: "阻塞" },
  { value: "done", label: "完成" },
  { value: "wontfix", label: "不处理" },
];

export default function GroupWorkspacePage({ params }: PageProps) {
  const { id: worktreeId } = ReactUse(params);
  const router = useRouter();
  const searchParams = useSearchParams();
  const scope: ChatScope = searchParams.get("scope") === "GLOBAL" ? "GLOBAL" : "WORKTREE";
  const [canvasLinkError, setCanvasLinkError] = useState<string | null>(null);
  const [cliProfileId, setCliProfileId] = useState("");
  const [cliSession, setCliSession] = useState<(TaskCliSessionReceipt & { requested_worktree_id: string }) | null>(null);
  const [cliSessionStatus, setCliSessionStatus] = useState<TaskCliSessionStatus | null>(null);
  const [cliSessionHistory, setCliSessionHistory] = useState<TaskCliSessionStatus[]>([]);
  const [cliSessionHistoryPending, setCliSessionHistoryPending] = useState(false);
  const [cliSessionHistoryError, setCliSessionHistoryError] = useState<string | null>(null);
  const [cliSessionHistoryRefresh, setCliSessionHistoryRefresh] = useState(0);
  const [cliSessionPending, setCliSessionPending] = useState(false);
  const [cliSessionControlPending, setCliSessionControlPending] = useState(false);
  const cliSessionControlRequest = useRef<string | null>(null);
  const [cliSessionError, setCliSessionError] = useState<string | null>(null);
  const [cliWsConnected, setCliWsConnected] = useState(false);
  const [cliNeedsReattach, setCliNeedsReattach] = useState(false);
  const [cliTerminalGeneration, setCliTerminalGeneration] = useState(0);
  const cliSessionCommands = useRef(new Map<string, PendingTaskCliSessionCommand>());
  const cliTicketClaimed = useRef(false);
  const [chatTargets, setChatTargets] = useState<ScopedChatTarget[]>([]);
  const [chatTargetCursor, setChatTargetCursor] = useState<string | null>(null);
  const [chatTargetsPending, setChatTargetsPending] = useState(false);
  const [chatTargetsError, setChatTargetsError] = useState<string | null>(null);
  const [selectedChatTargetIds, setSelectedChatTargetIds] = useState<string[]>([]);
  const [canvasTaskTitle, setCanvasTaskTitle] = useState("");
  const [canvasTaskDescription, setCanvasTaskDescription] = useState("");
  const [canvasTaskError, setCanvasTaskError] = useState<string | null>(null);
  const [createdCanvasTaskId, setCreatedCanvasTaskId] = useState<string | null>(null);
  const [canvasTaskPending, setCanvasTaskPending] = useState(false);
  const [canvasBootstrapPending, setCanvasBootstrapPending] = useState(false);
  const [canvasBootstrapError, setCanvasBootstrapError] = useState<string | null>(null);
  const canvasBootstrapCommands = useRef(new Map<string, PendingCanvasBootstrapCommand>());
  const canvasTaskCommands = useRef(new Map<string, PendingCanvasTaskCommand>());
  const [pendingCanvasDocumentDraft, setPendingCanvasDocumentDraft] = useState<CanvasDocumentDraft | null>(null);
  const [selectedCanvasElementIds, setSelectedCanvasElementIds] = useState<string[]>([]);
  const [selectedCanvasFrameId, setSelectedCanvasFrameId] = useState("");
  const [selectedCanvasConnectorId, setSelectedCanvasConnectorId] = useState("");
  const [canvasFrameTitle, setCanvasFrameTitle] = useState("");
  const [canvasDocumentSavePending, setCanvasDocumentSavePending] = useState(false);
  const [canvasDocumentError, setCanvasDocumentError] = useState<string | null>(null);
  const [canvasDocumentSaved, setCanvasDocumentSaved] = useState(false);
  const canvasDocumentCommand = useRef<PendingCanvasDocumentCommand | null>(null);
  const [projectionRefreshKey, setProjectionRefreshKey] = useState(0);
  const [groupAppsRefreshKey, setGroupAppsRefreshKey] = useState(0);
  const [groupAppsState, setGroupAppsState] = useState<GroupAppRegistryState>({
    mode: "preview",
    worktreeId: "",
    client: null,
    requestKey: 0,
  });
  const [showCanvasTaskForm, setShowCanvasTaskForm] = useState(false);
  const [selectedExistingWorkItemId, setSelectedExistingWorkItemId] = useState("");
  const [canvasElementLinkPending, setCanvasElementLinkPending] = useState(false);
  const [canvasElementLinkError, setCanvasElementLinkError] = useState<string | null>(null);
  const [linkedCanvasWorkItemId, setLinkedCanvasWorkItemId] = useState<string | null>(null);
  const canvasElementLinkCommands = useRef(new Map<string, PendingCanvasElementCommand>());
  const canvasElementMoveCommands = useRef(new Map<string, PendingCanvasElementMoveCommand>());
  const lifecycleCommands = useRef(new Map<string, PendingLifecycleCommand>());
  const reviewCommands = useRef(new Map<string, PendingLifecycleCommand>());
  const [reviewReasons, setReviewReasons] = useState<Record<string, string>>({});
  const [pendingLifecycleWorkItemId, setPendingLifecycleWorkItemId] = useState<string | null>(null);
  const [lifecycleError, setLifecycleError] = useState<{ workItemId: string; message: string } | null>(null);
  const canvasElementMovePendingRef = useRef(false);
  const [canvasElementMovePending, setCanvasElementMovePending] = useState(false);
  const [canvasElementMoveError, setCanvasElementMoveError] = useState<string | null>(null);
  const [canvasElementMoveSaved, setCanvasElementMoveSaved] = useState(false);
  const canvasElementDeleteCommands = useRef(new Map<string, PendingCanvasElementDeleteCommand>());
  const canvasElementDeletePendingRef = useRef(false);
  const [canvasElementDeletePending, setCanvasElementDeletePending] = useState(false);
  const [canvasElementDeleteError, setCanvasElementDeleteError] = useState<string | null>(null);
  const [canvasElementDeleteSaved, setCanvasElementDeleteSaved] = useState(false);
  const canvasElementContentCommands = useRef(new Map<string, PendingCanvasElementMoveCommand>());
  const canvasElementContentPendingRef = useRef(false);
  const [canvasElementContentPending, setCanvasElementContentPending] = useState(false);
  const [canvasElementContentError, setCanvasElementContentError] = useState<string | null>(null);
  const [canvasElementContentSaved, setCanvasElementContentSaved] = useState(false);
  const [canvasElementTextDraft, setCanvasElementTextDraft] = useState("");
  const canvasElementGeometryCommands = useRef(new Map<string, PendingCanvasElementMoveCommand>());
  const canvasElementGeometryPendingRef = useRef(false);
  const [canvasElementGeometryPending, setCanvasElementGeometryPending] = useState(false);
  const [canvasElementGeometryError, setCanvasElementGeometryError] = useState<string | null>(null);
  const [canvasElementGeometrySaved, setCanvasElementGeometrySaved] = useState(false);
  const [canvasElementGeometryDraft, setCanvasElementGeometryDraft] = useState<CanvasElementGeometryDraft | null>(null);
  const [showCanvasLinkForm, setShowCanvasLinkForm] = useState(false);
  const [enabledPlugins, setEnabledPlugins] = useState<Record<string, boolean>>({
    "canvas-insights": true,
    "release-helper": true,
    "scm-bridge": false,
  });

  const worktrees = useStore((state) => state.worktrees);
  const projects = useStore((state) => state.projects);
  const workItems = useStore((state) => state.workItems);
  const canvases = useStore((state) => state.canvases);
  const canvasElements = useStore((state) => state.canvasElements);
  const canvasConnectors = useStore((state) => state.canvasConnectors);
  const transitionWorkItem = useStore((state) => state.transitionWorkItem);

  const groupApi = useWorktreeGroupApi();
  const requestedApp = searchParams.get("app");
  const requestedCanvasId = searchParams.get("canvas_id") ?? undefined;
  const activeGroupAppsState: GroupAppRegistryState = groupAppsState.worktreeId === worktreeId
      && groupAppsState.requestKey === groupAppsRefreshKey
      && groupAppsState.client === groupApi
    ? groupAppsState
    : groupApi
      ? { mode: "loading", worktreeId, client: groupApi, requestKey: groupAppsRefreshKey }
      : { mode: "preview", worktreeId, client: null, requestKey: groupAppsRefreshKey };
  const availablePlugins = activeGroupAppsState.mode === "live"
    ? activeGroupAppsState.apps.map((plugin) => ({
        id: plugin.plugin_id,
        name: plugin.label,
        manifestVersion: plugin.manifest_version,
        source: "registry" as const,
      }))
    : activeGroupAppsState.mode === "preview"
      ? PLUGIN_PREVIEW
          .filter((plugin) => enabledPlugins[plugin.id])
          .map((plugin) => ({ ...plugin, source: "preview" as const }))
      : [];
  const currentApp = isAppId(requestedApp, availablePlugins.map((plugin) => plugin.id))
    ? requestedApp
    : "multica";
  const visibleGroupApps: Array<{ id: AppId; label: string; subtitle: string; icon: LucideIcon }> = [
    ...GROUP_APPS,
    ...availablePlugins.map((plugin) => ({
        id: `plugin:${plugin.id}` as const,
        label: plugin.name,
        subtitle: plugin.source === "registry"
          ? `Plugin App · v${plugin.manifestVersion}`
          : "Plugin App · 本地预览",
        icon: Plug,
      })),
  ];
  const selectedPlugin = typeof currentApp === "string" && currentApp.startsWith("plugin:")
    ? availablePlugins.find((plugin) => `plugin:${plugin.id}` === currentApp)
    : undefined;
  const groupApiGeneration = useRef({ client: groupApi, worktreeId });
  const resetPendingGroupCommands = useCallback(() => {
    groupApiGeneration.current = { client: groupApi, worktreeId };
    canvasBootstrapCommands.current.clear();
    canvasTaskCommands.current.clear();
    canvasElementLinkCommands.current.clear();
    canvasElementMoveCommands.current.clear();
    canvasElementDeleteCommands.current.clear();
    canvasElementContentCommands.current.clear();
    canvasElementGeometryCommands.current.clear();
    lifecycleCommands.current.clear();
    reviewCommands.current.clear();
    cliSessionCommands.current.clear();
    cliSessionControlRequest.current = null;
    cliTicketClaimed.current = false;
    setCliSession(null);
    setCliSessionStatus(null);
    setCliSessionHistory([]);
    setCliSessionHistoryPending(false);
    setCliSessionHistoryError(null);
    setCliSessionHistoryRefresh(0);
    setCliSessionPending(false);
    setCliSessionControlPending(false);
    setCliSessionError(null);
    setCliWsConnected(false);
    setCliNeedsReattach(false);
    setChatTargets([]);
    setChatTargetCursor(null);
    setChatTargetsPending(false);
    setChatTargetsError(null);
    setSelectedChatTargetIds([]);
    setCliProfileId("");
    setReviewReasons({});
    canvasElementMovePendingRef.current = false;
    canvasElementDeletePendingRef.current = false;
    canvasElementContentPendingRef.current = false;
    canvasElementGeometryPendingRef.current = false;
    canvasDocumentCommand.current = null;
    setCanvasTaskTitle("");
    setCanvasTaskDescription("");
    setCanvasTaskError(null);
    setCreatedCanvasTaskId(null);
    setCanvasTaskPending(false);
    setCanvasBootstrapError(null);
    setCanvasBootstrapPending(false);
    setPendingCanvasDocumentDraft(null);
    setSelectedCanvasElementIds([]);
    setSelectedCanvasFrameId("");
    setSelectedCanvasConnectorId("");
    setCanvasFrameTitle("");
    setCanvasDocumentSavePending(false);
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
    setShowCanvasTaskForm(false);
    setSelectedExistingWorkItemId("");
    setCanvasElementLinkPending(false);
    setPendingLifecycleWorkItemId(null);
    setLifecycleError(null);
    setCanvasElementMovePending(false);
    setCanvasElementMoveError(null);
    setCanvasElementMoveSaved(false);
    setCanvasElementDeletePending(false);
    setCanvasElementDeleteError(null);
    setCanvasElementDeleteSaved(false);
    setCanvasElementContentPending(false);
    setCanvasElementContentError(null);
    setCanvasElementContentSaved(false);
    setCanvasElementTextDraft("");
    setCanvasElementGeometryPending(false);
    setCanvasElementGeometryError(null);
    setCanvasElementGeometrySaved(false);
    setCanvasElementGeometryDraft(null);
    setCanvasElementLinkError(null);
    setLinkedCanvasWorkItemId(null);
    setShowCanvasLinkForm(false);
  }, [groupApi, worktreeId]);
  useEffect(resetPendingGroupCommands, [resetPendingGroupCommands]);

  useEffect(() => {
    if (!groupApi) {
      setGroupAppsState({ mode: "preview", worktreeId, client: null, requestKey: groupAppsRefreshKey });
      return;
    }

    let active = true;
    let loading = false;
    const load = async () => {
      if (!active || loading) return;
      loading = true;
      setGroupAppsState({ mode: "loading", worktreeId, client: groupApi, requestKey: groupAppsRefreshKey });
      try {
        const projection = await groupApi.listGroupApps(worktreeId);
        if (!active) return;
        const apps = projectGroupAppsForNavigation(projection, worktreeId);
        if (!apps) {
          throw new Error("服务端返回了无效的 Group App Registry 投影。");
        }
        setGroupAppsState({
          mode: "live",
          worktreeId,
          client: groupApi,
          requestKey: groupAppsRefreshKey,
          registryVersion: projection.registry_version,
          apps,
        });
      } catch (error) {
        if (!active) return;
        setGroupAppsState({
          mode: "error",
          worktreeId,
          client: groupApi,
          requestKey: groupAppsRefreshKey,
          message: error instanceof Error ? error.message : "无法读取当前 Worktree 的授权插件列表。",
        });
      } finally {
        loading = false;
      }
    };

    void load();
    const interval = window.setInterval(() => {
      if (document.visibilityState === "visible") void load();
    }, 30_000);
    const refreshOnReturn = () => {
      if (document.visibilityState === "visible") void load();
    };
    document.addEventListener("visibilitychange", refreshOnReturn);
    return () => {
      active = false;
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", refreshOnReturn);
    };
  }, [groupApi, groupAppsRefreshKey, worktreeId]);

  const groupProjection = useWorktreeGroupProjection(worktreeId, currentApp === "canvas", projectionRefreshKey, requestedCanvasId);
  const storeWorktree = worktrees.find((item) => item.id === worktreeId);
  const worktree = groupProjection.mode === "live" ? groupProjection.worktree : storeWorktree;
  const chatTargetOptions: ScopedChatTarget[] = groupProjection.mode === "live" ? chatTargets : [];
  useEffect(() => {
    if (scope !== "GLOBAL" || groupProjection.mode !== "live" || !groupApi) return;
    let active = true;
    setChatTargetsPending(true);
    setChatTargetsError(null);
    groupApi.listGlobalChatTargets(worktreeId, { limit: 50 })
      .then((page) => {
        if (!active) return;
        setChatTargets(page.targets);
        setChatTargetCursor(page.next_cursor);
      })
      .catch((error: unknown) => {
        if (!active) return;
        setChatTargetsError(error instanceof Error ? error.message : "无法读取已授权 Worktree");
        setChatTargets([]);
        setChatTargetCursor(null);
      })
      .finally(() => {
        if (active) setChatTargetsPending(false);
      });
    return () => { active = false; };
  }, [groupApi, groupProjection.mode, scope, worktreeId]);
  const project = projects.find((item) => item.id === worktree?.project_id);
  const storeWorkItems = worktree
    ? workItems.filter((item) =>
        item.project_id === worktree.project_id &&
        (!item.worktree_id || item.worktree_id === worktree.id),
      )
    : [];
  const projectWorkItems = groupProjection.mode === "live" ? groupProjection.work_items : storeWorkItems;
  const currentWorkItemId = searchParams.get("work_item_id");
  const selectedTask = projectWorkItems.find((item) => item.id === currentWorkItemId);
  const cliPreviewOpen = searchParams.get("cli") === "1" && selectedTask?.worktree_id === worktreeId;
  const cliSessionForSelectedTask = cliSession && selectedTask &&
    cliSession.worktree_id === worktreeId &&
    cliSession.work_item_id === selectedTask.id &&
    cliSession.requested_worktree_id === worktreeId
    ? cliSession
    : null;
  useEffect(() => {
    if (cliSession && (
      cliSession.worktree_id !== worktreeId ||
      cliSession.work_item_id !== currentWorkItemId ||
      cliSession.requested_worktree_id !== worktreeId
    )) {
      cliTicketClaimed.current = false;
      setCliSession(null);
      setCliSessionStatus(null);
      setCliSessionError(null);
      setCliWsConnected(false);
      setCliNeedsReattach(false);
    }
  }, [cliSession, currentWorkItemId, worktreeId]);
  const getCliAttachmentTicket = useCallback(async () => {
    const current = cliSession;
    if (
      !current ||
      current.worktree_id !== worktreeId ||
      current.work_item_id !== currentWorkItemId ||
      current.requested_worktree_id !== worktreeId ||
      Date.parse(current.attachment_ticket_expires_at) <= Date.now() ||
      cliTicketClaimed.current
    ) {
      throw new Error("Task CLI attachment ticket is unavailable or already used.");
    }
    cliTicketClaimed.current = true;
    return current.attachment_ticket;
  }, [cliSession, currentWorkItemId, worktreeId]);
  const handleTaskCliConnectionChange = useCallback((connected: boolean) => {
    setCliWsConnected(connected);
    if (connected) {
      setCliNeedsReattach(false);
    } else if (cliTicketClaimed.current) {
      setCliNeedsReattach(true);
    }
  }, []);
  useEffect(() => {
    if (!cliPreviewOpen || !selectedTask || groupProjection.mode !== "live" || !groupApi) {
      setCliSessionHistory([]);
      setCliSessionHistoryPending(false);
      setCliSessionHistoryError(null);
      return;
    }

    let active = true;
    const requestGeneration = groupApiGeneration.current;
    setCliSessionHistoryPending(true);
    setCliSessionHistoryError(null);
    groupApi.listTaskCliSessions(worktreeId, selectedTask.id, crypto.randomUUID(), 20)
      .then((page) => {
        if (!active || groupApiGeneration.current !== requestGeneration) return;
        const allowedStates = new Set(["starting", "running", "disconnected", "cancelling", "completed", "failed", "cancelled", "timed_out", "lost"]);
        const seenSessionIds = new Set<string>();
        if (!Array.isArray(page.sessions) || page.sessions.length > 20 || !page.sessions.every((session) => {
          const valid = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(session.session_id) &&
            allowedStates.has(session.state) &&
            (session.exit_code === null || Number.isInteger(session.exit_code)) &&
            Number.isFinite(Date.parse(session.updated_at)) &&
            !seenSessionIds.has(session.session_id);
          if (valid) seenSessionIds.add(session.session_id);
          return valid;
        })) {
          throw new Error("Task CLI session list response is invalid.");
        }
        setCliSessionHistory(page.sessions);
      })
      .catch((error: unknown) => {
        if (!active || groupApiGeneration.current !== requestGeneration) return;
        setCliSessionHistory([]);
        setCliSessionHistoryError(error instanceof Error ? error.message : "无法读取当前任务的 Session 列表。");
      })
      .finally(() => {
        if (active && groupApiGeneration.current === requestGeneration) setCliSessionHistoryPending(false);
      });
    return () => {
      active = false;
    };
  }, [cliPreviewOpen, cliSession?.session_id, cliSessionHistoryRefresh, groupApi, groupProjection.mode, selectedTask?.id, worktreeId]);
  const groupCanvas = groupProjection.mode === "live"
    ? groupProjection.canvas
    : worktree
    ? canvases.find((canvas) => canvas.ref_kind === "worktree" && canvas.ref_id === worktree.id)
    : undefined;
  const groupCanvases = groupProjection.mode === "live"
    ? groupProjection.canvases
    : worktree
    ? canvases.filter((canvas) => canvas.ref_kind === "worktree" && canvas.ref_id === worktree.id)
    : [];
  useEffect(clearCanvasBootstrapCommand, [groupProjection.mode, groupCanvas?.id, worktreeId]);
  useEffect(clearCanvasDocumentDraft, [groupCanvas?.id]);
  useEffect(() => {
    setSelectedExistingWorkItemId("");
    setCanvasElementLinkError(null);
    setLinkedCanvasWorkItemId(null);
    setShowCanvasLinkForm(false);
  }, [groupCanvas?.id]);
  useEffect(() => {
    setCanvasElementMoveError(null);
    setCanvasElementMoveSaved(false);
  }, [groupCanvas?.id]);
  useEffect(() => {
    if (groupProjection.mode !== "live" || !requestedCanvasId || !groupCanvas || requestedCanvasId === groupCanvas.id) return;
    const query = new URLSearchParams(searchParams.toString());
    query.set("canvas_id", groupCanvas.id);
    router.replace(`/worktree/${encodeURIComponent(worktreeId)}/group?${query.toString()}`, { scroll: false });
  }, [groupProjection.mode, groupCanvas?.id, requestedCanvasId, router, searchParams, worktreeId]);
  const elements = groupProjection.mode === "live"
    ? groupProjection.elements
    : groupCanvas
    ? canvasElements.filter((element) => element.canvas_id === groupCanvas.id)
    : [];
  const connectors = groupProjection.mode === "live"
    ? groupProjection.connectors
    : groupCanvas
    ? canvasConnectors.filter((connector) => connector.canvas_id === groupCanvas.id)
    : [];
  const displayedCanvas = groupCanvas && groupProjection.mode === "live" && pendingCanvasDocumentDraft
    ? { ...groupCanvas, viewport: pendingCanvasDocumentDraft.viewport, frames: pendingCanvasDocumentDraft.frames }
    : groupCanvas;
  const displayedConnectors = groupProjection.mode === "live" && pendingCanvasDocumentDraft
    ? pendingCanvasDocumentDraft.connectors
    : connectors;
  const displayedFrames = displayedCanvas?.frames ?? [];
  const selectedFrame = displayedFrames.find((frame) => frame.id === selectedCanvasFrameId) ?? displayedFrames[0];
  const selectedConnector = displayedConnectors.find((connector) => connector.id === selectedCanvasConnectorId) ?? displayedConnectors[0];
  const selectedCanvasElement = groupProjection.mode === "live" && selectedCanvasElementIds.length === 1
    ? groupProjection.elements.find((element) => element.id === selectedCanvasElementIds[0])
    : undefined;
  const selectedCanvasElementCanEditText = Boolean(
    selectedCanvasElement &&
    (selectedCanvasElement.kind === "text" || selectedCanvasElement.kind === "sticky_note") &&
    !selectedCanvasElement.locked &&
    !selectedCanvasElement.entity_ref &&
    !selectedCanvasElement.content.work_item_id &&
    !selectedCanvasElement.content.worktree_id &&
    !selectedCanvasElement.content.agent_session_id &&
    !selectedCanvasElement.content.automation_id &&
    !selectedCanvasElement.content.comment_id,
  );
  useEffect(() => {
    setCanvasElementTextDraft(selectedCanvasElement?.content.text ?? "");
    setCanvasElementContentError(null);
    setCanvasElementContentSaved(false);
    setCanvasElementGeometryDraft(selectedCanvasElement
      ? { width: selectedCanvasElement.width, height: selectedCanvasElement.height, rotation: selectedCanvasElement.rotation }
      : null);
    setCanvasElementGeometryError(null);
    setCanvasElementGeometrySaved(false);
  }, [selectedCanvasElement?.id]);
  const linkedWorkItemIds = new Set(elements.flatMap((element) =>
    element.entity_ref?.ref_type === "work_item" ? [element.entity_ref.ref_id] : [],
  ));
  const unlinkedCanvasWorkItems = groupProjection.mode === "live"
    ? groupProjection.work_items.filter((item) => !linkedWorkItemIds.has(item.id) && item.id !== linkedCanvasWorkItemId)
    : [];
  const selectedTaskCanvasElement = selectedTask
    ? elements.find((element) =>
        element.entity_ref?.ref_type === "work_item" &&
        element.entity_ref.ref_id === selectedTask.id &&
        element.entity_ref.worktree_id === worktreeId,
      )
    : undefined;
  const basePath = `/worktree/${encodeURIComponent(worktreeId)}/group`;
  function taskHref(workItemId: string, cli = false) {
    const query = new URLSearchParams({ app: "task-card", work_item_id: workItemId, scope });
    if (cli) query.set("cli", "1");
    return `${basePath}?${query.toString()}`;
  }

  function appHref(appId: AppId, params: Record<string, string> = {}) {
    const query = new URLSearchParams({ app: appId, scope, ...params });
    return `${basePath}?${query.toString()}`;
  }

  function changeScope(nextScope: ChatScope) {
    if (nextScope === "GLOBAL") setSelectedChatTargetIds([]);
    const query = new URLSearchParams(searchParams.toString());
    query.set("scope", nextScope);
    router.replace(`${basePath}?${query.toString()}`, { scroll: false });
  }

  function openCanvasTask(workItemId: string, refWorktreeId?: string) {
    const taskIsBoundHere = projectWorkItems.some(
      (item) => item.id === workItemId && item.worktree_id === worktreeId,
    );
    if (refWorktreeId !== worktreeId || !taskIsBoundHere) {
      setCanvasLinkError("此 Canvas 引用没有当前 Worktree 的显式授权关联，已阻止跳转。请先在当前 Worktree 中建立 canonical 任务关联。");
      return;
    }
    setCanvasLinkError(null);
    router.push(taskHref(workItemId));
  }

  function toggleGlobalChatTarget(targetId: string) {
    setSelectedChatTargetIds((current) => {
      if (current.includes(targetId)) return current.filter((id) => id !== targetId);
      if (current.length >= 20) return current;
      return [...current, targetId];
    });
  }

  async function loadMoreGlobalChatTargets() {
    if (!groupApi || !chatTargetCursor || chatTargetsPending) return;
    setChatTargetsPending(true);
    setChatTargetsError(null);
    try {
      const page = await groupApi.listGlobalChatTargets(worktreeId, { limit: 50, cursor: chatTargetCursor });
      setChatTargets((current) => {
        const seen = new Set(current.map((target) => target.worktree_id));
        return [...current, ...page.targets.filter((target) => !seen.has(target.worktree_id))];
      });
      setChatTargetCursor(page.next_cursor);
    } catch (error) {
      setChatTargetsError(error instanceof Error ? error.message : "无法读取更多已授权 Worktree");
    } finally {
      setChatTargetsPending(false);
    }
  }

  function selectCanvas(canvasId: string) {
    const query = new URLSearchParams(searchParams.toString());
    query.set("canvas_id", canvasId);
    router.replace(`${basePath}?${query.toString()}`, { scroll: false });
  }

  function clearCanvasBootstrapCommand() {
    if (groupProjection.mode === "live") {
      for (const [key, command] of canvasBootstrapCommands.current) {
        if (command.canvasId && groupProjection.canvases.some((canvas) => canvas.id === command.canvasId)) {
          canvasBootstrapCommands.current.delete(key);
        }
      }
    }
  }

  function clearCanvasDocumentDraft() {
    canvasDocumentCommand.current = null;
    setPendingCanvasDocumentDraft(null);
    setSelectedCanvasElementIds([]);
    setSelectedCanvasFrameId("");
    setSelectedCanvasConnectorId("");
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  async function createCanvasTask(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const title = canvasTaskTitle.trim();
    if (groupProjection.mode !== "live" || !groupApi || !groupProjection.canvas || !title) return;
    const requestGeneration = groupApiGeneration.current;
    setCanvasTaskPending(true);
    setCanvasTaskError(null);
    setCreatedCanvasTaskId(null);
    try {
      const intentKey = JSON.stringify([worktreeId, groupProjection.canvas.id, title, canvasTaskDescription.trim()]);
      let command = canvasTaskCommands.current.get(intentKey);
      if (!command) {
        const viewport = groupProjection.canvas.viewport;
        const zoom = Math.max(0.1, viewport.zoom);
        const x = Math.max(-900_000, Math.min(900_000, viewport.x + 600 / zoom));
        const y = Math.max(-900_000, Math.min(900_000, viewport.y + 400 / zoom));
        command = {
          correlationId: crypto.randomUUID(),
          idempotencyKey: crypto.randomUUID(),
          body: {
            item_type: "task",
            title,
            description: canvasTaskDescription.trim(),
            priority: "medium",
            labels: [],
            ai_task_data: null,
            x,
            y,
            width: 240,
            height: 120,
            rotation: 0,
            z_index: elements.reduce((maximum, element) => Math.max(maximum, element.z_index), 0) + 1,
          },
        };
        canvasTaskCommands.current.set(intentKey, command);
      }
      const response = await groupApi.createWorkItemOnCanvas<{
        work_item: { work_item_id: string };
      }>(worktreeId, groupProjection.canvas.id, {
        ...command.body,
        correlation_id: command.correlationId,
      }, command.idempotencyKey);
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasTaskCommands.current.delete(intentKey);
      setCreatedCanvasTaskId(response.work_item.work_item_id);
      setCanvasTaskTitle("");
      setCanvasTaskDescription("");
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasTaskError(error instanceof Error ? error.message : "无法从 Canvas 创建 Task Card。");
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setCanvasTaskPending(false);
    }
  }

  async function createWorktreeCanvas(title = "Worktree Canvas") {
    if (groupProjection.mode !== "live" || !groupApi || canvasBootstrapPending) return;
    const requestGeneration = groupApiGeneration.current;
    setCanvasBootstrapPending(true);
    setCanvasBootstrapError(null);
    try {
      const commandKey = JSON.stringify([worktreeId, title]);
      let command = canvasBootstrapCommands.current.get(commandKey);
      if (!command) {
        command = { correlationId: crypto.randomUUID(), idempotencyKey: crypto.randomUUID() };
        canvasBootstrapCommands.current.set(commandKey, command);
      }
      const response = await groupApi.createCanvas<{ canvas: { canvas_id: string } }>(worktreeId, {
        title,
        correlation_id: command.correlationId,
      }, command.idempotencyKey);
      if (groupApiGeneration.current !== requestGeneration) return;
      command.canvasId = response.canvas.canvas_id;
      selectCanvas(response.canvas.canvas_id);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasBootstrapError(error instanceof Error ? error.message : "无法创建当前 Worktree 的 Canvas。");
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setCanvasBootstrapPending(false);
    }
  }

  async function linkExistingCanvasWorkItem(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (groupProjection.mode !== "live" || !groupApi || !groupProjection.canvas || !selectedExistingWorkItemId || canvasElementLinkPending) return;
    const selectedWorkItem = groupProjection.work_items.find((item) => item.id === selectedExistingWorkItemId);
    if (!selectedWorkItem || selectedWorkItem.worktree_id !== worktreeId) {
      setCanvasElementLinkError("只能关联当前 Worktree 中已授权的 Task Card。");
      return;
    }

    const requestGeneration = groupApiGeneration.current;
    const intentKey = JSON.stringify([worktreeId, groupProjection.canvas.id, selectedWorkItem.id]);
    setCanvasElementLinkPending(true);
    setCanvasElementLinkError(null);
    setLinkedCanvasWorkItemId(null);
    try {
      let command = canvasElementLinkCommands.current.get(intentKey);
      if (!command) {
        const viewport = groupProjection.canvas.viewport;
        const zoom = Math.max(0.1, viewport.zoom);
        const commandCorrelationId = crypto.randomUUID();
        command = {
          correlationId: commandCorrelationId,
          idempotencyKey: crypto.randomUUID(),
          body: {
            kind: "work_item_card",
            x: Math.max(-900_000, Math.min(900_000, viewport.x + 600 / zoom)),
            y: Math.max(-900_000, Math.min(900_000, viewport.y + 400 / zoom)),
            width: 240,
            height: 120,
            rotation: 0,
            z_index: elements.reduce((maximum, element) => Math.max(maximum, element.z_index), 0) + 1,
            content: {},
            entity_ref: { ref_type: "work_item", ref_id: selectedWorkItem.id, worktree_id: worktreeId },
            correlation_id: commandCorrelationId,
          },
        };
        canvasElementLinkCommands.current.set(intentKey, command);
      }
      await groupApi.createCanvasElement(
        worktreeId,
        groupProjection.canvas.id,
        command.body,
        command.idempotencyKey,
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasElementLinkCommands.current.delete(intentKey);
      setLinkedCanvasWorkItemId(selectedWorkItem.id);
      setSelectedExistingWorkItemId("");
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasElementLinkError(error instanceof Error ? error.message : "无法将 Task Card 关联到当前 Canvas。");
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setCanvasElementLinkPending(false);
    }
  }

  function stageCanvasViewport(viewport: CanvasViewport) {
    const draft = getCanvasDocumentDraft();
    if (!draft) return;
    setPendingCanvasDocumentDraft({ ...draft, viewport });
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function getCanvasDocumentDraft(): CanvasDocumentDraft | null {
    if (pendingCanvasDocumentDraft) return pendingCanvasDocumentDraft;
    if (groupProjection.mode !== "live" || !groupProjection.canvas) return null;
    return {
      expectedVersion: groupProjection.canvas.version,
      viewport: groupProjection.canvas.viewport,
      frames: groupProjection.canvas.frames,
      connectors: groupProjection.connectors,
    };
  }

  function stageCanvasFrame(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = getCanvasDocumentDraft();
    const title = canvasFrameTitle.trim();
    if (!draft || !groupCanvas || !title || title.length > 200 || draft.frames.length >= 1_000) return;
    const frame: CanvasFrame = {
      id: crypto.randomUUID(),
      canvas_id: groupCanvas.id,
      title,
      x: draft.viewport.x + 80,
      y: draft.viewport.y + 80,
      width: 640,
      height: 360,
      element_ids: [],
      is_slide: false,
      order: draft.frames.reduce((maxOrder, item) => Math.max(maxOrder, item.order), -1) + 1,
    };
    setPendingCanvasDocumentDraft({ ...draft, frames: [...draft.frames, frame] });
    setSelectedCanvasFrameId(frame.id);
    setCanvasFrameTitle("");
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function removeCanvasFrame(frameId: string) {
    const draft = getCanvasDocumentDraft();
    if (!draft) return;
    setPendingCanvasDocumentDraft({ ...draft, frames: draft.frames.filter((frame) => frame.id !== frameId) });
    if (selectedCanvasFrameId === frameId) setSelectedCanvasFrameId("");
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function addSelectedElementsToCanvasFrame() {
    const draft = getCanvasDocumentDraft();
    const selectedIds = selectedCanvasElementIds.filter((id) => elements.some((element) => element.id === id));
    if (!draft || !selectedCanvasFrameId || selectedIds.length === 0) return;
    setPendingCanvasDocumentDraft({
      ...draft,
      frames: draft.frames.map((frame) => frame.id === selectedCanvasFrameId
        ? { ...frame, element_ids: [...new Set([...frame.element_ids, ...selectedIds])] }
        : frame),
    });
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function removeSelectedElementsFromCanvasFrame() {
    const draft = getCanvasDocumentDraft();
    const selectedIds = new Set(selectedCanvasElementIds);
    if (!draft || !selectedCanvasFrameId || selectedIds.size === 0) return;
    setPendingCanvasDocumentDraft({
      ...draft,
      frames: draft.frames.map((frame) => frame.id === selectedCanvasFrameId
        ? { ...frame, element_ids: frame.element_ids.filter((elementId) => !selectedIds.has(elementId)) }
        : frame),
    });
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function addCanvasVisualConnector() {
    const draft = getCanvasDocumentDraft();
    const selectedIds = selectedCanvasElementIds.filter((id) => elements.some((element) => element.id === id));
    if (!draft || selectedIds.length !== 2 || selectedIds[0] === selectedIds[1] || draft.connectors.length >= 5_000 || !groupCanvas) return;
    const connector: CanvasConnector = {
      id: crypto.randomUUID(),
      canvas_id: groupCanvas.id,
      kind: "free",
      from_element_id: selectedIds[0],
      to_element_id: selectedIds[1],
      routing: "curved",
      arrow_start: false,
      arrow_end: true,
      color: "#79c0ff",
      width: 2,
    };
    setPendingCanvasDocumentDraft({ ...draft, connectors: [...draft.connectors, connector] });
    setSelectedCanvasConnectorId(connector.id);
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function removeCanvasVisualConnector(connectorId: string) {
    const draft = getCanvasDocumentDraft();
    if (!draft) return;
    setPendingCanvasDocumentDraft({ ...draft, connectors: draft.connectors.filter((connector) => connector.id !== connectorId) });
    if (selectedCanvasConnectorId === connectorId) setSelectedCanvasConnectorId("");
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function updateCanvasFrame(frameId: string, patch: Partial<CanvasFrame>) {
    const draft = getCanvasDocumentDraft();
    if (!draft) return;
    setPendingCanvasDocumentDraft({
      ...draft,
      frames: draft.frames.map((frame) => frame.id === frameId ? { ...frame, ...patch } : frame),
    });
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  function updateCanvasConnector(connectorId: string, patch: Partial<CanvasConnector>) {
    const draft = getCanvasDocumentDraft();
    if (!draft) return;
    setPendingCanvasDocumentDraft({
      ...draft,
      connectors: draft.connectors.map((connector) => connector.id === connectorId ? { ...connector, ...patch } : connector),
    });
    setCanvasDocumentError(null);
    setCanvasDocumentSaved(false);
  }

  async function saveCanvasElementPosition(position: { elementId: string; x: number; y: number; expectedVersion: number }) {
    if (
      groupProjection.mode !== "live" ||
      !groupApi ||
      !groupProjection.canvas ||
      canvasElementMovePendingRef.current
    ) return;

    const element = groupProjection.elements.find((candidate) => candidate.id === position.elementId);
    if (!element || element.version !== position.expectedVersion) {
      setCanvasElementMoveError("画布元素已被其他操作更新。已刷新当前投影，请重新拖动。");
      setProjectionRefreshKey((current) => current + 1);
      return;
    }

    const requestGeneration = groupApiGeneration.current;
    canvasElementMovePendingRef.current = true;
    setCanvasElementMovePending(true);
    setCanvasElementMoveError(null);
    setCanvasElementMoveSaved(false);
    const fingerprint = JSON.stringify([
      worktreeId,
      groupProjection.canvas.id,
      position.elementId,
      position.expectedVersion,
      position.x,
      position.y,
    ]);
    let command = canvasElementMoveCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      const body = {
        expected_version: position.expectedVersion,
        update_mode: "position",
        x: position.x,
        y: position.y,
        correlation_id: correlationId,
      };
      command = {
        fingerprint,
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body,
      };
      canvasElementMoveCommands.current.set(fingerprint, command);
    }

    try {
      await groupApi.updateCanvasElement(
        worktreeId,
        groupProjection.canvas.id,
        position.elementId,
        command.body,
        command.idempotencyKey,
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasElementMoveCommands.current.delete(fingerprint);
      setCanvasElementMoveSaved(true);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasElementMoveError(error instanceof Error ? error.message : "无法保存 Canvas 元素位置。");
        setProjectionRefreshKey((current) => current + 1);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) {
        canvasElementMovePendingRef.current = false;
        setCanvasElementMovePending(false);
      }
    }
  }

  async function deleteCanvasElements(elementIds: string[]) {
    if (
      groupProjection.mode !== "live" ||
      !groupApi ||
      !groupProjection.canvas ||
      canvasElementDeletePendingRef.current
    ) return;
    const selectedElementIds = new Set(elementIds);
    const selectedElements = groupProjection.elements.filter((element) => selectedElementIds.has(element.id));
    if (selectedElements.length === 0) return;

    const requestGeneration = groupApiGeneration.current;
    canvasElementDeletePendingRef.current = true;
    setCanvasElementDeletePending(true);
    setCanvasElementDeleteError(null);
    setCanvasElementDeleteSaved(false);
    try {
      for (const element of selectedElements) {
        const fingerprint = JSON.stringify([
          worktreeId,
          groupProjection.canvas.id,
          element.id,
          element.version,
        ]);
        let command = canvasElementDeleteCommands.current.get(fingerprint);
        if (!command) {
          const correlationId = crypto.randomUUID();
          command = {
            fingerprint,
            correlationId,
            idempotencyKey: crypto.randomUUID(),
            body: { expected_version: element.version, correlation_id: correlationId },
          };
          canvasElementDeleteCommands.current.set(fingerprint, command);
        }
        await groupApi.deleteCanvasElement(
          worktreeId,
          groupProjection.canvas.id,
          element.id,
          command.body,
          command.idempotencyKey,
        );
        if (groupApiGeneration.current !== requestGeneration) return;
        canvasElementDeleteCommands.current.delete(fingerprint);
      }
      setSelectedCanvasElementIds([]);
      setCanvasElementDeleteSaved(true);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasElementDeleteError(error instanceof Error ? error.message : "无法删除 Canvas 元素。");
        setProjectionRefreshKey((current) => current + 1);
      }
      throw error;
    } finally {
      if (groupApiGeneration.current === requestGeneration) {
        canvasElementDeletePendingRef.current = false;
        setCanvasElementDeletePending(false);
      }
    }
  }

  async function saveCanvasElementContent() {
    if (
      groupProjection.mode !== "live" ||
      !groupApi ||
      !groupProjection.canvas ||
      !selectedCanvasElement ||
      !selectedCanvasElementCanEditText ||
      canvasElementContentPendingRef.current
    ) return;
    if (canvasElementTextDraft.length > 16_000) {
      setCanvasElementContentError("便笺文字不能超过 16,000 个字符。");
      return;
    }
    const element = selectedCanvasElement;
    const requestGeneration = groupApiGeneration.current;
    const fingerprint = JSON.stringify([
      worktreeId,
      groupProjection.canvas.id,
      element.id,
      element.version,
      canvasElementTextDraft,
    ]);
    let command = canvasElementContentCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      command = {
        fingerprint,
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: {
          expected_version: element.version,
          update_mode: "content",
          content: { ...element.content, text: canvasElementTextDraft },
          correlation_id: correlationId,
        },
      };
      canvasElementContentCommands.current.set(fingerprint, command);
    }
    canvasElementContentPendingRef.current = true;
    setCanvasElementContentPending(true);
    setCanvasElementContentError(null);
    setCanvasElementContentSaved(false);
    try {
      await groupApi.updateCanvasElement(
        worktreeId,
        groupProjection.canvas.id,
        element.id,
        command.body,
        command.idempotencyKey,
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasElementContentCommands.current.delete(fingerprint);
      setCanvasElementContentSaved(true);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasElementContentError(error instanceof Error ? error.message : "无法保存 Canvas 便笺文字。");
        setProjectionRefreshKey((current) => current + 1);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) {
        canvasElementContentPendingRef.current = false;
        setCanvasElementContentPending(false);
      }
    }
  }

  async function saveCanvasElementGeometry() {
    if (
      groupProjection.mode !== "live" ||
      !groupApi ||
      !groupProjection.canvas ||
      !selectedCanvasElement ||
      selectedCanvasElement.locked ||
      !canvasElementGeometryDraft ||
      canvasElementGeometryPendingRef.current
    ) return;
    const draft = canvasElementGeometryDraft;
    if (
      !Number.isFinite(draft.width) || draft.width <= 0 || draft.width > 10_000 ||
      !Number.isFinite(draft.height) || draft.height <= 0 || draft.height > 10_000 ||
      !Number.isFinite(draft.rotation) || Math.abs(draft.rotation) > 36_000
    ) {
      setCanvasElementGeometryError("尺寸须大于 0 且不超过 10,000；旋转角度须在 ±36,000° 内。");
      return;
    }
    const element = selectedCanvasElement;
    const requestGeneration = groupApiGeneration.current;
    const fingerprint = JSON.stringify([
      worktreeId,
      groupProjection.canvas.id,
      element.id,
      element.version,
      draft.width,
      draft.height,
      draft.rotation,
    ]);
    let command = canvasElementGeometryCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      command = {
        fingerprint,
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: {
          expected_version: element.version,
          update_mode: "geometry",
          width: draft.width,
          height: draft.height,
          rotation: draft.rotation,
          correlation_id: correlationId,
        },
      };
      canvasElementGeometryCommands.current.set(fingerprint, command);
    }
    canvasElementGeometryPendingRef.current = true;
    setCanvasElementGeometryPending(true);
    setCanvasElementGeometryError(null);
    setCanvasElementGeometrySaved(false);
    try {
      await groupApi.updateCanvasElement(
        worktreeId,
        groupProjection.canvas.id,
        element.id,
        command.body,
        command.idempotencyKey,
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasElementGeometryCommands.current.delete(fingerprint);
      setCanvasElementGeometrySaved(true);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCanvasElementGeometryError(error instanceof Error ? error.message : "无法保存 Canvas 元素尺寸与旋转角度。");
        setProjectionRefreshKey((current) => current + 1);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) {
        canvasElementGeometryPendingRef.current = false;
        setCanvasElementGeometryPending(false);
      }
    }
  }

  async function startSelectedTaskCliSession() {
    if (groupProjection.mode !== "live" || !groupApi || !selectedTask || cliSessionPending) return;
    if (
      selectedTask.worktree_id !== worktreeId ||
      !("lifecycle_version" in selectedTask) ||
      selectedTask.lifecycle_status !== "in_progress" ||
      selectedTask.active_worktree_id !== worktreeId
    ) {
      setCliSessionError("任务必须处于当前 Worktree 的进行中状态，且服务端还会复核 active claimant。");
      return;
    }
    const profileId = cliProfileId.trim();
    if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(profileId)) {
      setCliSessionError("请填写管理员提供的 Approved Launch Profile UUID。");
      return;
    }

    const item = selectedTask;
    const requestGeneration = groupApiGeneration.current;
    const fingerprint = JSON.stringify([worktreeId, item.id, item.lifecycle_version, profileId]);
    let command = cliSessionCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      command = {
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: {
          expected_lifecycle_version: item.lifecycle_version,
          approved_launch_profile_id: profileId,
          correlation_id: correlationId,
        },
      };
      cliSessionCommands.current.set(fingerprint, command);
    }

    setCliSessionPending(true);
    setCliSessionError(null);
    try {
      const receipt = await groupApi.startTaskCliSession(
        worktreeId,
        item.id,
        command.body,
        command.idempotencyKey,
      );
      const now = Date.now();
      const expiry = Date.parse(receipt.attachment_ticket_expires_at);
      if (
        receipt.status !== "running" ||
        receipt.worktree_id !== worktreeId ||
        receipt.work_item_id !== item.id ||
        !receipt.session_id ||
        !receipt.runtime_id ||
        !receipt.attachment_ticket ||
        !Number.isFinite(expiry) ||
        expiry <= now ||
        expiry > now + 60_000
      ) {
        throw new Error("Task CLI provisioner returned an invalid session or attachment ticket.");
      }
      if (groupApiGeneration.current !== requestGeneration) return;
      cliSessionCommands.current.delete(fingerprint);
      cliTicketClaimed.current = false;
      setCliSessionStatus(null);
      setCliWsConnected(false);
      setCliNeedsReattach(false);
      setCliSession({ ...receipt, requested_worktree_id: worktreeId });
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCliSessionError(error instanceof Error ? error.message : "无法启动受控 Task CLI Session。");
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setCliSessionPending(false);
    }
  }

  function applyTaskCliSessionStatus(status: TaskCliSessionStatus) {
    setCliSessionHistory((current) => current.map((entry) => entry.session_id === status.session_id ? status : entry));
    if (cliSessionForSelectedTask?.session_id === status.session_id) {
      setCliSessionStatus(status);
      if (status.state !== "running" && status.state !== "disconnected") setCliNeedsReattach(false);
    }
  }

  async function refreshTaskCliSessionStatus(sessionId: string) {
    if (
      groupProjection.mode !== "live" || !groupApi || !selectedTask ||
      selectedTask.worktree_id !== worktreeId || cliSessionControlRequest.current
    ) return;

    const requestGeneration = groupApiGeneration.current;
    const controlRequestId = crypto.randomUUID();
    cliSessionControlRequest.current = controlRequestId;
    setCliSessionControlPending(true);
    setCliSessionError(null);
    try {
      const status = await groupApi.getTaskCliSessionStatus(
        worktreeId,
        selectedTask.id,
        sessionId,
        crypto.randomUUID(),
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      if (status.session_id !== sessionId) throw new Error("Task CLI status response did not match the selected session.");
      applyTaskCliSessionStatus(status);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCliSessionError(error instanceof Error ? error.message : "无法读取 Task CLI Session 状态。");
      }
    } finally {
      if (cliSessionControlRequest.current === controlRequestId) {
        cliSessionControlRequest.current = null;
        if (groupApiGeneration.current === requestGeneration) setCliSessionControlPending(false);
      }
    }
  }

  async function reattachTaskCliSession(
    sessionId: string,
    expectedRuntimeId?: string,
    previousTicket?: string,
  ) {
    if (
      groupProjection.mode !== "live" || !groupApi || !selectedTask ||
      selectedTask.worktree_id !== worktreeId || cliSessionControlRequest.current || cliWsConnected
    ) return;

    const requestGeneration = groupApiGeneration.current;
    const controlRequestId = crypto.randomUUID();
    cliSessionControlRequest.current = controlRequestId;
    setCliSessionControlPending(true);
    setCliSessionError(null);
    try {
      const status = await groupApi.getTaskCliSessionStatus(
        worktreeId,
        selectedTask.id,
        sessionId,
        crypto.randomUUID(),
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      if (status.session_id !== sessionId) throw new Error("Task CLI status response did not match the selected session.");
      applyTaskCliSessionStatus(status);
      if (status.state !== "running" && status.state !== "disconnected") {
        throw new Error(`当前 Session 状态为 ${status.state}，不能重新连接。`);
      }

      const correlationId = crypto.randomUUID();
      const receipt = await groupApi.reattachTaskCliSession(
        worktreeId,
        selectedTask.id,
        sessionId,
        correlationId,
      );
      const now = Date.now();
      const expiry = Date.parse(receipt.attachment_ticket_expires_at);
      if (
        receipt.status !== "attachment_ticket_issued" ||
        receipt.session_id !== sessionId ||
        receipt.worktree_id !== worktreeId ||
        receipt.work_item_id !== selectedTask.id ||
        !receipt.runtime_id ||
        (expectedRuntimeId !== undefined && receipt.runtime_id !== expectedRuntimeId) ||
        receipt.correlation_id !== correlationId ||
        receipt.attachment_ticket === previousTicket ||
        !receipt.attachment_ticket ||
        receipt.attachment_ticket.length > 512 ||
        !Number.isFinite(expiry) ||
        expiry <= now ||
        expiry > now + 60_000
      ) {
        throw new Error("Task CLI provisioner returned an invalid reattachment ticket.");
      }
      if (groupApiGeneration.current !== requestGeneration) return;

      cliTicketClaimed.current = false;
      setCliNeedsReattach(false);
      setCliWsConnected(false);
      setCliSession({ ...receipt, status: "running", requested_worktree_id: worktreeId });
      setCliTerminalGeneration((generation) => generation + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCliSessionError(error instanceof Error ? error.message : "无法重新连接 Task CLI Session。");
      }
    } finally {
      if (cliSessionControlRequest.current === controlRequestId) {
        cliSessionControlRequest.current = null;
        if (groupApiGeneration.current === requestGeneration) setCliSessionControlPending(false);
      }
    }
  }

  async function cancelTaskCliSession(sessionId: string) {
    if (
      groupProjection.mode !== "live" || !groupApi || !selectedTask ||
      selectedTask.worktree_id !== worktreeId || cliSessionControlRequest.current
    ) return;

    const requestGeneration = groupApiGeneration.current;
    const controlRequestId = crypto.randomUUID();
    cliSessionControlRequest.current = controlRequestId;
    setCliSessionControlPending(true);
    setCliSessionError(null);
    try {
      const status = await groupApi.cancelTaskCliSession(
        worktreeId,
        selectedTask.id,
        sessionId,
        crypto.randomUUID(),
      );
      if (groupApiGeneration.current !== requestGeneration) return;
      if (status.session_id !== sessionId) throw new Error("Task CLI cancel response did not match the selected session.");
      applyTaskCliSessionStatus(status);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setCliSessionError(error instanceof Error ? error.message : "无法结束 Task CLI Session。");
      }
    } finally {
      if (cliSessionControlRequest.current === controlRequestId) {
        cliSessionControlRequest.current = null;
        if (groupApiGeneration.current === requestGeneration) setCliSessionControlPending(false);
      }
    }
  }

  function refreshSelectedTaskCliSessionStatus() {
    const current = cliSessionForSelectedTask;
    if (current) void refreshTaskCliSessionStatus(current.session_id);
  }

  function reattachSelectedTaskCliSession() {
    const current = cliSessionForSelectedTask;
    if (current) void reattachTaskCliSession(current.session_id, current.runtime_id, current.attachment_ticket);
  }

  function cancelSelectedTaskCliSession() {
    const current = cliSessionForSelectedTask;
    if (current) void cancelTaskCliSession(current.session_id);
  }

  async function transitionLiveWorkItem(
    item: GroupWorkItemView,
    targetStatus: LifecycleTargetStatus,
  ) {
    if (groupProjection.mode !== "live" || !groupApi || pendingLifecycleWorkItemId) return;

    const requestGeneration = groupApiGeneration.current;
    const fingerprint = JSON.stringify([worktreeId, item.id, item.lifecycle_version, targetStatus]);
    let command = lifecycleCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      command = {
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: {
          target_status: targetStatus,
          expected_version: item.lifecycle_version,
          correlation_id: correlationId,
        },
      };
      lifecycleCommands.current.set(fingerprint, command);
    }

    setPendingLifecycleWorkItemId(item.id);
    setLifecycleError(null);
    try {
      await groupApi.transitionWorkItem(worktreeId, item.id, command.body, command.idempotencyKey);
      if (groupApiGeneration.current !== requestGeneration) return;
      lifecycleCommands.current.delete(fingerprint);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setLifecycleError({
          workItemId: item.id,
          message: error instanceof Error ? error.message : "任务生命周期命令失败。",
        });
        setProjectionRefreshKey((current) => current + 1);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setPendingLifecycleWorkItemId(null);
    }
  }

  async function reviewLiveWorkItem(item: GroupWorkItemView, action: ReviewAction) {
    if (groupProjection.mode !== "live" || !groupApi || pendingLifecycleWorkItemId) return;
    const reason = action === "reject" ? reviewReasons[item.id]?.trim() : undefined;
    if (action === "reject" && !reason) {
      setLifecycleError({ workItemId: item.id, message: "驳回评审必须填写原因。" });
      return;
    }

    const requestGeneration = groupApiGeneration.current;
    const fingerprint = JSON.stringify([worktreeId, item.id, item.lifecycle_version, action, reason ?? ""]);
    let command = reviewCommands.current.get(fingerprint);
    if (!command) {
      const correlationId = crypto.randomUUID();
      command = {
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: {
          action,
          expected_version: item.lifecycle_version,
          correlation_id: correlationId,
          ...(reason ? { reason } : {}),
        },
      };
      reviewCommands.current.set(fingerprint, command);
    }

    setPendingLifecycleWorkItemId(item.id);
    setLifecycleError(null);
    try {
      await groupApi.reviewWorkItem(worktreeId, item.id, command.body, command.idempotencyKey);
      if (groupApiGeneration.current !== requestGeneration) return;
      reviewCommands.current.delete(fingerprint);
      if (action === "reject") setReviewReasons((current) => ({ ...current, [item.id]: "" }));
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        setLifecycleError({
          workItemId: item.id,
          message: error instanceof Error ? error.message : "评审命令失败。",
        });
        setProjectionRefreshKey((current) => current + 1);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setPendingLifecycleWorkItemId(null);
    }
  }

  async function saveCanvasDocument() {
    if (
      groupProjection.mode !== "live" ||
      !groupApi ||
      !groupProjection.canvas ||
      !pendingCanvasDocumentDraft ||
      canvasDocumentSavePending
    ) return;

    const requestGeneration = groupApiGeneration.current;
    setCanvasDocumentSavePending(true);
    setCanvasDocumentError(null);
    const document = {
      expected_version: pendingCanvasDocumentDraft.expectedVersion,
      viewport: pendingCanvasDocumentDraft.viewport,
      frames: pendingCanvasDocumentDraft.frames,
      connectors: pendingCanvasDocumentDraft.connectors.map((connector) => ({
        id: connector.id,
        canvas_id: connector.canvas_id,
        kind: connector.kind,
        from_element_id: connector.from_element_id,
        to_element_id: connector.to_element_id,
        routing: connector.routing,
        arrow_start: connector.arrow_start,
        arrow_end: connector.arrow_end,
        color: connector.color,
        width: connector.width,
        label: connector.label,
      })),
    };
    const fingerprint = JSON.stringify([worktreeId, groupProjection.canvas.id, document]);
    let command = canvasDocumentCommand.current;
    if (!command || command.fingerprint !== fingerprint) {
      const correlationId = crypto.randomUUID();
      command = {
        fingerprint,
        correlationId,
        idempotencyKey: crypto.randomUUID(),
        body: { ...document, correlation_id: correlationId },
      };
      canvasDocumentCommand.current = command;
    }

    try {
      await groupApi.updateCanvasDocument(
        worktreeId,
        groupProjection.canvas.id,
        command.body,
        command.idempotencyKey,
    );
      if (groupApiGeneration.current !== requestGeneration) return;
      canvasDocumentCommand.current = null;
      setPendingCanvasDocumentDraft(null);
      setSelectedCanvasElementIds([]);
      setSelectedCanvasFrameId("");
      setSelectedCanvasConnectorId("");
      setCanvasDocumentSaved(true);
      setProjectionRefreshKey((current) => current + 1);
    } catch (error) {
      if (groupApiGeneration.current === requestGeneration) {
        const reason = error instanceof Error ? error.message : "无法保存 Canvas 文档。";
        setCanvasDocumentError(`${reason} 草稿仍保留，可使用同一命令重试；若发生版本冲突，请放弃草稿并加载最新版本。`);
      }
    } finally {
      if (groupApiGeneration.current === requestGeneration) setCanvasDocumentSavePending(false);
    }
  }

  if (groupProjection.mode === "loading") {
    return <main className="mx-auto w-full max-w-4xl p-6"><section className="card" role="status">正在读取已授权的 Worktree Group 数据…</section></main>;
  }

  if (groupProjection.mode === "error") {
    return (
      <main className="mx-auto w-full max-w-4xl p-6">
        <section className="card space-y-3" role="alert">
          <h1 className="text-lg font-semibold">无法加载 Worktree Group</h1>
          <p className="text-sm text-ink-dim">{groupProjection.message}</p>
          <p className="text-xs text-warning">已配置认证 API，因此不会回退显示本地 seed 数据。请检查登录会话、当前 Worktree 权限和 API 服务。</p>
          <Link href="/worktree" className="btn w-fit">返回 Worktree Index</Link>
        </section>
      </main>
    );
  }

  if (!worktree) {
    return (
      <main className="mx-auto w-full max-w-4xl p-6">
        <section className="card space-y-3">
          <h1 className="text-lg font-semibold">找不到 Worktree</h1>
          <p className="text-sm text-ink-dim">当前 Worktree ID：{worktreeId}</p>
          <Link href="/worktree" className="btn w-fit">返回 Worktree Index</Link>
        </section>
      </main>
    );
  }
  const currentWorktreeId = worktree.id;

  function getTaskDisplayStatus(item: (typeof projectWorkItems)[number]): WorkItemStatus {
    if (groupProjection.mode === "live" && "review_state" in item && item.review_state === "pending_review") return "review";
    return item.status;
  }

  function renderLifecycleActions(item: (typeof projectWorkItems)[number]) {
    if (groupProjection.mode !== "live" || !("lifecycle_status" in item)) return null;

    const lifecycleActions: Record<LifecycleTargetStatus, Array<{ target: LifecycleTargetStatus; label: string }>> = {
      pending: [{ target: "claimed", label: "认领" }],
      claimed: [{ target: "in_progress", label: "开始" }, { target: "cancelled", label: "取消" }],
      in_progress: [
        ...(item.review_state === "pending_review" ? [] : [{ target: "completed" as const, label: "完成" }]),
        { target: "failed", label: "失败" },
        { target: "cancelled", label: "取消" },
      ],
      completed: [],
      failed: [{ target: "pending", label: "重试" }],
      cancelled: [],
    };
    const actions = lifecycleActions[item.lifecycle_status];
    const runsOnAnotherWorktree =
      ["claimed", "in_progress"].includes(item.lifecycle_status) &&
      item.active_worktree_id !== null &&
      item.active_worktree_id !== currentWorktreeId;

    return (
      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        <span className="rounded border border-line px-1.5 py-0.5 text-[9px] text-ink-mute">lifecycle v{item.lifecycle_version}</span>
        {actions.map((action) => (
          <button
            key={action.target}
            type="button"
            disabled={pendingLifecycleWorkItemId !== null || runsOnAnotherWorktree}
            onClick={() => void transitionLiveWorkItem(item, action.target)}
            className="btn text-[10px] disabled:opacity-40"
          >
            {pendingLifecycleWorkItemId === item.id ? "提交中…" : action.label}
          </button>
        ))}
        {item.review_state === "pending_review" && item.lifecycle_status === "in_progress" && (
          <>
            <input
              aria-label={`驳回 ${item.key} 的原因`}
              value={reviewReasons[item.id] ?? ""}
              onChange={(event) => setReviewReasons((current) => ({ ...current, [item.id]: event.target.value }))}
              placeholder="驳回原因（必填）"
              maxLength={4_000}
              className="min-w-48 rounded border border-line bg-bg px-2 py-1 text-[10px]"
            />
            <button
              type="button"
              disabled={pendingLifecycleWorkItemId !== null || runsOnAnotherWorktree}
              onClick={() => void reviewLiveWorkItem(item, "accept")}
              className="btn text-[10px] disabled:opacity-40"
            >通过评审</button>
            <button
              type="button"
              disabled={pendingLifecycleWorkItemId !== null || runsOnAnotherWorktree || !(reviewReasons[item.id] ?? "").trim()}
              onClick={() => void reviewLiveWorkItem(item, "reject")}
              className="btn text-[10px] disabled:opacity-40"
            >驳回评审</button>
          </>
        )}
        {item.lifecycle_status === "in_progress" && item.review_state !== "pending_review" && (
          <button
            type="button"
            disabled={pendingLifecycleWorkItemId !== null || runsOnAnotherWorktree}
            onClick={() => void reviewLiveWorkItem(item, "submit")}
            className="btn text-[10px] disabled:opacity-40"
          >提交评审</button>
        )}
        {runsOnAnotherWorktree && (
          <span className="text-[10px] text-warning">任务当前由另一个 Worktree 执行</span>
        )}
        {lifecycleError?.workItemId === item.id && (
          <span role="alert" className="text-[10px] text-error">{lifecycleError.message} 已刷新任务状态。</span>
        )}
      </div>
    );
  }

  const taskList = (
    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
      {projectWorkItems.map((item) => (
        <article key={item.id} className="card transition hover:border-accent/60" data-testid={`group-task-${item.id}`}>
          <Link href={taskHref(item.id)} className="block rounded-sm hover:bg-bg-soft/60">
            <div className="mb-2 flex items-center justify-between gap-2">
              <span className="font-mono text-xs text-info">{item.key}</span>
              <StatusPill value={getTaskDisplayStatus(item)} size="xs" />
            </div>
            <div className="text-sm font-medium">{item.title}</div>
            <div className="mt-3 flex items-center justify-between text-[10px] text-ink-mute">
              <span>{item.kind} · {item.priority}</span>
              <span>{item.worktree_id === worktree.id ? "已绑定当前 Worktree" : "项目级示例任务"}</span>
            </div>
          </Link>
          {renderLifecycleActions(item)}
        </article>
      ))}
    </div>
  );

  return (
    <main className="min-h-full pb-36">
      <header className="border-b border-line bg-bg-soft/70 px-5 py-4 md:px-7">
        <div className="mx-auto flex max-w-[1500px] flex-wrap items-center justify-between gap-4">
          <div className="min-w-0">
            <div className="mb-1 flex items-center gap-2 text-[10px] uppercase tracking-[0.18em] text-ink-mute">
              <GitBranch size={12} /> Project · {project?.name ?? "Unknown Project"} / Worktree Group
            </div>
            <h1 className="truncate text-xl font-semibold">{worktree.name}</h1>
            <div className="mt-1 flex flex-wrap items-center gap-2 font-mono text-xs text-ink-dim">
              <span>{worktree.id}</span><span>·</span><span>{worktree.branch}</span>
              <span>·</span><StatusPill value={worktree.status} size="xs" />
            </div>
          </div>
          <div className="flex items-center gap-2">
            {groupProjection.mode === "live"
              ? <span className="rounded border border-success/40 bg-success/10 px-2 py-1 text-[10px] text-success">已授权 API 投影 · 受限写入</span>
              : <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">前端原型 · mock store</span>}
            <Link href={`/worktree?project_id=${encodeURIComponent(worktree.project_id)}`} className="btn text-xs">Project Worktree Index</Link>
          </div>
        </div>
      </header>

      <div className="mx-auto grid max-w-[1500px] gap-5 px-4 py-5 md:grid-cols-[220px_minmax(0,1fr)] md:px-7">
        <nav aria-label="Worktree 群组应用" className="h-fit rounded-lg border border-line bg-bg-card p-2 md:sticky md:top-4">
          <div className="px-2 pb-2 pt-1 text-[10px] font-semibold uppercase tracking-widest text-ink-mute">{worktree.name} · 同级应用</div>
          <div className="grid grid-cols-2 gap-1 md:grid-cols-1">
            {visibleGroupApps.map((app) => {
              const Icon = app.icon;
              const href = appHref(app.id);
              const active = currentApp === app.id;
              return (
                <Link
                  key={app.id}
                  href={href}
                  aria-current={active ? "page" : undefined}
                  className={`flex min-w-0 items-center gap-2 rounded-md px-2.5 py-2 text-left transition ${active ? "bg-accent/15 text-accent" : "text-ink-dim hover:bg-bg-soft hover:text-ink"}`}
                  data-testid={`group-app-${app.id}`}
                >
                  <Icon size={15} className="shrink-0" />
                  <span className="min-w-0">
                    <span className="block truncate text-xs font-medium">{app.label}</span>
                    <span className="hidden truncate text-[9px] text-ink-mute md:block">{app.subtitle}</span>
                  </span>
                </Link>
              );
            })}
          </div>
          <div className="mt-3 border-t border-line px-2 pt-3 text-[10px] leading-relaxed text-ink-mute">
            所有 App 共用 Worktree 上下文与 canonical WorkItem。切换入口不切换根节点。
          </div>
        </nav>

        <section className="min-w-0" aria-live="polite">
          {currentApp === "multica" && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Multica · 生命周期</h2><p className="text-xs text-ink-mute">领取、执行与评审共用 WorkItem 状态源。</p></div>
                {groupProjection.mode === "live"
                  ? <span className="rounded border border-success/40 bg-success/10 px-2 py-1 text-[10px] text-success">服务端生命周期与版本</span>
                  : <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">示例数据只在浏览器 store</span>}
              </div>
              {taskList}
            </div>
          )}

          {currentApp === "jira" && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">Jira 等价视图</h2><p className="text-xs text-ink-mute">Board / Backlog / Sprint 视图复用同一组任务卡。</p></div>
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {(groupProjection.mode === "live"
                  ? ["todo", "in_progress", "review", "blocked", "done", "wontfix"] as WorkItemStatus[]
                  : ["todo", "in_progress", "review"] as WorkItemStatus[]).map((status) => (
                  <section key={status} className="rounded-lg border border-line bg-bg-soft/40 p-3">
                    <h3 className="mb-3 flex items-center justify-between text-xs font-semibold">
                      {STATUS_OPTIONS.find((option) => option.value === status)?.label}
                      <span className="font-mono text-ink-mute">{projectWorkItems.filter((item) => getTaskDisplayStatus(item) === status).length}</span>
                    </h3>
                    <div className="space-y-2">
                      {projectWorkItems.filter((item) => getTaskDisplayStatus(item) === status).map((item) => (
                        <article key={item.id} className="rounded-md border border-line bg-bg-card p-3">
                          <Link href={taskHref(item.id)} className="block hover:text-accent">
                            <div className="font-mono text-[10px] text-info">{item.key}</div>
                            <div className="mt-1 text-xs">{item.title}</div>
                          </Link>
                          {renderLifecycleActions(item)}
                        </article>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
              <p className="text-[10px] text-ink-mute">在线视图按 canonical lifecycle 与 review gate 分类；Board / Backlog / Sprint 的自定义列仍待接入。</p>
            </div>
          )}

          {currentApp === "task-card" && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Task Card Index</h2><p className="text-xs text-ink-mute">Task Card 与 Canvas、Multica、Jira 视图是同级入口。</p></div>
                <span className={`rounded border px-2 py-1 text-[10px] ${cliSessionForSelectedTask ? "border-success/40 bg-success/10 text-success" : "border-warning/40 bg-warning/10 text-warning"}`}>
                  {cliSessionForSelectedTask ? "Task CLI Session" : "CLI 预览 / 等待受控 Runtime"}
                </span>
              </div>
              {selectedTask ? (
                <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.9fr)]">
                  <article className="card space-y-4">
                    <div className="flex items-start justify-between gap-3">
                      <div><div className="font-mono text-xs text-info">{selectedTask.key} · {selectedTask.id}</div><h3 className="mt-2 text-lg font-semibold">{selectedTask.title}</h3></div>
                      <StatusPill value={getTaskDisplayStatus(selectedTask)} />
                    </div>
                    <p className="text-sm text-ink-dim">{selectedTask.description}</p>
                    <div className="flex flex-wrap gap-2 text-[10px] text-ink-mute">
                      <span className="rounded border border-line px-2 py-1">Worktree: {selectedTask.worktree_id ?? "未绑定（seed）"}</span>
                      <span className="rounded border border-line px-2 py-1">Canonical ID: {selectedTask.id}</span>
                    </div>
                    <div>
                      <div className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-ink-mute">{groupProjection.mode === "live" ? "服务端生命周期" : "本地预览状态迁移"}</div>
                      {groupProjection.mode === "live" && "lifecycle_status" in selectedTask
                        ? renderLifecycleActions(selectedTask)
                        : <div className="flex flex-wrap gap-1.5">
                            {STATUS_OPTIONS.map((option) => (
                              <button
                                key={option.value}
                                type="button"
                                disabled={selectedTask.status === option.value}
                                onClick={() => transitionWorkItem(selectedTask.id, option.value)}
                                className="btn text-[10px] disabled:opacity-40"
                              >
                                {option.label}
                              </button>
                            ))}
                          </div>}
                    </div>
                    <div className="flex flex-wrap gap-2 border-t border-line pt-3">
                      {selectedTask.worktree_id === worktreeId
                        ? <Link href={taskHref(selectedTask.id, true)} className="btn-primary text-xs"><CheckSquare size={13} /> 卡内 CLI 预览</Link>
                        : <span className="rounded border border-line px-2 py-1 text-[10px] text-ink-mute">先绑定当前 Worktree 才能打开 CLI</span>}
                      {selectedTaskCanvasElement && <Link href={appHref("canvas", { highlight: selectedTaskCanvasElement.id })} className="btn text-xs"><LayoutGrid size={13} /> 在 Canvas 中定位</Link>}
                      <Link href={appHref("jira")} className="btn text-xs"><ArrowUpRight size={13} /> 打开 Jira 视图</Link>
                    </div>
                  </article>
                  <section className="card min-h-[320px] space-y-3">
                    <div className="flex items-center justify-between gap-3">
                      <div><h3 className="text-sm font-semibold">{cliSessionForSelectedTask ? "卡内 CLI · 已启动" : cliPreviewOpen ? "卡内 CLI" : "执行入口"}</h3><p className="text-[10px] text-ink-mute">{cliSessionForSelectedTask ? `session ${cliSessionForSelectedTask.session_id}` : "命令只能由当前 Worktree 的授权 Local Runtime 执行"}</p></div>
                      {cliPreviewOpen && !cliSessionForSelectedTask && <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">mock preview</span>}
                    </div>
                    {cliSessionForSelectedTask ? (
                      <>
                        <div className="space-y-2 rounded border border-line p-2 text-[10px] leading-relaxed text-ink-dim">
                          <p>状态：{cliSessionStatus?.state ?? "尚未查询"}{cliSessionStatus?.exit_code != null ? ` · exit ${cliSessionStatus.exit_code}` : ""}{cliSessionStatus ? ` · ${cliSessionStatus.updated_at}` : ""}</p>
                          <p>WebSocket ticket 只保存在当前页面内存。断线后手动重连会先检查 session，再请求新的短时 ticket；页面刷新后暂时没有 session 列表接口恢复关联。</p>
                          <div className="flex flex-wrap gap-2">
                            <button type="button" className="btn text-[10px]" onClick={() => void refreshSelectedTaskCliSessionStatus()} disabled={cliSessionControlPending || groupProjection.mode !== "live" || !groupApi}>
                              {cliSessionControlPending ? "处理中…" : "刷新状态"}
                            </button>
                            {cliNeedsReattach && (!cliSessionStatus || cliSessionStatus.state === "running" || cliSessionStatus.state === "disconnected") && <button type="button" className="btn-primary text-[10px]" onClick={() => void reattachSelectedTaskCliSession()} disabled={cliSessionControlPending || cliWsConnected || groupProjection.mode !== "live" || !groupApi}>
                              检查状态并重新连接
                            </button>}
                            <button type="button" className="btn text-[10px]" onClick={() => void cancelSelectedTaskCliSession()} disabled={cliSessionControlPending || groupProjection.mode !== "live" || !groupApi || Boolean(cliSessionStatus && ["completed", "failed", "cancelled", "timed_out", "lost"].includes(cliSessionStatus.state))}>
                              结束 Session
                            </button>
                          </div>
                          {cliSessionError && <p role="alert" className="text-xs text-error">{cliSessionError}</p>}
                        </div>
                        <TerminalStackContainer
                          key={`${cliSessionForSelectedTask.session_id}:${cliTerminalGeneration}`}
                          sessionId={cliSessionForSelectedTask.session_id}
                          getAttachmentTicket={getCliAttachmentTicket}
                          onConnectionChange={handleTaskCliConnectionChange}
                          autoReconnect={false}
                          allowSplit={false}
                        />
                      </>
                    ) : cliPreviewOpen ? (
                      <>
                        {groupProjection.mode === "live" ? (
                          <div className="space-y-3 rounded border border-line p-3">
                            <p className="text-[10px] leading-relaxed text-ink-dim">会话 API 会重新验证当前 claimant、生命周期版本、Runtime 与 Approved Launch Profile。需要管理员提供 Profile UUID；客户端不能提交命令文本或 checkout 路径。</p>
                            <label className="block space-y-1 text-[10px] text-ink-mute">
                              <span>Approved Launch Profile UUID</span>
                              <input className="input w-full font-mono text-xs" aria-label="Approved Launch Profile UUID" value={cliProfileId} onChange={(event) => setCliProfileId(event.target.value)} placeholder="由管理员提供的 UUID" maxLength={36} />
                            </label>
                            <button type="button" className="btn-primary text-xs" onClick={() => void startSelectedTaskCliSession()} disabled={cliSessionPending || groupProjection.mode !== "live" || !selectedTask || selectedTask.worktree_id !== worktreeId || !("lifecycle_status" in selectedTask) || selectedTask.lifecycle_status !== "in_progress" || selectedTask.active_worktree_id !== worktreeId}>
                              {cliSessionPending ? "正在请求受控 Session…" : "启动 Task CLI Session"}
                            </button>
                            {cliSessionError && <p role="alert" className="text-xs text-error">{cliSessionError}</p>}
                            <p className="text-[10px] text-ink-mute">若 Session provisioner、sandbox 或当前授权未就绪，服务端会拒绝启动；ticket 未产生前终端保持断开。</p>
                          </div>
                        ) : (
                          <div className="rounded border border-warning/30 bg-warning/5 p-2 text-[10px] leading-relaxed text-warning">
                            这里只展示卡内终端布局。当前没有认证 Group API provider，命令执行保持禁用。
                          </div>
                        )}
                        <TerminalStackContainer
                          key={selectedTask.id}
                          renderPane={(paneId) => (
                            <div className="flex h-full flex-col justify-between bg-[#0d1117] p-3 font-mono text-[10px] text-emerald-300" data-testid="task-cli-preview-pane">
                              <div><div>task: {selectedTask.key}</div><div>worktree: {selectedTask.worktree_id ?? "unbound"}</div><div>session: not provisioned</div></div>
                              <div><span className="text-ink-mute">{paneId}</span> $ <span className="animate-pulse">▍</span></div>
                            </div>
                          )}
                        />
                      </>
                    ) : (
                      <div className="flex h-48 flex-col items-center justify-center gap-2 rounded border border-dashed border-line text-center">
                        <CheckSquare size={20} className="text-accent" />
                        <p className="text-xs text-ink-dim">CLI 从此 Task Card 打开，并继承当前 Worktree。</p>
                        <p className="text-[10px] text-ink-mute">生产执行必须由服务端构造 TaskExecutionContext。</p>
                      </div>
                    )}
                    {cliPreviewOpen && groupProjection.mode === "live" && (
                      <section className="space-y-2 rounded border border-line p-3" aria-label="Task CLI Session 列表">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <div>
                            <h4 className="text-xs font-semibold">当前任务的最近 Session</h4>
                            <p className="text-[10px] text-ink-mute">最多显示 20 项脱敏状态；刷新页面后可从这里重新发现会话。恢复时服务端签发新 ticket。</p>
                          </div>
                          <button type="button" className="btn text-[10px]" onClick={() => setCliSessionHistoryRefresh((value) => value + 1)} disabled={cliSessionHistoryPending || cliSessionControlPending || !groupApi}>
                            {cliSessionHistoryPending ? "读取中…" : "刷新列表"}
                          </button>
                        </div>
                        {cliSessionHistoryError && <p role="alert" className="text-[10px] text-error">无法读取 Session 列表：{cliSessionHistoryError}</p>}
                        {!cliSessionHistoryPending && !cliSessionHistoryError && cliSessionHistory.length === 0 && <p className="text-[10px] text-ink-mute">当前任务没有可恢复的 Session，或服务端尚未配置 Session provisioner。</p>}
                        {cliSessionHistory.length > 0 && (
                          <ul className="space-y-2">
                            {cliSessionHistory.map((session) => {
                              const isCurrent = cliSessionForSelectedTask?.session_id === session.session_id;
                              const canReattach = session.state === "running" || session.state === "disconnected";
                              const canCancel = ["starting", "running", "disconnected", "cancelling"].includes(session.state);
                              return (
                                <li key={session.session_id} className="flex flex-col gap-2 rounded border border-line/70 bg-bg-soft/30 p-2 sm:flex-row sm:items-center sm:justify-between">
                                  <div className="min-w-0 text-[10px]">
                                    <div className="flex flex-wrap items-center gap-2">
                                      <span className="font-mono">{session.session_id}</span>
                                      <span className="rounded border border-line px-1.5 py-0.5">{session.state}{session.exit_code != null ? ` · exit ${session.exit_code}` : ""}</span>
                                      {isCurrent && <span className="text-success">当前连接</span>}
                                    </div>
                                    <p className="mt-1 text-ink-mute">更新于 {session.updated_at}</p>
                                  </div>
                                  <div className="flex shrink-0 flex-wrap gap-1.5">
                                    <button type="button" className="btn text-[10px]" onClick={() => void refreshTaskCliSessionStatus(session.session_id)} disabled={cliSessionControlPending || !groupApi}>状态</button>
                                    {canReattach && <button type="button" className="btn-primary text-[10px]" onClick={() => void reattachTaskCliSession(session.session_id, isCurrent ? cliSessionForSelectedTask?.runtime_id : undefined, isCurrent ? cliSessionForSelectedTask?.attachment_ticket : undefined)} disabled={cliSessionControlPending || cliWsConnected || !groupApi || (isCurrent && !cliNeedsReattach)}>{isCurrent && !cliWsConnected ? "重新连接" : "恢复连接"}</button>}
                                    {canCancel && <button type="button" className="btn text-[10px]" onClick={() => void cancelTaskCliSession(session.session_id)} disabled={cliSessionControlPending || !groupApi}>结束</button>}
                                  </div>
                                </li>
                              );
                            })}
                          </ul>
                        )}
                        {cliSessionError && <p role="alert" className="text-[10px] text-error">{cliSessionError}</p>}
                      </section>
                    )}
                  </section>
                </div>
              ) : taskList}
            </div>
          )}

          {currentApp === "canvas" && (
            <div className="space-y-3">
              <div className="flex flex-wrap items-end justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Infinite Canvas · Miro 等价入口</h2><p className="text-xs text-ink-mute">Canvas 与 Task Card 同级；typed EntityRef 指向当前 Worktree 内的 canonical WorkItem。</p></div>
                <div className="flex items-center gap-2">
                  {groupProjection.mode === "live" && groupCanvases.length > 1 && groupCanvas && <label className="flex items-center gap-2 text-xs">Canvas<select className="input py-1 text-xs" aria-label="选择 Worktree Canvas" value={groupCanvas.id} onChange={(event) => selectCanvas(event.target.value)}>{groupCanvases.map((canvas) => <option key={canvas.id} value={canvas.id}>{canvas.title}</option>)}</select></label>}
                  {groupProjection.mode === "live" && groupCanvas && <button type="button" className="btn text-xs" disabled={canvasBootstrapPending} onClick={() => void createWorktreeCanvas(`Worktree Canvas ${groupCanvases.length + 1}`)}>{canvasBootstrapPending ? "正在创建…" : "新建 Canvas"}</button>}
                  {groupProjection.mode === "live" && groupCanvas && <button type="button" className="btn flex items-center gap-1.5 text-xs" aria-expanded={showCanvasTaskForm} aria-controls="canvas-task-create-form" onClick={() => { setCanvasTaskError(null); setCreatedCanvasTaskId(null); setCanvasTaskTitle(""); setCanvasTaskDescription(""); setShowCanvasTaskForm((current) => !current); }}><Plus size={13} />从画布创建任务</button>}
                  {groupProjection.mode === "live" && groupCanvas && <button type="button" className="btn flex items-center gap-1.5 text-xs" aria-expanded={showCanvasLinkForm} aria-controls="canvas-task-link-form" onClick={() => { setCanvasElementLinkError(null); setLinkedCanvasWorkItemId(null); setShowCanvasLinkForm((current) => !current); }}><Plus size={13} />关联已有任务卡</button>}
                  {groupProjection.mode === "live" && groupCanvas && <form className="flex items-center gap-1" onSubmit={stageCanvasFrame}>
                    <input className="input w-28 py-1 text-[10px]" aria-label="新 Frame 名称" placeholder="Frame 名称" maxLength={200} value={canvasFrameTitle} onChange={(event) => setCanvasFrameTitle(event.target.value)} />
                    <button type="submit" className="btn text-xs" disabled={!canvasFrameTitle.trim() || displayedFrames.length >= 1_000 || canvasDocumentSavePending}>添加 Frame</button>
                  </form>}
                  {groupProjection.mode === "live" && displayedFrames.length > 0 && <>
                    <label className="flex items-center gap-1 text-[10px] text-ink-mute">Frame
                      <select className="input max-w-32 py-1 text-[10px]" aria-label="选择 Canvas Frame" value={selectedFrame?.id ?? ""} onChange={(event) => setSelectedCanvasFrameId(event.target.value)}>
                        {displayedFrames.map((frame) => <option key={frame.id} value={frame.id}>{frame.title}</option>)}
                      </select>
                    </label>
                    <button type="button" className="btn text-[10px]" disabled={!selectedFrame || canvasDocumentSavePending} onClick={addSelectedElementsToCanvasFrame}>将选中元素加入 Frame</button>
                    <button type="button" className="btn text-[10px]" disabled={!selectedFrame || selectedCanvasElementIds.length === 0 || canvasDocumentSavePending} onClick={removeSelectedElementsFromCanvasFrame}>从 Frame 移除选中元素</button>
                    <button type="button" className="btn text-[10px] text-err" disabled={!selectedFrame || canvasDocumentSavePending} onClick={() => selectedFrame && removeCanvasFrame(selectedFrame.id)}>删除 Frame</button>
                  </>}
                  {groupProjection.mode === "live" && <>
                    {displayedConnectors.length > 0 && <label className="flex items-center gap-1 text-[10px] text-ink-mute">视觉连线
                      <select className="input max-w-32 py-1 text-[10px]" aria-label="选择视觉连线" value={selectedConnector?.id ?? ""} onChange={(event) => setSelectedCanvasConnectorId(event.target.value)}>
                        {displayedConnectors.map((connector) => <option key={connector.id} value={connector.id}>{connector.label || `${connector.from_element_id.slice(0, 8)} → ${connector.to_element_id.slice(0, 8)}`}</option>)}
                      </select>
                    </label>}
                    <button type="button" className="btn text-[10px]" disabled={selectedCanvasElementIds.filter((id) => elements.some((element) => element.id === id)).length !== 2 || displayedConnectors.length >= 5_000 || canvasDocumentSavePending} onClick={addCanvasVisualConnector}>连接选中元素</button>
                    {selectedConnector && <button type="button" className="btn text-[10px] text-err" disabled={canvasDocumentSavePending} onClick={() => removeCanvasVisualConnector(selectedConnector.id)}>删除连线</button>}
                  </>}
                  {groupProjection.mode === "live" && pendingCanvasDocumentDraft && <button type="button" className="btn btn-primary text-xs" disabled={canvasDocumentSavePending} onClick={() => void saveCanvasDocument()}>{canvasDocumentSavePending ? "正在保存画布…" : "保存画布更改"}</button>}
                  <span className="rounded border border-warning/40 bg-warning/10 px-2 py-1 text-[10px] text-warning">{groupProjection.mode === "live" ? "Worktree scope · Task/Frame/视觉连线可编辑 · 不改变任务关系" : "Worktree scope · 持久化 API 未接"}</span>
                </div>
              </div>
              {groupProjection.mode === "live" && selectedFrame && <div className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 sm:grid-cols-3 lg:grid-cols-7" data-testid="canvas-frame-editor">
                <label className="grid gap-1 text-[10px]">Frame 名称<input className="input py-1 text-xs" maxLength={200} value={selectedFrame.title} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasFrame(selectedFrame.id, { title: event.currentTarget.value })} /></label>
                <label className="grid gap-1 text-[10px]">X<input className="input py-1 text-xs" type="number" min={-1_000_000} max={1_000_000} step={1} value={selectedFrame.x} disabled={canvasDocumentSavePending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) updateCanvasFrame(selectedFrame.id, { x: value }); }} /></label>
                <label className="grid gap-1 text-[10px]">Y<input className="input py-1 text-xs" type="number" min={-1_000_000} max={1_000_000} step={1} value={selectedFrame.y} disabled={canvasDocumentSavePending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) updateCanvasFrame(selectedFrame.id, { y: value }); }} /></label>
                <label className="grid gap-1 text-[10px]">宽<input className="input py-1 text-xs" type="number" min={1} max={100_000} step={1} value={selectedFrame.width} disabled={canvasDocumentSavePending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) updateCanvasFrame(selectedFrame.id, { width: value }); }} /></label>
                <label className="grid gap-1 text-[10px]">高<input className="input py-1 text-xs" type="number" min={1} max={100_000} step={1} value={selectedFrame.height} disabled={canvasDocumentSavePending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) updateCanvasFrame(selectedFrame.id, { height: value }); }} /></label>
                <label className="flex items-center gap-2 text-[10px]"><input type="checkbox" checked={selectedFrame.is_slide} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasFrame(selectedFrame.id, { is_slide: event.currentTarget.checked })} />作为演示页</label>
              </div>}
              {groupProjection.mode === "live" && selectedConnector && <div className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 sm:grid-cols-3 lg:grid-cols-7" data-testid="canvas-connector-editor">
                <label className="grid gap-1 text-[10px]">颜色<input className="input h-8 p-1" type="color" value={selectedConnector.color} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasConnector(selectedConnector.id, { color: event.currentTarget.value })} /></label>
                <label className="grid gap-1 text-[10px]">线宽<input className="input py-1 text-xs" type="number" min={0.5} max={32} step={0.5} value={selectedConnector.width} disabled={canvasDocumentSavePending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) updateCanvasConnector(selectedConnector.id, { width: value }); }} /></label>
                <label className="grid gap-1 text-[10px]">走线<select className="input py-1 text-xs" value={selectedConnector.routing} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasConnector(selectedConnector.id, { routing: event.currentTarget.value as CanvasConnector["routing"] })}><option value="straight">直线</option><option value="curved">曲线</option><option value="orthogonal">直角</option></select></label>
                <label className="grid gap-1 text-[10px]">标签<input className="input py-1 text-xs" maxLength={500} value={selectedConnector.label ?? ""} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasConnector(selectedConnector.id, { label: event.currentTarget.value })} /></label>
                <label className="flex items-center gap-2 text-[10px]"><input type="checkbox" checked={selectedConnector.arrow_start} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasConnector(selectedConnector.id, { arrow_start: event.currentTarget.checked })} />起点箭头</label>
                <label className="flex items-center gap-2 text-[10px]"><input type="checkbox" checked={selectedConnector.arrow_end} disabled={canvasDocumentSavePending} onChange={(event) => updateCanvasConnector(selectedConnector.id, { arrow_end: event.currentTarget.checked })} />终点箭头</label>
                {selectedConnector.kind !== "free" && <p className="text-[10px] text-ink-mute lg:col-span-7">当前是其他领域投影的连线；这里仅编辑画布展示，不改变其 Jira/WorkItem 关系。</p>}
              </div>}
              {groupProjection.mode === "live" && selectedCanvasElement && !selectedCanvasElement.locked && canvasElementGeometryDraft && <div className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 sm:grid-cols-4" data-testid="canvas-element-geometry-editor">
                <label className="grid gap-1 text-xs">元素宽度<input className="input py-1 text-xs" type="number" min={1} max={10_000} step={1} value={canvasElementGeometryDraft.width} disabled={canvasElementGeometryPending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) { setCanvasElementGeometryDraft((current) => current ? { ...current, width: value } : current); setCanvasElementGeometryError(null); setCanvasElementGeometrySaved(false); } }} /></label>
                <label className="grid gap-1 text-xs">元素高度<input className="input py-1 text-xs" type="number" min={1} max={10_000} step={1} value={canvasElementGeometryDraft.height} disabled={canvasElementGeometryPending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) { setCanvasElementGeometryDraft((current) => current ? { ...current, height: value } : current); setCanvasElementGeometryError(null); setCanvasElementGeometrySaved(false); } }} /></label>
                <label className="grid gap-1 text-xs">旋转角度<input className="input py-1 text-xs" type="number" min={-36_000} max={36_000} step={1} value={canvasElementGeometryDraft.rotation} disabled={canvasElementGeometryPending} onChange={(event) => { const value = event.currentTarget.valueAsNumber; if (Number.isFinite(value)) { setCanvasElementGeometryDraft((current) => current ? { ...current, rotation: value } : current); setCanvasElementGeometryError(null); setCanvasElementGeometrySaved(false); } }} /></label>
                <button type="button" className="btn btn-primary self-end text-xs" disabled={canvasElementGeometryPending || (canvasElementGeometryDraft.width === selectedCanvasElement.width && canvasElementGeometryDraft.height === selectedCanvasElement.height && canvasElementGeometryDraft.rotation === selectedCanvasElement.rotation)} onClick={() => void saveCanvasElementGeometry()}>{canvasElementGeometryPending ? "正在保存…" : "保存尺寸与旋转"}</button>
              </div>}
              {groupProjection.mode === "live" && selectedCanvasElement && selectedCanvasElementCanEditText && <div className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 md:grid-cols-[1fr_auto] md:items-end" data-testid="canvas-element-text-editor">
                <label className="grid gap-1 text-xs">便笺文字<textarea className="input min-h-20" maxLength={16_000} value={canvasElementTextDraft} disabled={canvasElementContentPending} onChange={(event) => { setCanvasElementTextDraft(event.currentTarget.value); setCanvasElementContentError(null); setCanvasElementContentSaved(false); }} /></label>
                <button type="button" className="btn btn-primary text-xs" disabled={canvasElementContentPending || canvasElementTextDraft === (selectedCanvasElement.content.text ?? "")} onClick={() => void saveCanvasElementContent()}>{canvasElementContentPending ? "正在保存…" : "保存便笺"}</button>
              </div>}
              {groupProjection.mode === "live" && showCanvasTaskForm && groupCanvas && (
                <form id="canvas-task-create-form" className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 md:grid-cols-[1fr_2fr_auto] md:items-end" onSubmit={createCanvasTask}>
                  <label className="grid gap-1 text-xs">任务标题<input className="input" required maxLength={500} autoFocus value={canvasTaskTitle} onChange={(event) => setCanvasTaskTitle(event.target.value)} /></label>
                  <label className="grid gap-1 text-xs">说明（可选）<input className="input" maxLength={100000} value={canvasTaskDescription} onChange={(event) => setCanvasTaskDescription(event.target.value)} /></label>
                  <button type="submit" className="btn btn-primary text-xs" disabled={canvasTaskPending || !canvasTaskTitle.trim()}>{canvasTaskPending ? "正在创建…" : "创建并关联"}</button>
                </form>
              )}
              {groupProjection.mode === "live" && showCanvasLinkForm && groupCanvas && (
                <form id="canvas-task-link-form" className="grid gap-2 rounded-lg border border-line bg-bg-card p-3 md:grid-cols-[1fr_auto] md:items-end" onSubmit={linkExistingCanvasWorkItem}>
                  <label className="grid gap-1 text-xs">当前 Worktree 中未关联的 Task Card
                    <select className="input" required value={selectedExistingWorkItemId} onChange={(event) => setSelectedExistingWorkItemId(event.target.value)}>
                      <option value="">选择任务卡</option>
                      {unlinkedCanvasWorkItems.map((item) => <option key={item.id} value={item.id}>{item.key} · {item.title}</option>)}
                    </select>
                  </label>
                  <button type="submit" className="btn btn-primary text-xs" disabled={canvasElementLinkPending || !selectedExistingWorkItemId || unlinkedCanvasWorkItems.length === 0}>{canvasElementLinkPending ? "正在关联…" : "放到画布"}</button>
                  {unlinkedCanvasWorkItems.length === 0 && <p className="text-xs text-ink-mute md:col-span-2">当前 Worktree 没有可添加的未关联任务卡。</p>}
                </form>
              )}
              {canvasTaskError && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Task Card 创建失败：{canvasTaskError}</p>}
              {canvasElementLinkError && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Task Card 关联失败：{canvasElementLinkError}</p>}
              {linkedCanvasWorkItemId && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">任务卡已关联到当前 Canvas。<Link className="ml-2 underline" href={taskHref(linkedCanvasWorkItemId)}>打开任务卡</Link></p>}
              {canvasBootstrapError && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Canvas 创建失败：{canvasBootstrapError}</p>}
              {createdCanvasTaskId && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">已创建并关联当前 Worktree 的 Task Card。<Link className="ml-2 underline" href={taskHref(createdCanvasTaskId)}>打开任务卡</Link></p>}
              {canvasDocumentError && <div role="alert" className="flex flex-wrap items-center gap-2 rounded border border-err/30 bg-err/5 p-3 text-xs text-err"><span>Canvas 文档保存失败：{canvasDocumentError}</span><button type="button" className="btn text-[10px]" onClick={() => { clearCanvasDocumentDraft(); setProjectionRefreshKey((current) => current + 1); }}>放弃草稿并加载最新版本</button></div>}
              {canvasDocumentSaved && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">Canvas 文档更改已保存到当前 Worktree。</p>}
              {canvasElementMoveError && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Canvas 元素移动失败：{canvasElementMoveError}</p>}
              {canvasElementMoveSaved && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">Canvas 元素位置已保存到当前 Worktree。</p>}
              {canvasElementGeometryError && selectedCanvasElement && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Canvas 元素尺寸/旋转保存失败：{canvasElementGeometryError}。草稿仍保留；<button type="button" className="underline" onClick={() => { setCanvasElementGeometryDraft({ width: selectedCanvasElement.width, height: selectedCanvasElement.height, rotation: selectedCanvasElement.rotation }); setCanvasElementGeometryError(null); }}>重新加载投影</button></p>}
              {canvasElementGeometrySaved && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">Canvas 元素尺寸与旋转角度已保存到当前 Worktree。</p>}
              {canvasElementDeleteError && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Canvas 元素删除失败：{canvasElementDeleteError}。已完成的删除保留，未完成项可重试。</p>}
              {canvasElementDeleteSaved && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">Canvas 元素已从当前 Worktree 移除；引用它的 Frame 成员关系和视觉连线已在同一事务清理。</p>}
              {canvasElementContentError && selectedCanvasElement && <p role="alert" className="rounded border border-err/30 bg-err/5 p-3 text-xs text-err">Canvas 便笺保存失败：{canvasElementContentError}。草稿仍保留；重新加载按钮会用当前授权投影内容覆盖草稿。<button type="button" className="ml-2 underline" onClick={() => { setCanvasElementTextDraft(selectedCanvasElement.content.text ?? ""); setCanvasElementContentError(null); }}>重新加载投影</button></p>}
              {canvasElementContentSaved && <p role="status" className="rounded border border-accent/30 bg-accent/5 p-3 text-xs">Canvas 便笺文字已保存到当前 Worktree。</p>}
              {groupProjection.mode === "live" && groupProjection.refreshError && <p role="status" className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning">Canvas 更新暂不可用：{groupProjection.refreshError}。保留最后一次成功的授权投影并继续重试。</p>}
              {canvasLinkError && <p role="alert" className="rounded border border-warning/30 bg-warning/5 p-3 text-xs text-warning">{canvasLinkError}</p>}
              {groupCanvas ? (
                <div className="relative h-[min(68vh,760px)] overflow-hidden rounded-lg border border-line bg-bg-card" data-testid="group-canvas-preview">
                  <div className="border-b border-line px-3 py-2 text-xs font-medium">{groupCanvas.title}</div>
                  <CanvasView
                    key={`${groupCanvas.id}:${"version" in groupCanvas ? groupCanvas.version : "preview"}`}
                    canvas={displayedCanvas ?? groupCanvas}
                    elements={elements}
                    connectors={displayedConnectors}
                    highlightElementId={searchParams.get("highlight") ?? undefined}
                    readOnly={groupProjection.mode === "live"}
                    groupWorkItems={groupProjection.mode === "live" ? groupProjection.work_items : undefined}
                    groupWorktrees={groupProjection.mode === "live" ? [groupProjection.worktree] : undefined}
                    onOpenWorkItem={openCanvasTask}
                    onViewportChange={groupProjection.mode === "live" ? stageCanvasViewport : undefined}
                    onElementPositionChange={groupProjection.mode === "live" ? saveCanvasElementPosition : undefined}
                    onDeleteElements={groupProjection.mode === "live" ? deleteCanvasElements : undefined}
                    onSelectionChange={groupProjection.mode === "live" ? setSelectedCanvasElementIds : undefined}
                  />
                  {canvasDocumentSavePending && <div className="absolute inset-0 z-30 grid place-items-center bg-bg/60 text-sm" role="status">正在保存 Canvas 文档…</div>}
                  {canvasElementMovePending && <div className="absolute inset-0 z-30 grid place-items-center bg-bg/60 text-sm" role="status">正在保存元素位置…</div>}
                  {canvasElementDeletePending && <div className="absolute inset-0 z-30 grid place-items-center bg-bg/60 text-sm" role="status">正在删除 Canvas 元素…</div>}
                </div>
              ) : (
                <div className="card flex min-h-56 flex-col items-center justify-center gap-2 px-6 text-center">
                  <p className="text-sm text-ink-dim">当前 Worktree 没有显式绑定的 Group Canvas。</p>
                  <p className="max-w-xl text-xs leading-relaxed text-ink-mute">{groupProjection.mode === "live" ? "服务端尚未返回当前 Worktree 的 Canvas；不会使用 Project / Free Canvas seed 代替。" : "旧 Project / Free Canvas 不会自动继承到这里；原型不会用全局 seed 冒充当前 Worktree 画布。"}</p>
                  {groupProjection.mode === "live" && <button type="button" className="btn btn-primary mt-2 text-xs" disabled={canvasBootstrapPending} onClick={() => void createWorktreeCanvas()}>{canvasBootstrapPending ? "正在创建…" : "创建此 Worktree 的 Canvas"}</button>}
                </div>
              )}
            </div>
          )}

          {currentApp === "workflow" && (
            <div className="space-y-4">
              <div><h2 className="text-lg font-semibold">Workflow · LangGraph / L0</h2><p className="text-xs text-ink-mute">流程执行是 Worktree Group 的同级应用能力，不创建第二个聊天栏或任务事实源。</p></div>
              <section className="card grid gap-4 md:grid-cols-3">
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Owner</div><div className="mt-1 text-sm">Workflow Runtime</div></div>
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Context</div><div className="mt-1 font-mono text-xs">{worktree.id} · {worktree.project_id}</div></div>
                <div><div className="text-[10px] uppercase tracking-widest text-ink-mute">Runtime</div><div className="mt-1 text-sm text-warning">not connected</div></div>
              </section>
              <div className="rounded-lg border border-warning/30 bg-warning/5 p-4 text-xs leading-relaxed text-ink-dim">
                LangGraph 每次 start/resume 都必须重新解析 GroupContext 并复验权限。当前仓库 chat stub 不带 scope；此入口不创建 Flow、不运行 Agent，也不把 checkpoint 当作授权凭据。
              </div>
            </div>
          )}

          {currentApp === "plugins" && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div><h2 className="text-lg font-semibold">Group Plugin Apps</h2><p className="text-xs text-ink-mute">App Registry 与 LangGraph SubAgentRegistry 分离；此列表只呈现当前 Worktree 已授权的导航入口。</p></div>
                {groupApi && (
                  <button type="button" className="btn text-xs" onClick={() => setGroupAppsRefreshKey((current) => current + 1)} disabled={activeGroupAppsState.mode === "loading"}>
                    {activeGroupAppsState.mode === "loading" ? "正在刷新…" : "刷新授权列表"}
                  </button>
                )}
              </div>

              {activeGroupAppsState.mode === "preview" && (
                <>
                  <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-[10px] text-warning">本地原型列表。开关只改变本页预览，不会安装、卸载或授予任何 capability。</div>
                  <div className="grid gap-3 lg:grid-cols-2">
                    {PLUGIN_PREVIEW.map((plugin) => (
                      <article key={plugin.id} className="card flex items-center justify-between gap-4">
                        <div className="min-w-0">
                          <div className="flex items-center gap-2 text-sm font-medium"><Plug size={14} className="text-accent" />{plugin.name}</div>
                          <div className="mt-1 break-all font-mono text-[10px] text-ink-mute">{plugin.capability}</div>
                        </div>
                        <label className="flex shrink-0 items-center gap-2 text-xs">
                          <input
                            type="checkbox"
                            checked={enabledPlugins[plugin.id]}
                            onChange={(event) => setEnabledPlugins((current) => ({ ...current, [plugin.id]: event.target.checked }))}
                            aria-label={`${plugin.name} 原型开关`}
                            data-testid={`plugin-preview-${plugin.id}`}
                          />
                          {enabledPlugins[plugin.id] ? "预览启用" : "预览停用"}
                        </label>
                      </article>
                    ))}
                  </div>
                </>
              )}

              {activeGroupAppsState.mode === "loading" && (
                <div className="card text-sm text-ink-mute" role="status">正在读取当前 Worktree 的授权 Plugin Registry；读取期间不显示本地预览项。</div>
              )}

              {activeGroupAppsState.mode === "error" && (
                <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-err/30 bg-err/5 p-3 text-xs text-err" role="alert">
                  <span>无法读取授权插件列表：{activeGroupAppsState.message}</span>
                  <button type="button" className="btn text-xs" onClick={() => setGroupAppsRefreshKey((current) => current + 1)}>重试</button>
                </div>
              )}

              {activeGroupAppsState.mode === "live" && (
                <>
                  <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-line bg-bg-soft p-3 text-[10px] text-ink-mute">
                    <span>服务端授权投影 · Registry 版本 {activeGroupAppsState.registryVersion}</span>
                    <span>{activeGroupAppsState.apps.length} 个可见 App</span>
                  </div>
                  {activeGroupAppsState.apps.length > 0 ? (
                    <div className="grid gap-3 lg:grid-cols-2">
                      {activeGroupAppsState.apps.map((plugin) => (
                        <article key={plugin.plugin_id} className="card flex items-center justify-between gap-4" data-testid={`group-plugin-${plugin.plugin_id}`}>
                          <div className="min-w-0">
                            <div className="flex items-center gap-2 text-sm font-medium"><Plug size={14} className="text-accent" />{plugin.label}</div>
                            <div className="mt-1 font-mono text-[10px] text-ink-mute">{plugin.plugin_id} · manifest v{plugin.manifest_version}</div>
                          </div>
                          <span className="shrink-0 rounded-full border border-accent/30 px-2 py-1 text-[9px] text-accent">已授权入口</span>
                        </article>
                      ))}
                    </div>
                  ) : (
                    <div className="card text-sm text-ink-mute">当前 Worktree 没有已授权显示的 Plugin App。</div>
                  )}
                  <p className="text-[10px] leading-relaxed text-ink-mute">导航可见性不等于 capability 执行权限。插件执行界面、逐次 capability gateway、隔离运行时与热撤权仍未接入。</p>
                </>
              )}
            </div>
          )}

          {selectedPlugin && (
            <div className="space-y-4" data-testid={`plugin-app-${selectedPlugin.id}`}>
              <div>
                <h2 className="text-lg font-semibold">{selectedPlugin.name}</h2>
                <p className="text-xs text-ink-mute">此插件入口作为 Worktree Group 的同级 App 显示。</p>
              </div>
              {selectedPlugin.source === "registry" ? (
                <>
                  <div className="rounded-lg border border-accent/30 bg-accent/5 p-3 text-xs text-accent">此入口来自当前 Worktree 的服务端授权 Registry 投影，manifest 版本为 {selectedPlugin.manifestVersion}。</div>
                  <div className="card text-xs leading-relaxed text-ink-mute">插件执行 surface 尚未安装。可见导航不授予插件能力；每次运行仍须由后端 capability gateway 按当前 actor、Project、Worktree、manifest 与 grant 版本重新授权。</div>
                </>
              ) : (
                <>
                  <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-xs text-warning">本地预览入口。启用状态保存在当前页面内存中；尚未连接服务端 Plugin Registry、capability grant 或插件运行时。</div>
                  <div className="card">
                    <div className="text-[10px] font-semibold uppercase tracking-widest text-ink-mute">声明能力（预览）</div>
                    <div className="mt-2 break-all font-mono text-xs text-ink-dim">{selectedPlugin.capability}</div>
                    <div className="mt-3 text-[10px] leading-relaxed text-ink-mute">此页面不调用这些 capability。生产接入必须经 Plugin Gateway 按当前 actor、Project、Worktree 和授权版本逐次检查。</div>
                  </div>
                </>
              )}
            </div>
          )}
        </section>
      </div>

      {selectedTask && currentApp === "task-card" && (
        <Link href={appHref("task-card")} className="fixed bottom-36 right-4 z-30 rounded-full border border-line bg-bg-card px-3 py-2 text-[10px] text-ink-dim shadow-lg md:bottom-24">返回全部 Task Cards</Link>
      )}

      <aside className="fixed bottom-[4.6rem] left-3 right-3 z-40 mx-auto max-w-[960px] rounded-xl border border-line bg-bg-card/95 p-3 shadow-2xl backdrop-blur md:bottom-4 md:left-[17rem] md:right-5" data-testid="group-chat-bar">
        <div className="flex flex-wrap items-center gap-2">
          <MessageSquareText size={15} className="shrink-0 text-accent" />
          <label htmlFor="group-chat-scope" className="text-[10px] font-semibold uppercase tracking-wider text-ink-mute">Scope</label>
          <select
            id="group-chat-scope"
            value={scope}
            onChange={(event) => changeScope(event.target.value as ChatScope)}
            className="rounded-md border border-line bg-bg-soft px-2 py-1.5 text-xs"
            data-testid="group-chat-scope"
          >
            <option value="WORKTREE">WORKTREE · {worktree.id}</option>
            <option value="GLOBAL">GLOBAL · 需选择授权目标</option>
          </select>
          {scope === "GLOBAL" && (
            <div className="w-full pl-6" data-testid="global-chat-targets">
              <div className="mb-1 flex items-center justify-between text-[9px] text-ink-mute">
                <span>仅显示当前账号可访问的 Worktree；提交时服务端会再次逐个授权（最多 20 个）</span>
                <span>{selectedChatTargetIds.length}/20 已选</span>
              </div>
              {groupProjection.mode !== "live" ? (
                <div className="rounded-md border border-warning/30 bg-warning/5 px-2 py-2 text-[10px] text-warning">
                  登录会话与宿主 Group API 尚未接入，无法读取授权目标。
                </div>
              ) : (
                <div className="max-h-28 overflow-y-auto rounded-md border border-line bg-bg-soft p-2">
                  <div className="grid gap-1 sm:grid-cols-2">
                    {chatTargetOptions.map((target) => (
                      <label key={target.worktree_id} className="flex min-w-0 items-center gap-2 text-[10px] text-ink-dim">
                        <input
                          type="checkbox"
                          checked={selectedChatTargetIds.includes(target.worktree_id)}
                          onChange={() => toggleGlobalChatTarget(target.worktree_id)}
                          disabled={!selectedChatTargetIds.includes(target.worktree_id) && selectedChatTargetIds.length >= 20}
                        />
                        <span className="truncate">{target.name}</span>
                        <span className="shrink-0 text-[9px] text-ink-mute">{target.project_id.slice(0, 8)}</span>
                      </label>
                    ))}
                  </div>
                  {chatTargetsPending && <div className="mt-1 text-[9px] text-ink-mute">正在读取 Worktree…</div>}
                  {chatTargetsError && <div role="alert" className="mt-1 text-[9px] text-danger">{chatTargetsError}</div>}
                  {chatTargetCursor && (
                    <button type="button" disabled={chatTargetsPending} onClick={loadMoreGlobalChatTargets} className="mt-2 text-[9px] text-accent disabled:opacity-50">
                      加载更多 Worktree
                    </button>
                  )}
                  {!chatTargetsPending && chatTargetOptions.length === 0 && (
                    <div className="text-[9px] text-ink-mute">当前账号没有可用于 Global Chat 的 Worktree。</div>
                  )}
                </div>
              )}
            </div>
          )}
          <div className="min-w-[180px] flex-1">
            <input
              disabled
              value=""
              readOnly
              placeholder={scope === "GLOBAL" ? "选择一个或多个授权 Worktree" : "Worktree scoped chat"}
              className="w-full rounded-md border border-line bg-bg-soft px-3 py-2 text-xs opacity-70"
              aria-label="Group Chat message"
            />
          </div>
          <button type="button" disabled className="btn-primary text-xs opacity-50">发送</button>
          <span className="w-full pl-6 text-[9px] text-warning">
            Group Chat workflow / Transcript / LangGraph runtime 尚未接入；发送保持禁用。目标选择不会授予额外权限。
          </span>
        </div>
      </aside>
    </main>
  );
}

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx"}),
      (page:Function {name:"GroupWorkspacePage"}),
      (api:Function {name:"WorktreeGroupApiClient.listGlobalChatTargets"}),
      (worktree:Variable {name:"worktree"}),
      (scope:Variable {name:"scope"});
CREATE (targets:Variable {name:"chatTargets",type:"variable",language:"typescript"}),
       (cursor:Variable {name:"chatTargetCursor",type:"variable",language:"typescript"}),
       (pending:Variable {name:"chatTargetsPending",type:"variable",language:"typescript"}),
       (error:Variable {name:"chatTargetsError",type:"variable",language:"typescript"}),
       (selected:Variable {name:"selectedChatTargetIds",type:"variable",language:"typescript"}),
       (options:Variable {name:"chatTargetOptions",type:"variable",language:"typescript"}),
       (toggle:Function {name:"toggleGlobalChatTarget",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (loadMore:Function {name:"loadMoreGlobalChatTargets",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (fetchTargets:Function {name:"loadGlobalChatTargetsEffect",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (file)-[:CONTAINS]->(targets),(file)-[:CONTAINS]->(cursor),(file)-[:CONTAINS]->(pending),(file)-[:CONTAINS]->(error),(file)-[:CONTAINS]->(selected),(file)-[:CONTAINS]->(options),(page)-[:USES]->(targets),(page)-[:USES]->(cursor),(page)-[:USES]->(pending),(page)-[:USES]->(error),(page)-[:USES]->(selected),(page)-[:USES]->(options),(page)-[:CONTAINS]->(toggle),(page)-[:CONTAINS]->(loadMore),(page)-[:CONTAINS]->(fetchTargets),(page)-[:CALLS]->(toggle),(page)-[:CALLS]->(loadMore),(page)-[:USES]->(scope),(options)-[:DERIVES_FROM]->(worktree),(fetchTargets)-[:CALLS]->(api),(loadMore)-[:CALLS]->(api),(toggle)-[:USES]->(selected);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx"}),
      (page:Function {name:"GroupWorkspacePage"}),
      (listGroupApps:Function {name:"WorktreeGroupApiClient.listGroupApps"});
CREATE (registryState:Variable {name:"groupAppsState",type:"variable",language:"typescript"}),
       (activeRegistryState:Variable {name:"activeGroupAppsState",type:"variable",language:"typescript"}),
       (availablePlugins:Variable {name:"availablePlugins",type:"variable",language:"typescript"}),
       (refreshKey:Variable {name:"groupAppsRefreshKey",type:"variable",language:"typescript"}),
       (loadRegistry:Function {name:"loadGroupAppsEffect",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
       (file)-[:CONTAINS]->(registryState),
       (file)-[:CONTAINS]->(activeRegistryState),
       (file)-[:CONTAINS]->(availablePlugins),
       (file)-[:CONTAINS]->(refreshKey),
       (page)-[:USES]->(registryState),
       (page)-[:USES]->(activeRegistryState),
       (page)-[:USES]->(availablePlugins),
       (page)-[:USES]->(refreshKey),
       (page)-[:CONTAINS]->(loadRegistry),
       (activeRegistryState)-[:DERIVES_FROM]->(registryState),
       (availablePlugins)-[:DERIVES_FROM]->(activeRegistryState),
       (loadRegistry)-[:CALLS]->(listGroupApps),
       (loadRegistry)-[:USES]->(refreshKey);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/app/worktree/[id]/group/page.tsx"}),
      (page:Function {name:"GroupWorkspacePage"}),
      (projectApps:Function {name:"projectGroupAppsForNavigation"});
CREATE (page)-[:CALLS]->(projectApps);
*/
