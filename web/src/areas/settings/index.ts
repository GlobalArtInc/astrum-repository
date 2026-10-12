import { Settings } from 'lucide-react';
import type { Area } from '../../app/area';
import { SettingsHubPage } from './SettingsHubPage';

export const settingsArea: Area = {
  id: 'settings',
  routes: [{ path: 'settings', page: SettingsHubPage }],
  menu: [
    {
      id: 'settings',
      to: '/settings',
      label: 'settings.menu.label',
      icon: Settings,
      group: 'admin',
      order: 900,
      activeFor: ['/admin'],
      visible: ({ settingsTiles }) => settingsTiles.length > 0,
    },
  ],
};
