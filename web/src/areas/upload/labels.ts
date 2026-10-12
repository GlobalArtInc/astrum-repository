import type { Translate } from '../../i18n';
import type { UploadFieldSpec } from './model';

const LABELS: Record<string, string> = {
  'raw.directory': 'upload.field.directory',
  'raw.filename': 'upload.field.filename',
  'maven2.groupId': 'upload.field.groupId',
  'maven2.artifactId': 'upload.field.artifactId',
  'maven2.version': 'upload.field.version',
  'maven2.generate-pom': 'upload.field.generatePom',
  'maven2.packaging': 'upload.field.packaging',
  'maven2.classifier': 'upload.field.classifier',
  'maven2.extension': 'upload.field.extension',
  asset: 'upload.field.file',
  directory: 'upload.field.directory',
  filename: 'upload.field.filename',
  version: 'upload.field.version',
  name: 'upload.field.name',
};

const HINTS: Record<string, string> = {
  'raw.directory': 'upload.hint.directory',
  'raw.filename': 'upload.hint.filename',
  'maven2.groupId': 'upload.hint.groupId',
  'maven2.artifactId': 'upload.hint.artifactId',
  'maven2.version': 'upload.hint.version',
  'maven2.generate-pom': 'upload.hint.generatePom',
  'maven2.packaging': 'upload.hint.packaging',
  'maven2.classifier': 'upload.hint.classifier',
  'maven2.extension': 'upload.hint.extension',
};

const GROUPS: Record<string, string> = {
  'Component coordinates': 'upload.group.coordinates',
  'Component attributes': 'upload.group.attributes',
};

const INTROS: Record<string, string> = {
  maven2: 'upload.intro.maven2',
  npm: 'upload.intro.npm',
  pypi: 'upload.intro.pypi',
  raw: 'upload.intro.raw',
};

export function humanize(name: string): string {
  const spaced = name
    .replace(/[-_.]+/g, ' ')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .trim();
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

export function fieldLabel(t: Translate, format: string, field: UploadFieldSpec): string {
  const key = LABELS[`${format}.${field.name}`] ?? LABELS[field.name];
  return key ? t(key) : humanize(field.name);
}

export function fieldHint(t: Translate, format: string, field: UploadFieldSpec): string | undefined {
  const key = HINTS[`${format}.${field.name}`];
  if (key) {
    return t(key);
  }
  return field.description ?? undefined;
}

export function groupTitle(t: Translate, title: string | null): string | null {
  if (title === null) {
    return null;
  }
  const key = GROUPS[title];
  return key ? t(key) : title;
}

export function formatIntro(t: Translate, format: string): string | null {
  const key = INTROS[format];
  return key ? t(key) : null;
}
