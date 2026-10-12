import { crc32, gzipSync } from 'node:zlib';
import { expect } from '@playwright/test';
import { ADMIN, type Server } from './fixtures';

type Entry = [string, Buffer | string];

function bytes(data: Buffer | string): Buffer {
  return typeof data === 'string' ? Buffer.from(data, 'utf8') : data;
}

function octal(value: number, length: number): string {
  return `${value.toString(8).padStart(length - 1, '0')}\0`;
}

export function tar(entries: Entry[]): Buffer {
  const blocks: Buffer[] = [];
  for (const [name, data] of entries) {
    const body = bytes(data);
    const header = Buffer.alloc(512);
    header.write(name, 0, 100, 'utf8');
    header.write(octal(0o644, 8), 100, 'ascii');
    header.write(octal(0, 8), 108, 'ascii');
    header.write(octal(0, 8), 116, 'ascii');
    header.write(octal(body.length, 12), 124, 'ascii');
    header.write(octal(Math.floor(Date.now() / 1000), 12), 136, 'ascii');
    header.write('        ', 148, 'ascii');
    header.write('0', 156, 'ascii');
    header.write('ustar\0', 257, 'ascii');
    header.write('00', 263, 'ascii');
    let sum = 0;
    for (const byte of header) {
      sum += byte;
    }
    header.write(`${sum.toString(8).padStart(6, '0')}\0 `, 148, 'ascii');
    blocks.push(header, body, Buffer.alloc((512 - (body.length % 512)) % 512));
  }
  blocks.push(Buffer.alloc(1024));
  return Buffer.concat(blocks);
}

export function zip(entries: Entry[]): Buffer {
  const locals: Buffer[] = [];
  const centrals: Buffer[] = [];
  let offset = 0;
  for (const [name, data] of entries) {
    const body = bytes(data);
    const fileName = Buffer.from(name, 'utf8');
    const checksum = crc32(body);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt16LE(0, 6);
    local.writeUInt16LE(0, 8);
    local.writeUInt32LE(0, 10);
    local.writeUInt32LE(checksum, 14);
    local.writeUInt32LE(body.length, 18);
    local.writeUInt32LE(body.length, 22);
    local.writeUInt16LE(fileName.length, 26);
    local.writeUInt16LE(0, 28);
    const central = Buffer.alloc(46);
    central.writeUInt32LE(0x02014b50, 0);
    central.writeUInt16LE(20, 4);
    central.writeUInt16LE(20, 6);
    central.writeUInt16LE(0, 8);
    central.writeUInt16LE(0, 10);
    central.writeUInt32LE(0, 12);
    central.writeUInt32LE(checksum, 16);
    central.writeUInt32LE(body.length, 20);
    central.writeUInt32LE(body.length, 24);
    central.writeUInt16LE(fileName.length, 28);
    central.writeUInt32LE(offset, 42);
    locals.push(local, fileName, body);
    centrals.push(central, fileName);
    offset += local.length + fileName.length + body.length;
  }
  const directory = Buffer.concat(centrals);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, directory, end]);
}

export function npmTarball(name: string, version: string, description = `The ${name} package`): Buffer {
  const manifest = JSON.stringify({ name, version, description, main: 'index.js', license: 'MIT' });
  return gzipSync(
    tar([
      ['package/package.json', manifest],
      ['package/index.js', `module.exports = ${JSON.stringify(`${name}@${version}`)};\n`],
    ]),
  );
}

export function wheelName(name: string, version: string): string {
  return `${name.replace(/[-.]+/g, '_')}-${version}-py3-none-any.whl`;
}

export function wheel(name: string, version: string, summary: string, keywords = ''): Buffer {
  const module = name.replace(/[-.]+/g, '_');
  const info = `${module}-${version}.dist-info`;
  const metadata = [
    'Metadata-Version: 2.1',
    `Name: ${name}`,
    `Version: ${version}`,
    `Summary: ${summary}`,
    ...(keywords ? [`Keywords: ${keywords}`] : []),
    'Classifier: Programming Language :: Python :: 3',
    '',
    '',
  ].join('\n');
  return zip([
    [`${module}/__init__.py`, `__version__ = "${version}"\n`],
    [`${info}/METADATA`, metadata],
    [
      `${info}/WHEEL`,
      'Wheel-Version: 1.0\nGenerator: astrum-e2e\nRoot-Is-Purelib: true\nTag: py3-none-any\n',
    ],
    [`${info}/RECORD`, ''],
  ]);
}

export function jar(name: string): Buffer {
  return zip([
    ['META-INF/MANIFEST.MF', `Manifest-Version: 1.0\nImplementation-Title: ${name}\n`],
    [`${name.replace(/-/g, '/')}/Main.class`, 'not really a class'],
  ]);
}

export function pom(group: string, artifact: string, version: string, packaging = 'jar'): string {
  return `<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
  <modelVersion>4.0.0</modelVersion>
  <groupId>${group}</groupId>
  <artifactId>${artifact}</artifactId>
  <version>${version}</version>
  <packaging>${packaging}</packaging>
</project>
`;
}

export function authorization(server: Server): string {
  return `Basic ${Buffer.from(`${ADMIN}:${server.adminPassword}`).toString('base64')}`;
}

export async function call(
  server: Server,
  method: string,
  path: string,
  body?: unknown,
  headers: Record<string, string> = {},
): Promise<Response> {
  const json = body !== undefined && !(body instanceof Buffer) && !(body instanceof FormData);
  const response = await fetch(`${server.url}${path}`, {
    method,
    headers: {
      Authorization: authorization(server),
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...headers,
    },
    body: json ? JSON.stringify(body) : (body as BodyInit | undefined),
  });
  return response;
}

