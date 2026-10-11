

import { useState, useEffect } from 'react';
import { restClient } from '../../../../interface/api';
import SharedStrings from '../../constants/SharedStrings';

export interface BlobStoreInfo {
  name: string;
  type: string;
  path?: string;
  unavailable: boolean;
  totalSizeInBytes?: number;
  availableSpaceInBytes?: number;
  blobCount?: number;
}

export interface UseBlobStoreInfoResult {
  data: BlobStoreInfo | null;
  isLoading: boolean;
  error: string | null;
}

const BLOB_STORES_INTERNAL_URL = '/service/rest/internal/ui/blobstores';

async function fetchBlobStore(blobStoreName: string): Promise<BlobStoreInfo | null> {
  try {
    const data = await restClient.get<unknown[]>(BLOB_STORES_INTERNAL_URL);
    const match = Array.isArray(data)
      ? (data as Array<Record<string, unknown>>).find((b) => b.name === blobStoreName)
      : null;
    if (!match) {
      return null;
    }
    const bs = match;
    return {
      name: String(bs.name),
      type: String(bs.typeName ?? bs.typeId ?? SharedStrings.BLOB_STORE_INFO.UNKNOWN),
      path: bs.path ? String(bs.path) : undefined,
      unavailable: Boolean(bs.unavailable),
      totalSizeInBytes: bs.totalSizeInBytes ? Number(bs.totalSizeInBytes) : undefined,
      availableSpaceInBytes: bs.availableSpaceInBytes ? Number(bs.availableSpaceInBytes) : undefined,
      blobCount: bs.blobCount ? Number(bs.blobCount) : undefined,
    };
  } catch (err) {
    console.warn('Could not fetch blob store details:', err);
    throw err;
  }
}

export function useBlobStoreInfo(
  blobStoreName: string,
  enabled: boolean = true
): UseBlobStoreInfoResult {
  const [data, setData] = useState<BlobStoreInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!enabled || !blobStoreName) {
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    fetchBlobStore(blobStoreName)
      .then((result) => {
        if (!cancelled) {
          setData(result);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          const errorMessage = err instanceof Error ? err.message : SharedStrings.BLOB_STORE_INFO.FAILED_FETCH_BLOB_STORE;
          setError(errorMessage);
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [blobStoreName, enabled]);

  return { data, isLoading, error };
}
