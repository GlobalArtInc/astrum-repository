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
      to: '/admin/repository/blobstores',
      need: READ,
    },
  ],
};
