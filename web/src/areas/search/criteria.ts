import { withQuery } from '../../api';
import { SEARCH } from './model';

export type Operator = 'is' | 'startsWith';

export interface CriterionDefinition {
  key: string;
  label: string;
  example: string;
}

export interface Criterion {
  id: number;
  key: string;
  operator: Operator;
  value: string;
}

export const OPERATORS: ReadonlyArray<{ value: Operator; label: string }> = [
  { value: 'is', label: 'search.operator.is' },
  { value: 'startsWith', label: 'search.operator.startsWith' },
];

export const CRITERIA: readonly CriterionDefinition[] = [
  { key: 'q', label: 'search.criterion.keyword', example: 'spring-boot' },
  { key: 'format', label: 'search.criterion.format', example: 'maven2, npm, docker' },
  { key: 'repository', label: 'search.criterion.repository', example: 'maven-central' },
  { key: 'group', label: 'search.criterion.group', example: 'org.apache.commons' },
  { key: 'name', label: 'search.criterion.name', example: 'commons-lang3' },
  { key: 'version', label: 'search.criterion.version', example: '3.14.0' },
];

export const FIRST_KEY = 'q';

export function definition(key: string): CriterionDefinition | undefined {
  return CRITERIA.find((candidate) => candidate.key === key);
}

export function blankCriterion(id: number): Criterion {
  return { id, key: FIRST_KEY, operator: 'is', value: '' };
}

export function buildValue(criterion: Pick<Criterion, 'operator' | 'value'>): string {
  const value = criterion.value.trim();
  if (value === '' || criterion.operator !== 'startsWith' || value.endsWith('*')) {
    return value;
  }
  return `${value}*`;
}

export function parseValue(raw: string): Pick<Criterion, 'operator' | 'value'> {
  if (raw.length > 1 && raw.endsWith('*') && !raw.slice(0, -1).includes('*')) {
    return { operator: 'startsWith', value: raw.slice(0, -1) };
  }
  return { operator: 'is', value: raw };
}

export function readCriteria(params: URLSearchParams): Criterion[] {
  const criteria: Criterion[] = [];
  for (const candidate of CRITERIA) {
    const raw = params.get(candidate.key);
    if (raw !== null && raw.trim() !== '') {
      criteria.push({ id: criteria.length + 1, key: candidate.key, ...parseValue(raw) });
    }
  }
  return criteria;
}

export function writeCriteria(criteria: readonly Criterion[]): URLSearchParams {
  const params = new URLSearchParams();
  for (const criterion of criteria) {
    const value = buildValue(criterion);
    if (value !== '' && definition(criterion.key) && !params.has(criterion.key)) {
      params.set(criterion.key, value);
    }
  }
  return params;
}

export function criteriaPath(params: URLSearchParams): string | null {
  const criteria = readCriteria(params);
  if (criteria.length === 0) {
    return null;
  }
  const query: Record<string, string> = {};
  for (const criterion of criteria) {
    query[criterion.key] = buildValue(criterion);
  }
  return withQuery(SEARCH, query);
}
