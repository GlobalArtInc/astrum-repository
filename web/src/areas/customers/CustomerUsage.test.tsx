import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { formatBytes } from '../../i18n';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import type { DayDownloads } from '../dashboard/types';
import { usageUrl } from './api';
import { CustomerUsage } from './CustomerUsage';

const chart = vi.hoisted(() => vi.fn());

interface ChartProps {
  days: DayDownloads[];
}

vi.mock('../dashboard/DownloadsChart', () => ({
  DownloadsChart: (props: ChartProps) => {
    chart(props);
    return null;
  },
}));

const DAY_MS = 24 * 60 * 60 * 1000;

function daysAgo(count: number): string {
  return new Date(Date.now() - count * DAY_MS).toISOString().slice(0, 10);
}

const REPORT = {
  customer: 'acme',
  days: 30,
  totals: { downloads: 1234, bytes: 2048 },
  daily: [
    { day: daysAgo(2), repository: 'releases', downloads: 1000, bytes: 1024 },
    { day: daysAgo(0), repository: 'releases', downloads: 200, bytes: 512 },
    { day: daysAgo(0), repository: 'charts', downloads: 34, bytes: 512 },
  ],
};

afterEach(() => {
  vi.unstubAllGlobals();
  chart.mockClear();
});

function renderUsage() {
  return renderWithApp(<CustomerUsage customerId="acme" />);
}

describe('the usage panel', () => {
  it('shows the totals and one row per repository, the biggest first', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({ [`GET ${usageUrl('acme')}?days=30`]: () => jsonResponse(REPORT) }).fetch,
    );
    renderUsage();
    expect(await screen.findByText('1,234')).toBeInTheDocument();
    expect(screen.getByText(formatBytes('en', 2048).replace(/\s/g, ' '))).toBeInTheDocument();
    const table = screen.getByRole('table', { name: 'Downloads per repository' });
    const rows = within(table).getAllByRole('row');
    expect(rows[1]).toHaveTextContent('releases');
    expect(rows[1]).toHaveTextContent('1,200');
    expect(rows[2]).toHaveTextContent('charts');
    expect(rows[2]).toHaveTextContent('34');
  });

  it('gives the chart one day per day of the period, zero where nothing was downloaded', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({ [`GET ${usageUrl('acme')}?days=30`]: () => jsonResponse(REPORT) }).fetch,
    );
    renderUsage();
    await screen.findByText('1,234');
    const days = chart.mock.lastCall?.[0].days as DayDownloads[];
    expect(days).toHaveLength(30);
    expect(days[0]).toEqual({ day: daysAgo(29), downloads: 0 });
    expect(days[27]).toEqual({ day: daysAgo(2), downloads: 1000 });
    expect(days[29]).toEqual({ day: daysAgo(0), downloads: 234 });
    expect(days.filter((day) => day.downloads === 0)).toHaveLength(28);
  });

  it('asks for the chosen period and points the CSV link at it', async () => {
    const fake = fakeServer({
      [`GET ${usageUrl('acme')}?days=30`]: () => jsonResponse(REPORT),
      [`GET ${usageUrl('acme')}?days=90`]: () => jsonResponse({ ...REPORT, days: 90 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderUsage();
    await screen.findByText('1,234');
    const link = screen.getByRole('link', { name: 'Download CSV' });
    expect(link).toHaveAttribute('href', '/service/rest/v1/customers/usage.csv?customer=acme&days=30');
    expect(link).toHaveAttribute('download');
    await userEvent.click(screen.getByRole('combobox', { name: 'Period' }));
    await userEvent.click(await screen.findByRole('option', { name: '90 days' }));
    await screen.findByRole('link', { name: 'Download CSV' });
    expect(screen.getByRole('link', { name: 'Download CSV' })).toHaveAttribute(
      'href',
      '/service/rest/v1/customers/usage.csv?customer=acme&days=90',
    );
    await waitFor(() => expect(chart.mock.lastCall?.[0].days).toHaveLength(90));
    expect(fake.calls.some((call) => call.path === `${usageUrl('acme')}?days=90`)).toBe(true);
  });
});
