import { describe, expect, it } from 'vitest';
import type { UiFormat } from '../../api';
import { availablePresets, PRESETS, plan } from './presets';

const FORMATS: UiFormat[] = [
  { id: 'docker', apiName: 'docker', types: ['hosted', 'proxy', 'group'] },
  { id: 'maven2', apiName: 'maven', types: ['hosted', 'proxy', 'group'] },
  { id: 'go', apiName: 'go', types: ['proxy', 'group'] },
  { id: 'helm', apiName: 'helm', types: ['hosted'] },
];

function preset(id: string) {
  const found = PRESETS.find((candidate) => candidate.id === id);
  if (!found) {
    throw new Error(id);
  }
  return found;
}

describe('repository presets', () => {
  it('offers the formats that have hosted repositories on this server', () => {
    expect(availablePresets(FORMATS).map((candidate) => candidate.id)).toEqual(['docker', 'maven', 'helm']);
  });

  it('creates only hosted repositories', () => {
    const planned = plan([preset('maven'), preset('docker')], FORMATS, 'default');
    expect(planned.map((repository) => repository.name)).toEqual([
      'maven-releases',
      'maven-snapshots',
      'docker-hosted',
    ]);
    for (const repository of planned) {
      expect(repository.body.proxy).toBeUndefined();
      expect(repository.body.group).toBeUndefined();
      expect(repository.body.storage).toMatchObject({ blobStoreName: 'default' });
    }
  });

  it('fills the settings each repository needs', () => {
    const planned = plan([preset('maven'), preset('docker')], FORMATS, 'blobs');
    const byName = Object.fromEntries(planned.map((repository) => [repository.name, repository]));
    expect(byName['maven-releases']?.apiFormat).toBe('maven');
    expect(byName['maven-releases']?.body.storage).toMatchObject({ writePolicy: 'ALLOW_ONCE' });
    expect(byName['maven-snapshots']?.body.maven).toMatchObject({ versionPolicy: 'SNAPSHOT' });
    expect(byName['maven-snapshots']?.body.storage).toMatchObject({ writePolicy: 'ALLOW' });
    expect(byName['docker-hosted']?.body.storage).toMatchObject({
      writePolicy: 'ALLOW',
      blobStoreName: 'blobs',
    });
    expect(byName['docker-hosted']?.body.name).toBe('docker-hosted');
  });
});
