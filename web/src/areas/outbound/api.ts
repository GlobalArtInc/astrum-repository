import { CSRF_COOKIE, CSRF_HEADER, UI_HEADER, V1 } from '../../api/client';
import { errorFromBody } from '../../api/errors';
import { readCookie } from '../../i18n/language';

export const HTTP_SETTINGS = `${V1}/http`;
export const EMAIL_SETTINGS = `${V1}/email`;
export const EMAIL_VERIFY = `${V1}/email/verify`;
export const SSL = `${V1}/security/ssl`;
export const TRUSTSTORE = `${V1}/security/ssl/truststore`;
export const CERTIFICATE_DETAILS = '/service/rest/internal/ui/ssl/certificate-details';

export const SETTINGS_READ = 'astrum:settings:read';
export const SETTINGS_UPDATE = 'astrum:settings:update';
export const TRUSTSTORE_READ = 'astrum:ssl-truststore:read';
export const TRUSTSTORE_CREATE = 'astrum:ssl-truststore:create';
export const TRUSTSTORE_DELETE = 'astrum:ssl-truststore:delete';

export const PLACEHOLDER = '#~ASTRUM~PLACEHOLDER~PASSWORD~#';

export const HTTP_DOCS = 'http://links.globalart.dev/products/astrum/docs/http-request-and-proxy-settings';
export const EMAIL_DOCS = 'https://help.globalart.dev/en/email-server-configuration.html';
export const SSL_DOCS = 'http://links.globalart.dev/products/astrum/docs/ssl-certificates';

export const SSL_LIST = '/admin/security/ssl-certificates';
export const SSL_ADD = '/admin/security/ssl-certificates/add';

export function certificateHref(id: string): string {
  return `${SSL_LIST}/${encodeURIComponent(id)}`;
}

export interface Certificate {
  id: string;
  fingerprint: string;
  pem: string;
  serialNumber: string | null;
  subjectCommonName: string | null;
  subjectOrganization: string | null;
  subjectOrganizationalUnit: string | null;
  issuerCommonName: string | null;
  issuerOrganization: string | null;
  issuerOrganizationalUnit: string | null;
  issuedOn: number;
  expiresOn: number;
}

export interface ProxyAuth {
  enabled: boolean;
  username: string | null;
  password: string | null;
  ntlmHost: string | null;
  ntlmDomain: string | null;
}

export interface ProxySettings {
  enabled: boolean;
  host: string | null;
  port: string | null;
  authInfo: ProxyAuth | null;
}

export interface HttpSettings {
  userAgent: string | null;
  timeout: number | null;
  retries: number | null;
  httpProxy: ProxySettings | null;
  httpsProxy: ProxySettings | null;
  nonProxyHosts: string[] | null;
}

export interface EmailSettings {
  enabled: boolean;
  host: string | null;
  port: number | null;
  username: string | null;
  password: string | null;
  fromAddress: string | null;
  subjectPrefix: string | null;
  startTlsEnabled: boolean;
  startTlsRequired: boolean;
  sslOnConnectEnabled: boolean;
  sslServerIdentityCheckEnabled: boolean;
  astrumTrustStoreEnabled: boolean;
}

export interface TextReply {
  status: number;
  body: unknown;
}

async function read(response: Response): Promise<unknown> {
  const text = await response.text();
  if ((response.headers.get('content-type') ?? '').includes('json') && text !== '') {
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  return text;
}

export async function postText(path: string, text: string): Promise<TextReply> {
  const headers = new Headers({ [UI_HEADER]: 'true', Accept: 'application/json' });
  const csrf = readCookie(CSRF_COOKIE);
  if (csrf) {
    headers.set(CSRF_HEADER, csrf);
  }
  const response = await fetch(path, {
    method: 'POST',
    headers,
    body: new Blob([text], { type: 'text/plain' }),
    credentials: 'same-origin',
  });
  return { status: response.status, body: await read(response) };
}

export async function sendText<T>(path: string, text: string): Promise<T> {
  const reply = await postText(path, text);
  if (reply.status < 200 || reply.status >= 300) {
    throw errorFromBody(reply.status, '', reply.body);
  }
  return reply.body as T;
}
