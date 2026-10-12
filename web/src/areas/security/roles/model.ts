import type { Translate } from '../../../i18n';
import type { Privilege, Role } from '../api';
import { typeLabel } from '../privileges/labels';
import type { PickerOption } from '../shared/ChipPicker';

export interface RoleValues {
  id: string;
  name: string;
  description: string;
  privileges: string[];
  roles: string[];
}

export const EMPTY_ROLE: RoleValues = { id: '', name: '', description: '', privileges: [], roles: [] };

export function fromRole(role: Role): RoleValues {
  return {
    id: role.id,
    name: role.name,
    description: role.description ?? '',
    privileges: [...role.privileges],
    roles: [...role.roles],
  };
}

export function toRoleRequest(values: RoleValues): Omit<Role, 'source' | 'readOnly'> {
  return {
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

export function validateRole(t: Translate, values: RoleValues): Partial<Record<keyof RoleValues, string>> {
  const errors: Partial<Record<keyof RoleValues, string>> = {};
  const id = validateId(t, values.id);
  if (id) {
    errors.id = id;
  }
  if (values.name.trim() === '') {
    errors.name = t('security.form.required');
  }
  return errors;
}

export function privilegeOptions(t: Translate, privileges: readonly Privilege[]): PickerOption[] {
  return [...privileges]
    .sort((left, right) => left.name.localeCompare(right.name))
    .map((privilege) => ({
      value: privilege.name,
      label: privilege.name,
      detail: [typeLabel(t, privilege.type), privilege.description].filter(Boolean).join(' · '),
    }));
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
