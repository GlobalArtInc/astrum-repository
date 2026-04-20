import type { Area } from '../../app/area';
import { CREATE, READ } from './model';

export const tasksArea: Area = {
  id: 'tasks',
  routes: [
    { path: 'admin/system/tasks', load: () => import('./TasksPage'), need: READ },
    {
      path: 'admin/system/tasks/create',
      load: () => import('./CreateTaskPage'),
      need: { all: [READ, CREATE] },
    },
    {
      path: 'admin/system/tasks/create/:type',
      load: () => import('./CreateTaskPage'),
      need: { all: [READ, CREATE] },
    },
    { path: 'admin/system/tasks/:id', load: () => import('./TaskPage'), need: READ },
  ],
  settings: [{ id: 'tasks', to: '/admin/system/tasks', need: READ }],
};
