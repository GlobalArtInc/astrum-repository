import type { FieldError } from '../../api';
import type { UploadFieldSpec, UploadSpec } from './model';

export interface AssetDraft {
  key: number;
  file: File | null;
  values: Record<string, string>;
}

export interface UploadDraft {
  component: Record<string, string>;
  assets: AssetDraft[];
}

export interface Problem {
  message: string;
  vars?: Record<string, string | number>;
}

export type Problems = Record<string, Problem>;

export interface FieldGroup {
  title: string | null;
  fields: UploadFieldSpec[];
}

export const FILE = 'FILE';
export const BOOLEAN = 'BOOLEAN';

let lastKey = 0;

export function nextAssetKey(): number {
  lastKey += 1;
  return lastKey;
}

export function optionValues(field: UploadFieldSpec): string[] {
  if (!field.options) {
    return [];
  }
  return Array.isArray(field.options) ? field.options : Object.keys(field.options);
}

function initialValue(field: UploadFieldSpec): string {
  if (field.type === BOOLEAN) {
    return 'false';
  }
  return '';
}

export function emptyAsset(spec: UploadSpec): AssetDraft {
  const values: Record<string, string> = {};
  for (const field of assetTextFields(spec)) {
    values[field.name] = initialValue(field);
  }
  return { key: nextAssetKey(), file: null, values };
}

export function emptyDraft(spec: UploadSpec): UploadDraft {
  const component: Record<string, string> = {};
  for (const field of spec.componentFields) {
    component[field.name] = initialValue(field);
  }
  return { component, assets: [emptyAsset(spec)] };
}

export function componentGroups(spec: UploadSpec): FieldGroup[] {
  const groups: FieldGroup[] = [];
  for (const field of spec.componentFields) {
    const title = field.group ?? null;
    const found = groups.find((group) => group.title === title);
    if (found) {
      found.fields.push(field);
    } else {
      groups.push({ title, fields: [field] });
    }
  }
  return groups;
}

export function fileField(spec: UploadSpec): UploadFieldSpec | undefined {
  return spec.assetFields.find((field) => field.type === FILE);
}

export function assetTextFields(spec: UploadSpec): UploadFieldSpec[] {
  return spec.assetFields.filter((field) => field.type !== FILE);
}

export function componentFieldName(spec: UploadSpec, field: string): string {
  return `${spec.format}.${field}`;
}

export function assetFileName(spec: UploadSpec, index: number): string {
  return spec.multipleUpload ? `${spec.format}.asset${index + 1}` : `${spec.format}.asset`;
}

export function assetFieldName(spec: UploadSpec, index: number, field: string): string {
  return `${assetFileName(spec, index)}.${field}`;
}

const MAVEN_PART = /^[^\s/\\:]+$/;

export function splitMavenFileName(
  fileName: string,
  artifactId: string,
  version: string,
): { classifier: string; extension: string } {
  const prefix = artifactId && version ? `${artifactId}-${version}` : '';
  if (prefix && fileName.startsWith(prefix)) {
    const rest = fileName.slice(prefix.length);
    const match = /^(?:-([^.]+))?\.(.+)$/.exec(rest);
    if (match) {
      return { classifier: match[1] ?? '', extension: match[2] ?? '' };
    }
  }
  const compound = /\.(tar\.gz|tar\.bz2|tar\.xz)$/i.exec(fileName);
  if (compound?.[1]) {
    return { classifier: '', extension: compound[1].toLowerCase() };
  }
  const dot = fileName.lastIndexOf('.');
  return { classifier: '', extension: dot > 0 ? fileName.slice(dot + 1) : '' };
}

export function suggestAssetValues(
  spec: UploadSpec,
  draft: UploadDraft,
  asset: AssetDraft,
  file: File,
): Record<string, string> {
  const values = { ...asset.values };
  const fields = new Set(assetTextFields(spec).map((field) => field.name));
  if (fields.has('filename') && (values.filename ?? '') === '') {
    values.filename = file.name;
  }
  if (spec.format === 'maven2' && fields.has('extension')) {
    const split = splitMavenFileName(
      file.name,
      draft.component.artifactId ?? '',
      draft.component.version ?? '',
    );
    if ((values.extension ?? '') === '') {
      values.extension = split.extension;
    }
    if (fields.has('classifier') && (values.classifier ?? '') === '') {
      values.classifier = split.classifier;
    }
  }
  return values;
}

