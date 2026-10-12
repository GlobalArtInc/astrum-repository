import { describe, expect, it } from 'vitest';
import { ApiError } from '../../../api';
import { makeTranslate } from '../../../i18n';
import { canonical, clone, getPath, type RepositoryDocument } from './document';
import {
  changedPaths,
  documentMembers,
  formField,
  fromDocument,
  onFormFields,
  type RepositoryValues,
  storedSecrets,
  template,
  toDocument,
  validate,
} from './form';
import reference from './reference-documents.json';
import { apiFormat, formatFromUrl, type Spec, sections } from './spec';

const t = makeTranslate('en');
const DOCUMENTS = Object.entries(reference as unknown as Record<string, RepositoryDocument>);
const SERVED = ['raw', 'maven2', 'npm', 'nuget', 'pypi', 'docker'];

function specOf(document: RepositoryDocument): Spec {
  return { format: String(document.format), type: document.type as Spec['type'] };
}

function edit(document: RepositoryDocument, change: Partial<RepositoryValues>): RepositoryDocument {
  return toDocument({ ...fromDocument(document), ...change }, document);
}

function differences(left: RepositoryDocument, right: RepositoryDocument): string[] {
  const found: string[] = [];
  const walk = (a: unknown, b: unknown, path: string) => {
    if (typeof a === 'object' && a !== null && !Array.isArray(a) && typeof b === 'object' && b !== null) {
      const keys = new Set([...Object.keys(a), ...Object.keys(b as object)]);
      for (const key of keys) {
        walk(
          (a as Record<string, unknown>)[key],
          (b as Record<string, unknown>)[key],
          path ? `${path}.${key}` : key,
        );
      }
    } else if (JSON.stringify(a) !== JSON.stringify(b)) {
      found.push(path);
    }
  };
  walk(left, right, '');
  return found;
}

describe('reference documents', () => {
  it('cover every format and type the server registry offers, twice', () => {
    expect(DOCUMENTS).toHaveLength(120);
    const kinds = new Set(DOCUMENTS.map(([, document]) => `${document.format}/${document.type}`));
    expect(kinds.size).toBe(60);
  });

  it.each(DOCUMENTS)('%s is saved unchanged when nothing is edited', (_, document) => {
    const values = fromDocument(document);
    const saved = toDocument(values, document);
    expect(canonical(saved)).toEqual(canonical(document));
    expect(changedPaths(values, document)).toEqual([]);
  });

  it.each(DOCUMENTS)('%s passes the checks of the form unchanged', (_, document) => {
    const spec = specOf(document);
    expect(
      validate(fromDocument(document), { spec, creating: false, stored: storedSecrets(document) }, t),
    ).toEqual({});
  });
});

describe('editing a document', () => {
  const rawProxy = (reference as unknown as Record<string, RepositoryDocument>)[
    'raw/proxy/max'
  ] as RepositoryDocument;
  const mavenProxy = (reference as unknown as Record<string, RepositoryDocument>)[
    'maven/proxy/max'
  ] as RepositoryDocument;
  const dockerHosted = (reference as unknown as Record<string, RepositoryDocument>)[
    'docker/hosted/max'
  ] as RepositoryDocument;
  const dockerGroup = (reference as unknown as Record<string, RepositoryDocument>)[
    'docker/group/max'
  ] as RepositoryDocument;

  it('changes only the path of the edited field', () => {
    const saved = edit(rawProxy, { remoteUrl: ' https://example.org/files/ ' });
    expect(differences(rawProxy, saved)).toEqual(['proxy.remoteUrl']);
    expect(getPath(saved, ['proxy', 'remoteUrl'])).toBe('https://example.org/files/');
  });

  it('keeps fields it does not show, such as cleanup, replication and query parameters', () => {
    const document = clone(rawProxy);
    document.raw = {
      contentDisposition: 'INLINE',
      forwardQueryParameters: true,
      excludedQueryParameters: ['apiKey'],
    };
    document.dockerProxy = { ecrRegion: 'eu-central-1' };
    const saved = edit(document, { timeToLive: 5 });
    expect(saved.raw).toEqual(document.raw);
    expect(saved.cleanup).toEqual(document.cleanup);
    expect(saved.replication).toEqual(document.replication);
    expect(saved.dockerProxy).toEqual({ ecrRegion: 'eu-central-1' });
    expect(differences(document, saved)).toEqual(['negativeCache.timeToLive']);
  });

  it('writes the latest policy of docker hosted repositories', () => {
    const saved = edit(dockerHosted, { latestPolicy: false });
    expect(getPath(saved, ['storage', 'latestPolicy'])).toBe(false);
    expect(differences(dockerHosted, saved)).toEqual(['storage.latestPolicy']);
  });

  it('sends a new secret only when one is typed', () => {
    expect(getPath(edit(mavenProxy, {}), ['httpClient', 'authentication', 'password'])).toBeUndefined();
    const saved = edit(mavenProxy, { password: 'n3w s3cret' });
    expect(getPath(saved, ['httpClient', 'authentication', 'password'])).toBe('n3w s3cret');
  });

  it('drops the authentication when its type becomes none', () => {
    const saved = edit(mavenProxy, { authType: '', username: 'ignored' });
    expect(getPath(saved, ['httpClient', 'authentication'])).toBeNull();
  });

  it('creates the authentication when there was none', () => {
    const document = clone(rawProxy);
    (document.httpClient as Record<string, unknown>).authentication = null;
    const saved = edit(document, { authType: 'bearerToken', bearerToken: 'token-1' });
    expect(getPath(saved, ['httpClient', 'authentication'])).toEqual({
      type: 'bearerToken',
      bearerToken: 'token-1',
    });
  });

  it('reorders members and clears the writable member', () => {
    const saved = edit(dockerGroup, { memberNames: ['b', 'a'], writableMember: '' });
    expect(getPath(saved, ['group', 'memberNames'])).toEqual(['b', 'a']);
    expect(getPath(saved, ['group', 'writableMember'])).toBeNull();
    expect(documentMembers(saved)).toEqual(['b', 'a']);
  });

  it('turns text lists into arrays', () => {
    const document = (reference as unknown as Record<string, RepositoryDocument>)[
      'docker/proxy/max'
    ] as RepositoryDocument;
    const saved = edit(document, { foreignLayerUrlWhitelist: ' .*a\n\n.*b \n' });
    expect(getPath(saved, ['dockerProxy', 'foreignLayerUrlWhitelist'])).toEqual(['.*a', '.*b']);
  });

  it('knows which secrets are stored', () => {
    expect(storedSecrets(mavenProxy)).toEqual({ password: true, bearerToken: false });
    expect(storedSecrets(dockerHosted)).toEqual({ password: false, bearerToken: false });
  });
});

