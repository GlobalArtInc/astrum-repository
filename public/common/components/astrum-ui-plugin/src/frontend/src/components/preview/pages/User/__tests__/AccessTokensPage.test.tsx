import React from 'react';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { AccessTokensPage, serverUrl, usageExamples } from '../AccessTokensPage';

const mockCheckPermission = jest.fn(() => false);
const mockShowSuccessMessage = jest.fn();
const mockAbsolutePath = jest.fn((path: string) => `https://repo.example.com/${path}`);

jest.mock('../../../../../interface/ExtJS', () => ({
  ExtJS: {
    useUser: jest.fn(() => ({ id: 'alice' })),
    checkPermission: (...args: unknown[]) => mockCheckPermission(...args),
    usePermission: (getValue: () => unknown) => getValue(),
    showSuccessMessage: (...args: unknown[]) => mockShowSuccessMessage(...args),
    absolutePath: (path: string) => mockAbsolutePath(path),
  },
}));

const mockGet = jest.fn();
const mockPost = jest.fn();
const mockDelete = jest.fn();

jest.mock('../../../../../interface/api/rest-client', () => ({
  API_INTERNAL_UI: '/service/rest/internal/ui',
  restClient: {
    get: (...args: unknown[]) => mockGet(...args),
    post: (...args: unknown[]) => mockPost(...args),
    delete: (...args: unknown[]) => mockDelete(...args),
  },
}));

Object.defineProperty(navigator, 'clipboard', {
  value: { writeText: jest.fn().mockResolvedValue(undefined) },
  configurable: true,
});

const SECRET = `astrum_${'a'.repeat(40)}`;

const token = {
  id: 't1',
  name: 'ci',
  owner: 'alice',
  scope: 'read',
  hint: 'astrum_aaaa…',
  created: '2026-10-11T10:00:00Z',
  expires: '2027-01-09T10:00:00Z',
  lastUsed: null,
  expired: false,
};

function renderPage() {
  return render(
    <Theme>
      <AccessTokensPage />
    </Theme>
  );
}

