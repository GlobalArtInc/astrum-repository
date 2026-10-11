

import localize from '../../../../../interface/localize';

export const GA_REPOSITORIES_STRINGS = localize('react-ui', {
  FILTER_PLACEHOLDER: 'Filter',
  CLEAR_SEARCH_ARIA: 'Clear search',
  FILTER_BUTTON: 'Filter',
  RESET_FILTERS: 'Reset filters',
  EXPORT_CSV: 'Export CSV',
  EXPORT_CSV_ARIA: 'Export all filtered results as CSV',
  EXPORT_FILENAME: 'repositories.csv',
  FILTERS: {
    SORT_LABEL: 'Sort',
    TYPE_LABEL: 'Type',
    SORT_ASC: 'Ascending',
    SORT_DESC: 'Descending',
  },
  TYPE_OPTIONS: [
    { value: 'hosted', label: 'Hosted' },
    { value: 'proxy', label: 'Proxy' },
    { value: 'group', label: 'Group' },
  ],
  COLUMNS: {
    REPOSITORY: 'Repository',
    TYPE: 'Type',
    VERSIONS_IN_REPO: 'Versions in Repo',
  },
  EMPTY: {
    // Kept separate from NO_RESULTS_TITLE: this fires when the API returned zero rows
    // (no repositories contain the selected version); NO_RESULTS_TITLE fires when the
    // user's client-side search/filter narrowed a non-empty result set to zero. The two
    // states may diverge in copy later — keep the keys distinct.
    NO_REPOS_TITLE: 'No repositories found',
    NO_REPOS_FOR_VERSION: (version: string) => `No repositories contain version ${version}.`,
    NO_REPOS_FOR_COMPONENT: 'No repositories are available for this component.',
    NO_RESULTS_TITLE: 'No repositories found',
    NO_RESULTS_DETAIL: 'Try adjusting your search terms or filters.',
  },
  VERSIONS_BADGE: {
    ONE: (count: number) => `${count} version`,
    FEW: (count: number) => `${count} versions`,
    MANY: (count: number) => `${count} versions`,
    OTHER: (count: number) => `${count} versions`,
  },
}, 'PREVIEW.COMPONENT_DETAIL.REPOSITORIES');
