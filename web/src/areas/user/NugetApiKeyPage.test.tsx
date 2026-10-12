import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { userArea } from '.';
import NugetApiKeyPage from './NugetApiKeyPage';
import { keySnippets, sourceUrl } from './nugetKey';

const KEY = '3a5a1d8a-0b1c-4d2e-8f90-123456789abc';
const STATE = { ...ADMIN_STATE, nugetApiKeyRealm: true };

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the NuGet key snippets', () => {
  it('name the source, the user and the key or its placeholder', () => {
    expect(sourceUrl('https://repo.example/')).toBe(
      'https://repo.example/repository/{your-nuget-repo}/index.json',
    );
    const [dotnet, nuget, choco] = keySnippets('https://repo.example', 'jane', null);
    expect(dotnet?.code).toBe(
      'dotnet nuget add source "Astrum" \\\n' +
        '  --source https://repo.example/repository/{your-nuget-repo}/index.json \\\n' +
        '  --username jane \\\n' +
        '  --password {API_KEY}',
    );
    expect(nuget?.code).toBe(
      'nuget.exe setapikey {API_KEY} \\\n  -Source https://repo.example/repository/{your-nuget-repo}/index.json',
    );
    expect(choco?.code).toContain('--user="jane"');
    expect(keySnippets('https://repo.example', 'jane', KEY)[2]?.code).toContain(`--password="${KEY}"`);
  });
});

describe('the NuGet API key page', () => {
  it('reveals the key after the password and puts it into the instructions', async () => {
    const server = fakeServer({
      'POST /service/rest/internal/ui/user/nuget-api-key': ({ body }) =>
        (body as { password: string }).password === 'secret-1'
          ? jsonResponse({ apiKey: KEY, createdAt: '2026-10-12T10:00:00Z' })
          : jsonResponse([{ id: 'password', message: 'the password is wrong' }], 400),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<NugetApiKeyPage />, { state: STATE });
    expect(screen.getByRole('heading', { name: 'NuGet API Key', level: 1 })).toBeInTheDocument();
    expect(screen.getByText(/--password \{API_KEY\}/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: 'Chocolatey CLI' }));
    expect(screen.getByText(/choco source add --name="Astrum"/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Access API Key' }));
    const auth = await screen.findByRole('dialog', { name: 'Authenticate' });
    expect(within(auth).getByText('Please authenticate to access your NuGet API key.')).toBeInTheDocument();
    await userEvent.type(within(auth).getByLabelText('Password'), 'wrong');
    await userEvent.click(within(auth).getByRole('button', { name: 'Authenticate' }));
    expect(await screen.findByText('Authentication failed')).toBeInTheDocument();
    await userEvent.clear(within(auth).getByLabelText('Password'));
    await userEvent.type(within(auth).getByLabelText('Password'), 'secret-1');
    await userEvent.click(within(auth).getByRole('button', { name: 'Authenticate' }));

    const shown = await screen.findByRole('dialog', { name: 'NuGet API Key' });
    expect(within(shown).getByTestId('nuget-api-key')).toHaveTextContent(KEY);
    expect(
      within(shown).getByText(
        'Keep this key secret. Store it in your NuGet configuration, not in source code.',
        {
          selector: 'p',
        },
      ),
    ).toBeInTheDocument();
    expect(within(shown).getByText(new RegExp(`--password ${KEY}`))).toBeInTheDocument();
    await userEvent.click(within(shown).getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(screen.getByText(new RegExp(`--password="${KEY}"`))).toBeInTheDocument();
    expect(server.calls.filter((call) => call.method === 'POST')).toHaveLength(2);
  }, 20_000);

  it('resets the key after a confirmation and the password', async () => {
    const server = fakeServer({
      'POST /service/rest/internal/ui/user/nuget-api-key/reset': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<NugetApiKeyPage />, { state: STATE });
    await userEvent.click(screen.getByRole('button', { name: 'Reset API Key' }));
    const confirm = await screen.findByRole('alertdialog', { name: 'Reset NuGet API Key' });
    await userEvent.click(within(confirm).getByRole('button', { name: 'Reset Key' }));
    const auth = await screen.findByRole('dialog', { name: 'Authenticate' });
    expect(within(auth).getByText('Please authenticate to reset your NuGet API key.')).toBeInTheDocument();
    await userEvent.type(within(auth).getByLabelText('Password'), 'secret-1');
    await userEvent.click(within(auth).getByRole('button', { name: 'Authenticate' }));
    expect(await screen.findByText('API key reset. Generate a new one.')).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({ password: 'secret-1' });
  }, 20_000);

  it('is not offered while the NuGet API key realm is off', async () => {
    renderWithApp(<div />, {
      state: { ...ADMIN_STATE, nugetApiKeyRealm: false },
      path: '/user/nuget-api-key',
      areas: [userArea],
    });
    await waitFor(() => expect(screen.queryByText('NuGet API Key')).toBeNull());
  });

  it('speaks Russian', async () => {
    renderWithApp(<NugetApiKeyPage />, { state: STATE, language: 'ru' });
    expect(screen.getByRole('heading', { name: 'API-ключ NuGet', level: 1 })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Получить API-ключ' })).toBeInTheDocument();
    expect(screen.getByText('Как использовать')).toBeInTheDocument();
  });
});
