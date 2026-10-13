import { path } from '../../../api';
import { DEFAULT_SOURCE } from '../api';

export const PRIVILEGES_PAGE = '/admin/security/privileges';
export const ROLES_PAGE = '/admin/security/roles';
export const USERS_PAGE = '/admin/security/users';

export function privilegeHref(name: string): string {
  return path`/admin/security/privileges/${name}`;
}

export function roleHref(id: string): string {
  return path`/admin/security/roles/${id}`;
}

export function userHref(userId: string, source: string = DEFAULT_SOURCE): string {
  return source === DEFAULT_SOURCE
    ? path`/admin/security/users/${userId}`
    : path`/admin/security/users/${userId}/${source}`;
}

export function privilegeProfileHref(name: string): string {
  return path`/admin/security/privileges/${name}/profile`;
}

export function roleProfileHref(id: string): string {
  return path`/admin/security/roles/${id}/profile`;
}
