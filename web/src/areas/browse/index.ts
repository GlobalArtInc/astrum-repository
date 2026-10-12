import { FolderOpen } from 'lucide-react';
import type { Area } from '../../app/area';
import type { Need } from '../../auth/permissions';

export const BROWSE_NEED: Need = {
  any: [{ some: 'astrum:repository-view' }, { some: 'astrum:repository-content-selector' }],
};

export const browseArea: Area = {
  id: 'browse',
  routes: [
    { path: 'browse', load: () => import('./list/RepositoryListPage'), need: BROWSE_NEED },
    { path: 'browse/:repository/*', load: () => import('./RepositoryPage'), need: BROWSE_NEED },
    {
      path: 'search/component/*',
      load: () => import('./component/ComponentPage'),
      need: 'astrum:search:read',
    },
  ],
  menu: [
    {
      id: 'browse',
      to: '/browse',
      label: 'browse.menu.label',
      icon: FolderOpen,
      group: 'main',
      order: 30,
      need: BROWSE_NEED,
    },
  ],
};
