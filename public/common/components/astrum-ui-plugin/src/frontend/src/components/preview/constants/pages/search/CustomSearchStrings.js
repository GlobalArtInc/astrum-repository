import localize from '../../../../../interface/localize';

/**
 * Texts of the custom search: components found by several criteria at once.
 */
export default localize('react-ui', {
  TITLE: 'Custom search',
  SUBTITLE: 'Find components by several criteria at once',
  CRITERIA: 'Search criteria',
  CRITERIA_DESCRIPTION: 'Components must match every criterion.',
  ADD_FILTER: 'Add criterion',
  CLEAR_ALL: 'Clear all',
  SEARCH: 'Search',
  SEARCHING: 'Searching…',
  REMOVE_FILTER: 'Remove criterion',
  ONLY_FILTER: 'At least one criterion is required',
  FIELD_LABEL: 'Field',
  OPERATOR_LABEL: 'Condition',
  VALUE_LABEL: 'Value',
  VALUE_PLACEHOLDER: 'Enter a value…',
  EXAMPLE: (example) => `e.g., ${example}`,
  FIELDS: {
    KEYWORD: 'Keyword',
    FORMAT: 'Format',
    REPOSITORY: 'Repository',
    GROUP: 'Group',
    NAME: 'Name',
    VERSION: 'Version',
    TAG: 'Tag',
  },
  OPERATORS: {
    EQUALS: 'Is',
    STARTS_WITH: 'Starts with',
  },
  START_TITLE: 'Build your search',
  START_DESCRIPTION: 'Add criteria above and click Search to find components.',
  EMPTY_TITLE: 'No components found',
  EMPTY_DESCRIPTION: 'Try other criteria.',
  FOUND: {
    ONE: (count) => `${count} component found`,
    FEW: (count) => `${count} components found`,
    MANY: (count) => `${count} components found`,
    OTHER: (count) => `${count} components found`,
  },
  COLUMNS: {
    FORMAT: 'Format',
    NAME: 'Name',
    GROUP: 'Group',
    VERSION: 'Version',
    REPOSITORY: 'Repository',
    LAST_MODIFIED: 'Last modified',
  },
  OPEN: (name) => `Open ${name} in its repository`,
  LOAD_MORE: 'Load more',
  LOADING_MORE: 'Loading more results…',
  SEARCH_FAILED: 'Search failed',
  LOAD_MORE_FAILED: 'Failed to load more results',
}, 'PREVIEW.CUSTOM_SEARCH');
