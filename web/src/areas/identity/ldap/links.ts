import { path } from '../../../api';

export const LDAP_PAGE = '/admin/security/ldap';

export function ldapHref(name: string): string {
  return `${LDAP_PAGE}${path`/${name}`}`;
}
