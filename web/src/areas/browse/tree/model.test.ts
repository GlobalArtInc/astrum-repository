import { describe, expect, it } from 'vitest';
import type { TreeListing, TreeNode } from '../types';
import {
  ancestorsOf,
  browseUrl,
  commonFolder,
  findNode,
  flatten,
  keyAction,
  type Loaded,
  nodeRows,
  parentOf,
  searchTarget,
  selectionFromPathname,
  selectionFromSplat,
} from './model';

const folder = (path: string, childCount = 1): TreeNode => ({
  kind: 'folder',
  name: path.split('/').filter(Boolean).pop() ?? '',
  path,
  childCount,
});

const asset = (path: string): TreeNode => ({ kind: 'asset', name: path.split('/').pop() ?? '', path });

function listing(path: string, children: TreeNode[], truncated = false): Loaded {
  const value: TreeListing = { repository: 'r', format: 'raw', type: 'hosted', path, truncated, children };
  return { status: 'success', listing: value };
}

const LOADED = new Map<string, Loaded>([
  ['/', listing('/', [folder('/a/', 2), asset('/c.txt')])],
  ['/a/', listing('/a/', [folder('/a/b/'), asset('/a/one.txt')], true)],
]);

describe('tree paths', () => {
  it('reads the selection from the address', () => {
    expect(selectionFromSplat(undefined)).toBe('');
    expect(selectionFromSplat('')).toBe('');
    expect(selectionFromSplat('a/b/c.txt')).toBe('/a/b/c.txt');
    expect(selectionFromSplat('a/b/')).toBe('/a/b/');
    expect(selectionFromSplat('a%20b/c%3Ad')).toBe('/a b/c:d');
    expect(selectionFromPathname('/browse/raw/a/b%2Fc/')).toBe('/a/b/c/');
    expect(selectionFromPathname('/browse/raw')).toBe('');
    expect(selectionFromPathname('/browse/raw/')).toBe('');
  });

  it('finds parents and ancestors of files and folders', () => {
    expect(parentOf('/a/b/c.txt')).toBe('/a/b/');
    expect(parentOf('/a/b/')).toBe('/a/');
    expect(parentOf('/a')).toBe('/');
    expect(ancestorsOf('/a/b/c.txt')).toEqual(['/', '/a/', '/a/b/']);
    expect(ancestorsOf('/a/b')).toEqual(['/', '/a/']);
    expect(ancestorsOf('/c.txt')).toEqual(['/']);
    expect(ancestorsOf('')).toEqual([]);
  });

  it('matches a selection to a file or a folder', () => {
    const root = LOADED.get('/')?.listing;
    expect(findNode(root, '/a')?.path).toBe('/a/');
    expect(findNode(root, '/c.txt')?.kind).toBe('asset');
    expect(findNode(root, '/missing')).toBeUndefined();
    expect(findNode(root, '/a/')?.path).toBe('/a/');
    expect(findNode(root, '/c.txt/')).toBeUndefined();
  });

  it('tells a folder from a file of the same name', () => {
    const value: TreeListing = {
      repository: 'npm',
      format: 'npm',
      type: 'hosted',
      path: '/',
      truncated: false,
      children: [folder('/left-pad/'), asset('/left-pad')],
    };
    expect(findNode(value, '/left-pad')?.kind).toBe('asset');
    expect(findNode(value, '/left-pad/')?.kind).toBe('folder');
  });

  it('builds addresses with encoded segments', () => {
    expect(browseUrl('raw files')).toBe('/browse/raw%20files');
    expect(browseUrl('r', '/v2/app/manifests/sha256:ab')).toBe('/browse/r/v2/app/manifests/sha256%3Aab');
    expect(browseUrl('r', '/a b/c#1.txt')).toBe('/browse/r/a%20b/c%231.txt');
    expect(browseUrl('r', '/a/b/')).toBe('/browse/r/a/b/');
    expect(browseUrl('r', '/')).toBe('/browse/r');
  });
});

