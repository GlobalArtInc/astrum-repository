import { ApiError, fieldName } from '../../api';
import {
  type Certificate,
  type EmailSettings,
  type HttpSettings,
  PLACEHOLDER,
  type ProxyAuth,
  type ProxySettings,
} from './api';

export type AuthType = 'none' | 'username';

export interface HttpValues {
  userAgent: string;
  timeout: number | null;
  retries: number | null;
  httpEnabled: boolean;
  httpHost: string;
  httpPort: string;
  httpAuth: AuthType;
  httpUsername: string;
  httpPassword: string;
  httpNtlmHost: string;
  httpNtlmDomain: string;
  httpsEnabled: boolean;
  httpsHost: string;
  httpsPort: string;
  httpsAuth: AuthType;
  httpsUsername: string;
  httpsPassword: string;
  httpsNtlmHost: string;
  httpsNtlmDomain: string;
  nonProxyHosts: string[];
}

export type ProxyKind = 'http' | 'https';

const text = (value: string | number | null | undefined) =>
  value === null || value === undefined ? '' : String(value);

export function storedPassword(password: string | null | undefined): boolean {
  return password !== null && password !== undefined && password !== '';
}

export function fromHttp(settings: HttpSettings): HttpValues {
  const proxy = (value: ProxySettings | null) => ({
    enabled: value?.enabled === true,
    host: text(value?.host),
    port: text(value?.port),
    auth: (value?.authInfo?.enabled ? 'username' : 'none') as AuthType,
    username: text(value?.authInfo?.username),
    ntlmHost: text(value?.authInfo?.ntlmHost),
    ntlmDomain: text(value?.authInfo?.ntlmDomain),
  });
  const http = proxy(settings.httpProxy);
  const https = proxy(settings.httpsProxy);
  return {
    userAgent: text(settings.userAgent),
    timeout: settings.timeout ?? null,
    retries: settings.retries ?? null,
    httpEnabled: http.enabled,
    httpHost: http.host,
    httpPort: http.port,
    httpAuth: http.auth,
    httpUsername: http.username,
    httpPassword: '',
    httpNtlmHost: http.ntlmHost,
    httpNtlmDomain: http.ntlmDomain,
    httpsEnabled: https.enabled,
    httpsHost: https.host,
    httpsPort: https.port,
    httpsAuth: https.auth,
    httpsUsername: https.username,
    httpsPassword: '',
    httpsNtlmHost: https.ntlmHost,
    httpsNtlmDomain: https.ntlmDomain,
    nonProxyHosts: [...(settings.nonProxyHosts ?? [])],
  };
}

const orNull = (value: string) => (value.trim() === '' ? null : value.trim());

function proxyRequest(values: HttpValues, kind: ProxyKind, stored: boolean): ProxySettings {
  const pick = <K extends string>(name: K) => values[`${kind}${name}` as keyof HttpValues];
  const enabled = pick('Enabled') as boolean;
  const auth = (pick('Auth') as AuthType) === 'username';
  const password = pick('Password') as string;
  const authInfo: ProxyAuth = {
    enabled: enabled && auth,
    username: auth ? orNull(pick('Username') as string) : null,
    password: !auth ? null : password !== '' ? password : stored ? PLACEHOLDER : null,
    ntlmHost: auth ? orNull(pick('NtlmHost') as string) : null,
    ntlmDomain: auth ? orNull(pick('NtlmDomain') as string) : null,
  };
  return {
    enabled,
    host: orNull(pick('Host') as string),
    port: orNull(pick('Port') as string),
    authInfo,
  };
}

export function toHttp(
  values: HttpValues,
  stored: { http: boolean; https: boolean } = { http: false, https: false },
): HttpSettings {
  const anyProxy = values.httpEnabled || values.httpsEnabled;
  return {
    userAgent: orNull(values.userAgent),
    timeout: values.timeout,
    retries: values.retries,
    httpProxy: proxyRequest(values, 'http', stored.http),
    httpsProxy: proxyRequest(values, 'https', stored.https),
    nonProxyHosts: anyProxy ? values.nonProxyHosts : [],
  };
}

export interface HttpTexts {
  timeout: string;
  retries: string;
  hostRequired: Record<ProxyKind, string>;
  portRequired: Record<ProxyKind, string>;
  hostShape: string;
  port: string;
  username: string;
}

