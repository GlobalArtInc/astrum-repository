import { INTERNAL } from '../../api';
import { matchesWords, queryWords } from './model';

export const OAUTH = `${INTERNAL}/ui/oauth`;
export const GRANTS = `${OAUTH}/grants`;

export interface AgentGrant {
  id: string;
  clientName: string;
  scope: 'READ' | 'WRITE';
  createdAt: string;
  lastUsedAt: string | null;
  expiresAt: string;
}

export interface AuthorizationRequest {
  id: string;
  clientName: string;
  clientId: string;
  redirectHost: string;
  requestedScope: string[];
  writeRequested: boolean;
  user: string;
  expiresAt: string;
}

export function filterGrants(grants: AgentGrant[], query: string): AgentGrant[] {
  const words = queryWords(query);
  if (words.length === 0) {
    return grants;
  }
  return grants.filter((grant) => matchesWords(words, grant.clientName, grant.scope));
}

export function requestPath(id: string, action?: 'approve' | 'deny'): string {
  const base = `${OAUTH}/requests/${encodeURIComponent(id)}`;
  return action ? `${base}/${action}` : base;
}

export function grantPath(id: string): string {
  return `${GRANTS}/${encodeURIComponent(id)}`;
}
