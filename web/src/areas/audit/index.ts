import type { Area } from '../../app/area';

export const AUDIT_NEED = 'astrum:audit:read';

export const auditArea: Area = {
  id: 'audit',
  routes: [{ path: 'audit', load: () => import('./AuditPage'), need: AUDIT_NEED }],
};