export const ACCEPTED: Record<string, string[]> = {
  npm: ['.tgz', '.tar.gz'],
  pypi: ['.whl', '.tar.gz', '.tgz', '.zip', '.egg'],
};

export function hasPom(spec: UploadSpec, draft: UploadDraft): boolean {
  return (
    spec.format === 'maven2' &&
    draft.assets.some(
      (asset) =>
        asset.file !== null &&
        (asset.values.extension ?? '').trim() === 'pom' &&
        (asset.values.classifier ?? '').trim() === '',
    )
  );
}

export function isRequired(spec: UploadSpec, draft: UploadDraft, field: UploadFieldSpec): boolean {
  if (field.optional || field.type === BOOLEAN) {
    return false;
  }
  if (spec.format === 'maven2' && field.group && hasPom(spec, draft)) {
    return false;
  }
  return true;
}

function hasBadSegment(path: string): boolean {
  return path
    .split('/')
    .some(
      (segment, index, all) =>
        segment === '..' || segment === '.' || (segment === '' && index > 0 && index < all.length - 1),
    );
}

export function validateDraft(spec: UploadSpec, draft: UploadDraft): Problems {
  const problems: Problems = {};
  for (const field of spec.componentFields) {
    const value = (draft.component[field.name] ?? '').trim();
    const name = componentFieldName(spec, field.name);
    if (value === '' && isRequired(spec, draft, field)) {
      problems[name] = { message: 'upload.validation.required' };
    }
  }
  const files = fileField(spec);
  draft.assets.forEach((asset, index) => {
    if (files && !asset.file) {
      problems[assetFileName(spec, index)] = { message: 'upload.validation.file' };
    }
    for (const field of assetTextFields(spec)) {
      const value = (asset.values[field.name] ?? '').trim();
      if (value === '' && isRequired(spec, draft, field)) {
        problems[assetFieldName(spec, index, field.name)] = { message: 'upload.validation.required' };
      }
    }
    const accepted = ACCEPTED[spec.format];
    if (asset.file && accepted) {
      const lower = asset.file.name.toLowerCase();
      if (!accepted.some((suffix) => lower.endsWith(suffix))) {
        problems[assetFileName(spec, index)] = {
          message: 'upload.validation.fileType',
          vars: { types: accepted.join(', ') },
        };
      }
    }
  });
  if (spec.format === 'maven2') {
    validateMaven(spec, draft, problems);
  }
  if (spec.format === 'raw') {
    validateRaw(spec, draft, problems);
  }
  return problems;
}

function validateMaven(spec: UploadSpec, draft: UploadDraft, problems: Problems): void {
  for (const name of ['groupId', 'artifactId', 'version', 'packaging']) {
    const value = (draft.component[name] ?? '').trim();
    const key = componentFieldName(spec, name);
    if (value !== '' && !problems[key] && !MAVEN_PART.test(value)) {
      problems[key] = { message: 'upload.validation.mavenPart' };
    }
  }
  const group = (draft.component.groupId ?? '').trim();
  const groupKey = componentFieldName(spec, 'groupId');
  if (group !== '' && !problems[groupKey] && group.split('.').some((part) => part === '')) {
    problems[groupKey] = { message: 'upload.validation.mavenGroup' };
  }
  const seen = new Map<string, number>();
  draft.assets.forEach((asset, index) => {
    for (const name of ['classifier', 'extension']) {
      const value = (asset.values[name] ?? '').trim();
      const key = assetFieldName(spec, index, name);
      if (value !== '' && !problems[key] && !MAVEN_PART.test(value)) {
        problems[key] = { message: 'upload.validation.mavenPart' };
      }
    }
    const pair = `${(asset.values.classifier ?? '').trim()}|${(asset.values.extension ?? '').trim()}`;
    const first = seen.get(pair);
    if (first !== undefined && (asset.values.extension ?? '').trim() !== '') {
      problems[assetFieldName(spec, index, 'extension')] = {
        message: 'upload.validation.duplicateAsset',
        vars: { number: first + 1 },
      };
    } else {
      seen.set(pair, index);
    }
  });
}

