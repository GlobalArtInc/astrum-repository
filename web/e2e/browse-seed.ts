import { createHash } from 'node:crypto';

export const VIEWER = { user: 'group-viewer', password: 'Viewer-pass-1' };

export const SEED = {
  raw: 'raw-files',
  rawMore: 'raw-more',
  rawGroup: 'raw-all',
  rawProxy: 'raw-remote',
  maven: 'maven-releases',
  npm: 'npm-internal',
  pypi: 'pypi-internal',
  docker: 'docker-internal',
} as const;

export const REPOSITORY_COUNT = Object.keys(SEED).length;

type Fetch = (path: string, init?: RequestInit) => Promise<Response>;

function client(url: string, password: string): Fetch {
  const authorization = `Basic ${Buffer.from(`admin:${password}`).toString('base64')}`;
  return (path, init = {}) =>
    fetch(`${url}${path}`, {
      ...init,
      headers: { authorization, ...(init.headers as Record<string, string> | undefined) },
    });
}

async function expectOk(response: Response, what: string): Promise<Response> {
  if (!response.ok) {
    throw new Error(`${what}: ${response.status} ${await response.text()}`);
  }
  return response;
}

function sha(algorithm: 'sha1' | 'sha256', data: Buffer | string): string {
  return createHash(algorithm).update(data).digest('hex');
}

const STORAGE = { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'ALLOW' };

async function create(call: Fetch, format: string, type: string, body: Record<string, unknown>) {
  await expectOk(
    await call(`/service/rest/v1/repositories/${format}/${type}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ online: true, ...body }),
    }),
    `create ${body.name}`,
  );
}

export async function put(
  call: Fetch,
  path: string,
  body: string | Buffer,
  type = 'application/octet-stream',
) {
  await expectOk(
    await call(path, {
      method: 'PUT',
      headers: { 'content-type': type },
      body: new Uint8Array(Buffer.from(body)),
    }),
    `put ${path}`,
  );
}

function pom(group: string, artifact: string, version: string): string {
  return `<?xml version="1.0" encoding="UTF-8"?>\n<project xmlns="http://maven.apache.org/POM/4.0.0">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>${group}</groupId>\n  <artifactId>${artifact}</artifactId>\n  <version>${version}</version>\n  <packaging>jar</packaging>\n</project>\n`;
}

export async function deployMaven(
  call: Fetch,
  repository: string,
  group: string,
  artifact: string,
  version: string,
) {
  const base = `/repository/${repository}/${group.replaceAll('.', '/')}/${artifact}/${version}/${artifact}-${version}`;
  await put(
    call,
    `${base}.jar`,
    `PK\u0003\u0004 ${artifact} ${version} jar content`,
    'application/java-archive',
  );
  await put(call, `${base}.pom`, pom(group, artifact, version), 'application/xml');
}

async function publishNpm(call: Fetch, repository: string, name: string, version: string) {
  const data = Buffer.concat([
    Buffer.from([0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3]),
    Buffer.from(`package ${name}`),
  ]);
  const bare = name.split('/').pop() ?? name;
  const document = {
    _id: name,
    name,
    description: `${bare} does things`,
    'dist-tags': { latest: version },
    versions: {
      [version]: {
        name,
        version,
        description: `${bare} does things`,
        dist: { tarball: `http://client.invalid/${name}/-/${bare}-${version}.tgz` },
      },
    },
    _attachments: {
      [`${name}-${version}.tgz`]: {
        content_type: 'application/octet-stream',
        data: data.toString('base64'),
        length: data.length,
      },
    },
  };
  await expectOk(
    await call(`/repository/${repository}/${name.replace('/', '%2f')}`, {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(document),
    }),
    `npm publish ${name}`,
  );
}

async function uploadPypi(call: Fetch, repository: string, name: string, version: string) {
  const data = Buffer.concat([
    Buffer.from([0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3]),
    Buffer.from(`sdist ${name}`),
  ]);
  const form = new FormData();
  const fields: Array<[string, string]> = [
    [':action', 'file_upload'],
    ['protocol_version', '1'],
    ['metadata_version', '2.1'],
    ['name', name],
    ['version', version],
    ['summary', `${name} for the browse tests`],
    ['sha256_digest', sha('sha256', data)],
  ];
  for (const [key, value] of fields) {
    form.append(key, value);
  }
  form.append('content', new Blob([new Uint8Array(data)]), `${name}-${version}.tar.gz`);
  await expectOk(
    await call(`/repository/${repository}/`, { method: 'POST', body: form }),
    `pypi upload ${name}`,
  );
}

async function pushBlob(call: Fetch, prefix: string, data: Buffer): Promise<string> {
  const digest = `sha256:${sha('sha256', data)}`;
  await expectOk(
    await call(`${prefix}/blobs/uploads/?digest=${digest}`, {
      method: 'POST',
      headers: { 'content-type': 'application/octet-stream' },
      body: new Uint8Array(data),
    }),
    `docker blob ${digest}`,
  );
  return digest;
}

