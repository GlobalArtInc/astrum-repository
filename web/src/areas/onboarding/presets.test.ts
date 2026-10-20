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
  it('offers only the formats and types the server knows', () => {
    expect(availablePresets(FORMATS).map((candidate) => candidate.id)).toEqual([
      'docker',
      'maven',
      'helm',
      'go',
    ]);
  });

  it('creates members before the groups that hold them', () => {
    const planned = plan([preset('maven'), preset('docker')], FORMATS, 'default');
    expect(planned.map((repository) => repository.name)).toEqual([
      'maven-releases',
      'maven-snapshots',
      'docker-hosted',
      'maven-central',
      'docker-hub',
      'maven-public',
      'docker',
    ]);
    expect(
      planned.every(
        (repository) => (repository.body.storage as { blobStoreName: string }).blobStoreName === 'default',
      ),
    ).toBe(true);
  });

  it('fills the settings each repository needs', () => {
    const planned = plan([preset('maven'), preset('docker')], FORMATS, 'blobs');
    const byName = Object.fromEntries(planned.map((repository) => [repository.name, repository]));
    expect(byName['maven-central']?.apiFormat).toBe('maven');
    expect(byName['maven-central']?.body.proxy).toMatchObject({
      remoteUrl: 'https://repo1.maven.org/maven2/',
    });
    expect(byName['maven-snapshots']?.body.maven).toMatchObject({ versionPolicy: 'SNAPSHOT' });
    expect(byName['maven-snapshots']?.body.storage).toMatchObject({ writePolicy: 'ALLOW' });
    expect(byName['maven-releases']?.body.storage).toMatchObject({ writePolicy: 'ALLOW_ONCE' });
    expect(byName['docker-hub']?.body.dockerProxy).toMatchObject({ indexType: 'HUB' });
    expect(byName.docker?.body.group).toEqual({ memberNames: ['docker-hosted', 'docker-hub'] });
    expect(byName.docker?.body.name).toBe('docker');
  });
});
