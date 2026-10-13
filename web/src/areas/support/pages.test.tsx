import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { flatten, type HealthChecks, type Logger, orderSections, sectionTitle } from './api';
import LoggerPage from './LoggerPage';
import LoggingPage from './LoggingPage';
import LogsPage from './LogsPage';
import LogViewerPage from './LogViewerPage';
import MetricHealthPage, { orderChecks } from './MetricHealthPage';
import SupportZipPage from './SupportZipPage';
import SystemInformationPage from './SystemInformationPage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer(handlers);
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    routes: [
      { path: '/admin/support/logs', element: <LogsPage /> },
      { path: '/admin/support/logs/:file', element: <LogViewerPage /> },
      { path: '/admin/support/logging', element: <LoggingPage /> },
      { path: '/admin/support/logging/create', element: <LoggerPage /> },
      { path: '/admin/support/logging/:name', element: <LoggerPage /> },
      { path: '/admin/support/system-information', element: <SystemInformationPage /> },
      { path: '/admin/support/metric-health', element: <MetricHealthPage /> },
      { path: '/admin/support/metric-health/:check', element: <MetricHealthPage /> },
      { path: '/admin/support/support-zip', element: <SupportZipPage /> },
    ],
  });
}

const LOGS = '/service/rest/internal/logging/logs';
const LOGGERS = '/service/rest/internal/ui/loggingConfiguration';

function text(body: string): Response {
  return new Response(body, { status: 200, headers: { 'content-type': 'text/plain;charset=utf-8' } });
}

