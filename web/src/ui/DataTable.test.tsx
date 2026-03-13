import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ApiError } from '../api';
import { renderWithApp } from '../test/render';
import { setMedia } from '../test/setup';
import { type Column, DataTable, type DataTableProps } from './DataTable';
import { PHONE } from './hooks';

interface Row {
  name: string;
  size: number;
}

const ROWS: Row[] = [
  { name: 'beta', size: 20 },
  { name: 'alpha', size: 300 },
  { name: 'gamma', size: 1 },
  { name: 'Delta', size: 5 },
  { name: 'epsilon', size: 9 },
];

const COLUMNS: Column<Row>[] = [
  { id: 'name', header: 'Name', cell: (row) => row.name, sortValue: (row) => row.name, primary: true },
  {
    id: 'size',
    header: 'Size',
    cell: (row) => `${row.size} B`,
    sortValue: (row) => row.size,
    align: 'right',
  },
];

function table(props: Partial<DataTableProps<Row>> = {}) {
  return renderWithApp(
    <DataTable label="Things" columns={COLUMNS} rows={ROWS} getRowId={(row) => row.name} {...props} />,
  );
}

function names(): string[] {
  return screen.getAllByRole('rowheader').map((cell) => cell.textContent ?? '');
}

describe('DataTable', () => {
  it('sorts by a column and flips the direction', async () => {
    table({ initialSort: { column: 'name', direction: 'asc' } });
    expect(names()).toEqual(['alpha', 'beta', 'Delta', 'epsilon', 'gamma']);
    await userEvent.click(screen.getByRole('button', { name: 'Size' }));
    expect(names()).toEqual(['gamma', 'Delta', 'epsilon', 'beta', 'alpha']);
    expect(screen.getByRole('columnheader', { name: 'Size' })).toHaveAttribute('aria-sort', 'ascending');
    await userEvent.click(screen.getByRole('button', { name: 'Size' }));
    expect(names()[0]).toBe('alpha');
    expect(screen.getByRole('columnheader', { name: 'Size' })).toHaveAttribute('aria-sort', 'descending');
  });

  it('pages on the client', async () => {
    table({ pageSize: 2, initialSort: { column: 'name', direction: 'asc' } });
    expect(names()).toEqual(['alpha', 'beta']);
    expect(screen.getByText('1–2 of 5')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Previous' })).toBeDisabled();
    await userEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(names()).toEqual(['Delta', 'epsilon']);
    await userEvent.click(screen.getByRole('button', { name: 'Next' }));
    expect(names()).toEqual(['gamma']);
    expect(screen.getByRole('button', { name: 'Next' })).toBeDisabled();
  });

  it('shows more rows on demand', async () => {
    table({ pageSize: 2, paging: 'more' });
    expect(names()).toHaveLength(2);
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(names()).toHaveLength(4);
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(names()).toHaveLength(5);
    expect(screen.queryByRole('button', { name: 'Show more' })).toBeNull();
  });

  it('asks the server for more rows', async () => {
    const more = vi.fn();
    table({ hasMore: true, onLoadMore: more });
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(more).toHaveBeenCalledOnce();
  });

  it('shows skeletons while loading, an empty state and errors', async () => {
    const { unmount } = table({ rows: undefined, loading: true });
    expect(screen.getByRole('table', { name: 'Things' }).closest('[aria-busy]')).toHaveAttribute(
      'aria-busy',
      'true',
    );
    unmount();
    const empty = table({ rows: [], empty: <p>No things yet</p> });
    expect(screen.getByText('No things yet')).toBeInTheDocument();
    empty.unmount();
    const retry = vi.fn();
    table({ rows: undefined, error: new ApiError(500, 'boom'), onRetry: retry });
    expect(screen.getByRole('alert')).toHaveTextContent('The server failed: boom');
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(retry).toHaveBeenCalledOnce();
  });

  it('links rows and offers row actions', async () => {
    const remove = vi.fn();
    table({
      rowHref: (row) => `/things/${row.name}`,
      rowActions: (row) => [{ id: 'delete', label: `Delete ${row.name}`, onSelect: remove, danger: true }],
      initialSort: { column: 'name', direction: 'asc' },
    });
    expect(screen.getByRole('link', { name: 'alpha' })).toHaveAttribute('href', '/things/alpha');
    const [first] = screen.getAllByRole('button', { name: 'Actions' });
    await userEvent.click(first as HTMLElement);
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Delete alpha' }));
    expect(remove).toHaveBeenCalledWith(ROWS[1]);
  });

  it('stays a table on phones', () => {
    setMedia(PHONE, true);
    table({ initialSort: { column: 'name', direction: 'asc' } });
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getAllByRole('row')).toHaveLength(6);
  });

  it('turns rows into cards on phones when asked', () => {
    setMedia(PHONE, true);
    table({ initialSort: { column: 'name', direction: 'asc' }, phoneLayout: 'cards' });
    expect(screen.queryByRole('table')).toBeNull();
    const cards = within(screen.getByRole('region', { name: 'Things' })).getAllByRole('listitem');
    expect(cards).toHaveLength(5);
    expect(cards[0]).toHaveTextContent('alpha');
    expect(cards[0]).toHaveTextContent('Size300 B');
  });
});
