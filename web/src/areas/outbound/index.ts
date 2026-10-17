import type { Area } from '../../app/area';
import { SETTINGS_READ, TRUSTSTORE_CREATE, TRUSTSTORE_READ } from './api';

export const outboundArea: Area = {
  id: 'outbound',
  routes: [
    { path: 'admin/system/http', load: () => import('./HttpPage'), need: SETTINGS_READ },
    { path: 'admin/system/email', load: () => import('./EmailPage'), need: SETTINGS_READ },
    {
      path: 'admin/security/ssl-certificates',
      load: () => import('./ssl/CertificatesPage'),
      need: TRUSTSTORE_READ,
    },
    {
      path: 'admin/security/ssl-certificates/add',
      load: () => import('./ssl/AddCertificatePage'),
      need: { all: [TRUSTSTORE_READ, TRUSTSTORE_CREATE] },
    },
    {
      path: 'admin/security/ssl-certificates/:id',
      load: () => import('./ssl/CertificatePage'),
      need: TRUSTSTORE_READ,
    },
  ],
  settings: [
    { id: 'http', to: '/admin/system/http', need: SETTINGS_READ },
    { id: 'email', to: '/admin/system/email', need: SETTINGS_READ },
    { id: 'ssl-certificates', to: '/admin/security/ssl-certificates', need: TRUSTSTORE_READ },
  ],
};
