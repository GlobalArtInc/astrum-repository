import type { Translate } from '../../../i18n';
import { DEFAULT_SOURCE, type Role } from '../api';
import type { PickerOption } from '../shared/ChipPicker';

export type RoleType = 'astrum' | 'external';

export interface RoleValues {
  type: RoleType;
  source: string;
  id: string;
  name: string;
  description: string;
  privileges: string[];
  roles: string[];
}

export const EMPTY_ROLE: RoleValues = {
  type: 'astrum',
  source: DEFAULT_SOURCE,
  id: '',
  name: '',
  description: '',
  privileges: [],
  roles: [],
};

export function fromRole(role: Role): RoleValues {
  return {
    type: role.source === DEFAULT_SOURCE ? 'astrum' : 'external',
    source: role.source,
    id: role.id,
    name: role.name,
    description: role.description ?? '',
    privileges: [...role.privileges],
    roles: [...role.roles],
  };
}

export function toRoleRequest(values: RoleValues): Omit<Role, 'readOnly'> {
  return {
    source: values.type === 'external' ? values.source : DEFAULT_SOURCE,
    id: values.id.trim(),
    name: values.name.trim(),
    description: values.description.trim(),
    privileges: [...values.privileges],
    roles: [...values.roles],
  };
}

export function validateId(t: Translate, value: string): string | undefined {
  if (value.trim() === '') {
    return t('security.form.required');
  }
  if (value.trim() !== value) {
    return t('security.form.idSpaces');
  }
  if (value.includes(':')) {
    return t('security.form.idColon');
  }
  return undefined;
}

export function roleOptions(
  assignable: readonly Role[],
  all: readonly Role[],
  chosen: readonly string[],
  self?: string,
): PickerOption[] {
  const byId = new Map(all.map((role) => [role.id, role]));
  const ids = new Set(assignable.map((role) => role.id));
  for (const id of chosen) {
    ids.add(id);
  }
  if (self) {
    ids.delete(self);
  }
  return [...ids]
    .sort((left, right) => left.localeCompare(right))
    .map((id) => {
      const role = byId.get(id) ?? assignable.find((candidate) => candidate.id === id);
      const name = role?.name && role.name !== id ? role.name : '';
      return {
        value: id,
        label: id,
        detail: [name, role?.description].filter(Boolean).join(' · ') || undefined,
      };
    });
}
