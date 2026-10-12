import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { makeTranslate } from '../../i18n';
import type { StoreType } from './api';
import {
  formField,
  fromDocument,
  MB,
  onFormFields,
  type StoreDocument,
  type StoreValues,
  storedSecret,
  template,
  toDocument,
  validate,
} from './model';

const t = makeTranslate('en');

const FILE: StoreDocument = { softQuota: { type: 'spaceUsedQuota', limit: 10_500_000 }, path: 'one' };
const GROUP: StoreDocument = {
  name: 'pool',
  softQuota: null,
  members: ['one', 'two'],
  fillPolicy: 'roundRobin',
};
const S3: StoreDocument = {
  name: 'cloud',
  softQuota: null,
  bucketConfiguration: {
    activeRegion: 'eu-central-1',
    bucket: { region: 'eu-central-1', name: 'astrum-blobs', prefix: 'prod', expiration: 3 },
    bucketSecurity: { accessKeyId: 'AKIAEXAMPLE', role: null },
    encryption: { encryptionType: 'kmsManagedEncryption', encryptionKey: 'key-1' },
    advancedBucketConnection: {
      endpoint: 'https://s3.example.org',
      signerType: 'AWSS3V4SignerType',
      forcePathStyle: true,
      maxConnectionPoolSize: 20,
    },
    failoverBuckets: [{ region: 'eu-west-1', bucketName: 'astrum-failover' }],
    preSignedUrlEnabled: false,
  },
  type: 'S3',
};
const MINIMAL_S3: StoreDocument = {
  name: 'plain',
  softQuota: null,
  bucketConfiguration: {
    activeRegion: 'us-east-1',
    bucket: { region: 'DEFAULT', name: 'plain-bucket', prefix: '' },
    bucketSecurity: null,
    encryption: null,
    advancedBucketConnection: null,
    failoverBuckets: [],
    preSignedUrlEnabled: false,
  },
  type: 'S3',
};

function edit(document: StoreDocument, type: StoreType, change: Partial<StoreValues>): StoreDocument {
  return toDocument({ ...fromDocument(document), ...change }, document, type);
}

describe('blob store documents', () => {
  it.each([
    ['file', FILE],
    ['group', GROUP],
    ['s3', S3],
    ['s3', MINIMAL_S3],
  ] as const)('a %s store is saved unchanged when nothing is edited', (type, document) => {
    expect(toDocument(fromDocument(document), document, type)).toEqual(document);
  });

  it('reads the soft quota in megabytes and keeps the exact bytes', () => {
    const values = fromDocument(FILE);
    expect(values.quotaEnabled).toBe(true);
    expect(values.quotaLimit).toBeCloseTo(10.01, 2);
    expect(fromDocument({ softQuota: { type: 'spaceUsedQuota', limit: 1 } }).quotaLimit).toBeGreaterThan(0);
    expect(edit(FILE, 'file', { quotaType: 'spaceRemainingQuota' }).softQuota).toEqual({
      type: 'spaceRemainingQuota',
      limit: 10_500_000,
    });
    expect(edit(FILE, 'file', { quotaLimit: 20 }).softQuota).toEqual({
      type: 'spaceUsedQuota',
      limit: 20 * MB,
    });
    expect(edit(FILE, 'file', { quotaEnabled: false }).softQuota).toBeNull();
    expect(edit(GROUP, 'group', { quotaEnabled: true, quotaLimit: 1 }).softQuota).toEqual({
      type: 'spaceUsedQuota',
      limit: MB,
    });
  });

  it('changes only the edited S3 fields', () => {
    const saved = edit(S3, 's3', { prefix: 'other', forcePathStyle: false });
    expect(saved.bucketConfiguration).toEqual({
      ...(S3.bucketConfiguration as object),
      bucket: { region: 'eu-central-1', name: 'astrum-blobs', prefix: 'other', expiration: 3 },
      advancedBucketConnection: {
        endpoint: 'https://s3.example.org',
        signerType: 'AWSS3V4SignerType',
        forcePathStyle: false,
        maxConnectionPoolSize: 20,
      },
    });
  });

  it('sends S3 secrets only when typed and drops encryption when none is chosen', () => {
    expect(JSON.stringify(edit(S3, 's3', {}))).not.toContain('secretAccessKey');
    const keyed = edit(MINIMAL_S3, 's3', { accessKeyId: 'AKIA', secretAccessKey: 'secret' });
    expect((keyed.bucketConfiguration as Record<string, unknown>).bucketSecurity).toEqual({
      accessKeyId: 'AKIA',
      secretAccessKey: 'secret',
    });
    const plain = edit(S3, 's3', { encryptionType: '', encryptionKey: 'ignored' });
    expect((plain.bucketConfiguration as Record<string, unknown>).encryption).toBeNull();
  });

  it('keeps the order of group members', () => {
    expect(edit(GROUP, 'group', { members: ['two', 'one', 'three'] }).members).toEqual([
      'two',
      'one',
      'three',
    ]);
  });

  it('starts new stores from templates', () => {
    expect(template('file')).toEqual({ name: '', path: '', softQuota: null });
    expect(edit(template('group'), 'group', { name: 'g', members: ['a'] })).toEqual({
      name: 'g',
      softQuota: null,
      members: ['a'],
      fillPolicy: 'writeToFirst',
    });
    const s3 = edit(template('s3'), 's3', { name: 'c', bucket: 'my-bucket' });
    expect(s3).toMatchObject({
      name: 'c',
      type: 'S3',
      bucketConfiguration: { bucket: { name: 'my-bucket', region: 'DEFAULT' } },
    });
  });
});

