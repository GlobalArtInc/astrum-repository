import { createHmac } from 'node:crypto';
import type { CDPSession, Page } from '@playwright/test';
import { expect, signIn, test } from './fixtures';
import { AdminApi, signInAs, signOut } from './security-support';

const PASSWORD = 'passkey-e2e-password';
const localBase = new Map<string, Promise<void>>();

function base32Key(secret: string): Buffer {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  const bits = [...secret.replace(/[\s=]/g, '').toUpperCase()]
    .map((char) => alphabet.indexOf(char).toString(2).padStart(5, '0'))
    .join('');
  const bytes: number[] = [];
  for (let index = 0; index + 8 <= bits.length; index += 8) {
    bytes.push(Number.parseInt(bits.slice(index, index + 8), 2));
  }
  return Buffer.from(bytes);
}

function totp(secret: string, ahead = 0): string {
  const counter = Buffer.alloc(8);
  counter.writeBigUInt64BE(BigInt(Math.floor(Date.now() / 1000 / 30) + ahead));
  const digest = createHmac('sha1', base32Key(secret)).update(counter).digest();
  const offset = (digest[digest.length - 1] ?? 0) & 0x0f;
  return ((digest.readUInt32BE(offset) & 0x7fffffff) % 1_000_000).toString().padStart(6, '0');
}

async function useLocalhost(admin: AdminApi, url: string): Promise<string> {
  const port = new URL(url).port;
  const base = `http://localhost:${port}`;
  if (!localBase.has(url)) {
    localBase.set(
      url,
      admin
        .send('POST', '/service/rest/v1/capabilities', {
          type: 'baseurl',
          enabled: true,
          notes: '',
          properties: { url: base },
        })
        .then(() => undefined),
    );
  }
  await localBase.get(url);
  return base;
}

async function virtualAuthenticator(page: Page): Promise<CDPSession> {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', {
    options: {
      protocol: 'ctap2',
      transport: 'internal',
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
    },
  });
  return cdp;
}

test('a passkey is added, used at sign-in, renamed and removed', async ({ page, request, server }) => {
  const local = await useLocalhost(new AdminApi(request, server), server.url);
  await virtualAuthenticator(page);

  await page.goto(`${local}/login`);
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();

  await page.goto(`${local}/user/security`);
  await page.getByRole('button', { name: 'Add a Passkey' }).click();
  const add = page.getByRole('dialog', { name: 'Add a Passkey' });
  await add.getByLabel('Passkey Name').fill('Laptop');
  await add.getByLabel('Current Password').fill(server.adminPassword);
  await add.getByRole('button', { name: 'Continue' }).click();
  const codes = page.getByRole('dialog', { name: 'Recovery Codes' });
  await expect(codes.getByText('These codes are shown only once')).toBeVisible();
  await codes.getByRole('checkbox', { name: 'I saved these codes' }).click();
  await codes.getByRole('button', { name: 'Done' }).click();
  await expect(page.getByText('Laptop', { exact: true })).toBeVisible();

  await signOut(page);
  await page.goto(`${local}/login`);
  await page.getByLabel('User name').fill('admin');
  await page.getByLabel('Password', { exact: true }).fill(server.adminPassword);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Two-factor authentication' })).toBeVisible();
  await page.getByRole('button', { name: 'Use a passkey' }).click();
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();

  await page.goto(`${local}/user/security`);
  await page.getByRole('button', { name: 'Rename Laptop' }).click();
  const rename = page.getByRole('dialog', { name: 'Rename Passkey' });
  await rename.getByLabel('Passkey Name').fill('Work laptop');
  await rename.getByRole('button', { name: 'Rename' }).click();
  await expect(page.getByText('Work laptop', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Remove Work laptop' }).click();
  const remove = page.getByRole('dialog', { name: 'Remove Passkey' });
  await remove.getByLabel('Current Password').fill(server.adminPassword);
  await remove.getByRole('button', { name: 'Remove' }).click();
  await expect(page.getByText('No passkey is added yet.')).toBeVisible();
});

test('an authenticator app is set up and asked for at sign-in', async ({ page, request, server }) => {
  const admin = new AdminApi(request, server);
  await admin.user('totp-user', PASSWORD, []);

  await signInAs(page, 'totp-user', PASSWORD);
  await page.goto('/user/security');
  await page.getByRole('button', { name: 'Set Up' }).click();
  const setUp = page.getByRole('dialog', { name: 'Set Up Authenticator App' });
  await setUp.getByLabel('Current Password').fill(PASSWORD);
  await setUp.getByRole('button', { name: 'Continue' }).click();
  const secret = (await setUp.locator('code').first().innerText()).replace(/\s/g, '');
  await setUp.getByLabel('Code').fill(totp(secret));
  await setUp.getByRole('button', { name: 'Confirm' }).click();
  const codes = page.getByRole('dialog', { name: 'Recovery Codes' });
  await expect(codes.getByText('These codes are shown only once')).toBeVisible();
  await codes.getByRole('checkbox', { name: 'I saved these codes' }).click();
  await codes.getByRole('button', { name: 'Done' }).click();
  await expect(page.getByText('Set up', { exact: true })).toBeVisible();

  await signOut(page);
  await page.getByLabel('User name').fill('totp-user');
  await page.getByLabel('Password', { exact: true }).fill(PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('heading', { name: 'Two-factor authentication' })).toBeVisible();
  await page.getByLabel('Authentication code').fill(totp(secret, 1));
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
});