describe('the logs', () => {
  it('lists the log files and filters them by name', async () => {
    serve({
      [`GET ${LOGS}`]: () =>
        jsonResponse([
          { fileName: 'astrum-a.log', size: 2048, lastModified: 1791911926746, node: 'a', current: true },
          { fileName: 'astrum-a.log.1', size: 10, lastModified: 1791911926746, node: 'a', current: false },
        ]),
    });
    const { router } = page('/admin/support/logs');
    expect(await screen.findByRole('button', { name: 'Open astrum-a.log' })).toBeInTheDocument();
    expect(screen.getByText('About logs')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter by file name' }), '.1');
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Open astrum-a.log' })).toBeNull());
    await userEvent.click(screen.getByRole('button', { name: 'Open astrum-a.log.1' }));
    expect(router.state.location.pathname).toBe('/admin/support/logs/astrum-a.log.1');
  });

  it('shows the tail of a log, offers the download and inserts a mark', async () => {
    const server = serve({
      [`GET ${LOGS}/astrum-a.log?bytesCount=-25600`]: () => text('first line\nsecond line\n'),
      [`GET ${LOGS}/astrum-a.log?bytesCount=-51200`]: () => text('more\n'),
      'POST /service/rest/internal/logging/log/mark': () => new Response(null, { status: 204 }),
    });
    page('/admin/support/logs/astrum-a.log');
    const log = await screen.findByRole('log', { name: 'Contents of astrum-a.log' });
    await waitFor(() => expect(log).toHaveTextContent('second line'));
    expect(screen.getByRole('heading', { level: 1, name: 'Log Viewer' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Download' })).toHaveAttribute('href', `${LOGS}/astrum-a.log`);
    await userEvent.type(screen.getByRole('textbox', { name: 'Marker to insert:' }), 'before upgrade');
    await userEvent.click(screen.getByRole('button', { name: 'Insert' }));
    expect(await screen.findByText('The mark was inserted')).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toBe('before upgrade');
    await userEvent.click(screen.getByRole('combobox', { name: 'Size:' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Last 50 KB' }));
    await waitFor(() => expect(log).toHaveTextContent('more'));
  });
});

const LOGGER_LIST: Logger[] = [
  { name: 'ROOT', level: 'INFO', override: false },
  { name: 'astrum_api', level: 'DEBUG', override: true },
  { name: 'sqlx', level: 'WARN', override: false },
];

describe('the logging levels', () => {
  it('counts levels, filters by them and resets every logger', async () => {
    const server = serve({
      [`GET ${LOGGERS}`]: () => jsonResponse(LOGGER_LIST),
      [`POST ${LOGGERS}/reset`]: () => new Response(null, { status: 204 }),
    });
    page('/admin/support/logging');
    expect(await screen.findByText('Showing 3 of 3 loggers')).toBeInTheDocument();
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('checkbox', { name: /WARN/ }));
    expect(await screen.findByText('Showing 1 of 3 loggers')).toBeInTheDocument();
    expect(screen.getAllByRole('button', { name: 'Edit sqlx' }).length).toBeGreaterThan(0);
    await userEvent.click(screen.getByRole('button', { name: 'Reset to Default Levels' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Reset All' }));
    expect(await screen.findByText('Every logger is back at its default level')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'POST' && call.path === `${LOGGERS}/reset`)).toBe(
      true,
    );
  });

  it('creates a logger with a level', async () => {
    const server = serve({
      [`GET ${LOGGERS}`]: () => jsonResponse(LOGGER_LIST),
      [`PUT ${LOGGERS}/astrum_content%3A%3Aproxy`]: () => new Response(null, { status: 204 }),
    });
    const { router } = page('/admin/support/logging/create');
    const name = await screen.findByRole('textbox', { name: /Logger Name/ });
    await userEvent.type(name, 'org.example');
    expect(
      await screen.findByText('Use ROOT or a Rust module path such as astrum_api::routes'),
    ).toBeInTheDocument();
    await userEvent.clear(name);
    await userEvent.type(name, 'astrum_content::proxy');
    await userEvent.click(screen.getByRole('combobox', { name: /Logger Level/ }));
    await userEvent.click(await screen.findByRole('option', { name: 'TRACE' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create Logger' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/support/logging'));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual({
      name: 'astrum_content::proxy',
      level: 'TRACE',
    });
  });

  it('removes the override of a logger', async () => {
    const server = serve({
      [`GET ${LOGGERS}/astrum_api`]: () => jsonResponse(LOGGER_LIST[1]),
      [`POST ${LOGGERS}/astrum_api/reset`]: () => new Response(null, { status: 204 }),
    });
    page('/admin/support/logging/astrum_api');
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('The level override of astrum_api was removed')).toBeInTheDocument();
    expect(server.calls.some((call) => call.path === `${LOGGERS}/astrum_api/reset`)).toBe(true);
  });
});

const CHECKS: HealthChecks = {
  Database: {
    healthy: true,
    message: 'PostgreSQL answers.',
    error: null,
    details: null,
    time: 1,
    duration: 3,
    timestamp: '2026-10-13T17:24:47.840Z',
  },
  'Default Admin Credentials': {
    healthy: false,
    message: 'The generated password is still there.',
    error: null,
    details: null,
    time: 1,
    duration: 0,
    timestamp: '2026-10-13T17:24:47.840Z',
  },
};

describe('the metric health', () => {
  it('puts failing checks first and shows the chosen one', async () => {
    serve({ 'GET /service/rest/internal/ui/status-check': () => jsonResponse(CHECKS) });
    page('/admin/support/metric-health');
    expect(await screen.findByText('1 healthy')).toBeInTheDocument();
    expect(screen.getByText('1 unhealthy')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 2, name: 'Default Admin Credentials' })).toBeInTheDocument();
    expect(screen.getByText('Unhealthy')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Database' }));
    expect(await screen.findByRole('heading', { level: 2, name: 'Database' })).toBeInTheDocument();
    expect(screen.getByText('PostgreSQL answers.')).toBeInTheDocument();
    expect(screen.getByText('Duration: 3 ms')).toBeInTheDocument();
  });

  it('orders checks by health, then by name', () => {
    expect(orderChecks(CHECKS).map(([name]) => name)).toEqual(['Default Admin Credentials', 'Database']);
  });
});

describe('the system information', () => {
  it('shows sections with their item counts and expands them', async () => {
    serve({
      'GET /service/rest/atlas/system-information': () =>
        jsonResponse({
          'astrum-status': { version: '3.0.0', edition: 'CORE' },
          'astrum-node': { 'node-id': 'n1' },
          'system-database': { product: 'PostgreSQL', version: '18.0' },
        }),
    });
    page('/admin/support/system-information');
    expect(await screen.findByText('3.0.0')).toBeInTheDocument();
    expect(screen.getAllByText('(2 items)')).toHaveLength(2);
    expect(screen.queryByText('18.0')).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: /System Database/ }));
    expect(await screen.findByText('18.0')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Collapse All' }));
    expect(screen.queryByText('3.0.0')).toBeNull();
  });

  it('flattens nested values and names sections', () => {
    expect(flatten({ a: { b: 1, c: [true, null] } })).toEqual([
      ['a.b', '1'],
      ['a.c.0', 'true'],
      ['a.c.1', ''],
    ]);
    expect(sectionTitle('system-filestores')).toBe('System Filestores');
    expect(
      orderSections({ 'system-os': 1, 'astrum-node': 2, 'astrum-status': 3, extra: 4 }).map(([key]) => key),
    ).toEqual(['astrum-status', 'astrum-node', 'system-os', 'extra']);
  });
});

describe('the support ZIP', () => {
  it('creates a ZIP and offers it for download', async () => {
    const server = serve({
      'POST /service/rest/v1/support/supportzippath': () =>
        jsonResponse({
          file: '/astrum-data/downloads/support-1.zip',
          name: 'support-1.zip',
          size: 2048,
          truncated: false,
        }),
    });
    page('/admin/support/support-zip');
    await userEvent.click(await screen.findByRole('checkbox', { name: 'Audit log files' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create Support ZIP' }));
    expect(await screen.findByText('Support ZIP created')).toBeInTheDocument();
    expect(screen.getByText('/astrum-data/downloads/support-1.zip')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Download' })).toHaveAttribute(
      'href',
      '/service/rest/wonderland/download/support-1.zip',
    );
    expect(server.calls[0]?.body).toEqual({
      systemInformation: true,
      configuration: true,
      security: true,
      log: true,
      taskLog: true,
      auditLog: false,
      metrics: true,
      archivedLog: 0,
      limitFileSizes: true,
      limitZipSize: true,
    });
  });
});
