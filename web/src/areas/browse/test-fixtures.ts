import type { RepositoryItem, TreeListing, TreeNode } from './types';

export const FLAGS = {
  browse: true,
  read: true,
  add: true,
  edit: true,
  delete: true,
  admin: true,
  adminEdit: true,
  adminDelete: true,
};

export function repository(
  name: string,
  format: string,
  type: RepositoryItem['type'],
  extra: Partial<RepositoryItem> = {},
): RepositoryItem {
  return {
    name,
    format,
    type,
    url: `http://localhost/repository/${name}`,
    online: true,
    blobStoreName: 'default',
    remoteUrl: null,
    memberOf: [],
    componentCount: 1,
    assetCount: 1,
    totalSize: 10,
    lastUploaded: '2026-10-01T10:00:00.000+00:00',
    limited: false,
    permissions: FLAGS,
    ...extra,
  };
}

export function folder(path: string, childCount = 1): TreeNode {
  return { kind: 'folder', name: path.split('/').filter(Boolean).pop() ?? '', path, childCount };
}

export function file(path: string, repositoryName = 'files'): TreeNode {
  return {
    kind: 'asset',
    name: path.split('/').pop() ?? '',
    path,
    assetId: `id-${path}`,
    repository: repositoryName,
    size: 5,
    contentType: 'text/plain',
    lastModified: '2026-10-01T10:00:00.000+00:00',
    downloadUrl: `http://localhost/repository/${repositoryName}${path}`,
  };
}

export function listing(name: string, path: string, children: TreeNode[], truncated = false): TreeListing {
  return { repository: name, format: 'raw', type: 'hosted', path, truncated, children };
}
