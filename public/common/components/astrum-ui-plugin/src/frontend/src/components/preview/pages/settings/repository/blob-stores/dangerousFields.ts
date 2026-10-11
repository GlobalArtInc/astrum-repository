

import type { BlobStoreFormData } from './blobStoreFormMachine';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

export interface DangerousFieldInfo {
  field: string;
  label: string;
}

const DANGEROUS_FIELDS: Record<string, DangerousFieldInfo[]> = {
  file: [
    { field: 'path', label: BlobStoresStrings.DANGEROUS_FIELDS.PATH },
  ],
  s3: [
    { field: 'bucketConfiguration.bucket.name', label: BlobStoresStrings.DANGEROUS_FIELDS.BUCKET_NAME },
    { field: 'bucketConfiguration.bucket.region', label: BlobStoresStrings.DANGEROUS_FIELDS.REGION },
    { field: 'bucketConfiguration.encryption', label: BlobStoresStrings.DANGEROUS_FIELDS.ENCRYPTION },
  ],
  google: [
    { field: 'bucketConfiguration.bucket.name', label: BlobStoresStrings.DANGEROUS_FIELDS.BUCKET_NAME },
    { field: 'bucketConfiguration.bucket.projectId', label: BlobStoresStrings.DANGEROUS_FIELDS.PROJECT_ID },
    { field: 'bucketConfiguration.bucket.region', label: BlobStoresStrings.DANGEROUS_FIELDS.REGION },
    { field: 'bucketConfiguration.encryption', label: BlobStoresStrings.DANGEROUS_FIELDS.ENCRYPTION },
  ],
  azure: [
    { field: 'bucketConfiguration.accountName', label: BlobStoresStrings.DANGEROUS_FIELDS.ACCOUNT_NAME },
    { field: 'bucketConfiguration.containerName', label: BlobStoresStrings.DANGEROUS_FIELDS.CONTAINER_NAME },
  ],
  group: [
    { field: 'members', label: BlobStoresStrings.DANGEROUS_FIELDS.MEMBERS },
  ],
};

function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
  return path.split('.').reduce<unknown>(
    (current, key) => (current != null && typeof current === 'object') ? (current as Record<string, unknown>)[key] : undefined,
    obj
  );
}

function valuesAreEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a == null && b == null) return true;
  if (a == null || b == null) return false;

  if (Array.isArray(a) && Array.isArray(b)) {
    return JSON.stringify(a) === JSON.stringify(b);
  }

  if (typeof a === 'object' && typeof b === 'object') {
    return JSON.stringify(a) === JSON.stringify(b);
  }

  return false;
}

export function hasDangerousFieldChanges(
  pristineData: BlobStoreFormData,
  currentData: BlobStoreFormData,
  type: string
): boolean {
  const fields = DANGEROUS_FIELDS[type?.toLowerCase()] || [];
  return fields.some(({ field }) => {
    const pristineValue = getNestedValue(pristineData as unknown as Record<string, unknown>, field);
    const currentValue = getNestedValue(currentData as unknown as Record<string, unknown>, field);
    return !valuesAreEqual(pristineValue, currentValue);
  });
}

export function getDangerousFieldsChanged(
  pristineData: BlobStoreFormData,
  currentData: BlobStoreFormData,
  type: string
): DangerousFieldInfo[] {
  const fields = DANGEROUS_FIELDS[type?.toLowerCase()] || [];
  return fields.filter(({ field }) => {
    const pristineValue = getNestedValue(pristineData as unknown as Record<string, unknown>, field);
    const currentValue = getNestedValue(currentData as unknown as Record<string, unknown>, field);
    return !valuesAreEqual(pristineValue, currentValue);
  });
}
