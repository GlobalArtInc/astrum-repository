

/**
 * Raw Search Types
 *
 * Types for searching raw format repositories.
 */

/**
 * Search filters for raw format.
 */
export interface RawSearchFilters {
  /** Keyword search across all fields */
  keyword?: string;
  /** Filter by group/path prefix */
  group?: string;
  /** Filter by file name */
  name?: string;
  /** Filter by repository */
  repository?: string;
}

/**
 * A single raw search result (file/asset).
 */
export interface RawResult {
  /** Unique identifier */
  readonly id: string;
  /** File path */
  path: string;
  /** File name */
  name: string;
  /** Group/directory path */
  group?: string;
  /** Repository name */
  repository: string;
  /** Content type */
  contentType?: string;
  /** File size in bytes */
  size?: number;
  /** Last modified timestamp */
  lastModified?: string;
  /** Download URL */
  downloadUrl?: string;
  /** Checksum values */
  checksums?: {
    sha1?: string;
    sha256?: string;
    sha512?: string;
    md5?: string;
  };
}

/**
 * Raw search response from API.
 */
export interface RawSearchResponse {
  /** Search results */
  items: RawResult[];
  /** Total count of results */
  totalCount: number;
  /** Token for pagination */
  continuationToken?: string;
}

/**
 * State for raw search hook.
 */
export interface RawSearchState {
  /** Current filter values */
  filters: RawSearchFilters;
  /** Search results */
  results: RawResult[];
  /** Total result count */
  totalCount: number;
  /** Loading state */
  loading: boolean;
  /** Error message if any */
  error?: string;
  /** Pagination token */
  continuationToken?: string;
}


