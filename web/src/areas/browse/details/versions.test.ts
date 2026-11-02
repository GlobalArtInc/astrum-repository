import { describe, expect, it } from 'vitest';
import type { TreeNode } from '../types';
import {
  isVersionNode,
  newestStable,
  sortVersions,
  type VersionColumn,
  type VersionDirection,
} from './versions';

function version(name: string, extra: Partial<TreeNode> = {}): TreeNode {
  return { kind: 'component', name, path: `/app/${name}/`, ...extra };
}

const NODES = [
  version('1.0.0', { lastModified: '2026-01-02T00:00:00.000Z', downloadCount: 5 }),
  version('0.2.9', { lastModified: '2026-03-01T00:00:00.000Z', downloadCount: 50 }),
  version('0.2.10', { lastModified: '2026-02-01T00:00:00.000Z', downloadCount: 1 }),
];

function names(nodes: TreeNode[]): string[] {
  return nodes.map((node) => node.name);
}

type SortCase = [VersionColumn, VersionDirection, string[]];

describe('version helpers', () => {
  it('takes components, and docker assets only for docker', () => {
    expect(isVersionNode(version('1.0.0'), 'maven2')).toBe(true);
    expect(isVersionNode({ kind: 'asset', name: 'latest', path: '/app/latest' }, 'docker')).toBe(true);
    expect(isVersionNode({ kind: 'asset', name: 'a.txt', path: '/a.txt' }, 'raw')).toBe(false);
    expect(isVersionNode({ kind: 'folder', name: 'a', path: '/a/' }, 'docker')).toBe(false);
  });

  it.each<SortCase>([
    ['name', 'asc', ['0.2.9', '0.2.10', '1.0.0']],
    ['name', 'desc', ['1.0.0', '0.2.10', '0.2.9']],
    ['modified', 'asc', ['1.0.0', '0.2.10', '0.2.9']],
    ['modified', 'desc', ['0.2.9', '0.2.10', '1.0.0']],
    ['downloads', 'asc', ['0.2.10', '1.0.0', '0.2.9']],
    ['downloads', 'desc', ['0.2.9', '1.0.0', '0.2.10']],
  ])('sorts by %s %s, comparing names in natural order', (column, direction, expected) => {
    expect(names(sortVersions(NODES, column, direction))).toEqual(expected);
  });

  it('keeps name ascending as the tie-break when the direction is descending', () => {
    const tied = [
      version('b', { downloadCount: 1 }),
      version('a', { downloadCount: 1 }),
      version('c', { downloadCount: 2 }),
    ];
    expect(names(sortVersions(tied, 'downloads', 'desc'))).toEqual(['c', 'a', 'b']);
  });

  it('puts missing dates last when descending', () => {
    const undated = version('0.0.1');
    const sorted = sortVersions([undated, ...NODES], 'modified', 'desc');
    expect(names(sorted).at(-1)).toBe('0.0.1');
    expect(names(sortVersions([undated, ...NODES], 'modified', 'asc'))[0]).toBe('0.0.1');
  });

  it('finds the newest stable version and skips pre-releases', () => {
    const nodes = [
      version('1.0.0', { lastModified: '2026-01-02T00:00:00.000Z' }),
      version('1.1.0-rc1', { lastModified: '2026-03-01T00:00:00.000Z' }),
    ];
    expect(newestStable(nodes, 'maven2')).toBe('/app/1.0.0/');
    expect(newestStable([version('2.0.0-rc1')], 'maven2')).toBeUndefined();
  });
});
