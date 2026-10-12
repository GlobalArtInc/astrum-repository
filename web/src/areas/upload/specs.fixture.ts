import type { UploadSpec } from './model';

const field = (name: string, extra: object = {}) => ({
  name,
  type: 'STRING',
  description: null,
  optional: false,
  group: null,
  groupHelpText: null,
  options: null,
  ...extra,
});

export const RAW: UploadSpec = {
  format: 'raw',
  multipleUpload: true,
  componentFields: [
    field('directory', {
      description: 'Destination for uploaded files (e.g. /path/to/files/)',
      group: 'Component attributes',
    }),
  ],
  assetFields: [field('filename'), field('asset', { type: 'FILE' })],
};

export const MAVEN: UploadSpec = {
  format: 'maven2',
  multipleUpload: true,
  componentFields: [
    field('groupId', { group: 'Component coordinates' }),
    field('artifactId', { group: 'Component coordinates' }),
    field('version', { group: 'Component coordinates' }),
    field('generate-pom', { type: 'BOOLEAN', optional: true, group: 'Component coordinates' }),
    field('packaging', { optional: true, group: 'Component coordinates' }),
  ],
  assetFields: [
    field('classifier', { optional: true }),
    field('extension'),
    field('asset', { type: 'FILE' }),
  ],
};

export const NPM: UploadSpec = {
  format: 'npm',
  multipleUpload: false,
  componentFields: [],
  assetFields: [field('asset', { type: 'FILE' })],
};

export const PYPI: UploadSpec = { ...NPM, format: 'pypi' };

export const TERRAFORM: UploadSpec = {
  format: 'terraform',
  multipleUpload: false,
  componentFields: [
    field('uploadType', { optional: true, group: 'Coordinates', options: ['module', 'provider'] }),
    field('namespace', { group: 'Coordinates', description: 'Organization of the module' }),
  ],
  assetFields: [field('asset', { type: 'FILE' })],
};
