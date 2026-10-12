import { LayoutDashboard } from 'lucide-react';
import type { Area } from '../../app/area';
import { DashboardPage } from './DashboardPage';

export const dashboardArea: Area = {
  id: 'dashboard',
  routes: [{ path: '/', page: DashboardPage }],
  menu: [
    {
      id: 'dashboard',
      to: '/',
      label: 'dashboard.menu.label',
      icon: LayoutDashboard,
      group: 'main',
      order: 10,
    },
  ],
};
