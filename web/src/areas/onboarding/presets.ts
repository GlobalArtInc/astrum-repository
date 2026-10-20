import type { UiFormat } from '../../api';
import type { JsonObject } from '../repositories/model/document';
import { template } from '../repositories/model/form';
import { apiFormat } from '../repositories/model/spec';

export interface PresetRepository {
  name: string;
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
    repositories: [{ name: 'docker-hosted', tune: allowRedeploy }],
  },
  {
    id: 'maven',
    label: 'Maven',
    format: 'maven2',
    repositories: [
      { name: 'maven-releases' },
      {
        name: 'maven-snapshots',
        tune: (document) => {
          allowRedeploy(document);
          (document.maven as JsonObject).versionPolicy = 'SNAPSHOT';
        },
      },
    ],
  },
  { id: 'npm', label: 'npm', format: 'npm', repositories: [{ name: 'npm-hosted' }] },
  { id: 'pypi', label: 'PyPI', format: 'pypi', repositories: [{ name: 'pypi-hosted' }] },
  { id: 'nuget', label: 'NuGet', format: 'nuget', repositories: [{ name: 'nuget-hosted' }] },
  { id: 'helm', label: 'Helm', format: 'helm', repositories: [{ name: 'helm-hosted' }] },
  {
    id: 'raw',
    label: 'Raw',
    format: 'raw',
    repositories: [
      {
        name: 'raw-hosted',
        tune: (document) => {
          allowRedeploy(document);
          storage(document).strictContentTypeValidation = false;
        },
      },
    ],
  },
];

export function availablePresets(formats: readonly UiFormat[]): Preset[] {
  return PRESETS.filter((preset) =>
    formats.some((known) => known.id === preset.format && known.types.includes('hosted')),
  );
}

export interface PlannedRepository {
  name: string;
  format: string;
  apiFormat: string;
  body: JsonObject;
}

export function plan(
  presets: readonly Preset[],
  formats: readonly UiFormat[],
  blobStoreName: string,
): PlannedRepository[] {
  return presets.flatMap((preset) =>
    preset.repositories.map((repository) => {
      const body = template({ format: preset.format, type: 'hosted' }, blobStoreName) as JsonObject;
      body.name = repository.name;
      repository.tune?.(body);
      return {
        name: repository.name,
        format: preset.format,
        apiFormat: apiFormat(preset.format, formats),
        body,
      };
    }),
  );
}
