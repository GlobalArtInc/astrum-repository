import { HardDrive } from 'lucide-react';
import type { Area } from '../../app/area';

const READ = 'astrum:blobstores:read';

export const blobstoresArea: Area = {
  id: 'blobstores',
  routes: [
    { path: 'admin/repository/blobstores', load: () => import('./pages/BlobStoresPage'), need: READ },
    {
      path: 'admin/repository/blobstores/create',
      load: () => import('./pages/CreatePickerPage'),
      need: { all: [READ, 'astrum:blobstores:create'] },
    },
    {
      path: 'admin/repository/blobstores/create/:type',
      load: () => import('./pages/CreateBlobStorePage'),
      need: { all: [READ, 'astrum:blobstores:create'] },
    },
    {
      path: 'admin/repository/blobstores/:type/:name',
      load: () => import('./pages/BlobStorePage'),
      need: READ,
    },
  ],
  settings: [
    {
      id: 'blobstores',
      section: 'repository',
      to: '/admin/repository/blobstores',
      title: 'blobstores.tile.title',
      description: 'blobstores.tile.description',
      icon: HardDrive,
      order: 20,
      need: READ,
    },
  ],
};
