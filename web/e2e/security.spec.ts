import { expect, test } from './fixtures';
import { AdminApi, choose, signInAs, signOut, stranger, toast } from './security-support';

test.beforeEach(async ({ page, server }) => {
  await signInAs(page, 'admin', server.adminPassword);
});

test('a content selector is written with a live preview and kept while privileges use it', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server);
  await admin.rawRepository('e2e-sel-files');
  await admin.putFile('e2e-sel-files', 'docs/guide.txt', 'hello');
  await admin.putFile('e2e-sel-files', 'bin/tool.sh', 'echo hello');

  await page.goto('/settings');
  await page.getByRole('link', { name: /^Content selectors/i }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/selectors$/);
  await page.getByRole('link', { name: 'Create selector' }).click();
  await page.getByRole('textbox', { name: /^Name/ }).fill('e2e-docs');
  const expression = page.getByRole('textbox', { name: /^Expression/ });
  await expression.fill('path =^');
  await expect(page.getByText(/Not valid yet/)).toBeVisible();
  await page.getByRole('button', { name: /^Use the example format == "raw"$/ }).click();
  await expect(expression).toHaveValue('format == "raw"');
  await expect(page.getByRole('link', { name: '/bin/tool.sh' })).toBeVisible();
  await expression.fill('format == "raw" and path =^ "/docs/"');
  await expect(page.getByText('The expression is valid')).toBeVisible();
  await choose(page, 'Repository', 'e2e-sel-files');
  await expect(page.getByRole('link', { name: '/docs/guide.txt' })).toBeVisible();
  await expect(page.getByRole('link', { name: '/bin/tool.sh' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Create selector' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/selectors\/e2e-docs$/);
  const saved = await admin.get<{ expression: string }>(
    '/service/rest/v1/security/content-selectors/e2e-docs',
  );
  expect(saved.expression).toBe('format == "raw" and path =^ "/docs/"');

  await admin.send('POST', '/service/rest/v1/security/privileges/repository-content-selector', {
    name: 'e2e-docs-read',
    description: '',
    contentSelector: 'e2e-docs',
    format: 'raw',
    repository: '*',
    actions: ['READ'],
  });
  await page.reload();
  await page.getByRole('button', { name: 'Delete' }).click();
  const dialog = page.getByRole('alertdialog');
  await dialog.getByRole('button', { name: 'Delete' }).click();
  await expect(dialog.getByRole('alert')).toContainText('used by the privilege(s) e2e-docs-read');
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await admin.send('DELETE', '/service/rest/v1/security/privileges/e2e-docs-read');
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/selectors$/);
  await expect(page.getByRole('link', { name: 'e2e-docs' })).toHaveCount(0);
});

