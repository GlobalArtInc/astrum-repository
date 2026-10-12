import type { TreeListing, TreeNode } from '../types';

export const ROOT = '/';

export interface Loaded {
  status: 'pending' | 'error' | 'success';
  listing?: TreeListing;
  error?: unknown;
}

export type Row =
  | {
      type: 'node';
      node: TreeNode;
      level: number;
      parent: string;
      position: number;
      size: number;
      expanded: boolean | undefined;
    }
  | { type: 'loading'; level: number; parent: string }
  | { type: 'error'; level: number; parent: string; error: unknown }
  | { type: 'truncated'; level: number; parent: string; shown: number }
  | { type: 'empty'; level: number; parent: string };

export function isBranch(node: TreeNode): boolean {
  return node.kind !== 'asset';
}

function decodeSegment(segment: string): string {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

export function selectionFromSplat(splat: string | undefined): string {
  const segments = (splat ?? '').split('/').filter((segment) => segment !== '');
  if (segments.length === 0) {
    return '';
  }
  const folder = (splat ?? '').endsWith('/');
  return `/${segments.map(decodeSegment).join('/')}${folder ? '/' : ''}`;
}

export function selectionFromPathname(pathname: string): string {
  const parts = pathname.split('/');
  if (parts[1] !== 'browse' || parts.length < 3) {
    return '';
  }
  return selectionFromSplat(parts.slice(3).join('/'));
}

export function folderOf(path: string): string {
  return path.endsWith('/') ? path : `${path}/`;
}

export function parentOf(path: string): string {
  const trimmed = path.endsWith('/') ? path.slice(0, -1) : path;
  const index = trimmed.lastIndexOf('/');
  return index <= 0 ? ROOT : trimmed.slice(0, index + 1);
}

export function ancestorsOf(path: string): string[] {
  if (path === '' || path === ROOT) {
    return [];
  }
  const ancestors: string[] = [];
  let current = parentOf(path);
  while (true) {
    ancestors.unshift(current);
    if (current === ROOT) {
      return ancestors;
    }
    current = parentOf(current);
  }
}

export function sameNode(node: TreeNode, selection: string): boolean {
  if (selection === '') {
    return false;
  }
  return node.path === selection || (!selection.endsWith('/') && node.path === `${selection}/`);
}

export function findNode(listing: TreeListing | undefined, selection: string): TreeNode | undefined {
  if (!listing || selection === '') {
    return undefined;
  }
  return (
    listing.children.find((node) => node.path === selection) ??
    (selection.endsWith('/') ? undefined : listing.children.find((node) => node.path === `${selection}/`))
  );
}

export function browseUrl(repository: string, path = ''): string {
  const segments = path
    .split('/')
    .filter((segment) => segment !== '')
    .map((segment) => encodeURIComponent(segment));
  const base = `/browse/${encodeURIComponent(repository)}`;
  if (segments.length === 0) {
    return base;
  }
  return `${base}/${segments.join('/')}${path.endsWith('/') ? '/' : ''}`;
}

export function flatten(
  loaded: ReadonlyMap<string, Loaded>,
  expanded: ReadonlySet<string>,
  folder = ROOT,
  level = 1,
): Row[] {
  const state = loaded.get(folder);
  if (!state || state.status === 'pending') {
    return [{ type: 'loading', level, parent: folder }];
  }
  if (state.status === 'error' || !state.listing) {
    return [{ type: 'error', level, parent: folder, error: state.error }];
  }
  const { children, truncated } = state.listing;
  if (children.length === 0) {
    return [{ type: 'empty', level, parent: folder }];
  }
  const rows: Row[] = [];
  children.forEach((node, index) => {
    const open = isBranch(node) ? expanded.has(node.path) : undefined;
    rows.push({
      type: 'node',
      node,
      level,
      parent: folder,
      position: index + 1,
      size: children.length,
      expanded: open,
    });
    if (open) {
      rows.push(...flatten(loaded, expanded, node.path, level + 1));
    }
  });
  if (truncated) {
    rows.push({ type: 'truncated', level, parent: folder, shown: children.length });
  }
  return rows;
}

export type TreeAction =
  | { type: 'focus'; path: string }
  | { type: 'expand'; path: string }
  | { type: 'collapse'; path: string }
  | { type: 'select'; node: TreeNode };

type NodeRow = Extract<Row, { type: 'node' }>;

export function nodeRows(rows: readonly Row[]): NodeRow[] {
  return rows.filter((row): row is NodeRow => row.type === 'node');
}

export function keyAction(rows: readonly Row[], focused: string, key: string): TreeAction | null {
  const nodes = nodeRows(rows);
  if (nodes.length === 0) {
    return null;
  }
  const index = nodes.findIndex((row) => row.node.path === focused);
  const current = nodes[index];
  const first = nodes[0];
  const last = nodes[nodes.length - 1];
  if (!current || !first || !last) {
    return first ? { type: 'focus', path: first.node.path } : null;
  }
  switch (key) {
    case 'ArrowDown': {
      const next = nodes[index + 1];
      return next ? { type: 'focus', path: next.node.path } : null;
    }
    case 'ArrowUp': {
      const previous = nodes[index - 1];
      return previous ? { type: 'focus', path: previous.node.path } : null;
    }
    case 'Home':
      return { type: 'focus', path: first.node.path };
    case 'End':
      return { type: 'focus', path: last.node.path };
    case 'ArrowRight': {
      if (current.expanded === false) {
        return { type: 'expand', path: current.node.path };
      }
      const child = nodes[index + 1];
      if (current.expanded && child && child.parent === current.node.path) {
        return { type: 'focus', path: child.node.path };
      }
      return null;
    }
    case 'ArrowLeft': {
      if (current.expanded) {
        return { type: 'collapse', path: current.node.path };
      }
      const parent = nodes.find((row) => row.node.path === current.parent);
      return parent ? { type: 'focus', path: parent.node.path } : null;
    }
    case 'Enter':
    case ' ':
      return { type: 'select', node: current.node };
    default:
      return null;
  }
}

export function commonFolder(paths: readonly string[]): string {
  if (paths.length === 0) {
    return ROOT;
  }
  const split = paths.map((path) => parentOf(path).split('/').filter(Boolean));
  const shortest = Math.min(...split.map((parts) => parts.length));
  const common: string[] = [];
  for (let index = 0; index < shortest; index += 1) {
    const part = split[0]?.[index];
    if (part === undefined || split.some((parts) => parts[index] !== part)) {
      break;
    }
    common.push(part);
  }
  return common.length === 0 ? ROOT : `/${common.join('/')}/`;
}

export function searchTarget(version: string, paths: readonly string[]): string {
  const folder = commonFolder(paths);
  const segments = folder.split('/').filter(Boolean);
  if (version !== '' && segments[segments.length - 1] === version && paths.length > 0) {
    return folder;
  }
  return paths[0] ?? '';
}
