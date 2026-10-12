import { KeyRound, Network, UserCheck } from 'lucide-react';
import type { Area } from '../../app/area';

const SETTINGS_READ = 'astrum:settings:read';
const LDAP_READ = 'astrum:ldap:read';

export const identityArea: Area = {
  id: 'identity',
  routes: [
    { path: 'admin/security/ldap', load: () => import('./ldap/LdapListPage'), need: LDAP_READ },
    {
      path: 'admin/security/ldap/create',
      load: () => import('./ldap/LdapServerPage'),
      need: { all: [LDAP_READ, 'astrum:ldap:create'] },
    },
    { path: 'admin/security/ldap/:name', load: () => import('./ldap/LdapServerPage'), need: LDAP_READ },
    { path: 'admin/security/oauth2', load: () => import('./oidc/OidcPage'), need: SETTINGS_READ },
    {
      path: 'admin/security/default-role',
      load: () => import('./defaultRole/DefaultRolePage'),
      need: { all: [SETTINGS_READ, 'astrum:roles:read'] },
    },
  ],
  settings: [
    {
      id: 'ldap',
      section: 'security',
      to: '/admin/security/ldap',
      title: 'identity.tiles.ldap',
      description: 'identity.tiles.ldapText',
      icon: Network,
      order: 52,
      need: LDAP_READ,
    },
    {
      id: 'oauth2',
      section: 'security',
      to: '/admin/security/oauth2',
      title: 'identity.tiles.oidc',
      description: 'identity.tiles.oidcText',
      icon: KeyRound,
      order: 54,
      need: SETTINGS_READ,
    },
    {
      id: 'default-role',
      section: 'security',
      to: '/admin/security/default-role',
      title: 'identity.tiles.defaultRole',
      description: 'identity.tiles.defaultRoleText',
      icon: UserCheck,
      order: 56,
      need: { all: [SETTINGS_READ, 'astrum:roles:read'] },
    },
  ],
};
