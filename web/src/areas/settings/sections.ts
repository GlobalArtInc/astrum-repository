import {
  Activity,
  Bell,
  Boxes,
  CalendarClock,
  Code,
  Database,
  Eraser,
  FileArchive,
  FileKey,
  Funnel,
  Globe,
  HardDrive,
  Info,
  KeyRound,
  Lock,
  LogIn,
  type LucideIcon,
  Mail,
  Network,
  Puzzle,
  Route,
  ScrollText,
  ShieldCheck,
  SlidersHorizontal,
  UserCog,
  Users,
  VenetianMask,
} from 'lucide-react';
import type { SettingsSection, SettingsTile } from '../../app/area';

export const SECTIONS: Array<{ id: SettingsSection; title: string }> = [
  { id: 'repository', title: 'settings.hub.repository' },
  { id: 'security', title: 'settings.hub.security' },
  { id: 'support', title: 'settings.hub.support' },
  { id: 'system', title: 'settings.hub.system' },
];

export interface CatalogTile {
  id: string;
  section: SettingsSection;
  order: number;
  path: string;
  title: string;
  description: string;
  icon: LucideIcon;
}

function tile(
  id: string,
  section: SettingsSection,
  order: number,
  path: string,
  icon: LucideIcon,
  title: string,
  description: string,
): CatalogTile {
  return { id, section, order, path, icon, title, description };
}

export const CATALOG: CatalogTile[] = [
  tile(
    'repositories',
    'repository',
    10,
    '/admin/repository/repositories',
    Database,
    'settings.tile.repositories',
    'settings.tile.repositoriesText',
  ),
  tile(
    'blobstores',
    'repository',
    20,
    '/admin/repository/blobstores',
    HardDrive,
    'settings.tile.blobstores',
    'settings.tile.blobstoresText',
  ),
  tile(
    'cleanup-policies',
    'repository',
    30,
    '/admin/repository/cleanup-policies',
    Eraser,
    'settings.tile.cleanupPolicies',
    'settings.tile.cleanupPoliciesText',
  ),
  tile(
    'routing-rules',
    'repository',
    40,
    '/admin/repository/routing-rules',
    Route,
    'settings.tile.routingRules',
    'settings.tile.routingRulesText',
  ),
  tile(
    'selectors',
    'repository',
    50,
    '/admin/repository/selectors',
    Funnel,
    'settings.tile.selectors',
    'settings.tile.selectorsText',
  ),
  tile(
    'proprietary',
    'repository',
    60,
    '/admin/repository/proprietary',
    Lock,
    'settings.tile.proprietary',
    'settings.tile.proprietaryText',
  ),
  tile(
    'users',
    'security',
    10,
    '/admin/security/users',
    Users,
    'settings.tile.users',
    'settings.tile.usersText',
  ),
  tile(
    'roles',
    'security',
    20,
    '/admin/security/roles',
    UserCog,
    'settings.tile.roles',
    'settings.tile.rolesText',
  ),
  tile(
    'privileges',
    'security',
    30,
    '/admin/security/privileges',
    KeyRound,
    'settings.tile.privileges',
    'settings.tile.privilegesText',
  ),
  tile(
    'realms',
    'security',
    40,
    '/admin/security/realms',
    ShieldCheck,
    'settings.tile.realms',
    'settings.tile.realmsText',
  ),
  tile(
    'ldap',
    'security',
    50,
    '/admin/security/ldap',
    Network,
    'settings.tile.ldap',
    'settings.tile.ldapText',
  ),
  tile('oidc', 'security', 60, '/admin/security/oidc', LogIn, 'settings.tile.oidc', 'settings.tile.oidcText'),
  tile(
    'anonymous',
    'security',
    70,
    '/admin/security/anonymous',
    VenetianMask,
    'settings.tile.anonymous',
    'settings.tile.anonymousText',
  ),
  tile(
    'ssl-certificates',
    'security',
    80,
    '/admin/security/ssl-certificates',
    FileKey,
    'settings.tile.sslCertificates',
    'settings.tile.sslCertificatesText',
  ),
  tile(
    'system-information',
    'support',
    10,
    '/admin/support/system-information',
    Info,
    'settings.tile.systemInformation',
    'settings.tile.systemInformationText',
  ),
  tile(
    'logs',
    'support',
    20,
    '/admin/support/logs',
    ScrollText,
    'settings.tile.logs',
    'settings.tile.logsText',
  ),
  tile(
    'logging',
    'support',
    30,
    '/admin/support/logging',
    SlidersHorizontal,
    'settings.tile.logging',
    'settings.tile.loggingText',
  ),
  tile(
    'metric-health',
    'support',
    40,
    '/admin/support/metric-health',
    Activity,
    'settings.tile.metricHealth',
    'settings.tile.metricHealthText',
  ),
  tile(
    'support-zip',
    'support',
    50,
    '/admin/support/support-zip',
    FileArchive,
    'settings.tile.supportZip',
    'settings.tile.supportZipText',
  ),
  tile(
    'tasks',
    'system',
    10,
    '/admin/system/tasks',
    CalendarClock,
    'settings.tile.tasks',
    'settings.tile.tasksText',
  ),
  tile('email', 'system', 20, '/admin/system/email', Mail, 'settings.tile.email', 'settings.tile.emailText'),
  tile('http', 'system', 30, '/admin/system/http', Globe, 'settings.tile.http', 'settings.tile.httpText'),
  tile('api', 'system', 40, '/api', Code, 'settings.tile.api', 'settings.tile.apiText'),
  tile(
    'capabilities',
    'system',
    50,
    '/admin/system/capabilities',
    Puzzle,
    'settings.tile.capabilities',
    'settings.tile.capabilitiesText',
  ),
  tile(
    'notifications',
    'system',
    60,
    '/admin/system/notifications',
    Bell,
    'settings.tile.notifications',
    'settings.tile.notificationsText',
  ),
  tile('nodes', 'system', 70, '/admin/system/nodes', Boxes, 'settings.tile.nodes', 'settings.tile.nodesText'),
];

const BY_ID = new Map(CATALOG.map((entry) => [entry.id, entry]));

export interface ResolvedTile {
  id: string;
  to: string;
  section: SettingsSection;
  order: number;
  title: string;
  description: string;
  icon: LucideIcon;
}

export function resolveTile(tile: SettingsTile): ResolvedTile | null {
  const known = BY_ID.get(tile.id);
  if (known) {
    return {
      id: tile.id,
      to: tile.to,
      section: known.section,
      order: known.order,
      title: known.title,
      description: known.description,
      icon: known.icon,
    };
  }
  if (!tile.section || !tile.title || !tile.description || !tile.icon) {
    return null;
  }
  return {
    id: tile.id,
    to: tile.to,
    section: tile.section,
    order: tile.order ?? 1000,
    title: tile.title,
    description: tile.description,
    icon: tile.icon,
  };
}