describe('blob store checks', () => {
  function check(
    type: StoreType,
    change: Partial<StoreValues>,
    creating = true,
    storedKeyId: string | null = null,
  ) {
    const values = {
      ...fromDocument(template(type)),
      name: 'ok',
      path: 'ok',
      bucket: 'ok-bucket',
      ...change,
    };
    return validate(values, { type, creating, storedKeyId }, t);
  }

  it('checks names, paths and quotas', () => {
    expect(check('file', { name: '' }).name).toBe('Enter a name');
    expect(check('file', { name: '.x' }).name).toContain('cannot start');
    expect(check('file', { path: ' ' }).path).toBe('Enter a path');
    expect(check('file', { quotaEnabled: true, quotaLimit: 0 }).quotaLimit).toBe('Enter a limit above 0');
    expect(check('file', { quotaEnabled: true, quotaLimit: 5 })).toEqual({});
  });

  it('checks buckets and credentials', () => {
    expect(check('s3', { bucket: 'Bad_Bucket' }).bucket).toContain('lower-case');
    expect(check('s3', { region: '' }).region).toBe('Enter a region, or DEFAULT');
    expect(check('s3', { expiration: -2 }).expiration).toBeDefined();
    expect(check('s3', { accessKeyId: 'AKIA' }).secretAccessKey).toBe('Enter the secret access key');
    expect(check('s3', { accessKeyId: 'AKIA' }, false, 'AKIA')).toEqual({});
    expect(check('s3', { accessKeyId: 'OTHER' }, false, 'AKIA').secretAccessKey).toBeDefined();
    expect(check('s3', { secretAccessKey: 'x' }).accessKeyId).toBeDefined();
    expect(check('s3', { endpoint: 'minio:9000' }).endpoint).toBeDefined();
    expect(storedSecret(S3)).toBe('AKIAEXAMPLE');
    expect(storedSecret(MINIMAL_S3)).toBeNull();
  });

  it('needs members in a group', () => {
    expect(check('group', {}).members).toBe('Add at least one member');
    expect(check('group', { members: ['a'] })).toEqual({});
  });

  it('puts server problems on their fields', () => {
    expect(formField('members')).toBe('members');
    expect(formField('PARAMETER path')).toBe('path');
    expect(formField('bucketConfiguration.bucket.name')).toBe('bucket');
    expect(formField('bucketConfiguration.bucketSecurity.secretAccessKey')).toBe('secretAccessKey');
    expect(formField('*')).toBeNull();
    const mapped = onFormFields(
      new ApiError(400, 'x', {
        fieldErrors: [
          { id: 'members', message: 'one holds 3 blobs of the group; removing it would lose them' },
          { id: '*', message: 'overlaps default' },
        ],
      }),
    ) as ApiError;
    expect(mapped.fieldErrors).toEqual([
      { id: 'members', message: 'one holds 3 blobs of the group; removing it would lose them' },
      { id: '*', message: 'overlaps default' },
    ]);
  });
});
