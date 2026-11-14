import type { Area } from '../../app/area';
import { READ } from './api';

export const replicationArea: Area = {
  id: 'replication',
  routes: [
    { path: 'admin/replication', load: () => import('./ReplicationPage'), need: READ },
    {
      path: 'admin/replication/replications/:id',
      load: () => import('./ReplicationDetailPage'),
      need: READ,
    },
  ],
  settings: [{ id: 'replication', to: '/admin/replication', need: READ }],
};
