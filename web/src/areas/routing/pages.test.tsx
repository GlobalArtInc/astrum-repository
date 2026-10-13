import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import type { PreviewNode, RoutingRuleDetails } from './model';
import PreviewPage from './PreviewPage';
import RoutingRulePage from './RoutingRulePage';
import RoutingRulesPage from './RoutingRulesPage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

const UI = '/service/rest/internal/ui/routing-rules';
const V1 = '/service/rest/v1/routing-rules';

function rule(name: string, extra: Partial<RoutingRuleDetails> = {}): RoutingRuleDetails {
  return {
    name,
    description: '',
    mode: 'BLOCK',
    matchers: ['^/blocked/.*'],
    assignedRepositoryCount: 0,
    assignedRepositoryNames: [],
    ...extra,
  };
}

const RULES = [
  rule('block-sources', { description: 'No source jars', matchers: ['.*-sources\\.jar', '.*/secret/.*'] }),
  rule('only-org', {
    mode: 'ALLOW',
    matchers: ['^/org/.*'],
    assignedRepositoryCount: 2,
    assignedRepositoryNames: ['a', 'b'],
  }),
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer({
    [`GET ${UI}?includeRepositoryNames=true`]: () => jsonResponse(RULES),
    [`GET ${UI}/block-sources`]: () => jsonResponse(RULES[0]),
    [`GET ${UI}/only-org`]: () => jsonResponse(RULES[1]),
    [`POST ${UI}/test`]: ({ body }) => {
      const sent = body as { matchers: string[]; path: string };
      const results = sent.matchers.map((pattern) => {
        try {
          return { matches: new RegExp(`^(?:${pattern})$`).test(sent.path), error: null };
        } catch {
          return { matches: false, error: 'unclosed group' };
        }
      });
      return jsonResponse({ allowed: !results.some((result) => result.matches), matchers: results });
    },
    ...handlers,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    routes: [
      { path: '/admin/repository/routing-rules', element: <RoutingRulesPage /> },
      { path: '/admin/repository/routing-rules/create', element: <RoutingRulePage /> },
      { path: '/admin/repository/routing-rules/preview', element: <PreviewPage /> },
      { path: '/admin/repository/routing-rules/:name', element: <RoutingRulePage /> },
    ],
  });
}

describe('the routing rule list', () => {
  it('lists, sorts and filters the rules', async () => {
    serve({});
    const { router } = page('/admin/repository/routing-rules');
    const table = await screen.findByRole('table', { name: 'Routing Rules' });
    await waitFor(() => expect(within(table).getAllByRole('row')).toHaveLength(3));
    const names = () =>
      within(table)
        .getAllByRole('row')
        .slice(1)
        .map((row) => row.querySelector('td')?.textContent);
    expect(names()).toEqual(['block-sources', 'only-org']);
    expect(within(table).getByText('No source jars')).toBeInTheDocument();
    expect(within(table).getByText('Allow')).toBeInTheDocument();
    await userEvent.click(within(table).getByRole('button', { name: /Repositories/ }));
    await userEvent.click(within(table).getByRole('button', { name: /Repositories/ }));
    expect(names()).toEqual(['only-org', 'block-sources']);
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter by name or description' }), 'jars');
    expect(names()).toEqual(['block-sources']);
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter by name or description' }), 'zzz');
    expect(screen.getByText('No matching rules')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Create Rule' })).toHaveAttribute(
      'href',
      '/admin/repository/routing-rules/create',
    );
    expect(screen.getByRole('link', { name: 'Preview' })).toHaveAttribute(
      'href',
      '/admin/repository/routing-rules/preview',
    );
    expect(screen.getByText('What are Routing Rules?')).toBeInTheDocument();
    await userEvent.clear(screen.getByRole('textbox', { name: 'Filter by name or description' }));
    await userEvent.click(screen.getByRole('row', { name: 'Open only-org' }));
    expect(router.state.location.pathname).toBe('/admin/repository/routing-rules/only-org');
  });

  it('says when there are no rules', async () => {
    serve({ [`GET ${UI}?includeRepositoryNames=true`]: () => jsonResponse([]) });
    page('/admin/repository/routing-rules');
    expect(await screen.findByText('No routing rules')).toBeInTheDocument();
  });
});

