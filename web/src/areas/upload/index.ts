import { HardDriveUpload } from 'lucide-react';
import type { Area } from '../../app/area';
import type { Need } from '../../auth/permissions';

export const UPLOAD_NEED: Need = {
  any: [{ some: 'astrum:repository-view:*:*:add' }, { some: 'astrum:repository-content-selector:*:*:*:add' }],
};

export const uploadArea: Area = {
  id: 'upload',
  routes: [
    { path: 'upload', load: () => import('./UploadListPage'), need: UPLOAD_NEED, requiresUser: true },
    {
      path: 'upload/:repository',
      load: () => import('./UploadPage'),
      need: UPLOAD_NEED,
      requiresUser: true,
    },
  ],
  menu: [
    {
      id: 'upload',
      to: '/upload',
      label: 'upload.menu.label',
      icon: HardDriveUpload,
      group: 'main',
      order: 40,
      need: UPLOAD_NEED,
      requiresUser: true,
    },
  ],
};
