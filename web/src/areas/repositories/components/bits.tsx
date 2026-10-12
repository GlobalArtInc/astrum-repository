import { formatNumber, useI18n, useT } from '../../../i18n';
import { CopyButton, StatusBadge } from '../../../ui';
import styles from '../repositories.module.css';

export function RepositoryUrl({
  name,
  url,
  compact = false,
}: {
  name: string;
  url: string;
  compact?: boolean;
}) {
  const t = useT();
  return (
    <span className={compact ? `${styles.url} ${styles.urlCompact}` : styles.url}>
      <span className={styles.urlText} title={url}>
        {url}
      </span>
      <CopyButton text={url} label={t('repositories.list.copyUrl', { name })} />
    </span>
  );
}

export function Status({ online }: { online: boolean }) {
  const t = useT();
  return (
    <StatusBadge tone={online ? 'success' : 'neutral'}>
      {online ? t('repositories.status.online') : t('repositories.status.offline')}
    </StatusBadge>
  );
}

export function Count({ value }: { value: number | null }) {
  const { language } = useI18n();
  return <>{value === null ? '—' : formatNumber(language, value)}</>;
}
