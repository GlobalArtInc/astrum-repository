import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

async function expectOk(response: Response, what: string) {
  expect(response.ok, `${what}: ${response.status} ${await response.text()}`).toBe(true);
}

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

test('a terraform repository takes its registry host from the form and shows module usage', async ({
  page,
  server,
}) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Terraform', exact: true }).click();
  await page.getByRole('button', { name: /hosted Terraform/ }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('modules');
  await expect(page.getByText('Terraform Settings', { exact: true })).toBeVisible();
  await expect(page.getByText('Provider Signing', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Registry Host', exact: true }).fill('https://bad/');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(
    page.getByText('Enter a host name, with a :port if needed, without https:// and a path'),
  ).toBeVisible();
  await page.getByRole('textbox', { name: 'Registry Host', exact: true }).fill('registry.example.org');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/terraform/hosted/modules')
  ).json();
  expect(stored.terraform.registryHost).toBe('registry.example.org');
  expect(stored.terraform.requireAuthentication).toBe(false);

  const module = gzipSync(tar([['vpc/main.tf', 'output "greeting" {\n  value = "hello"\n}\n']]));
  await expectOk(
    await call(server, 'PUT', '/repository/modules/v1/modules/acme/vpc/aws/1.2.0/vpc.tar.gz', module),
    'put module',
  );
  const discovery = await call(server, 'GET', '/repository/modules/v1/modules/acme/vpc/aws/versions');
  expect(await discovery.json()).toEqual({ modules: [{ versions: [{ version: '1.2.0' }] }] });

  const host = new URL(server.url).host;
  await page.goto('/browse/modules/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz');
  await expect(page.getByRole('tab', { name: 'main.tf' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`source  = "${host}/acme/vpc/aws"`, { exact: false })).toBeVisible();
  await expect(
    page.getByText(`Terraform looks the registry up at https://${host}/.well-known/terraform.json`, {
      exact: false,
    }),
  ).toBeVisible();
  await page.getByRole('tab', { name: '.terraformrc' }).click();
  await expect(page.getByText(`${server.url}/repository/modules/token`, { exact: false })).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(
      `curl -u <user> --upload-file vpc-aws-1.2.0.tar.gz ${server.url}/repository/modules/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz`,
    ),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/modules/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz');
  await expect(page.getByRole('tab', { name: 'main.tf' })).toBeVisible();
  await noHorizontalScroll(page);
});
