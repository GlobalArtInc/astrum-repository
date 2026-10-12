import { withQuery } from '../../api';
import { SEARCH, SEARCH_ASSETS } from './model';

export type Operator = 'is' | 'startsWith' | 'contains';
export type CriterionKind = 'keyword' | 'text' | 'choice' | 'format' | 'repository' | 'checksum';
export type Mode = 'components' | 'assets';

export interface CriterionDefinition {
  key: string;
  label: string;
  kind: CriterionKind;
  format?: string;
  example?: string;
  length?: number;
  options?: ReadonlyArray<{ value: string; label: string }>;
}

export interface Criterion {
  id: number;
  key: string;
  operator: Operator;
  value: string;
}

const LIKE: Operator[] = ['is', 'startsWith', 'contains'];

export const CRITERIA: readonly CriterionDefinition[] = [
  { key: 'q', label: 'search.criterion.keyword', kind: 'keyword', example: 'commons' },
  { key: 'format', label: 'search.criterion.format', kind: 'format' },
  { key: 'repository', label: 'search.criterion.repository', kind: 'repository' },
  { key: 'group', label: 'search.criterion.group', kind: 'text', example: 'org.example' },
  { key: 'name', label: 'search.criterion.name', kind: 'text', example: 'my-library' },
  { key: 'version', label: 'search.criterion.version', kind: 'text', example: '1.0.0' },
  { key: 'md5', label: 'search.criterion.md5', kind: 'checksum', length: 32 },
  { key: 'sha1', label: 'search.criterion.sha1', kind: 'checksum', length: 40 },
  { key: 'sha256', label: 'search.criterion.sha256', kind: 'checksum', length: 64 },
  { key: 'sha512', label: 'search.criterion.sha512', kind: 'checksum', length: 128 },
  {
    key: 'maven.groupId',
    label: 'search.criterion.mavenGroupId',
    kind: 'text',
    format: 'maven2',
    example: 'org.example',
  },
  {
    key: 'maven.artifactId',
    label: 'search.criterion.mavenArtifactId',
    kind: 'text',
    format: 'maven2',
    example: 'my-library',
  },
  {
    key: 'maven.baseVersion',
    label: 'search.criterion.mavenBaseVersion',
    kind: 'text',
    format: 'maven2',
    example: '1.0-SNAPSHOT',
  },
  {
    key: 'maven.classifier',
    label: 'search.criterion.mavenClassifier',
    kind: 'text',
    format: 'maven2',
    example: 'sources',
  },
  {
    key: 'maven.extension',
    label: 'search.criterion.mavenExtension',
    kind: 'text',
    format: 'maven2',
    example: 'jar',
  },
  { key: 'npm.scope', label: 'search.criterion.npmScope', kind: 'text', format: 'npm', example: 'acme' },
  {
    key: 'pypi.classifiers',
    label: 'search.criterion.pypiClassifiers',
    kind: 'text',
    format: 'pypi',
    example: 'Programming Language :: Python :: 3',
  },
  {
    key: 'pypi.description',
    label: 'search.criterion.pypiDescription',
    kind: 'text',
    format: 'pypi',
  },
  { key: 'pypi.keywords', label: 'search.criterion.pypiKeywords', kind: 'text', format: 'pypi' },
  { key: 'pypi.summary', label: 'search.criterion.pypiSummary', kind: 'text', format: 'pypi' },
  {
    key: 'docker.imageName',
    label: 'search.criterion.dockerImageName',
    kind: 'text',
    format: 'docker',
    example: 'team/app',
  },
  {
    key: 'docker.imageTag',
    label: 'search.criterion.dockerImageTag',
    kind: 'text',
    format: 'docker',
    example: 'latest',
  },
  {
    key: 'docker.contentDigest',
    label: 'search.criterion.dockerContentDigest',
    kind: 'text',
    format: 'docker',
    example: 'sha256:…',
  },
  {
    key: 'docker.mediaType',
    label: 'search.criterion.dockerMediaType',
    kind: 'text',
    format: 'docker',
    example: 'application/vnd.oci.image.manifest.v1+json',
  },
  {
    key: 'docker.artifactType',
    label: 'search.criterion.dockerArtifactType',
    kind: 'text',
    format: 'docker',
    example: 'application/vnd.dev.sigstore.bundle.v0.3+json',
  },
  {
    key: 'docker.artifactKind',
    label: 'search.criterion.dockerArtifactKind',
    kind: 'choice',
    format: 'docker',
    options: [
      { value: 'image', label: 'search.artifactKind.image' },
      { value: 'helm', label: 'search.artifactKind.helm' },
      { value: 'signature', label: 'search.artifactKind.signature' },
      { value: 'sbom', label: 'search.artifactKind.sbom' },
      { value: 'attestation', label: 'search.artifactKind.attestation' },
      { value: 'wasm', label: 'search.artifactKind.wasm' },
      { value: 'model', label: 'search.artifactKind.model' },
      { value: 'other', label: 'search.artifactKind.other' },
    ],
  },
];

