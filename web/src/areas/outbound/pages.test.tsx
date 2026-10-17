import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { type Certificate, type EmailSettings, type HttpSettings, PLACEHOLDER } from './api';
import EmailPage from './EmailPage';
import HttpPage from './HttpPage';
import AddCertificatePage from './ssl/AddCertificatePage';
import CertificatePage from './ssl/CertificatePage';
import CertificatesPage from './ssl/CertificatesPage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

afterEach(() => {
  vi.unstubAllGlobals();
});

const HTTP: HttpSettings = {
  userAgent: null,
  timeout: null,
  retries: null,
  httpProxy: {
    enabled: true,
    host: 'proxy.example',
    port: '3128',
    authInfo: {
      enabled: true,
      username: 'proxy-user',
      password: PLACEHOLDER,
      ntlmHost: null,
      ntlmDomain: null,
    },
  },
  httpsProxy: {
    enabled: false,
    host: null,
    port: null,
    authInfo: { enabled: false, username: null, password: null, ntlmHost: null, ntlmDomain: null },
  },
  nonProxyHosts: ['localhost'],
};

const EMAIL: EmailSettings = {
  enabled: true,
  host: 'smtp.example',
  port: 25,
  username: '',
  password: null,
  fromAddress: 'astrum@example.org',
  subjectPrefix: null,
  startTlsEnabled: false,
  startTlsRequired: false,
  sslOnConnectEnabled: false,
  sslServerIdentityCheckEnabled: false,
  astrumTrustStoreEnabled: false,
};

const FAR = Date.UTC(2030, 0, 1);

const CERTIFICATE: Certificate = {
  id: 'C8:3E:08:1F',
  fingerprint: 'C8:3E:08:1F',
  pem: '-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----',
  serialNumber: '1',
  subjectCommonName: 'repo.example',
  subjectOrganization: 'Example Org',
  subjectOrganizationalUnit: 'QA',
  issuerCommonName: 'Example CA',
  issuerOrganization: 'Example Trust',
  issuerOrganizationalUnit: null,
  issuedOn: Date.UTC(2025, 0, 1),
  expiresOn: FAR,
};

const OTHER: Certificate = {
  ...CERTIFICATE,
  id: 'AA:BB',
  fingerprint: 'AA:BB',
  subjectCommonName: 'mirror.example',
  subjectOrganization: 'Mirror Org',
  issuerOrganization: 'Mirror CA',
};

const ROUTES = [
  { path: '/admin/system/http', element: <HttpPage /> },
  { path: '/admin/system/email', element: <EmailPage /> },
  { path: '/admin/security/ssl-certificates', element: <CertificatesPage /> },
  { path: '/admin/security/ssl-certificates/add', element: <AddCertificatePage /> },
  { path: '/admin/security/ssl-certificates/:id', element: <CertificatePage /> },
];

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer(handlers);
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string, state = ADMIN_STATE) {
  return renderWithApp(null, { path, routes: ROUTES, state });
}

function textOf(body: unknown): Promise<string> {
  if (!(body instanceof Blob)) {
    return Promise.resolve(String(body));
  }
  return new Promise((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.readAsText(body);
  });
}

