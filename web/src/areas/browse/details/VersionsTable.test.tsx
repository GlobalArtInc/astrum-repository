import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../../../test/render';
import { repository } from '../test-fixtures';
import type { TreeNode } from '../types';
import { VersionsTable } from './VersionsTable';

function tag(name: string, lastModified: string, downloadCount: number): TreeNode {
  return { kind: 'asset', name, path: `/app/${name}`, lastModified, downloadCount };
}

const TAGS = [
  tag('0.9.0', '2025-12-01T00:00:00.000Z', 50),
  tag('1.0.0', '2026-01-02T00:00:00.000Z', 5),
  tag('1.1.0-rc1', '2026-03-01T00:00:00.000Z', 1),
];

function renderTable(nodes: TreeNode[] = TAGS) {
  return renderWithApp(
    <VersionsTable repository={repository('images', 'docker', 'hosted')} nodes={nodes} truncated={false} />,
  );
}

function names(): string[] {
  return within(screen.getByRole('table'))
    .getAllByRole('link')
    .map((link) => link.textContent ?? '');
}

function rowOf(name: string): HTMLElement {
  return screen.getByRole('link', { name }).closest('tr') as HTMLElement;
}

describe('the versions table', () => {
  it('lists the newest tags first by default', () => {
    renderTable();
    expect(names()).toEqual(['1.1.0-rc1', '1.0.0', '0.9.0']);
  });

  it('puts the Newest badge on the newest stable tag, not on a newer pre-release', () => {
    renderTable();
    expect(within(rowOf('1.0.0')).getByText('Newest')).toBeInTheDocument();
    expect(within(rowOf('1.1.0-rc1')).queryByText('Newest')).toBeNull();
  });

  it('hides the pre-releases when asked', async () => {
    renderTable();
    await userEvent.click(screen.getByRole('switch', { name: 'Hide pre-releases' }));
    expect(names()).toEqual(['1.0.0', '0.9.0']);
  });

  it('sorts by the tag name when the Tag header is clicked', async () => {
    renderTable();
    await userEvent.click(screen.getByRole('button', { name: 'Tag' }));
    expect(names()).toEqual(['0.9.0', '1.0.0', '1.1.0-rc1']);
    expect(screen.getByRole('columnheader', { name: 'Tag' })).toHaveAttribute('aria-sort', 'ascending');
  });

  it('sorts the pulls from most to fewest on the first click of the Pulls header', async () => {
    renderTable();
    await userEvent.click(screen.getByRole('button', { name: 'Pulls' }));
    expect(names()).toEqual(['0.9.0', '1.0.0', '1.1.0-rc1']);
    expect(screen.getByRole('columnheader', { name: 'Pulls' })).toHaveAttribute('aria-sort', 'descending');
  });

  it('narrows the rows to the tags that match the filter', async () => {
    renderTable();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter the list of versions' }), '0.9');
    expect(names()).toEqual(['0.9.0']);
  });
});
