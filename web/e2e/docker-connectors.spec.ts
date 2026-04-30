import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { AdminApi } from './admin-api';
import { expect, signIn, test } from './fixtures';

const PORT = 18497;
const DIR = process.env.ASTRUM_SHOTS_DIR;

test('the Docker form suggests free connector ports and the server listens on the saved one', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server.url, server.adminPassword);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();

  await page.goto('/admin/repository/repositories/create/docker/hosted');
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('ported');
  await page.getByRole('combobox', { name: 'Routing Mode' }).click();
  await page.getByRole('option', { name: 'Use connectors (ports and/or subdomain)' }).click();
  const http = page.getByRole('spinbutton', { name: 'HTTP Connector' });
  const https = page.getByRole('spinbutton', { name: 'HTTPS Connector' });
  const suggest = page.getByRole('button', { name: 'Suggest available port' });
  await expect(suggest).toHaveCount(2);
  await suggest.first().click();
  await expect(http).toHaveValue('8082');
  await suggest.last().click();
  await expect(https).toHaveValue('8083');
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.getByRole('region', { name: 'Docker Registry API Support' }).scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, 'docker-ports-1440-dark.png') });
  }

  await http.fill(String(PORT));
  await https.fill('');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText('Repository "ported" created', { exact: true })).toBeVisible();
  const document = await admin.repository('docker', 'hosted', 'ported');
  expect(document.docker).toMatchObject({ httpPort: PORT });

  await expect
    .poll(async () => {
      try {
        return (await request.get(`http://127.0.0.1:${PORT}/v2/`)).status();
      } catch {
        return 0;
      }
    })
    .toBe(401);
  const catalog = await request.get(`http://127.0.0.1:${PORT}/v2/_catalog`, {
    headers: { Authorization: `Basic ${Buffer.from(`admin:${server.adminPassword}`).toString('base64')}` },
  });
  expect(await catalog.json()).toEqual({ repositories: [] });

  await admin.send('DELETE', '/v1/repositories/ported');
  await expect
    .poll(async () => {
      try {
        return (await request.get(`http://127.0.0.1:${PORT}/v2/`)).status();
      } catch {
        return 0;
      }
    })
    .toBe(0);
});
