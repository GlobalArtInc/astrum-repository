

import { useState, useEffect, useCallback, useMemo } from 'react';
import { restClient, ENDPOINTS, parseApiError, isNotFoundError } from '../../../../../interface/api';

import type {
  UploadDefinitionExtended,
  RepositorySettings,
  UploadComponentField,
  UploadFieldDefinition,
  UseUploadDefinitionResult,
} from '../upload.types';
import UploadStrings from '../../../constants/pages/upload/UploadStrings';

// REST API endpoints
// Available in both self-hosted and cloud distributions (see ASTRUM-54057).
const UPLOAD_SPECS_ENDPOINT = '/service/rest/v1/formats/upload-specs';

/**
 * Groups component fields by their group property.
 */
function groupFieldsByGroup(
  fields: UploadComponentField[]
): Record<string, UploadComponentField[]> {
  return fields.reduce((acc, field) => {
    const group = field.group || 'Other';
    if (!acc[group]) {
      acc[group] = [];
    }
    acc[group].push(field);
    return acc;
  }, {} as Record<string, UploadComponentField[]>);
}

/**
 * Hook to fetch upload definition for a specific repository.
 *
 * Fetches both the repository settings and the upload definition for
 * the repository's format, providing all the field metadata needed
 * to render the upload form.
 *
 * @param repositoryName - The name of the repository to upload to
 * @returns Upload definition state and actions
 */
export function useUploadDefinition(repositoryName: string): UseUploadDefinitionResult {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploadDefinition, setUploadDefinition] = useState<UploadDefinitionExtended | null>(null);
  const [repositorySettings, setRepositorySettings] = useState<RepositorySettings | null>(null);

  /**
   * Fetch repository settings and upload definition.
   * Uses REST endpoints instead of ExtDirect.
   */
  const fetchData = useCallback(async () => {
    if (!repositoryName) {
      setError(UploadStrings.DEFINITION_HOOK.REPOSITORY_NAME_REQUIRED);
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const fetchUploadDefinitions = async (): Promise<UploadDefinitionExtended[]> => {
        const definitions = await restClient.get<UploadDefinitionExtended[]>(
          UPLOAD_SPECS_ENDPOINT
        );
        return definitions || [];
      };

      const fetchRepositories = async (): Promise<RepositorySettings[]> => {
        try {
          return await restClient.get<RepositorySettings[]>(ENDPOINTS.REPOSITORIES_DETAILS);
        } catch (err) {
          const apiError = parseApiError(err);
          if (isNotFoundError(apiError)) {
            return await restClient.get<RepositorySettings[]>(ENDPOINTS.REPOSITORIES);
          }
          throw err;
        }
      };

      const [repositories, uploadDefinitions] = await Promise.all([
        fetchRepositories(),
        fetchUploadDefinitions(),
      ]);

      // Find the repository by name
      const repoSettings = (repositories || []).find(
        (repo: RepositorySettings) => repo.name === repositoryName
      );

      if (!repoSettings) {
        throw new Error(UploadStrings.DEFINITION_HOOK.REPOSITORY_NOT_FOUND(repositoryName));
      }

      // Find the upload definition for this repository's format
      const definition = (uploadDefinitions || []).find(
        (def: UploadDefinitionExtended) => def.format === repoSettings.format
      );

      if (!definition) {
        throw new Error(
          UploadStrings.DEFINITION_HOOK.NO_UPLOAD_DEFINITION_FOUND_FORMAT(repoSettings.format)
        );
      }

      // Verify the repository supports UI upload
      // Note: the upload-specs endpoint does not return a uiUpload field,
      // so we assume formats in upload-specs support UI upload unless explicitly false
      if (definition.uiUpload === false) {
        throw new Error(UploadStrings.DEFINITION_HOOK.REPOSITORY_DOES_NOT_SUPPORT_UPLOAD(repositoryName));
      }

      // Verify it's a hosted repository
      if (repoSettings.type !== 'hosted') {
        throw new Error(UploadStrings.DEFINITION_HOOK.REPOSITORY_NOT_HOSTED_REPOSITORY(repositoryName));
      }

      setRepositorySettings(repoSettings);
      setUploadDefinition(definition);
    } catch (err) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      setUploadDefinition(null);
      setRepositorySettings(null);
    } finally {
      setLoading(false);
    }
  }, [repositoryName]);

  // Fetch data on mount or when repository name changes
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  /**
   * Computed component fields with group support.
   */
  const componentFields = useMemo((): UploadComponentField[] => {
    return (uploadDefinition?.componentFields || []) as UploadComponentField[];
  }, [uploadDefinition]);

  /**
   * Component fields grouped by their group property.
   */
  const componentFieldsByGroup = useMemo(() => {
    return groupFieldsByGroup(componentFields);
  }, [componentFields]);

  /**
   * Asset fields for file uploads.
   */
  const assetFields = useMemo((): UploadFieldDefinition[] => {
    return uploadDefinition?.assetFields || [];
  }, [uploadDefinition]);

  /**
   * Whether multiple file uploads are allowed.
   */
  const multipleUpload = useMemo(() => {
    return uploadDefinition?.multipleUpload ?? false;
  }, [uploadDefinition]);

  /**
   * Regex map for automatic field extraction from filename.
   */
  const regexMap = useMemo(() => {
    return uploadDefinition?.regexMap || null;
  }, [uploadDefinition]);

  return {
    loading,
    error,
    uploadDefinition,
    repositorySettings,
    componentFields,
    componentFieldsByGroup,
    assetFields,
    multipleUpload,
    regexMap,
    refetch: fetchData,
  };
}

export default useUploadDefinition;

