

import type { RawResult, RawSearchFilters, RawSearchResponse } from './raw.types';

/**
 * Mock raw search results for development and testing.
 */
export const mockRawResults: RawResult[] = [
  {
    id: 'raw-1',
    path: '/docs/readme.md',
    name: 'readme.md',
    group: 'docs',
    repository: 'raw-hosted',
    contentType: 'text/markdown',
    size: 2048,
    lastModified: '2024-01-15T10:30:00Z',
    downloadUrl: '/repository/raw-hosted/docs/readme.md',
    checksums: {
      sha1: 'abc123def456',
      md5: '789xyz',
    },
  },
  {
    id: 'raw-2',
    path: '/binaries/app-1.0.0.tar.gz',
    name: 'app-1.0.0.tar.gz',
    group: 'binaries',
    repository: 'raw-hosted',
    contentType: 'application/gzip',
    size: 15728640,
    lastModified: '2024-01-14T08:00:00Z',
    downloadUrl: '/repository/raw-hosted/binaries/app-1.0.0.tar.gz',
    checksums: {
      sha256: 'sha256hash123',
    },
  },
  {
    id: 'raw-3',
    path: '/config/settings.json',
    name: 'settings.json',
    group: 'config',
    repository: 'raw-hosted',
    contentType: 'application/json',
    size: 512,
    lastModified: '2024-01-13T16:45:00Z',
    downloadUrl: '/repository/raw-hosted/config/settings.json',
  },
  {
    id: 'raw-4',
    path: '/scripts/deploy.sh',
    name: 'deploy.sh',
    group: 'scripts',
    repository: 'raw-hosted',
    contentType: 'application/x-sh',
    size: 1024,
    lastModified: '2024-01-12T12:00:00Z',
    downloadUrl: '/repository/raw-hosted/scripts/deploy.sh',
  },
  {
    id: 'raw-5',
    path: '/images/logo.png',
    name: 'logo.png',
    group: 'images',
    repository: 'raw-proxy',
    contentType: 'image/png',
    size: 32768,
    lastModified: '2024-01-11T09:15:00Z',
    downloadUrl: '/repository/raw-proxy/images/logo.png',
  },
];

/**
 * Mock detail for a single raw file.
 */
export const mockRawDetail: RawResult = {
  id: 'raw-1',
  path: '/docs/readme.md',
  name: 'readme.md',
  group: 'docs',
  repository: 'raw-hosted',
  contentType: 'text/markdown',
  size: 2048,
  lastModified: '2024-01-15T10:30:00Z',
  downloadUrl: '/repository/raw-hosted/docs/readme.md',
  checksums: {
    sha1: 'abc123def456',
    sha256: 'sha256fullhash',
    md5: '789xyz',
  },
};

/**
 * Mock search API for development.
 */
export async function mockRawSearchApi(
  filters: RawSearchFilters,
  continuationToken?: string
): Promise<RawSearchResponse> {
  // Simulate network delay
  await new Promise((resolve) => setTimeout(resolve, 300));

  let results = [...mockRawResults];

  // Apply filters
  if (filters.keyword) {
    const kw = filters.keyword.toLowerCase();
    results = results.filter(
      (r) =>
        r.name.toLowerCase().includes(kw) ||
        r.path.toLowerCase().includes(kw) ||
        r.group?.toLowerCase().includes(kw)
    );
  }
  if (filters.name) {
    const name = filters.name.toLowerCase();
    results = results.filter((r) => r.name.toLowerCase().includes(name));
  }
  if (filters.group) {
    const group = filters.group.toLowerCase();
    results = results.filter((r) => r.group?.toLowerCase().includes(group));
  }
  if (filters.repository) {
    results = results.filter((r) => r.repository === filters.repository);
  }

  return {
    items: results,
    totalCount: results.length,
    continuationToken: undefined,
  };
}

/**
 * Mock detail API for development.
 */
export async function mockRawDetailApi(id: string): Promise<RawResult> {
  await new Promise((resolve) => setTimeout(resolve, 200));
  return mockRawDetail;
}


