import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Policy } from './model';
import PoliciesPage from './PoliciesPage';

const STRICT: Policy = {
  name: 'strict',
  notes: '',
  vulnerabilities: { mode: 'block', severity: 'high', malicious: 'block', unscanned: 'allow', notify: null },
  licenses: { mode: 'off', list: null, ids: [], unknown: 'report' },
  signatures: { mode: 'off', keys: [] },
  revision: 3,
};

const LOOSE: Policy = {
  ...STRICT,
  name: 'loose',
  vulnerabilities: { ...STRICT.vulnerabilities, mode: 'report' },
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the policies list', () => {
  it('lists the policies with the mode of each block and links to them', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/policies': () => jsonResponse([STRICT, LOOSE]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PoliciesPage />);
    const row = (await screen.findByRole('button', { name: 'Open strict' })).closest('tr') as HTMLElement;
    expect(row).toHaveTextContent('Block');
    expect(row).toHaveTextContent('Off');
    expect(screen.getByRole('link', { name: 'Create Policy' })).toBeInTheDocument();
  });

  it('hides the create button from a reader', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/policies': () => jsonResponse([STRICT]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PoliciesPage />, { state: { ...ADMIN_STATE, permissions: ['astrum:policies:read'] } });
    expect(await screen.findByRole('button', { name: 'Open strict' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Create Policy' })).not.toBeInTheDocument();
  });
});
