import type { Translate } from '../../../i18n';
import { type StoreType, storeType } from '../api';

const TYPE_LABELS: Record<StoreType, string> = {
  file: 'blobstores.types.file',
  s3: 'blobstores.types.s3',
  remote: 'blobstores.types.remote',
  group: 'blobstores.types.group',
};

export function typeLabel(t: Translate, type: string): string {
  const known = storeType(type);
  return known ? t(TYPE_LABELS[known]) : type;
}
