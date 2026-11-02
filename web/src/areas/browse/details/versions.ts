import { isPreRelease } from '../tree/model';
import type { TreeNode } from '../types';

export type VersionColumn = 'name' | 'modified' | 'downloads';
export type VersionDirection = 'asc' | 'desc';

export function isVersionNode(node: TreeNode, format: string): boolean {
  return node.kind === 'component' || (node.kind === 'asset' && format === 'docker');
}

function byName(left: TreeNode, right: TreeNode): number {
  return left.name.localeCompare(right.name, undefined, { numeric: true });
}

function compareMain(left: TreeNode, right: TreeNode, column: VersionColumn): number {
  switch (column) {
    case 'name':
      return byName(left, right);
    case 'modified': {
      const before = left.lastModified ?? '';
      const after = right.lastModified ?? '';
      return before < after ? -1 : before > after ? 1 : 0;
    }
    case 'downloads':
      return (left.downloadCount ?? -1) - (right.downloadCount ?? -1);
  }
}

export function sortVersions(
  nodes: TreeNode[],
  column: VersionColumn,
  direction: VersionDirection,
): TreeNode[] {
  const sign = direction === 'asc' ? 1 : -1;
  return [...nodes].sort((left, right) => sign * compareMain(left, right, column) || byName(left, right));
}

export function newestStable(nodes: TreeNode[], format: string): string | undefined {
  let newest: TreeNode | undefined;
  for (const node of nodes) {
    if (isPreRelease(node, format)) {
      continue;
    }
    if (newest === undefined || (node.lastModified ?? '') > (newest.lastModified ?? '')) {
      newest = node;
    }
  }
  return newest?.path;
}
