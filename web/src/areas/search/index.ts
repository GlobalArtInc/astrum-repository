import { Search } from 'lucide-react';
import type { Area } from '../../app/area';
import { HeaderSearchResults, submitSearch } from './HeaderSearch';

export const SEARCH_NEED = 'astrum:search:read';

export const searchArea: Area = {
  id: 'search',
  routes: [
    { path: 'search', load: () => import('./SearchPage'), need: SEARCH_NEED },
    { path: 'search/custom', load: () => import('./CriteriaPage'), need: SEARCH_NEED },
  ],
  menu: [
    {
      id: 'search',
      to: '/search',
      label: 'search.menu.label',
      icon: Search,
      group: 'main',
      order: 20,
      need: SEARCH_NEED,
    },
  ],
  search: [
    {
      id: 'search',
      order: 10,
      need: SEARCH_NEED,
      Results: HeaderSearchResults,
      submit: submitSearch,
    },
  ],
};
