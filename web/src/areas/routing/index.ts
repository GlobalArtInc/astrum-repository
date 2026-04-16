import type { Area } from '../../app/area';
import { ADMIN } from './model';

export const routingArea: Area = {
  id: 'routing',
  routes: [
    { path: 'admin/repository/routing-rules', load: () => import('./RoutingRulesPage'), need: ADMIN },
    { path: 'admin/repository/routing-rules/create', load: () => import('./RoutingRulePage'), need: ADMIN },
    { path: 'admin/repository/routing-rules/preview', load: () => import('./PreviewPage'), need: ADMIN },
    { path: 'admin/repository/routing-rules/:name', load: () => import('./RoutingRulePage'), need: ADMIN },
  ],
  settings: [{ id: 'routing-rules', to: '/admin/repository/routing-rules', need: ADMIN }],
};