export function definition(key: string): CriterionDefinition | undefined {
  return CRITERIA.find((candidate) => candidate.key === key);
}

export function availableCriteria(formats: readonly string[]): CriterionDefinition[] {
  return CRITERIA.filter((candidate) => !candidate.format || formats.includes(candidate.format));
}

export function operatorsOf(key: string): Operator[] {
  const kind = definition(key)?.kind;
  if (kind === 'text') {
    return LIKE;
  }
  return kind === 'keyword' ? ['contains'] : ['is'];
}

export function defaultOperator(key: string): Operator {
  return operatorsOf(key)[0] ?? 'is';
}

export function buildValue(criterion: Pick<Criterion, 'key' | 'operator' | 'value'>): string {
  const value = criterion.value.trim();
  if (value === '' || definition(criterion.key)?.kind !== 'text') {
    return value;
  }
  switch (criterion.operator) {
    case 'startsWith':
      return value.endsWith('*') ? value : `${value}*`;
    case 'contains':
      return `*${value.replace(/^\*+|\*+$/g, '')}*`;
    default:
      return value;
  }
}

export function parseValue(key: string, raw: string): Pick<Criterion, 'operator' | 'value'> {
  if (definition(key)?.kind !== 'text') {
    return { operator: defaultOperator(key), value: raw };
  }
  if (raw.length > 2 && raw.startsWith('*') && raw.endsWith('*')) {
    return { operator: 'contains', value: raw.slice(1, -1) };
  }
  if (raw.length > 1 && raw.endsWith('*') && !raw.slice(0, -1).includes('*')) {
    return { operator: 'startsWith', value: raw.slice(0, -1) };
  }
  return { operator: 'is', value: raw };
}

export function readCriteria(params: URLSearchParams): { criteria: Criterion[]; mode: Mode } {
  const criteria: Criterion[] = [];
  for (const candidate of CRITERIA) {
    const raw = params.get(candidate.key);
    if (raw !== null && raw.trim() !== '') {
      criteria.push({ id: criteria.length + 1, key: candidate.key, ...parseValue(candidate.key, raw) });
    }
  }
  return { criteria, mode: params.get('mode') === 'assets' ? 'assets' : 'components' };
}

export function writeCriteria(criteria: readonly Criterion[], mode: Mode): URLSearchParams {
  const params = new URLSearchParams();
  for (const criterion of criteria) {
    const value = buildValue(criterion);
    if (value !== '' && definition(criterion.key) && !params.has(criterion.key)) {
      params.set(criterion.key, value);
    }
  }
  if (mode === 'assets') {
    params.set('mode', 'assets');
  }
  return params;
}

export function criteriaPath(params: URLSearchParams): string | null {
  const { criteria, mode } = readCriteria(params);
  if (criteria.length === 0) {
    return null;
  }
  const query: Record<string, string> = {};
  for (const criterion of criteria) {
    query[criterion.key] = buildValue(criterion);
  }
  return withQuery(mode === 'assets' ? SEARCH_ASSETS : SEARCH, query);
}

export type CriterionProblem = 'empty' | 'checksum';

export function checkCriterion(criterion: Criterion): CriterionProblem | null {
  const found = definition(criterion.key);
  const value = criterion.value.trim();
  if (value === '') {
    return 'empty';
  }
  if (found?.kind === 'checksum' && !new RegExp(`^[0-9a-fA-F]{${found.length}}$`).test(value)) {
    return 'checksum';
  }
  return null;
}

export function nextKey(criteria: readonly Criterion[], formats: readonly string[]): string | null {
  const used = new Set(criteria.map((criterion) => criterion.key));
  return availableCriteria(formats).find((candidate) => !used.has(candidate.key))?.key ?? null;
}