const HOST = /^[A-Za-z0-9._-]+$|^\[[0-9A-Fa-f:.]+\]$|^[0-9A-Fa-f:]+$/;

export function validPort(value: string): boolean {
  if (!/^\d+$/.test(value.trim())) {
    return false;
  }
  const port = Number(value);
  return port >= 1 && port <= 65535;
}

export function validateHttp(
  values: HttpValues,
  texts: HttpTexts,
): Partial<Record<keyof HttpValues, string>> {
  const errors: Partial<Record<keyof HttpValues, string>> = {};
  if (
    values.timeout !== null &&
    (!Number.isInteger(values.timeout) || values.timeout < 1 || values.timeout > 3600)
  ) {
    errors.timeout = texts.timeout;
  }
  if (
    values.retries !== null &&
    (!Number.isInteger(values.retries) || values.retries < 0 || values.retries > 10)
  ) {
    errors.retries = texts.retries;
  }
  for (const kind of ['http', 'https'] as const) {
    if (!values[`${kind}Enabled`]) {
      continue;
    }
    const host = values[`${kind}Host`].trim();
    if (host === '') {
      errors[`${kind}Host`] = texts.hostRequired[kind];
    } else if (!HOST.test(host)) {
      errors[`${kind}Host`] = texts.hostShape;
    }
    const port = values[`${kind}Port`].trim();
    if (port === '') {
      errors[`${kind}Port`] = texts.portRequired[kind];
    } else if (!validPort(port)) {
      errors[`${kind}Port`] = texts.port;
    }
    if (values[`${kind}Auth`] === 'username' && values[`${kind}Username`].trim() === '') {
      errors[`${kind}Username`] = texts.username;
    }
  }
  return errors;
}

const HTTP_FIELDS: Record<string, keyof HttpValues> = {
  userAgent: 'userAgent',
  timeout: 'timeout',
  retries: 'retries',
  nonProxyHosts: 'nonProxyHosts',
};

export function httpField(id: string): keyof HttpValues | undefined {
  const name = fieldName(id);
  const proxy = /^(https?)Proxy\.(?:authInfo\.)?(\w+)$/.exec(name);
  if (proxy) {
    const kind = proxy[1] as ProxyKind;
    const part = proxy[2] ?? '';
    const map: Record<string, string> = {
      host: 'Host',
      port: 'Port',
      username: 'Username',
      password: 'Password',
      ntlmHost: 'NtlmHost',
      ntlmDomain: 'NtlmDomain',
      enabled: 'Enabled',
    };
    const suffix = map[part];
    return suffix ? (`${kind}${suffix}` as keyof HttpValues) : undefined;
  }
  return HTTP_FIELDS[name];
}

export function renameFieldErrors(error: unknown, rename: (id: string) => string | undefined): unknown {
  if (!(error instanceof ApiError) || error.fieldErrors.length === 0) {
    return error;
  }
  return new ApiError(error.status, error.message, {
    id: error.id,
    fieldErrors: error.fieldErrors.map((problem) => ({
      id: rename(problem.id) ?? problem.id,
      message: problem.message,
    })),
  });
}

export function hostPattern(value: string): string | null {
  const trimmed = value.trim();
  if (trimmed === '' || /\s/.test(trimmed)) {
    return null;
  }
  return trimmed;
}

export interface EmailValues {
  enabled: boolean;
  host: string;
  port: number | null;
  astrumTrustStoreEnabled: boolean;
  useAuthentication: boolean;
  username: string;
  password: string;
  fromAddress: string;
  subjectPrefix: string;
  startTlsEnabled: boolean;
  startTlsRequired: boolean;
  sslOnConnectEnabled: boolean;
  sslServerIdentityCheckEnabled: boolean;
}

export function fromEmail(settings: EmailSettings): EmailValues {
  return {
    enabled: settings.enabled === true,
    host: text(settings.host),
    port: settings.port ?? null,
    astrumTrustStoreEnabled: settings.astrumTrustStoreEnabled === true,
    useAuthentication: text(settings.username) !== '' || storedPassword(settings.password),
    username: text(settings.username),
    password: '',
    fromAddress: text(settings.fromAddress),
    subjectPrefix: text(settings.subjectPrefix),
    startTlsEnabled: settings.startTlsEnabled === true,
    startTlsRequired: settings.startTlsRequired === true,
    sslOnConnectEnabled: settings.sslOnConnectEnabled === true,
    sslServerIdentityCheckEnabled: settings.sslServerIdentityCheckEnabled === true,
  };
}

