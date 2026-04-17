import type { Area } from '../../app/area';
import { CREATE, READ } from './model';

export const cleanupArea: Area = {
  id: 'cleanup',
  routes: [
    { path: 'admin/repository/cleanup-policies', load: () => import('./CleanupPoliciesPage'), need: READ },
    {
      path: 'admin/repository/cleanup-policies/create',
      load: () => import('./CleanupPolicyPage'),
      need: { all: [READ, CREATE] },
    },
    {
      path: 'admin/repository/cleanup-policies/:name',
      load: () => import('./CleanupPolicyPage'),
      need: READ,
    },
  ],
  settings: [{ id: 'cleanup-policies', to: '/admin/repository/cleanup-policies', need: READ }],
};
