/**
 * CredentialTab - 凭证 tab 组件 (per ADR-0051 §2.3)
 *
 * 守门合规:
 * - 守门 #5: 不打印 value, 仅 key 引用
 * - 守门 #13 c: per-agent 隔离
 */

import * as React from "react";
import {
  AgentCredentialStore,
  getAgentCredentialStore,
} from "../../lib/agent/credentials";
import {
  type AgentCredential,
  type AgentId,
  type CredentialKind,
} from "../../lib/agent/types";

interface CredentialTabProps {
  agentId: AgentId;
  store?: AgentCredentialStore;
}

export function CredentialTab({
  agentId,
  store = getAgentCredentialStore(),
}: CredentialTabProps): React.ReactElement {
  const [creds, setCreds] = React.useState<AgentCredential[]>([]);
  const [newKind, setNewKind] = React.useState<CredentialKind>("openai");
  const [newDisplayName, setNewDisplayName] = React.useState("");
  const [newEnvKey, setNewEnvKey] = React.useState("");

  // 测用: 初始加载
  React.useEffect(() => {
    setCreds(store.list(agentId));
  }, [agentId, store]);

  const handleAdd = () => {
    if (!newDisplayName.trim() || !newEnvKey.trim()) return;
    const cred = store.add(agentId, {
      kind: newKind,
      displayName: newDisplayName,
      envKey: newEnvKey,
    });
    setCreds(store.list(agentId));
    setNewDisplayName("");
    setNewEnvKey("");
    return cred;
  };

  const handleRemove = (id: string) => {
    store.remove(agentId, id);
    setCreds(store.list(agentId));
  };

  const handleActivate = (id: string) => {
    store.activate(agentId, id);
    setCreds(store.list(agentId));
  };

  return (
    <div className="credential-tab" data-testid="credential-tab">
      <h3>{agentId} 凭证配置 (per-agent, per ADR-0051)</h3>

      {/* 新增凭证 form (per 守门 #5: env_key 不存 value) */}
      <div className="credential-tab__form">
        <select
          value={newKind}
          onChange={(e) => setNewKind(e.target.value as CredentialKind)}
          data-testid="credential-tab-kind"
        >
          <option value="openai">OpenAI</option>
          <option value="claude">Claude</option>
          <option value="anthropic">Anthropic</option>
          <option value="github_copilot">GitHub Copilot</option>
          <option value="cursor">Cursor</option>
          <option value="custom">Custom</option>
        </select>
        <input
          type="text"
          placeholder="Display Name"
          value={newDisplayName}
          onChange={(e) => setNewDisplayName(e.target.value)}
          data-testid="credential-tab-display-name"
        />
        <input
          type="text"
          placeholder="Env Key (e.g. OPENAI_API_KEY)"
          value={newEnvKey}
          onChange={(e) => setNewEnvKey(e.target.value)}
          data-testid="credential-tab-env-key"
        />
        <button
          onClick={handleAdd}
          data-testid="credential-tab-add"
        >
          + Add Credential
        </button>
      </div>

      {/* 凭证列表 */}
      <ul className="credential-tab__list" data-testid="credential-tab-list">
        {creds.length === 0 && (
          <li className="credential-tab__empty">暂无凭证, 在上方表单添加</li>
        )}
        {creds.map((cred) => (
          <li
            key={cred.id}
            className={`credential-tab__item ${cred.active ? "active" : ""}`}
            data-testid={`credential-tab-item-${cred.kind}`}
          >
            <div className="credential-tab__item-info">
              <strong>{cred.displayName}</strong>
              <span>{cred.kind}</span>
              <span>env: {cred.envKey}</span>
              {cred.active && <span className="badge">active</span>}
            </div>
            <div className="credential-tab__item-actions">
              {!cred.active && (
                <button
                  onClick={() => handleActivate(cred.id)}
                  data-testid={`credential-tab-activate-${cred.kind}`}
                >
                  Activate
                </button>
              )}
              <button
                onClick={() => handleRemove(cred.id)}
                data-testid={`credential-tab-remove-${cred.kind}`}
              >
                Remove
              </button>
            </div>
          </li>
        ))}
      </ul>

      {/* 守门 #5: 凭证值不在此显示, 仅 env_key (per EX-05 IdempotencyManager 同模式) */}
      <p className="credential-tab__note">
        * 凭证值由 KMS 加密 (per V2-1 crates/star-credential), 运行时解密注入, 不在此显示 (per 守门 #5)
      </p>
    </div>
  );
}

export default CredentialTab;
