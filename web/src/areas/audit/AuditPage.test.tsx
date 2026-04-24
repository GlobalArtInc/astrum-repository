import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, jsonResponse, renderWithApp } from '../../test/render';
import AuditPage from './AuditPage';

vi.setConfig({ testTimeout: 20_000 });

function event(id: number, extra: object = {}) {
  return {
    id,
    timestamp: '2026-10-12T10:00:00.000+00:00',
    domain: 'repository',
    type: 'created',
    context: `repo-${id}`,
    initiator: 'admin/127.0.0.1',
    attributes: { format: 'raw', type: 'hosted' },
    ...extra,
  };
}

function stub(total = 45) {
  const calls: URL[] = [];
  vi.stubGlobal('fetch', async (input: RequestInfo | URL) => {
    const url = new URL(String(input), 'http://localhost');
    calls.push(url);
    if (url.pathname === '/service/rest/v1/repositories') {
      return jsonResponse([
        { name: 'files', format: 'raw', type: 'hosted', url: '' },
        { name: 'releases', format: 'maven2', type: 'hosted', url: '' },
      ]);
    }
    if (url.pathname === '/service/rest/internal/ui/audit-log') {
      const page = Number(url.searchParams.get('page') ?? 1);
      const limit = Number(url.searchParams.get('limit') ?? 20);
      const items =
        url.searchParams.get('types') === 'login'
          ? [event(900, { domain: 'security.session', type: 'created', context: 'jane', attributes: {} })]
          : Array.from({ length: Math.min(limit, total - (page - 1) * limit) }, (_, index) =>
              event(1000 - (page - 1) * limit - index),
            );
      const count = url.searchParams.get('types') === 'login' ? 1 : total;
      return jsonResponse({
        items,
        pagination: {
          totalItems: count,
          totalPages: Math.ceil(count / limit),
          currentPage: page,
          itemsPerPage: limit,
        },
      });
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

const logCalls = (calls: URL[]) =>
  calls.filter((url) => url.pathname === '/service/rest/internal/ui/audit-log');

describe('the audit log', () => {
  it('shows events with categories, summaries and initiators and pages by number', async () => {
    const calls = stub();
    const { router } = page('/audit');
    expect(await screen.findByText('45 events')).toBeInTheDocument();
    const table = screen.getByRole('table');
    expect(within(table).getAllByText('Repository').length).toBeGreaterThan(0);
    expect(
      within(table).getByText('Repository “repo-1000” was created (initiator: admin/127.0.0.1)'),
    ).toBeInTheDocument();
    expect(within(table).getAllByText('admin/127.0.0.1')).toHaveLength(20);
    const first = logCalls(calls)[0];
    expect(first?.searchParams.get('page')).toBe('1');
    expect(first?.searchParams.get('limit')).toBe('20');
    const start = Date.parse(first?.searchParams.get('startDate') ?? '');
    const end = Date.parse(first?.searchParams.get('endDate') ?? '');
    expect(Math.round((end - start) / 86_400_000)).toBe(30);
    expect(screen.getByRole('navigation', { name: 'Pagination, page 1 of 3' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: '3' }));
    await waitFor(() => expect(router.state.location.search).toBe('?page=3'));
    expect(await screen.findByText(/repo-960/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Next page' })).toBeDisabled();
    await userEvent.click(screen.getAllByRole('button', { name: 'Show details' })[0] as HTMLElement);
    expect(screen.getByRole('heading', { name: 'Event Details' })).toBeInTheDocument();
    expect(screen.getByText(/"format": "raw"/)).toBeInTheDocument();
  });

  it('filters by category, type, repository and text', async () => {
    const calls = stub();
    const { router } = page('/audit?repositoryName=files');
    await screen.findByText('45 events');
    expect(logCalls(calls).at(-1)?.searchParams.get('repository')).toBe('files');
    const filters = screen.getByRole('complementary', { name: 'Audit log filters' });
    await userEvent.click(within(filters).getByRole('checkbox', { name: 'Security' }));
    await userEvent.click(within(filters).getByRole('checkbox', { name: 'Sign-in' }));
    await waitFor(() =>
      expect(router.state.location.search).toBe('?categories=security&types=login&repositoryName=files'),
    );
    expect(await screen.findByText('User “jane”: signed in')).toBeInTheDocument();
    expect(screen.getByText('1 event')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: 'Search by context' }), 'jan');
    await waitFor(() => expect(logCalls(calls).at(-1)?.searchParams.get('q')).toBe('jan'));
    await userEvent.click(within(filters).getByRole('button', { name: 'Reset Filters' }));
    await waitFor(() => expect(router.state.location.search).toBe(''));
  });
});
