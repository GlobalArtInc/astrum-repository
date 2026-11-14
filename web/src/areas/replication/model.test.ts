import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../i18n';
import type { ComponentItem, ReplicationRemote } from './api';
import {
  bytesFromMiB,
  filterSubject,
  miBFromBytes,
  type Preview,
  previewFilters,
  RECONCILE_MINUTES,
  type RemoteValues,
  type ReplicationValues,
  remoteBody,
  remoteValues,
  replicationBody,
  replicationValues,
  validateRemote,
  validateReplication,
} from './model';

const t = makeTranslate('en');

const REMOTE: ReplicationRemote = {
  id: 'r1',
  name: 'Mirror',
  url: 'https://mirror.example.com',
  hasToken: true,
  useTrustStore: false,
  allowPrivateNetwork: false,
  maxConcurrency: 2,
  maxBytesPerSecond: 1572864,
  remoteInstanceId: null,
  remoteVersion: null,
  checkedAt: null,
  createdAt: '2026-11-01T10:00:00Z',
  updatedAt: '2026-11-01T10:00:00Z',
};

function remoteInput(overrides: Partial<RemoteValues> = {}): RemoteValues {
  return { ...remoteValues(REMOTE), ...overrides };
}

function replicationInput(overrides: Partial<ReplicationValues> = {}): ReplicationValues {
  return {
    name: 'Nightly',
    sourceRepository: 'releases',
    remoteId: 'r1',
    remoteRepository: 'releases-eu',
    enabled: true,
    nameFilter: '',
    versionFilter: '',
    propagateDeletes: false,
    settleSeconds: 30,
    reconcileMinutes: 360,
    alertChannels: [],
    ...overrides,
  };
}

describe('remote validation', () => {
  it('accepts https without a path and refuses http unless private addresses are allowed', () => {
    expect(validateRemote(t, remoteInput(), REMOTE)).toEqual({});
    expect(validateRemote(t, remoteInput({ url: 'http://mirror.example.com' }), REMOTE).url).toContain(
      'https',
    );
    expect(
      validateRemote(t, remoteInput({ url: 'http://10.0.0.5:8081', allowPrivateNetwork: true }), REMOTE).url,
    ).toBeUndefined();
    expect(validateRemote(t, remoteInput({ url: 'https://mirror.example.com/raw' }), REMOTE).url).toContain(
      'no path',
    );
  });

  it('keeps concurrency between 1 and 16 and bandwidth above zero or empty', () => {
    expect(validateRemote(t, remoteInput({ maxConcurrency: 0 }), REMOTE).maxConcurrency).toBe(
      'Enter a whole number from 1 to 16',
    );
    expect(validateRemote(t, remoteInput({ maxConcurrency: 17 }), REMOTE).maxConcurrency).toBeDefined();
    expect(validateRemote(t, remoteInput({ maxConcurrency: 16 }), REMOTE).maxConcurrency).toBeUndefined();
    expect(validateRemote(t, remoteInput({ bandwidthMiB: 0 }), REMOTE).bandwidthMiB).toBeDefined();
    expect(validateRemote(t, remoteInput({ bandwidthMiB: null }), REMOTE).bandwidthMiB).toBeUndefined();
  });

  it('converts MiB per second to bytes per second', () => {
    expect(bytesFromMiB(1.5)).toBe(1572864);
    expect(miBFromBytes(1572864)).toBe(1.5);
    expect(remoteBody(remoteInput({ bandwidthMiB: 1.5 })).maxBytesPerSecond).toBe(1572864);
    expect(remoteBody(remoteInput({ bandwidthMiB: null })).maxBytesPerSecond).toBeNull();
  });

  it('omits the token from the body when it is empty and requires it when the URL changed', () => {
    expect(remoteBody(remoteInput()), 'token omitted').not.toHaveProperty('token');
    expect(remoteBody(remoteInput({ token: 'secret-1' })).token).toBe('secret-1');
    const moved = remoteInput({ url: 'https://other.example.com' });
    expect(validateRemote(t, moved, REMOTE).token).toBe(
      'Enter the token: the URL is new or the token is not stored',
    );
    expect(validateRemote(t, remoteInput({ token: 'secret-1' }), null).token).toBeUndefined();
    expect(validateRemote(t, remoteInput(), null).token).toBeDefined();
  });
});

