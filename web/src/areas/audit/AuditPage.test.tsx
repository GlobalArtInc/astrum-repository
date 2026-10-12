import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, jsonResponse, renderWithApp } from '../../test/render';
import AuditPage from './AuditPage';
import { PAGE_SIZE } from './query';

vi.setConfig({ testTimeout: 20_000 });

function event(id: number, extra: object = {}) {
  return {
    id,
    at: '2026-10-12T10:00:00+00:00',
    domain: 'repository',
    type: 'created',
    context: `repo-${id}`,
    initiator: 'admin',
    attributes: { format: 'raw', type: 'hosted' },
    ...extra,
  };
}

function stub() {
  const calls: URL[] = [];
  vi.stubGlobal('fetch', async (input: RequestInfo | URL) => {
    const url = new URL(String(input), 'http://localhost');
    calls.push(url);
    if (url.pathname === '/service/rest/internal/ui/audit/facets') {
      return jsonResponse({
        domains: ['repository', 'security.user'],
        types: ['created'],
        initiators: ['admin', 'jane'],
      });
    }
    if (url.pathname === '/service/rest/v1/repositories') {
      return jsonResponse([{ name: 'files', format: 'raw', type: 'hosted', url: '' }]);
    }
    if (url.pathname === '/service/rest/internal/audit') {
      const before = Number(url.searchParams.get('before') ?? 1000);
      if (url.searchParams.get('initiator') === 'jane') {
        return jsonResponse([
          event(7, { domain: 'security.user', context: 'jane', initiator: 'jane', attributes: {} }),
        ]);
      }
      const count = before === 1000 ? PAGE_SIZE : 3;
      return jsonResponse(Array.from({ length: count }, (_, index) => event(before - 1 - index)));
    }
    return jsonResponse({ id: '*', message: 'missing' }, 404);
  });
  return calls;
}

function page(path: string) {
  return renderWithApp(null, {
    state: ADMIN_STATE,
    path,
    routes: [{ path: '/audit', element: <AuditPage /> }],
  });
}

describe('the audit page', () => {
  it('lists events with expandable attributes and loads older ones', async () => {
    const calls = stub();
    page('/audit');
    expect(
      await screen.findByText(`${PAGE_SIZE} events shown, older ones can be loaded`),
    ).toBeInTheDocument();
    expect(screen.getAllByText('2 attributes')[0]?.closest('details')).not.toHaveAttribute('open');
    await userEvent.click(screen.getAllByText('2 attributes')[0] as HTMLElement);
    expect(screen.getAllByText('hosted')[0]).toBeVisible();
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(await screen.findByText(`${PAGE_SIZE + 3} events`)).toBeInTheDocument();
    const second = calls.filter((url) => url.pathname === '/service/rest/internal/audit')[1];
    expect(second?.searchParams.get('before')).toBe(String(1000 - PAGE_SIZE));
  });

  it('keeps the filters in the address and sends them', async () => {
    const calls = stub();
    const { router } = page('/audit?repositoryName=files');
    expect(
      await screen.findByText(`${PAGE_SIZE} events shown, older ones can be loaded`),
    ).toBeInTheDocument();
    expect(
      calls.find((url) => url.pathname === '/service/rest/internal/audit')?.searchParams.get('repository'),
    ).toBe('files');
    await userEvent.click(screen.getByRole('combobox', { name: 'Initiator' }));
    await userEvent.click(await screen.findByRole('option', { name: 'jane' }));
    await waitFor(() => expect(router.state.location.search).toBe('?initiator=jane&repositoryName=files'));
    expect(await screen.findByText('1 event')).toBeInTheDocument();
    expect(screen.getByText('User')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText('Search'), 'jan');
    await waitFor(() =>
      expect(router.state.location.search).toBe('?q=jan&initiator=jane&repositoryName=files'),
    );
    await userEvent.click(screen.getByRole('button', { name: 'Reset filters' }));
    await waitFor(() => expect(router.state.location.search).toBe(''));
    expect(screen.getByLabelText('Search')).toHaveValue('');
  });

  it('offers known and seen domains with readable names', async () => {
    stub();
    page('/audit');
    await userEvent.click(await screen.findByRole('combobox', { name: 'Domain' }));
    expect(await screen.findByRole('option', { name: 'Blob store' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Content selector' })).toBeInTheDocument();
  });
});
