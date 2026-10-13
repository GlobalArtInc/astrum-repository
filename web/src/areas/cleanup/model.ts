import { INTERNAL } from '../../api';
import { formatName } from '../../ui';

export const POLICIES = `${INTERNAL}/cleanup-policies`;
export const FORMATS = `${POLICIES}/criteria/formats`;
export const PREVIEW = `${POLICIES}/preview/components`;
export const SUMMARY = `${POLICIES}/preview/summary`;
export const CSV = `${POLICIES}/preview/components/csv`;
export const LIST_PAGE = '/admin/repository/cleanup-policies';
export const READ = 'astrum:cleanup-policies:read';
export const CREATE = 'astrum:cleanup-policies:create';
export const UPDATE = 'astrum:cleanup-policies:update';
export const DELETE = 'astrum:cleanup-policies:delete';
export const ALL_FORMATS = '*';
export const MAX_DAYS = 24855;
export const PAGE_SIZE = 25;

export type ReleaseType = 'RELEASES' | 'PRERELEASES';
export const ANY_RELEASE = 'ANY';
export const DOCS = 'http://links.globalart.dev/products/astrum/docs/cleanup-policy';

export interface Policy {
  name: string;
  format: string;
  notes: string;
  criteriaLastBlobUpdated: number | null;
  criteriaLastDownloaded: number | null;
  criteriaReleaseType: ReleaseType | null;
  criteriaAssetRegex: string | null;
  inUseCount: number;
  repositories: string[];
}

export interface FormatCriteria {
  id: string;
  name: string;
  availableCriteria: string[];
}

export interface PreviewComponent {
  id: string;
  repository: string;
  format: string;
  group: string;
  name: string;
  version: string;
  assetCount: number;
  size: number;
  lastUpdated: string | null;
  lastDownloaded: string | null;
}

export interface PreviewPage {
  total: number;
  complete: boolean;
  page: number;
  size: number;
  results: PreviewComponent[];
}

export interface Summary {
  repository: string;
  componentCount: number;
  assetCount: number;
  totalSize: number;
  complete: boolean;
}

export interface PolicyValues {
  name: string;
  format: string;
  notes: string;
  ageOn: boolean;
  age: string;
  usageOn: boolean;
  usage: string;
  releaseType: ReleaseType | typeof ANY_RELEASE;
  regexOn: boolean;
  regex: string;
}

export function policyHref(name: string): string {
  return `${LIST_PAGE}/${encodeURIComponent(name)}`;
}

export function formatLabel(format: string, all: string): string {
  return format === ALL_FORMATS ? all : formatName(format);
}

export const EMPTY_POLICY: PolicyValues = {
  name: '',
  format: '',
  notes: '',
  ageOn: false,
  age: '',
  usageOn: false,
  usage: '',
  releaseType: ANY_RELEASE,
  regexOn: false,
  regex: '',
};

export function fromPolicy(policy: Policy): PolicyValues {
  return {
    name: policy.name,
    format: policy.format,
    notes: policy.notes ?? '',
    ageOn: policy.criteriaLastBlobUpdated !== null,
    age: policy.criteriaLastBlobUpdated === null ? '' : String(policy.criteriaLastBlobUpdated),
    usageOn: policy.criteriaLastDownloaded !== null,
    usage: policy.criteriaLastDownloaded === null ? '' : String(policy.criteriaLastDownloaded),
    releaseType: policy.criteriaReleaseType ?? ANY_RELEASE,
    regexOn: policy.criteriaAssetRegex !== null,
    regex: policy.criteriaAssetRegex ?? '',
  };
}

export function available(formats: readonly FormatCriteria[] | undefined, format: string): Set<string> {
  return new Set(formats?.find((candidate) => candidate.id === format)?.availableCriteria ?? []);
}

function days(on: boolean, value: string): number | null {
  return on && value.trim() !== '' ? Number(value) : null;
}

