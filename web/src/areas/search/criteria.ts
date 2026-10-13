import { withQuery } from '../../api';
import { SEARCH } from './model';

export type Operator = 'is' | 'startsWith' | 'contains';
export type CriterionKind = 'base' | 'text' | 'checksum' | 'choice';

export interface CriterionDefinition {
  key: string;
  label: string;
  kind: CriterionKind;
  example?: string;
  format?: string;
  options?: ReadonlyArray<{ value: string; label: string }>;
}

export interface Criterion {
  id: number;
  key: string;
  operator: Operator;
  value: string;
}

export const OPERATOR_LABELS: Record<Operator, string> = {
  is: 'search.operator.is',
  startsWith: 'search.operator.startsWith',
  contains: 'search.operator.contains',
};

export const BASE_CRITERIA: readonly CriterionDefinition[] = [
  { key: 'q', label: 'search.criterion.keyword', kind: 'base', example: 'spring-boot' },
  { key: 'format', label: 'search.criterion.format', kind: 'base', example: 'maven2, npm, docker' },
  { key: 'repository', label: 'search.criterion.repository', kind: 'base', example: 'maven-central' },
  { key: 'group', label: 'search.criterion.group', kind: 'base', example: 'org.apache.commons' },
  { key: 'name', label: 'search.criterion.name', kind: 'base', example: 'commons-lang3' },
  { key: 'version', label: 'search.criterion.version', kind: 'base', example: '3.14.0' },
];

const MORE: readonly CriterionDefinition[] = [
  { key: 'md5', label: 'search.criterion.md5', kind: 'checksum' },
  { key: 'sha1', label: 'search.criterion.sha1', kind: 'checksum' },
  { key: 'sha256', label: 'search.criterion.sha256', kind: 'checksum' },
  { key: 'sha512', label: 'search.criterion.sha512', kind: 'checksum' },
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
    key: 'nuget.id',
    label: 'search.criterion.nugetId',
    kind: 'text',
    format: 'nuget',
    example: 'Newtonsoft.Json',
  },
  { key: 'nuget.tags', label: 'search.criterion.nugetTags', kind: 'text', format: 'nuget', example: 'json' },
  {
    key: 'pypi.classifiers',
    label: 'search.criterion.pypiClassifiers',
    kind: 'text',
    format: 'pypi',
    example: 'Programming Language :: Python :: 3',
  },
  { key: 'pypi.description', label: 'search.criterion.pypiDescription', kind: 'text', format: 'pypi' },
  { key: 'pypi.keywords', label: 'search.criterion.pypiKeywords', kind: 'text', format: 'pypi' },
  { key: 'pypi.summary', label: 'search.criterion.pypiSummary', kind: 'text', format: 'pypi' },
  {
    key: 'helm.appVersion',
    label: 'search.criterion.helmAppVersion',
    kind: 'text',
    format: 'helm',
    example: '1.16.0',
  },
  { key: 'helm.description', label: 'search.criterion.helmDescription', kind: 'text', format: 'helm' },
  {
    key: 'helm.keywords',
    label: 'search.criterion.helmKeywords',
    kind: 'text',
    format: 'helm',
    example: 'web',
  },
  {
    key: 'terraform.namespace',
    label: 'search.criterion.terraformNamespace',
    kind: 'text',
    format: 'terraform',
    example: 'hashicorp',
  },
  {
    key: 'terraform.name',
    label: 'search.criterion.terraformName',
    kind: 'text',
    format: 'terraform',
    example: 'vpc',
  },
  {
    key: 'terraform.provider',
    label: 'search.criterion.terraformProvider',
    kind: 'text',
    format: 'terraform',
    example: 'aws',
  },
  {
    key: 'terraform.kind',
    label: 'search.criterion.terraformKind',
    kind: 'text',
    format: 'terraform',
    example: 'module',
  },
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

export const CRITERIA: readonly CriterionDefinition[] = [...BASE_CRITERIA, ...MORE];

export const FIRST_KEY = 'q';

export function definition(key: string): CriterionDefinition | undefined {
  return CRITERIA.find((candidate) => candidate.key === key);
}

export function moreCriteria(formats: readonly string[], keep: string): CriterionDefinition[] {
  return MORE.filter(
    (candidate) => !candidate.format || formats.includes(candidate.format) || candidate.key === keep,
  );
}

export function operatorsOf(key: string): Operator[] {
  switch (definition(key)?.kind) {
    case 'base':
      return ['is', 'startsWith'];
    case 'text':
      return ['is', 'startsWith', 'contains'];
    default:
      return ['is'];
  }
}

export function withKey(criterion: Criterion, key: string): Criterion {
  const operator = operatorsOf(key).includes(criterion.operator) ? criterion.operator : 'is';
  const choice = definition(key)?.kind === 'choice' || definition(criterion.key)?.kind === 'choice';
  return { ...criterion, key, operator, value: choice ? '' : criterion.value };
}

export function blankCriterion(id: number): Criterion {
  return { id, key: FIRST_KEY, operator: 'is', value: '' };
}

export function buildValue(criterion: Pick<Criterion, 'key' | 'operator' | 'value'>): string {
  const value = criterion.value.trim();
  if (value === '' || !operatorsOf(criterion.key).includes(criterion.operator)) {
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
  const allowed = operatorsOf(key);
  if (
    allowed.includes('contains') &&
    raw.length > 2 &&
    raw.startsWith('*') &&
    raw.endsWith('*') &&
    !raw.slice(1, -1).includes('*')
  ) {
    return { operator: 'contains', value: raw.slice(1, -1) };
  }
  if (
    allowed.includes('startsWith') &&
    raw.length > 1 &&
    raw.endsWith('*') &&
    !raw.slice(0, -1).includes('*')
  ) {
    return { operator: 'startsWith', value: raw.slice(0, -1) };
  }
  return { operator: 'is', value: raw };
}

export function readCriteria(params: URLSearchParams): Criterion[] {
  const criteria: Criterion[] = [];
  for (const candidate of CRITERIA) {
    const raw = params.get(candidate.key);
    if (raw !== null && raw.trim() !== '') {
      criteria.push({ id: criteria.length + 1, key: candidate.key, ...parseValue(candidate.key, raw) });
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