describe('new repositories', () => {
  const specs: Spec[] = SERVED.flatMap((format) =>
    (['hosted', 'proxy', 'group'] as const).map((type) => ({ format, type })),
  );

  it.each(specs)('a $format $type template holds the sections the server expects', (spec) => {
    const document = template(spec, 'default');
    expect(getPath(document, ['storage', 'blobStoreName'])).toBe('default');
    expect('proxy' in document).toBe(spec.type === 'proxy');
    expect('httpClient' in document).toBe(spec.type === 'proxy');
    expect('group' in document).toBe(spec.type === 'group');
    expect('maven' in document).toBe(sections.maven(spec));
    expect('docker' in document).toBe(spec.format === 'docker');
    expect('dockerProxy' in document).toBe(spec.format === 'docker' && spec.type === 'proxy');
    expect(getPath(document, ['nugetProxy'])).toEqual(
      sections.nugetProxy(spec) ? { nugetVersion: 'V3', queryCacheItemMaxAge: 3600 } : undefined,
    );
    expect(getPath(document, ['storage', 'writePolicy'])).toBe(
      spec.type === 'hosted' ? 'ALLOW_ONCE' : undefined,
    );
  });

  it.each(specs)('a filled $format $type form becomes a complete request', (spec) => {
    const document = template(spec, 'default');
    const values: RepositoryValues = {
      ...fromDocument(document),
      name: `new-${spec.format}`,
      remoteUrl: spec.type === 'proxy' ? 'https://remote.example.org/' : '',
      memberNames: spec.type === 'group' ? ['member'] : [],
    };
    expect(
      validate(values, { spec, creating: true, stored: { password: false, bearerToken: false } }, t),
    ).toEqual({});
    const request = toDocument(values, document);
    expect(request.name).toBe(`new-${spec.format}`);
    expect(request.online).toBe(true);
    if (spec.type === 'proxy') {
      expect(getPath(request, ['proxy', 'remoteUrl'])).toBe('https://remote.example.org/');
      expect(getPath(request, ['httpClient', 'authentication'])).toBeNull();
    }
    if (spec.type === 'group') {
      expect(getPath(request, ['group', 'memberNames'])).toEqual(['member']);
    }
    if (spec.format === 'docker') {
      expect(getPath(request, ['docker', 'pathEnabled'])).toBe(true);
    }
  });
});

