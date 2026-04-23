import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { createServer as createHttpServer, type IncomingHttpHeaders } from 'node:http';
import { createServer as createHttpsServer } from 'node:https';
import { createServer as createTcpServer, type Server as TcpServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, ensure } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const SHOTS = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;
const PLACEHOLDER = '#~ASTRUM~PLACEHOLDER~PASSWORD~#';

interface Listening {
  port: number;
  close(): Promise<void>;
}

function listen(server: TcpServer | ReturnType<typeof createHttpServer>): Promise<number> {
  return new Promise((done) => {
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      done(typeof address === 'object' && address ? address.port : 0);
    });
  });
}

function closing(server: TcpServer | ReturnType<typeof createHttpServer>): () => Promise<void> {
  return () =>
    new Promise((done) => {
      if ('closeAllConnections' in server) {
        server.closeAllConnections();
      }
      server.close(() => done());
    });
}

async function startProxy(): Promise<
  Listening & { seen: Array<{ url: string; headers: IncomingHttpHeaders }> }
> {
  const seen: Array<{ url: string; headers: IncomingHttpHeaders }> = [];
  const server = createHttpServer((request, response) => {
    seen.push({ url: request.url ?? '', headers: request.headers });
    response.writeHead(200, { 'Content-Type': 'text/plain' });
    response.end(`via proxy ${request.url}`);
  });
  const port = await listen(server);
  return { port, seen, close: closing(server) };
}

interface Mail {
  commands: string[];
  data: string;
}

async function startSmtpSink(): Promise<Listening & { mails: Mail[] }> {
  const mails: Mail[] = [];
  const server = createTcpServer((socket) => {
    let buffer = '';
    let inData = false;
    const mail: Mail = { commands: [], data: '' };
    socket.write('220 outbound-e2e sink ready\r\n');
    socket.on('data', (chunk) => {
      buffer += chunk.toString('utf8');
      let end = buffer.indexOf('\r\n');
      while (end >= 0) {
        const line = buffer.slice(0, end);
        buffer = buffer.slice(end + 2);
        if (inData) {
          if (line === '.') {
            inData = false;
            mails.push({ commands: [...mail.commands], data: mail.data });
            socket.write('250 queued\r\n');
          } else {
            mail.data += `${line}\r\n`;
          }
        } else {
          mail.commands.push(line);
          const upper = line.toUpperCase();
          if (upper.startsWith('EHLO')) {
            socket.write('250-outbound-e2e\r\n250 AUTH PLAIN LOGIN\r\n');
          } else if (upper.startsWith('AUTH')) {
            socket.write('235 accepted\r\n');
          } else if (upper === 'DATA') {
            inData = true;
            socket.write('354 go ahead\r\n');
          } else if (upper === 'QUIT') {
            socket.end('221 bye\r\n');
          } else {
            socket.write('250 ok\r\n');
          }
        }
        end = buffer.indexOf('\r\n');
      }
    });
    socket.on('error', () => {});
  });
  const port = await listen(server);
  return { port, mails, close: closing(server) };
}

function makeCertificate(): { dir: string; cert: string; key: string } {
  const dir = mkdtempSync(join(tmpdir(), 'astrum-e2e-tls-'));
  execFileSync(
    'openssl',
    [
      'req',
      '-x509',
      '-newkey',
      'rsa:2048',
      '-nodes',
      '-days',
      '30',
      '-keyout',
      join(dir, 'key.pem'),
      '-out',
      join(dir, 'cert.pem'),
      '-subj',
      '/CN=localhost/O=Outbound E2E/OU=Tests',
      '-addext',
      'subjectAltName=DNS:localhost',
    ],
    { stdio: 'pipe' },
  );
  return {
    dir,
    cert: readFileSync(join(dir, 'cert.pem'), 'utf8'),
    key: readFileSync(join(dir, 'key.pem'), 'utf8'),
  };
}

async function startTls(cert: string, key: string): Promise<Listening> {
  const server = createHttpsServer({ cert, key }, (_request, response) => {
    response.writeHead(200, { 'Content-Type': 'text/plain' });
    response.end('secure content');
  });
  const port = await listen(server);
  return { port, close: closing(server) };
}

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function allowLoopback(server: Server) {
  await ensure(
    await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    }),
    'allow loopback targets',
  );
}

async function rawProxy(server: Server, name: string, remoteUrl: string, useTrustStore = false) {
  const existing = await call(server, 'GET', `/service/rest/v1/repositories/${name}`);
  if (existing.ok) {
    return;
  }
  await ensure(
    await call(server, 'POST', '/service/rest/v1/repositories/raw/proxy', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false },
      proxy: { remoteUrl, contentMaxAge: 1440, metadataMaxAge: 1440 },
      negativeCache: { enabled: false, timeToLive: 1440 },
      httpClient: { blocked: false, autoBlock: false, connection: { retries: 0, useTrustStore } },
      raw: { contentDisposition: 'ATTACHMENT' },
    }),
    `create ${name}`,
  );
}

