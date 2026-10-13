import type { Page } from '@playwright/test';
import { call, zip } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const BOUNDARY = 'SwiftPMBoundary0123456789';

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

function release(version: string): Buffer {
  const archive = zip([
    ['LinkedList/Package.swift', '// swift-tools-version:5.7\nimport PackageDescription\n'],
    ['LinkedList/Sources/LinkedList/List.swift', `let version = "${version}"\n`],
  ]);
  const metadata = JSON.stringify({ description: 'One linked list', author: { name: 'Mona' } });
  return Buffer.concat([
    Buffer.from(
      `--${BOUNDARY}\r\nContent-Disposition: form-data; name="source-archive"\r\nContent-Type: application/zip\r\n\r\n`,
    ),
    archive,
    Buffer.from(
      `\r\n--${BOUNDARY}\r\nContent-Disposition: form-data; name="metadata"\r\nContent-Type: application/json\r\n\r\n${metadata}\r\n--${BOUNDARY}--\r\n`,
    ),
  ]);
}

test('a swift release published by the client shows in browse with its usage', async ({ page, server }) => {
  const created = await call(server, 'POST', '/service/rest/v1/repositories/swift/hosted', {
    name: 'packages',
    online: true,
    storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
  });
  expect(created.status).toBe(201);
  const published = await call(
    server,
    'PUT',
    '/repository/packages/mona/LinkedList/1.1.1',
    release('1.1.1'),
    {
      Accept: 'application/vnd.swift.registry.v1+json',
      'Content-Type': `multipart/form-data;boundary="${BOUNDARY}"`,
    },
  );
  expect(published.status, await published.text()).toBe(201);
  const releases = await call(server, 'GET', '/repository/packages/mona/LinkedList', undefined, {
    Accept: 'application/vnd.swift.registry.v1+json',
  });
  expect(Object.keys((await releases.json()).releases)).toEqual(['1.1.1']);

  await signInAsAdmin(page, server);
  await page.goto('/browse/packages/mona/LinkedList/1.1.1.zip');
  const url = `${server.url}/repository/packages/`;
  await expect(page.getByRole('tab', { name: 'swift package-registry', exact: true })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByText(`swift package-registry set --allow-insecure-http ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'Package.swift' }).click();
  await expect(page.getByText('.package(id: "mona.LinkedList", from: "1.1.1")')).toBeVisible();
  await page.getByRole('tab', { name: 'swift package-registry publish' }).click();
  await expect(
    page.getByText(`swift package-registry publish mona.LinkedList 1.1.1 --url ${url} --allow-insecure-http`),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/packages/mona/LinkedList/1.1.1.zip');
  await expect(page.getByRole('tab', { name: 'Package.swift' })).toBeVisible();
  await noHorizontalScroll(page);
});
