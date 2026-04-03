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
  'conda.channel': 'upload.field.channel',
  'r.pathId': 'upload.field.packagePath',
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
  'conda.channel': 'upload.hint.condaChannel',
  'r.pathId': 'upload.hint.packagePath',
};

const PLACEHOLDERS: Record<string, string> = {
  'raw.directory': 'upload.placeholder.directory',
  'maven2.groupId': 'upload.placeholder.groupId',
  'maven2.artifactId': 'upload.placeholder.artifactId',
  'maven2.version': 'upload.placeholder.version',
  'r.pathId': 'upload.placeholder.packagePath',
};

const SECTIONS: Record<string, string> = {
  raw: 'upload.group.raw',
  maven2: 'upload.group.coordinates',
};

export const PACKAGING = ['jar', 'war', 'ear', 'pom', 'maven-plugin', 'ejb', 'bundle'];

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

export function fieldPlaceholder(t: Translate, format: string, field: UploadFieldSpec): string | undefined {
  const key = PLACEHOLDERS[`${format}.${field.name}`];
  return key ? t(key) : undefined;
}

export function sectionTitle(t: Translate, format: string): string {
  const key = SECTIONS[format];
  return key ? t(key) : t('upload.group.component');
}