describe('the HTTP settings page', () => {
  it('shows the stored proxy with a masked password and keeps it on save', async () => {
    const server = serve({
      'GET /service/rest/v1/http': () => jsonResponse(HTTP),
      'PUT /service/rest/v1/http': ({ body }) => jsonResponse(body),
    });
    page('/admin/system/http');
    expect(await screen.findByRole('heading', { level: 1, name: 'HTTP Settings' })).toBeInTheDocument();
    expect(await screen.findByRole('checkbox', { name: /Enable HTTP Proxy/ })).toBeChecked();
    expect(screen.getByRole('textbox', { name: /HTTP Proxy Host/ })).toHaveValue('proxy.example');
    expect(screen.getByRole('textbox', { name: /^Username/ })).toHaveValue('proxy-user');
    expect(screen.getByLabelText(/^Password/)).toHaveValue('');
    expect(screen.getByText('Hosts to Exclude from Proxy (1)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();

    await userEvent.type(screen.getByRole('textbox', { name: /User-Agent Suffix/ }), 'Corp/1.0');
    await userEvent.type(screen.getByRole('textbox', { name: /Host Pattern/ }), '*.internal');
    await userEvent.click(screen.getByRole('button', { name: 'Add' }));
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    const sent = server.calls.find((call) => call.method === 'PUT')?.body as HttpSettings;
    expect(sent.userAgent).toBe('Corp/1.0');
    expect(sent.nonProxyHosts).toEqual(['localhost', '*.internal']);
    expect(sent.httpProxy?.authInfo?.password).toBe(PLACEHOLDER);
    expect(await screen.findByText('HTTP settings saved')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByText('Unsaved changes')).not.toBeInTheDocument());
  });

  it('checks the proxy fields before saving and shows server problems on their fields', async () => {
    const server = serve({
      'GET /service/rest/v1/http': () => jsonResponse({ ...HTTP, httpProxy: null, nonProxyHosts: [] }),
      'PUT /service/rest/v1/http': () =>
        jsonResponse([{ id: 'httpsProxy.host', message: 'Host is required' }], 400),
    });
    page('/admin/system/http');
    await userEvent.click(await screen.findByRole('checkbox', { name: /Enable HTTPS Proxy/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Enter the HTTPS proxy host')).toBeInTheDocument();
    expect(screen.getByText('Enter the HTTPS proxy port')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
    await userEvent.type(screen.getByRole('textbox', { name: /HTTPS Proxy Host/ }), 'proxy.example');
    await userEvent.type(screen.getByRole('textbox', { name: /HTTPS Proxy Port/ }), '8443');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Host is required')).toBeInTheDocument();
  });

  it('is read only without the permission to change settings', async () => {
    serve({ 'GET /service/rest/v1/http': () => jsonResponse(HTTP) });
    page('/admin/system/http', { ...ADMIN_STATE, permissions: ['astrum:settings:read'] });
    expect(await screen.findByText(/read-only version/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /HTTP Proxy Host/ })).toBeDisabled();
  });
});

describe('the email server page', () => {
  it('saves the settings and sends a test email with the reason of a failure', async () => {
    let verify = 0;
    const server = serve({
      'GET /service/rest/v1/email': () => jsonResponse(EMAIL),
      'PUT /service/rest/v1/email': () => new Response(null, { status: 204 }),
      'POST /service/rest/v1/email/verify': () => {
        verify += 1;
        return verify === 1
          ? jsonResponse({ success: true, reason: null })
          : jsonResponse({ success: false, reason: 'Connection refused' }, 400);
      },
    });
    page('/admin/system/email');
    expect(await screen.findByRole('heading', { level: 1, name: 'Email Server' })).toBeInTheDocument();
    await userEvent.click(await screen.findByRole('checkbox', { name: /Enable authentication/ }));
    await userEvent.type(screen.getByRole('textbox', { name: /^Username/ }), 'mailer');
    await userEvent.type(screen.getByLabelText(/^Password/), 'secret');
    await userEvent.click(screen.getByRole('button', { name: /SSL\/TLS Options/ }));
    await userEvent.click(screen.getByRole('checkbox', { name: /Enable STARTTLS/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
      username: 'mailer',
      password: 'secret',
      startTlsEnabled: true,
      host: 'smtp.example',
      port: 25,
    });
    expect(await screen.findByText('Email server settings saved')).toBeInTheDocument();

    const address = screen.getByRole('textbox', { name: /Test Email Address/ });
    await userEvent.type(address, 'test@example.invalid');
    await userEvent.click(screen.getByRole('button', { name: 'Send Test' }));
    expect(await screen.findByText('Test email sent. Check your inbox.')).toBeInTheDocument();
    const call = server.calls.find((entry) => entry.path === '/service/rest/v1/email/verify');
    expect(await textOf(call?.body)).toBe('test@example.invalid');
    await userEvent.click(screen.getByRole('button', { name: 'Send Test' }));
    expect(await screen.findByText('Failed to send the test email: Connection refused')).toBeInTheDocument();
  });

  it('checks the required fields and disables the test while the server is off', async () => {
    serve({ 'GET /service/rest/v1/email': () => jsonResponse({ ...EMAIL, enabled: false }) });
    page('/admin/system/email');
    expect(await screen.findByRole('textbox', { name: /Test Email Address/ })).toBeDisabled();
    await userEvent.clear(screen.getByRole('textbox', { name: /SMTP Host/ }));
    await userEvent.clear(screen.getByRole('textbox', { name: /From Address/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Enter the SMTP host')).toBeInTheDocument();
    expect(screen.getByText('Enter the sender address')).toBeInTheDocument();
  });

  it('shows the certificate of the mail server and adds it to the truststore', async () => {
    const server = serve({
      'GET /service/rest/v1/email': () => jsonResponse(EMAIL),
      'GET /service/rest/v1/security/ssl?host=smtp.example&port=25': () => jsonResponse(CERTIFICATE),
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([]),
      'POST /service/rest/v1/security/ssl/truststore': () => jsonResponse(CERTIFICATE, 201),
    });
    page('/admin/system/email');
    await userEvent.click(await screen.findByRole('button', { name: 'View Certificate' }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText('repo.example')).toBeInTheDocument();
    expect(within(dialog).getByText(/untrusted connection/)).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Certificate to Truststore' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    const post = server.calls.find((call) => call.method === 'POST');
    expect(await textOf(post?.body)).toBe(CERTIFICATE.pem);
  });
});

describe('the SSL certificate pages', () => {
  it('lists, filters and opens the certificates of the truststore', async () => {
    serve({ 'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([CERTIFICATE, OTHER]) });
    page('/admin/security/ssl-certificates');
    expect(await screen.findByRole('row', { name: 'Open certificate repo.example' })).toBeInTheDocument();
    expect(screen.getByText('What is SSL?')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Filter by name/ }), 'mirror');
    await waitFor(() =>
      expect(screen.queryByRole('row', { name: 'Open certificate repo.example' })).not.toBeInTheDocument(),
    );
    await userEvent.click(screen.getByRole('row', { name: 'Open certificate mirror.example' }));
    expect(
      await screen.findByRole('heading', { level: 1, name: 'Certificate mirror.example' }),
    ).toBeInTheDocument();
    expect(screen.getByText('Mirror CA')).toBeInTheDocument();
  });

  it('says when the truststore is empty', async () => {
    serve({ 'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([]) });
    page('/admin/security/ssl-certificates');
    expect(await screen.findByText('There are no SSL certificates')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /Add Certificate/ })).toHaveAttribute(
      'href',
      '/admin/security/ssl-certificates/add',
    );
  });

  it('adds a pasted certificate after showing its details', async () => {
    const server = serve({
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([]),
      'POST /service/rest/internal/ui/ssl/certificate-details': () =>
        jsonResponse({ ...CERTIFICATE, expiresOn: Date.now() + 5 * 24 * 60 * 60 * 1000 }),
      'POST /service/rest/v1/security/ssl/truststore': () => jsonResponse(CERTIFICATE, 201),
    });
    page('/admin/security/ssl-certificates/add');
    const load = await screen.findByRole('button', { name: 'Load Certificate' });
    expect(load).toBeDisabled();
    await userEvent.click(screen.getByRole('radio', { name: 'Paste PEM' }));
    expect(load).toBeEnabled();
    await userEvent.click(screen.getByRole('textbox', { name: /Certificate in PEM Format/ }));
    await userEvent.paste(CERTIFICATE.pem);
    await userEvent.click(screen.getByRole('button', { name: 'Load Certificate' }));
    expect(await screen.findByText('This certificate expires soon')).toBeInTheDocument();
    expect(screen.getByText('Example Org')).toBeInTheDocument();
    expect(screen.getByText(/means you trust the certificate authority/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Add Certificate' }));
    await waitFor(() =>
      expect(
        server.calls.some(
          (call) => call.path === '/service/rest/v1/security/ssl/truststore' && call.method === 'POST',
        ),
      ).toBe(true),
    );
    expect(
      await screen.findByText('SSL certificate "repo.example" added to the truststore'),
    ).toBeInTheDocument();
  });

  it('loads a certificate from a server and refuses one that is already trusted', async () => {
    serve({
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([CERTIFICATE]),
      'GET /service/rest/v1/security/ssl?host=repo.example&port=8443': () => jsonResponse(CERTIFICATE),
    });
    page('/admin/security/ssl-certificates/add');
    await userEvent.type(
      await screen.findByRole('textbox', { name: /Hostname or URL/ }),
      'https://repo.example:8443/x',
    );
    await userEvent.click(screen.getByRole('button', { name: 'Load Certificate' }));
    expect(await screen.findByText(/already in the truststore/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Add Certificate' })).toBeDisabled();
  });

  it('shows why a server certificate could not be retrieved', async () => {
    serve({
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([]),
      'GET /service/rest/v1/security/ssl?host=blocked.example&port=443': () =>
        jsonResponse({ id: '*', message: 'the SSRF protection blocks this host' }, 400),
    });
    page('/admin/security/ssl-certificates/add');
    await userEvent.type(await screen.findByRole('textbox', { name: /Hostname or URL/ }), 'blocked.example');
    await userEvent.click(screen.getByRole('button', { name: 'Load Certificate' }));
    expect(await screen.findByText(/the SSRF protection blocks this host/)).toBeInTheDocument();
  });

  it('removes a certificate after a confirmation', async () => {
    const server = serve({
      'GET /service/rest/v1/security/ssl/truststore': () => jsonResponse([CERTIFICATE]),
      'DELETE /service/rest/v1/security/ssl/truststore/C8%3A3E%3A08%3A1F': () =>
        new Response(null, { status: 204 }),
    });
    page(`/admin/security/ssl-certificates/${encodeURIComponent(CERTIFICATE.id)}`);
    expect(
      await screen.findByRole('heading', { level: 1, name: 'Certificate repo.example' }),
    ).toBeInTheDocument();
    expect(screen.getByText('C8:3E:08:1F')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Delete Certificate' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(/Delete the certificate "repo.example"/)).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete Certificate' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true));
    expect(
      await screen.findByText('SSL certificate "repo.example" removed from the truststore'),
    ).toBeInTheDocument();
  });
});
