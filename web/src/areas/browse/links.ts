import type { RepositoryItem } from './types';

export function componentKey(format: string, group: string, name: string): string {
  return `${format}:${group}:${name}`;
}

export function parseComponentKey(key: string): { format: string; group: string; name: string } | null {
  const first = key.indexOf(':');
  if (first <= 0) {
    return null;
  }
  const second = key.indexOf(':', first + 1);
  if (second < 0) {
    return null;
  }
  const name = key.slice(second + 1);
  if (name === '') {
    return null;
  }
  return { format: key.slice(0, first), group: key.slice(first + 1, second), name };
}

export function componentPageUrl(
  format: string,
  group: string,
  name: string,
  version?: string,
  tab?: string,
): string {
  const params = new URLSearchParams();
  if (version) {
    params.set('version', version);
  }
  if (tab) {
    params.set('tab', tab);
  }
  const query = params.toString();
  return `/search/component/${encodeURIComponent(componentKey(format, group, name))}${query ? `?${query}` : ''}`;
}

export function componentTitle(format: string, group: string, name: string): string {
  if (group === '' || format === 'raw') {
    return name;
  }
  if (format === 'npm') {
    return `@${group.replace(/^@/, '')}/${name}`;
  }
  return `${group}:${name}`;
}

export function guideComponentName(format: string, group: string, name: string): string {
  if (group === '') {
    return name;
  }
  switch (format) {
    case 'maven2':
      return `${group}:${name}`;
    case 'npm':
      return `@${group.replace(/^@/, '')}/${name}`;
    case 'composer':
      return `${group}/${name}`;
    default:
      return name;
  }
}

export function fromProxy(
  repositories: Pick<RepositoryItem, 'name' | 'type'>[] | undefined,
  names: (string | undefined)[],
): boolean {
  return names.some(
    (name) =>
      name !== undefined && (repositories ?? []).some((item) => item.name === name && item.type === 'proxy'),
  );
}