async function pushImage(call: Fetch, repository: string, image: string, tag: string) {
  const prefix = `/v2/${repository}/${image}`;
  const config = Buffer.from('{"os":"linux","architecture":"amd64"}');
  const layer = Buffer.from(`layer of ${image}:${tag}`);
  const configDigest = await pushBlob(call, prefix, config);
  const layerDigest = await pushBlob(call, prefix, layer);
  const manifest = {
    schemaVersion: 2,
    mediaType: 'application/vnd.oci.image.manifest.v1+json',
    config: {
      mediaType: 'application/vnd.oci.image.config.v1+json',
      digest: configDigest,
      size: config.length,
    },
    layers: [
      { mediaType: 'application/vnd.oci.image.layer.v1.tar+gzip', digest: layerDigest, size: layer.length },
    ],
  };
  await expectOk(
    await call(`${prefix}/manifests/${tag}`, {
      method: 'PUT',
      headers: { 'content-type': manifest.mediaType },
      body: JSON.stringify(manifest),
    }),
    `docker manifest ${image}:${tag}`,
  );
}

async function viewer(call: Fetch) {
  const json = { 'content-type': 'application/json' };
  await expectOk(
    await call('/service/rest/v1/security/roles', {
      method: 'POST',
      headers: json,
      body: JSON.stringify({
        id: 'group-viewer-role',
        name: 'group-viewer-role',
        description: 'Sees only the raw group',
        privileges: [
          `as-repository-view-raw-${SEED.rawGroup}-browse`,
          `as-repository-view-raw-${SEED.rawGroup}-read`,
        ],
        roles: [],
      }),
    }),
    'viewer role',
  );
  await expectOk(
    await call('/service/rest/v1/security/users', {
      method: 'POST',
      headers: json,
      body: JSON.stringify({
        userId: VIEWER.user,
        firstName: 'Group',
        lastName: 'Viewer',
        emailAddress: 'viewer@example.org',
        password: VIEWER.password,
        status: 'active',
        roles: ['group-viewer-role'],
      }),
    }),
    'viewer user',
  );
}

const seeding = new Map<string, Promise<void>>();

async function seed(url: string, password: string) {
  const call = client(url, password);
  const existing = await call(`/service/rest/v1/repositories/${SEED.raw}`);
  if (existing.ok) {
    return;
  }
  await create(call, 'raw', 'hosted', {
    name: SEED.raw,
    storage: STORAGE,
    raw: { contentDisposition: 'ATTACHMENT' },
  });
  await create(call, 'raw', 'hosted', {
    name: SEED.rawMore,
    storage: STORAGE,
    raw: { contentDisposition: 'ATTACHMENT' },
  });
  await create(call, 'raw', 'group', {
    name: SEED.rawGroup,
    storage: { blobStoreName: 'default', strictContentTypeValidation: false },
    group: { memberNames: [SEED.raw, SEED.rawMore] },
  });
  await create(call, 'raw', 'proxy', {
    name: SEED.rawProxy,
    storage: { blobStoreName: 'default', strictContentTypeValidation: false },
    proxy: { remoteUrl: 'http://upstream.invalid/', contentMaxAge: 1440, metadataMaxAge: 1440 },
    negativeCache: { enabled: false, timeToLive: 1 },
    httpClient: { blocked: true, autoBlock: false },
    raw: { contentDisposition: 'ATTACHMENT' },
  });
  await create(call, 'maven', 'hosted', {
    name: SEED.maven,
    storage: STORAGE,
    maven: { versionPolicy: 'MIXED', layoutPolicy: 'STRICT', contentDisposition: 'ATTACHMENT' },
  });
  await create(call, 'npm', 'hosted', { name: SEED.npm, storage: STORAGE });
  await create(call, 'pypi', 'hosted', { name: SEED.pypi, storage: STORAGE });
  await create(call, 'docker', 'hosted', {
    name: SEED.docker,
    storage: STORAGE,
    docker: { v1Enabled: false, forceBasicAuth: true, pathEnabled: true },
  });

  const files: Array<[string, string]> = [
    ['/docs/readme.txt', 'Read me first.\n'],
    ['/docs/guide/intro.md', '# Intro\n'],
    ['/docs/guide/setup.md', '# Setup\n'],
    ['/tools/install.sh', '#!/bin/sh\necho install\n'],
    ['/hello.txt', 'hello\n'],
  ];
  for (const [path, body] of files) {
    await put(call, `/repository/${SEED.raw}${path}`, body, 'text/plain');
  }
  await put(
    call,
    `/repository/${SEED.rawMore}/shared/notes.txt`,
    'notes from the second member\n',
    'text/plain',
  );
  await put(
    call,
    `/repository/${SEED.rawMore}/docs/readme.txt`,
    'hidden behind the first member\n',
    'text/plain',
  );
  for (const version of ['1.0.0', '1.1.0', '2.0.0']) {
    await deployMaven(call, SEED.maven, 'org.acme', 'demo-lib', version);
  }
  await deployMaven(call, SEED.maven, 'com.example', 'tools', '0.9.0');
  await publishNpm(call, SEED.npm, '@team/tool', '1.0.0');
  await publishNpm(call, SEED.npm, 'left-pad', '1.3.0');
  await uploadPypi(call, SEED.pypi, 'tool', '0.3.0');
  await pushImage(call, SEED.docker, 'team/app', '1.0');
  await viewer(call);
  for (let index = 0; index < 3; index += 1) {
    await expectOk(await call(`/repository/${SEED.raw}/docs/readme.txt`), 'download');
  }
  await expectOk(
    await call(`/repository/${SEED.maven}/org/acme/demo-lib/2.0.0/demo-lib-2.0.0.jar`),
    'download',
  );
}

export function seedBrowse(url: string, password: string): Promise<void> {
  let running = seeding.get(url);
  if (!running) {
    running = seed(url, password);
    seeding.set(url, running);
  }
  return running;
}

export function adminFetch(url: string, password: string): Fetch {
  return client(url, password);
}
