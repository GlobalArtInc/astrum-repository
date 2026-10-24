import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Certificate } from '../../outbound/api';
import OidcPage from './OidcPage';

const PATH = '/service/rest/internal/ui/oauth2';
const EMPTY = {
  issuerUrl: null,
  clientId: null,
  clientSecret: null,
  clientSecretSet: false,
  idpAuthorizationUrl: null,
  idpTokenUrl: null,
  idpJwksUrl: null,
  idpLogoutUrl: null,
  idpJwsAlgorithm: null,
  idpJwks: null,
  usernameClaim: 'preferred_username',
  firstNameClaim: 'given_name',
  lastNameClaim: 'family_name',
  emailClaim: 'email',
  groupsClaim: null,
  exactMatchClaims: {},
  authorizationCustomParams: {},
  tokenRequestCustomParams: {},
  useTrustStore: false,
  redirectUri: 'http://localhost/oidc/callback',
  configured: false,
  enabled: false,
};

const CERTIFICATE: Certificate = {
  id: 'AB:CD',
  fingerprint: 'AB:CD',
  pem: '-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----',
  serialNumber: '7',
  subjectCommonName: 'sso.example.com',
  subjectOrganization: 'Identity Org',
  subjectOrganizationalUnit: null,
  issuerCommonName: 'Private CA',
  issuerOrganization: null,
  issuerOrganizationalUnit: null,
  issuedOn: Date.UTC(2025, 0, 1),
  expiresOn: Date.UTC(2030, 0, 1),
};

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(reply: () => Response = () => new Response(null, { status: 204 })) {
  const fake = fakeServer({
    [`GET ${PATH}`]: () => jsonResponse(EMPTY),
    [`PUT ${PATH}`]: reply,
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

const field = (name: string) => screen.getByRole('textbox', { name });

describe('the OpenID Connect page', () => {
  it('shows the redirect URI and saves the settings', async () => {
    const fake = serve();
    renderWithApp(<OidcPage />);
    expect(await screen.findByText('http://localhost/oidc/callback')).toBeInTheDocument();
    expect(field('Username Claim')).toHaveValue('preferred_username');
    expect(field('Exact Match Claims')).toHaveValue('{}');
    expect(screen.getByRole('checkbox', { name: 'Use the Astrum Repository Truststore' })).toBeDisabled();
    const save = screen.getByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    await userEvent.type(field('Issuer URL'), 'https://sso.example.com');
    expect(screen.getByRole('checkbox', { name: 'Use the Astrum Repository Truststore' })).toBeEnabled();
    await userEvent.click(save);
    expect(await screen.findByText('Client ID is required')).toBeInTheDocument();
    await userEvent.type(field('Client ID'), 'astrum');
    await userEvent.type(screen.getByLabelText('Client Secret'), 'top-secret');
    await userEvent.type(field('Groups Claim'), 'groups');
    await userEvent.clear(field('Exact Match Claims'));
    await userEvent.type(field('Exact Match Claims'), '{{"tenant": "main"}');
    await userEvent.click(save);
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
        issuerUrl: 'https://sso.example.com',
        clientId: 'astrum',
        clientSecret: 'top-secret',
        groupsClaim: 'groups',
        exactMatchClaims: { tenant: 'main' },
        authorizationCustomParams: {},
      }),
    );
    expect(await screen.findByText('OpenID Connect settings saved successfully')).toBeInTheDocument();
  }, 30_000);

  it('puts server problems on their fields and refuses bad JSON', async () => {
    serve(() =>
      jsonResponse([{ id: 'issuerUrl', message: 'Could not load the identity provider configuration' }], 400),
    );
    renderWithApp(<OidcPage />);
    await userEvent.type(await screen.findByRole('textbox', { name: 'Client ID' }), 'astrum');
    await userEvent.clear(field('Token Request Custom Parameters'));
    await userEvent.type(field('Token Request Custom Parameters'), '[[]');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(
      await screen.findByText(
        'The field "Token Request Custom Parameters" must contain a valid JSON object with string values',
      ),
    ).toBeInTheDocument();
    await userEvent.clear(field('Token Request Custom Parameters'));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Could not load the identity provider configuration')).toBeInTheDocument();
  }, 30_000);

  it('shows the certificate of an https provider', async () => {
    const fake = fakeServer({
      [`GET ${PATH}`]: () =>
        jsonResponse({ ...EMPTY, issuerUrl: 'https://sso.example.com:8443/realms/main', clientId: 'astrum' }),
      'GET /service/rest/v1/security/ssl?host=sso.example.com&port=8443': () => jsonResponse(CERTIFICATE),
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([]),
      'POST /service/rest/v1/security/ssl/truststore': () => jsonResponse(CERTIFICATE, 201),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<OidcPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'View Certificate' }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText('Identity Org')).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Certificate to Truststore' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'POST')).toBe(true));
  }, 30_000);

  it('offers the certificate only for https endpoints and with the truststore permission', async () => {
    serve();
    const { unmount } = renderWithApp(<OidcPage />);
    expect(await screen.findByRole('button', { name: 'View Certificate' })).toBeDisabled();
    await userEvent.type(field('Issuer URL'), 'http://sso.example.com');
    expect(screen.getByRole('button', { name: 'View Certificate' })).toBeDisabled();
    await userEvent.type(field('IDP Token URL'), 'https://sso.example.com/token');
    expect(screen.getByRole('button', { name: 'View Certificate' })).toBeEnabled();
    unmount();
    renderWithApp(<OidcPage />, { state: { ...ADMIN_STATE, permissions: ['astrum:settings:*'] } });
    expect(await screen.findByText('http://localhost/oidc/callback')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'View Certificate' })).not.toBeInTheDocument();
  }, 30_000);
});