describe('replication validation', () => {
  it('checks the settle range', () => {
    expect(validateReplication(t, replicationInput({ settleSeconds: 3600 })).settleSeconds).toBeUndefined();
    expect(validateReplication(t, replicationInput({ settleSeconds: 3601 })).settleSeconds).toBeDefined();
    expect(validateReplication(t, replicationInput({ settleSeconds: -1 })).settleSeconds).toBeDefined();
  });

  it('accepts only the reconcile periods the server takes', () => {
    expect(RECONCILE_MINUTES).toHaveLength(13);
    for (const minutes of RECONCILE_MINUTES) {
      expect(
        validateReplication(t, replicationInput({ reconcileMinutes: minutes })).reconcileMinutes,
      ).toBeUndefined();
    }
    for (const minutes of [45, 0, 10080]) {
      expect(validateReplication(t, replicationInput({ reconcileMinutes: minutes })).reconcileMinutes).toBe(
        'Choose one of the listed periods',
      );
    }
  });

  it('sends blank filters as null', () => {
    const body = replicationBody(replicationValues(null));
    expect(body.nameFilter).toBeNull();
    expect(body.versionFilter).toBeNull();
    expect(replicationBody(replicationInput({ nameFilter: 'lib-.*' })).nameFilter).toBe('lib-.*');
  });
});

function sentFlags(preview: Preview): boolean[] {
  return preview.kind === 'rows' ? preview.rows.map((row) => row.sent) : [];
}

describe('filter subject', () => {
  it.each([
    ['empty namespace', null, 'lib', 'lib'],
    ['empty string namespace', '', 'lib', 'lib'],
    ['root namespace', '/', 'top.txt', 'top.txt'],
    ['raw namespace already in the name', '/docs', '/docs/a.txt', '/docs/a.txt'],
    ['raw root file', '/', '/top.txt', '/top.txt'],
    ['maven group', 'com.acme', 'lib', 'com.acme/lib'],
    ['npm scope', 'scope', 'pkg', 'scope/pkg'],
    ['name that only starts with the namespace text', 'ab', 'abc', 'ab/abc'],
  ])('%s', (_, namespace, name, subject) => {
    expect(filterSubject(namespace, name)).toBe(subject);
  });
});

describe('filter preview', () => {
  const COMPONENTS: ComponentItem[] = [
    { repository: 'releases', format: 'raw', group: null, name: 'lib-core', version: '1.0' },
    { repository: 'releases', format: 'raw', group: 'org', name: 'lib-util', version: '2.0' },
    { repository: 'releases', format: 'raw', group: null, name: 'app', version: '1.0' },
  ];

  it('matches the whole name and the whole version, anchored', () => {
    expect(sentFlags(previewFilters(COMPONENTS, 'lib-.*', '1\\..*'))).toEqual([true, false, false]);
    expect(sentFlags(previewFilters(COMPONENTS, 'core', ''))).toEqual([false, false, false]);
  });

  it('matches the name with its namespace when the component has one', () => {
    expect(sentFlags(previewFilters(COMPONENTS, '^org/.*', ''))).toEqual([false, true, false]);
    expect(sentFlags(previewFilters(COMPONENTS, 'org/lib-util', ''))).toEqual([false, true, false]);
  });

  it('matches the full path of a raw component once, without repeating its namespace', () => {
    const RAW: ComponentItem[] = [
      { repository: 'docs', format: 'raw', group: '/docs', name: '/docs/a.txt', version: '1' },
    ];
    expect(sentFlags(previewFilters(RAW, '^/docs/.*', ''))).toEqual([true]);
    expect(sentFlags(previewFilters(RAW, '^/docs//docs/.*', ''))).toEqual([false]);
  });

  it('reports a pattern that does not compile, and does not let it break out of its group', () => {
    expect(previewFilters(COMPONENTS, '(lib', '')).toEqual({ kind: 'invalid', field: 'nameFilter' });
    expect(previewFilters(COMPONENTS, 'a)|(b', '')).toEqual({ kind: 'invalid', field: 'nameFilter' });
    expect(previewFilters(COMPONENTS, '', '[0-')).toEqual({ kind: 'invalid', field: 'versionFilter' });
  });
});
