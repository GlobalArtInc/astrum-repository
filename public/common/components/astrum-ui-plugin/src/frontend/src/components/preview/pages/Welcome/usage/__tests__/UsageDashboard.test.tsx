import React from 'react';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Theme } from '@radix-ui/themes';
import { UsageDashboard } from '../UsageDashboard';
import { fetchUsageDashboard } from '../usageDashboard.api';
import UIStrings from '../../../../../../constants/UIStrings';
import { dashboard, emptyDashboard } from './usageDashboard.fixtures';

jest.mock('../usageDashboard.api', () => ({
  fetchUsageDashboard: jest.fn(),
}));

jest.mock('recharts', () => {
  const Actual = jest.requireActual('recharts');
  const { cloneElement } = jest.requireActual('react');
  return {
    ...Actual,
    ResponsiveContainer: ({ children }: { children: React.ReactElement }) => cloneElement(children,
      { width: 800, height: 260 }),
  };
});

const fetchMock = fetchUsageDashboard as jest.MockedFunction<typeof fetchUsageDashboard>;
const STRINGS = UIStrings.USAGE_DASHBOARD;

beforeAll(() => {
  window.HTMLElement.prototype.hasPointerCapture = jest.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = jest.fn();
  window.HTMLElement.prototype.scrollIntoView = jest.fn();
});

function renderDashboard(props: React.ComponentProps<typeof UsageDashboard> = {}) {
  return render(<Theme><UsageDashboard {...props} /></Theme>);
}

