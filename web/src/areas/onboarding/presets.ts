import type { RepositoryType, UiFormat } from '../../api';
import type { JsonObject } from '../repositories/model/document';
import { template } from '../repositories/model/form';
import { apiFormat } from '../repositories/model/spec';

export interface PresetRepository {
  name: string;
  type: RepositoryType;
  remoteUrl?: string;
  members?: string[];
  tune?(document: JsonObject): void;
}

export interface Preset {
  id: string;
  label: string;
  format: string;
  repositories: PresetRepository[];
}

function storage(document: JsonObject): JsonObject {
  return document.storage as JsonObject;
}

const allowRedeploy = (document: JsonObject) => {
  storage(document).writePolicy = 'ALLOW';
};

export const PRESETS: readonly Preset[] = [
  {
    id: 'docker',
    label: 'Docker',
    format: 'docker',
    repositories: [
      { name: 'docker-hosted', type: 'hosted', tune: allowRedeploy },
      {
        name: 'docker-hub',
        type: 'proxy',
        remoteUrl: 'https://registry-1.docker.io',
        tune: (document) => {
          (document.dockerProxy as JsonObject).indexType = 'HUB';
        },
      },
      { name: 'docker', type: 'group', members: ['docker-hosted', 'docker-hub'] },
    ],
  },
  {
    id: 'maven',
    label: 'Maven',
    format: 'maven2',
    repositories: [
      { name: 'maven-releases', type: 'hosted' },
      {
        name: 'maven-snapshots',
        type: 'hosted',
        tune: (document) => {
          allowRedeploy(document);
          (document.maven as JsonObject).versionPolicy = 'SNAPSHOT';
        },
      },
      { name: 'maven-central', type: 'proxy', remoteUrl: 'https://repo1.maven.org/maven2/' },
      {
        name: 'maven-public',
        type: 'group',
        members: ['maven-releases', 'maven-snapshots', 'maven-central'],
      },
    ],
  },
  {
    id: 'npm',
    label: 'npm',
    format: 'npm',
    repositories: [
      { name: 'npm-hosted', type: 'hosted' },
      { name: 'npm-proxy', type: 'proxy', remoteUrl: 'https://registry.npmjs.org' },
      { name: 'npm', type: 'group', members: ['npm-hosted', 'npm-proxy'] },
    ],
  },
  {
    id: 'pypi',
    label: 'PyPI',
    format: 'pypi',
    repositories: [
      { name: 'pypi-hosted', type: 'hosted' },
      { name: 'pypi-proxy', type: 'proxy', remoteUrl: 'https://pypi.org' },
      { name: 'pypi', type: 'group', members: ['pypi-hosted', 'pypi-proxy'] },
    ],
  },
  {
    id: 'nuget',
    label: 'NuGet',
    format: 'nuget',
    repositories: [
      { name: 'nuget-hosted', type: 'hosted' },
      { name: 'nuget.org-proxy', type: 'proxy', remoteUrl: 'https://api.nuget.org/v3/index.json' },
      { name: 'nuget-group', type: 'group', members: ['nuget-hosted', 'nuget.org-proxy'] },
    ],
  },
  {
    id: 'helm',
    label: 'Helm',
    format: 'helm',
    repositories: [{ name: 'helm-hosted', type: 'hosted' }],
  },
  {
    id: 'go',
    label: 'Go',
    format: 'go',
    repositories: [{ name: 'go-proxy', type: 'proxy', remoteUrl: 'https://proxy.golang.org' }],
  },
  {
    id: 'raw',
    label: 'Raw',
    format: 'raw',
    repositories: [
      {
        name: 'raw-hosted',
        type: 'hosted',
        tune: (document) => {
          allowRedeploy(document);
          storage(document).strictContentTypeValidation = false;
        },
      },
    ],
  },
];

export function availablePresets(formats: readonly UiFormat[]): Preset[] {
  return PRESETS.filter((preset) => {
    const format = formats.find((known) => known.id === preset.format);
    return format && preset.repositories.every((repository) => format.types.includes(repository.type));
  });
}

export interface PlannedRepository {
  name: string;
  format: string;
  apiFormat: string;
  type: RepositoryType;
  body: JsonObject;
}

const ORDER: Record<RepositoryType, number> = { hosted: 0, proxy: 1, group: 2 };

export function plan(
  presets: readonly Preset[],
  formats: readonly UiFormat[],
  blobStoreName: string,
): PlannedRepository[] {
  const planned = presets.flatMap((preset) =>
    preset.repositories.map((repository) => {
      const body = template({ format: preset.format, type: repository.type }, blobStoreName) as JsonObject;
      body.name = repository.name;
      if (repository.remoteUrl !== undefined) {
        (body.proxy as JsonObject).remoteUrl = repository.remoteUrl;
      }
      if (repository.members !== undefined) {
        (body.group as JsonObject).memberNames = repository.members;
      }
      repository.tune?.(body);
      return {
        name: repository.name,
        format: preset.format,
        apiFormat: apiFormat(preset.format, formats),
        type: repository.type,
        body,
      };
    }),
  );
  return planned.sort((left, right) => ORDER[left.type] - ORDER[right.type]);
}
