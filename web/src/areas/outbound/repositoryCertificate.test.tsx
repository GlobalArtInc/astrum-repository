import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { RepositoryForm } from '../repositories/components/RepositoryForm';
import type { RepositoryDocument } from '../repositories/model/document';
import reference from '../repositories/model/reference-documents.json';
import type { Certificate } from './api';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

afterEach(() => {
  vi.unstubAllGlobals();
});

const CERTIFICATE: Certificate = {
  id: 'C8:3E',
  fingerprint: 'C8:3E',
  pem: '-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----',
  serialNumber: '1',
  subjectCommonName: 'remote.example',
  subjectOrganization: 'Remote Org',
  subjectOrganizationalUnit: null,
  issuerCommonName: 'Remote CA',
  issuerOrganization: null,
  issuerOrganizationalUnit: null,
  issuedOn: Date.UTC(2025, 0, 1),
  expiresOn: Date.UTC(2030, 0, 1),
};

function proxy(remoteUrl: string): RepositoryDocument {
  const document = structuredClone(
    (reference as unknown as Record<string, RepositoryDocument>)['raw/proxy/max'] as RepositoryDocument,
  );
  (document.proxy as Record<string, unknown>).remoteUrl = remoteUrl;
  return document;
}

function renderForm(document: RepositoryDocument, permissions = ADMIN_STATE.permissions) {
  return renderWithApp(
    <RepositoryForm
      spec={{ format: 'raw', type: 'proxy' }}
      creating={false}
      document={document}
      repositories={[]}
      blobStores={null}
      submitLabel="Save"
      onSubmit={async () => {}}
    />,
    { state: { ...ADMIN_STATE, permissions } },
  );
}

describe('the certificate of a proxy remote', () => {
  it('is shown for an https remote and can be removed from the truststore', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/routing-rules': () => jsonResponse([]),
      'GET /service/rest/internal/ui/cleanup-policies/names': () => jsonResponse([]),
      'GET /service/rest/v1/security/ssl?host=remote.example&port=8443': () => jsonResponse(CERTIFICATE),
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([CERTIFICATE]),
      'DELETE /service/rest/v1/security/ssl/truststore/C8%3A3E': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderForm(proxy('https://remote.example:8443/files/'));
    await userEvent.click(screen.getByRole('button', { name: 'View Certificate' }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText('Remote Org')).toBeInTheDocument();
    expect(within(dialog).queryByText(/untrusted connection/)).not.toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Remove Certificate from Truststore' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true));
  });

  it('is not offered for plain http remotes or without the truststore permission', () => {
    vi.stubGlobal('fetch', fakeServer({}).fetch);
    const { unmount } = renderForm(proxy('http://remote.example/files/'));
    expect(screen.queryByRole('button', { name: 'View Certificate' })).not.toBeInTheDocument();
    unmount();
    renderForm(proxy('https://remote.example/files/'), ['astrum:repository-admin:*:*:*']);
    expect(screen.queryByRole('button', { name: 'View Certificate' })).not.toBeInTheDocument();
  });
});
