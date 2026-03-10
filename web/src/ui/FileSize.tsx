import { formatBytes, formatNumber, useI18n } from '../i18n';

export function FileSize({ bytes }: { bytes: number }) {
  const { language, t } = useI18n();
  return (
    <span title={t('ui.fileSize.exact', { bytes: formatNumber(language, bytes) })}>
      {formatBytes(language, bytes)}
    </span>
  );
}
