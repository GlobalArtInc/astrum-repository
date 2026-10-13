import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { type HttpSettings, PLACEHOLDER } from './api';
import {
  emailField,
  expiry,
  fromEmail,
  fromHttp,
  hostPattern,
  httpField,
  matchesCertificate,
  parseServer,
  remoteServer,
  renameFieldErrors,
  toEmail,
  toHttp,
  validateEmail,
  validateHttp,
} from './model';

const SETTINGS: HttpSettings = {
  userAgent: 'Probe/1',
  timeout: 15,
  retries: 3,
  httpProxy: {
    enabled: true,
    host: 'proxy.example',
    port: '3128',
    authInfo: { enabled: true, username: 'u', password: PLACEHOLDER, ntlmHost: null, ntlmDomain: null },
  },
  httpsProxy: {
    enabled: false,
    host: null,
    port: null,
    authInfo: { enabled: false, username: null, password: null, ntlmHost: null, ntlmDomain: null },
  },
  nonProxyHosts: ['*.example.com', 'localhost'],
};

const HTTP_TEXTS = {
  timeout: 'timeout',
  retries: 'retries',
  hostRequired: { http: 'http host', https: 'https host' },
  portRequired: { http: 'http port', https: 'https port' },
  hostShape: 'shape',
  port: 'port range',
  username: 'username',
};

describe('HTTP settings', () => {
  it('turns the stored document into form values and back, keeping the stored password', () => {
    const values = fromHttp(SETTINGS);
    expect(values).toMatchObject({
      userAgent: 'Probe/1',
      timeout: 15,
      httpEnabled: true,
      httpHost: 'proxy.example',
      httpPort: '3128',
      httpAuth: 'username',
      httpUsername: 'u',
      httpPassword: '',
      httpsEnabled: false,
      httpsAuth: 'none',
    });
    const request = toHttp(values, { http: true, https: false });
    expect(request.httpProxy?.authInfo).toEqual({
      enabled: true,
      username: 'u',
      password: PLACEHOLDER,
      ntlmHost: null,
      ntlmDomain: null,
    });
    expect(request.httpsProxy?.authInfo?.enabled).toBe(false);
    expect(request.nonProxyHosts).toEqual(['*.example.com', 'localhost']);
    const changed = toHttp({ ...values, httpPassword: 'new', userAgent: ' ' }, { http: true, https: false });
    expect(changed.httpProxy?.authInfo?.password).toBe('new');
    expect(changed.userAgent).toBeNull();
  });

  it('drops the exclusions without any proxy and the credentials without authentication', () => {
    const values = { ...fromHttp(SETTINGS), httpEnabled: false, httpAuth: 'none' as const };
    const request = toHttp(values, { http: true, https: false });
    expect(request.nonProxyHosts).toEqual([]);
    expect(request.httpProxy?.authInfo).toEqual({
      enabled: false,
      username: null,
      password: null,
      ntlmHost: null,
      ntlmDomain: null,
    });
  });

  it('checks ranges, hosts, ports and the user name of enabled proxies', () => {
    const values = {
      ...fromHttp(SETTINGS),
      timeout: 0,
      retries: 11,
      httpHost: 'http://proxy',
      httpPort: '70000',
      httpUsername: '',
      httpsEnabled: true,
    };
    expect(validateHttp(values, HTTP_TEXTS)).toEqual({
      timeout: 'timeout',
      retries: 'retries',
      httpHost: 'shape',
      httpPort: 'port range',
      httpUsername: 'username',
      httpsHost: 'https host',
      httpsPort: 'https port',
    });
    expect(validateHttp(fromHttp(SETTINGS), HTTP_TEXTS)).toEqual({});
  });

  it('maps server field ids onto the form', () => {
    expect(httpField('httpProxy.host')).toBe('httpHost');
    expect(httpField('httpsProxy.authInfo.username')).toBe('httpsUsername');
    expect(httpField('PARAMETER timeout')).toBe('timeout');
    expect(httpField('PARAMETER retries')).toBe('retries');
    expect(httpField('httpProxy.unknown')).toBeUndefined();
    const renamed = renameFieldErrors(
      new ApiError(400, 'bad', { fieldErrors: [{ id: 'httpProxy.port', message: 'bad port' }] }),
      httpField,
    ) as ApiError;
    expect(renamed.fieldErrors).toEqual([{ id: 'httpPort', message: 'bad port' }]);
  });

  it('accepts host patterns without spaces', () => {
    expect(hostPattern(' *.example.com ')).toBe('*.example.com');
    expect(hostPattern('a b')).toBeNull();
    expect(hostPattern('')).toBeNull();
  });
});