describe('UsageDashboard', () => {
  beforeEach(() => {
    fetchMock.mockReset();
  });

  it('shows the headline figures of the last 30 days', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard();

    expect(await screen.findByText('2,418')).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith(30, null, expect.anything());
    expect(screen.getByRole('heading', { level: 1, name: STRINGS.TITLE })).toBeInTheDocument();
    expect(screen.getByText(STRINGS.SUBTITLE('30 days'))).toBeInTheDocument();
    expect(screen.getByText('1,200')).toBeInTheDocument();
    expect(screen.getByText('today 40 · avg 40 a day')).toBeInTheDocument();
    expect(screen.getByText('downloaded in the period: 380')).toBeInTheDocument();
    expect(screen.getByText('19,774')).toBeInTheDocument();
    expect(screen.getByText('in 3 repositories')).toBeInTheDocument();
    expect(screen.getByText('5 GB')).toBeInTheDocument();
    expect(screen.getByText('incl. proxy cache 1 GB')).toBeInTheDocument();
    expect(screen.getByText('hosted 2 · proxy 1 · group 0')).toBeInTheDocument();
    expect(screen.getByText('Not used for 90 days')).toBeInTheDocument();
    expect(screen.getByText('41 GB can be freed')).toBeInTheDocument();
  });

  it('charts the downloads per day with the peak', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard();

    expect(await screen.findByText(STRINGS.CHART.TITLE)).toBeInTheDocument();
    expect(screen.getByText('Last 30 days · 1,200 in total · peak 200 (Oct 2)')).toBeInTheDocument();
    expect(screen.getByText(STRINGS.CHART.WEEKDAYS)).toBeInTheDocument();
    expect(screen.getByText(STRINGS.CHART.WEEKENDS)).toBeInTheDocument();
    expect(screen.getByRole('img', { name: STRINGS.CHART.LABEL })).toBeInTheDocument();
  });

  it('links the most downloaded components to the browse tree of Astrum One', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard({ canSearch: true });

    const table = await screen.findByRole('table');
    const rows = within(table).getAllByRole('row');
    expect(rows).toHaveLength(3);
    expect(within(rows[1]).getByRole('link', { name: 'org.example:demo' }))
      .toHaveAttribute('href', '#browse/maven-releases/org/example/demo/1.0.0');
    expect(within(rows[1]).getByText('1.0.0')).toBeInTheDocument();
    expect(within(rows[1]).getByText('Maven')).toBeInTheDocument();
    expect(within(rows[2]).getByRole('link', { name: 'tools/keel.tar.gz' }))
      .toHaveAttribute('href', '#browse/raw-files/tools/keel.tar.gz');
    expect(screen.getByRole('link', { name: STRINGS.TOP.SEARCH_LINK })).toHaveAttribute('href', '#browse/search');
  });

  it('links the repositories and the cleanup policies to their Astrum One pages', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard({ canSearch: true, canConfigureCleanup: true });

    expect((await screen.findAllByRole('link', { name: 'raw-files' }))[0]).toHaveAttribute('href', '#browse/raw-files/');
    expect(screen.getByRole('link', { name: STRINGS.UNUSED.CLEANUP }))
      .toHaveAttribute('href', '#admin/repository/cleanup-policies');
  });

  it('switches the top list to all time', async () => {
    fetchMock.mockResolvedValue(dashboard());
    renderDashboard();
    await screen.findByRole('table');

    fireEvent.click(screen.getByRole('radio', { name: STRINGS.TOP.ALL_TIME }));

    expect(screen.getByRole('link', { name: 'org.example:legacy' })).toBeInTheDocument();
    expect(screen.getByText('4,000')).toBeInTheDocument();
  });

  it('ranks the repositories by downloads or by size', async () => {
    fetchMock.mockResolvedValue(dashboard());
    renderDashboard();
    const card = await screen.findByLabelText(STRINGS.REPOSITORIES.TITLE);

    expect(within(card).getByText('900')).toBeInTheDocument();

    fireEvent.click(within(card).getByRole('radio', { name: STRINGS.REPOSITORIES.BY_SIZE }));

    const names = within(card).getAllByRole('link').map((link) => link.textContent);
    expect(names).toEqual(['raw-files', 'maven-releases']);
    expect(within(card).getByText('3 GB')).toBeInTheDocument();
  });

  it('lists what was published lately and what is not used', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard({ canConfigureCleanup: true });

    const recent = await screen.findByLabelText(STRINGS.RECENT.TITLE);
    expect(within(recent).getByRole('link', { name: 'org.example:fresh' })).toBeInTheDocument();
    expect(within(recent).getByText('maven-releases · ci-bot')).toBeInTheDocument();
    const unused = screen.getByLabelText('Not downloaded for 90 days');
    expect(within(unused).getByText('612')).toBeInTheDocument();
    expect(within(unused).getByText('41 GB')).toBeInTheDocument();
    expect(within(unused).getByText('600 components · 40 GB')).toBeInTheDocument();
    expect(within(unused).getByText('never downloaded: 31 (1 MB)')).toBeInTheDocument();
    expect(within(unused).getByRole('link', { name: STRINGS.UNUSED.CLEANUP }))
      .toHaveAttribute('href', '#admin/repository/cleanup-policies');
  });

  it('offers cleanup policies only to those who may configure them', async () => {
    fetchMock.mockResolvedValue(dashboard());

    renderDashboard({ canConfigureCleanup: false, canSearch: false });

    await screen.findByLabelText('Not downloaded for 90 days');
    expect(screen.queryByRole('link', { name: STRINGS.UNUSED.CLEANUP })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: STRINGS.TOP.SEARCH_LINK })).not.toBeInTheDocument();
  });

  it('explains that there is no data yet', async () => {
    fetchMock.mockResolvedValue(emptyDashboard());

    renderDashboard();

    expect(await screen.findByText(STRINGS.CHART.EMPTY)).toBeInTheDocument();
    expect(screen.getByText(STRINGS.CHART.SUBTITLE_EMPTY('30 days'))).toBeInTheDocument();
    expect(screen.getByText(STRINGS.TOP.EMPTY)).toBeInTheDocument();
    expect(screen.getByText(STRINGS.REPOSITORIES.EMPTY_DOWNLOADS)).toBeInTheDocument();
    expect(screen.getByText(STRINGS.RECENT.EMPTY)).toBeInTheDocument();
    expect(screen.getByText(STRINGS.UNUSED.EMPTY('90 days'))).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });

  it('says so when the user may browse no repository', async () => {
    fetchMock.mockResolvedValue(dashboard({ repositories: [] }));

    renderDashboard();

    expect(await screen.findByText(STRINGS.NO_REPOSITORIES)).toBeInTheDocument();
    expect(screen.queryByText(STRINGS.CHART.TITLE)).not.toBeInTheDocument();
  });

  it('shows placeholders while loading', () => {
    fetchMock.mockReturnValue(new Promise(() => undefined));

    renderDashboard();

    expect(screen.getByRole('status', { name: STRINGS.LOADING })).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });

  it('shows the error and loads again on retry', async () => {
    fetchMock.mockRejectedValueOnce(new Error('Request failed with status code 500'));
    fetchMock.mockResolvedValueOnce(dashboard());

    renderDashboard();

    const alert = await screen.findByRole('alert');
    expect(within(alert).getByText(STRINGS.ERROR_TITLE)).toBeInTheDocument();
    expect(within(alert).getByText('Request failed with status code 500')).toBeInTheDocument();

    fireEvent.click(within(alert).getByRole('button', { name: STRINGS.RETRY }));

    expect(await screen.findByText('2,418')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('loads another period when one is picked', async () => {
    fetchMock.mockResolvedValue(dashboard());
    renderDashboard();
    await screen.findByText('2,418');

    fireEvent.click(screen.getByRole('radio', { name: '7 days' }));

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith(7, null, expect.anything()));
    expect(screen.getByText(STRINGS.SUBTITLE('7 days'))).toBeInTheDocument();
  });

  it('limits the figures to a repository picked from the list', async () => {
    fetchMock.mockResolvedValue(dashboard());
    renderDashboard();
    await screen.findByText('2,418');

    await act(async () => {
      await userEvent.click(screen.getByRole('combobox', { name: STRINGS.REPOSITORY_LABEL }));
    });
    await act(async () => {
      await userEvent.click(await screen.findByRole('option', { name: 'raw-files' }));
    });

    await waitFor(() => expect(fetchMock).toHaveBeenLastCalledWith(30, 'raw-files', expect.anything()));
    expect(screen.getByText(STRINGS.SUBTITLE_REPOSITORY('raw-files', '30 days'))).toBeInTheDocument();
  });
});