test('privileges of every type are created from forms built from their type', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server);
  await admin.rawRepository('e2e-priv-files');
  await admin.send('POST', '/service/rest/v1/security/content-selectors', {
    name: 'e2e-priv-sel',
    description: '',
    expression: 'format == "raw"',
  });
  const create = async (type: RegExp, name: string) => {
    await page.goto('/admin/security/privileges');
    await page.getByRole('link', { name: 'Create privilege' }).click();
    await page.getByRole('link', { name: type }).click();
    await page.getByRole('textbox', { name: /^Name/ }).fill(name);
  };
  const submit = async () => {
    await page.getByRole('button', { name: 'Create privilege' }).click();
    await expect(page).toHaveURL(/\/admin\/security\/privileges$/);
  };

  await create(/^Application/, 'e2e-app');
  await page.getByRole('textbox', { name: /^Domain/ }).fill('e2e');
  await page.getByRole('checkbox', { name: 'Read', exact: true }).click();
  await page.getByRole('checkbox', { name: 'Edit', exact: true }).click();
  await expect(page.getByText('astrum:e2e:read,update')).toBeVisible();
  await submit();

  await create(/^Wildcard/, 'e2e-wild');
  await page.getByRole('textbox', { name: /^Permission/ }).fill('astrum:search:read');
  await submit();

  await create(/^Repository view/, 'e2e-view');
  await choose(page, 'Format', 'Raw');
  await choose(page, 'Repository', 'e2e-priv-files');
  await page.getByRole('checkbox', { name: 'Browse', exact: true }).click();
  await page.getByRole('checkbox', { name: 'Read', exact: true }).click();
  await expect(page.getByText('astrum:repository-view:raw:e2e-priv-files:browse,read')).toBeVisible();
  await submit();

  await create(/^Repository administration/, 'e2e-admin');
  await page.getByRole('checkbox', { name: 'All', exact: true }).click();
  await submit();

  await create(/^Repository content selector/, 'e2e-csel');
  await choose(page, 'Content selector', 'e2e-priv-sel');
  await choose(page, 'Format', 'Raw');
  await page.getByRole('checkbox', { name: 'Read', exact: true }).click();
  await submit();

  const stored = async (name: string) =>
    admin.get<Record<string, unknown>>(`/service/rest/v1/security/privileges/${name}`);
  expect(await stored('e2e-app')).toMatchObject({
    type: 'application',
    domain: 'e2e',
    actions: ['READ', 'EDIT'],
  });
  expect(await stored('e2e-wild')).toMatchObject({ type: 'wildcard', pattern: 'astrum:search:read' });
  expect(await stored('e2e-view')).toMatchObject({
    type: 'repository-view',
    format: 'raw',
    repository: 'e2e-priv-files',
    actions: ['BROWSE', 'READ'],
  });
  expect(await stored('e2e-admin')).toMatchObject({ format: '*', repository: '*', actions: ['ALL'] });
  expect(await stored('e2e-csel')).toMatchObject({
    contentSelector: 'e2e-priv-sel',
    format: 'raw',
    actions: ['READ'],
  });

  await choose(page, 'Type', 'Wildcard');
  await page.getByRole('searchbox', { name: /Find by name/ }).fill('e2e');
  await expect(page.getByRole('link', { name: 'e2e-wild' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'e2e-app' })).toHaveCount(0);
  await page.getByRole('link', { name: 'e2e-wild' }).click();
  await page.getByRole('textbox', { name: /^Description/ }).fill('Search for everyone');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(toast(page, 'Privilege e2e-wild is saved')).toBeVisible();
  expect(await stored('e2e-wild')).toMatchObject({ description: 'Search for everyone' });
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page).toHaveURL(/\/admin\/security\/privileges$/);
  await admin.send('GET', '/service/rest/v1/security/privileges/e2e-wild', undefined, [404]);

  await page.goto('/admin/security/privileges/as-all');
  await expect(page.getByText(/This privilege is built in/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Save' })).toHaveCount(0);
});

