

/**
 * Yum/RPM Search Types
 *
 * Types specific to Yum (RPM) package search functionality.
 */

/**
 * Common RPM architectures.
 */
export const YUM_ARCHITECTURES = [
  'x86_64',
  'noarch',
  'i686',
  'i386',
  'aarch64',
  'ppc64le',
  's390x',
  'src',
] as const;

export type YumArchitecture = typeof YUM_ARCHITECTURES[number] | string;

/**
 * Yum search filter parameters.
 */
export interface YumSearchFilters {
  /** Package name */
  name?: string;
  /** Package version */
  version?: string;
  /** Architecture (x86_64, noarch, i686, etc.) */
  architecture?: string;
  /** Repository filter */
  repository?: string;
}

/**
 * A single Yum/RPM package result in search results.
 */
export interface YumResult {
  /** Unique identifier */
  readonly id: string;
  /** Package name */
  readonly name: string;
  /** Display name (name-version-release.arch) */
  readonly displayName: string;
  /** Latest version */
  readonly latestVersion: string;
  /** Release string */
  readonly release: string;
  /** Architecture */
  readonly architecture: string;
  /** Total number of versions */
  readonly versionsCount: number;
  /** Package summary */
  readonly summary?: string;
  /** Number of repositories containing this package */
  readonly repositoriesCount: number;
  /** ISO 8601 timestamp of last update */
  readonly lastUpdated: string;
}

/**
 * Yum search response.
 */
export interface YumSearchResponse {
  /** Search results */
  readonly items: readonly YumResult[];
  /** Total count of matching packages */
  readonly totalCount: number;
  /** Continuation token for pagination */
  readonly continuationToken?: string;
}

/**
 * Yum package version info.
 */
export interface YumVersion {
  /** Version string */
  readonly version: string;
  /** Release string */
  readonly release: string;
  /** Full version-release string */
  readonly versionRelease: string;
  /** Architecture */
  readonly architecture: string;
  /** ISO 8601 timestamp */
  readonly published: string;
  /** Repository containing this version */
  readonly repository: string;
  /** Download URL */
  readonly downloadUrl?: string;
}

/**
 * Yum package detail.
 */
export interface YumDetail {
  /** Package ID */
  readonly id: string;
  /** Package name */
  readonly name: string;
  /** Display name */
  readonly displayName: string;
  /** Summary (short description) */
  readonly summary?: string;
  /** Full description */
  readonly description?: string;
  /** License */
  readonly license?: string;
  /** URL */
  readonly url?: string;
  /** Vendor */
  readonly vendor?: string;
  /** Group/category */
  readonly group?: string;
  /** All versions */
  readonly versions: readonly YumVersion[];
  /** Repositories containing this package */
  readonly repositories: readonly string[];
}

/**
 * Yum search state.
 */
export interface YumSearchState {
  /** Current filters */
  readonly filters: YumSearchFilters;
  /** Loading state */
  readonly loading: boolean;
  /** Error message */
  readonly error?: string;
  /** Search results */
  readonly results: readonly YumResult[];
  /** Total count */
  readonly totalCount: number;
  /** Continuation token */
  readonly continuationToken?: string;
}


