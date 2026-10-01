/* CYPHER STRUCTURE MANIFEST
CREATE (f:File {name:"frontend/src/lib/run/runDirectorySession.tsx",type:"file",language:"tsx"}),
 (session:Class {name:"RunDirectoryHostSession",type:"class"}),(state:Class {name:"RunDirectorySessionState",type:"class"}),
 (provider:Function {name:"RunDirectoryProvider",type:"function"}),(hook:Function {name:"useRunDirectory",type:"function"}),
 (assemble:Function {name:"assembleRunDirectoryClient",type:"function"}),(focus:Function {name:"setRunFocus",type:"function"}),
 (clear:Function {name:"clearRunFocus",type:"function"}),(invalidate:Function {name:"invalidateRunSession",type:"function"}),
 (hold:Function {name:"holdRunSession",type:"function"}),(release:Function {name:"releaseRunSession",type:"function"}),
 (leases:Variable {name:"leases",type:"variable"}),(origins:Variable {name:"EMPTY_ORIGINS",type:"variable"}),
 (ctx:Variable {name:"RunDirectoryContext",type:"variable"}),(missing:Variable {name:"MISSING_SESSION",type:"variable"}),
 (client:Class {name:"RunDirectoryApiClient",type:"class"}),(dispose:Function {name:"RunDirectoryApiClient.dispose",type:"function"}),
 (memo:Function {name:"useMemo",type:"function"}),(effect:Function {name:"useEffect",type:"function"}),(callback:Function {name:"useCallback",type:"function"}),
 (context:Function {name:"useContext",type:"function"}),(reactState:Function {name:"useState",type:"function"}),(ref:Function {name:"useRef",type:"function"}),
 (f)-[:CONTAINS]->(session),(f)-[:CONTAINS]->(state),(f)-[:CONTAINS]->(provider),(f)-[:CONTAINS]->(hook),(f)-[:CONTAINS]->(ctx),(f)-[:CONTAINS]->(missing),
 (provider)-[:CONTAINS]->(assemble),(provider)-[:CONTAINS]->(focus),(provider)-[:CONTAINS]->(clear),(assemble)-[:CONTAINS]->(invalidate),
 (provider)-[:CONTAINS]->(hold),(hold)-[:CONTAINS]->(release),(provider)-[:CONTAINS]->(leases),(f)-[:CONTAINS]->(origins),
 (provider)-[:CALLS]->(ref),(hold)-[:USES]->(leases),(release)-[:USES]->(leases),(provider)-[:USES]->(origins),
 (provider)-[:CALLS]->(memo),(provider)-[:CALLS]->(effect),(provider)-[:CALLS]->(callback),(provider)-[:CALLS]->(reactState),(assemble)-[:CALLS]->(client),(release)-[:CALLS]->(dispose),
 (hook)-[:CALLS]->(context),(hook)-[:USES]->(ctx),(provider)-[:USES]->(ctx),(provider)-[:USES]->(missing);
*/

"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { RunDirectoryApiClient, type RunAccessTokenProvider, type RunContextEnvelope } from "./runDirectoryApi";

/** A real host supplies this contract. No browser credential storage or token issuer is installed here. */
export interface RunDirectoryHostSession {
  /** Non-secret generation; replace on login/logout, actor, tenant, or authority change. */
  generation: string;
  getAccessToken: RunAccessTokenProvider;
  baseUrl?: string;
  trustedOrigins?: readonly string[];
  fetcher?: typeof fetch;
}
export interface RunDirectorySessionState {
  status: "missing" | "ready" | "blocked";
  message?: string;
  client: RunDirectoryApiClient | null;
  focus: RunContextEnvelope | null;
  setFocus(context: RunContextEnvelope): void;
  clearFocus(): void;
}
const MISSING_SESSION: RunDirectorySessionState = {
  status: "missing", message: "宿主认证会话尚未装配。Project → Branch → Engineering Run → Worktree 导航暂不可用。",
  client: null, focus: null, setFocus: () => undefined, clearFocus: () => undefined,
};
const RunDirectoryContext = createContext<RunDirectorySessionState>(MISSING_SESSION);
const EMPTY_ORIGINS: readonly string[] = [];

export function RunDirectoryProvider({ children, session = null }: { children: ReactNode; session?: RunDirectoryHostSession | null }) {
  const [invalidatedClient, setInvalidatedClient] = useState<RunDirectoryApiClient | null>(null);
  const [selected, setSelected] = useState<{ client: RunDirectoryApiClient; context: RunContextEnvelope } | null>(null);
  const leases = useRef(new Map<RunDirectoryApiClient, object>());
  const generation = session?.generation;
  const getAccessToken = session?.getAccessToken;
  const baseUrl = session?.baseUrl;
  const trustedOrigins = session?.trustedOrigins ?? EMPTY_ORIGINS;
  const fetcher = session?.fetcher;
  const bundle = useMemo(function assembleRunDirectoryClient() {
    if (!getAccessToken || !generation) return { client: null, message: MISSING_SESSION.message };
    if (!generation.trim() || generation.length > 256 || /[\u0000-\u001f\u007f]/.test(generation)) {
      return { client: null, message: "宿主会话 generation 无效，目录已禁用。" };
    }
    try {
      const client = new RunDirectoryApiClient({
        getAccessToken, baseUrl, trustedOrigins, fetcher,
        onAuthorizationError: function invalidateRunSession() {
          setInvalidatedClient(client);
          setSelected((current) => current?.client === client ? null : current);
        },
      });
      return { client, message: undefined };
    } catch {
      return { client: null, message: "宿主目录 API 配置无效，目录已禁用。" };
    }
  }, [baseUrl, fetcher, generation, getAccessToken, trustedOrigins]);
  const client = bundle.client;
  const blocked = client !== null && invalidatedClient === client;
  useEffect(function holdRunSession() {
    setSelected((current) => current?.client === client ? current : null);
    setInvalidatedClient((current) => current === client ? current : null);
    if (!client) return;
    const lease = {};
    const activeLeases = leases.current;
    activeLeases.set(client, lease);
    // A replayed effect renews the same client; a new generation disposes its old client.
    return () => queueMicrotask(function releaseRunSession() {
      if (activeLeases.get(client) === lease) { activeLeases.delete(client); client.dispose(); }
    });
  }, [client]);
  const setFocus = useCallback(function setRunFocus(context: RunContextEnvelope) {
    if (client && !blocked && context.focus) setSelected({ client, context });
  }, [blocked, client]);
  const clearFocus = useCallback(function clearRunFocus() { setSelected(null); }, []);
  const value = useMemo<RunDirectorySessionState>(() => ({
    status: blocked ? "blocked" : client ? "ready" : session ? "blocked" : "missing",
    message: blocked ? "会话或授权已失效。目录和 checkout 焦点已清除，请重新连接宿主会话。" : bundle.message,
    client: blocked ? null : client,
    focus: !blocked && selected?.client === client ? selected.context : null,
    setFocus, clearFocus,
  }), [blocked, bundle.message, clearFocus, client, selected, session, setFocus]);
  return <RunDirectoryContext.Provider value={value}>{children}</RunDirectoryContext.Provider>;
}

export function useRunDirectory(): RunDirectorySessionState { return useContext(RunDirectoryContext); }
