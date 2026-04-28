import type { Area } from '../../app/area';
import { ATLAS_READ, LOGGING_READ, LOGGING_UPDATE, METRICS_READ } from './api';

export const supportArea: Area = {
  id: 'support',
  routes: [
    { path: 'admin/support/logs', load: () => import('./LogsPage'), need: LOGGING_READ },
    { path: 'admin/support/logs/:file', load: () => import('./LogViewerPage'), need: LOGGING_READ },
    { path: 'admin/support/logging', load: () => import('./LoggingPage'), need: LOGGING_READ },
    {
      path: 'admin/support/logging/create',
      load: () => import('./LoggerPage'),
      need: { all: [LOGGING_READ, LOGGING_UPDATE] },
    },
    { path: 'admin/support/logging/:name', load: () => import('./LoggerPage'), need: LOGGING_READ },
    {
      path: 'admin/support/system-information',
      load: () => import('./SystemInformationPage'),
      need: ATLAS_READ,
    },
    { path: 'admin/support/metric-health', load: () => import('./MetricHealthPage'), need: METRICS_READ },
    {
      path: 'admin/support/metric-health/:check',
      load: () => import('./MetricHealthPage'),
      need: METRICS_READ,
    },
    { path: 'admin/support/support-zip', load: () => import('./SupportZipPage'), need: ATLAS_READ },
  ],
  settings: [
    { id: 'system-information', to: '/admin/support/system-information', need: ATLAS_READ },
    { id: 'logs', to: '/admin/support/logs', need: LOGGING_READ },
    { id: 'logging', to: '/admin/support/logging', need: LOGGING_READ },
    { id: 'metric-health', to: '/admin/support/metric-health', need: METRICS_READ },
    { id: 'support-zip', to: '/admin/support/support-zip', need: ATLAS_READ },
  ],
};
