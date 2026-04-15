import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
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
});
