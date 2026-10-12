import type { Translate } from '../../../i18n';
import { useT } from '../../../i18n';
import { StatusBadge } from '../../../ui';
import { type StoreType, storeType, type UiBlobStore } from '../api';

const TYPE_LABELS: Record<StoreType, string> = {
  file: 'blobstores.types.file',
  s3: 'blobstores.types.s3',
  group: 'blobstores.types.group',
};

export function typeLabel(t: Translate, type: string): string {
  const known = storeType(type);
  return known ? t(TYPE_LABELS[known]) : type;
}

export function StateBadge({ store }: { store: UiBlobStore }) {
  const t = useT();
  return store.unavailable ? (
    <StatusBadge tone="danger">{t('blobstores.list.stateUnavailable')}</StatusBadge>
  ) : (
    <StatusBadge tone="success">{t('blobstores.list.stateAvailable')}</StatusBadge>
  );
}

export function QuotaBadge({ store }: { store: UiBlobStore }) {
  const t = useT();
  if (!store.quotaStatus) {
    return <>{t('blobstores.list.noQuota')}</>;
  }
  return (
    <span title={store.quotaStatus.message}>
      {store.quotaStatus.violation ? (
        <StatusBadge tone="warning">{t('blobstores.list.quotaViolated')}</StatusBadge>
      ) : (
        <StatusBadge tone="success">{t('blobstores.list.quotaOk')}</StatusBadge>
      )}
    </span>
  );
}

export function usedBy(t: Translate, store: UiBlobStore): string {
  const parts: string[] = [];
  if (store.repositoryCount > 0) {
    parts.push(t('blobstores.list.repositories', { count: store.repositoryCount }));
  }
  if (store.groups.length > 0) {
    parts.push(t('blobstores.list.inGroup', { groups: store.groups.join(', ') }));
  }
  return parts.length > 0 ? parts.join('; ') : t('blobstores.list.unused');
}