test('a role holds privileges and a nested role', async ({ page, request, server }) => {
  const admin = new AdminApi(request, server);
  const pick = async (picker: string, value: string) => {
    await page.getByRole('combobox', { name: `Find ${picker}` }).fill(value);
    await page.getByRole('option', { name: new RegExp(`^${value}`) }).click();
    await page.getByRole('combobox', { name: `Find ${picker}` }).fill('');
  };

  await page.goto('/admin/security/roles');
  await page.getByRole('link', { name: 'Create role' }).click();
  await page.getByRole('textbox', { name: /^Role ID/ }).fill('e2e-readers');
  await page.getByRole('textbox', { name: /^Name/ }).fill('Readers');
  await pick('Privileges', 'as-search-read');
  await page.getByRole('button', { name: 'Create role' }).click();
  await expect(page).toHaveURL(/\/admin\/security\/roles\/e2e-readers$/);

  await page.goto('/admin/security/roles/create');
  await page.getByRole('textbox', { name: /^Role ID/ }).fill('e2e-devs');
  await page.getByRole('textbox', { name: /^Name/ }).fill('Developers');
  await page.getByRole('textbox', { name: /^Description/ }).fill('Uploads and everything readers do');
  await pick('Privileges', 'as-component-upload');
  await pick('Contained roles', 'e2e-readers');
  await expect(page.getByRole('list', { name: 'Chosen Contained roles' })).toContainText('e2e-readers');
  await page.getByRole('button', { name: 'Create role' }).click();
  await expect(page).toHaveURL(/\/admin\/security\/roles\/e2e-devs$/);
  const effective = page.getByRole('region', { name: 'Effective privileges' });
  await expect(effective.getByRole('row', { name: /as-search-read/ })).toContainText(
    'e2e-devs → e2e-readers',
  );
  await expect(effective.getByRole('row', { name: /as-component-upload/ })).toContainText('e2e-devs');

  const role = await admin.get<{ privileges: string[]; roles: string[] }>(
    '/service/rest/v1/security/roles/e2e-devs',
  );
  expect(role).toMatchObject({ privileges: ['as-component-upload'], roles: ['e2e-readers'] });

  await page.goto('/admin/security/roles/as-admin');
  await expect(page.getByText(/This role is built in/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Save' })).toHaveCount(0);
});

test('a user gets roles and a new password and signs in with it', async ({ page, request, server }) => {
  const admin = new AdminApi(request, server);
  await admin.role('e2e-user-role', ['as-privileges-read']);
  await page.goto('/admin/security/users');
  await expect(page.getByRole('link', { name: 'admin' })).toBeVisible();
  await page.getByRole('link', { name: 'Create user' }).click();
  await page.getByRole('textbox', { name: /^User ID/ }).fill('e2e-jane');
  await page.getByRole('textbox', { name: /^First name/ }).fill('Jane');
  await page.getByRole('textbox', { name: /^Last name/ }).fill('Doe');
  await page.getByRole('textbox', { name: /^Email/ }).fill('jane@example.org');
  await page.getByLabel(/^Password/).fill('first-password');
  await page.getByLabel('Repeat the password').fill('first-password');
  await page.getByRole('combobox', { name: 'Find Roles' }).fill('e2e-user');
  await page.getByRole('option', { name: /^e2e-user-role/ }).click();
  await page.getByRole('button', { name: 'Create user' }).click();
  await expect(page).toHaveURL(/\/admin\/security\/users\/e2e-jane$/);
  await expect(
    page
      .getByRole('region', { name: 'Effective privileges' })
      .getByRole('row', { name: /as-privileges-read/ }),
  ).toBeVisible();

  await page.getByRole('button', { name: 'Change password' }).click();
  const dialog = page.getByRole('dialog', { name: 'Change the password of e2e-jane' });
  await dialog.getByLabel(/^New password/).fill('second-password');
  await dialog.getByLabel('Repeat the new password').fill('second-password');
  await dialog.getByRole('button', { name: 'Change password' }).click();
  await expect(toast(page, 'The password of e2e-jane is changed')).toBeVisible();

  await page.goto('/admin/security/users?q=e2e-ja');
  await expect(page.getByRole('link', { name: 'e2e-jane' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'admin', exact: true })).toHaveCount(0);

  await signOut(page);
  await signInAs(page, 'e2e-jane', 'second-password');
  await expect(page.getByRole('button', { name: 'Account menu of Jane Doe' })).toBeVisible();
});

test('anonymous access is seen from a private browser context', async ({ page, browser, server }) => {
  await page.goto('/admin/security/anonymous');
  const toggle = page.getByRole('checkbox', { name: 'Let anonymous users access the server' });
  await expect(toggle).not.toBeChecked();
  await toggle.click();
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(toast(page, 'Anonymous access is on')).toBeVisible();

  const outsider = await stranger(browser, server.url);
  await outsider.page.goto('/');
  await expect(outsider.page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await expect(outsider.page.getByRole('link', { name: 'Sign in' })).toBeVisible();

  await toggle.click();
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(toast(page, 'Anonymous access is off')).toBeVisible();
  await outsider.page.goto('/');
  await expect(outsider.page).toHaveURL(/\/login$/);
  await outsider.context.close();
});

test('realms are reordered with buttons and by dragging', async ({ page, request, server }) => {
  const admin = new AdminApi(request, server);
  try {
    await page.goto('/admin/security/realms');
    const active = page.getByRole('listbox', { name: 'Active realms' });
    const available = page.getByRole('listbox', { name: 'Available realms' });
    await available.getByRole('option', { name: 'Docker Bearer Token Realm' }).click();
    await page.getByRole('button', { name: 'Activate the selected realm' }).click();
    await active.getByRole('option', { name: /Docker Bearer Token Realm/ }).click();
    await page.getByRole('button', { name: 'Move the selected realm up' }).click();
    await expect(active.getByRole('option').nth(1)).toContainText('Docker Bearer Token Realm');
    await active
      .getByRole('option', { name: /Access Token Realm/ })
      .dragTo(active.getByRole('option', { name: /Local Authenticating Realm/ }));
    await expect(active.getByRole('option').first()).toContainText('Access Token Realm');
    await active.getByRole('option', { name: /Local Authenticating Realm/ }).dblclick();
    await page.getByRole('button', { name: 'Save' }).click();
    const warning = page.getByRole('alertdialog', { name: 'Deactivate local users?' });
    await warning.getByRole('button', { name: 'Cancel' }).click();
    await page.getByRole('button', { name: 'Discard Changes' }).click();
    await available.getByRole('option', { name: 'Docker Bearer Token Realm' }).dblclick();
    await active.getByRole('option', { name: /Docker Bearer Token Realm/ }).click();
    await page.getByRole('button', { name: 'Move the selected realm up' }).click();
    await page.getByRole('button', { name: 'Move the selected realm up' }).click();
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(toast(page, 'The realms are saved')).toBeVisible();
    expect(await admin.get<string[]>('/service/rest/v1/security/realms/active')).toEqual([
      'DockerToken',
      'AstrumAuthenticatingRealm',
      'AccessTokenRealm',
    ]);
  } finally {
    await admin.send('PUT', '/service/rest/v1/security/realms/active', [
      'AstrumAuthenticatingRealm',
      'AccessTokenRealm',
    ]);
  }
});

test('SSRF lists are validated before they are saved', async ({ page, request, server }) => {
  const admin = new AdminApi(request, server);
  try {
    await page.goto('/admin/security/ssrf-protection');
    const ips = page.getByRole('textbox', { name: 'Allowed addresses and networks' });
    await ips.fill('10.0.0');
    await ips.press('Enter');
    await expect(page.getByText('10.0.0 is not an IP address or a network such as 10.0.0.0/8')).toBeVisible();
    await ips.fill('10.0.0.0/8');
    await ips.press('Enter');
    await ips.fill('192.168.1.10');
    await ips.press('Enter');
    await page.getByRole('button', { name: 'Remove 192.168.1.10' }).click();
    const domains = page.getByRole('textbox', { name: 'Allowed domains' });
    await domains.fill('-bad.example');
    await domains.press('Enter');
    await expect(page.getByText(/-bad\.example is not a domain name/)).toBeVisible();
    await domains.fill('*.Corp.Example');
    await domains.press('Enter');
    await expect(page.getByRole('list', { name: 'Allowed domains' })).toContainText('*.corp.example');
    await page.getByRole('button', { name: 'Save' }).click();
    await expect(toast(page, 'The protection settings are saved')).toBeVisible();
    expect(await admin.get('/service/rest/v1/security/ssrf-protection')).toEqual({
      enabled: true,
      allowedIPs: ['10.0.0.0/8'],
      allowedDomains: ['*.corp.example'],
    });
  } finally {
    await admin.send('PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: [],
      allowedDomains: [],
    });
  }
});

test('a user without administration rights sees only what their roles allow', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server);
  await admin.role('e2e-auditors', ['as-privileges-read', 'as-selectors-read']);
  await admin.user('e2e-auditor', 'auditor-password', ['e2e-auditors']);
  await signOut(page);
  await signInAs(page, 'e2e-auditor', 'auditor-password');

  await page.goto('/settings');
  await expect(page.getByRole('link', { name: /^Privileges/ })).toBeVisible();
  await expect(page.getByRole('link', { name: /^Content selectors/i })).toBeVisible();
  for (const hidden of [/^Roles/, /^Users/, /^Anonymous access/i, /^Realms/, /^SSRF protection/i]) {
    await expect(page.getByRole('link', { name: hidden })).toHaveCount(0);
  }
  await page.goto('/admin/security/privileges');
  await expect(page.getByRole('link', { name: 'as-all', exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Create privilege' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Actions' })).toHaveCount(0);
  await page.getByRole('link', { name: 'as-all', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Back' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete' })).toHaveCount(0);
  await page.goto('/admin/repository/selectors');
  await expect(page.getByRole('heading', { level: 1, name: 'Content selectors' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Create selector' })).toHaveCount(0);
  for (const path of [
    '/admin/security/users',
    '/admin/security/roles',
    '/admin/security/ssrf-protection',
    '/onboarding',
  ]) {
    await page.goto(path);
    await expect(page.getByRole('heading', { level: 1, name: 'You cannot open this page' })).toBeVisible();
  }
});
