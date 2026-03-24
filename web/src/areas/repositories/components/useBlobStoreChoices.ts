import { INTERNAL, useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import type { BlobStoreChoice } from './RepositoryForm';

export const UI_BLOB_STORES = `${INTERNAL}/ui/blobstores`;

interface UiBlobStoreSummary {
  name: string;
  groups?: string[];
}

export function useBlobStoreChoices(): { ready: boolean; choices: BlobStoreChoice[] | null } {
  const canRead = useCan('astrum:blobstores:read');
  const query = useApi<UiBlobStoreSummary[]>(canRead ? UI_BLOB_STORES : null);
  if (!canRead || query.error) {
    return { ready: true, choices: null };
  }
  if (!query.data) {
    return { ready: false, choices: null };
  }
  return {
    ready: true,
    choices: query.data.map((store) => ({ name: store.name, groups: store.groups ?? [] })),
  };
}
