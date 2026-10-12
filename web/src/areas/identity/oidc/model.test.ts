import { describe, expect, it } from 'vitest';
import type { Translate } from '../../../i18n';
import type { OidcSettings } from '../api';
import { fromSettings, parseMap, secureUrl, toOidcRequest, validateOidc } from './model';

const t = ((key: string, vars?: Record<string, string | number>) =>
  vars ? `${key}:${vars.field}` : key) as unknown as Translate;

const SETTINGS: OidcSettings = {
  issuerUrl: 'https://sso.example.com',
  clientId: 'astrum',
  clientSecret: null,
  clientSecretSet: true,
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
  groupsClaim: 'groups',
  exactMatchClaims: { tenant: 'main' },
  authorizationCustomParams: {},
  tokenRequestCustomParams: {},
  useTrustStore: false,
  redirectUri: 'https://repo.example.com/oidc/callback',
  configured: true,
  enabled: true,
};

describe('the OpenID Connect form model', () => {
  it('keeps a stored secret unless it is replaced or removed', () => {
    const values = fromSettings(SETTINGS);
    expect(values.exactMatchClaims).toContain('"tenant": "main"');
    expect(values.authorizationCustomParams).toBe('{}');
    expect(toOidcRequest(values, true).clientSecret).toBeNull();
    expect(toOidcRequest({ ...values, clientSecret: 'new' }, true).clientSecret).toBe('new');
    expect(toOidcRequest({ ...values, removeSecret: true }, true).clientSecret).toBe('');
    expect(toOidcRequest({ ...values, removeSecret: true }, false).clientSecret).toBeNull();
  });

  it('sends empty fields as null and JSON fields as objects', () => {
    const request = toOidcRequest(
      { ...fromSettings(SETTINGS), groupsClaim: '  ', tokenRequestCustomParams: '' },
      true,
    );
    expect(request).toMatchObject({
      issuerUrl: 'https://sso.example.com',
      groupsClaim: null,
      exactMatchClaims: { tenant: 'main' },
      tokenRequestCustomParams: {},
    });
  });

  it('requires a client ID and JSON objects of strings', () => {
    const labels = { exactMatchClaims: 'Exact Match Claims' };
    const errors = validateOidc(
      t,
      {
        ...fromSettings(SETTINGS),
        clientId: ' ',
        exactMatchClaims: '{"a": 1}',
        tokenRequestCustomParams: '[]',
      },
      labels,
    );
    expect(errors).toEqual({
      clientId: 'identity.oidc.clientIdRequired',
      exactMatchClaims: 'identity.oidc.badJson:Exact Match Claims',
      tokenRequestCustomParams: 'identity.oidc.badJson:tokenRequestCustomParams',
    });
    expect(parseMap('{"a": "b"}')).toEqual({ a: 'b' });
    expect(parseMap('nope')).toBeNull();
  });

  it('only offers the truststore for https endpoints', () => {
    expect(secureUrl('https://sso.example.com')).toBe(true);
    expect(secureUrl('http://sso.example.com')).toBe(false);
  });
});
