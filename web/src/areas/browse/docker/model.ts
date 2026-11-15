import { path, V1, withQuery } from '../../../api';
import type { Tone } from '../../../ui';
import type { SbomFormat, SbomPackage, Signature, SignatureScheme, SignatureState } from './api';

export const SBOM_PAGE_SIZE = 100;
export const FILTER_DELAY_MS = 300;
const DIGEST_HEX_SHOWN = 12;
const PURL_SHOWN = 60;

export type SbomProblem = 'notFound' | 'badRequest' | 'tooLarge' | 'unreadable' | 'busy' | 'failed';

export function supplyChainUrl(repository: string, image: string, reference: string): string {
  return withQuery(`${V1}${path`/docker/${repository}/supply-chain`}`, { image, reference });
}

export function sbomUrl(
  repository: string,
  image: string,
  digest: string,
  page: number,
  filter: string,
): string {
  return withQuery(`${V1}${path`/docker/${repository}/sbom`}`, {
    image,
    digest,
    page,
    filter: filter.trim(),
  });
}

export function shortDigest(digest: string): string {
  const colon = digest.indexOf(':');
  const algorithm = colon < 0 ? '' : digest.slice(0, colon + 1);
  const hex = digest.slice(colon + 1);
  return `${algorithm}${hex.slice(0, DIGEST_HEX_SHOWN)}${hex.length > DIGEST_HEX_SHOWN ? '…' : ''}`;
}

export function signatureTone(state: SignatureState | undefined): Tone {
  if (state === 'verified') {
    return 'success';
  }
  if (state === 'invalid') {
    return 'danger';
  }
  return 'neutral';
}

const STATE_KEYS: Record<SignatureState | 'absent', string> = {
  verified: 'browse.docker.state.verified',
  invalid: 'browse.docker.state.invalid',
  unsupported: 'browse.docker.state.unsupported',
  unknown: 'browse.docker.state.unknown',
  absent: 'browse.docker.state.unknown',
};

export function signatureStateKey(state: SignatureState | undefined): string {
  return STATE_KEYS[state ?? 'absent'];
}

const SCHEME_KEYS: Record<SignatureScheme, string> = {
  legacy: 'browse.docker.scheme.legacy',
  referrer: 'browse.docker.scheme.referrer',
  bundle: 'browse.docker.scheme.bundle',
  notation: 'browse.docker.scheme.notation',
};

export function signatureSchemeKey(scheme: SignatureScheme): string {
  return SCHEME_KEYS[scheme];
}

const FORMAT_KEYS: Record<SbomFormat, string> = {
  spdx: 'browse.docker.sbom.format.spdx',
  cyclonedx: 'browse.docker.sbom.format.cyclonedx',
  unknown: 'browse.docker.sbom.format.unknown',
};

export function sbomFormatKey(format: SbomFormat): string {
  return FORMAT_KEYS[format];
}

export function signedBy(signatures: Signature[]): string | null {
  const verified = signatures.find((signature) => signature.state === 'verified');
  return verified ? (verified.signer ?? '') : null;
}

export function sbomProblem(status: number): SbomProblem {
  switch (status) {
    case 404:
      return 'notFound';
    case 400:
      return 'badRequest';
    case 413:
      return 'tooLarge';
    case 422:
      return 'unreadable';
    case 503:
      return 'busy';
    default:
      return 'failed';
  }
}

export function licencesText(licenses: string[]): string {
  return licenses.length > 0 ? licenses.join(', ') : '—';
}

export function truncatePurl(purl: string): string {
  return purl.length > PURL_SHOWN ? `${purl.slice(0, PURL_SHOWN - 1)}…` : purl;
}

export function packageKey(page: number, index: number, item: SbomPackage): string {
  return `${page}-${index}-${item.name}-${item.version}`;
}
