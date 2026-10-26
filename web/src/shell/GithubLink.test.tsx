import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../test/render';
import { GITHUB_STARS, GITHUB_URL, GithubLink } from './GithubLink';

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(stars: number | null | 'broken') {
  const server = fakeServer({
    [`GET ${GITHUB_STARS}`]: () =>
      stars === 'broken' ? jsonResponse({ id: '*', message: 'no' }, 500) : jsonResponse({ stars }),
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

describe('the GitHub link of the header', () => {
  it('points at the repository and shows how many stars it has', async () => {
    serve(1234);
    renderWithApp(<GithubLink />);
    const link = await screen.findByRole('link', { name: 'Astrum Repository on GitHub. Stars: 1,234' });
    expect(link).toHaveAttribute('href', GITHUB_URL);
    expect(link).toHaveAttribute('target', '_blank');
    expect(link.getAttribute('rel')).toContain('noopener');
    expect(link).toHaveTextContent('1,234');
  });

  it('leaves the number out when the server cannot learn it', async () => {
    serve(null);
    renderWithApp(<GithubLink />);
    const link = await screen.findByRole('link', { name: 'Astrum Repository on GitHub' });
    expect(link).toHaveAttribute('href', GITHUB_URL);
    expect(link).toHaveTextContent('');
  });

  it('keeps the link when the request fails', async () => {
    const server = serve('broken');
    renderWithApp(<GithubLink />);
    expect(await screen.findByRole('link', { name: 'Astrum Repository on GitHub' })).toHaveTextContent('');
    expect(server.calls.filter((call) => call.path === GITHUB_STARS)).toHaveLength(1);
  });
});
