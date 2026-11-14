import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../../test/http';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { ComponentSecurity, Verdict, Vulnerability } from '../../security/findings/model';
import { repository } from '../test-fixtures';
import type { SearchComponent } from '../types';
import ComponentPage from './ComponentPage';

const VERSIONS = '/service/rest/internal/ui/search/versions?format=maven2&group=org.acme&name=lib';
const SECURITY = '/service/rest/v1/components/releases-1.0/security';
const URL = `/search/component/${encodeURIComponent('maven2:org.acme:lib')}`;

const COMPONENT: SearchComponent = {
  id: 'releases-1.0',
  repository: 'releases',
  format: 'maven2',
  group: 'org.acme',
  name: 'lib',
  version: '1.0',
  assets: [],
};

const VERDICT: Verdict = {
  policy: 'strict',
  revision: 1,
  state: 'block',
  blocked: true,
  counts: { critical: 1, high: 0, medium: 0, low: 0, unknown: 0 },
  maxSeverity: 'critical',
  reasons: [
    { kind: 'vulnerability', advisory: 'GHSA-aaaa-0001', package: 'lib@1.0' },
    { kind: 'would-block' },
  ],
  computedAt: '2026-10-02T10:00:00.000Z',
};

function vulnerability(overrides: Partial<Vulnerability>): Vulnerability {
  return {
    id: 'GHSA-aaaa-0001',
    aliases: [],
    severity: 'critical',
    score: 9.8,
    summary: 'summary of the advisory',
    fixed: '1.2.3',
    malicious: false,
    excepted: false,
    ...overrides,
  };
}

function security(overrides: Partial<ComponentSecurity> = {}): ComponentSecurity {
  return {
    scan: { state: 'scanned', scannedAt: '2026-10-01T10:00:00.000Z', packages: 2 },
    policy: { name: 'strict', revision: 1 },
    verdict: VERDICT,
    vulnerabilities: [vulnerability({})],
    truncated: false,
    license: null,
    ...overrides,
  };
}

