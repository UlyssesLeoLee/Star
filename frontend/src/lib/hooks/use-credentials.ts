// Star Frontend - Credentials React Query Hooks
// V2-2 完整版

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { credentialsApi, Provider, CreateCredentialRequest, RotateRequest } from "@/lib/api/credentials";

export function useCredentials(provider?: Provider) {
  return useQuery({
    queryKey: ["credentials", provider ?? "all"],
    queryFn: () => credentialsApi.list(provider),
  });
}

export function useCreateCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateCredentialRequest) => credentialsApi.create(req),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["credentials"] }),
  });
}

export function useRotateCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, req }: { id: string; req: RotateRequest }) =>
      credentialsApi.rotate(id, req),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["credentials"] }),
  });
}

export function useRevokeCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => credentialsApi.revoke(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["credentials"] }),
  });
}

export function useAuditLog(id: string | null) {
  return useQuery({
    queryKey: ["audit", id],
    queryFn: () => (id ? credentialsApi.audit(id) : []),
    enabled: !!id,
  });
}

export const PROVIDER_LABELS: Record<Provider, string> = {
  openclaw: "OpenClaw (LLM agent 编排)",
  hermes: "Hermes (消息总线)",
  kms_vault: "HashiCorp Vault (KMS)",
  kms_aws: "AWS KMS",
  kms_local_mock: "Local Mock (dev/test)",
};
