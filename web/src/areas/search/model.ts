export interface SearchAsset {
  id: string;
  path: string;
  downloadUrl: string;
  repository: string;
  format: string;
  contentType: string | null;
  lastModified: string | null;
  fileSize: number;
  checksum: Record<string, string | null>;
}

export interface SearchComponent {
  id: string;
  repository: string;
  format: string;
  group: string | null;
  name: string;
  version: string | null;
  assets: SearchAsset[];
}

export interface SearchPage<T> {
  items: T[];
  continuationToken: string | null;
}

export interface RepositorySummary {
  name: string;
  format: string;
  type: 'hosted' | 'proxy' | 'group';
  url: string;
}

export const SEARCH = '/service/rest/v1/search';
export const SEARCH_ASSETS = '/service/rest/v1/search/assets';
export const REPOSITORIES = '/service/rest/v1/repositories';

export function componentPath(component: {
  format: string;
  group?: string | null;
  name: string;
  version?: string | null;
}): string {
  const id = encodeURIComponent(`${component.format}:${component.group ?? ''}:${component.name}`);
  const version = component.version ? `?version=${encodeURIComponent(component.version)}` : '';
  return `/search/component/${id}${version}`;
}

export function browsePath(repository: string, path = ''): string {
  const segments = path
    .split('/')
    .filter((segment) => segment !== '')
    .map(encodeURIComponent);
  return ['/browse', encodeURIComponent(repository), ...segments].join('/');
}

export function folderOf(path: string): string {
  const trimmed = path.replace(/\/+$/, '');
  const cut = trimmed.lastIndexOf('/');
  return cut <= 0 ? '' : trimmed.slice(0, cut);
}

export function lastModified(component: SearchComponent): string | null {
  let newest: string | null = null;
  let newestTime = Number.NEGATIVE_INFINITY;
  for (const asset of component.assets) {
    const time = asset.lastModified ? Date.parse(asset.lastModified) : Number.NaN;
    if (asset.lastModified && !Number.isNaN(time) && time > newestTime) {
      newest = asset.lastModified;
      newestTime = time;
    }
  }
  return newest;
}
