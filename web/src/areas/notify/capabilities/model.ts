import {
  Activity,
  Calendar,
  FileSearch,
  Globe,
  HardDrive,
  Palette,
  Puzzle,
  Server,
  Settings,
} from 'lucide-react';
import type { Translate } from '../../../i18n';

export const CAPABILITIES = '/service/rest/v1/capabilities';
export const TYPES = `${CAPABILITIES}/types`;
export const LIST_PAGE = '/admin/system/capabilities';
export const READ = 'astrum:capabilities:read';
export const CREATE = 'astrum:capabilities:create';
export const UPDATE = 'astrum:capabilities:update';
export const DELETE = 'astrum:capabilities:delete';

export const CATEGORY_ICONS: Record<string, typeof Puzzle> = {
  core: Server,
  ui: Settings,
  audit: FileSearch,
  scheduling: Calendar,
  monitoring: Activity,
};

export const TYPE_ICONS: Record<string, typeof Puzzle> = {
  baseurl: Globe,
  'rapture.settings': Settings,
  'rapture.branding': Palette,
  'metrics.prometheus': Activity,
  'blobstores.space': HardDrive,
};

export function categoryIcon(category: string | undefined): typeof Puzzle {
  return CATEGORY_ICONS[(category ?? '').toLowerCase()] ?? Puzzle;
}

export interface Capability {
  id: string;
  type: string;
  typeName: string;
  enabled: boolean;
  active: boolean;
  error: boolean;
  state: string;
  notes: string;
  description: string | null;
  status: string | null;
  properties: Record<string, string>;
  tags: Record<string, string>;
}

export interface FormField {
  id: string;
  type: 'string' | 'url' | 'number' | 'checkbox' | 'text-area' | string;
  label: string;
  helpText: string | null;
  required: boolean;
  initialValue: string | null;
  minValue: string | null;
  maxValue: string | null;
}

export interface CapabilityType {
  id: string;
  name: string;
  about: string;
  formFields: FormField[];
}

export interface CapabilityValues {
  enabled: boolean;
  notes: string;
  properties: Record<string, string>;
}

export function categoryLabel(t: Translate, category: string | undefined): string {
  switch ((category ?? '').toLowerCase()) {
    case 'core':
      return t('notify.capabilities.categories.core');
    case 'ui':
      return 'UI';
    case 'audit':
      return t('notify.capabilities.categories.audit');
    case 'scheduling':
      return t('notify.capabilities.categories.scheduling');
    case 'security':
      return t('notify.capabilities.categories.security');
    case 'monitoring':
      return t('notify.capabilities.categories.monitoring');
    default:
      return category && category.length > 0 ? category : t('notify.capabilities.categories.other');
  }
}

export function stateLabel(t: Translate, capability: Capability): string {
  if (capability.error) {
    return t('notify.capabilities.states.error');
  }
  return capability.active
    ? t('notify.capabilities.states.active')
    : t('notify.capabilities.states.disabled');
}

export function initialCapabilityValues(
  type: CapabilityType,
  capability: Capability | null,
): CapabilityValues {
  const properties: Record<string, string> = {};
  for (const field of type.formFields) {
    properties[field.id] =
      capability?.properties[field.id] ?? field.initialValue ?? (field.type === 'checkbox' ? 'false' : '');
  }
  return { enabled: capability?.enabled ?? true, notes: capability?.notes ?? '', properties };
}

export function validateCapability(
  t: Translate,
  type: CapabilityType,
  values: CapabilityValues,
): Record<string, string> {
  const errors: Record<string, string> = {};
  for (const field of type.formFields) {
    const value = (values.properties[field.id] ?? '').trim();
    if (field.type === 'checkbox') {
      continue;
    }
    if (value === '') {
      if (field.required) {
        errors[field.id] = t('notify.capabilities.errors.required', { field: field.label });
      }
      continue;
    }
    if (field.type === 'url' && !/^https?:\/\/[^\s/?#]+/i.test(value)) {
      errors[field.id] = t('notify.capabilities.errors.url');
    }
    if (field.type === 'number') {
      if (!/^-?\d+$/.test(value)) {
        errors[field.id] = t('notify.capabilities.errors.number', { field: field.label });
      } else if (field.minValue !== null && Number(value) < Number(field.minValue)) {
        errors[field.id] = t('notify.capabilities.errors.min', { field: field.label, min: field.minValue });
      } else if (field.maxValue !== null && Number(value) > Number(field.maxValue)) {
        errors[field.id] = t('notify.capabilities.errors.max', { field: field.label, max: field.maxValue });
      }
    }
  }
  return errors;
}

export function toCapabilityRequest(type: CapabilityType, values: CapabilityValues): Record<string, unknown> {
  const properties: Record<string, string> = {};
  for (const field of type.formFields) {
    const value = values.properties[field.id] ?? '';
    properties[field.id] = field.type === 'text-area' ? value : value.trim();
  }
  return { type: type.id, enabled: values.enabled, notes: values.notes, properties };
}

export function aboutParagraphs(about: string): string[] {
  return about
    .replace(/<\/?(p|em|strong|b|i|br\s*\/?)>/gi, '\n')
    .replace(/<[^>]*>/g, '')
    .split(/\n\s*\n|\n/)
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
}

export function serverProblems(
  type: CapabilityType,
  problems: { id: string; message: string }[],
): { fields: Record<string, string>; other: string[] } {
  const fields: Record<string, string> = {};
  const other: string[] = [];
  for (const problem of problems) {
    const name = problem.id.replace(/^properties\./, '');
    if (type.formFields.some((field) => field.id === name)) {
      fields[name] = problem.message;
    } else {
      other.push(problem.message);
    }
  }
  return { fields, other };
}
