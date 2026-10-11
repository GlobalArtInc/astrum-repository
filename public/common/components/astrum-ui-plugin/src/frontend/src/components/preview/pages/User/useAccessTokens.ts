import { useCallback, useEffect, useState } from 'react';
import { restClient, API_INTERNAL_UI } from '../../../../interface/api/rest-client';

export const ACCESS_TOKENS_URL = `${API_INTERNAL_UI}/access-tokens`;

export type AccessTokenScope = 'read' | 'write';

export interface AccessToken {
  id: string;
  name: string;
  owner: string;
  scope: AccessTokenScope;
  hint: string;
  created: string;
  expires: string | null;
  lastUsed: string | null;
  expired: boolean;
}

export interface CreatedAccessToken {
  token: AccessToken;
  secret: string;
}

export interface CreateAccessTokenRequest {
  name: string;
  scope: AccessTokenScope;
  expiresInDays: number | null;
}

export function errorMessage(error: unknown, fallback: string): string {
  const data = (error as { response?: { data?: unknown } })?.response?.data;
  if (Array.isArray(data) && data[0]?.message) {
    return String(data[0].message);
  }
  if (data && typeof data === 'object' && 'message' in data && (data as { message?: unknown }).message) {
    return String((data as { message: unknown }).message);
  }
  if (typeof data === 'string' && data.trim()) {
    return data;
  }
  return fallback;
}

export function useAccessTokens(all: boolean) {
  const [tokens, setTokens] = useState<AccessToken[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await restClient.get<AccessToken[]>(ACCESS_TOKENS_URL, { params: all ? { all: true } : undefined });
      setTokens(result ?? []);
      setError(null);
    } catch (e) {
      setError(e);
    } finally {
      setLoading(false);
    }
  }, [all]);

  useEffect(() => {
    load();
  }, [load]);

  const create = useCallback(
    async (request: CreateAccessTokenRequest) => {
      const created = await restClient.post<CreatedAccessToken, CreateAccessTokenRequest>(ACCESS_TOKENS_URL, request);
      await load();
      return created;
    },
    [load]
  );

  const revoke = useCallback(
    async (id: string) => {
      await restClient.delete(`${ACCESS_TOKENS_URL}/${encodeURIComponent(id)}`);
      await load();
    },
    [load]
  );

  return { tokens, loading, error, reload: load, create, revoke };
}