describe('the routing rule form', () => {
  it('creates a rule from typed matchers and shows server problems', async () => {
    const posted: unknown[] = [];
    let first = true;
    const server = serve({
      [`POST ${V1}`]: ({ body }) => {
        posted.push(body);
        if (first) {
          first = false;
          return jsonResponse(
            [{ id: 'PARAMETER name', message: 'is already used by another routing rule' }],
            400,
          );
        }
        return new Response(null, { status: 204 });
      },
    });
    const { router } = page('/admin/repository/routing-rules/create');
    expect(await screen.findByRole('heading', { name: 'Create Routing Rule', level: 1 })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Create' })).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /^Description/ }), 'x');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('Enter a name')).toBeInTheDocument();
    expect(screen.getByText('At least one matcher is required')).toBeInTheDocument();
    await userEvent.clear(screen.getByRole('textbox', { name: /^Description/ }));
    await userEvent.type(screen.getByRole('textbox', { name: /^Name/ }), 'no-sources');
    await userEvent.type(screen.getByRole('textbox', { name: 'Matcher 1' }), '(');
    await userEvent.click(screen.getByRole('button', { name: 'Add Matcher' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: /Ends with/ }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Matcher 2' }), '-sources.jar');
    expect(await screen.findByText('Invalid regular expression: (')).toBeInTheDocument();
    expect(screen.getByText('Invalid')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Remove matcher 1' }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Test Path' }), 'a/b-sources.jar');
    expect(await screen.findByText('This request would be BLOCKED')).toBeInTheDocument();
    expect(screen.getByText('Matches')).toBeInTheDocument();
    expect(screen.getByText('ends with “-sources.jar”')).toBeInTheDocument();
    const testCall = server.calls.filter((call) => call.path.endsWith('/test')).at(-1);
    expect(testCall?.body).toEqual({
      mode: 'BLOCK',
      matchers: ['.*-sources\\.jar'],
      path: '/a/b-sources.jar',
    });
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('is already used by another routing rule')).toBeInTheDocument();
    await userEvent.clear(screen.getByRole('textbox', { name: /^Name/ }));
    await userEvent.type(screen.getByRole('textbox', { name: /^Name/ }), 'no-sources-2');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/repository/routing-rules'));
    expect(posted.at(-1)).toEqual({
      name: 'no-sources-2',
      description: '',
      mode: 'BLOCK',
      matchers: ['.*-sources\\.jar'],
    });
  });

  it('edits a rule, says how it is used and refuses to delete it while assigned', async () => {
    const put: unknown[] = [];
    serve({
      [`PUT ${V1}/only-org`]: ({ body }) => {
        put.push(body);
        return new Response(null, { status: 204 });
      },
    });
    const { router } = page('/admin/repository/routing-rules/only-org');
    expect(await screen.findByRole('heading', { name: 'Edit only-org', level: 1 })).toBeInTheDocument();
    expect(
      await screen.findByText('Mode: Allow • 1 matcher • Assigned to 2 repositories'),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeDisabled();
    expect(screen.getByText(/assigned to 2 repositories: a, b/)).toBeInTheDocument();
    expect(screen.getByText('starts with “/org/”')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /^Description/ }), 'Only our own');
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/repository/routing-rules'));
    expect(put).toEqual([
      { name: 'only-org', description: 'Only our own', mode: 'ALLOW', matchers: ['^/org/.*'] },
    ]);
  });

  it('deletes an unused rule after asking', async () => {
    const deleted: string[] = [];
    serve({
      [`DELETE ${V1}/block-sources`]: ({ path }) => {
        deleted.push(path);
        return new Response(null, { status: 204 });
      },
    });
    const { router } = page('/admin/repository/routing-rules/block-sources');
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Delete the routing rule?' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/repository/routing-rules'));
    expect(deleted).toHaveLength(1);
  });
});

describe('the global routing preview', () => {
  const tree: PreviewNode = {
    repository: null,
    type: null,
    format: null,
    rule: null,
    allowed: null,
    expanded: true,
    expandable: true,
    children: [
      {
        repository: 'raw-all',
        type: 'group',
        format: 'raw',
        rule: null,
        allowed: true,
        expanded: true,
        expandable: true,
        children: [
          {
            repository: 'raw-remote',
            type: 'proxy',
            format: 'raw',
            rule: 'block-sources',
            allowed: false,
            expanded: false,
            expandable: false,
            children: null,
          },
        ],
      },
      {
        repository: 'raw-remote',
        type: 'proxy',
        format: 'raw',
        rule: 'block-sources',
        allowed: false,
        expanded: false,
        expandable: false,
        children: null,
      },
    ],
  };

  it('tests a path in every repository and opens the rule that decided', async () => {
    const server = serve({
      [`GET ${UI}/preview?path=%2Fa%2Fx-sources.jar&filter=proxies`]: () => jsonResponse(tree),
    });
    page('/admin/repository/routing-rules/preview');
    expect(await screen.findByText('How to Use')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Test' })).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: 'Test Path' }), 'a/x-sources.jar');
    await userEvent.click(screen.getByRole('combobox', { name: 'Filter' }));
    await userEvent.click(screen.getByRole('option', { name: 'Proxy Repositories' }));
    await userEvent.click(screen.getByRole('button', { name: 'Test' }));
    const status = await screen.findByRole('status');
    expect(within(status).getByText('3')).toBeInTheDocument();
    expect(within(status).getByText('1')).toBeInTheDocument();
    expect(within(status).getByText('2')).toBeInTheDocument();
    expect(server.calls.some((call) => call.path.includes('filter=proxies'))).toBe(true);
    const list = screen.getByRole('list', { name: 'Global Routing Preview' });
    expect(within(list).getAllByText('raw-remote')).toHaveLength(2);
    await userEvent.click(screen.getByRole('button', { name: 'Hide the members of raw-all' }));
    expect(within(list).getAllByText('raw-remote')).toHaveLength(1);
    await userEvent.click(screen.getByRole('button', { name: 'Open the routing rule block-sources' }));
    const dialog = await screen.findByRole('dialog', { name: 'block-sources' });
    expect(within(dialog).getByText('/a/x-sources.jar')).toBeInTheDocument();
    expect(within(dialog).getByText('.*/secret/.*')).toBeInTheDocument();
  });
});
