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
    {
      path: 'admin/security/privileges/:name/profile',
      load: () => import('./privileges/PrivilegeProfilePage'),
      need: PRIVILEGES_READ,
    },
    { path: 'admin/security/roles', load: () => import('./roles/RolesPage'), need: ROLES_NEED },
    {
      path: 'admin/security/roles/create',
      load: () => import('./roles/RolePage'),
      need: { all: [ROLES_NEED, 'astrum:roles:create'] },
    },
    { path: 'admin/security/roles/:id', load: () => import('./roles/RolePage'), need: ROLES_NEED },
    {
      path: 'admin/security/roles/:id/profile',
      load: () => import('./roles/RoleProfilePage'),
      need: ROLES_NEED,
    },
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
      to: '/admin/security/privileges',
      need: PRIVILEGES_READ,
    },
    {
      id: 'roles',
      to: '/admin/security/roles',
      need: ROLES_NEED,
    },
    {
      id: 'users',
      to: '/admin/security/users',
      need: USERS_NEED,
    },
    {
      id: 'anonymous',
      to: '/admin/security/anonymous',
      need: SETTINGS_READ,
    },
    {
      id: 'realms',
      to: '/admin/security/realms',
      need: SETTINGS_READ,
    },
  ],
};