function validateRaw(spec: UploadSpec, draft: UploadDraft, problems: Problems): void {
  const directory = (draft.component.directory ?? '').trim();
  const directoryKey = componentFieldName(spec, 'directory');
  if (directory !== '' && !problems[directoryKey] && hasBadSegment(directory)) {
    problems[directoryKey] = { message: 'upload.validation.path' };
  }
  const seen = new Map<string, number>();
  draft.assets.forEach((asset, index) => {
    const name = (asset.values.filename ?? '').trim().replace(/^\/+/, '');
    const key = assetFieldName(spec, index, 'filename');
    if (name === '' || problems[key]) {
      return;
    }
    if (hasBadSegment(name) || name.endsWith('/')) {
      problems[key] = { message: 'upload.validation.path' };
      return;
    }
    const first = seen.get(name);
    if (first !== undefined) {
      problems[key] = { message: 'upload.validation.duplicateName', vars: { number: first + 1 } };
    } else {
      seen.set(name, index);
    }
  });
}

export function toFormData(spec: UploadSpec, draft: UploadDraft): FormData {
  const data = new FormData();
  const pom = hasPom(spec, draft);
  for (const field of spec.componentFields) {
    const value = (draft.component[field.name] ?? '').trim();
    if (field.type === BOOLEAN) {
      if (value === 'true' && !(pom && field.name === 'generate-pom')) {
        data.append(componentFieldName(spec, field.name), 'true');
      }
    } else if (value !== '') {
      data.append(componentFieldName(spec, field.name), value);
    }
  }
  draft.assets.forEach((asset, index) => {
    for (const field of assetTextFields(spec)) {
      const value = (asset.values[field.name] ?? '').trim();
      if (field.type === BOOLEAN) {
        if (value === 'true') {
          data.append(assetFieldName(spec, index, field.name), 'true');
        }
      } else if (value !== '') {
        data.append(assetFieldName(spec, index, field.name), value);
      }
    }
    if (asset.file) {
      data.append(assetFileName(spec, index), asset.file, asset.file.name);
    }
  });
  return data;
}

export function knownFieldNames(spec: UploadSpec, draft: UploadDraft): string[] {
  const names = spec.componentFields.map((field) => componentFieldName(spec, field.name));
  draft.assets.forEach((_, index) => {
    names.push(assetFileName(spec, index));
    for (const field of assetTextFields(spec)) {
      names.push(assetFieldName(spec, index, field.name));
    }
  });
  return names;
}

export function placeServerErrors(
  names: readonly string[],
  errors: readonly FieldError[],
): { fields: Record<string, string>; general: string[] } {
  const fields: Record<string, string> = {};
  const general: string[] = [];
  const longestFirst = [...names].sort((left, right) => right.length - left.length);
  for (const error of errors) {
    const id = error.id.replace(/^PARAMETER\s+/, '').trim();
    const target =
      names.find((name) => name === id) ??
      longestFirst.find(
        (name) =>
          error.message.startsWith(`${name} `) ||
          error.message.startsWith(`${name}:`) ||
          error.message.startsWith(`${name},`),
      );
    if (target && !fields[target]) {
      fields[target] = error.message;
    } else {
      general.push(error.message);
    }
  }
  return { fields, general };
}

export function folderGuess(spec: UploadSpec, draft: UploadDraft): string {
  if (spec.format === 'raw') {
    return (draft.component.directory ?? '').trim().replace(/^\/+|\/+$/g, '');
  }
  if (spec.format === 'maven2') {
    const group = (draft.component.groupId ?? '').trim();
    const artifact = (draft.component.artifactId ?? '').trim();
    const version = (draft.component.version ?? '').trim();
    if (group && artifact && version) {
      return [...group.split('.'), artifact, version].join('/');
    }
  }
  return '';
}
