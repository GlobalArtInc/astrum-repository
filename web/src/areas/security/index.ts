import { KeyRound, Layers, ShieldAlert, ShieldCheck, UsersRound, UserX } from 'lucide-react';
import type { Area } from '../../app/area';

const PRIVILEGES_READ = 'astrum:privileges:read';
const ROLES_NEED = { all: ['astrum:roles:read', PRIVILEGES_READ] };
const USERS_NEED = { all: ['astrum:users:read', 'astrum:roles:read'] };
const SETTINGS_READ = 'astrum:settings:read';
const ADMINISTRATOR = 'astrum:*';

export const securityArea: Area = {
  id: 'security',
  routes: [
    {
      path: 'admin/security/privileges',
      load: () => import('./privileges/PrivilegesPage'),
      need: PRIVILEGES_READ,
    },
    {
      path: 'admin/security/privileges/create',
      load: () => import('./privileges/PrivilegeTypePage'),
      need: 'astrum:privileges:create',
    },
    {
      path: 'admin/security/privileges/create/:type',
      load: () => import('./privileges/PrivilegePage'),
      need: 'astrum:privileges:create',
    },
    {
      path: 'admin/security/privileges/:name',
      load: () => import('./privileges/PrivilegePage'),
      need: PRIVILEGES_READ,
    },
    { path: 'admin/security/roles', load: () => import('./roles/RolesPage'), need: ROLES_NEED },
    {
      path: 'admin/security/roles/create',
      load: () => import('./roles/RolePage'),
      need: { all: [ROLES_NEED, 'astrum:roles:create'] },
    },
    { path: 'admin/security/roles/:id', load: () => import('./roles/RolePage'), need: ROLES_NEED },
    { path: 'admin/security/users', load: () => import('./users/UsersPage'), need: USERS_NEED },
    {
      path: 'admin/security/users/create',
      load: () => import('./users/UserPage'),
      need: { all: [USERS_NEED, 'astrum:users:create'] },
    },
    { path: 'admin/security/users/:userId', load: () => import('./users/UserPage'), need: USERS_NEED },
    {
      path: 'admin/security/users/:userId/:source',
      load: () => import('./users/UserPage'),
      need: USERS_NEED,
    },
    {
      path: 'admin/security/anonymous',
      load: () => import('./anonymous/AnonymousPage'),
      need: SETTINGS_READ,
    },
    { path: 'admin/security/realms', load: () => import('./realms/RealmsPage'), need: SETTINGS_READ },
    { path: 'admin/security/ssrf-protection', load: () => import('./ssrf/SsrfPage'), need: ADMINISTRATOR },
  ],
  settings: [
    {
      id: 'privileges',
      section: 'security',
      to: '/admin/security/privileges',
      title: 'security.tiles.privileges',
      description: 'security.tiles.privilegesText',
      icon: KeyRound,
      order: 10,
      need: PRIVILEGES_READ,
    },
    {
      id: 'roles',
      section: 'security',
      to: '/admin/security/roles',
      title: 'security.tiles.roles',
      description: 'security.tiles.rolesText',
      icon: ShieldCheck,
      order: 20,
      need: ROLES_NEED,
    },
    {
      id: 'users',
      section: 'security',
      to: '/admin/security/users',
      title: 'security.tiles.users',
      description: 'security.tiles.usersText',
      icon: UsersRound,
      order: 30,
      need: USERS_NEED,
    },
    {
      id: 'anonymous',
      section: 'security',
      to: '/admin/security/anonymous',
      title: 'security.tiles.anonymous',
      description: 'security.tiles.anonymousText',
      icon: UserX,
      order: 40,
      need: SETTINGS_READ,
    },
    {
      id: 'realms',
      section: 'security',
      to: '/admin/security/realms',
      title: 'security.tiles.realms',
      description: 'security.tiles.realmsText',
      icon: Layers,
      order: 50,
      need: SETTINGS_READ,
    },
    {
      id: 'ssrf-protection',
      section: 'security',
      to: '/admin/security/ssrf-protection',
      title: 'security.tiles.ssrf',
      description: 'security.tiles.ssrfText',
      icon: ShieldAlert,
      order: 60,
      need: ADMINISTRATOR,
    },
  ],
};
