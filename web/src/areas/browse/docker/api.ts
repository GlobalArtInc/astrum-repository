import { UI_HEADER } from '../../../api/client';

export type SignatureScheme = 'legacy' | 'referrer' | 'bundle' | 'notation';
export type SignatureState = 'verified' | 'invalid' | 'unsupported' | 'unknown';
export type SbomFormat = 'spdx' | 'cyclonedx' | 'unknown';

export interface Signature {
  digest: string;
  scheme: SignatureScheme;
  state?: SignatureState;
  signer?: string;
}

export interface Attestation {
  digest: string;
}

export interface SbomReference {
  digest: string;
  format: SbomFormat;
}

export interface SupplyChain {
  digest: string;
  children: string[];
  signatures: Signature[];
  attestations: Attestation[];
  sboms: SbomReference[];
}

export interface SbomPackage {
  name: string;
  version: string;
  purl: string;
  licenses: string[];
  ecosystem: string;
}

export interface SbomPage {
  format: SbomFormat;
  total: number;
  page: number;
  pageSize: number;
  packages: SbomPackage[];
}

export class SbomRequestError extends Error {
  readonly status: number;
  readonly retryAfter: number | null;

  constructor(status: number, retryAfter: number | null) {
    super(`HTTP ${status}`);
    this.name = 'SbomRequestError';
    this.status = status;
    this.retryAfter = retryAfter;
  }
}

export function parseRetryAfter(value: string | null): number | null {
  if (value === null || !/^\d+$/.test(value.trim())) {
    return null;
  }
  return Number.parseInt(value, 10);
}

export async function fetchSbomPage(url: string, signal?: AbortSignal): Promise<SbomPage> {
  const headers = new Headers({ [UI_HEADER]: 'true', Accept: 'application/json' });
  const response = await fetch(url, { headers, credentials: 'same-origin', signal });
  if (!response.ok) {
    throw new SbomRequestError(response.status, parseRetryAfter(response.headers.get('Retry-After')));
  }
  return (await response.json()) as SbomPage;
}