async function json<T>(server: Server, path: string): Promise<T> {
  const response = await call(server, 'GET', path);
  await ensure(response, path);
  return (await response.json()) as T;
}

test('HTTP settings send proxy repositories through the configured proxy', async ({ page, server }) => {
  const proxy = await startProxy();
  try {
    await rawProxy(server, 'ob-remote', 'http://upstream.invalid/');
    await signInAsAdmin(page, server.adminPassword);
    await page.goto('/settings');
    await page.getByRole('link', { name: /^HTTP/ }).click();
    await expect(page).toHaveURL(/\/admin\/system\/http$/);
    await expect(page.getByRole('heading', { level: 1, name: 'HTTP Settings' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();

    await page.getByRole('textbox', { name: 'User-Agent Suffix' }).fill('OutboundE2E/1');
    await page.getByRole('spinbutton', { name: 'Connection Timeout' }).fill('5000');
    await page.getByRole('spinbutton', { name: 'Connection Retries' }).fill('1');
    await page.getByRole('checkbox', { name: 'Enable HTTP Proxy' }).click();
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('The timeout must be between 1 and 3600 seconds')).toBeVisible();
    await expect(page.getByText('Enter the HTTP proxy host')).toBeVisible();
    await page.getByRole('spinbutton', { name: 'Connection Timeout' }).fill('20');
    await page.getByRole('textbox', { name: /^HTTP Proxy Host/ }).fill('127.0.0.1');
    await page.getByRole('textbox', { name: /^HTTP Proxy Port/ }).fill(String(proxy.port));
    await page.getByRole('button', { name: 'HTTP Proxy Authentication' }).click();
    await page.getByRole('combobox', { name: 'Authentication' }).click();
    await page.getByRole('option', { name: 'Username' }).click();
    await page.getByRole('textbox', { name: /^Username/ }).fill('proxy-user');
    await page.getByLabel('Password', { exact: true }).fill('proxy-secret');
    await page.getByRole('textbox', { name: 'Host Pattern' }).fill('*.direct.invalid');
    await page.getByRole('button', { name: 'Add', exact: true }).click();
    await expect(page.getByRole('list', { name: 'Hosts to Exclude from Proxy' })).toContainText(
      '*.direct.invalid',
    );
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('HTTP settings saved').first()).toBeVisible();

    const saved = await json<{
      userAgent: string;
      timeout: number;
      retries: number;
      httpProxy: {
        enabled: boolean;
        host: string;
        port: string;
        authInfo: { username: string; password: string };
      };
      nonProxyHosts: string[];
    }>(server, '/service/rest/v1/http');
    expect(saved).toMatchObject({
      userAgent: 'OutboundE2E/1',
      timeout: 20,
      retries: 1,
      httpProxy: {
        enabled: true,
        host: '127.0.0.1',
        port: String(proxy.port),
        authInfo: { username: 'proxy-user', password: PLACEHOLDER },
      },
      nonProxyHosts: ['*.direct.invalid'],
    });

    const fetched = await page.request.get(`${server.url}/repository/ob-remote/docs/readme.txt`);
    expect(fetched.status()).toBe(200);
    expect(await fetched.text()).toBe('via proxy http://upstream.invalid/docs/readme.txt');
    const head = proxy.seen.at(-1)?.headers ?? {};
    expect(head['proxy-authorization']).toBe(
      `Basic ${Buffer.from('proxy-user:proxy-secret').toString('base64')}`,
    );
    expect(head['user-agent']).toMatch(/^Astrum\/\S+ OutboundE2E\/1$/);

    await page.reload();
    await expect(page.getByRole('textbox', { name: 'User-Agent Suffix' })).toHaveValue('OutboundE2E/1');
    await expect(page.getByRole('textbox', { name: /^HTTP Proxy Port/ })).toHaveValue(String(proxy.port));
    await page.getByRole('checkbox', { name: 'Enable HTTP Proxy' }).click();
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('HTTP settings saved').first()).toBeVisible();
    const cleared = await json<{ httpProxy: { enabled: boolean }; nonProxyHosts: string[] }>(
      server,
      '/service/rest/v1/http',
    );
    expect(cleared.httpProxy.enabled).toBe(false);
    expect(cleared.nonProxyHosts).toEqual([]);
  } finally {
    await proxy.close();
  }
});

