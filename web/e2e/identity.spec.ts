import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Browser, Page } from '@playwright/test';
import { expect, noHorizontalScroll, test } from './fixtures';
import {
  LDAP_BASE,
  LDAP_SYSTEM_DN,
  LDAP_SYSTEM_PASSWORD,
  type LdapFixture,
  startLdap,
  startOidcProvider,
} from './identity-support';
import { AdminApi, basic, signInAs } from './security-support';

let ldap: LdapFixture;

test.beforeAll(async () => {
  ldap = await startLdap();
});

test.afterAll(async () => {
  await ldap?.stop();
});

function ldapServer(name: string) {
  return {
    name,
    protocol: 'LDAP',
    host: '127.0.0.1',
    port: ldap.port,
    searchBase: LDAP_BASE,
    authScheme: 'SIMPLE',
    authUsername: LDAP_SYSTEM_DN,
    authPassword: LDAP_SYSTEM_PASSWORD,
    connectionTimeoutSeconds: 30,
    connectionRetryDelaySeconds: 300,
    maxIncidentsCount: 3,
    userBaseDn: 'ou=people',
    userSubtree: false,
    userObjectClass: 'inetOrgPerson',
    userIdAttribute: 'uid',
    userRealNameAttribute: 'cn',
    userEmailAddressAttribute: 'mail',
    ldapGroupsAsRoles: true,
    groupType: 'DYNAMIC',
    userMemberOfAttribute: 'memberOf',
  };
}

async function newSession(browser: Browser, baseURL: string, user: string, password: string): Promise<Page> {
  const context = await browser.newContext({ baseURL });
  const page = await context.newPage();
  await signInAs(page, user, password);
  return page;
}