describe('form checks', () => {
  const hosted: Spec = { format: 'raw', type: 'hosted' };
  const proxy: Spec = { format: 'maven2', type: 'proxy' };
  const none = { password: false, bearerToken: false };

  function check(spec: Spec, change: Partial<RepositoryValues>, stored = none, creating = true) {
    const values = {
      ...fromDocument(template(spec, 'default')),
      name: 'ok',
      remoteUrl: 'https://x.example/',
      ...change,
    };
    return validate(values, { spec, creating, stored }, t);
  }

  it('checks names when creating', () => {
    expect(check(hosted, { name: '' }).name).toBe('Enter a name');
    expect(check(hosted, { name: '_x' }).name).toContain('cannot start');
    expect(check(hosted, { name: 'a b' }).name).toContain('Use letters');
    expect(check(hosted, { name: 'x'.repeat(201) }).name).toBe('Use at most 200 characters');
    expect(check(hosted, { name: '-ok.name_1' })).toEqual({});
    expect(check(hosted, { name: '' }, none, false).name).toBeUndefined();
  });

  it('checks the remote and the numbers of proxies', () => {
    expect(check(proxy, { remoteUrl: '' }).remoteUrl).toBe('Enter the address of the remote repository');
    expect(check(proxy, { remoteUrl: 'ftp://x.example' }).remoteUrl).toContain('http://');
    expect(check(proxy, { contentMaxAge: -2 }).contentMaxAge).toBeDefined();
    expect(check(proxy, { metadataMaxAge: null }).metadataMaxAge).toBeDefined();
    expect(check(proxy, { timeToLive: -1 }).timeToLive).toBeDefined();
    expect(check(proxy, { retries: 11 }).retries).toBeDefined();
    expect(check(proxy, { timeout: 0 }).timeout).toBeDefined();
    expect(check(proxy, { retries: null, timeout: null })).toEqual({});
  });

  it('checks the query cache age of NuGet proxies', () => {
    const nuget: Spec = { format: 'nuget', type: 'proxy' };
    expect(check(nuget, {})).toEqual({});
    expect(check(nuget, { queryCacheItemMaxAge: -1 }).queryCacheItemMaxAge).toBe(
      'Enter a whole number of seconds, 0 or more',
    );
    expect(check(nuget, { queryCacheItemMaxAge: null }).queryCacheItemMaxAge).toBeDefined();
    const saved = toDocument(
      { ...fromDocument(template(nuget, 'default')), nugetVersion: 'V2', queryCacheItemMaxAge: 60 },
      template(nuget, 'default'),
    );
    expect(getPath(saved, ['nugetProxy'])).toEqual({ nugetVersion: 'V2', queryCacheItemMaxAge: 60 });
  });

  it('asks for credentials unless they are stored', () => {
    expect(check(proxy, { authType: 'username' })).toEqual({
      username: 'Enter the user name',
      password: 'Enter the password',
    });
    expect(check(proxy, { authType: 'ntlm', username: 'u' }, { password: true, bearerToken: false })).toEqual(
      {},
    );
    expect(check(proxy, { authType: 'bearerToken' }).bearerToken).toBe('Enter the token');
    expect(check(proxy, { authType: 'bearerToken' }, { password: false, bearerToken: true })).toEqual({});
  });

  it('needs members in groups and sane docker ports', () => {
    expect(check({ format: 'raw', type: 'group' }, {}).memberNames).toBe('Add at least one member');
    const docker: Spec = { format: 'docker', type: 'hosted' };
    expect(check(docker, { httpPort: 70000 }).httpPort).toBeDefined();
    expect(check(docker, { httpPort: 5000, httpsPort: 5000 }).httpsPort).toBe(
      'Use another port than the HTTP port',
    );
    expect(check(docker, { httpPort: 5000, httpsPort: 5001 })).toEqual({});
  });

  it('needs an index address for a custom docker index', () => {
    const docker: Spec = { format: 'docker', type: 'proxy' };
    expect(check(docker, { indexType: 'CUSTOM', indexUrl: '' }).indexUrl).toBe(
      'Enter the address of the index',
    );
    expect(check(docker, { indexType: 'HUB', indexUrl: 'nope' }).indexUrl).toBeDefined();
    expect(check(docker, { indexType: 'REGISTRY', indexUrl: 'nope' })).toEqual({});
  });
});

describe('server problems', () => {
  it('land on the fields they name', () => {
    const raw: Spec = { format: 'raw', type: 'proxy' };
    expect(formField('PARAMETER remoteUrl', raw)).toBe('remoteUrl');
    expect(formField('PARAMETER enabled', raw)).toBe('negativeCacheEnabled');
    expect(formField('PARAMETER type', raw)).toBe('authType');
    expect(formField('PARAMETER contentDisposition', raw)).toBe('rawContentDisposition');
    expect(formField('PARAMETER contentDisposition', { format: 'maven2', type: 'hosted' })).toBe(
      'mavenContentDisposition',
    );
    expect(formField('PARAMETER policyNames', raw)).toBeNull();
    const error = onFormFields(
      new ApiError(400, 'x', {
        fieldErrors: [
          { id: 'PARAMETER timeToLive', message: 'must be greater than or equal to 0' },
          { id: 'PARAMETER storage', message: 'must not be null' },
          { id: '*', message: 'broken' },
        ],
      }),
      raw,
    ) as ApiError;
    expect(error.fieldErrors).toEqual([
      { id: 'timeToLive', message: 'must be greater than or equal to 0' },
      { id: '*', message: 'storage: must not be null' },
      { id: '*', message: 'broken' },
    ]);
    const other = new ApiError(403, 'no');
    expect(onFormFields(other, raw)).toBe(other);
  });
});

describe('format names in addresses', () => {
  it('maps between format ids and API names', () => {
    expect(apiFormat('maven2')).toBe('maven');
    expect(apiFormat('raw', [{ id: 'raw', apiName: 'raw', types: ['hosted'] }])).toBe('raw');
    expect(formatFromUrl('maven')).toBe('maven2');
    expect(formatFromUrl('maven2')).toBe('maven2');
    expect(formatFromUrl('npm')).toBe('npm');
  });
});