test('SSL certificates are loaded, trusted by proxies, viewed and removed', async ({ page, server }) => {
  const certificate = makeCertificate();
  const tls = await startTls(certificate.cert, certificate.key);
  try {
    await allowLoopback(server);
    await rawProxy(server, 'ob-secure', `https://localhost:${tls.port}/`, true);
    const untrusted = await call(server, 'GET', '/repository/ob-secure/a.txt');
    expect(untrusted.status).toBe(502);

    await signInAsAdmin(page, server.adminPassword);
    await page.goto('/settings');
    await page.getByRole('link', { name: /SSL Certificates/ }).click();
    await expect(page.getByRole('heading', { level: 1, name: 'SSL Certificates' })).toBeVisible();
    await expect(page.getByText('There are no SSL certificates')).toBeVisible();

    await page.getByRole('link', { name: 'Add Certificate' }).click();
    await expect(page.getByRole('heading', { level: 1, name: 'Add SSL Certificate' })).toBeVisible();
    await page
      .getByRole('textbox', { name: /^Hostname or URL/ })
      .fill(`https://localhost:${tls.port}/maven2/`);
    await page.getByRole('button', { name: 'Load Certificate' }).click();
    await expect(page.getByText('Outbound E2E').first()).toBeVisible();
    await expect(page.getByText(/adding a certificate to the truststore means you trust/)).toBeVisible();
    await page.getByRole('button', { name: 'Add Certificate' }).click();
    await expect(page.getByText('SSL certificate "localhost" added to the truststore').first()).toBeVisible();
    await expect(page.getByRole('heading', { level: 1, name: 'Certificate localhost' })).toBeVisible();

    const trusted = await call(server, 'GET', '/repository/ob-secure/a.txt');
    const body = await trusted.text();
    expect(trusted.status, body).toBe(200);
    expect(body).toBe('secure content');

    await page.goto('/admin/security/ssl-certificates');
    const row = page.getByRole('row', { name: 'Open certificate localhost' });
    await expect(row).toContainText('Outbound E2E');
    await page
      .getByRole('textbox', { name: 'Filter by name, organization or fingerprint' })
      .fill('nothing-like-this');
    await expect(page.getByText('No certificates match "nothing-like-this"')).toBeVisible();
    await page.getByRole('textbox', { name: 'Filter by name, organization or fingerprint' }).fill('');

    await page.goto('/admin/repository/repositories/ob-secure');
    await page.getByRole('button', { name: 'View Certificate' }).click();
    const dialog = page.getByRole('dialog', { name: 'Certificate Details' });
    await expect(dialog.getByText(`Certificate presented by localhost:${tls.port}`)).toBeVisible();
    await expect(dialog.getByText('Outbound E2E').first()).toBeVisible();
    await dialog.getByRole('button', { name: 'Remove Certificate from Truststore' }).click();
    await expect(
      page.getByText('SSL certificate "localhost" removed from the truststore').first(),
    ).toBeVisible();
    await expect(dialog.getByRole('button', { name: 'Add Certificate to Truststore' })).toBeVisible();
    await dialog.getByRole('button', { name: 'Add Certificate to Truststore' }).click();
    await expect(page.getByText('SSL certificate "localhost" added to the truststore').first()).toBeVisible();
    await dialog.getByRole('button', { name: 'Close' }).click();

    await page.goto('/admin/security/ssl-certificates');
    await page.getByRole('row', { name: 'Open certificate localhost' }).click();
    await page.getByRole('button', { name: 'Delete Certificate' }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Delete Certificate' }).click();
    await expect(page.getByText('There are no SSL certificates')).toBeVisible();

    await page.getByRole('link', { name: 'Add Certificate' }).click();
    await page.getByRole('radio', { name: 'Paste PEM' }).click();
    await page.getByRole('textbox', { name: /Certificate in PEM Format/ }).fill('not a certificate');
    await page.getByRole('button', { name: 'Load Certificate' }).click();
    await expect(page.getByText('This is not a PEM encoded certificate')).toBeVisible();
    await page.getByRole('textbox', { name: /Certificate in PEM Format/ }).fill(certificate.cert);
    await page.getByRole('button', { name: 'Load Certificate' }).click();
    await page.getByRole('button', { name: 'Add Certificate' }).click();
    await expect(page.getByRole('heading', { level: 1, name: 'Certificate localhost' })).toBeVisible();
    const listed = await json<Array<{ subjectOrganization: string }>>(
      server,
      '/service/rest/v1/security/ssl/truststore',
    );
    expect(listed.map((entry) => entry.subjectOrganization)).toEqual(['Outbound E2E']);
  } finally {
    await tls.close();
    rmSync(certificate.dir, { recursive: true, force: true });
  }
});

test('email settings are saved and a test email reaches the SMTP server', async ({ page, server }) => {
  const sink = await startSmtpSink();
  try {
    await signInAsAdmin(page, server.adminPassword);
    await page.goto('/settings');
    await page.getByRole('link', { name: /^Email/ }).click();
    await expect(page).toHaveURL(/\/admin\/system\/email$/);
    await expect(page.getByRole('heading', { level: 1, name: 'Email Server' })).toBeVisible();
    await expect(page.getByRole('textbox', { name: /^SMTP Host/ })).toHaveValue('localhost');
    await expect(page.getByRole('button', { name: 'Send Test' })).toBeDisabled();

    await page.getByRole('checkbox', { name: 'Enable email server' }).click();
    await page.getByRole('textbox', { name: /^SMTP Host/ }).fill('127.0.0.1');
    await page.getByRole('spinbutton', { name: /^SMTP Port/ }).fill(String(sink.port));
    await page.getByRole('checkbox', { name: 'Enable authentication' }).click();
    await page.getByRole('textbox', { name: /^Username/ }).fill('mailer');
    await page.getByLabel('Password', { exact: true }).fill('mail-secret');
    await page.getByRole('textbox', { name: /^From Address/ }).fill('repository@example.invalid');
    await page.getByRole('textbox', { name: /^Subject Prefix/ }).fill('[E2E]');
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('Email server settings saved').first()).toBeVisible();

    const saved = await json<Record<string, unknown>>(server, '/service/rest/v1/email');
    expect(saved).toMatchObject({
      enabled: true,
      host: '127.0.0.1',
      port: sink.port,
      username: 'mailer',
      password: PLACEHOLDER,
      fromAddress: 'repository@example.invalid',
      subjectPrefix: '[E2E]',
    });

    await page.getByRole('textbox', { name: 'Test Email Address' }).fill('admin@example.invalid');
    await page.getByRole('button', { name: 'Send Test' }).click();
    await expect(page.getByText('Test email sent. Check your inbox.')).toBeVisible();
    expect(sink.mails).toHaveLength(1);
    const [mail] = sink.mails;
    expect(mail?.commands).toContain('MAIL FROM:<repository@example.invalid>');
    expect(mail?.commands).toContain('RCPT TO:<admin@example.invalid>');
    expect(mail?.commands).toContain(`AUTH PLAIN ${Buffer.from('\0mailer\0mail-secret').toString('base64')}`);
    expect(mail?.data).toContain('Subject: [E2E] Email configuration test\r\n');

    await page.reload();
    await expect(page.getByRole('textbox', { name: /^Username/ })).toHaveValue('mailer');
    await page.getByRole('spinbutton', { name: /^SMTP Port/ }).fill('1');
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('Email server settings saved').first()).toBeVisible();
    await page.getByRole('textbox', { name: 'Test Email Address' }).fill('admin@example.invalid');
    await page.getByRole('button', { name: 'Send Test' }).click();
    await expect(page.getByText(/^Failed to send the test email: /)).toBeVisible();
    expect(sink.mails).toHaveLength(1);
  } finally {
    await sink.close();
  }
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`outbound screens at ${size.name} in ${theme}`, async ({ page, server }) => {
        const shoot = async (name: string) => {
          await noHorizontalScroll(page);
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(400);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        const certificate = makeCertificate();
        try {
          const listed = await json<unknown[]>(server, '/service/rest/v1/security/ssl/truststore');
          if (listed.length === 0) {
            const added = await call(
              server,
              'POST',
              '/service/rest/v1/security/ssl/truststore',
              certificate.cert,
              {
                'Content-Type': 'text/plain',
              },
            );
            await ensure(added, 'add a certificate');
          }
        } finally {
          rmSync(certificate.dir, { recursive: true, force: true });
        }
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await signInAsAdmin(page, server.adminPassword);
        await page.goto('/admin/system/http');
        await expect(page.getByRole('heading', { level: 1, name: 'HTTP Settings' })).toBeVisible();
        await shoot('outbound-http');
        await page.goto('/admin/system/email');
        await expect(page.getByRole('heading', { level: 1, name: 'Email Server' })).toBeVisible();
        await shoot('outbound-email');
        await page.goto('/admin/security/ssl-certificates');
        await expect(page.getByRole('row', { name: 'Open certificate localhost' })).toBeVisible();
        await shoot('outbound-ssl-list');
        await page.getByRole('row', { name: 'Open certificate localhost' }).click();
        await expect(page.getByRole('heading', { level: 1, name: 'Certificate localhost' })).toBeVisible();
        await shoot('outbound-ssl-certificate');
        await page.goto('/admin/security/ssl-certificates/add');
        await expect(page.getByRole('heading', { level: 1, name: 'Add SSL Certificate' })).toBeVisible();
        await shoot('outbound-ssl-add');
      });
    }
  }
});
