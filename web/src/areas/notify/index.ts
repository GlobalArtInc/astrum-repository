import type { Area } from '../../app/area';
import { CREATE as CAPABILITY_CREATE, READ as CAPABILITY_READ } from './capabilities/model';
import { READ } from './notifications/model';

export const notifyArea: Area = {
  id: 'notify',
  routes: [
    {
      path: 'admin/system/notifications',
      load: () => import('./notifications/NotificationsPage'),
      need: READ,
    },
    {
      path: 'admin/system/capabilities',
      load: () => import('./capabilities/CapabilitiesPage'),
      need: CAPABILITY_READ,
    },
    {
      path: 'admin/system/capabilities/create',
      load: () => import('./capabilities/CreateCapabilityPage'),
      need: { all: [CAPABILITY_READ, CAPABILITY_CREATE] },
    },
    {
      path: 'admin/system/capabilities/:id',
      load: () => import('./capabilities/CapabilityPage'),
      need: CAPABILITY_READ,
    },
    { path: 'admin/system/nodes', load: () => import('./nodes/NodesPage'), need: READ },
  ],
  settings: [
    { id: 'capabilities', to: '/admin/system/capabilities', need: CAPABILITY_READ },
    { id: 'notifications', to: '/admin/system/notifications', need: READ },
    { id: 'nodes', to: '/admin/system/nodes', need: READ },
  ],
};
