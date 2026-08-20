

/**
 * PyPI Search Types
 *
 * Types specific to PyPI (Python Package Index) package search functionality.
 */

/**
 * PyPI search filter parameters.
 * Based on SearchPypiExt.jsx criteria:
 * - classifiers
 * - description
 * - keywords
 * - summary
 */
export interface PyPISearchFilters {
  /** Package name */
  name?: string;
  /** Package version */
  version?: string;
  /** Package summary/short description */
  summary?: string;
  /** Package description/long description */
  description?: string;
  /** Package keywords */
  keywords?: string;
  /** Trove classifiers (e.g., "Development Status :: 5 - Production/Stable") */
  classifiers?: string;
  /** Repository filter */
  repository?: string;
}

/**
 * A single PyPI package result in search results.
 */
export interface PyPIResult {
  /** Unique identifier */
  readonly id: string;
  /** Package name */
  readonly name: string;
  /** Display name */
  readonly displayName: string;
  /** Latest published version */
  readonly latestVersion: string;
  /** Total number of versions */
  readonly versionsCount: number;
  /** Package summary (short description) */
  readonly summary?: string;
  /** Package author */
  readonly author?: string;
  /** Package license */
  readonly license?: string;
  /** Keywords */
  readonly keywords?: readonly string[];
  /** Classifiers */
  readonly classifiers?: readonly string[];
  /** Number of repositories containing this package */
  readonly repositoriesCount: number;
  /** ISO 8601 timestamp of last update */
  readonly lastUpdated: string;
}

/**
 * PyPI search response.
 */
export interface PyPISearchResponse {
  /** Search results */
  readonly items: readonly PyPIResult[];
  /** Total count of matching packages */
  readonly totalCount: number;
  /** Continuation token for pagination */
  readonly continuationToken?: string;
}

/**
 * PyPI package version info.
 */
export interface PyPIVersion {
  /** Version string (PEP 440) */
  readonly version: string;
  /** ISO 8601 timestamp */
  readonly published: string;
  /** Repository containing this version */
  readonly repository: string;
  /** Python version requirements */
  readonly requiresPython?: string;
}

/**
 * PyPI package detail.
 */
export interface PyPIDetail {
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
  /** Author info */
  readonly author?: string;
  /** Author email */
  readonly authorEmail?: string;
  /** License */
  readonly license?: string;
  /** Homepage URL */
  readonly homepage?: string;
  /** Project URL */
  readonly projectUrl?: string;
  /** Keywords */
  readonly keywords?: readonly string[];
  /** Classifiers */
  readonly classifiers?: readonly string[];
  /** All versions */
  readonly versions: readonly PyPIVersion[];
  /** Repositories containing this package */
  readonly repositories: readonly string[];
  /** Required Python version */
  readonly requiresPython?: string;
}

/**
 * PyPI search state.
 */
export interface PyPISearchState {
  /** Current filters */
  readonly filters: PyPISearchFilters;
  /** Loading state */
  readonly loading: boolean;
  /** Error message */
  readonly error?: string;
  /** Search results */
  readonly results: readonly PyPIResult[];
  /** Total count */
  readonly totalCount: number;
  /** Continuation token */
  readonly continuationToken?: string;
}