describe('flattening the tree', () => {
  it('shows expanded folders with their children and notices', () => {
    const rows = flatten(LOADED, new Set(['/a/', '/a/b/']));
    expect(rows.map((row) => (row.type === 'node' ? row.node.path : `${row.type}:${row.parent}`))).toEqual([
      '/a/',
      '/a/b/',
      'loading:/a/b/',
      '/a/one.txt',
      'truncated:/a/',
      '/c.txt',
    ]);
    const nodes = nodeRows(rows);
    expect(nodes[0]).toMatchObject({ level: 1, position: 1, size: 2, expanded: true });
    expect(nodes[1]).toMatchObject({ level: 2, parent: '/a/', expanded: true });
    expect(nodes[2]?.expanded).toBeUndefined();
  });

  it('marks errors and empty folders', () => {
    const loaded = new Map<string, Loaded>([
      ['/', listing('/', [folder('/a/'), folder('/b/')])],
      ['/a/', { status: 'error', error: new Error('boom') }],
      ['/b/', listing('/b/', [])],
    ]);
    const rows = flatten(loaded, new Set(['/a/', '/b/']));
    expect(rows.map((row) => row.type)).toEqual(['node', 'error', 'node', 'empty']);
  });
});

describe('keyboard navigation', () => {
  const rows = flatten(LOADED, new Set(['/a/']));

  it('moves up and down, to the start and the end', () => {
    expect(keyAction(rows, '/a/', 'ArrowDown')).toEqual({ type: 'focus', path: '/a/b/' });
    expect(keyAction(rows, '/a/b/', 'ArrowUp')).toEqual({ type: 'focus', path: '/a/' });
    expect(keyAction(rows, '/a/', 'ArrowUp')).toBeNull();
    expect(keyAction(rows, '/a/one.txt', 'End')).toEqual({ type: 'focus', path: '/c.txt' });
    expect(keyAction(rows, '/c.txt', 'Home')).toEqual({ type: 'focus', path: '/a/' });
    expect(keyAction(rows, '/c.txt', 'ArrowDown')).toBeNull();
  });

  it('opens and closes folders with the arrows', () => {
    expect(keyAction(rows, '/a/b/', 'ArrowRight')).toEqual({ type: 'expand', path: '/a/b/' });
    expect(keyAction(rows, '/a/', 'ArrowRight')).toEqual({ type: 'focus', path: '/a/b/' });
    expect(keyAction(rows, '/a/', 'ArrowLeft')).toEqual({ type: 'collapse', path: '/a/' });
    expect(keyAction(rows, '/a/one.txt', 'ArrowLeft')).toEqual({ type: 'focus', path: '/a/' });
    expect(keyAction(rows, '/c.txt', 'ArrowRight')).toBeNull();
  });

  it('selects with Enter and Space and focuses the first row when lost', () => {
    expect(keyAction(rows, '/c.txt', 'Enter')).toMatchObject({ type: 'select', node: { path: '/c.txt' } });
    expect(keyAction(rows, '/c.txt', ' ')).toMatchObject({ type: 'select' });
    expect(keyAction(rows, '/gone', 'ArrowDown')).toEqual({ type: 'focus', path: '/a/' });
    expect(keyAction(rows, '/a/', 'x')).toBeNull();
  });
});

describe('search results in the tree', () => {
  it('finds the folder shared by the files of a component', () => {
    expect(commonFolder(['/org/a/lib/1.0/lib-1.0.jar', '/org/a/lib/1.0/lib-1.0.pom'])).toBe(
      '/org/a/lib/1.0/',
    );
    expect(commonFolder(['/a/x.txt', '/b/y.txt'])).toBe('/');
    expect(commonFolder([])).toBe('/');
  });

  it('opens version folders as components and other results as their first file', () => {
    expect(searchTarget('1.0', ['/org/a/lib/1.0/lib-1.0.jar', '/org/a/lib/1.0/lib-1.0.pom'])).toBe(
      '/org/a/lib/1.0/',
    );
    expect(searchTarget('1.0.0', ['/left-pad/-/left-pad-1.0.0.tgz'])).toBe('/left-pad/-/left-pad-1.0.0.tgz');
    expect(searchTarget('', ['/docs/a.txt'])).toBe('/docs/a.txt');
    expect(searchTarget('1.0', [])).toBe('');
  });
});
