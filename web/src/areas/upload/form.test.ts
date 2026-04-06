import { describe, expect, it } from 'vitest';
import {
  assetFieldName,
  assetFileName,
  componentFieldName,
  componentGroups,
  emptyDraft,
  folderGuess,
  hasPom,
  isRequired,
  knownFieldNames,
  placeServerErrors,
  splitMavenFileName,
  suggestAssetValues,
  toFormData,
  type UploadDraft,
  validateDraft,
} from './form';
import { uploadTargets } from './model';
import { MAVEN, NPM, PYPI, RAW, TERRAFORM } from './specs.fixture';

const file = (name: string, content = 'x') => new File([content], name);

function draft(
  spec: typeof RAW,
  component: Record<string, string>,
  assets: Array<[File | null, Record<string, string>]>,
) {
  const base = emptyDraft(spec);
  return {
    component: { ...base.component, ...component },
    assets: assets.map(([chosen, values], index) => ({ key: index + 1, file: chosen, values })),
  } satisfies UploadDraft;
}

function entries(data: FormData) {
  return [...data.entries()].map(([key, value]) => [
    key,
    typeof value === 'string' ? value : `file:${value.name}`,
  ]);
}

describe('upload forms built from specs', () => {
  it('start with one empty file and grouped component fields', () => {
    const maven = emptyDraft(MAVEN);
    expect(maven.component).toEqual({
      groupId: '',
      artifactId: '',
      version: '',
      'generate-pom': 'false',
      packaging: 'jar',
    });
    expect(maven.assets).toHaveLength(1);
    expect(maven.assets[0]?.values).toEqual({ classifier: '', extension: '' });
    expect(componentGroups(MAVEN).map((group) => [group.title, group.fields.length])).toEqual([
      ['Component coordinates', 5],
    ]);
    expect(componentGroups(NPM)).toEqual([]);
  });

  it('name fields like the components API expects', () => {
    expect(componentFieldName(RAW, 'directory')).toBe('raw.directory');
    expect(assetFileName(RAW, 0)).toBe('raw.asset1');
    expect(assetFieldName(RAW, 1, 'filename')).toBe('raw.asset2.filename');
    expect(assetFileName(NPM, 0)).toBe('npm.asset');
    expect(assetFieldName(TERRAFORM, 0, 'x')).toBe('terraform.asset.x');
    expect(knownFieldNames(MAVEN, emptyDraft(MAVEN))).toEqual([
      'maven2.groupId',
      'maven2.artifactId',
      'maven2.version',
      'maven2.generate-pom',
      'maven2.packaging',
      'maven2.asset1',
      'maven2.asset1.classifier',
      'maven2.asset1.extension',
    ]);
  });

  it('suggest file names, extensions and classifiers from the chosen file', () => {
    const raw = emptyDraft(RAW);
    expect(suggestAssetValues(RAW, raw, raw.assets[0] as never, file('notes.txt'))).toEqual({
      filename: 'notes.txt',
    });
    expect(
      entries(toFormData(RAW, draft(RAW, { directory: '/d' }, [[file('notes.txt'), { filename: '' }]]))),
    ).toEqual([
      ['raw.directory', '/d'],
      ['raw.asset1.filename', 'notes.txt'],
      ['raw.asset1', 'file:notes.txt'],
    ]);
    const maven = draft(MAVEN, { artifactId: 'lib', version: '1.0' }, [
      [null, { classifier: '', extension: '' }],
    ]);
    expect(suggestAssetValues(MAVEN, maven, maven.assets[0] as never, file('lib-1.0-sources.jar'))).toEqual({
      classifier: 'sources',
      extension: 'jar',
    });
    expect(splitMavenFileName('lib-1.0.pom', 'lib', '1.0')).toEqual({ classifier: '', extension: 'pom' });
    expect(splitMavenFileName('bundle.tar.gz', '', '')).toEqual({ classifier: '', extension: 'tar.gz' });
    expect(splitMavenFileName('README', 'lib', '1.0')).toEqual({ classifier: '', extension: '' });
    const typed = draft(MAVEN, {}, [[null, { classifier: 'x', extension: 'zip' }]]);
    expect(suggestAssetValues(MAVEN, typed, typed.assets[0] as never, file('a.jar'))).toEqual({
      classifier: 'x',
      extension: 'zip',
    });
  });

  it('require fields and files the spec requires', () => {
    expect(validateDraft(RAW, emptyDraft(RAW))).toEqual({
      'raw.directory': { message: 'upload.validation.required' },
      'raw.asset1': { message: 'upload.validation.file' },
    });
    expect(validateDraft(NPM, draft(NPM, {}, [[file('pkg.tgz'), {}]]))).toEqual({});
    expect(validateDraft(TERRAFORM, draft(TERRAFORM, {}, [[file('m.zip'), {}]]))).toEqual({
      'terraform.namespace': { message: 'upload.validation.required' },
    });
  });

  it('check file types, paths and duplicates', () => {
    expect(validateDraft(NPM, draft(NPM, {}, [[file('pkg.zip'), {}]]))['npm.asset']).toEqual({
      message: 'upload.validation.fileType',
      vars: { types: '.tgz, .tar.gz' },
    });
    expect(validateDraft(PYPI, draft(PYPI, {}, [[file('x-1.0-py3-none-any.whl'), {}]]))).toEqual({});
    const raw = validateDraft(
      RAW,
      draft(RAW, { directory: '/a/../b' }, [
        [file('a'), { filename: 'same.txt' }],
        [file('b'), { filename: '/same.txt' }],
        [file('c'), { filename: 'x/./y' }],
      ]),
    );
    expect(raw['raw.directory']?.message).toBe('upload.validation.path');
    expect(raw['raw.asset2.filename']).toEqual({
      message: 'upload.validation.duplicateName',
      vars: { number: 1 },
    });
    expect(raw['raw.asset3.filename']?.message).toBe('upload.validation.path');
  });

  it('check maven coordinates and let a POM stand in for them', () => {
    const bad = validateDraft(
      MAVEN,
      draft(MAVEN, { groupId: 'org..x', artifactId: 'my lib', version: '1/0' }, [
        [file('a.jar'), { classifier: '', extension: 'jar' }],
        [file('b.jar'), { classifier: '', extension: 'jar' }],
      ]),
    );
    expect(bad['maven2.groupId']?.message).toBe('upload.validation.mavenGroup');
    expect(bad['maven2.artifactId']?.message).toBe('upload.validation.mavenPart');
    expect(bad['maven2.version']?.message).toBe('upload.validation.mavenPart');
    expect(bad['maven2.asset2.extension']).toEqual({
      message: 'upload.validation.duplicateAsset',
      vars: { number: 1 },
    });
    const withPom = draft(MAVEN, {}, [
      [file('lib.jar'), { classifier: '', extension: 'jar' }],
      [file('pom.xml'), { classifier: '', extension: 'pom' }],
    ]);
    expect(hasPom(MAVEN, withPom)).toBe(true);
    expect(isRequired(MAVEN, withPom, MAVEN.componentFields[0] as never)).toBe(false);
    expect(validateDraft(MAVEN, withPom)).toEqual({});
  });

  it('send component fields, asset fields and files', () => {
    const maven = draft(
      MAVEN,
      { groupId: ' org.x ', artifactId: 'lib', version: '1.0', 'generate-pom': 'true' },
      [
        [file('lib-1.0.jar'), { classifier: '', extension: 'jar' }],
        [file('lib-1.0-sources.jar'), { classifier: 'sources', extension: 'jar' }],
      ],
    );
    expect(entries(toFormData(MAVEN, maven))).toEqual([
      ['maven2.groupId', 'org.x'],
      ['maven2.artifactId', 'lib'],
      ['maven2.version', '1.0'],
      ['maven2.generate-pom', 'true'],
      ['maven2.packaging', 'jar'],
      ['maven2.asset1.extension', 'jar'],
      ['maven2.asset1', 'file:lib-1.0.jar'],
      ['maven2.asset2.classifier', 'sources'],
      ['maven2.asset2.extension', 'jar'],
      ['maven2.asset2', 'file:lib-1.0-sources.jar'],
    ]);
    const pom = draft(MAVEN, { 'generate-pom': 'true' }, [
      [file('p.pom'), { classifier: '', extension: 'pom' }],
    ]);
    expect(entries(toFormData(MAVEN, pom)).map(([key]) => key)).toEqual([
      'maven2.asset1.extension',
      'maven2.asset1',
    ]);
    expect(entries(toFormData(NPM, draft(NPM, {}, [[file('p.tgz'), {}]])))).toEqual([
      ['npm.asset', 'file:p.tgz'],
    ]);
  });

  it('place server errors on fields when they name one', () => {
    const names = ['raw.directory', 'raw.asset1', 'raw.asset1.filename', 'raw.asset10'];
    expect(
      placeServerErrors(names, [
        { id: 'PARAMETER raw.directory', message: 'bad directory' },
        { id: '*', message: 'raw.asset1 needs a file name' },
        { id: '*', message: 'raw.asset10: too big' },
        { id: '*', message: 'the repository is read-only' },
      ]),
    ).toEqual({
      fields: {
        'raw.directory': 'bad directory',
        'raw.asset1': 'raw.asset1 needs a file name',
        'raw.asset10': 'raw.asset10: too big',
      },
      general: ['the repository is read-only'],
    });
  });

  it('guess the folder of the upload for browse links', () => {
    expect(folderGuess(RAW, draft(RAW, { directory: '/tools/linux/' }, []))).toBe('tools/linux');
    expect(
      folderGuess(MAVEN, draft(MAVEN, { groupId: 'org.x', artifactId: 'lib', version: '1.0' }, [])),
    ).toBe('org/x/lib/1.0');
    expect(folderGuess(MAVEN, draft(MAVEN, {}, []))).toBe('');
    expect(folderGuess(NPM, draft(NPM, {}, []))).toBe('');
  });
});

describe('upload targets', () => {
  it('are hosted repositories the caller may add to whose format has a spec', () => {
    const repository = (name: string, format: string, type: string, add = true) => ({
      name,
      format,
      type: type as 'hosted',
      url: '',
      online: true,
      componentCount: 0,
      lastUploaded: null,
      permissions: { add, edit: add },
    });
    const targets = uploadTargets(
      [
        repository('z-raw', 'raw', 'hosted'),
        repository('a-maven', 'maven2', 'hosted'),
        repository('proxy', 'raw', 'proxy'),
        repository('read-only', 'raw', 'hosted', false),
        repository('docker', 'docker', 'hosted'),
      ],
      [RAW, MAVEN],
    );
    expect(targets.map((target) => target.name)).toEqual(['a-maven', 'z-raw']);
    expect(uploadTargets(undefined, [RAW])).toEqual([]);
  });
});
