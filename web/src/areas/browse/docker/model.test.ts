import { describe, expect, it } from 'vitest';
import { parseRetryAfter } from './api';
import {
  licencesText,
  sbomProblem,
  sbomUrl,
  shortDigest,
  signatureStateKey,
  signatureTone,
  signedBy,
  supplyChainUrl,
  truncatePurl,
} from './model';

const DIGEST = `sha256:${'ab'.repeat(32)}`;

describe('docker supply chain model', () => {
  it('shortens a digest to its algorithm and the first twelve hex digits', () => {
    expect(shortDigest(DIGEST)).toBe('sha256:abababababab…');
    expect(shortDigest('sha256:abc')).toBe('sha256:abc');
  });

  it('maps a signature state to its badge tone and text key', () => {
    expect(signatureTone('verified')).toBe('success');
    expect(signatureTone('invalid')).toBe('danger');
    expect(signatureTone('unsupported')).toBe('neutral');
    expect(signatureTone(undefined)).toBe('neutral');
    expect(signatureStateKey('invalid')).toBe('browse.docker.state.invalid');
    expect(signatureStateKey(undefined)).toBe('browse.docker.state.unknown');
  });

  it('builds the SBOM query with the page and only a non-blank filter', () => {
    expect(sbomUrl('images', 'app', DIGEST, 2, '  log4j ')).toBe(
      `/service/rest/v1/docker/images/sbom?image=app&digest=${encodeURIComponent(DIGEST)}&page=2&filter=log4j`,
    );
    expect(sbomUrl('images', 'app', DIGEST, 1, '   ')).not.toContain('filter=');
    expect(supplyChainUrl('images', 'team/app', '1.0')).toBe(
      '/service/rest/v1/docker/images/supply-chain?image=team%2Fapp&reference=1.0',
    );
  });

  it('maps an SBOM error status to its message', () => {
    expect(sbomProblem(413)).toBe('tooLarge');
    expect(sbomProblem(422)).toBe('unreadable');
    expect(sbomProblem(503)).toBe('busy');
    expect(sbomProblem(404)).toBe('notFound');
    expect(sbomProblem(500)).toBe('failed');
  });

  it('reads the signer only from a verified signature', () => {
    expect(signedBy([{ digest: DIGEST, scheme: 'referrer', state: 'invalid' }])).toBeNull();
    expect(signedBy([{ digest: DIGEST, scheme: 'bundle', state: 'verified', signer: 'release-key' }])).toBe(
      'release-key',
    );
  });

  it('reads Retry-After as whole seconds and ignores other values', () => {
    expect(parseRetryAfter('7')).toBe(7);
    expect(parseRetryAfter('Wed, 21 Oct 2026 07:28:00 GMT')).toBeNull();
    expect(parseRetryAfter(null)).toBeNull();
  });

  it('shows the licences and cuts a long purl with an ellipsis', () => {
    expect(licencesText([])).toBe('—');
    expect(licencesText(['MIT', 'Apache-2.0'])).toBe('MIT, Apache-2.0');
    expect(truncatePurl(`pkg:npm/${'a'.repeat(80)}`)).toHaveLength(60);
  });
});