describe('email settings', () => {
  const stored = {
    enabled: true,
    host: 'smtp.example',
    port: 587,
    username: 'mailer',
    password: PLACEHOLDER,
    fromAddress: 'astrum@example.org',
    subjectPrefix: null,
    startTlsEnabled: true,
    startTlsRequired: false,
    sslOnConnectEnabled: false,
    sslServerIdentityCheckEnabled: true,
    astrumTrustStoreEnabled: false,
  };

  it('keeps the stored password and clears the credentials when authentication is off', () => {
    const values = fromEmail(stored);
    expect(values.useAuthentication).toBe(true);
    expect(values.password).toBe('');
    expect(toEmail(values, true)).toMatchObject({
      username: 'mailer',
      password: PLACEHOLDER,
      subjectPrefix: null,
    });
    expect(toEmail({ ...values, password: 'secret' }, true).password).toBe('secret');
    expect(toEmail({ ...values, useAuthentication: false }, true)).toMatchObject({
      username: '',
      password: null,
    });
    expect(fromEmail({ ...stored, username: '', password: null }).useAuthentication).toBe(false);
  });

  it('checks the host, port and sender', () => {
    const texts = { host: 'host', port: 'port', portRange: 'range', from: 'from', address: 'address' };
    expect(validateEmail({ ...fromEmail(stored), host: ' ', port: null, fromAddress: '' }, texts)).toEqual({
      host: 'host',
      port: 'port',
      fromAddress: 'from',
    });
    expect(validateEmail({ ...fromEmail(stored), port: 70000, fromAddress: 'nobody' }, texts)).toEqual({
      port: 'range',
      fromAddress: 'address',
    });
    expect(emailField('PARAMETER fromAddress')).toBe('fromAddress');
    expect(emailField('PARAMETER other')).toBeUndefined();
  });
});

describe('certificates', () => {
  const certificate = {
    id: 'AB:CD',
    fingerprint: 'AB:CD',
    pem: '',
    serialNumber: '1',
    subjectCommonName: 'repo.example',
    subjectOrganization: 'Example Org',
    subjectOrganizationalUnit: null,
    issuerCommonName: 'Example CA',
    issuerOrganization: 'Example Trust',
    issuerOrganizationalUnit: null,
    issuedOn: 0,
    expiresOn: 0,
  };

  it('tells valid, expiring and expired certificates apart', () => {
    const now = Date.UTC(2026, 9, 13);
    const day = 24 * 60 * 60 * 1000;
    expect(expiry({ expiresOn: now + 90 * day }, now)).toBe('valid');
    expect(expiry({ expiresOn: now + 10 * day }, now)).toBe('expiring');
    expect(expiry({ expiresOn: now - day }, now)).toBe('expired');
  });

  it('filters by name, organization and fingerprint', () => {
    expect(matchesCertificate(certificate, 'REPO')).toBe(true);
    expect(matchesCertificate(certificate, 'trust')).toBe(true);
    expect(matchesCertificate(certificate, 'ab:cd')).toBe(true);
    expect(matchesCertificate(certificate, 'other')).toBe(false);
  });

  it('reads servers from host names, host:port and URLs', () => {
    expect(parseServer('repo.example')).toEqual({ host: 'repo.example', port: 443 });
    expect(parseServer('repo.example:8443')).toEqual({ host: 'repo.example', port: 8443 });
    expect(parseServer('https://repo.example/maven2/')).toEqual({ host: 'repo.example', port: 443 });
    expect(parseServer('[::1]:9443')).toEqual({ host: '::1', port: 9443 });
    expect(parseServer('two words')).toBeNull();
    expect(remoteServer('https://repo.example:8443/x')).toEqual({ host: 'repo.example', port: 8443 });
    expect(remoteServer('http://repo.example/x')).toBeNull();
  });
});
