

/**
 * Custom Search Types
 *
 * Types for building dynamic search queries with user-defined filters.
 */
import S from '../../../constants/pages/search/CustomSearchStrings';

/**
 * Available filter fields for custom search.
 */
export type FilterField =
  | 'format'
  | 'repository'
  | 'group'
  | 'name'
  | 'version'
  | 'tag'
  | 'keyword';

/**
 * Available filter operators for custom search. The search refuses a leading wildcard, so a value can only be matched
 * whole or by its start.
 */
export type FilterOperator =
  | 'equals'
  | 'startsWith';

/**
 * A single filter criterion in the custom search builder.
 */
export interface CustomFilter {
  /** Unique identifier for React key and state management */
  readonly id: string;
  /** The field to filter on */
  field: FilterField;
  /** The comparison operator */
  operator: FilterOperator;
  /** The filter value */
  value: string;
}

/**
 * Configuration for a filter field option.
 */
export interface FilterFieldOption {
  /** Field value */
  readonly value: FilterField;
  /** Display label */
  readonly label: string;
  /** Placeholder text for value input */
  readonly placeholder: string;
}

/**
 * Configuration for a filter operator option.
 */
export interface FilterOperatorOption {
  /** Operator value */
  readonly value: FilterOperator;
  /** Display label */
  readonly label: string;
}

/**
 * Custom search result item.
 * Generic result that works across all formats.
 */
export interface CustomSearchResult {
  /** Unique identifier */
  readonly id: string;
  /** Repository name */
  readonly repository: string;
  /** Format (maven2, npm, docker, etc.) */
  readonly format: string;
  /** Group/namespace */
  readonly group?: string;
  /** Name/artifact ID */
  readonly name: string;
  /** Version */
  readonly version: string;
  /** ISO 8601 timestamp of the first file, when known */
  readonly lastModified?: string;
  /** Path of the first file of the component in its repository */
  readonly path?: string;
  /** Tags if applicable */
  readonly tags?: readonly string[];
}

/**
 * Custom search response from API.
 */
export interface CustomSearchResponse {
  /** Search results */
  readonly items: readonly CustomSearchResult[];
  /** Total count of matching items */
  readonly totalCount: number;
  /** Continuation token for pagination */
  readonly continuationToken?: string;
}

/**
 * Custom search state.
 */
export interface CustomSearchState {
  /** Array of filter criteria */
  readonly filters: readonly CustomFilter[];
  /** Search results */
  readonly results: readonly CustomSearchResult[];
  /** Total count of results */
  readonly totalCount: number;
  /** Whether search is in progress */
  readonly loading: boolean;
  /** Error message if any */
  readonly error?: string;
  /** Continuation token for pagination */
  readonly continuationToken?: string;
}

/**
 * Available filter field options.
 */
export const FILTER_FIELD_OPTIONS: readonly FilterFieldOption[] = [
  { value: 'keyword', label: S.FIELDS.KEYWORD, placeholder: S.EXAMPLE('spring-boot') },
  { value: 'format', label: S.FIELDS.FORMAT, placeholder: S.EXAMPLE('maven2, npm, docker') },
  { value: 'repository', label: S.FIELDS.REPOSITORY, placeholder: S.EXAMPLE('maven-central') },
  { value: 'group', label: S.FIELDS.GROUP, placeholder: S.EXAMPLE('org.apache.commons') },
  { value: 'name', label: S.FIELDS.NAME, placeholder: S.EXAMPLE('commons-lang3') },
  { value: 'version', label: S.FIELDS.VERSION, placeholder: S.EXAMPLE('3.14.0') },
  { value: 'tag', label: S.FIELDS.TAG, placeholder: S.EXAMPLE('latest') },
];

/**
 * The fields offered by an edition: tags exist in the PRO edition only.
 */
export function filterFieldOptions(proEdition: boolean): readonly FilterFieldOption[] {
  return proEdition ? FILTER_FIELD_OPTIONS : FILTER_FIELD_OPTIONS.filter((option) => option.value !== 'tag');
}

/**
 * Available filter operator options.
 */
export const FILTER_OPERATOR_OPTIONS: readonly FilterOperatorOption[] = [
  { value: 'equals', label: S.OPERATORS.EQUALS },
  { value: 'startsWith', label: S.OPERATORS.STARTS_WITH },
];

/**
 * Create a new empty filter with a unique ID.
 */
export function createEmptyFilter(): CustomFilter {
  return {
    id: `filter-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
    field: 'keyword',
    operator: 'equals',
    value: '',
  };
}