export async function ensure(response: Response, what: string): Promise<void> {
  expect(response.ok, `${what}: ${response.status} ${await response.clone().text()}`).toBe(true);
}

const STORAGE = { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'ALLOW' };

export async function hosted(
  server: Server,
  format: string,
  name: string,
  extra: object = {},
): Promise<void> {
  const existing = await call(server, 'GET', `/service/rest/v1/repositories/${name}`);
  if (existing.ok) {
    return;
  }
  const response = await call(server, 'POST', `/service/rest/v1/repositories/${format}/hosted`, {
    name,
    online: true,
    storage: STORAGE,
    ...extra,
  });
  await ensure(response, `create ${name}`);
}

export async function put(server: Server, path: string, data: Buffer | string): Promise<void> {
  await ensure(await call(server, 'PUT', path, bytes(data)), `PUT ${path}`);
}

export async function npmPublish(server: Server, repository: string, name: string, version: string) {
  const data = npmTarball(name, version);
  const bare = name.split('/').pop() ?? name;
  const document = {
    _id: name,
    name,
    'dist-tags': { latest: version },
    versions: { [version]: { name, version, description: `The ${name} package`, dist: {} } },
    _attachments: {
      [`${bare}-${version}.tgz`]: {
        content_type: 'application/octet-stream',
        data: data.toString('base64'),
        length: data.length,
      },
    },
  };
  await ensure(
    await call(server, 'PUT', `/repository/${repository}/${name.replace('/', '%2f')}`, document),
    `publish ${name}@${version}`,
  );
}

export async function componentUpload(server: Server, repository: string, form: FormData): Promise<void> {
  await ensure(
    await call(server, 'POST', `/service/rest/v1/components?repository=${repository}`, form),
    `upload to ${repository}`,
  );
}

export const SEARCH_REPOSITORIES = {
  raw: 'search-raw',
  docs: 'search-docs',
  maven: 'search-maven',
  npm: 'search-npm',
  pypi: 'search-pypi',
};

export const TOOLS = 12;

export async function seedSearch(server: Server): Promise<void> {
  const marker = await call(server, 'GET', `/service/rest/v1/repositories/${SEARCH_REPOSITORIES.pypi}`);
  if (marker.ok) {
    return;
  }
  await hosted(server, 'raw', SEARCH_REPOSITORIES.raw, { raw: { contentDisposition: 'ATTACHMENT' } });
  await hosted(server, 'raw', SEARCH_REPOSITORIES.docs, { raw: { contentDisposition: 'INLINE' } });
  await hosted(server, 'maven', SEARCH_REPOSITORIES.maven, {
    maven: { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT', contentDisposition: 'ATTACHMENT' },
  });
  await hosted(server, 'npm', SEARCH_REPOSITORIES.npm);
  await hosted(server, 'pypi', SEARCH_REPOSITORIES.pypi);
  for (let index = 1; index <= TOOLS; index += 1) {
    await put(
      server,
      `/repository/${SEARCH_REPOSITORIES.raw}/tools/${index}/tool-${index}.bin`,
      `tool number ${index}\n`,
    );
  }
  await put(
    server,
    `/repository/${SEARCH_REPOSITORIES.docs}/handbook/architecture.pdf`,
    '%PDF-1.4 architecture',
  );
  await put(server, `/repository/${SEARCH_REPOSITORIES.docs}/handbook/onboarding.pdf`, '%PDF-1.4 onboarding');
  const maven = `/repository/${SEARCH_REPOSITORIES.maven}`;
  for (const version of ['1.2.0', '1.3.0', '1.4.0']) {
    const base = `${maven}/com/globalart/astrum-client/${version}/astrum-client-${version}`;
    await put(server, `${base}.pom`, pom('com.globalart', 'astrum-client', version));
    await put(server, `${base}.jar`, jar(`astrum-client-${version}`));
  }
  const ledger = `${maven}/org/acme/ledger/1.8.0/ledger-1.8.0`;
  await put(server, `${ledger}.pom`, pom('org.acme', 'ledger', '1.8.0'));
  await put(server, `${ledger}.jar`, jar('ledger'));
  await put(server, `${ledger}-sources.jar`, jar('ledger-sources'));
  await npmPublish(server, SEARCH_REPOSITORIES.npm, '@globalart/ui-kit', '4.2.0');
  await npmPublish(server, SEARCH_REPOSITORIES.npm, '@globalart/ui-kit', '4.3.1');
  await npmPublish(server, SEARCH_REPOSITORIES.npm, '@globalart/icons', '7.1.0');
  await npmPublish(server, SEARCH_REPOSITORIES.npm, 'left-pad', '1.3.0');
  for (const [name, version, summary, keywords] of [
    ['astrum-tools', '1.0.3', 'Command line tools for Astrum', 'astrum cli'],
    ['astrum-tools', '1.1.0', 'Command line tools for Astrum', 'astrum cli'],
    ['report-builder', '0.4.2', 'Builds reports from data', 'reports'],
  ] as const) {
    const form = new FormData();
    form.append(
      'pypi.asset',
      new Blob([new Uint8Array(wheel(name, version, summary, keywords))]),
      wheelName(name, version),
    );
    await componentUpload(server, SEARCH_REPOSITORIES.pypi, form);
  }
}
