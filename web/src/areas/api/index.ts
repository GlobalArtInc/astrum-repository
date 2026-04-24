import { FileCode } from 'lucide-react';
import type { Area } from '../../app/area';
import { MENU_ORDER } from '../../app/area';
import { READ } from './model';

export const apiArea: Area = {
  id: 'api',
  routes: [{ path: 'api', load: () => import('./ApiPage'), need: READ, requiresUser: true }],
  menu: [
    {
      id: 'api',
      to: '/api',
      label: 'api.menu.label',
      icon: FileCode,
      group: 'main',
      order: MENU_ORDER.api,
      need: READ,
      requiresUser: true,
    },
  ],
  settings: [{ id: 'api', to: '/api', need: READ }],
};
