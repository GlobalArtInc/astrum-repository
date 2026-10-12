import type { RepositoryType, UiFormat } from '../../../api';

export interface Spec {
  format: string;
  type: RepositoryType;
}

const WRITABLE_MEMBER = new Set(['cargo', 'docker', 'swift']);
const API_NAMES: Record<string, string> = { maven2: 'maven' };
export const TYPES: readonly RepositoryType[] = ['hosted', 'proxy', 'group'];

export function isRepositoryType(value: unknown): value is RepositoryType {
  return typeof value === 'string' && (TYPES as readonly string[]).includes(value);
}

export function apiFormat(format: string, formats: readonly UiFormat[] = []): string {
  return formats.find((known) => known.id === format)?.apiName ?? API_NAMES[format] ?? format;
}

export function formatFromUrl(segment: string, formats: readonly UiFormat[] = []): string {
  const known = formats.find((format) => format.id === segment || format.apiName === segment);
  if (known) {
    return known.id;
  }
  const byApiName = Object.entries(API_NAMES).find(([, apiName]) => apiName === segment);
  return byApiName ? byApiName[0] : segment;
}

export const sections = {
  writePolicy: (spec: Spec) => spec.type === 'hosted',
  latestPolicy: (spec: Spec) => spec.format === 'docker' && spec.type === 'hosted',
  proxy: (spec: Spec) => spec.type === 'proxy',
  group: (spec: Spec) => spec.type === 'group',
  writableMember: (spec: Spec) => spec.type === 'group' && WRITABLE_MEMBER.has(spec.format),
  preemptive: (spec: Spec) => spec.format === 'maven2' && spec.type === 'proxy',
  maven: (spec: Spec) => spec.format === 'maven2' && spec.type !== 'group',
  docker: (spec: Spec) => spec.format === 'docker',
  dockerProxy: (spec: Spec) => spec.format === 'docker' && spec.type === 'proxy',
  raw: (spec: Spec) => spec.format === 'raw',
  rawQuery: (spec: Spec) => spec.format === 'raw' && spec.type === 'proxy',
};
