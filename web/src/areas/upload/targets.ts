import { useMemo } from 'react';
import { useApi } from '../../api';
import {
  UI_REPOSITORIES,
  UPLOAD_SPECS,
  type UploadRepository,
  type UploadSpec,
  uploadTargets,
} from './model';

export function useUploadTargets() {
  const repositories = useApi<UploadRepository[]>(UI_REPOSITORIES);
  const specs = useApi<UploadSpec[]>(UPLOAD_SPECS);
  const targets = useMemo(
    () => uploadTargets(repositories.data, specs.data),
    [repositories.data, specs.data],
  );
  return {
    targets,
    specs: specs.data,
    loading: repositories.isPending || specs.isPending,
    error: repositories.error ?? specs.error,
    retry: () => {
      void repositories.refetch();
      void specs.refetch();
    },
  };
}