describe('AccessTokensPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockCheckPermission.mockReturnValue(false);
  });

  it('shows an empty list', async () => {
    mockGet.mockResolvedValue([]);
    renderPage();
    expect(await screen.findByTestId('access-token-empty')).toHaveTextContent('You have no access tokens yet.');
    expect(mockGet).toHaveBeenCalledWith('/service/rest/internal/ui/access-tokens', { params: undefined });
    expect(screen.queryByTestId('access-token-view')).not.toBeInTheDocument();
  });

  it('lists tokens without their secrets', async () => {
    mockGet.mockResolvedValue([token, { ...token, id: 't2', name: 'old', scope: 'write', expired: true }]);
    renderPage();
    const row = await screen.findByTestId('access-token-row-ci');
    expect(within(row).getByText('Read only')).toBeInTheDocument();
    expect(within(row).getByText('astrum_aaaa…')).toBeInTheDocument();
    expect(within(screen.getByTestId('access-token-row-old')).getByText('Expired')).toBeInTheDocument();
    expect(within(screen.getByTestId('access-token-row-old')).getByText('Read and write')).toBeInTheDocument();
  });

  it('asks for a name before creating a token', async () => {
    mockGet.mockResolvedValue([]);
    renderPage();
    await screen.findByTestId('access-token-empty');
    fireEvent.click(screen.getByTestId('access-token-create-button'));
    expect(await screen.findByTestId('access-token-create-error')).toHaveTextContent('Give the token a name');
    expect(mockPost).not.toHaveBeenCalled();
  });

  it('creates a token and shows it once with usage examples', async () => {
    mockGet.mockResolvedValueOnce([]).mockResolvedValue([token]);
    mockPost.mockResolvedValue({ token, secret: SECRET });
    renderPage();
    await screen.findByTestId('access-token-empty');

    fireEvent.change(screen.getByTestId('access-token-name'), { target: { value: ' ci ' } });
    fireEvent.click(screen.getByTestId('access-token-create-button'));

    expect(await screen.findByTestId('access-token-secret')).toHaveTextContent(SECRET);
    expect(mockPost).toHaveBeenCalledWith('/service/rest/internal/ui/access-tokens', {
      name: 'ci',
      scope: 'read',
      expiresInDays: 90,
    });
    expect(screen.getByTestId('access-token-usage-docker')).toHaveTextContent(
      `docker login repo.example.com -u alice --password-stdin`
    );
    fireEvent.click(screen.getByTestId('access-token-created-done'));
    await waitFor(() => expect(screen.queryByTestId('access-token-secret')).not.toBeInTheDocument());
    expect(await screen.findByTestId('access-token-row-ci')).toBeInTheDocument();
  });

  it('shows why the server refused to create a token', async () => {
    mockGet.mockResolvedValue([]);
    mockPost.mockRejectedValue({ response: { data: { id: '*', message: "You already have a token named 'ci'" } } });
    renderPage();
    await screen.findByTestId('access-token-empty');
    fireEvent.change(screen.getByTestId('access-token-name'), { target: { value: 'ci' } });
    fireEvent.click(screen.getByTestId('access-token-create-button'));
    expect(await screen.findByTestId('access-token-create-error')).toHaveTextContent(
      "You already have a token named 'ci'"
    );
  });

  it('revokes a token after confirmation', async () => {
    mockGet.mockResolvedValueOnce([token]).mockResolvedValue([]);
    mockDelete.mockResolvedValue(undefined);
    renderPage();
    fireEvent.click(await screen.findByTestId('access-token-revoke-ci'));
    fireEvent.click(await screen.findByRole('button', { name: 'Revoke' }));
    await waitFor(() => expect(mockDelete).toHaveBeenCalledWith('/service/rest/internal/ui/access-tokens/t1'));
    expect(await screen.findByTestId('access-token-empty')).toBeInTheDocument();
    expect(mockShowSuccessMessage).toHaveBeenCalledWith('Token "ci" revoked');
  });

  it('lets administrators see everyone\'s tokens', async () => {
    mockCheckPermission.mockReturnValue(true);
    mockGet.mockResolvedValue([{ ...token, owner: 'bob' }]);
    renderPage();
    expect(await screen.findByTestId('access-token-view')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('radio', { name: 'All users' }));
    await waitFor(() =>
      expect(mockGet).toHaveBeenLastCalledWith('/service/rest/internal/ui/access-tokens', { params: { all: true } })
    );
    expect(await screen.findByText('bob')).toBeInTheDocument();
  });
});

describe('usageExamples', () => {
  it('builds client configuration for the instance', () => {
    const examples = Object.fromEntries(
      usageExamples('https://repo.example.com/', 'a@b.c', SECRET).map((example) => [example.id, example.code])
    );
    expect(examples.docker).toBe(`echo '${SECRET}' | docker login repo.example.com -u a@b.c --password-stdin`);
    expect(examples.npm).toContain(`//repo.example.com/repository/<repository>/:_auth=${btoa(`a@b.c:${SECRET}`)}`);
    expect(examples.maven).toContain(`<password>${SECRET}</password>`);
    expect(examples.pip).toContain(`https://a%40b.c:${SECRET}@repo.example.com/repository/<repository>/simple/`);
    expect(examples.curl).toBe(`curl -u a@b.c:${SECRET} -O https://repo.example.com/repository/<repository>/<path>`);
  });
});

describe('serverUrl', () => {
  it('is the absolute address of the instance', () => {
    expect(serverUrl()).toBe('https://repo.example.com/');
  });

  it('falls back to the page origin when only a path is known', () => {
    mockAbsolutePath.mockReturnValueOnce('/');
    expect(serverUrl()).toBe(`${window.location.origin}/`);
  });
});