test('an LDAP server is created, verified and used to sign in with a mapped role', async ({
  page,
  request,
  server,
  browser,
}) => {
  test.setTimeout(120_000);
  await signInAs(page, 'admin', server.adminPassword);
  await page.goto('/admin/security/ldap');
  await expect(page.getByText('No LDAP servers configured')).toBeVisible();
  await page.getByRole('link', { name: 'Create LDAP Server' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Create LDAP Server' })).toBeVisible();

  const continueButton = page.getByRole('button', { name: 'Continue' });
  await expect(continueButton).toBeDisabled();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('corp');
  await page.getByLabel('Hostname').fill('127.0.0.1');
  await page.getByLabel('Port').fill(String(ldap.port));
  await page.getByLabel('Search Base DN').fill(LDAP_BASE);
  await page.getByLabel('Username').fill(LDAP_SYSTEM_DN);
  await page.getByRole('textbox', { name: 'Password', exact: true }).fill('wrong-secret');
  await page.getByRole('button', { name: 'Verify Connection' }).click();
  await expect(page.getByText('Connection verification failed')).toBeVisible();
  await expect(
    page.getByText('The LDAP server rejected the username or password of the system account'),
  ).toBeVisible();
  await page.getByRole('textbox', { name: 'Password', exact: true }).fill(LDAP_SYSTEM_PASSWORD);
  await page.getByRole('button', { name: 'Verify Connection' }).click();
  await expect(page.getByText('Connection successful')).toBeVisible();

  await continueButton.click();
  await expect(page.getByRole('heading', { name: 'User & Group' })).toBeVisible();
  await page.getByLabel('User relative DN').fill('ou=people');
  await page.getByRole('button', { name: 'Verify User Mapping' }).click();
  const found = page.getByRole('table', { name: 'Verified LDAP users' });
  await expect(found.getByRole('row', { name: /alice Alice Smith alice@example.com/ })).toBeVisible();
  await expect(found.getByRole('row', { name: /bob Bob Jones/ })).toContainText('developers');

  await page.getByRole('button', { name: 'Verify Login' }).click();
  const dialog = page.getByRole('dialog', { name: 'Test LDAP Login' });
  await dialog.getByLabel('Username').fill('alice');
  await dialog.getByLabel('Password').fill('not-her-password');
  await dialog.getByRole('button', { name: 'Test Login' }).click();
  await expect(dialog.getByText("The password for user 'alice' is incorrect")).toBeVisible();
  await dialog.getByLabel('Password').fill('alice-secret');
  await dialog.getByRole('button', { name: 'Test Login' }).click();
  await expect(
    dialog.getByText(`LDAP login completed successfully on: ldap://127.0.0.1:${ldap.port}`),
  ).toBeVisible();
  await dialog.getByRole('button', { name: 'Close' }).click();

  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await expect(page.getByText('LDAP server "corp" created successfully', { exact: true })).toBeVisible();
  await expect(page).toHaveURL(/\/admin\/security\/ldap$/);
  await expect(page.getByRole('link', { name: 'corp', exact: true })).toBeVisible();
  await expect(page.getByText(`ldap://127.0.0.1:${ldap.port}`)).toBeVisible();

  await page.goto('/admin/security/roles/create');
  await page.getByLabel('Type').click();
  await page.getByRole('option', { name: 'External Role Mapping' }).click();
  await expect(page.getByLabel('External Source')).toContainText('LDAP');
  const mapped = page.getByRole('combobox', { name: 'Mapped Role' });
  await mapped.click();
  const searched = page.waitForResponse((response) => response.url().includes('/ui/roles?source=LDAP'));
  await mapped.fill('develop');
  expect((await searched).status()).toBe(200);
  await page.getByRole('option', { name: 'developers' }).click();
  await expect(page.getByRole('textbox', { name: /^Role ID/ })).toHaveValue('developers');
  await page.getByRole('textbox', { name: /^Name/ }).fill('LDAP developers');
  const privileges = page.getByRole('combobox', { name: 'Find Privileges' });
  for (const privilege of ['as-users-read', 'as-roles-read']) {
    await privileges.fill(privilege);
    await page.getByRole('option', { name: new RegExp(`^${privilege}`) }).click();
    await privileges.fill('');
  }
  await page.getByRole('button', { name: 'Create role' }).click();
  await expect(page.getByText('Role developers is created', { exact: true })).toBeVisible();

  const bob = await newSession(browser, server.url, 'bob', 'bob-secret');
  await bob.goto('/admin/security/users');
  await expect(bob.getByRole('heading', { level: 1, name: 'Users' })).toBeVisible();
  const asBob = await request.get(`${server.url}/service/rest/v1/security/users`, {
    headers: { Authorization: basic('bob', 'bob-secret') },
  });
  expect(asBob.status()).toBe(200);
  const carol = await request.get(`${server.url}/service/rest/v1/security/users`, {
    headers: { Authorization: basic('carol', 'carol-secret') },
  });
  expect(carol.status()).toBe(403);
  await bob.context().close();

  await page.goto('/admin/security/users?source=LDAP');
  await expect(page.getByRole('row', { name: /alice/ })).toContainText('developers');

  const admin = new AdminApi(request, server);
  await admin.send('POST', '/service/rest/v1/security/ldap', ldapServer('backup'));
  await page.goto('/admin/security/ldap');
  await page.getByRole('button', { name: 'Change Order' }).click();
  const order = page.getByRole('dialog', { name: 'Change Server Order' });
  await order.getByRole('button', { name: 'Move backup up' }).click();
  await order.getByRole('button', { name: 'Save Order' }).click();
  await expect(page.getByText('Server order updated successfully', { exact: true })).toBeVisible();
  await expect(page.getByRole('row').nth(1)).toContainText('backup');
  const servers = await admin.get<Array<{ name: string; order: number }>>('/service/rest/v1/security/ldap');
  expect(servers.map((item) => item.name)).toEqual(['backup', 'corp']);

  await page.getByRole('button', { name: 'Clear Cache' }).click();
  await expect(page.getByText('LDAP cache cleared successfully', { exact: true })).toBeVisible();

  await page.getByRole('link', { name: 'corp', exact: true }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Edit corp' })).toBeVisible();
  await page.getByRole('button', { name: 'Verify Connection' }).click();
  const password = page.getByRole('dialog', { name: 'Enter LDAP Password' });
  await password.getByLabel('Password').fill(LDAP_SYSTEM_PASSWORD);
  await password.getByRole('button', { name: 'Verify' }).click();
  await expect(page.getByText('Connection successful')).toBeVisible();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Real Name Attribute').fill('sn');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('LDAP server "corp" updated successfully', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Delete backup' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete Server' }).click();
  await expect(page.getByText('LDAP server "backup" deleted successfully', { exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: 'backup', exact: true })).toHaveCount(0);
});

test('a user signs in through OpenID Connect in the browser and gets the mapped role', async ({
  page,
  request,
  server,
  browser,
}) => {
  test.setTimeout(120_000);
  const provider = await startOidcProvider();
  try {
    const admin = new AdminApi(request, server);
    await admin.send('PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    });
    await signInAs(page, 'admin', server.adminPassword);
    await page.goto('/admin/security/oauth2');
    await expect(page.getByText(`${server.url}/oidc/callback`)).toBeVisible();
    await page.getByLabel('Issuer URL').fill(provider.issuer);
    await page.getByLabel('Client ID').fill(provider.clientId);
    await page.getByRole('textbox', { name: 'Client Secret', exact: true }).fill(provider.clientSecret);
    await page.getByLabel('Groups Claim').fill('groups');
    await page.getByLabel('Exact Match Claims').fill('{ not json');
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(
      page.getByText('The field "Exact Match Claims" must contain a valid JSON object with string values'),
    ).toBeVisible();
    await page.getByLabel('Exact Match Claims').fill('{}');
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByText('OpenID Connect settings saved successfully', { exact: true })).toBeVisible();
    await page.reload();
    await expect(page.getByLabel('Client ID')).toHaveValue(provider.clientId);
    await expect(page.getByText('A client secret is stored.', { exact: false })).toBeVisible();

    await admin.send('POST', '/service/rest/v1/security/roles', {
      id: 'oidc-readers',
      name: 'OIDC readers',
      description: 'Members of the provider group',
      source: 'OAuth2',
      privileges: ['as-users-read', 'as-roles-read'],
      roles: [],
    });
    provider.signInAs({
      sub: 'subject-1',
      preferred_username: 'carla',
      given_name: 'Carla',
      family_name: 'Diaz',
      email: 'carla@example.org',
      groups: ['oidc-readers'],
    });

    const context = await browser.newContext({ baseURL: server.url });
    const visitor = await context.newPage();
    await visitor.goto('/admin/security/users');
    await expect(visitor).toHaveURL(/\/login/);
    await expect(visitor.getByText('or', { exact: true })).toBeVisible();
    await visitor.getByRole('button', { name: 'Continue with SSO' }).click();
    await expect(visitor).toHaveURL(/\/admin\/security\/users/);
    await expect(visitor.getByRole('heading', { level: 1, name: 'Users' })).toBeVisible();
    await expect(visitor.getByRole('button', { name: /Account menu of/ })).toBeVisible();
    expect(provider.tokenRequests()).toBe(1);
    await context.close();

    await page.goto('/admin/security/users?source=OAuth2');
    const carla = page.getByRole('row', { name: /carla/ });
    await expect(carla).toContainText('Carla Diaz');
    await expect(carla).toContainText('oidc-readers');

    const failed = await browser.newContext({ baseURL: server.url });
    const late = await failed.newPage();
    await late.goto('/oidc/callback?code=nothing&state=nothing');
    await expect(late).toHaveURL(/\/login\?sso=/);
    await expect(late.getByRole('alert')).toBeVisible();
    await failed.close();
  } finally {
    await provider.stop();
  }
});

test('the default role reaches every signed-in user until it is switched off', async ({
  page,
  request,
  server,
  browser,
}) => {
  const admin = new AdminApi(request, server);
  await admin.role('e2e-viewers', ['as-users-read', 'as-roles-read']);
  await admin.send('POST', '/service/rest/v1/security/users', {
    userId: 'plain',
    firstName: 'Plain',
    lastName: 'User',
    emailAddress: 'plain@example.org',
    password: 'plain-password',
    status: 'active',
    roles: [],
  });
  const asPlain = () =>
    request.get(`${server.url}/service/rest/v1/security/users`, {
      headers: { Authorization: basic('plain', 'plain-password') },
    });
  expect((await asPlain()).status()).toBe(403);

  await signInAs(page, 'admin', server.adminPassword);
  await page.goto('/admin/security/default-role');
  await expect(page.getByRole('heading', { level: 1, name: 'Default Role' })).toBeVisible();
  const saved = () =>
    page.waitForResponse(
      (response) =>
        response.url().endsWith('/internal/ui/default-role') && response.request().method() === 'PUT',
    );
  await page.getByLabel('Role').click();
  await page.getByRole('option', { name: /e2e-viewers/ }).click();
  const first = saved();
  await page.getByRole('button', { name: 'Save' }).click();
  expect((await first).status()).toBe(204);
  await expect(page.getByText('Default role saved', { exact: true })).toBeVisible();
  expect((await asPlain()).status()).toBe(200);

  const plain = await newSession(browser, server.url, 'plain', 'plain-password');
  await plain.goto('/admin/security/users');
  await expect(plain.getByRole('heading', { level: 1, name: 'Users' })).toBeVisible();
  await plain.context().close();

  const enabled = page.getByRole('checkbox', { name: 'Enable this capability' });
  await enabled.click();
  await expect(enabled).not.toBeChecked();
  const second = saved();
  await page.getByRole('button', { name: 'Save' }).click();
  expect((await second).status()).toBe(204);
  expect((await asPlain()).status()).toBe(403);
  await page.reload();
  await expect(page.getByRole('checkbox', { name: 'Enable this capability' })).not.toBeChecked();

  await page.getByRole('button', { name: 'Remove' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Remove' }).click();
  await expect(page.getByText('Default role removed', { exact: true })).toBeVisible();
  const stored = await admin.get<{ role: string | null }>('/service/rest/internal/ui/default-role');
  expect(stored.role).toBeNull();
});

const DIR = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;

async function shoot(page: Page, name: string) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(300);
  await noHorizontalScroll(page);
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

for (const size of SIZES) {
  for (const theme of ['dark', 'light'] as const) {
    test(`identity pages at ${size.name} in ${theme}`, async ({ page, request, server, browser }) => {
      test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
      test.setTimeout(120_000);
      const tag = `${size.name}-${theme}`;
      const admin = new AdminApi(request, server);
      const existing = await admin.get<Array<{ name: string }>>('/service/rest/v1/security/ldap');
      for (const name of ['shot-corp', 'shot-backup']) {
        if (!existing.some((item) => item.name === name)) {
          await admin.send('POST', '/service/rest/v1/security/ldap', ldapServer(name));
        }
      }
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await signInAs(page, 'admin', server.adminPassword);

      await page.goto('/admin/security/ldap');
      await expect(page.getByRole('link', { name: 'shot-corp', exact: true })).toBeVisible();
      await shoot(page, `ldap-${tag}`);
      await page.getByRole('button', { name: 'Change Order' }).click();
      await shoot(page, `ldap-order-${tag}`);
      await page.keyboard.press('Escape');
      await page.goto('/admin/security/ldap/create');
      await shoot(page, `ldap-create-${tag}`);
      await page.goto('/admin/security/ldap/shot-corp');
      await expect(page.getByRole('heading', { level: 1, name: 'Edit shot-corp' })).toBeVisible();
      await page.getByRole('button', { name: 'Continue' }).click();
      await page.getByRole('button', { name: 'Verify User Mapping' }).click();
      await page
        .getByRole('dialog', { name: 'Enter LDAP Password' })
        .getByLabel('Password')
        .fill(LDAP_SYSTEM_PASSWORD);
      await page.getByRole('button', { name: 'Verify', exact: true }).click();
      await expect(page.getByRole('table', { name: 'Verified LDAP users' })).toBeVisible();
      await shoot(page, `ldap-mapping-${tag}`);
      await page.getByRole('button', { name: 'Verify Login' }).click();
      await shoot(page, `ldap-login-${tag}`);
      await page.keyboard.press('Escape');

      await page.goto('/admin/security/oauth2');
      await shoot(page, `oidc-${tag}`);
      await page.goto('/admin/security/default-role');
      await shoot(page, `default-role-${tag}`);
      await page.goto('/admin/security/roles/create');
      await page.getByLabel('Type').click();
      await page.getByRole('option', { name: 'External Role Mapping' }).click();
      await shoot(page, `role-external-${tag}`);

      const context = await browser.newContext({
        baseURL: server.url,
        viewport: { width: size.width, height: size.height },
      });
      const visitor = await context.newPage();
      await visitor.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await visitor.route('**/service/rest/internal/ui/state', async (route) => {
        const response = await route.fetch();
        const state = await response.json();
        await route.fulfill({ response, json: { ...state, oidcEnabled: true } });
      });
      await visitor.goto('/login');
      await expect(visitor.getByRole('button', { name: 'Continue with SSO' })).toBeVisible();
      await shoot(visitor, `login-sso-${tag}`);
      await context.close();
    });
  }
}
