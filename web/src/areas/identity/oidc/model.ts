import type { Translate } from '../../../i18n';
import type { OidcSettings } from '../api';

export interface OidcValues {
  issuerUrl: string;
  clientId: string;
  clientSecret: string;
  removeSecret: boolean;
  idpAuthorizationUrl: string;
  idpLogoutUrl: string;
  idpTokenUrl: string;
  idpJwksUrl: string;
  useTrustStore: boolean;
  usernameClaim: string;
  firstNameClaim: string;
  lastNameClaim: string;
  emailClaim: string;
  groupsClaim: string;
  idpJwsAlgorithm: string;
  idpJwks: string;
  authorizationCustomParams: string;
  tokenRequestCustomParams: string;
  exactMatchClaims: string;
}

export type OidcErrors = Partial<Record<keyof OidcValues, string>>;

const JSON_FIELDS = ['authorizationCustomParams', 'tokenRequestCustomParams', 'exactMatchClaims'] as const;

function json(map: Record<string, string>): string {
  return Object.keys(map).length === 0 ? '{}' : JSON.stringify(map, null, 2);
}

export function fromSettings(settings: OidcSettings): OidcValues {
  return {
    issuerUrl: settings.issuerUrl ?? '',
    clientId: settings.clientId ?? '',
    clientSecret: '',
    removeSecret: false,
    idpAuthorizationUrl: settings.idpAuthorizationUrl ?? '',
    idpLogoutUrl: settings.idpLogoutUrl ?? '',
    idpTokenUrl: settings.idpTokenUrl ?? '',
    idpJwksUrl: settings.idpJwksUrl ?? '',
    useTrustStore: settings.useTrustStore,
    usernameClaim: settings.usernameClaim ?? '',
    firstNameClaim: settings.firstNameClaim ?? '',
    lastNameClaim: settings.lastNameClaim ?? '',
    emailClaim: settings.emailClaim ?? '',
    groupsClaim: settings.groupsClaim ?? '',
    idpJwsAlgorithm: settings.idpJwsAlgorithm ?? '',
    idpJwks: settings.idpJwks ?? '',
    authorizationCustomParams: json(settings.authorizationCustomParams),
    tokenRequestCustomParams: json(settings.tokenRequestCustomParams),
    exactMatchClaims: json(settings.exactMatchClaims),
  };
}

export function parseMap(text: string): Record<string, string> | null {
  const trimmed = text.trim();
  if (trimmed === '') {
    return {};
  }
  try {
    const value: unknown = JSON.parse(trimmed);
    if (value === null || typeof value !== 'object' || Array.isArray(value)) {
      return null;
    }
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.some(([, entry]) => typeof entry !== 'string')) {
      return null;
    }
    return Object.fromEntries(entries) as Record<string, string>;
  } catch {
    return null;
  }
}

export function secureUrl(value: string): boolean {
  return /^https:\/\/\S+$/i.test(value.trim());
}

export function validateOidc(t: Translate, values: OidcValues, labels: Record<string, string>): OidcErrors {
  const errors: OidcErrors = {};
  if (values.clientId.trim() === '') {
    errors.clientId = t('identity.oidc.clientIdRequired');
  }
  for (const field of JSON_FIELDS) {
    if (parseMap(values[field]) === null) {
      errors[field] = t('identity.oidc.badJson', { field: labels[field] ?? field });
    }
  }
  return errors;
}

function optional(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === '' ? null : trimmed;
}

export function toOidcRequest(values: OidcValues, secretStored: boolean) {
  let clientSecret: string | null = null;
  if (values.clientSecret !== '') {
    clientSecret = values.clientSecret;
  } else if (secretStored && values.removeSecret) {
    clientSecret = '';
  }
  return {
    issuerUrl: optional(values.issuerUrl),
    clientId: values.clientId.trim(),
    clientSecret,
    idpAuthorizationUrl: optional(values.idpAuthorizationUrl),
    idpLogoutUrl: optional(values.idpLogoutUrl),
    idpTokenUrl: optional(values.idpTokenUrl),
    idpJwksUrl: optional(values.idpJwksUrl),
    useTrustStore: values.useTrustStore,
    usernameClaim: optional(values.usernameClaim),
    firstNameClaim: optional(values.firstNameClaim),
    lastNameClaim: optional(values.lastNameClaim),
    emailClaim: optional(values.emailClaim),
    groupsClaim: optional(values.groupsClaim),
    idpJwsAlgorithm: optional(values.idpJwsAlgorithm),
    idpJwks: optional(values.idpJwks),
    authorizationCustomParams: parseMap(values.authorizationCustomParams) ?? {},
    tokenRequestCustomParams: parseMap(values.tokenRequestCustomParams) ?? {},
    exactMatchClaims: parseMap(values.exactMatchClaims) ?? {},
  };
}
