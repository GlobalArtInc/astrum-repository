import type { Translate } from '../../../i18n';
import type { Action, Privilege, PrivilegeField, PrivilegeType } from '../api';

export const ANY = '*';

const APPLICATION_NAMES: Record<Action, string> = {
  READ: 'read',
  BROWSE: 'browse',
  EDIT: 'update',
  ADD: 'create',
  DELETE: 'delete',
  RUN: 'run',
  ASSOCIATE: 'associate',
  DISASSOCIATE: 'disassociate',
  START: 'start',
  STOP: 'stop',
  ALL: '*',
};

const REPOSITORY_NAMES: Record<Action, string> = {
  ...APPLICATION_NAMES,
  EDIT: 'edit',
  ADD: 'add',
};

function joinActions(actions: readonly Action[] | undefined, names: Record<Action, string>): string | null {
  if (!actions || actions.length === 0) {
    return null;
  }
  if (actions.includes('ALL')) {
    return '*';
  }
  return [...new Set(actions.map((action) => names[action] ?? action.toLowerCase()))].sort().join(',');
}

export function permissionOf(privilege: Privilege): string | null {
  switch (privilege.type) {
    case 'application': {
      const actions = joinActions(privilege.actions, APPLICATION_NAMES);
      return privilege.domain && actions ? `astrum:${privilege.domain}:${actions}` : null;
    }
    case 'wildcard':
      return privilege.pattern || null;
    case 'repository-view':
    case 'repository-admin': {
      const actions = joinActions(privilege.actions, REPOSITORY_NAMES);
      return actions
        ? `astrum:${privilege.type}:${privilege.format ?? ANY}:${privilege.repository ?? ANY}:${actions}`
        : null;
    }
    case 'repository-content-selector': {
      const actions = joinActions(privilege.actions, REPOSITORY_NAMES);
      return actions
        ? `astrum:repository-content-selector:${privilege.contentSelector ?? ''}:${privilege.format ?? ANY}:${privilege.repository ?? ANY}:${actions}`
        : null;
    }
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

type ValueField = Exclude<keyof PrivilegeValues, 'actions'>;

const VALUE_FIELDS: readonly string[] = [
  'name',
  'description',
  'domain',
  'pattern',
  'format',
  'repository',
  'contentSelector',
];

function isValueField(id: string): id is ValueField {
  return VALUE_FIELDS.includes(id);
}

export function toRequest(type: PrivilegeType, values: PrivilegeValues): Record<string, unknown> {
  const body: Record<string, unknown> = {};
  for (const field of type.fields) {
    if (field.kind === 'actions') {
      body.actions = [...values.actions];
    } else if (isValueField(field.id)) {
      body[field.id] = field.id === 'description' ? values.description : values[field.id].trim();
    }
  }
  return body;
}

export function toggleAction(actions: readonly Action[], action: Action, on: boolean): Action[] {
  if (!on) {
    return actions.filter((item) => item !== action);
  }
  if (action === 'ALL') {
    return ['ALL'];
  }
  return [...actions.filter((item) => item !== 'ALL' && item !== action), action];
}

const NAME = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;

export function validatePrivilege(
  t: Translate,
  type: PrivilegeType,
  values: PrivilegeValues,
): Partial<Record<keyof PrivilegeValues, string>> {
  const errors: Partial<Record<keyof PrivilegeValues, string>> = {};
  for (const field of type.fields) {
    if (field.kind === 'actions') {
      if (field.required && values.actions.length === 0) {
        errors.actions = t('security.privilege.actionsRequired');
      }
      continue;
    }
    if (!isValueField(field.id)) {
      continue;
    }
    const value = values[field.id].trim();
    if (field.required && value === '') {
      errors[field.id] = t('security.form.required');
    }
  }
  if (values.name.trim() !== '' && !NAME.test(values.name.trim())) {
    errors.name = t('security.form.nameRule');
  }
  return errors;
}

export function fieldOf(type: PrivilegeType | undefined, id: string): PrivilegeField | undefined {
  return type?.fields.find((field) => field.id === id);
}
