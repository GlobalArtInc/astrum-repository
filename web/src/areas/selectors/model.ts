import { INTERNAL, path, V1 } from '../../api';
import type { Translate } from '../../i18n';

export const SELECTORS = `${V1}/security/content-selectors`;
export const PREVIEW = `${INTERNAL}/ui/content-selectors/preview`;
export const REPOSITORIES = `${V1}/repositories`;
export const PRIVILEGES = `${V1}/security/privileges`;
export const LIST_PAGE = '/admin/repository/selectors';
export const ALL_REPOSITORIES = '*';

export interface ContentSelector {
  name: string;
  type: string;
  description: string | null;
  expression: string;
}

export interface Repository {
  name: string;
  format: string;
  type: string;
}

export interface PreviewMatch {
  repository: string;
  format: string;
  path: string;
  assetId: string;
}

export interface Preview {
  items: PreviewMatch[];
  more: boolean;
}

export interface SelectorValues {
  name: string;
  description: string;
  expression: string;
}

export const EMPTY_SELECTOR: SelectorValues = { name: '', description: '', expression: '' };

export function fromSelector(selector: ContentSelector): SelectorValues {
  return {
    name: selector.name,
    description: selector.description ?? '',
    expression: selector.expression,
  };
}

export function toCreateRequest(values: SelectorValues) {
  return {
    name: values.name.trim(),
    description: values.description.trim(),
    expression: values.expression.trim(),
  };
}

export function toUpdateRequest(values: SelectorValues) {
  return { description: values.description.trim(), expression: values.expression.trim() };
}

const NAME = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;

export function validateSelector(
  t: Translate,
  values: SelectorValues,
  creating: boolean,
): Partial<Record<keyof SelectorValues, string>> {
  const errors: Partial<Record<keyof SelectorValues, string>> = {};
  if (creating) {
    if (values.name.trim() === '') {
      errors.name = t('selectors.form.required');
    } else if (!NAME.test(values.name.trim())) {
      errors.name = t('selectors.form.nameRule');
    }
  }
  if (values.expression.trim() === '') {
    errors.expression = t('selectors.form.required');
  }
  return errors;
}

export function selectorHref(name: string): string {
  return path`/admin/repository/selectors/${name}`;
}

export function browseHref(repository: string, assetPath: string): string {
  const segments = assetPath
    .split('/')
    .filter((segment) => segment !== '')
    .map((segment) => encodeURIComponent(segment));
  return [`/browse/${encodeURIComponent(repository)}`, ...segments].join('/');
}

export function insertAt(
  text: string,
  insertion: string,
  start: number,
  end: number,
): { text: string; caret: number } {
  const before = text.slice(0, start);
  const after = text.slice(end);
  const spaceBefore = before !== '' && !/\s$/.test(before) && !/^\s/.test(insertion) ? ' ' : '';
  const spaceAfter = after !== '' && !/^\s/.test(after) && !/\s$/.test(insertion) ? ' ' : '';
  const inserted = `${spaceBefore}${insertion}${spaceAfter}`;
  return {
    text: `${before}${inserted}${after}`,
    caret: before.length + spaceBefore.length + insertion.length,
  };
}