export function toRequest(values: PolicyValues, criteria: Set<string>) {
  return {
    name: values.name.trim(),
    format: values.format,
    notes: values.notes,
    criteriaLastBlobUpdated: criteria.has('lastBlobUpdated') ? days(values.ageOn, values.age) : null,
    criteriaLastDownloaded: criteria.has('lastDownloaded') ? days(values.usageOn, values.usage) : null,
    criteriaReleaseType:
      criteria.has('isPrerelease') && values.releaseType !== ANY_RELEASE ? values.releaseType : null,
    criteriaAssetRegex:
      criteria.has('regex') && values.regexOn && values.regex.trim() !== '' ? values.regex.trim() : null,
  };
}

export function hasCriteria(values: PolicyValues, criteria: Set<string>): boolean {
  const request = toRequest(values, criteria);
  return (
    request.criteriaLastBlobUpdated !== null ||
    request.criteriaLastDownloaded !== null ||
    request.criteriaAssetRegex !== null
  );
}

export interface PolicyTexts {
  name: string;
  nameRule: string;
  format: string;
  notes: string;
  criteria: string;
  age: string;
  usage: string;
  days: string;
  regex: string;
  regexInvalid: string;
}

const NAME = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;
export const MAX_NOTES = 4000;

export function validRegex(pattern: string): boolean {
  try {
    new RegExp(pattern);
    return true;
  } catch {
    return false;
  }
}

export function validate(
  values: PolicyValues,
  criteria: Set<string>,
  creating: boolean,
  texts: PolicyTexts,
): Partial<Record<keyof PolicyValues | 'criteria', string>> {
  const problems: Partial<Record<keyof PolicyValues | 'criteria', string>> = {};
  const name = values.name.trim();
  if (creating) {
    if (name === '') {
      problems.name = texts.name;
    } else if (!NAME.test(name) || name.length > 255) {
      problems.name = texts.nameRule;
    }
  }
  if (values.format === '') {
    problems.format = texts.format;
  }
  if (values.notes.length > MAX_NOTES) {
    problems.notes = texts.notes;
  }
  const check = (on: boolean, value: string, key: 'age' | 'usage', missing: string) => {
    if (!on) {
      return;
    }
    if (value.trim() === '') {
      problems[key] = missing;
    } else if (!/^\d+$/.test(value.trim()) || Number(value) < 1 || Number(value) > MAX_DAYS) {
      problems[key] = texts.days;
    }
  };
  if (criteria.has('lastBlobUpdated')) {
    check(values.ageOn, values.age, 'age', texts.age);
  }
  if (criteria.has('lastDownloaded')) {
    check(values.usageOn, values.usage, 'usage', texts.usage);
  }
  if (criteria.has('regex') && values.regexOn) {
    if (values.regex.trim() === '') {
      problems.regex = texts.regex;
    } else if (!validRegex(values.regex.trim())) {
      problems.regex = texts.regexInvalid;
    }
  }
  if (
    values.format !== '' &&
    !hasCriteria(values, criteria) &&
    !problems.age &&
    !problems.usage &&
    !problems.regex
  ) {
    problems.criteria = texts.criteria;
  }
  return problems;
}

export function csvHref(values: PolicyValues, criteria: Set<string>, repository: string): string {
  const request = toRequest(values, criteria);
  const query = new URLSearchParams({ repository, format: request.format });
  if (request.name) {
    query.set('name', request.name);
  }
  for (const [key, value] of Object.entries(request)) {
    if (key.startsWith('criteria') && value !== null) {
      query.set(key, String(value));
    }
  }
  return `${CSV}?${query.toString()}`;
}

export function matchesFilter(policy: Policy, filter: string, label: string): boolean {
  const words = filter.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const text = `${policy.name} ${policy.format} ${label} ${policy.notes ?? ''}`.toLowerCase();
  return words.every((word) => text.includes(word));
}
