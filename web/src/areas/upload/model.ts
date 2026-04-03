export const UI_REPOSITORIES = '/service/rest/internal/ui/repositories';
export const UPLOAD_SPECS = '/service/rest/v1/formats/upload-specs';
export const COMPONENTS = '/service/rest/v1/components';

export interface UploadFieldSpec {
  name: string;
  type: string;
  description?: string | null;
  optional: boolean;
  group?: string | null;
  groupHelpText?: string | null;
  options?: string[] | Record<string, string> | null;
}

export interface UploadSpec {
  format: string;
  multipleUpload: boolean;
  componentFields: UploadFieldSpec[];
  assetFields: UploadFieldSpec[];
}

export interface UploadRepository {
  name: string;
  format: string;
  type: 'hosted' | 'proxy' | 'group';
  url: string;
  online: boolean;
  componentCount: number | null;
  lastUploaded: string | null;
  permissions: { add: boolean; edit: boolean };
}

export function uploadTargets(
  repositories: readonly UploadRepository[] | undefined,
  specs: readonly UploadSpec[] | undefined,
): UploadRepository[] {
  if (!repositories || !specs) {
    return [];
  }
  const formats = new Set(specs.map((spec) => spec.format));
  return repositories
    .filter(
      (repository) =>
        repository.type === 'hosted' && repository.permissions.add && formats.has(repository.format),
    )
    .sort((left, right) => left.name.localeCompare(right.name));
}