function serve(response: () => Response, version = '1.0', extra: Record<string, Handler> = {}) {
  const server = fakeServer({
    [`GET ${VERSIONS}`]: () =>
      jsonResponse({
        format: 'maven2',
        group: 'org.acme',
        name: 'lib',
        versions: [
          {
            version,
            repositories: ['releases'],
            componentIds: ['releases-1.0'],
            lastModified: '2026-10-01T10:00:00.000+00:00',
          },
        ],
      }),
    'GET /service/rest/v1/components/releases-1.0': () => jsonResponse(COMPONENT),
    [`GET ${SECURITY}`]: response,
    'GET /service/rest/internal/ui/repositories': () =>
      jsonResponse([repository('releases', 'maven2', 'hosted')]),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string, state = ADMIN_STATE) {
  return renderWithApp(null, {
    path,
    state,
    routes: [{ path: 'search/component/*', element: <ComponentPage /> }],
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the security tab of a component', { timeout: 20_000 }, () => {
  it.each([
    ['pass', 'Pass'],
    ['report', 'Report'],
    ['block', 'Block'],
    ['unscanned', 'Unscanned'],
    ['error', 'Policy error'],
  ])('shows the %s verdict as its label', async (state, label) => {
    serve(() => jsonResponse(security({ verdict: { ...VERDICT, state, blocked: state === 'block' } })));
    page(`${URL}/security?version=1.0`);
    expect(await screen.findByText(label, { exact: true })).toBeInTheDocument();
  });

  it('says that no policy and an unjudged component are not a pass', async () => {
    serve(() => jsonResponse(security({ policy: null, verdict: null })));
    page(`${URL}/security?version=1.0`);
    expect(
      await screen.findByText(
        'No policy applies to this repository, so nothing is judged. This is not a pass.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText('Pass')).not.toBeInTheDocument();
  });

  it('lists the stored reasons and the not-available licence', async () => {
    serve(() => jsonResponse(security({ truncated: true })));
    page(`${URL}/security?version=1.0`);
    expect(await screen.findByText('Blocked', { exact: true })).toBeInTheDocument();
    expect(screen.getByText('GHSA-aaaa-0001 in lib@1.0')).toBeInTheDocument();
    expect(screen.getByText('Would be blocked in block mode')).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('The list is cut');
    expect(screen.getByText('Not available yet')).toBeInTheDocument();
  });

  it('shows every severity with the markers of malicious and excepted advisories', async () => {
    serve(() =>
      jsonResponse(
        security({
          vulnerabilities: [
            vulnerability({ id: 'GHSA-crit', severity: 'critical', score: 9.8 }),
            vulnerability({ id: 'GHSA-high', severity: 'high', score: 7.5, fixed: null }),
            vulnerability({ id: 'GHSA-med', severity: 'medium', score: 5.0, excepted: true }),
            vulnerability({ id: 'GHSA-low', severity: 'low', score: 2.1 }),
            vulnerability({
              id: 'MAL-2024-1',
              severity: 'critical',
              score: null,
              fixed: null,
              malicious: true,
            }),
          ],
        }),
      ),
    );
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows).toHaveLength(5);
    expect(within(table).getByRole('rowheader', { name: /GHSA-high/ })).toBeInTheDocument();
    for (const label of ['Critical', 'High', 'Medium', 'Low']) {
      expect(within(table).getAllByText(label, { exact: true }).length).toBeGreaterThan(0);
    }
    expect(within(table).getByText('Malicious', { exact: true })).toBeInTheDocument();
    expect(within(table).getByText('Excepted', { exact: true })).toBeInTheDocument();
    expect(within(table).getByText('9.8')).toBeInTheDocument();
  });

  it('shows advisory text as plain text, never as markup', async () => {
    const hostile = '<script>alert("x")</script>';
    serve(() => jsonResponse(security({ vulnerabilities: [vulnerability({ summary: hostile })] })));
    page(`${URL}/security?version=1.0`);
    expect(await screen.findByText(hostile)).toBeInTheDocument();
    expect(document.querySelector('script')).toBeNull();
  });

  it('marks the selected version with the worst severity of its advisories', async () => {
    serve(() =>
      jsonResponse(
        security({
          vulnerabilities: [
            vulnerability({ id: 'GHSA-low', severity: 'low' }),
            vulnerability({ id: 'GHSA-high', severity: 'high' }),
          ],
        }),
      ),
    );
    page(`${URL}/versions?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Versions' });
    const row = (await within(table).findByRole('rowheader', { name: /1\.0/ })).closest('tr');
    expect(row).not.toBeNull();
    expect(await within(row as HTMLElement).findByText('High', { exact: true })).toBeInTheDocument();
  });

  it('hides the tab when the REST does not show security data', async () => {
    serve(() => jsonResponse({ id: '*', message: 'not visible' }, 404));
    page(`${URL}/versions?version=1.0`);
    await screen.findByRole('table', { name: 'Versions' });
    expect(screen.queryByRole('link', { name: /^Security/ })).not.toBeInTheDocument();
  });

  it('offers the tab with an error when the REST fails', async () => {
    serve(() => jsonResponse({ id: '*', message: 'boom' }, 500));
    page(`${URL}/versions?version=1.0`);
    expect(await screen.findByRole('link', { name: /^Security/ })).toBeInTheDocument();
  });
});

const POLICIES = '/service/rest/v1/security/policies';
const EXCEPTIONS = '/service/rest/v1/security/exceptions';
const ADD = { name: 'Add exception' };

function policies(): Response {
  return jsonResponse([{ name: 'strict' }]);
}

describe('adding an exception from the security tab', { timeout: 20_000 }, () => {
  it('opens the dialog prefilled with the advisory, the component and the policy', async () => {
    serve(() => jsonResponse(security()), '1.0', { [`GET ${POLICIES}`]: policies });
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    await userEvent.click(within(table).getByRole('button', ADD));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByRole('textbox', { name: 'Advisory' })).toHaveValue('GHSA-aaaa-0001');
    expect(within(dialog).getByRole('textbox', { name: 'Namespace' })).toHaveValue('org.acme');
    expect(within(dialog).getByRole('textbox', { name: 'Name' })).toHaveValue('lib');
    expect(within(dialog).getByRole('textbox', { name: 'Version' })).toHaveValue('1.0');
    expect(within(dialog).getByRole('combobox', { name: 'Format' })).toHaveTextContent('maven2');
    expect(within(dialog).getByRole('combobox', { name: 'Policy' })).toHaveTextContent('strict');
    expect(within(dialog).getByRole('combobox', { name: 'Subject' })).toHaveTextContent('Advisory');
  });

  it('sends exactly the advisory with the component and the policy', async () => {
    const server = serve(() => jsonResponse(security()), '1.0', {
      [`GET ${POLICIES}`]: policies,
      [`POST ${EXCEPTIONS}`]: () => jsonResponse({}, 201),
    });
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    await userEvent.click(within(table).getByRole('button', ADD));
    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Reason' }), 'vendor patch in review');
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Expiry' }));
    await userEvent.click(screen.getByRole('option', { name: 'Never' }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Exception' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(server.calls.find((call) => call.method === 'POST')).toEqual({
      method: 'POST',
      path: EXCEPTIONS,
      body: {
        policy: 'strict',
        rule: 'vulnerability',
        advisory: 'GHSA-aaaa-0001',
        format: 'maven2',
        namespace: 'org.acme',
        name: 'lib',
        version: '1.0',
        reason: 'vendor patch in review',
        expiresAt: null,
      },
    });
  });

  it('shows the refusal of the server in the dialog', async () => {
    serve(() => jsonResponse(security()), '1.0', {
      [`GET ${POLICIES}`]: policies,
      [`POST ${EXCEPTIONS}`]: () => jsonResponse({ id: '*', message: 'policy is read-only' }, 403),
    });
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    await userEvent.click(within(table).getByRole('button', ADD));
    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Reason' }), 'vendor patch in review');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Exception' }));
    expect(await within(dialog).findByText('The server refused: policy is read-only')).toBeInTheDocument();
  });

  it('hides the action without the privilege to create exceptions', async () => {
    serve(() => jsonResponse(security()));
    page(`${URL}/security?version=1.0`, { ...ADMIN_STATE, permissions: ['astrum:search:read'] });
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    expect(within(table).getByText('GHSA-aaaa-0001')).toBeInTheDocument();
    expect(screen.queryByRole('button', ADD)).not.toBeInTheDocument();
  });

  it('hides the action when the repository has no policy', async () => {
    serve(() => jsonResponse(security({ policy: null, verdict: null })));
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    expect(within(table).getByText('GHSA-aaaa-0001')).toBeInTheDocument();
    expect(screen.queryByRole('button', ADD)).not.toBeInTheDocument();
  });

  it('hides the action for an advisory that is already excepted', async () => {
    serve(
      () =>
        jsonResponse(
          security({
            vulnerabilities: [
              vulnerability({ id: 'GHSA-excepted', excepted: true }),
              vulnerability({ id: 'GHSA-open', excepted: false }),
            ],
          }),
        ),
      '1.0',
      { [`GET ${POLICIES}`]: policies },
    );
    page(`${URL}/security?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Vulnerabilities' });
    expect(within(table).getAllByRole('button', ADD)).toHaveLength(1);
    const excepted = within(table)
      .getByRole('rowheader', { name: /GHSA-excepted/ })
      .closest('tr') as HTMLElement;
    expect(within(excepted).queryByRole('button', ADD)).not.toBeInTheDocument();
  });
});
