import type { Action, Privilege } from '../api';

export const ANY = '*';

function joinActions(actions: readonly Action[] | undefined): string | null {
  if (!actions || actions.length === 0) {
    return null;
  }
  return actions.map((action) => action.toLowerCase()).join(',');
}

export function permissionOf(privilege: Privilege): string | null {
  const actions = joinActions(privilege.actions);
  switch (privilege.type) {
    case 'application':
      return privilege.domain && actions ? `astrum:${privilege.domain}:${actions}` : null;
    case 'wildcard':
      return privilege.pattern || null;
    case 'repository-view':
    case 'repository-admin':
      return actions
        ? `astrum:${privilege.type}:${privilege.format ?? ANY}:${privilege.repository ?? ANY}:${actions}`
        : null;
    case 'repository-content-selector':
      return actions
        ? `astrum:repository-content-selector:${privilege.format ?? ANY}:${privilege.repository ?? ANY}:${privilege.contentSelector ?? ''}:${actions}`
        : null;
    default:
      return null;
  }
}

export interface PrivilegeValues {
  name: string;
  description: string;
  domain: string;
  pattern: string;
  format: string;
  repository: string;
  contentSelector: string;
  actions: Action[];
}

export const EMPTY_VALUES: PrivilegeValues = {
  name: '',
  description: '',
  domain: '',
  pattern: '',
  format: ANY,
  repository: ANY,
  contentSelector: '',
  actions: [],
};

export function fromPrivilege(privilege: Privilege): PrivilegeValues {
  return {
    name: privilege.name,
    description: privilege.description ?? '',
    domain: privilege.domain ?? '',
    pattern: privilege.pattern ?? '',
    format: privilege.format ?? ANY,
    repository: privilege.repository ?? ANY,
    contentSelector: privilege.contentSelector ?? '',
    actions: privilege.actions ?? [],
  };
}
