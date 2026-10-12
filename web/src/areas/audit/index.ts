import { ScrollText } from 'lucide-react';
import type { Area } from '../../app/area';

export const AUDIT_NEED = 'astrum:audit:read';

export const auditArea: Area = {
  id: 'audit',
  routes: [{ path: 'audit', load: () => import('./AuditPage'), need: AUDIT_NEED }],
  settings: [
    {
      id: 'audit',
      section: 'security',
      to: '/audit',
      title: 'audit.tile.title',
      description: 'audit.tile.description',
      icon: ScrollText,
      order: 900,
      need: AUDIT_NEED,
    },
  ],
};