export function toEmail(values: EmailValues, stored: boolean): EmailSettings {
  const auth = values.useAuthentication;
  return {
    enabled: values.enabled,
    host: values.host.trim(),
    port: values.port,
    username: auth ? values.username.trim() : '',
    password: !auth ? null : values.password !== '' ? values.password : stored ? PLACEHOLDER : null,
    fromAddress: values.fromAddress.trim(),
    subjectPrefix: orNull(values.subjectPrefix),
    startTlsEnabled: values.startTlsEnabled,
    startTlsRequired: values.startTlsRequired,
    sslOnConnectEnabled: values.sslOnConnectEnabled,
    sslServerIdentityCheckEnabled: values.sslServerIdentityCheckEnabled,
    astrumTrustStoreEnabled: values.astrumTrustStoreEnabled,
  };
}

export interface EmailTexts {
  host: string;
  port: string;
  portRange: string;
  from: string;
  address: string;
}

export const ADDRESS = /^[^\s@]+@[^\s@]+$/;

export function validateEmail(
  values: EmailValues,
  texts: EmailTexts,
): Partial<Record<keyof EmailValues, string>> {
  const errors: Partial<Record<keyof EmailValues, string>> = {};
  if (values.host.trim() === '') {
    errors.host = texts.host;
  }
  if (values.port === null) {
    errors.port = texts.port;
  } else if (!Number.isInteger(values.port) || values.port < 1 || values.port > 65535) {
    errors.port = texts.portRange;
  }
  if (values.fromAddress.trim() === '') {
    errors.fromAddress = texts.from;
  } else if (!ADDRESS.test(values.fromAddress.trim())) {
    errors.fromAddress = texts.address;
  }
  return errors;
}

export function emailField(id: string): keyof EmailValues | undefined {
  const name = fieldName(id);
  const known: Array<keyof EmailValues> = [
    'host',
    'port',
    'username',
    'password',
    'fromAddress',
    'subjectPrefix',
  ];
  return known.find((field) => field === name);
}

export const EXPIRING_DAYS = 30;
const DAY = 24 * 60 * 60 * 1000;

export type Expiry = 'valid' | 'expiring' | 'expired';

export function expiry(certificate: Pick<Certificate, 'expiresOn'>, now = Date.now()): Expiry {
  if (certificate.expiresOn <= now) {
    return 'expired';
  }
  return certificate.expiresOn - now <= EXPIRING_DAYS * DAY ? 'expiring' : 'valid';
}

export function certificateName(certificate: Pick<Certificate, 'subjectCommonName' | 'fingerprint'>): string {
  return certificate.subjectCommonName?.trim() || certificate.fingerprint;
}

export function matchesCertificate(certificate: Certificate, filter: string): boolean {
  const needle = filter.trim().toLowerCase();
  if (needle === '') {
    return true;
  }
  return [
    certificate.subjectCommonName,
    certificate.subjectOrganization,
    certificate.issuerCommonName,
    certificate.issuerOrganization,
    certificate.fingerprint,
  ].some((value) => (value ?? '').toLowerCase().includes(needle));
}

export function parseServer(input: string): { host: string; port: number } | null {
  const trimmed = input.trim();
  if (trimmed === '' || /\s/.test(trimmed)) {
    return null;
  }
  try {
    const url = new URL(trimmed.includes('://') ? trimmed : `https://${trimmed}`);
    const host = url.hostname.replace(/^\[(.*)\]$/, '$1');
    if (host === '') {
      return null;
    }
    const port = url.port === '' ? 443 : Number(url.port);
    return { host, port };
  } catch {
    return null;
  }
}

export function remoteServer(remoteUrl: string): { host: string; port: number } | null {
  try {
    const url = new URL(remoteUrl.trim());
    if (url.protocol !== 'https:' || url.hostname === '') {
      return null;
    }
    return { host: url.hostname.replace(/^\[(.*)\]$/, '$1'), port: url.port === '' ? 443 : Number(url.port) };
  } catch {
    return null;
  }
}

export function looksLikePem(value: string): boolean {
  return /-----BEGIN CERTIFICATE-----[\s\S]+-----END CERTIFICATE-----/.test(value);
}
