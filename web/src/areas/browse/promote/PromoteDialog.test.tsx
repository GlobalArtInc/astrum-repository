import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { PromotionTarget } from './model';
import { PromoteDialog, type PromoteDialogProps } from './PromoteDialog';

const PROMOTE = '/service/rest/v1/promotions';
const TARGETS: PromotionTarget[] = [
  { name: 'releases-eu', format: 'maven2', type: 'hosted' },
  { name: 'releases-us', format: 'maven2', type: 'hosted' },
];

function dialog(overrides: Partial<PromoteDialogProps> = {}) {
  const props: PromoteDialogProps = {
    open: true,
    onOpenChange: vi.fn(),
    source: 'staging',
    format: 'maven2',
    items: [{ componentId: 'staging-1.0' }],
    targets: TARGETS,
    ...overrides,
  };
  return <PromoteDialog {...props} />;
}

function serve(respond: () => Response) {
  const fake = fakeServer({ [`POST ${PROMOTE}`]: respond });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

type Calls = ReturnType<typeof fakeServer>['calls'];

function body(calls: Calls) {
  return calls.filter((call) => call.path === PROMOTE).at(-1)?.body;
}

function rowAt(rows: HTMLElement[], index: number): HTMLElement {
  const row = rows[index];
  if (row === undefined) {
    throw new Error(`no row ${index}`);
  }
  return row;
}

async function pickTarget(name: string) {
  await userEvent.click(screen.getByRole('combobox'));
  await userEvent.click(await screen.findByRole('option', { name }));
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the promote dialog', () => {
  it('lists the targets and previews with dryRun, writing nothing', async () => {
    const fake = serve(() =>
      jsonResponse({
        dryRun: true,
        results: [
          {
            item: { componentId: 'staging-1.0' },
            status: 'copied',
            assetCount: 2,
            bytesCopied: 2048,
            bytesReused: 0,
          },
        ],
      }),
    );
    renderWithApp(dialog());
    expect(screen.getByRole('button', { name: 'Preview' })).toBeDisabled();
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('button', { name: 'Preview' }));
    expect(await screen.findByText('Preview: nothing was written')).toBeInTheDocument();
    expect(body(fake.calls)).toEqual({
      source: 'staging',
      target: 'releases-eu',
      items: [{ componentId: 'staging-1.0' }],
      move: false,
      conflict: 'fail',
      dryRun: true,
    });
    expect(screen.getByRole('checkbox', { name: /Move/ })).not.toBeChecked();
  });

  it('promotes a raw component with the chosen conflict mode', async () => {
    const fake = serve(() =>
      jsonResponse({ dryRun: false, results: [{ item: { componentId: 'staging-1.0' }, status: 'copied' }] }),
    );
    renderWithApp(dialog());
    await pickTarget('releases-us');
    await userEvent.click(screen.getByRole('radio', { name: /Skip/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Promote' }));
    await screen.findByText('Copied');
    expect(body(fake.calls)).toEqual({
      source: 'staging',
      target: 'releases-us',
      items: [{ componentId: 'staging-1.0' }],
      move: false,
      conflict: 'skip',
      dryRun: false,
    });
  });

  it('promotes a Docker tag with a move and the extra tags', async () => {
    const fake = serve(() =>
      jsonResponse({
        dryRun: false,
        results: [{ item: { image: 'team/app', tag: '1.2' }, status: 'copied' }],
      }),
    );
    renderWithApp(dialog({ format: 'docker', source: 'images', items: [{ image: 'team/app', tag: '1.2' }] }));
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('checkbox', { name: /Move/ }));
    await userEvent.click(screen.getByRole('radio', { name: /Replace/ }));
    await userEvent.type(screen.getByRole('textbox', { name: /Extra tags/ }), 'stable, lts');
    await userEvent.click(screen.getByRole('button', { name: 'Preview' }));
    await screen.findByText('Preview: nothing was written');
    await userEvent.click(screen.getByRole('button', { name: 'Promote' }));
    await waitFor(() => expect(fake.calls).toHaveLength(2));
    expect(body(fake.calls)).toEqual({
      source: 'images',
      target: 'releases-eu',
      items: [{ image: 'team/app', tag: '1.2', targetTags: ['stable', 'lts'] }],
      move: true,
      conflict: 'replace',
      dryRun: false,
    });
  });

  it('shows one row per item with its status, source kept and retry hint', async () => {
    serve(() =>
      jsonResponse({
        dryRun: false,
        results: [
          {
            item: { componentId: 'a-1.0' },
            status: 'copied',
            assetCount: 3,
            bytesCopied: 10,
            sourceKept: true,
            warning: 'kept by the immutable rule',
            skippedChildren: 2,
          },
          { item: { componentId: 'b-1.0' }, status: 'unchanged', retryable: true },
          {
            item: { componentId: 'c-1.0' },
            status: 'conflict',
            retryable: false,
            error: 'the target holds other content',
          },
          { item: { componentId: 'd-1.0' }, status: 'forbidden', error: 'no right to add to releases-eu' },
        ],
      }),
    );
    renderWithApp(
      dialog({
        items: [
          { componentId: 'a-1.0' },
          { componentId: 'b-1.0' },
          { componentId: 'c-1.0' },
          { componentId: 'd-1.0' },
        ],
      }),
    );
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('button', { name: 'Promote' }));
    const table = await screen.findByRole('table', { name: 'Results' });
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows).toHaveLength(4);
    const copied = rowAt(rows, 0);
    const unchanged = rowAt(rows, 1);
    const conflict = rowAt(rows, 2);
    const forbidden = rowAt(rows, 3);
    expect(within(copied).getByText('Copied')).toBeInTheDocument();
    expect(within(copied).getByText('Source kept')).toBeInTheDocument();
    expect(within(copied).getByText('kept by the immutable rule')).toBeInTheDocument();
    expect(within(copied).getByText('2 child manifests left out')).toBeInTheDocument();
    expect(within(unchanged).getByText('Unchanged')).toBeInTheDocument();
    expect(within(unchanged).getByText('A repeat may succeed.')).toBeInTheDocument();
    expect(within(conflict).getByText('Conflict')).toBeInTheDocument();
    expect(within(conflict).queryByText('A repeat may succeed.')).not.toBeInTheDocument();
    expect(within(forbidden).getByText('Forbidden')).toBeInTheDocument();
    expect(within(forbidden).getByText('no right to add to releases-eu')).toBeInTheDocument();
  });

  it('shows the server message on an error response', async () => {
    serve(() => jsonResponse({ id: 'FORBIDDEN', message: 'you may not add to releases-eu' }, 403));
    renderWithApp(dialog());
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('button', { name: 'Promote' }));
    const alert = await screen.findByRole('alert');
    await waitFor(() => expect(alert).toHaveTextContent('you may not add to releases-eu'));
  });

  it('holds Promote back with Move on until the settings have been previewed', async () => {
    serve(() =>
      jsonResponse({ dryRun: true, results: [{ item: { componentId: 'staging-1.0' }, status: 'copied' }] }),
    );
    renderWithApp(dialog());
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('checkbox', { name: /Move/ }));
    expect(screen.getByRole('button', { name: 'Promote' })).toBeDisabled();
    expect(screen.getByText(/run Preview with these settings first/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Preview' }));
    await screen.findByText('Preview: nothing was written');
    expect(screen.getByRole('button', { name: 'Promote' })).toBeEnabled();
    await userEvent.click(screen.getByRole('radio', { name: /Skip/ }));
    expect(screen.getByRole('button', { name: 'Promote' })).toBeDisabled();
  });

  it('cannot be closed by accident while a run is in flight', async () => {
    const onOpenChange = vi.fn();
    let answer: (response: Response) => void = () => {};
    vi.stubGlobal(
      'fetch',
      vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            answer = resolve;
          }),
      ),
    );
    renderWithApp(dialog({ onOpenChange }));
    await pickTarget('releases-eu');
    await userEvent.click(screen.getByRole('button', { name: 'Promote' }));
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
    await userEvent.keyboard('{Escape}');
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    answer(
      jsonResponse({ dryRun: false, results: [{ item: { componentId: 'staging-1.0' }, status: 'copied' }] }),
    );
    await screen.findByText('Copied');
  });
});
