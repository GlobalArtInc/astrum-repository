import type { Area } from '../../app/area';

const READ = 'astrum:selectors:read';

export const selectorsArea: Area = {
  id: 'selectors',
  routes: [
    { path: 'admin/repository/selectors', load: () => import('./SelectorsPage'), need: READ },
    {
      path: 'admin/repository/selectors/create',
      load: () => import('./SelectorPage'),
      need: { all: [READ, 'astrum:selectors:create'] },
    },
    { path: 'admin/repository/selectors/:name', load: () => import('./SelectorPage'), need: READ },
  ],
  settings: [
    {
      id: 'selectors',
      to: '/admin/repository/selectors',
      need: READ,
    },
  ],
};
