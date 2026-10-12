import { Database } from 'lucide-react';
import type { Area } from '../../app/area';

const ADMIN = { some: 'astrum:repository-admin' } as const;

export const repositoriesArea: Area = {
  id: 'repositories',
  routes: [
    { path: 'admin/repository/repositories', load: () => import('./pages/RepositoriesPage'), need: ADMIN },
    {
      path: 'admin/repository/repositories/create',
      load: () => import('./pages/CreatePickerPage'),
      need: { some: 'astrum:repository-admin:*:*:add' },
    },
    {
      path: 'admin/repository/repositories/create/:format/:type',
      load: () => import('./pages/CreateRepositoryPage'),
      need: { some: 'astrum:repository-admin:*:*:add' },
    },
    {
      path: 'admin/repository/repositories/:name',
      load: () => import('./pages/RepositoryPage'),
      need: ADMIN,
    },
    {
      path: 'admin/repository/repositories/:name/profile',
      load: () => import('./pages/ProfileRedirect'),
      need: ADMIN,
    },
  ],
  settings: [
    {
      id: 'repositories',
      section: 'repository',
      to: '/admin/repository/repositories',
      title: 'repositories.tile.title',
      description: 'repositories.tile.description',
      icon: Database,
      order: 10,
      need: ADMIN,
    },
  ],
};
